use super::migrations::run_migrations;
use crate::data::db_path;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DatabaseConnection {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// Opens the SQLite database at the OS-specific data directory,
    /// enables WAL journal mode and foreign keys, and runs pending migrations.
    pub fn open() -> Result<Self> {
        let path = db_path();
        let mut conn = Connection::open(&path)?;

        // WAL mode for high concurrency and performance
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // Enforce foreign key constraints
        conn.pragma_update(None, "foreign_keys", "ON")?;
        // Optimal cache size (64MB)
        conn.pragma_update(None, "cache_size", -64000)?;

        run_migrations(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// In-memory database connection for unit and integration testing.
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        run_migrations(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Access the underlying connection with a mutex lock.
    pub fn with_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Connection) -> Result<R>,
    {
        let mut guard = self.conn.lock().unwrap();
        f(&mut guard)
    }
}
