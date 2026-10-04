use crate::db::DatabaseConnection;
use chrono::Utc;
use rusqlite::params;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use super::typing_backup::hash_sentence;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawSentenceRecord {
    pub id: Option<i64>,
    pub text: Option<String>,
    pub content: Option<String>,
    pub text_content: Option<String>,
    pub category: Option<String>,
    pub title: Option<String>,
    pub word_count: Option<i64>,
    pub date: Option<String>,
}

struct StreamRootVisitor<'a, F> {
    on_record: &'a mut F,
    count: usize,
}

impl<'de, 'a, F: FnMut(RawSentenceRecord) -> Result<(), String>> Visitor<'de> for StreamRootVisitor<'a, F> {
    type Value = usize;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid JSON document with a 'records' array")
    }

    fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut found_records = false;

        while let Some(key) = map.next_key::<String>()? {
            if key == "records" {
                found_records = true;

                struct SeqVisitor<'b, F: FnMut(RawSentenceRecord) -> Result<(), String>> {
                    on_record: &'b mut F,
                    count: &'b mut usize,
                }

                impl<'de, 'b, F: FnMut(RawSentenceRecord) -> Result<(), String>> Visitor<'de> for SeqVisitor<'b, F> {
                    type Value = ();

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a records array")
                    }

                    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                    where
                        S: SeqAccess<'de>,
                    {
                        while let Some(record) = seq.next_element::<RawSentenceRecord>()? {
                            if let Err(e) = (self.on_record)(record) {
                                return Err(de::Error::custom(e));
                            }
                            *self.count += 1;
                        }
                        Ok(())
                    }
                }

                struct SeqSeed<'b, F: FnMut(RawSentenceRecord) -> Result<(), String>> {
                    on_record: &'b mut F,
                    count: &'b mut usize,
                }

                impl<'de, 'b, F: FnMut(RawSentenceRecord) -> Result<(), String>> DeserializeSeed<'de> for SeqSeed<'b, F> {
                    type Value = ();

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        deserializer.deserialize_seq(SeqVisitor {
                            on_record: self.on_record,
                            count: self.count,
                        })
                    }
                }

                map.next_value_seed(SeqSeed {
                    on_record: self.on_record,
                    count: &mut self.count,
                })?;
            } else {
                let _ = map.next_value::<de::IgnoredAny>()?;
            }
        }

        if !found_records {
            return Err(de::Error::custom("Invalid file format."));
        }

        Ok(self.count)
    }
}

pub struct TypingImport;

impl TypingImport {
    /// Ultra high-performance streaming import capable of processing 1,000,000+ records in batches.
    pub fn import_from_reader<R: std::io::Read>(
        db: &DatabaseConnection,
        user_id: Option<i64>,
        reader: R,
        progress: Option<Arc<AtomicUsize>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<usize, String> {
        // 1. Preload existing passage hashes in memory for O(1) duplicate checks
        let mut seen_hashes: std::collections::HashSet<u64> = std::collections::HashSet::new();
        db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT text_content FROM passages")?;
            let rows = stmt.query_map([], |r| {
                let text: String = r.get(0)?;
                Ok(hash_sentence(&text))
            })?;
            for h in rows {
                if let Ok(val) = h {
                    seen_hashes.insert(val);
                }
            }
            Ok(())
        }).map_err(|e| format!("DB error preloading sentence cache: {e}"))?;

        let mut imported_count = 0usize;
        let mut batch: Vec<(String, i64, String, i64)> = Vec::with_capacity(20_000);

        let flush_batch = |conn: &mut rusqlite::Connection, items: &mut Vec<(String, i64, String, i64)>| -> Result<(), rusqlite::Error> {
            if items.is_empty() {
                return Ok(());
            }
            let tx = conn.transaction()?;
            {
                let mut ins_passage = tx.prepare(
                    "INSERT INTO passages (text_content, word_count, category, is_custom, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
                )?;
                let mut ins_passage_fts = tx.prepare(
                    "INSERT INTO passages_fts (rowid, text_content, category) VALUES (?1, ?2, ?3)",
                )?;
                let mut ins_text = tx.prepare(
                    "INSERT INTO texts (title, content, char_count, source, created_by, created_at) VALUES (?1, ?2, ?3, 'user_paste', ?4, ?5)",
                )?;
                let mut ins_text_fts = tx.prepare(
                    "INSERT INTO texts_fts (rowid, title, content) VALUES (?1, ?2, ?3)",
                )?;

                for (text, word_count, cat, created_at) in items.drain(..) {
                    let char_count = text.chars().count() as i64;
                    ins_passage.execute(params![&text, word_count, &cat, created_at])?;
                    let pid = tx.last_insert_rowid();
                    ins_passage_fts.execute(params![pid, &text, &cat])?;
                    ins_text.execute(params![&cat, &text, char_count, user_id, created_at])?;
                    let tid = tx.last_insert_rowid();
                    ins_text_fts.execute(params![tid, &cat, &text])?;
                }
            }
            tx.commit()?;
            Ok(())
        };

