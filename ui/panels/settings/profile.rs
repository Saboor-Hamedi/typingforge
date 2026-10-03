pub mod typing_backup;
pub mod typing_import;

use self::typing_backup::TypingBackup;
use self::typing_import::TypingImport;
use crate::auth::LocalAuth;
use crate::data::AppConfig;
use crate::db::{DatabaseConnection, DbQueries, User};
use crate::ui::components::{ButtonVariant, UnifiedButton, UnifiedInput};
use crate::ui::confirm::ConfirmModal;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, RichText, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    SignIn,
    Register,
}

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum SyncOp {
    Idle,
    Exporting,
    Importing,
    Completed(String),
    Failed(String),
}

pub struct ProfileTabState {
    pub auth_mode: AuthMode,
    pub username_input: String,
    pub password_input: String,
    pub confirm_password_input: String,
    pub auth_error: Option<String>,
    pub auth_success: Option<String>,
    pub show_delete_confirm: bool,
    pub show_truncate_confirm: bool,
    pub truncate_status: Option<String>,
    pub sync_op: Arc<std::sync::Mutex<SyncOp>>,
    pub sync_progress: Arc<AtomicUsize>,
    pub sync_cancel: Arc<AtomicBool>,
}

impl Default for ProfileTabState {
    fn default() -> Self {
        Self {
            auth_mode: AuthMode::SignIn,
            username_input: String::new(),
            password_input: String::new(),
            confirm_password_input: String::new(),
            auth_error: None,
            auth_success: None,
            show_delete_confirm: false,
            show_truncate_confirm: false,
            truncate_status: None,
            sync_op: Arc::new(std::sync::Mutex::new(SyncOp::Idle)),
            sync_progress: Arc::new(AtomicUsize::new(0)),
            sync_cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub struct ProfileTab;

impl ProfileTab {
    pub fn show(
        ui: &mut egui::Ui,
        state: &mut ProfileTabState,
        config: &mut AppConfig,
        theme: &Theme,
        db: &DatabaseConnection,
        current_user: &mut Option<User>,
    ) {
        ui.vertical(|ui| {
            ui.add_space(8.0);

            let card_w = ui.available_width();
            let pad = 20.0;
            let inner_w = card_w - pad * 2.0;
            let mut should_sign_out = false;

            if let Some(user) = current_user.as_ref() {
                // ─────────────────────────────────────────────────────────────
                // LOGGED-IN PROFILE VIEW (Full Width Unified Container)
                // ─────────────────────────────────────────────────────────────
                let frame = egui::Frame::none()
                    .fill(theme.bg_surface)
                    .stroke(egui::Stroke::new(1.0, theme.border))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(pad));

                frame.show(ui, |ui| {
                    ui.set_min_width(inner_w);
                    ui.set_max_width(inner_w);
                    ui.vertical(|ui| {
                        // Prominent User Avatar & Identity
                        ui.horizontal(|ui| {
                            let initial = user
                                .username
                                .chars()
                                .next()
                                .unwrap_or('U')
                                .to_uppercase()
                                .to_string();

                            let (rect, _response) = ui.allocate_exact_size(Vec2::new(54.0, 54.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 27.0, theme.bg);
                            ui.painter().circle_stroke(rect.center(), 27.0, egui::Stroke::new(1.5, theme.accent));
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                initial,
                                egui::FontId::monospace(22.0),
                                theme.accent,
                            );

                            ui.add_space(16.0);

                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&user.username)
                                        .color(theme.text_active)
                                        .strong()
                                        .size(20.0)
                                        .monospace(),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new("LOCAL PROFILE  ·  ARGON2 SECURED")
                                        .color(theme.accent)
                                        .size(10.5)
                                        .monospace(),
                                );
                            });
                        });

                        ui.add_space(18.0);
                        ui.separator();
                        ui.add_space(14.0);

                        // Fetch Stats Summary from DB
                        let sessions_res = DbQueries::get_recent_sessions(db, Some(user.id), 100);
                        let (test_count, avg_wpm, best_wpm) = match sessions_res {
                            Ok(sessions) => {
                                let count = sessions.len();
                                if count > 0 {
                                    let sum_wpm: f32 = sessions.iter().map(|s| s.wpm as f32).sum();
                                    let max_wpm: f32 = sessions.iter().map(|s| s.wpm as f32).fold(0.0_f32, f32::max);
                                    (count, sum_wpm / count as f32, max_wpm)
                                } else {
                                    (0, 0.0, 0.0)
                                }
                            }
                            Err(_) => (0, 0.0, 0.0),
                        };

                        // Stats spread evenly across full card width
                        ui.horizontal(|ui| {
                            ui.columns(3, |cols| {
                                let col_h = 60.0;
                                // Tests
                                let (r0, _) = cols[0].allocate_exact_size(Vec2::new(cols[0].available_width(), col_h), egui::Sense::hover());
                                cols[0].painter().rect_filled(r0, 6.0, theme.bg);
                                cols[0].painter().rect_stroke(r0, 6.0, egui::Stroke::new(1.0, theme.border));
                                cols[0].painter().text(Pos2::new(r0.center().x, r0.min.y + 14.0), egui::Align2::CENTER_CENTER, "TESTS", egui::FontId::monospace(10.5), theme.text_dim);
                                cols[0].painter().text(Pos2::new(r0.center().x, r0.min.y + 38.0), egui::Align2::CENTER_CENTER, format!("{test_count}"), egui::FontId::monospace(17.0), theme.text_active);

                                // Best WPM
                                let (r1, _) = cols[1].allocate_exact_size(Vec2::new(cols[1].available_width(), col_h), egui::Sense::hover());
                                cols[1].painter().rect_filled(r1, 6.0, theme.bg);
                                cols[1].painter().rect_stroke(r1, 6.0, egui::Stroke::new(1.0, theme.border));
                                cols[1].painter().text(Pos2::new(r1.center().x, r1.min.y + 14.0), egui::Align2::CENTER_CENTER, "BEST WPM", egui::FontId::monospace(10.5), theme.text_dim);
                                cols[1].painter().text(Pos2::new(r1.center().x, r1.min.y + 38.0), egui::Align2::CENTER_CENTER, format!("{best_wpm:.0}"), egui::FontId::monospace(17.0), theme.accent);

                                // Avg WPM
                                let (r2, _) = cols[2].allocate_exact_size(Vec2::new(cols[2].available_width(), col_h), egui::Sense::hover());
                                cols[2].painter().rect_filled(r2, 6.0, theme.bg);
                                cols[2].painter().rect_stroke(r2, 6.0, egui::Stroke::new(1.0, theme.border));
                                cols[2].painter().text(Pos2::new(r2.center().x, r2.min.y + 14.0), egui::Align2::CENTER_CENTER, "AVG WPM", egui::FontId::monospace(10.5), theme.text_dim);
                                cols[2].painter().text(Pos2::new(r2.center().x, r2.min.y + 38.0), egui::Align2::CENTER_CENTER, format!("{avg_wpm:.0}"), egui::FontId::monospace(17.0), theme.text_active);
                            });
                        });

                        ui.add_space(20.0);

                        // Actions aligned consistently to the right
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if UnifiedButton::show(ui, "Delete Profile", ButtonVariant::Danger, theme, 125.0).clicked() {
                                    state.show_delete_confirm = true;
                                }
                                ui.add_space(10.0);
                                if UnifiedButton::show(ui, "Sign Out", ButtonVariant::Secondary, theme, 110.0).clicked() {
                                    should_sign_out = true;
                                }
                            });
                        });
                    });
                });

                if should_sign_out {
                    *current_user = None;
                    state.auth_success = Some("Signed out to Guest mode.".to_string());
                    state.auth_error = None;
                }

                if state.show_delete_confirm {
                    if let Some(user) = current_user.as_ref() {
                        if let Some(confirmed) = ConfirmModal::show(
                            ui.ctx(),
                            theme,
                            "DELETE USER PROFILE",
                            &user.username,
                            "Are you sure you want to permanently delete this profile? All your typing statistics, personal bests, and custom passages will be erased. This action cannot be undone.",
                            "Delete Account",
                        ) {
                            if confirmed {
                                let user_id = user.id;
                                let deleted_name = user.username.clone();
                                match DbQueries::delete_user(db, user_id) {
                                    Ok(_) => {
                                        *current_user = None;
                                        config.active_user_id = None;
                                        state.auth_success = Some(format!("Profile '{deleted_name}' was permanently deleted."));
                                        state.auth_error = None;
                                    }
                                    Err(e) => {
                                        state.auth_error = Some(format!("Failed to delete profile: {e}"));
                                    }
                                }
                            }
                            state.show_delete_confirm = false;
                        }
                    }
                }
            } else {
                // ─────────────────────────────────────────────────────────────
                // AUTH FORM: SIGN IN / REGISTER TABS
                // ─────────────────────────────────────────────────────────────
                let frame = egui::Frame::none()
                    .fill(theme.bg_surface)
                    .stroke(egui::Stroke::new(1.0, theme.border))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(pad));

                frame.show(ui, |ui| {
                    ui.set_min_width(inner_w);
                    ui.set_max_width(inner_w);
                    ui.vertical(|ui| {
                        // Auth Mode Toggle - Centered at the top
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

                                let signin_active = state.auth_mode == AuthMode::SignIn;
                                let signin_variant = if signin_active { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                                if UnifiedButton::show(ui, "Sign In", signin_variant, theme, 150.0).clicked() {
                                    state.auth_mode = AuthMode::SignIn;
                                    state.auth_error = None;
                                    state.auth_success = None;
                                }

                                let reg_active = state.auth_mode == AuthMode::Register;
                                let reg_variant = if reg_active { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                                if UnifiedButton::show(ui, "Create Account", reg_variant, theme, 150.0).clicked() {
                                    state.auth_mode = AuthMode::Register;
                                    state.auth_error = None;
                                    state.auth_success = None;
                                }
                            });
                        });

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(12.0);

                        // Form Subtitle
                        match state.auth_mode {
                            AuthMode::SignIn => {
                                ui.label(
                                    RichText::new("Sign in to sync your local typing progress")
                                        .color(theme.text_dim)
                                        .size(11.5)
                                        .monospace(),
                                );
                            }
                            AuthMode::Register => {
                                ui.label(
                                    RichText::new("Create a local account protected by Argon2 encryption")
                                        .color(theme.text_dim)
                                        .size(11.5)
                                        .monospace(),
                                );
                            }
                        }

                        ui.add_space(14.0);
                        let inner_w = ui.available_width();

                        // Username Input
                        ui.label(RichText::new("USERNAME").color(theme.text_dim).size(10.5).monospace());
                        ui.add_space(4.0);
                        UnifiedInput::singleline(ui, &mut state.username_input, "e.g. speed_demon", theme, inner_w);

                        ui.add_space(14.0);

                        // Password Input
                        ui.label(RichText::new("PASSWORD").color(theme.text_dim).size(10.5).monospace());
                        ui.add_space(4.0);
                        UnifiedInput::password(ui, &mut state.password_input, "At least 3 characters", theme, inner_w);

                        // Confirm Password (Only in Register mode)
                        if state.auth_mode == AuthMode::Register {
                            ui.add_space(14.0);
                            ui.label(RichText::new("CONFIRM PASSWORD").color(theme.text_dim).size(10.5).monospace());
                            ui.add_space(4.0);
                            UnifiedInput::password(ui, &mut state.confirm_password_input, "Repeat your password", theme, inner_w);
                        }

                        ui.add_space(18.0);

                        // Feedback Messages
                        if let Some(err) = &state.auth_error {
                            let err_frame = egui::Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(239, 68, 68, 25))
                                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0));
                            err_frame.show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("⚠  {err}"))
                                        .color(Color32::from_rgb(255, 92, 92))
                                        .size(11.5)
                                        .monospace(),
                                );
                            });
                            ui.add_space(10.0);
                        }

                        if let Some(msg) = &state.auth_success {
                            let ok_frame = egui::Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(16, 185, 129, 25))
                                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(16, 185, 129)))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0));
                            ok_frame.show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("✓  {msg}"))
                                        .color(Color32::from_rgb(52, 211, 153))
                                        .size(11.5)
                                        .monospace(),
                                );
                            });
                            ui.add_space(10.0);
                        }

                        // Submit Button - Left-aligned main action CTA
                        let btn_text = match state.auth_mode {
                            AuthMode::SignIn => "Sign In",
                            AuthMode::Register => "Register & Sign In",
                        };

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if UnifiedButton::show(ui, btn_text, ButtonVariant::Primary, theme, 160.0).clicked() {
                                match state.auth_mode {
                                    AuthMode::SignIn => {
                                        match LocalAuth::authenticate(db, &state.username_input, &state.password_input) {
                                            Ok(user) => {
                                                *current_user = Some(user);
                                                state.auth_error = None;
                                                state.auth_success = Some("Logged in successfully.".to_string());
                                                state.password_input.clear();
                                                state.confirm_password_input.clear();
                                            }
                                            Err(err) => {
                                                state.auth_error = Some(err);
                                                state.auth_success = None;
                                            }
                                        }
                                    }
                                    AuthMode::Register => {
                                        if state.password_input != state.confirm_password_input {
                                            state.auth_error = Some("Passwords do not match.".to_string());
                                            state.auth_success = None;
                                        } else {
                                            match LocalAuth::register(db, &state.username_input, &state.password_input) {
                                                Ok(user) => {
                                                    *current_user = Some(user);
                                                    state.auth_error = None;
                                                    state.auth_success = Some("Account created successfully!".to_string());
                                                    state.password_input.clear();
                                                    state.confirm_password_input.clear();
                                                }
                                                Err(err) => {
                                                    state.auth_error = Some(err);
                                                    state.auth_success = None;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                });
            });
            }

            if current_user.is_none() {
                ui.add_space(20.0);

                // Guest Mode Preferences Card
                let guest_frame = egui::Frame::none()
                    .fill(theme.bg_surface)
                    .stroke(egui::Stroke::new(1.0, theme.border))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(pad));

                guest_frame.show(ui, |ui| {
                    ui.set_min_width(inner_w);
                    ui.set_max_width(inner_w);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("GUEST PREFERENCES").color(theme.text_dim).size(10.5).monospace());
                        ui.add_space(8.0);
                        crate::ui::components::UnifiedToggle::row(
                            ui,
                            &mut config.persist_guest_sessions,
                            "Persist Guest Sessions",
                            "Record and save anonymous guest test runs into local history",
                            theme,
                        );
                    });
                });
            }

            ui.add_space(20.0);

            // Database & Storage Management Card
            let db_frame = egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(egui::Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(pad));

            db_frame.show(ui, |ui| {
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);
                ui.vertical(|ui| {
                    ui.label(RichText::new("DATABASE & STORAGE MANAGEMENT").color(theme.text_dim).size(10.5).monospace());
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Backup and restore your custom passages and typing sentences, or reset text passages back to clean curated prose.")
                            .color(theme.text_dim)
                            .size(11.0)
                            .monospace(),
                    );
                    ui.add_space(12.0);

                    let sync_op_guard = state.sync_op.lock().unwrap();
                    let current_sync = sync_op_guard.clone();
                    drop(sync_op_guard);

                    match current_sync {
                        SyncOp::Exporting => {
                            let prog = state.sync_progress.load(Ordering::Relaxed);
                            let status_frame = egui::Frame::none()
                                .fill(theme.bg)
                                .stroke(egui::Stroke::new(1.0, theme.accent))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(12.0, 8.0));
                            status_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.add_space(8.0);
                                     ui.label(
                                        RichText::new(format!("Streaming sentences to file... ({prog} records written)"))
                                            .color(theme.accent)
                                            .strong()
                                            .monospace()
                                            .size(11.5),
                                    );
                                });
                            });
                            ui.ctx().request_repaint_after(std::time::Duration::from_millis(80));
                            ui.add_space(10.0);
                        }
                        SyncOp::Importing => {
                            let prog = state.sync_progress.load(Ordering::Relaxed);
                            let status_frame = egui::Frame::none()
                                .fill(theme.bg)
                                .stroke(egui::Stroke::new(1.0, theme.accent))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(12.0, 8.0));
                            status_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(format!("Streaming sentences into database... ({prog} records imported)"))
                                            .color(theme.accent)
                                            .strong()
                                            .monospace()
                                            .size(11.5),
                                    );
                                });
                            });
                            ui.ctx().request_repaint_after(std::time::Duration::from_millis(80));
                            ui.add_space(10.0);
                        }
                        SyncOp::Completed(ref msg) => {
                            let status_frame = egui::Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(16, 185, 129, 25))
                                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(16, 185, 129)))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0));
                            status_frame.show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("✓  {msg}"))
                                        .color(Color32::from_rgb(52, 211, 153))
                                        .size(11.5)
                                        .monospace(),
                                );
                            });
                            ui.add_space(10.0);
                        }
                        SyncOp::Failed(ref err) => {
                            let err_frame = egui::Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(239, 68, 68, 25))
                                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0));
                            err_frame.show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("⚠  {err}"))
                                        .color(Color32::from_rgb(255, 92, 92))
                                        .size(11.5)
                                        .monospace(),
                                );
                            });
                            ui.add_space(10.0);
                        }
                        SyncOp::Idle => {}
                    }

                    if let Some(msg) = &state.truncate_status {
                        let status_frame = egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(16, 185, 129, 25))
                            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(16, 185, 129)))
                            .rounding(6.0)
                            .inner_margin(egui::Margin::symmetric(10.0, 6.0));
                        status_frame.show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("✓  {msg}"))
                                    .color(Color32::from_rgb(52, 211, 153))
                                    .size(11.5)
                                    .monospace(),
                            );
                        });
                        ui.add_space(10.0);
                    }

                    let is_syncing = matches!(current_sync, SyncOp::Exporting | SyncOp::Importing);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

                        if !is_syncing {
                            if UnifiedButton::show(ui, "Backup Data", ButtonVariant::Secondary, theme, 120.0).clicked() {
                                let file_opt = rfd::FileDialog::new()
                                    .set_title("Export Typing Sentences")
                                    .set_file_name("velotype_sentences.json")
                                    .add_filter("JSON Document (*.json)", &["json"])
                                    .save_file();

                                if let Some(path) = file_opt {
                                    let db_clone = db.clone();
                                    let op_arc = Arc::clone(&state.sync_op);
                                    let prog_arc = Arc::clone(&state.sync_progress);
                                    let cancel_arc = Arc::clone(&state.sync_cancel);
                                    prog_arc.store(0, Ordering::Relaxed);
                                    cancel_arc.store(false, Ordering::Relaxed);
                                    *op_arc.lock().unwrap() = SyncOp::Exporting;

                                    std::thread::spawn(move || {
                                        match TypingBackup::export_to_file(&db_clone, &path, Some(prog_arc), Some(cancel_arc)) {
                                            Ok(count) => {
                                                *op_arc.lock().unwrap() = SyncOp::Completed(format!("Exported {count} typing sentences successfully."));
                                            }
                                            Err(e) => {
                                                *op_arc.lock().unwrap() = SyncOp::Failed(e);
                                            }
                                        }
                                    });
                                }
                            }

                            if UnifiedButton::show(ui, "Import Data", ButtonVariant::Secondary, theme, 120.0).clicked() {
                                let file_opt = rfd::FileDialog::new()
                                    .set_title("Import Typing Sentences")
                                    .add_filter("JSON Document (*.json)", &["json"])
                                    .pick_file();

                                if let Some(path) = file_opt {
                                    let db_clone = db.clone();
                                    let user_id = current_user.as_ref().map(|u| u.id);
                                    let op_arc = Arc::clone(&state.sync_op);
                                    let prog_arc = Arc::clone(&state.sync_progress);
                                    let cancel_arc = Arc::clone(&state.sync_cancel);
                                    prog_arc.store(0, Ordering::Relaxed);
                                    cancel_arc.store(false, Ordering::Relaxed);
                                    *op_arc.lock().unwrap() = SyncOp::Importing;

                                    std::thread::spawn(move || {
                                        match TypingImport::import_from_file(&db_clone, user_id, &path, Some(prog_arc), Some(cancel_arc)) {
                                            Ok(count) => {
                                                *op_arc.lock().unwrap() = SyncOp::Completed(format!("Imported {count} typing sentences successfully."));
                                            }
                                            Err(e) => {
                                                *op_arc.lock().unwrap() = SyncOp::Failed(e);
                                            }
                                        }
                                    });
                                }
                            }

                            if UnifiedButton::show(ui, "⚠ Reset & Truncate Passages", ButtonVariant::Danger, theme, 220.0).clicked() {
                                state.show_truncate_confirm = true;
                            }
                        } else {
                            ui.label(RichText::new("Background data streaming in progress...").color(theme.text_dim).monospace().size(11.0));
                        }
                    });
                });
            });

            // Truncate Confirmation Modal
            if state.show_truncate_confirm {
                if let Some(confirmed) = ConfirmModal::show(
                    ui.ctx(),
                    theme,
                    "RESET PASSAGES DATABASE",
                    "Database Reset",
                    "Are you sure you want to reset the passages database?\n\nThis will purge corrupted or unwanted text passages and re-seed clean 25 & 40 word natural prose passages.\n\nYour user profile, account, and high scores will remain completely intact.",
                    "Reset Passages",
                ) {
                    if confirmed {
                        match DbQueries::truncate_database(db) {
                            Ok(()) => {
                                state.truncate_status = Some("Database truncated and successfully re-seeded with clean passages.".to_string());
                            }
                            Err(e) => {
                                state.truncate_status = Some(format!("Error truncating database: {e}"));
                            }
                        }
                    }
                    state.show_truncate_confirm = false;
                }
            }
        });
    }
}

