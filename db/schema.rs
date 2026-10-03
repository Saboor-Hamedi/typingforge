pub const CREATE_SCHEMA_VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);
"#;

pub const CREATE_USERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
"#;

pub const CREATE_SESSIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NULL,
    mode TEXT NOT NULL,
    duration INTEGER NOT NULL,
    wpm REAL NOT NULL,
    raw_wpm REAL NOT NULL,
    accuracy REAL NOT NULL,
    consistency REAL NOT NULL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE SET NULL
);
"#;

pub const CREATE_KEYSTROKE_LOGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS keystroke_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    expected_char TEXT NOT NULL,
    actual_char TEXT NOT NULL,
    is_correct INTEGER NOT NULL,
    latency_ms INTEGER NOT NULL,
    position INTEGER NOT NULL,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);
"#;

pub const CREATE_TEXTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS texts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    char_count INTEGER NOT NULL,
    source TEXT NOT NULL,
    created_by INTEGER NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(created_by) REFERENCES users(id) ON DELETE SET NULL
);
"#;

pub const CREATE_TEXTS_FTS_TABLE: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS texts_fts USING fts5(
    title,
    content,
    tokenize = 'porter unicode61'
);
"#;

pub const CREATE_PERSONAL_BESTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS personal_bests (
    user_id INTEGER NOT NULL,
    mode TEXT NOT NULL,
    duration INTEGER NOT NULL,
    wpm REAL NOT NULL,
    accuracy REAL NOT NULL,
    consistency REAL NOT NULL,
    session_id INTEGER NOT NULL,
    achieved_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, mode, duration)
);
"#;

pub const CREATE_PASSAGES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS passages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text_content TEXT NOT NULL,
    word_count INTEGER NOT NULL,
    category TEXT NOT NULL,
    is_custom INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
"#;

pub const CREATE_PASSAGES_FTS_TABLE: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS passages_fts USING fts5(
    text_content,
    category,
    tokenize = 'porter unicode61'
);
"#;
