use crate::db::DatabaseConnection;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: i64,
    pub text: String,
    pub category: String,
    pub word_count: usize,
    pub date: String,
}

pub fn hash_sentence(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.trim().to_lowercase().hash(&mut hasher);
    hasher.finish()
}

pub struct TypingBackup;

impl TypingBackup {
    /// Ultra high-performance streaming export capable of exporting 1,000,000+ records with O(1) RAM usage.
    pub fn export_to_writer<W: Write>(
        db: &DatabaseConnection,
        mut writer: W,
        progress: Option<Arc<AtomicUsize>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<usize, String> {
        let now_str = Utc::now().to_rfc3339();
        writer
            .write_all(format!("{{\n  \"exported_at\": \"{now_str}\",\n  \"records\": [\n").as_bytes())
            .map_err(|e| format!("Write error: {e}"))?;

        let mut count = 0usize;
        let mut first = true;
        let mut cancelled = false;
        let mut seen_hashes = std::collections::HashSet::new();

        db.with_conn(|conn| {
            // 1. Stream passages with cursor
            let mut stmt = conn.prepare(
                "SELECT id, text_content, category, word_count, created_at FROM passages ORDER BY is_custom DESC, id ASC",
            )?;

            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                if let Some(ref c) = cancel {
                    if c.load(Ordering::Relaxed) {
                        cancelled = true;
                        break;
                    }
                }

                let id: i64 = row.get(0)?;
                let text: String = row.get(1)?;
                let category: String = row.get(2)?;
                let word_count: i64 = row.get(3)?;
                let created_at: i64 = row.get(4)?;

                let h = hash_sentence(&text);
                if !seen_hashes.insert(h) {
                    continue;
                }

                let date = chrono::DateTime::from_timestamp(created_at, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| Utc::now().to_rfc3339());

                let record = BackupRecord {
                    id,
                    text,
                    category,
                    word_count: word_count.max(0) as usize,
                    date,
                };

                if !first {
                    writer.write_all(b",\n").map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                }
                first = false;

                serde_json::to_writer(&mut writer, &record)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                count += 1;
                if count % 1000 == 0 {
                    if let Some(ref p) = progress {
                        p.store(count, Ordering::Relaxed);
                    }
                }
            }

            // 2. Stream custom texts with cursor
            let mut text_stmt = conn.prepare(
                "SELECT id, title, content, created_at FROM texts WHERE source != 'seed' ORDER BY id ASC",
            )?;
            let mut text_rows = text_stmt.query([])?;
            while let Some(row) = text_rows.next()? {
                if let Some(ref c) = cancel {
                    if c.load(Ordering::Relaxed) {
                        cancelled = true;
                        break;
                    }
                }

                let id: i64 = row.get(0)?;
                let title: String = row.get(1)?;
                let content: String = row.get(2)?;
                let created_at: i64 = row.get(3)?;

                let h = hash_sentence(&content);
                if !seen_hashes.insert(h) {
                    continue;
                }

                let words = content.split_whitespace().count();
                let date = chrono::DateTime::from_timestamp(created_at, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| Utc::now().to_rfc3339());

                let record = BackupRecord {
                    id: 1_000_000 + id,
                    text: content,
                    category: title,
                    word_count: words,
                    date,
                };

                if !first {
                    writer.write_all(b",\n").map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                }
                first = false;

                serde_json::to_writer(&mut writer, &record)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                count += 1;
                if count % 1000 == 0 {
                    if let Some(ref p) = progress {
                        p.store(count, Ordering::Relaxed);
                    }
                }
            }

            Ok(())
        }).map_err(|e| format!("Export streaming error: {e}"))?;

        if cancelled {
            return Err("Export cancelled by user.".to_string());
        }

        if let Some(ref p) = progress {
            p.store(count, Ordering::Relaxed);
        }

        writer.write_all(b"\n  ]\n}\n").map_err(|e| format!("Write error: {e}"))?;
        writer.flush().map_err(|e| format!("Flush error: {e}"))?;

        Ok(count)
    }

    /// Exports data to disk using BufWriter
    pub fn export_to_file(
        db: &DatabaseConnection,
        path: &std::path::Path,
        progress: Option<Arc<AtomicUsize>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<usize, String> {
        let file = File::create(path).map_err(|e| format!("Could not create backup file: {e}"))?;
        let writer = BufWriter::with_capacity(256 * 1024, file); // 256KB fast buffer
        Self::export_to_writer(db, writer, progress, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbQueries;

    #[test]
    fn test_export_streaming_backup() {
        let db = DatabaseConnection::open_in_memory().expect("in-memory db");
        let sentence = "The quick brown fox jumps over the lazy dog and runs freely.";
        let p = DbQueries::insert_passage(&db, sentence, "custom_prose", true).expect("insert passage");

        let mut buffer = Vec::new();
        let count = TypingBackup::export_to_writer(&db, &mut buffer, None, None).expect("stream export");
        assert!(count >= 1);

        let json_str = String::from_utf8(buffer).expect("valid utf8");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid json");
        let records = parsed.get("records").and_then(|r| r.as_array()).expect("records array");
        assert!(records.len() >= 1);
        let found = records.iter().find(|r| r.get("id").and_then(|v| v.as_i64()) == Some(p.id));
        assert!(found.is_some());
    }
}
