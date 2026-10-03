use directories::ProjectDirs;
use std::path::PathBuf;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "velotype";
const APPLICATION: &str = "velotype";

/// Resolves the OS-specific application data directory using the directories crate.
/// Creates the directory if it does not already exist.
pub fn app_data_dir() -> PathBuf {
    let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .expect("Failed to determine OS project directories");
    let data_dir = proj_dirs.data_dir();

    if !data_dir.exists() {
        let _ = std::fs::create_dir_all(data_dir);
    }

    data_dir.to_path_buf()
}

/// Resolves the absolute path to the SQLite database file.
pub fn db_path() -> PathBuf {
    app_data_dir().join("velotype.db")
}

/// Resolves the absolute path to the JSON configuration file.
pub fn config_path() -> PathBuf {
    app_data_dir().join("config.json")
}
