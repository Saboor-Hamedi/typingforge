use super::paths::config_path;
use crate::audio::SoundPreset;
use crate::fx::CaretStyle;
use crate::game::{GameMode, TimedDuration, WordCountTarget};
use crate::ui::ThemeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime application configuration state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeId,
    #[serde(default)]
    pub custom_accent: Option<String>,
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
    #[serde(default)]
    pub high_scores: HashMap<String, f32>,
    pub persist_guest_sessions: bool,
    pub active_user_id: Option<i64>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeId::Midnight,
            custom_accent: None,
            caret_style: CaretStyle::Line,
            caret_width: 3.0,
            caret_smoothness: 0.18,
            caret_glow: 0.65,
            particles_enabled: true,
            screen_shake_enabled: false,
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
            persist_guest_sessions: false,
            active_user_id: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Beautifully Structured Settings File Model for `appData/typingforge/setting.json`
// ─────────────────────────────────────────────────────────────────────────────

/// Root serialized JSON document for `appData/typingforge/setting.json`.
/// Groups application configuration cleanly by functional category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsFile {
    /// Aesthetic theme, accent color, and custom color overrides.
    pub theme: ThemeSettings,
    /// Caret kinetic physics, geometry, and bloom glow parameters.
    pub caret: CaretSettings,
    /// Typography and font scaling settings.
    pub typography: TypographySettings,
    /// Visual effects, telemetry HUD, and particle toggles.
    pub visuals: VisualSettings,
    /// Audio synthesis, volume, and switch sound presets.
    pub audio: AudioSettings,
    /// Practice session defaults and guest persistence options.
    pub gameplay: GameplaySettings,
    /// User profile and session authentication state.
    pub account: AccountSettings,
    /// Cached high scores map by mode key.
    #[serde(default)]
    pub high_scores: HashMap<String, f32>,
}

/// Theme preferences and color specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    /// Active theme identifier.
    pub id: ThemeId,
    /// Human-readable theme name.
    pub name: String,
    /// Hex-encoded accent color string (e.g., "#818cf8").
    pub accent_hex: String,
    /// Optional user-specified custom accent override hex string.
    #[serde(default)]
    pub custom_accent: Option<String>,
}

/// Caret geometry, kinetics, and bloom glow preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaretSettings {
    /// Geometry rendering mode (Line, Bar, Block, Underline).
    pub style: CaretStyle,
    /// Base thickness/width in pixels.
    pub width: f32,
    /// Spring response smoothness constant in seconds.
    pub smoothness: f32,
    /// Multiplier for layered bloom glow intensity (0.0 to 1.0).
    pub glow_intensity: f32,
}

/// Typography scale preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographySettings {
    /// Typing test font size in points.
    pub font_size: f32,
}

/// Visual feedback and HUD telemetry options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualSettings {
    /// Toggles keystroke particle burst effects.
    pub particles_enabled: bool,
    /// Toggles dynamic screen shake on errors or typing bursts.
    pub screen_shake_enabled: bool,
    /// Toggles live real-time WPM readout counter.
    pub show_live_wpm: bool,
    /// Toggles the interactive velocity telemetry graph.
    pub show_velocity_graph: bool,
}

/// Keypress sound synthesis preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master toggle for typing sound synthesis.
    pub sound_enabled: bool,
    /// Master volume level (0.0 to 1.0).
    pub sound_volume: f32,
    /// Active mechanical switch profile (Mechanical, Cream, Typewriter, etc.).
    pub sound_preset: SoundPreset,
}

/// Default practice session mode and durations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Default game mode on launch (Timed or Words).
    pub default_mode: GameMode,
    /// Target duration for timed sessions (25s or 40s).
    pub default_duration: TimedDuration,
    /// Target word count for word sessions (25w or 40w).
    pub default_word_count: WordCountTarget,
    /// Whether to record anonymous guest sessions into SQLite database history.
    pub persist_guest_sessions: bool,
}

/// User profile linkage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSettings {
    /// Currently signed-in user ID in local SQLite database.
    pub active_user_id: Option<i64>,
}

