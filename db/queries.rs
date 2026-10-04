use super::connection::DatabaseConnection;
use super::models::{DbKeystrokeLog, DbPassage, DbSession, DbText, PassageId, PersonalBest, User};
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

    /// Returns aggregated statistics for a user: `(test_count, avg_wpm, max_wpm)` in a single query.
    pub fn get_user_stats_summary(
        db: &DatabaseConnection,
        user_id: Option<i64>,
    ) -> Result<(usize, f32, f32)> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT COUNT(*), COALESCE(AVG(wpm), 0.0), COALESCE(MAX(wpm), 0.0)
                 FROM sessions WHERE user_id IS ?1",
            )?;

            stmt.query_row(params![user_id], |row| {
                let count: i64 = row.get(0)?;
                let avg_wpm: f64 = row.get(1)?;
                let max_wpm: f64 = row.get(2)?;
                Ok((count as usize, avg_wpm as f32, max_wpm as f32))
            })
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

            // Synchronize with FTS5 table with explicit correlated rowid
            tx.execute(
                "INSERT INTO texts_fts (rowid, title, content) VALUES (?1, ?2, ?3)",
                params![text_id, title, content],
            )?;

            tx.commit()?;
            Ok(text_id)
        })
    }

    pub fn search_texts(
        db: &DatabaseConnection,
        search_query: &str,
    ) -> Result<Vec<DbText>> {
        let trimmed = search_query.trim();
        if trimmed.is_empty() {
            return Self::get_recent_texts(db, 20);
        }
        let escaped = escape_fts5_query(trimmed);
        if escaped.is_empty() {
            return Ok(Vec::new());
        }

        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.title, t.content, t.char_count, t.source, t.created_by, t.created_at
                 FROM texts t
                 JOIN texts_fts f ON f.rowid = t.id
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
            tx.execute(
                "INSERT INTO passages_fts (rowid, text_content, category) VALUES (?1, ?2, ?3)",
                params![id, text_content, category],
            )?;
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

    pub fn get_random_custom_passage(
        db: &DatabaseConnection,
    ) -> Result<Option<DbPassage>> {
        db.with_conn(|conn| {
            // 1. Check custom passages table (is_custom = 1)
            let mut stmt = conn.prepare(
                "SELECT id, text_content, word_count, category, is_custom, created_at
                 FROM passages
                 WHERE is_custom = 1
                 ORDER BY RANDOM() LIMIT 1",
            )?;

            let custom_passage = stmt
                .query_row([], |row| {
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

            if custom_passage.is_some() {
                return Ok(custom_passage);
            }

            // 2. Fallback: check custom texts in texts table (source != 'seed')
            let mut text_stmt = conn.prepare(
                "SELECT id, title, content, char_count, source, created_by, created_at
                 FROM texts
                 WHERE source != 'seed'
                 ORDER BY RANDOM() LIMIT 1",
            )?;

            let custom_text = text_stmt
                .query_row([], |row| {
                    let content: String = row.get(2)?;
                    let words = content.split_whitespace().count() as i64;
                    let title: String = row.get(1)?;
                    Ok(DbPassage {
                        id: row.get(0)?,
                        text_content: content,
                        word_count: words,
                        category: title,
                        is_custom: true,
                        created_at: row.get(6)?,
                    })
                })
                .optional()?;

            Ok(custom_text)
        })
    }

    pub fn get_all_passages_palette(
        db: &DatabaseConnection,
        limit: usize,
    ) -> Result<Vec<DbPassage>> {
        db.with_conn(|conn| {
            let mut list = Vec::new();
            let mut seen_texts = std::collections::HashSet::new();

            let mut stmt = conn.prepare(
                "SELECT id, text_content, word_count, category, is_custom, created_at
                 FROM passages
                 ORDER BY is_custom DESC, id DESC LIMIT ?1",
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

            for r in rows {
                if let Ok(p) = r {
                    seen_texts.insert(p.text_content.clone());
                    list.push(p);
                }
            }

            // Also include custom texts from texts table
            let mut text_stmt = conn.prepare(
                "SELECT id, title, content, char_count, source, created_by, created_at
                 FROM texts
                 ORDER BY CASE WHEN source != 'seed' THEN 0 ELSE 1 END, id DESC
                 LIMIT ?1",
            )?;

            let text_rows = text_stmt.query_map(params![limit as i64], |row| {
                let id: i64 = row.get(0)?;
                let title: String = row.get(1)?;
                let content: String = row.get(2)?;
                let source: String = row.get(4)?;
                let created_at: i64 = row.get(6)?;
                let word_count = content.split_whitespace().count() as i64;
                Ok(DbPassage {
                    id: -id,
                    text_content: content,
                    word_count,
                    category: title,
                    is_custom: source != "seed",
                    created_at,
                })
            })?;

            for r in text_rows {
                if let Ok(p) = r {
                    if !seen_texts.contains(&p.text_content) {
                        seen_texts.insert(p.text_content.clone());
                        list.push(p);
                    }
                }
            }

            Ok(list)
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
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Self::get_all_passages_palette(db, 50);
        }

        db.with_conn(|conn| {
            let escaped_like = trimmed.to_lowercase().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            let pattern = format!("%{}%", escaped_like);
            let mut list = Vec::new();
            let mut seen_texts = std::collections::HashSet::new();

            // 1. Search passages table (matching title/category OR content)
            let mut stmt = conn.prepare(
                "SELECT id, text_content, word_count, category, is_custom, created_at
                 FROM passages
                 WHERE LOWER(category) LIKE ?1 ESCAPE '\\' OR LOWER(text_content) LIKE ?1 ESCAPE '\\'
                 ORDER BY
                    CASE WHEN LOWER(category) LIKE ?1 ESCAPE '\\' THEN 0 ELSE 1 END,
                    is_custom DESC,
                    id DESC
                 LIMIT 50",
            )?;

            let rows = stmt.query_map(params![pattern], |row| {
                Ok(DbPassage {
                    id: row.get(0)?,
                    text_content: row.get(1)?,
                    word_count: row.get(2)?,
                    category: row.get(3)?,
                    is_custom: row.get::<_, i64>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?;

            for r in rows {
                if let Ok(p) = r {
                    seen_texts.insert(p.text_content.clone());
                    list.push(p);
                }
            }

            // 2. Also search texts table (where custom texts are saved)
            let mut text_stmt = conn.prepare(
                "SELECT id, title, content, char_count, source, created_by, created_at
                 FROM texts
                 WHERE LOWER(title) LIKE ?1 OR LOWER(content) LIKE ?1
                 ORDER BY
                    CASE WHEN LOWER(title) LIKE ?1 THEN 0 ELSE 1 END,
                    id DESC
                 LIMIT 50",
            )?;

            let text_rows = text_stmt.query_map(params![pattern], |row| {
                let id: i64 = row.get(0)?;
                let title: String = row.get(1)?;
                let content: String = row.get(2)?;
                let source: String = row.get(4)?;
                let created_at: i64 = row.get(6)?;
                let word_count = content.split_whitespace().count() as i64;
                Ok(DbPassage {
                    id: -id,
                    text_content: content,
                    word_count,
                    category: title,
                    is_custom: source != "seed",
                    created_at,
                })
            })?;

            for r in text_rows {
                if let Ok(p) = r {
                    if !seen_texts.contains(&p.text_content) {
                        seen_texts.insert(p.text_content.clone());
                        list.push(p);
                    }
                }
            }

            Ok(list)
        })
    }

    pub fn update_passage(
        db: &DatabaseConnection,
        passage_id: i64,
        text_content: &str,
        category: &str,
    ) -> Result<()> {
        let word_count = text_content.split_whitespace().count() as i64;
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            match PassageId::from_raw(passage_id) {
                PassageId::Passage(pid) => {
                    tx.execute(
                        "UPDATE passages SET text_content = ?1, word_count = ?2, category = ?3 WHERE id = ?4",
                        params![text_content, word_count, category, pid],
                    )?;
                    tx.execute(
                        "UPDATE passages_fts SET text_content = ?1, category = ?2 WHERE rowid = ?3",
                        params![text_content, category, pid],
                    )?;
                }
                PassageId::LegacyText(tid) => {
                    let char_count = text_content.chars().count() as i64;
                    tx.execute(
                        "UPDATE texts SET title = ?1, content = ?2, char_count = ?3 WHERE id = ?4",
                        params![category, text_content, char_count, tid],
                    )?;
                    tx.execute(
                        "UPDATE texts_fts SET title = ?1, content = ?2 WHERE rowid = ?3",
                        params![category, text_content, tid],
                    )?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    pub fn delete_passage(db: &DatabaseConnection, passage_id: i64) -> Result<()> {
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            match PassageId::from_raw(passage_id) {
                PassageId::Passage(pid) => {
                    tx.execute("DELETE FROM passages_fts WHERE rowid = ?1", params![pid])?;
                    tx.execute("DELETE FROM passages WHERE id = ?1", params![pid])?;
                }
                PassageId::LegacyText(tid) => {
                    tx.execute("DELETE FROM texts_fts WHERE rowid = ?1", params![tid])?;
                    tx.execute("DELETE FROM texts WHERE id = ?1", params![tid])?;
                }
            }
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
                let text_id = tx.last_insert_rowid();
                tx.execute(
                    "INSERT INTO texts_fts (rowid, title, content) VALUES (?1, ?2, ?3)",
                    params![text_id, title, content],
                )?;
            }

            // Re-seed clean curated 25 & 40 word passages
            for (category, content) in crate::db::migrations::CURATED_SEED_PASSAGES {
                let word_count = content.split_whitespace().count() as i64;
                tx.execute(
                    "INSERT INTO passages (text_content, word_count, category, is_custom, created_at) VALUES (?1, ?2, ?3, 0, ?4)",
                    params![content, word_count, category, now],
                )?;
                let passage_id = tx.last_insert_rowid();
                tx.execute(
                    "INSERT INTO passages_fts (rowid, text_content, category) VALUES (?1, ?2, ?3)",
                    params![passage_id, content, category],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }

    /// Bulk inserts generated or imported passages in a high-performance single SQLite transaction.
    ///
    /// For every passage:
    /// - Computes word count by splitting whitespace.
    /// - Inserts into the `passages` table.
    /// - Synchronizes the FTS5 full-text search index (`passages_fts`) using the generated `rowid`.
    /// - Commits the entire batch atomically, avoiding per-row transaction overhead.
    pub fn insert_passages_batch(
        db: &DatabaseConnection,
        passages: &[(&str, &str, bool)],
    ) -> Result<usize> {
        let now = chrono::Utc::now().timestamp();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            {
                let mut insert_stmt = tx.prepare(
                    "INSERT INTO passages (text_content, word_count, category, is_custom, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )?;
                let mut fts_stmt = tx.prepare(
                    "INSERT INTO passages_fts (rowid, text_content, category) VALUES (?1, ?2, ?3)",
                )?;

                for (text, category, is_custom) in passages {
                    let word_count = text.split_whitespace().count() as i64;
                    insert_stmt.execute(params![text, word_count, category, *is_custom as i64, now])?;
                    let id = tx.last_insert_rowid();
                    let _ = fts_stmt.execute(params![id, text, category]);
                }
            }
            tx.commit()?;
            Ok(passages.len())
        })
    }

    /// Normalizes existing database passages and texts in place.
    ///
    /// Scans both the `passages` and `texts` tables:
    /// - Replaces curly/smart quotes (“ ” „ ‟ « ») with standard ASCII `"`.
    /// - Replaces curved apostrophes and primes (‘ ’ ‚ ‛ ′) with standard ASCII `'`.
    /// - Normalizes dashes (— – ―) to hyphens (`-`).
    /// - Collapses whitespace and strips zero-width artifacts.
    /// - Updates both primary tables and their respective FTS5 full-text virtual tables in a single transaction.
    ///
    /// Returns the total number of records modified.
    pub fn normalize_existing_passages(db: &DatabaseConnection) -> Result<usize> {
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            let mut modified = 0usize;
            {
                // 1. Normalize passages table
                let mut select_stmt = tx.prepare("SELECT id, text_content, category FROM passages")?;
                let rows: Vec<(i64, String, String)> = select_stmt
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                    .filter_map(|r| r.ok())
                    .collect();

                let mut update_stmt = tx.prepare(
                    "UPDATE passages SET text_content = ?1, word_count = ?2 WHERE id = ?3",
                )?;
                let mut fts_update = tx.prepare(
                    "UPDATE passages_fts SET text_content = ?1 WHERE rowid = ?2",
                )?;

                for (id, content, _cat) in rows {
                    let normalized = crate::utils::text::sanitize_text(&content);
                    if normalized != content {
                        let word_count = normalized.split_whitespace().count() as i64;
                        update_stmt.execute(params![normalized, word_count, id])?;
                        let _ = fts_update.execute(params![normalized, id]);
                        modified += 1;
                    }
                }

                // 2. Normalize texts table
                let mut select_texts = tx.prepare("SELECT id, content FROM texts")?;
                let text_rows: Vec<(i64, String)> = select_texts
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .filter_map(|r| r.ok())
                    .collect();

                let mut update_text = tx.prepare(
                    "UPDATE texts SET content = ?1, char_count = ?2 WHERE id = ?3",
                )?;
                let mut fts_text_update = tx.prepare(
                    "UPDATE texts_fts SET content = ?1 WHERE rowid = ?2",
                )?;

                for (id, content) in text_rows {
                    let normalized = crate::utils::text::sanitize_text(&content);
                    if normalized != content {
                        let char_count = normalized.chars().count() as i64;
                        update_text.execute(params![normalized, char_count, id])?;
                        let _ = fts_text_update.execute(params![normalized, id]);
                        modified += 1;
                    }
                }
            }
            tx.commit()?;
            Ok(modified)
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
