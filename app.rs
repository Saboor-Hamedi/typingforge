//! Central desktop application state machine, update coordinator, and UI router.
//!
//! [`VelotypeApp`] coordinates:
//! - Subsystem integration: audio synthesis, particle physics, caret interpolation, and database queries.
//! - Screen navigation between [`AppScreen::Typing`], [`AppScreen::Results`], [`AppScreen::Settings`], and [`AppScreen::Editor`].
//! - Real-time dirty detection and automatic JSON synchronization (`setting.json`).
//! - Global keyboard accelerators (`Ctrl+P` command palette, `Ctrl+,` settings, `Tab+Enter` restart, `Escape` navigation).

use crate::audio::AudioManager;
use crate::data::{AppConfig, ConfigLoader};
use crate::db::{DatabaseConnection, DbKeystrokeLog, DbQueries, DbSession, PersonalBest, User};
use crate::fx::{CaretController, ParticleSystem, ScreenShake};
use crate::typing::{GameEngine, GameMode, GameState, SessionConfig};
use crate::ui::fuzzy::{FuzzyPalette, PaletteAction};
use crate::ui::panels::settings::setting_tab::SettingsTab;
use crate::ui::panels::{EditorPanel, ResultsView, SettingsPanel, TypingView};
use crate::ui::{HeaderScreen, HeaderWidget, Theme};
use egui::{Color32, Key, Pos2, Rect, Stroke};

/// Supported top-level views/screens in the application.
#[derive(Debug, PartialEq, Eq)]
pub enum AppScreen {
    /// Active interactive typing test view.
    Typing,
    /// Post-session benchmark scorecard and velocity time-series graph.
    Results,
    /// Comprehensive configuration and preferences panel.
    Settings,
    /// Custom text passage editor and corpus creator.
    Editor,
}

/// The root `eframe` application state for TypingForge.
pub struct VelotypeApp {
    /// Active user preferences and persistent settings.
    config: AppConfig,
    /// Core typing game engine tracking words, timing, and keystrokes.
    engine: GameEngine,
    /// Kinetic caret controller managing physics springs and glow rendering.
    caret: CaretController,
    /// Particle emitter for celebration bursts and key feedback.
    particles: ParticleSystem,
    /// Screen trauma/shake simulator for mistake feedback.
    shake: ScreenShake,
    /// Audio manager for key clicks and streak chimes.
    audio: AudioManager,
    /// Thread-safe SQLite database connection handle.
    db: DatabaseConnection,
    /// Currently authenticated local user profile, if logged in.
    current_user: Option<User>,
    /// Sub-panel for custom passage editing.
    editor_panel: EditorPanel,
    /// Sub-panel for settings tabs (Theme, Caret, Audio, Custom Texts, etc.).
    settings_panel: SettingsPanel,
    /// Global quick-action search and passage switcher modal.
    fuzzy_palette: FuzzyPalette,
    /// Active primary screen displayed to the user.
    current_screen: AppScreen,
    /// Wall-clock epoch timestamp of the previous frame for delta time calculation.
    last_frame_time: f64,
    /// Whether the most recently completed session was a new personal best.
    is_personal_best: bool,
    /// Countdown timer for the PB celebration badge display.
    pb_banner_timer: f32,
    /// Tracks window focus state across frames to prevent caret jump artifacts.
    was_focused: bool,
    /// Dropdown popup visibility for active user profile menu.
    is_profile_dropdown_open: bool,
    /// Cached copy of configuration used to detect modifications and trigger auto-saves.
    last_saved_config: AppConfig,
}

impl VelotypeApp {
    /// Constructs and initializes the application from persisted configuration and SQLite storage.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = ConfigLoader::load();
        let db = DatabaseConnection::open().unwrap_or_else(|e| {
            eprintln!("Warning: could not open local SQLite db on disk: {e}. Falling back to in-memory db.");
            DatabaseConnection::open_in_memory().expect("fallback in-memory db")
        });

        let session_config = SessionConfig {
            mode: config.default_mode,
            timed_duration: config.default_duration,
            word_target: config.default_word_count,
            include_punctuation: false,
            include_numbers: false,
        };

