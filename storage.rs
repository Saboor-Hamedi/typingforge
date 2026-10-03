//! Storage compatibility shim delegating to `crate::data`.

pub use crate::data::{AppConfig, ConfigLoader};

pub struct StorageManager;

impl StorageManager {
    pub fn load_config() -> AppConfig {
        ConfigLoader::load()
    }

    pub fn save_config(config: &AppConfig) {
        ConfigLoader::save(config);
    }
}
