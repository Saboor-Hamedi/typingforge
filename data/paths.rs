use directories::BaseDirs;
use std::path::PathBuf;

/// Resolves the application data directory: `appData/typingforge`.
/// Creates the directory if it does not already exist.
pub fn app_data_dir() -> PathBuf {
    let dir = if let Some(base) = BaseDirs::new() {
        base.data_dir().join("typingforge")
    } else if let Some(proj) = directories::ProjectDirs::from("", "", "typingforge") {
        proj.data_dir().to_path_buf()
    } else {
        std::env::temp_dir().join("typingforge")
    };

    if !dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[Paths] Failed to create {}: {}. Falling back to temp directory.", dir.display(), e);
            let temp = std::env::temp_dir().join("typingforge");
            let _ = std::fs::create_dir_all(&temp);
            return temp;
        }
    }

    dir
}

/// Resolves the absolute path to the SQLite database file: `appData/typingforge/typingforge.db`
pub fn db_path() -> PathBuf {
    let new_path = app_data_dir().join("typingforge.db");

    // Seamless migration from legacy locations if new database file doesn't exist yet
    if !new_path.exists() {
        // 1. Check legacy velotype.db in current typingforge directory
        let legacy_in_current = app_data_dir().join("velotype.db");
        if legacy_in_current.exists() {
            let _ = std::fs::copy(&legacy_in_current, &new_path);
        } else if let Some(proj) = directories::ProjectDirs::from("com", "velotype", "velotype") {
            // 2. Check old com/velotype/velotype/data/velotype.db
            let old_db = proj.data_dir().join("velotype.db");
            if old_db.exists() {
                let _ = std::fs::copy(&old_db, &new_path);
            }
        }
    }

    new_path
}

/// Resolves the absolute path to the JSON configuration file: `appData/typingforge/setting.json`
pub fn config_path() -> PathBuf {
    let new_path = app_data_dir().join("setting.json");

    // Seamless migration from legacy config.json if setting.json doesn't exist yet
    if !new_path.exists() {
        // 1. Check legacy config.json in typingforge directory
        let legacy_in_current = app_data_dir().join("config.json");
        if legacy_in_current.exists() {
            let _ = std::fs::copy(&legacy_in_current, &new_path);
        } else if let Some(proj) = directories::ProjectDirs::from("com", "velotype", "velotype") {
            // 2. Check old com/velotype/velotype/data/config.json
            let old_config = proj.data_dir().join("config.json");
            if old_config.exists() {
                let _ = std::fs::copy(&old_config, &new_path);
            }
        }
    }

    new_path
}

/// Alias for `config_path`, pointing to `appData/typingforge/setting.json`.
pub fn settings_path() -> PathBuf {
    config_path()
}