        let mut de = serde_json::Deserializer::from_reader(reader);

        let mut on_record = |rec: RawSentenceRecord| -> Result<(), String> {
            if let Some(ref c) = cancel {
                if c.load(Ordering::Relaxed) {
                    return Err("Operation cancelled by user".to_string());
                }
            }

            let text_opt = rec.text.or(rec.content).or(rec.text_content);
            let sentence = match text_opt {
                Some(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => return Ok(()),
            };

            let h = hash_sentence(&sentence);
            if !seen_hashes.insert(h) {
                return Ok(());
            }

            let category = rec.category.or(rec.title).unwrap_or_else(|| "custom".to_string());
            let words = rec.word_count.unwrap_or_else(|| sentence.split_whitespace().count() as i64);
            let created_at = rec.date
                .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|| Utc::now().timestamp());

            batch.push((sentence, words, category, created_at));
            imported_count += 1;

            if batch.len() >= 20_000 {
                db.with_conn(|conn| {
                    flush_batch(conn, &mut batch)
                }).map_err(|e| format!("Batch commit error: {e}"))?;

                if let Some(ref p) = progress {
                    p.store(imported_count, Ordering::Relaxed);
                }
            }

            Ok(())
        };

        let visitor = StreamRootVisitor {
            on_record: &mut on_record,
            count: 0,
        };

        let _ = de.deserialize_map(visitor).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("Invalid file format") {
                "Invalid file format.".to_string()
            } else {
                format!("JSON parse error: {msg}")
            }
        })?;

        if !batch.is_empty() {
            db.with_conn(|conn| {
                flush_batch(conn, &mut batch)
            }).map_err(|e| format!("Final batch commit error: {e}"))?;

            if let Some(ref p) = progress {
                p.store(imported_count, Ordering::Relaxed);
            }
        }

        Ok(imported_count)
    }

    /// Streams import from file on disk with 256KB BufReader
    pub fn import_from_file(
        db: &DatabaseConnection,
        user_id: Option<i64>,
        path: &Path,
        progress: Option<Arc<AtomicUsize>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<usize, String> {
        let meta = std::fs::metadata(path).map_err(|e| format!("Could not read file metadata: {e}"))?;
        const MAX_IMPORT_SIZE: u64 = 100 * 1024 * 1024; // 100 MB safety limit
        if meta.len() > MAX_IMPORT_SIZE {
            return Err(format!("Import file exceeds maximum allowed limit ({} MB)", MAX_IMPORT_SIZE / (1024 * 1024)));
        }

        let file = File::open(path).map_err(|e| format!("Could not open file: {e}"))?;
        let reader = BufReader::with_capacity(256 * 1024, file);
        Self::import_from_reader(db, user_id, reader, progress, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_import_typing_sentences() {
        let db = DatabaseConnection::open_in_memory().expect("in-memory db");
        let valid_json = r#"{
            "exported_at": "2026-10-03T12:00:00Z",
            "records": [
                { "id": 1, "text": "Focus on smooth movement across the keys for effortless speed.", "category": "prose", "word_count": 10 },
                { "id": 2, "text": "Repetition builds muscle memory and consistent cadence over time.", "category": "quotes", "word_count": 9 }
            ]
        }"#;

        let count = TypingImport::import_from_reader(&db, None, valid_json.as_bytes(), None, None).expect("import valid");
        assert_eq!(count, 2);

        // Deduplication: re-importing the same sentences should skip them
        let count_dupe = TypingImport::import_from_reader(&db, None, valid_json.as_bytes(), None, None).expect("import dupe");
        assert_eq!(count_dupe, 0);
    }

    #[test]
    fn test_stream_import_invalid_schema() {
        let db = DatabaseConnection::open_in_memory().expect("in-memory db");
        let invalid_json = r#"{ "not_valid": true }"#;

        let res = TypingImport::import_from_reader(&db, None, invalid_json.as_bytes(), None, None);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Invalid file format.");
    }
}