impl From<&AppConfig> for SettingsFile {
    fn from(c: &AppConfig) -> Self {
        let theme_obj = crate::ui::Theme::get(c.theme);
        let accent_hex = format!(
            "#{:02x}{:02x}{:02x}",
            theme_obj.accent.r(),
            theme_obj.accent.g(),
            theme_obj.accent.b()
        );

        SettingsFile {
            theme: ThemeSettings {
                id: c.theme,
                name: theme_obj.name.to_string(),
                accent_hex,
                custom_accent: c.custom_accent.clone(),
            },
            caret: CaretSettings {
                style: c.caret_style,
                width: c.caret_width,
                smoothness: c.caret_smoothness,
                glow_intensity: c.caret_glow,
            },
            typography: TypographySettings {
                font_size: c.font_size,
            },
            visuals: VisualSettings {
                particles_enabled: c.particles_enabled,
                screen_shake_enabled: c.screen_shake_enabled,
                show_live_wpm: c.show_live_wpm,
                show_velocity_graph: c.show_velocity_graph,
            },
            audio: AudioSettings {
                sound_enabled: c.sound_enabled,
                sound_volume: c.sound_volume,
                sound_preset: c.sound_preset,
            },
            gameplay: GameplaySettings {
                default_mode: c.default_mode,
                default_duration: c.default_duration,
                default_word_count: c.default_word_count,
                persist_guest_sessions: c.persist_guest_sessions,
            },
            account: AccountSettings {
                active_user_id: c.active_user_id,
            },
            high_scores: c.high_scores.clone(),
        }
    }
}

impl From<SettingsFile> for AppConfig {
    fn from(s: SettingsFile) -> Self {
        AppConfig {
            theme: s.theme.id,
            custom_accent: s.theme.custom_accent,
            caret_style: s.caret.style,
            caret_width: s.caret.width,
            caret_smoothness: s.caret.smoothness,
            caret_glow: s.caret.glow_intensity,
            particles_enabled: s.visuals.particles_enabled,
            screen_shake_enabled: s.visuals.screen_shake_enabled,
            sound_enabled: s.audio.sound_enabled,
            sound_volume: s.audio.sound_volume,
            sound_preset: s.audio.sound_preset,
            font_size: s.typography.font_size,
            show_live_wpm: s.visuals.show_live_wpm,
            show_velocity_graph: s.visuals.show_velocity_graph,
            default_mode: s.gameplay.default_mode,
            default_duration: s.gameplay.default_duration,
            default_word_count: s.gameplay.default_word_count,
            high_scores: s.high_scores,
            persist_guest_sessions: s.gameplay.persist_guest_sessions,
            active_user_id: s.account.active_user_id,
        }
    }
}

pub struct ConfigLoader;

impl ConfigLoader {
    /// Loads settings from `appData/typingforge/setting.json`.
    /// Handles both the structured `SettingsFile` format and legacy flat `AppConfig` format.
    pub fn load() -> AppConfig {
        let path = config_path();
        if !path.exists() {
            let default_cfg = AppConfig::default();
            Self::save(&default_cfg);
            return default_cfg;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return AppConfig::default(),
        };

        // 1. Try modern structured SettingsFile format
        if let Ok(settings_file) = serde_json::from_str::<SettingsFile>(&content) {
            return AppConfig::from(settings_file);
        }

        // 2. Fallback to legacy flat AppConfig format
        if let Ok(legacy_config) = serde_json::from_str::<AppConfig>(&content) {
            // Re-save in new structured format
            Self::save(&legacy_config);
            return legacy_config;
        }

        AppConfig::default()
    }

    /// Saves the current configuration to `appData/typingforge/setting.json` in clean, pretty JSON.
    pub fn save(config: &AppConfig) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let settings_file = SettingsFile::from(config);
        if let Ok(content) = serde_json::to_string_pretty(&settings_file) {
            let _ = std::fs::write(path, content);
        }
    }
}

/// Curated long-form seed passages (~200 characters each).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_file_roundtrip_and_serialization() {
        let mut config = AppConfig::default();
        config.theme = ThemeId::TokyoNight;
        config.caret_style = CaretStyle::Block;
        config.caret_width = 4.5;
        config.caret_smoothness = 0.22;
        config.caret_glow = 0.85;

        let settings_file = SettingsFile::from(&config);
        assert_eq!(settings_file.theme.name, "Tokyo Night");
        assert_eq!(settings_file.caret.style, CaretStyle::Block);
        assert_eq!(settings_file.caret.width, 4.5);

        let json_str = serde_json::to_string_pretty(&settings_file).expect("serialize settings");
        assert!(json_str.contains("\"caret\":"));
        assert!(json_str.contains("\"theme\":"));
        assert!(json_str.contains("\"TokyoNight\""));

        let deserialized: SettingsFile = serde_json::from_str(&json_str).expect("deserialize settings");
        let restored_config = AppConfig::from(deserialized);
        assert_eq!(restored_config.theme, ThemeId::TokyoNight);
        assert_eq!(restored_config.caret_style, CaretStyle::Block);
        assert_eq!(restored_config.caret_width, 4.5);
        assert_eq!(restored_config.caret_smoothness, 0.22);
        assert_eq!(restored_config.caret_glow, 0.85);
    }
}
