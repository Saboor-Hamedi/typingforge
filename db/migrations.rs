use super::schema::*;
use crate::data::SEED_PASSAGES;
use rusqlite::{Connection, Result};

pub const CURATED_SEED_PASSAGES: &[(&str, &str)] = &[
    // --- Prose (25 words) ---
    (
        "prose",
        "Simplicity is the essential foundation of reliable design. When systems remain clear and concise, errors disappear, focus improves, and every action feels effortless and direct.",
    ),
    (
        "prose",
        "True rhythm emerges when your thoughts move in harmony with your hands. Keystrokes flow across the surface with steady precision, turning language into instant reality.",
    ),
    (
        "prose",
        "Quiet concentration allows the mind to navigate difficult tasks with patience. By ignoring constant distractions, we discover a deeper sense of clarity and lasting satisfaction.",
    ),
    // --- Quotes (25 words) ---
    (
        "quotes",
        "Do not go where the path may lead, go instead where there is no path and leave a trail for others to follow behind you.",
    ),
    (
        "quotes",
        "Success is not final, failure is not fatal: it is the courage to continue that counts when facing the unpredictable challenges of life and work.",
    ),
    (
        "quotes",
        "In the middle of every difficulty lies great opportunity to grow stronger, wiser, and more prepared for the journey that still awaits ahead of us.",
    ),
    // --- Prose (40 words) ---
    (
        "prose",
        "The greatest achievements are built through patient daily practice rather than sudden bursts of effort. When you commit yourself to steady progress, small improvements compound over time into remarkable mastery, giving you confidence to overcome any obstacle along the way.",
    ),
    (
        "prose",
        "Writing and typing with cadence is like playing an instrument with effortless grace. Each movement anticipates the next note, guiding thoughts across the keys without hesitation, until complex ideas resolve into clean, articulate sentences that speak directly to the reader.",
    ),
    (
        "prose",
        "Clarity of thought always precedes clarity of expression. When we take time to understand the essence of our ideas before speaking or writing, words arrange themselves naturally, creating an enduring bridge of meaning and inspiration for everyone who listens attentively.",
    ),
    // --- Quotes (40 words) ---
    (
        "quotes",
        "It is not the critic who counts; not the man who points out how the strong man stumbles, or where the doer of deeds could have done them better. The credit belongs to the man who is in the arena.",
    ),
    (
        "quotes",
        "What lies behind us and what lies before us are tiny matters compared to what lies within us. When you bring that inner power into the world, you discover extraordinary strength to shape your own future with courage and grace.",
    ),
    (
        "quotes",
        "Twenty years from now you will be more disappointed by the things that you didn't do than by the ones you did do. So throw off the bowlines, sail away from the safe harbor, and catch the trade winds today.",
    ),
];

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    // 1. Ensure schema_version table exists
    conn.execute_batch(CREATE_SCHEMA_VERSION_TABLE)?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        apply_migration_v1(conn)?;
    }

    if current_version < 2 {
        apply_migration_v2(conn)?;
    }

    Ok(())
}

fn apply_migration_v1(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute_batch(CREATE_USERS_TABLE)?;
    tx.execute_batch(CREATE_SESSIONS_TABLE)?;
    tx.execute_batch(CREATE_KEYSTROKE_LOGS_TABLE)?;
    tx.execute_batch(CREATE_TEXTS_TABLE)?;
    tx.execute_batch(CREATE_TEXTS_FTS_TABLE)?;
    tx.execute_batch(CREATE_PERSONAL_BESTS_TABLE)?;

    // Populate initial seed texts
    let now = chrono::Utc::now().timestamp();
    for (title, content) in SEED_PASSAGES {
        let char_count = content.chars().count() as i64;
        tx.execute(
            "INSERT INTO texts (title, content, char_count, source, created_by, created_at) VALUES (?1, ?2, ?3, 'seed', NULL, ?4)",
            rusqlite::params![title, content, char_count, now],
        )?;

        // Populate FTS5 table
        tx.execute(
            "INSERT INTO texts_fts (title, content) VALUES (?1, ?2)",
            rusqlite::params![title, content],
        )?;
    }

    tx.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (1, ?1)",
        [now],
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_migration_v2(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute_batch(CREATE_PASSAGES_TABLE)?;
    tx.execute_batch(CREATE_PASSAGES_FTS_TABLE)?;

    let now = chrono::Utc::now().timestamp();

    // Populate curated seed passages across prose, code, and quotes
    for (category, content) in CURATED_SEED_PASSAGES {
        let word_count = content.split_whitespace().count() as i64;
        tx.execute(
            "INSERT INTO passages (text_content, word_count, category, is_custom, created_at) VALUES (?1, ?2, ?3, 0, ?4)",
            rusqlite::params![content, word_count, category, now],
        )?;

        tx.execute(
            "INSERT INTO passages_fts (text_content, category) VALUES (?1, ?2)",
            rusqlite::params![content, category],
        )?;
    }

    // Migrate existing custom texts from texts table if any
    let custom_check: Result<Vec<(String, i64)>> = {
        let mut stmt = tx.prepare("SELECT content, created_at FROM texts WHERE source != 'seed'")?;
        let rows = stmt.query_map([], |row| {
            let content: String = row.get(0)?;
            let created_at: i64 = row.get(1)?;
            Ok((content, created_at))
        })?;
        let mut list = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                list.push(item);
            }
        }
        Ok(list)
    };

    if let Ok(custom_items) = custom_check {
        for (content, created_at) in custom_items {
            let word_count = content.split_whitespace().count() as i64;
            let _ = tx.execute(
                "INSERT INTO passages (text_content, word_count, category, is_custom, created_at) VALUES (?1, ?2, 'custom', 1, ?3)",
                rusqlite::params![content, word_count, created_at],
            );
            let _ = tx.execute(
                "INSERT INTO passages_fts (text_content, category) VALUES (?1, 'custom')",
                rusqlite::params![content],
            );
        }
    }

    tx.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (2, ?1)",
        [now],
    )?;

    tx.commit()?;
    Ok(())
}
