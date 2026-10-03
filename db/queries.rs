use super::connection::DatabaseConnection;
use super::models::{DbKeystrokeLog, DbPassage, DbSession, DbText, PersonalBest, User};
use crate::utils::escape_fts5_query;
use rusqlite::{params, OptionalExtension, Result};

pub struct DbQueries;

impl DbQueries {
    // ─────────────────────────────────────────────────────────────────────────
    // User Queries
    // ─────────────────────────────────────────────────────────────────────────

    pub fn create_user(
        db: &DatabaseConnection,
        username: &str,
        password_hash: &str,
    ) -> Result<User> {
        let now = chrono::Utc::now().timestamp();
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
                params![username, password_hash, now],
            )?;
            let id = conn.last_insert_rowid();
            Ok(User {
                id,
                username: username.to_string(),
                password_hash: password_hash.to_string(),
                created_at: now,
            })
        })
    }

    pub fn get_user_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<User>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, username, password_hash, created_at FROM users WHERE LOWER(username) = LOWER(?1)",
            )?;
            let user = stmt
                .query_row(params![username], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .optional()?;
            Ok(user)
        })
    }

    pub fn get_user_by_id(
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<Option<User>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, username, password_hash, created_at FROM users WHERE id = ?1",
            )?;
            let user = stmt
                .query_row(params![user_id], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .optional()?;
            Ok(user)
        })
    }

    pub fn delete_user(
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<()> {
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            tx.execute("DELETE FROM personal_bests WHERE user_id = ?1", params![user_id])?;
            tx.execute(
                "DELETE FROM keystroke_logs WHERE session_id IN (SELECT id FROM sessions WHERE user_id = ?1)",
                params![user_id],
            )?;
            tx.execute("DELETE FROM sessions WHERE user_id = ?1", params![user_id])?;
            {
                let mut fts_stmt = tx.prepare("DELETE FROM texts_fts WHERE rowid IN (SELECT id FROM texts WHERE created_by = ?1)")?;
                let _ = fts_stmt.execute(params![user_id]);
            }
            tx.execute("DELETE FROM texts WHERE created_by = ?1", params![user_id])?;
            tx.execute("DELETE FROM users WHERE id = ?1", params![user_id])?;
            tx.commit()?;
            Ok(())
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Session & Keystroke Queries
    // ─────────────────────────────────────────────────────────────────────────

    pub fn insert_session(
        db: &DatabaseConnection,
        session: &DbSession,
        keystrokes: &[DbKeystrokeLog],
    ) -> Result<i64> {
        db.with_conn(|conn| {
            let tx = conn.transaction()?;

            tx.execute(
                "INSERT INTO sessions (user_id, mode, duration, wpm, raw_wpm, accuracy, consistency, started_at, ended_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    session.user_id,
                    session.mode,
                    session.duration,
                    session.wpm,
                    session.raw_wpm,
                    session.accuracy,
                    session.consistency,
                    session.started_at,
                    session.ended_at,
                ],
            )?;

            let session_id = tx.last_insert_rowid();

            // Batch insert keystrokes
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO keystroke_logs (session_id, expected_char, actual_char, is_correct, latency_ms, position)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;

                for log in keystrokes {
                    stmt.execute(params![
                        session_id,
                        log.expected_char,
                        log.actual_char,
                        if log.is_correct { 1 } else { 0 },
                        log.latency_ms,
                        log.position,
                    ])?;
                }
            }

            tx.commit()?;
            Ok(session_id)
        })
    }

    pub fn get_recent_sessions(
        db: &DatabaseConnection,
        user_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<DbSession>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, mode, duration, wpm, raw_wpm, accuracy, consistency, started_at, ended_at
                 FROM sessions WHERE user_id IS ?1 ORDER BY ended_at DESC LIMIT ?2",
            )?;

            let rows = stmt.query_map(params![user_id, limit as i64], |row| {
                Ok(DbSession {
                    id: Some(row.get(0)?),
                    user_id: row.get(1)?,
                    mode: row.get(2)?,
                    duration: row.get(3)?,
                    wpm: row.get(4)?,
                    raw_wpm: row.get(5)?,
                    accuracy: row.get(6)?,
                    consistency: row.get(7)?,
                    started_at: row.get(8)?,
                    ended_at: row.get(9)?,
                })
            })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r?);
            }
            Ok(list)
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Personal Bests Queries
    // ─────────────────────────────────────────────────────────────────────────

    pub fn get_personal_best(
        db: &DatabaseConnection,
        user_id: i64,
        mode: &str,
        duration: i64,
    ) -> Result<Option<PersonalBest>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT user_id, mode, duration, wpm, accuracy, consistency, session_id, achieved_at
                 FROM personal_bests WHERE user_id = ?1 AND mode = ?2 AND duration = ?3",
            )?;

            let pb = stmt
                .query_row(params![user_id, mode, duration], |row| {
                    Ok(PersonalBest {
                        user_id: row.get(0)?,
                        mode: row.get(1)?,
                        duration: row.get(2)?,
                        wpm: row.get(3)?,
                        accuracy: row.get(4)?,
                        consistency: row.get(5)?,
                        session_id: row.get(6)?,
                        achieved_at: row.get(7)?,
                    })
                })
                .optional()?;

            Ok(pb)
        })
    }

    pub fn save_personal_best(
        db: &DatabaseConnection,
        pb: &PersonalBest,
    ) -> Result<()> {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO personal_bests (user_id, mode, duration, wpm, accuracy, consistency, session_id, achieved_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(user_id, mode, duration) DO UPDATE SET
                    wpm = excluded.wpm,
                    accuracy = excluded.accuracy,
                    consistency = excluded.consistency,
                    session_id = excluded.session_id,
                    achieved_at = excluded.achieved_at",
                params![
                    pb.user_id,
                    pb.mode,
                    pb.duration,
                    pb.wpm,
                    pb.accuracy,
                    pb.consistency,
                    pb.session_id,
                    pb.achieved_at,
                ],
            )?;
            Ok(())
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Texts & FTS5 Search Queries
    // ─────────────────────────────────────────────────────────────────────────

    pub fn insert_text(
        db: &DatabaseConnection,
        title: &str,
        content: &str,
        source: &str,
        created_by: Option<i64>,
    ) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        let char_count = content.chars().count() as i64;

        db.with_conn(|conn| {
            let tx = conn.transaction()?;

            tx.execute(
                "INSERT INTO texts (title, content, char_count, source, created_by, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![title, content, char_count, source, created_by, now],
            )?;

            let text_id = tx.last_insert_rowid();

            // Synchronize with FTS5 table
            tx.execute(
                "INSERT INTO texts_fts (title, content) VALUES (?1, ?2)",
                params![title, content],
            )?;

            tx.commit()?;
            Ok(text_id)
        })
    }

    pub fn search_texts(
        db: &DatabaseConnection,
        search_query: &str,
    ) -> Result<Vec<DbText>> {
        let escaped = escape_fts5_query(search_query);
        if escaped.is_empty() {
            return Self::get_recent_texts(db, 20);
        }

        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.title, t.content, t.char_count, t.source, t.created_by, t.created_at
                 FROM texts t
                 JOIN texts_fts f ON f.title = t.title AND f.content = t.content
                 WHERE texts_fts MATCH ?1
                 ORDER BY rank LIMIT 20",
            )?;

            let rows = stmt.query_map(params![escaped], |row| {
                Ok(DbText {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    char_count: row.get(3)?,
                    source: row.get(4)?,
                    created_by: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r?);
            }
            Ok(list)
        })
    }

    pub fn get_recent_texts(
        db: &DatabaseConnection,
        limit: usize,
    ) -> Result<Vec<DbText>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, char_count, source, created_by, created_at
                 FROM texts ORDER BY id DESC LIMIT ?1",
            )?;

            let rows = stmt.query_map(params![limit as i64], |row| {
                Ok(DbText {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    char_count: row.get(3)?,
                    source: row.get(4)?,
                    created_by: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r?);
            }
            Ok(list)
        })
    }

    pub fn get_random_text(db: &DatabaseConnection) -> Result<Option<DbText>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, char_count, source, created_by, created_at
                 FROM texts ORDER BY RANDOM() LIMIT 1",
            )?;

            let text = stmt
                .query_row([], |row| {
                    Ok(DbText {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        content: row.get(2)?,
                        char_count: row.get(3)?,
                        source: row.get(4)?,
                        created_by: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                })
                .optional()?;

            Ok(text)
        })
    }

    pub fn delete_text(db: &DatabaseConnection, text_id: i64) -> Result<()> {
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            let _ = tx.execute("DELETE FROM texts_fts WHERE rowid = ?1", params![text_id]);
            tx.execute("DELETE FROM texts WHERE id = ?1", params![text_id])?;
            tx.commit()?;
            Ok(())
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Dynamic Passages Queries
    // ─────────────────────────────────────────────────────────────────────────

    pub fn insert_passage(
        db: &DatabaseConnection,
        text_content: &str,
        category: &str,
        is_custom: bool,
    ) -> Result<DbPassage> {
        let now = chrono::Utc::now().timestamp();
        let word_count = text_content.split_whitespace().count() as i64;
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO passages (text_content, word_count, category, is_custom, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![text_content, word_count, category, is_custom as i64, now],
            )?;
            let id = tx.last_insert_rowid();
            let _ = tx.execute(
                "INSERT INTO passages_fts (rowid, text_content, category) VALUES (?1, ?2, ?3)",
                params![id, text_content, category],
            );
            tx.commit()?;
            Ok(DbPassage {
                id,
                text_content: text_content.to_string(),
                word_count,
                category: category.to_string(),
                is_custom,
                created_at: now,
            })
        })
    }

    pub fn get_random_passage_for_words(
        db: &DatabaseConnection,
        target_words: usize,
    ) -> Result<Option<DbPassage>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, text_content, word_count, category, is_custom, created_at
                 FROM passages
                 WHERE word_count = ?1
                 ORDER BY RANDOM() LIMIT 1",
            )?;

            let passage = stmt
                .query_row(params![target_words as i64], |row| {
                    Ok(DbPassage {
                        id: row.get(0)?,
                        text_content: row.get(1)?,
                        word_count: row.get(2)?,
                        category: row.get(3)?,
                        is_custom: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                })
                .optional()?;

            if passage.is_some() {
                return Ok(passage);
            }

            // Fallback: closest within tolerance
            let mut fallback_stmt = conn.prepare(
                "SELECT id, text_content, word_count, category, is_custom, created_at
                 FROM passages
                 ORDER BY ABS(word_count - ?1) ASC, RANDOM() LIMIT 1",
            )?;

            let fallback = fallback_stmt
                .query_row(params![target_words as i64], |row| {
                    Ok(DbPassage {
                        id: row.get(0)?,
                        text_content: row.get(1)?,
                        word_count: row.get(2)?,
                        category: row.get(3)?,
                        is_custom: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                })
                .optional()?;

            Ok(fallback)
        })
    }

    pub fn get_random_passage(
        db: &DatabaseConnection,
        category: Option<&str>,
    ) -> Result<Option<DbPassage>> {
        db.with_conn(|conn| {
            let passage = if let Some(cat) = category {
                let mut stmt = conn.prepare(
                    "SELECT id, text_content, word_count, category, is_custom, created_at
                     FROM passages
                     WHERE category = ?1
                     ORDER BY RANDOM() LIMIT 1",
                )?;
                stmt.query_row(params![cat], |row| {
                    Ok(DbPassage {
                        id: row.get(0)?,
                        text_content: row.get(1)?,
                        word_count: row.get(2)?,
                        category: row.get(3)?,
                        is_custom: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                })
                .optional()?
            } else {
                let mut stmt = conn.prepare(
                    "SELECT id, text_content, word_count, category, is_custom, created_at
                     FROM passages
                     ORDER BY RANDOM() LIMIT 1",
                )?;
                stmt.query_row([], |row| {
                    Ok(DbPassage {
                        id: row.get(0)?,
                        text_content: row.get(1)?,
                        word_count: row.get(2)?,
                        category: row.get(3)?,
                        is_custom: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                })
                .optional()?
            };

            Ok(passage)
        })
    }

    pub fn get_custom_passages(
        db: &DatabaseConnection,
        limit: usize,
    ) -> Result<Vec<DbPassage>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, text_content, word_count, category, is_custom, created_at
                 FROM passages
                 WHERE is_custom = 1
                 ORDER BY id DESC LIMIT ?1",
            )?;

            let rows = stmt.query_map(params![limit as i64], |row| {
                Ok(DbPassage {
                    id: row.get(0)?,
                    text_content: row.get(1)?,
                    word_count: row.get(2)?,
                    category: row.get(3)?,
                    is_custom: row.get::<_, i64>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r?);
            }
            Ok(list)
        })
    }

    pub fn search_passages(
        db: &DatabaseConnection,
        query: &str,
    ) -> Result<Vec<DbPassage>> {
        let escaped = escape_fts5_query(query);
        if escaped.is_empty() {
            return Self::get_custom_passages(db, 20);
        }

        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT p.id, p.text_content, p.word_count, p.category, p.is_custom, p.created_at
                 FROM passages p
                 JOIN passages_fts f ON f.rowid = p.id
                 WHERE passages_fts MATCH ?1
                 ORDER BY rank LIMIT 20",
            )?;

            let rows = stmt.query_map(params![escaped], |row| {
                Ok(DbPassage {
                    id: row.get(0)?,
                    text_content: row.get(1)?,
                    word_count: row.get(2)?,
                    category: row.get(3)?,
                    is_custom: row.get::<_, i64>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r?);
            }
            Ok(list)
        })
    }

    pub fn delete_passage(db: &DatabaseConnection, passage_id: i64) -> Result<()> {
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            let _ = tx.execute("DELETE FROM passages_fts WHERE rowid = ?1", params![passage_id]);
            tx.execute("DELETE FROM passages WHERE id = ?1", params![passage_id])?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Completely truncates user activity, sessions, passages, and re-seeds clean default prose passages
    pub fn truncate_database(db: &DatabaseConnection) -> Result<()> {
        db.with_conn(|conn| {
            let tx = conn.transaction()?;

            // Truncate text passages and library texts data only (preserves user profile and scores)
            tx.execute("DELETE FROM passages_fts", [])?;
            tx.execute("DELETE FROM passages", [])?;
            tx.execute("DELETE FROM texts_fts", [])?;
            tx.execute("DELETE FROM texts", [])?;

            // Reset autoincrement sequences for passages and texts
            let _ = tx.execute(
                "DELETE FROM sqlite_sequence WHERE name IN ('passages', 'texts')",
                [],
            );

            // Re-seed clean default texts
            let now = chrono::Utc::now().timestamp();
            for (title, content) in crate::data::SEED_PASSAGES {
                let char_count = content.chars().count() as i64;
                tx.execute(
                    "INSERT INTO texts (title, content, char_count, source, created_by, created_at) VALUES (?1, ?2, ?3, 'seed', NULL, ?4)",
                    params![title, content, char_count, now],
                )?;
                tx.execute(
                    "INSERT INTO texts_fts (title, content) VALUES (?1, ?2)",
                    params![title, content],
                )?;
            }

            // Re-seed clean curated 25 & 40 word passages
            for (category, content) in crate::db::migrations::CURATED_SEED_PASSAGES {
                let word_count = content.split_whitespace().count() as i64;
                tx.execute(
                    "INSERT INTO passages (text_content, word_count, category, is_custom, created_at) VALUES (?1, ?2, ?3, 0, ?4)",
                    params![content, word_count, category, now],
                )?;
                tx.execute(
                    "INSERT INTO passages_fts (text_content, category) VALUES (?1, ?2)",
                    params![content, category],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_db_migrations_and_queries() {
        let db = DatabaseConnection::open_in_memory().expect("open in-memory db");

        // Seed texts should have been populated
        let texts = DbQueries::get_recent_texts(&db, 10).expect("get recent texts");
        assert!(!texts.is_empty());

        // FTS5 Search
        let search_results = DbQueries::search_texts(&db, "simplicity").expect("search texts");
        assert!(!search_results.is_empty());

        // Insert new user
        let user = DbQueries::create_user(&db, "alice", "argon2hash").expect("create user");
        assert_eq!(user.username, "alice");

        // Get user by username
        let fetched = DbQueries::get_user_by_username(&db, "alice").expect("get user");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, user.id);

        // Delete text
        let text_id = texts[0].id;
        DbQueries::delete_text(&db, text_id).expect("delete text");
        let texts_after = DbQueries::get_recent_texts(&db, 10).expect("get recent texts");
        assert!(!texts_after.iter().any(|t| t.id == text_id));

        // Passages migration & seed verification
        let rand_passage = DbQueries::get_random_passage(&db, None).expect("get random passage");
        assert!(rand_passage.is_some());

        let p25 = DbQueries::get_random_passage_for_words(&db, 25).expect("get 25w passage");
        assert!(p25.is_some());
        assert_eq!(p25.unwrap().word_count, 25);

        let p40 = DbQueries::get_random_passage_for_words(&db, 40).expect("get 40w passage");
        assert!(p40.is_some());
        assert_eq!(p40.unwrap().word_count, 40);

        // Custom passage creation & search
        let custom = DbQueries::insert_passage(&db, "Test custom speed passage for validation.", "custom", true)
            .expect("insert custom passage");
        assert!(custom.is_custom);
        let custom_list = DbQueries::get_custom_passages(&db, 10).expect("get custom passages");
        assert!(custom_list.iter().any(|p| p.id == custom.id));

        // Delete user
        DbQueries::delete_user(&db, user.id).expect("delete user");
        let deleted_fetch = DbQueries::get_user_by_id(&db, user.id).expect("get deleted user");
        assert!(deleted_fetch.is_none());

        // Test Truncate Database
        DbQueries::truncate_database(&db).expect("truncate db");
        let after_p25 = DbQueries::get_random_passage_for_words(&db, 25).expect("get 25w passage after truncate");
        assert!(after_p25.is_some());
        assert_eq!(after_p25.unwrap().word_count, 25);
        let after_p40 = DbQueries::get_random_passage_for_words(&db, 40).expect("get 40w passage after truncate");
        assert!(after_p40.is_some());
        assert_eq!(after_p40.unwrap().word_count, 40);
        let custom_after_truncate = DbQueries::get_custom_passages(&db, 10).expect("custom passages after truncate");
        assert!(custom_after_truncate.is_empty());
    }
}