        let mut caret = CaretController::default();
        caret.style = config.caret_style;
        caret.base_width = config.caret_width;
        caret.smoothness = config.caret_smoothness;
        caret.glow_intensity = config.caret_glow;

        let mut particles = ParticleSystem::default();
        particles.enabled = config.particles_enabled;

        let mut shake = ScreenShake::default();
        shake.enabled = config.screen_shake_enabled;

        let mut audio = AudioManager::new();
        audio.enabled = config.sound_enabled;
        audio.volume = config.sound_volume;
        audio.sound_preset = config.sound_preset;

        let fonts = egui::FontDefinitions::default();
        cc.egui_ctx.set_fonts(fonts);

        let current_user = if let Some(user_id) = config.active_user_id {
            DbQueries::get_user_by_id(&db, user_id).ok().flatten()
        } else {
            None
        };

        let last_saved_config = config.clone();

        let mut app = Self {
            config,
            engine: GameEngine::new(session_config),
            caret,
            particles,
            shake,
            audio,
            db,
            current_user,
            editor_panel: EditorPanel::new(),
            settings_panel: SettingsPanel::new(),
            fuzzy_palette: FuzzyPalette::new(),
            current_screen: AppScreen::Typing,
            last_frame_time: 0.0,
            is_personal_best: false,
            pb_banner_timer: 0.0,
            was_focused: true,
            is_profile_dropdown_open: false,
            last_saved_config,
        };
        app.restart_game();
        app
    }

    fn restart_game(&mut self) {
        // Prioritize user's custom inserted passages from database if present
        let custom_opt = DbQueries::get_random_custom_passage(&self.db).ok().flatten();

        let (passage_text, is_custom) = if let Some(p) = custom_opt {
            (Some(p.text_content), true)
        } else {
            let text = match self.engine.config.mode {
                GameMode::Words => {
                    let target = self.engine.config.word_target as usize;
                    DbQueries::get_random_passage_for_words(&self.db, target)
                        .ok()
                        .flatten()
                        .map(|p| p.text_content)
                }
                GameMode::Timed => {
                    let target = self.engine.config.timed_duration as usize;
                    DbQueries::get_random_passage_for_words(&self.db, target)
                        .ok()
                        .flatten()
                        .map(|p| p.text_content)
                        .or_else(|| {
                            DbQueries::get_random_passage(&self.db, None)
                                .ok()
                                .flatten()
                                .map(|p| p.text_content)
                        })
                }
            };
            (text, false)
        };

        if let Some(text) = passage_text {
            self.engine.load_passage_text(&text, is_custom);
        } else {
            self.engine.reset();
        }

        self.caret.snap_to(Pos2::ZERO);
        self.current_screen = AppScreen::Typing;
        self.is_personal_best = false;
        self.pb_banner_timer = 0.0;
    }

    fn check_and_record_personal_best(&mut self) {
        let mode_str = format!("{:?}", self.engine.config.mode);
        let duration_val = match self.engine.config.mode {
            GameMode::Timed => self.engine.config.timed_duration as i64,
            GameMode::Words => self.engine.config.word_target as i64,
        };

        let user_id = self.current_user.as_ref().map(|u| u.id).unwrap_or(0);
        let net_wpm = self.engine.stats.net_wpm as f64;
        let accuracy = self.engine.stats.accuracy as f64;
        let consistency = self.engine.stats.consistency as f64;
        let now = chrono::Utc::now().timestamp();

        // 1. Personal Best tiebreakers: WPM -> accuracy -> consistency (Part 7)
        let prev_pb = DbQueries::get_personal_best(&self.db, user_id, &mode_str, duration_val).ok().flatten();

        let is_new_pb = match &prev_pb {
            None => net_wpm > 5.0,
            Some(pb) => {
                if net_wpm > pb.wpm + 0.05 {
                    true
                } else if (net_wpm - pb.wpm).abs() <= 0.05 {
                    if accuracy > pb.accuracy + 0.05 {
                        true
                    } else if (accuracy - pb.accuracy).abs() <= 0.05 {
                        consistency > pb.consistency
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };

        // 2. Persist session to SQLite (if user logged in OR guest persistence enabled, Part 3)
        let should_persist = self.current_user.is_some() || self.config.persist_guest_sessions;
        let mut saved_session_id = 0;

        if should_persist {
            let session = DbSession {
                id: None,
                user_id: self.current_user.as_ref().map(|u| u.id),
                mode: mode_str.clone(),
                duration: duration_val,
                wpm: net_wpm,
                raw_wpm: self.engine.stats.raw_wpm as f64,
                accuracy,
                consistency,
                started_at: (now as f32 - self.engine.stats.elapsed_time) as i64,
                ended_at: now,
            };

            let keystrokes: Vec<DbKeystrokeLog> = self
                .engine
                .stats
                .keystroke_log
                .iter()
                .map(|k| DbKeystrokeLog {
                    id: None,
                    session_id: 0,
                    expected_char: k.expected_char.to_string(),
                    actual_char: k.actual_char.to_string(),
                    is_correct: k.is_correct,
                    latency_ms: k.latency_ms,
                    position: k.position as i64,
                })
                .collect();

            if let Ok(id) = DbQueries::insert_session(&self.db, &session, &keystrokes) {
                saved_session_id = id;
            }
        }

        // 3. PB Celebration banner (Brief 3s, skippable, non-blocking, Part 7)
        if is_new_pb {
            self.is_personal_best = true;
            self.pb_banner_timer = 3.0;

            let pb = PersonalBest {
                user_id,
                mode: mode_str,
                duration: duration_val,
                wpm: net_wpm,
                accuracy,
                consistency,
                session_id: saved_session_id,
                achieved_at: now,
            };
            let _ = DbQueries::save_personal_best(&self.db, &pb);
        }
    }

    fn handle_keyboard_inputs(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd);
        let comma_pressed = ctx.input(|i| i.key_pressed(Key::Comma));
        let p_pressed = ctx.input(|i| i.key_pressed(Key::P));

        // Ctrl + P (or Cmd + P) opens / toggles Fuzzy Command Palette!
        if ctrl && p_pressed {
            ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::CTRL, Key::P);
                i.consume_key(egui::Modifiers::COMMAND, Key::P);
            });
            self.fuzzy_palette.toggle(&self.db);
            return;
        }

        // When Fuzzy Palette is active, suspend typing input to prevent leaking keystrokes
        if self.fuzzy_palette.is_open {
            return;
        }

        // Ctrl + , (or Cmd + ,) opens Settings
        if ctrl && comma_pressed {
            ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::CTRL, Key::Comma);
                i.consume_key(egui::Modifiers::COMMAND, Key::Comma);
            });
            if self.current_screen == AppScreen::Settings {
                ConfigLoader::save(&self.config);
                self.current_screen = AppScreen::Typing;
            } else {
                self.current_screen = AppScreen::Settings;
            }
            return;
        }

        let tab_down = ctx.input(|i| i.key_down(Key::Tab));
        let tab_pressed = ctx.input(|i| i.key_pressed(Key::Tab));
        let enter_pressed = ctx.input(|i| i.key_pressed(Key::Enter));
        let esc_pressed = ctx.input(|i| i.key_pressed(Key::Escape));

        // Any keystroke dismisses celebration banner immediately (skippable, Part 7)
        if self.pb_banner_timer > 0.0 && (tab_pressed || enter_pressed || esc_pressed || ctx.input(|i| !i.events.is_empty())) {
            self.pb_banner_timer = 0.0;
        }

        // Restart with fresh text: Tab or Tab + Enter on Typing or Results screen ONLY!
        let can_restart_with_tab = self.current_screen == AppScreen::Typing || self.current_screen == AppScreen::Results;
        if can_restart_with_tab && (tab_pressed || (tab_down && enter_pressed) || (self.current_screen == AppScreen::Results && (enter_pressed || ctx.input(|i| i.key_pressed(Key::Space))))) {
            ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, Key::Tab);
                i.consume_key(egui::Modifiers::NONE, Key::Enter);
            });
            self.restart_game();
            return;
        }

        if esc_pressed {
            if self.current_screen == AppScreen::Settings || self.current_screen == AppScreen::Editor {
                ConfigLoader::save(&self.config);
                self.current_screen = AppScreen::Typing;
            } else if self.current_screen == AppScreen::Results {
                self.restart_game();
            } else {
                self.current_screen = AppScreen::Settings;
            }
            return;
        }

        if self.current_screen != AppScreen::Typing {
            return;
        }

        let ctrl = ctx.input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd);

        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Key {
                        key: Key::Backspace,
                        pressed: true,
                        ..
                    } => {
                        self.engine.handle_backspace(ctrl);
                        self.audio.play_click();
                    }
                    egui::Event::Text(text) => {
                        for c in text.chars() {
                            if c.is_control() {
                                continue;
                            }
                            let is_correct = self.engine.handle_char(c);
                            self.audio.play_click();
                            self.caret.is_typing = true;
                            if is_correct && self.config.particles_enabled {
                                self.particles.emit(
                                    self.caret.current_pos,
                                    Color32::from_rgb(56, 189, 248),
                                    5,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        if let Some(milestone) = self.engine.streak_milestone_hit.take() {
            self.audio.play_milestone(milestone);
        }

        if self.engine.state == GameState::Completed && self.current_screen == AppScreen::Typing {
            self.current_screen = AppScreen::Results;
            self.check_and_record_personal_best();
        }
    }
}

impl eframe::App for VelotypeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_time = ctx.input(|i| i.time);
        let dt = if self.last_frame_time == 0.0 {
            0.016
        } else {
            (current_time - self.last_frame_time) as f32
        }
        .clamp(0.001, 0.1);
        self.last_frame_time = current_time;

        if self.pb_banner_timer > 0.0 {
            self.pb_banner_timer = (self.pb_banner_timer - dt).max(0.0);
        }

        self.handle_keyboard_inputs(ctx);

        let is_focused = ctx.input(|i| i.raw.focused);
        if is_focused && !self.was_focused {
            // Window focus regained (e.g. returning from Alt-Tab):
            // Instantly snap caret to target position to prevent disappearance or physics explosion
            self.caret.snap_to(self.caret.target_pos);
        }
        self.was_focused = is_focused;

        self.engine.update(dt);
        self.caret.update(dt);
        self.particles.update(dt);
        self.shake.update(dt);

        self.caret.style = self.config.caret_style;
        self.caret.base_width = self.config.caret_width;
        self.caret.smoothness = self.config.caret_smoothness;
        self.caret.glow_intensity = self.config.caret_glow;
        self.particles.enabled = self.config.particles_enabled;
        self.shake.enabled = self.config.screen_shake_enabled;
        self.audio.enabled = self.config.sound_enabled;
        self.audio.volume = self.config.sound_volume;
        self.audio.sound_preset = self.config.sound_preset;

        // Automatically persist settings immediately whenever any configuration value is modified
        if self.config != self.last_saved_config {
            ConfigLoader::save(&self.config);
            self.last_saved_config = self.config.clone();
        }

        let theme = Theme::get(self.config.theme);
        let shake_offset = self.shake.current_offset();

        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(theme.text_active);
        visuals.panel_fill = theme.bg;
        visuals.window_fill = theme.bg;

        // Eliminate white borders on hover - use subtle fade background only!
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.bg_fill = Color32::from_white_alpha(16);
        visuals.widgets.hovered.rounding = egui::Rounding::same(5.0);

        visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.active.bg_fill = Color32::from_white_alpha(26);
        visuals.widgets.active.rounding = egui::Rounding::same(5.0);

        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, theme.border);
        visuals.widgets.inactive.rounding = egui::Rounding::same(5.0);

        visuals.selection.stroke = egui::Stroke::NONE;
        visuals.selection.bg_fill = theme.accent.linear_multiply(0.25);
        ctx.set_visuals(visuals);

        // Borderless outer frame with exact zero margin
        let frame = egui::Frame::none()
            .fill(theme.bg)
            .rounding(14.0)
            .stroke(egui::Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::ZERO);

        let is_typing_active = self.current_screen == AppScreen::Typing && self.engine.state == GameState::Running;
        let focus_factor = ctx.animate_bool_responsive(egui::Id::new("zen_focus_mode"), is_typing_active);
        let chrome_alpha = egui::lerp(1.0..=0.28, focus_factor);

        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let win_rect = ui.max_rect();

            // Borderless desktop window edge & corner interactive resizing
            if !is_maximized {
                let border_thick = 8.0_f32;
                let corner_size = 18.0_f32;
                if let Some(pos) = ctx.pointer_latest_pos() {
                    let on_left = pos.x >= win_rect.min.x && pos.x <= win_rect.min.x + border_thick;
                    let on_right = pos.x <= win_rect.max.x && pos.x >= win_rect.max.x - border_thick;
                    let on_top = pos.y >= win_rect.min.y && pos.y <= win_rect.min.y + border_thick;
                    let on_bottom = pos.y <= win_rect.max.y && pos.y >= win_rect.max.y - border_thick;

                    let in_corner_tl = pos.x <= win_rect.min.x + corner_size && pos.y <= win_rect.min.y + corner_size;
                    let in_corner_tr = pos.x >= win_rect.max.x - corner_size && pos.y <= win_rect.min.y + corner_size;
                    let in_corner_bl = pos.x <= win_rect.min.x + corner_size && pos.y >= win_rect.max.y - corner_size;
                    let in_corner_br = pos.x >= win_rect.max.x - corner_size && pos.y >= win_rect.max.y - corner_size;

                    let resize_action = if in_corner_br || (on_right && on_bottom) {
                        Some((egui::ResizeDirection::SouthEast, egui::CursorIcon::ResizeSouthEast))
                    } else if in_corner_bl || (on_left && on_bottom) {
                        Some((egui::ResizeDirection::SouthWest, egui::CursorIcon::ResizeSouthWest))
                    } else if in_corner_tr || (on_right && on_top) {
                        Some((egui::ResizeDirection::NorthEast, egui::CursorIcon::ResizeNorthEast))
                    } else if in_corner_tl || (on_left && on_top) {
                        Some((egui::ResizeDirection::NorthWest, egui::CursorIcon::ResizeNorthWest))
                    } else if on_left {
                        Some((egui::ResizeDirection::West, egui::CursorIcon::ResizeWest))
                    } else if on_right {
                        Some((egui::ResizeDirection::East, egui::CursorIcon::ResizeEast))
                    } else if on_top {
                        Some((egui::ResizeDirection::North, egui::CursorIcon::ResizeNorth))
                    } else if on_bottom {
                        Some((egui::ResizeDirection::South, egui::CursorIcon::ResizeSouth))
                    } else {
                        None
                    };

                    if let Some((dir, cursor)) = resize_action {
                        ctx.set_cursor_icon(cursor);
                        if ctx.input(|i| i.pointer.any_pressed()) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
                        }
                    }
                }
            }

            let shaken_rect = ui.available_rect_before_wrap().translate(shake_offset);
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(shaken_rect), |ui| {
                // 1. Unified Standard Window Header (Identical across all screens)
                let header_screen = match self.current_screen {
                    AppScreen::Typing => HeaderScreen::Typing,
                    AppScreen::Results => HeaderScreen::Results,
                    AppScreen::Settings => HeaderScreen::Settings,
                    AppScreen::Editor => HeaderScreen::Editor,
                };

                let mut mode_changed = false;
                let mut open_settings = false;
                let mut close_settings = false;
                let mut play_again = false;
                let mut sign_out = false;

                HeaderWidget::draw(
                    ui,
                    ctx,
                    header_screen,
                    &mut self.engine.config.mode,
                    &mut self.engine.config.timed_duration,
                    &mut self.engine.config.word_target,
                    &mut mode_changed,
                    &mut open_settings,
                    &mut close_settings,
                    &mut play_again,
                    self.current_user.as_ref(),
                    &mut self.is_profile_dropdown_open,
                    &mut sign_out,
                    &self.settings_panel.updater,
                    &theme,
                    chrome_alpha,
                );

                if sign_out {
                    self.current_user = None;
                    self.config.active_user_id = None;
                    ConfigLoader::save(&self.config);
                    self.restart_game();
                }

                if mode_changed && self.engine.state != GameState::Running {
                    self.config.default_mode = self.engine.config.mode;
                    self.config.default_duration = self.engine.config.timed_duration;
                    self.config.default_word_count = self.engine.config.word_target;
                    ConfigLoader::save(&self.config);
                    self.restart_game();
                }
                if open_settings {
                    self.current_screen = AppScreen::Settings;
                }
                if close_settings {
                    ConfigLoader::save(&self.config);
                    self.current_screen = AppScreen::Typing;
                }
                if play_again {
                    self.restart_game();
                }

                // 2. Exact Content Rectangle with Reserved Header & Footer
                let header_bottom = shaken_rect.min.y + HeaderWidget::HEIGHT;
                let footer_height = 36.0;
                let footer_rect = Rect::from_min_max(
                    Pos2::new(shaken_rect.min.x, shaken_rect.max.y - footer_height),
                    shaken_rect.max,
                );
                let content_rect = Rect::from_min_max(
                    Pos2::new(shaken_rect.min.x, header_bottom),
                    Pos2::new(shaken_rect.max.x, footer_rect.min.y),
                );

                let content_frame = egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(48.0, 0.0));

                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                    content_frame.show(ui, |ui| {
                        match self.current_screen {
                            AppScreen::Typing => {
                                TypingView::show(
                                    ui,
                                    &self.engine.words,
                                    self.engine.current_word,
                                    self.engine.current_char,
                                    &mut self.caret,
                                    &self.particles,
                                    &theme,
                                    &self.config,
                                    self.engine.config.mode,
                                    self.engine.remaining_time,
                                    self.engine.streak,
                                    self.engine.state,
                                    &self.engine.live_metrics,
                                    &self.engine.stats.velocity_history,
                                    self.engine.live_metrics.internal_instant_wpm,
                                    self.engine.live_metrics.internal_net_wpm,
                                );
                            }
                            AppScreen::Results => {
                                let mut restart = false;
                                let mut dismiss_pb = false;
                                ResultsView::show(
                                    ui,
                                    &self.engine.stats,
                                    &theme,
                                    self.is_personal_best,
                                    self.pb_banner_timer,
                                    &mut restart,
                                    &mut dismiss_pb,
                                );
                                if dismiss_pb {
                                    self.pb_banner_timer = 0.0;
                                }
                                if restart {
                                    self.restart_game();
                                }
                            }
                            AppScreen::Settings => {
                                let mut is_open = true;
                                let mut passage_to_load = None;
                                self.settings_panel.show(
                                    ui,
                                    &mut self.config,
                                    &theme,
                                    &self.audio,
                                    &self.db,
                                    &mut self.current_user,
                                    &mut passage_to_load,
                                    &mut is_open,
                                );
                                let active_id = self.current_user.as_ref().map(|u| u.id);
                                if self.config.active_user_id != active_id {
                                    self.config.active_user_id = active_id;
                                    ConfigLoader::save(&self.config);
                                }
                                if let Some(req) = passage_to_load {
                                    self.config.default_mode = req.mode;
                                    if let Some(wt) = req.word_target {
                                        self.config.default_word_count = wt;
                                    }
                                    if let Some(td) = req.duration {
                                        self.config.default_duration = td;
                                    }
                                    self.engine.load_passage_with_mode(
                                        &req.text,
                                        req.mode,
                                        req.word_target,
                                        req.duration,
                                    );
                                    self.caret.snap_to(Pos2::ZERO);
                                    self.current_screen = AppScreen::Typing;
                                } else if !is_open {
                                    ConfigLoader::save(&self.config);
                                    self.current_screen = AppScreen::Typing;
                                }
                            }
                            AppScreen::Editor => {
                                let mut applied_passage = None;
                                let mut close_editor = false;
                                self.editor_panel.show(
                                    ui,
                                    &theme,
                                    &self.db,
                                    self.current_user.as_ref().map(|u| u.id),
                                    &mut applied_passage,
                                    &mut close_editor,
                                    dt,
                                );
                                if let Some(passage) = applied_passage {
                                    self.engine.load_passage(&passage);
                                    self.current_screen = AppScreen::Typing;
                                } else if close_editor {
                                    self.current_screen = AppScreen::Typing;
                                }
                            }
                        }
                    });
                });

                // 3. Unified Standardized Status Bar across All Screens
                ui.scope(|ui| {
                    ui.set_opacity(chrome_alpha);
                    let fp = ui.painter_at(footer_rect);
                    let footer_margin = 48.0;

                    // Clean 1px divider aligned with content margins
                    fp.line_segment(
                        [Pos2::new(footer_rect.min.x + footer_margin, footer_rect.min.y), Pos2::new(footer_rect.max.x - footer_margin, footer_rect.min.y)],
                        Stroke::new(1.0, theme.border),
                    );

                    let footer_y = footer_rect.center().y + 1.0;

                    // Left status telemetry
                    let left_text = match self.current_screen {
                        AppScreen::Typing => format!(
                            "net: {} wpm  ·  acc: {}  ·  streak: {}  ·  audio: {}",
                            self.engine.live_metrics.format_wpm(),
                            self.engine.live_metrics.format_accuracy(),
                            self.engine.streak,
                            self.config.sound_preset.display_name()
                        ),
                        AppScreen::Results => format!(
                            "net: {:.0} wpm  ·  acc: {:.1}%  ·  raw: {:.0} wpm  ·  streak: {}  ·  time: {:.1}s",
                            self.engine.stats.net_wpm,
                            self.engine.stats.accuracy,
                            self.engine.stats.raw_wpm,
                            self.engine.stats.max_streak,
                            self.engine.stats.elapsed_time
                        ),
                        AppScreen::Settings => "preferences & customization  ·  changes save automatically".to_string(),
                        AppScreen::Editor => "passage editor  ·  press esc to close without applying".to_string(),
                    };

                    fp.text(
                        Pos2::new(footer_rect.min.x + footer_margin, footer_y),
                        egui::Align2::LEFT_CENTER,
                        left_text,
                        egui::FontId::monospace(10.5),
                        theme.text_dim,
                    );

                    // Right status telemetry
                    let right_text = match self.current_screen {
                        AppScreen::Settings => format!("theme: {}  ·  esc return", theme.name),
                        AppScreen::Editor => "esc return  ·  enter apply".to_string(),
                        AppScreen::Results => format!("theme: {}  ·  tab+enter play again", theme.name),
                        AppScreen::Typing => format!("theme: {}  ·  ctrl+p palette  ·  tab restart", theme.name),
                    };

                    fp.text(
                        Pos2::new(footer_rect.max.x - footer_margin, footer_y),
                        egui::Align2::RIGHT_CENTER,
                        right_text,
                        egui::FontId::monospace(10.5),
                        theme.text_dim,
                    );

                    // Sleek corner resize grip indicator when not maximized
                    if !is_maximized {
                        let br = Pos2::new(footer_rect.max.x - 7.0, footer_rect.max.y - 7.0);
                        for i in 0..3 {
                            let off = (i as f32) * 4.0;
                            fp.line_segment(
                                [Pos2::new(br.x - off, br.y), Pos2::new(br.x, br.y - off)],
                                Stroke::new(1.0, theme.border),
                            );
                        }
                    }
                });
            });
        });

        // 4. macOS Spotlight-style Fuzzy Command Palette Modal
        if let Some(action) = self.fuzzy_palette.show(ctx, &self.db, &theme) {
            match action {
                PaletteAction::SelectPassage(passage) => {
                    self.engine.load_passage_text(&passage.text_content, passage.is_custom);
                    self.caret.snap_to(Pos2::ZERO);
                    self.current_screen = AppScreen::Typing;
                    self.is_personal_best = false;
                    self.pb_banner_timer = 0.0;
                }
                PaletteAction::EditPassage(passage) => {
                    self.current_screen = AppScreen::Settings;
                    self.settings_panel.current_tab = SettingsTab::CustomTexts;
                    self.settings_panel.custom_texts_state.title_input = passage.category;
                    self.settings_panel.custom_texts_state.content_input = passage.text_content;
                }
            }
        }

        ctx.request_repaint();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        ConfigLoader::save(&self.config);
    }
}
