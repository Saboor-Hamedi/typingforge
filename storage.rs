use crate::audio::SoundPreset;
use crate::fx::CaretStyle;
use crate::game::{GameMode, TimedDuration, WordCountTarget};
use crate::ui::ThemeId;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeId,
    pub caret_style: CaretStyle,
    pub caret_width: f32,
    pub caret_smoothness: f32,
    pub caret_glow: f32,
    pub particles_enabled: bool,
    pub screen_shake_enabled: bool,
    pub sound_enabled: bool,
    pub sound_volume: f32,
    pub sound_preset: SoundPreset,
    pub font_size: f32,
    pub show_live_wpm: bool,
    pub show_velocity_graph: bool,
    pub default_mode: GameMode,
    pub default_duration: TimedDuration,
    pub default_word_count: WordCountTarget,
    pub high_scores: HashMap<String, f32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeId::Midnight,
            caret_style: CaretStyle::Line,
            caret_width: 3.0,
            caret_smoothness: 0.18,
            caret_glow: 0.65,
            particles_enabled: true,
            screen_shake_enabled: true,
            sound_enabled: true,
            sound_volume: 0.5,
            sound_preset: SoundPreset::Mechanical,
            font_size: 26.0,
            show_live_wpm: true,
            show_velocity_graph: true,
            default_mode: GameMode::Timed,
            default_duration: TimedDuration::Sec25,
            default_word_count: WordCountTarget::Words25,
            high_scores: HashMap::new(),
        }
    }
}

pub struct StorageManager;

impl StorageManager {
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "Velotype", "velotype")
            .map(|proj| proj.config_dir().join("settings.json"))
    }

    pub fn load_config() -> AppConfig {
        let Some(path) = Self::config_path() else {
            return AppConfig::default();
        };

        if !path.exists() {
            return AppConfig::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) => {
                eprintln!("[Storage] Failed to read settings.json: {e}");
                AppConfig::default()
            }
        }
    }

    pub fn save_config(config: &AppConfig) {
        let Some(path) = Self::config_path() else {
            return;
        };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(content) = serde_json::to_string_pretty(config) {
            if let Err(e) = fs::write(&path, content) {
                eprintln!("[Storage] Failed to write settings.json: {e}");
            }
        }
    }
}
