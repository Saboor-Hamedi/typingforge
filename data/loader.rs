use super::paths::config_path;
use crate::audio::SoundPreset;
use crate::fx::CaretStyle;
use crate::game::{GameMode, TimedDuration, WordCountTarget};
use crate::ui::ThemeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub persist_guest_sessions: bool,
    pub active_user_id: Option<i64>,
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
            screen_shake_enabled: false, // Default false as per directive
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
            persist_guest_sessions: false, // Default false as per directive
            active_user_id: None,
        }
    }
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> AppConfig {
        let path = config_path();
        if !path.exists() {
            return AppConfig::default();
        }

        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(config: &AppConfig) {
        let path = config_path();
        if let Ok(content) = serde_json::to_string_pretty(config) {
            let _ = std::fs::write(path, content);
        }
    }
}

/// Curated long-form seed passages (~200 characters each) as required by Part 5.
pub const SEED_PASSAGES: &[(&str, &str)] = &[
    (
        "Simplicity and Design",
        "Simplicity is prerequisite for reliability. Complex systems develop unexpected states and subtle bugs that frustrate users. Elegant software remains clear, concise, and focused on its purpose.",
    ),
    (
        "Crafting Fast Code",
        "The fastest code is the code that never runs. Before optimizing algorithms or adding caches, eliminate unnecessary work and redundant allocations to maintain smooth performance and responsiveness.",
    ),
    (
        "Readability First",
        "Programs must be written for people to read, and only incidentally for machines to execute. Clear variable names, modular functions, and honest comments turn complex logic into transparent intent.",
    ),
    (
        "Deep Focus",
        "Flow state emerges when challenges match your capabilities. Every keystroke lands with rhythm, every sentence unfolds naturally, and the friction between thought and machine dissolves completely.",
    ),
    (
        "The Rhythm of Typing",
        "Kinetic typing is tactile music. Each keypress provides instantaneous feedback, guiding fingers across the home row with speed and precision until the rhythm carries the mind forward effortlessly.",
    ),
    (
        "Building Resilient Systems",
        "Errors are inevitable in any real system. Resilient software embraces failure modes gracefully, providing clean recovery paths, informative logs, and decisive boundaries that protect user state.",
    ),
];
