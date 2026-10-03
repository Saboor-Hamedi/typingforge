use crate::db::{DatabaseConnection, DbPassage, DbQueries, User};
use crate::game::{GameMode, TimedDuration, WordCountTarget};
use crate::ui::components::{ButtonVariant, UnifiedButton, UnifiedInput};
use crate::ui::confirm::ConfirmModal;
use crate::ui::theme::Theme;
use crate::utils::text::sanitize_text;
use crate::utils::text_generator::TextGenerator;
use egui::{Color32, Frame, RichText, Sense, Stroke, Vec2};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct CustomPassageRequest {
    pub text: String,
    pub mode: GameMode,
    pub word_target: Option<WordCountTarget>,
    pub duration: Option<TimedDuration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassageCreationMode {
    Words,
    Time,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorStatus {
    Idle,
    Success(String),
    Error(String),
}

fn get_smart_passage_title(passage: &DbPassage) -> String {
    let cat = passage.category.trim();
    if cat.is_empty()
        || cat.eq_ignore_ascii_case("custom")
        || cat.eq_ignore_ascii_case("test")
        || cat.eq_ignore_ascii_case("seed")
        || cat.eq_ignore_ascii_case("default")
    {
        let words: Vec<&str> = passage.text_content.split_whitespace().take(5).collect();
        if words.is_empty() {
            format!("Passage #{}", passage.id.abs())
        } else {
            let mut preview = words.join(" ");
            if passage.text_content.split_whitespace().count() > 5 {
                preview.push_str("…");
            }
            preview
        }
    } else {
        cat.to_string()
    }
}

pub struct CustomTextsTabState {
    pub title_input: String,
    pub content_input: String,
    pub search_query: String,
    pub mode: PassageCreationMode,
    pub word_target: usize,     // 20, 40
    pub time_target_sec: usize, // 20, 40
    pub toast_message: Option<(String, f64)>, // (message, expiry_timestamp)
    pub limit_notice: Option<String>,
    pub text_to_delete: Option<(i64, String)>,
    pub editing_passage_id: Option<i64>,

    // Bulk text generator state
    pub generate_count_input: String,
    pub generate_error: Option<String>,
    pub generate_success: Option<String>,
    pub is_generating: Arc<AtomicBool>,
    pub generate_progress: Arc<AtomicUsize>,
    pub generate_total: Arc<AtomicUsize>,
    pub generate_cancel: Arc<AtomicBool>,
    pub generator_status: Arc<Mutex<GeneratorStatus>>,
}

impl Default for CustomTextsTabState {
    fn default() -> Self {
        Self {
            title_input: String::new(),
            content_input: String::new(),
            search_query: String::new(),
            mode: PassageCreationMode::Words,
            word_target: 20,
            time_target_sec: 20,
            toast_message: None,
            limit_notice: None,
            text_to_delete: None,
            editing_passage_id: None,
            generate_count_input: "50".to_string(),
            generate_error: None,
            generate_success: None,
            is_generating: Arc::new(AtomicBool::new(false)),
            generate_progress: Arc::new(AtomicUsize::new(0)),
            generate_total: Arc::new(AtomicUsize::new(0)),
            generate_cancel: Arc::new(AtomicBool::new(false)),
            generator_status: Arc::new(Mutex::new(GeneratorStatus::Idle)),
        }
    }
}

pub struct CustomTextsTab;

impl CustomTextsTab {
    pub fn show(
        ui: &mut egui::Ui,
        state: &mut CustomTextsTabState,
        db: &DatabaseConnection,
        theme: &Theme,
        _current_user: &Option<User>,
        passage_to_load: &mut Option<CustomPassageRequest>,
    ) {
        let current_time = ui.ctx().input(|i| i.time);

        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.label(
                RichText::new("Custom Typing Passages Library")
                    .color(theme.accent)
                    .strong()
                    .monospace()
                    .size(13.0),
            );
            ui.label(
                RichText::new("Design custom sentences with strict 25/40 word limits, live telemetry counters, and instant practice.")
                    .color(theme.text_dim)
                    .monospace()
                    .size(11.0),
            );
            ui.add_space(14.0);

            let card_w = ui.available_width();
            let pad = 20.0;
            let inner_w = card_w - pad * 2.0;

            // ─────────────────────────────────────────────────────────────
            // 1. CREATE NEW PASSAGE FORM
            // ─────────────────────────────────────────────────────────────
            let create_frame = Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(pad));

            create_frame.show(ui, |ui| {
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);

                if let Some(id) = state.editing_passage_id {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("✎ Editing Passage #{}", id.abs()))
                                .color(theme.accent)
                                .strong()
                                .size(12.5)
                                .monospace(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if UnifiedButton::show(ui, "Cancel Edit", ButtonVariant::Ghost, theme, 90.0).clicked() {
                                state.editing_passage_id = None;
                                state.title_input.clear();
                                state.content_input.clear();
                                state.limit_notice = None;
                            }
                        });
                    });
                } else {
                    ui.label(
                        RichText::new("Create New Practice Passage")
                            .color(theme.text_active)
                            .strong()
                            .size(12.0)
                            .monospace(),
                    );
                }
                ui.add_space(10.0);

                // 4 Uniform Selection Buttons: Word Mode, Time Mode, 20 Limit, 40 Limit
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                    // 1. Word Mode
                    let words_active = state.mode == PassageCreationMode::Words;
                    let words_var = if words_active { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                    if UnifiedButton::show(ui, "Word Mode", words_var, theme, 105.0).clicked() {
                        state.mode = PassageCreationMode::Words;
                        state.limit_notice = None;
                    }

                    // 2. Time Mode
                    let time_active = state.mode == PassageCreationMode::Time;
                    let time_var = if time_active { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                    if UnifiedButton::show(ui, "Time Mode", time_var, theme, 105.0).clicked() {
                        state.mode = PassageCreationMode::Time;
                        state.limit_notice = None;
                    }

                    // Divider space
                    ui.add_space(8.0);

                    // 3 & 4. Limit Buttons (Uniform styling matching mode buttons)
                    match state.mode {
                        PassageCreationMode::Words => {
                            let is_20 = state.word_target == 20;
                            let v_20 = if is_20 { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                            if UnifiedButton::show(ui, "20 Words", v_20, theme, 105.0).clicked() {
                                state.word_target = 20;
                                state.limit_notice = None;
                            }

                            let is_40 = state.word_target == 40;
                            let v_40 = if is_40 { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                            if UnifiedButton::show(ui, "40 Words", v_40, theme, 105.0).clicked() {
                                state.word_target = 40;
                                state.limit_notice = None;
                            }
                        }
                        PassageCreationMode::Time => {
                            let is_20 = state.time_target_sec == 20;
                            let v_20 = if is_20 { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                            if UnifiedButton::show(ui, "20 Seconds", v_20, theme, 105.0).clicked() {
                                state.time_target_sec = 20;
                                state.limit_notice = None;
                            }

                            let is_40 = state.time_target_sec == 40;
                            let v_40 = if is_40 { ButtonVariant::Primary } else { ButtonVariant::Secondary };
                            if UnifiedButton::show(ui, "40 Seconds", v_40, theme, 105.0).clicked() {
                                state.time_target_sec = 40;
                                state.limit_notice = None;
                            }
                        }
                    }
                });

                ui.add_space(14.0);

                let inner_w = ui.available_width();

                // Category / Title Input
                ui.label(RichText::new("PASSAGE CATEGORY / TITLE").color(theme.text_dim).size(10.5).monospace());
                ui.add_space(4.0);
                UnifiedInput::singleline(
                    ui,
                    &mut state.title_input,
                    "e.g., prose, code, quotes, custom",
                    theme,
                    inner_w,
                );

                ui.add_space(12.0);

                // Text Content Input
                ui.label(RichText::new("SENTENCE / PASSAGE TEXT").color(theme.text_dim).size(10.5).monospace());
                ui.add_space(4.0);
                UnifiedInput::multiline(
                    ui,
                    &mut state.content_input,
                    "Type or paste practice text here (strict 25 or 40 word limits enforced)...",
                    theme,
                    inner_w,
                    4,
                );

                // ─────────────────────────────────────────────────────────────
                // Live Counter & Strict Limits Enforcement
                // ─────────────────────────────────────────────────────────────
                let cur_words = state.content_input.split_whitespace().count();
                let cur_chars = state.content_input.chars().count();

                let max_words = match state.mode {
                    PassageCreationMode::Words => state.word_target,
                    PassageCreationMode::Time => match state.time_target_sec {
                        15 => 25,
                        30 => 40,
                        _ => 80,
                    },
                };

                let max_chars = max_words * 7;
                let is_full = cur_words >= max_words;

                if cur_words > max_words {
                    // Strict word limit truncator
                    let words_list: Vec<&str> = state.content_input.split_whitespace().take(max_words).collect();
                    state.content_input = words_list.join(" ");
                    state.limit_notice = Some(format!("Capped at {max_words} words maximum limit!"));
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("TELEMETRY:").color(theme.text_dim).monospace().size(10.5));
                    ui.add_space(4.0);

                    // Word count chip
                    let word_chip_color = if cur_words == max_words {
                        theme.accent
                    } else if cur_words > max_words {
                        Color32::from_rgb(255, 92, 92)
                    } else {
                        theme.text_active
                    };

                    Frame::none()
                        .fill(theme.bg)
                        .stroke(Stroke::new(1.0, theme.border))
                        .rounding(10.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("words: {cur_words} / {max_words}"))
                                    .color(word_chip_color)
                                    .monospace()
                                    .strong()
                                    .size(11.0),
                            );
                        });

                    ui.add_space(6.0);

                    // Char count chip
                    Frame::none()
                        .fill(theme.bg)
                        .stroke(Stroke::new(1.0, theme.border))
                        .rounding(10.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("chars: {cur_chars} / {max_chars}"))
                                    .color(theme.text_dim)
                                    .monospace()
                                    .size(11.0),
                            );
                        });

                    // Mode summary chip
                    let summary_label = match state.mode {
                        PassageCreationMode::Words => format!("target: {max_words} words"),
                        PassageCreationMode::Time => format!("target: {}s time", state.time_target_sec),
                    };

                    Frame::none()
                        .fill(theme.bg)
                        .stroke(Stroke::new(1.0, theme.border))
                        .rounding(10.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(summary_label).color(theme.text_dim).monospace().size(11.0));
                        });
                });

                // Visual Mini Progress Bar
                ui.add_space(6.0);
                let progress = (cur_words as f32 / max_words as f32).clamp(0.0, 1.0);
                let (prog_rect, _) = ui.allocate_exact_size(Vec2::new(inner_w, 4.0), Sense::hover());
                ui.painter().rect_filled(prog_rect, 2.0, theme.border);
                if progress > 0.0 {
                    let filled_w = prog_rect.width() * progress;
                    let fill_col = if is_full { Color32::from_rgb(255, 92, 92) } else { theme.accent };
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(prog_rect.min, Vec2::new(filled_w, 4.0)),
                        2.0,
                        fill_col,
                    );
                }

                if let Some(notice) = &state.limit_notice {
                    ui.add_space(6.0);
                    ui.label(RichText::new(notice).color(Color32::from_rgb(235, 175, 75)).monospace().size(10.5));
                }

                ui.add_space(14.0);

                // Action Buttons Row: Adapts dynamically for Create mode vs Edit mode
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

                    if UnifiedButton::show(ui, "Sanitize Text", ButtonVariant::Secondary, theme, 110.0).clicked() {
                        state.content_input = sanitize_text(&state.content_input);
                        state.toast_message = Some(("Normalized quotes and whitespace.".to_string(), current_time + 3.0));
                    }

                    if let Some(id) = state.editing_passage_id {
                        // ── EDIT MODE ACTIONS ──
                        if UnifiedButton::show(ui, "Update Passage", ButtonVariant::Primary, theme, 130.0).clicked() {
                            let clean = sanitize_text(&state.content_input);
                            if clean.trim().is_empty() {
                                state.toast_message = Some(("Please enter text content first.".to_string(), current_time + 3.0));
                            } else {
                                let category = if state.title_input.trim().is_empty() {
                                    "custom"
                                } else {
                                    state.title_input.trim()
                                };

                                match DbQueries::update_passage(db, id, &clean, category) {
                                    Ok(_) => {
                                        state.toast_message = Some((format!("Updated passage #{} successfully!", id.abs()), current_time + 3.0));
                                        state.editing_passage_id = None;
                                        state.title_input.clear();
                                        state.content_input.clear();
                                        state.limit_notice = None;
                                    }
                                    Err(e) => {
                                        state.toast_message = Some((format!("Error updating passage: {e}"), current_time + 3.0));
                                    }
                                }
                            }
                        }

                        if UnifiedButton::show(ui, "Update & Practice", ButtonVariant::Secondary, theme, 140.0).clicked() {
                            let clean = sanitize_text(&state.content_input);
                            if clean.trim().is_empty() {
                                state.toast_message = Some(("Please enter text content first.".to_string(), current_time + 3.0));
                            } else {
                                let category = if state.title_input.trim().is_empty() {
                                    "custom"
                                } else {
                                    state.title_input.trim()
                                };

                                let _ = DbQueries::update_passage(db, id, &clean, category);

                                let (mode, word_target, duration) = match state.mode {
                                    PassageCreationMode::Words => {
                                        let wt = match state.word_target {
                                            40 => WordCountTarget::Words40,
                                            _ => WordCountTarget::Words25,
                                        };
                                        (GameMode::Words, Some(wt), None)
                                    }
                                    PassageCreationMode::Time => {
                                        let td = match state.time_target_sec {
                                            40 => TimedDuration::Sec40,
                                            _ => TimedDuration::Sec25,
                                        };
                                        (GameMode::Timed, None, Some(td))
                                    }
                                };

                                *passage_to_load = Some(CustomPassageRequest {
                                    text: clean,
                                    mode,
                                    word_target,
                                    duration,
                                });
                                state.toast_message = Some(("Updated & loaded to practice session!".to_string(), current_time + 3.0));
                                state.editing_passage_id = None;
                                state.title_input.clear();
                                state.content_input.clear();
                            }
                        }

                        if UnifiedButton::show(ui, "Cancel", ButtonVariant::Ghost, theme, 70.0).clicked() {
                            state.editing_passage_id = None;
                            state.title_input.clear();
                            state.content_input.clear();
                            state.limit_notice = None;
                        }
                    } else {
                        // ── CREATE MODE ACTIONS ──
                        if UnifiedButton::show(ui, "Save & Practice Now", ButtonVariant::Primary, theme, 160.0).clicked() {
                            let clean = sanitize_text(&state.content_input);
                            if clean.trim().is_empty() {
                                state.toast_message = Some(("Please enter text content first.".to_string(), current_time + 3.0));
                            } else {
                                let category = if state.title_input.trim().is_empty() {
                                    "custom"
                                } else {
                                    state.title_input.trim()
                                };

                                let _ = DbQueries::insert_passage(db, &clean, category, true);

                                let (mode, word_target, duration) = match state.mode {
                                    PassageCreationMode::Words => {
                                        let wt = match state.word_target {
                                            40 => WordCountTarget::Words40,
                                            _ => WordCountTarget::Words25,
                                        };
                                        (GameMode::Words, Some(wt), None)
                                    }
                                    PassageCreationMode::Time => {
                                        let td = match state.time_target_sec {
                                            40 => TimedDuration::Sec40,
                                            _ => TimedDuration::Sec25,
                                        };
                                        (GameMode::Timed, None, Some(td))
                                    }
                                };

                                *passage_to_load = Some(CustomPassageRequest {
                                    text: clean,
                                    mode,
                                    word_target,
                                    duration,
                                });
                                state.toast_message = Some(("Loaded passage to practice session!".to_string(), current_time + 3.0));
                            }
                        }

                        if UnifiedButton::show(ui, "Save to Library", ButtonVariant::Secondary, theme, 120.0).clicked() {
                            let clean = sanitize_text(&state.content_input);
                            if !clean.trim().is_empty() {
                                let category = if state.title_input.trim().is_empty() {
                                    "custom"
                                } else {
                                    state.title_input.trim()
                                };

                                match DbQueries::insert_passage(db, &clean, category, true) {
                                    Ok(_) => {
                                        state.toast_message = Some(("Saved passage to database library.".to_string(), current_time + 3.0));
                                        state.title_input.clear();
                                        state.content_input.clear();
                                        state.limit_notice = None;
                                    }
                                    Err(e) => {
                                        state.toast_message = Some((format!("Error: {e}"), current_time + 3.0));
                                    }
                                }
                            }
                        }
                    }
                });

                // Toast Notification indicator (fades out after 3 seconds)
                if let Some((msg, expiry)) = &state.toast_message {
                    if current_time < *expiry {
                        ui.add_space(8.0);
                        let remaining = *expiry - current_time;
                        let alpha = (remaining / 0.5).min(1.0) as f32;
                        let toast_color = Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), (alpha * 255.0) as u8);

                        Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 20))
                            .stroke(Stroke::new(1.0, toast_color))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!("✓  {msg}")).color(toast_color).size(11.0).monospace());
                            });
                    }
                }
            });

            // Clean gap-6 between Create card and Generator card
            ui.add_space(24.0);

            // ─────────────────────────────────────────────────────────────
            // 2. BULK LITERATURE TEXT GENERATOR & NORMALIZER (Ported from generateText.py)
            // ─────────────────────────────────────────────────────────────
            let gen_frame = Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(pad));

            gen_frame.show(ui, |ui| {
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);

                // Drain any completed background generator status
                {
                    let mut st = state.generator_status.lock().unwrap();
                    match std::mem::replace(&mut *st, GeneratorStatus::Idle) {
                        GeneratorStatus::Success(msg) => {
                            state.generate_success = Some(msg);
                            state.generate_error = None;
                        }
                        GeneratorStatus::Error(err) => {
                            state.generate_error = Some(err);
                            state.generate_success = None;
                        }
                        GeneratorStatus::Idle => {}
                    }
                }

                ui.label(
                    RichText::new("Bulk Literature Text Generator & Normalizer")
                        .color(theme.text_active)
                        .strong()
                        .size(12.0)
                        .monospace(),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new("Generate authentic 25 & 40-word prose and quotes directly from classic literature. Automatically normalizes quotes (\"\" '' «» ″ → \") and special symbols into clean, standard database records.")
                        .color(theme.text_dim)
                        .size(11.0)
                        .monospace(),
                );
                ui.add_space(14.0);

                // Beautiful input styled identically to the Login Input
                let inner_w = ui.available_width();
                ui.label(RichText::new("NUMBER OF PASSAGES TO GENERATE").color(theme.text_dim).size(10.5).monospace());
                ui.add_space(4.0);
                UnifiedInput::singleline(
                    ui,
                    &mut state.generate_count_input,
                    "e.g. 10, 50, 500, 5000, 10000000",
                    theme,
                    inner_w,
                );

                ui.add_space(10.0);

                // Error feedback banner (matching profile login error style)
                if let Some(err) = &state.generate_error {
                    let err_frame = Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(239, 68, 68, 25))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
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

                // Success feedback banner (matching profile login success style)
                if let Some(msg) = &state.generate_success {
                    let ok_frame = Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(16, 185, 129, 25))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(16, 185, 129)))
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

                let is_gen = state.is_generating.load(Ordering::Relaxed);
                if is_gen {
                    let done = state.generate_progress.load(Ordering::Relaxed);
                    let total = state.generate_total.load(Ordering::Relaxed);

                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Generating & normalizing passages... ({done} / {total} created)"))
                                .color(theme.accent)
                                .strong()
                                .monospace()
                                .size(11.5),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if UnifiedButton::show(ui, "Cancel", ButtonVariant::Danger, theme, 75.0).clicked() {
                                state.generate_cancel.store(true, Ordering::Relaxed);
                            }
                        });
                    });

                    ui.add_space(6.0);
                    let progress = if total > 0 { (done as f32 / total as f32).clamp(0.0, 1.0) } else { 0.0 };
                    let (prog_rect, _) = ui.allocate_exact_size(Vec2::new(inner_w, 4.0), Sense::hover());
                    ui.painter().rect_filled(prog_rect, 2.0, theme.border);
                    if progress > 0.0 {
                        let filled_w = prog_rect.width() * progress;
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(prog_rect.min, Vec2::new(filled_w, 4.0)),
                            2.0,
                            theme.accent,
                        );
                    }
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(60));
                    ui.add_space(8.0);
                } else {
                    // Action Buttons Row
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

                        // 1. Primary Action: Generate & Insert into Database
                        if UnifiedButton::show(ui, "⚡ Generate to Database", ButtonVariant::Primary, theme, 185.0).clicked() {
                            let raw_str = state.generate_count_input.trim();
                            let clean_str: String = raw_str.chars().filter(|c| c.is_ascii_digit()).collect();

                            match clean_str.parse::<usize>() {
                                Ok(count) if count > 0 => {
                                    state.generate_error = None;
                                    state.generate_success = None;
                                    state.generate_progress.store(0, Ordering::Relaxed);
                                    state.generate_total.store(count, Ordering::Relaxed);
                                    state.generate_cancel.store(false, Ordering::Relaxed);
                                    state.is_generating.store(true, Ordering::Relaxed);

                                    let is_gen_arc = Arc::clone(&state.is_generating);
                                    let prog_arc = Arc::clone(&state.generate_progress);
                                    let cancel_arc = Arc::clone(&state.generate_cancel);
                                    let status_arc = Arc::clone(&state.generator_status);
                                    let db_clone = db.clone();

                                    std::thread::spawn(move || {
                                        match TextGenerator::generate_into_database(
                                            &db_clone,
                                            count,
                                            Some(prog_arc),
                                            Some(cancel_arc),
                                        ) {
                                            Ok(n) => {
                                                *status_arc.lock().unwrap() = GeneratorStatus::Success(
                                                    format!("Successfully generated and inserted {n} normalized passages into database!"),
                                                );
                                            }
                                            Err(e) => {
                                                *status_arc.lock().unwrap() = GeneratorStatus::Error(e);
                                            }
                                        }
                                        is_gen_arc.store(false, Ordering::Relaxed);
                                    });
                                }
                                _ => {
                                    state.generate_error = Some("Please enter a valid positive number (e.g. 10, 50, 500, 5000, 10000000).".to_string());
                                    state.generate_success = None;
                                }
                            }
                        }

                        // 2. Secondary Action: Normalize all existing database passages
                        if UnifiedButton::show(ui, "✨ Normalize Existing Passages", ButtonVariant::Secondary, theme, 220.0).clicked() {
                            match DbQueries::normalize_existing_passages(db) {
                                Ok(n) => {
                                    state.generate_success = Some(format!("Processed library: cleaned and normalized {n} passages with standard quotes & symbols."));
                                    state.generate_error = None;
                                }
                                Err(e) => {
                                    state.generate_error = Some(format!("Failed to normalize database passages: {e}"));
                                    state.generate_success = None;
                                }
                            }
                        }
                    });
                }
            });

            // Clean gap-6 between Generator card and Library card
            ui.add_space(24.0);

            // ─────────────────────────────────────────────────────────────
            // 3. SAVED PASSAGES LIBRARY (With SQLite & FTS5 search)
            // ─────────────────────────────────────────────────────────────
            let list_frame = Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(pad));

            list_frame.show(ui, |ui| {
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Saved Passages Library")
                            .color(theme.text_active)
                            .strong()
                            .size(12.0)
                            .monospace(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        UnifiedInput::singleline(
                            ui,
                            &mut state.search_query,
                            "Search library...",
                            theme,
                            220.0,
                        );
                    });
                });

                ui.add_space(14.0);

                let passages_res: rusqlite::Result<Vec<DbPassage>> = if state.search_query.trim().is_empty() {
                    DbQueries::get_custom_passages(db, 50)
                } else {
                    DbQueries::search_passages(db, state.search_query.trim())
                };

                match passages_res {
                    Ok(passages) => {
                        let passages_to_display: Vec<DbPassage> = passages.into_iter().take(50).collect();
                        if passages_to_display.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(14.0);
                                ui.label(
                                    RichText::new("No custom passages found in library.")
                                        .color(theme.text_dim)
                                        .monospace()
                                        .size(11.5),
                                );
                                ui.add_space(14.0);
                            });
                        } else {
                            // Dynamic height: No constrained max_height, grows naturally with items
                            for passage in passages_to_display {
                                let item_frame = Frame::none()
                                    .fill(theme.bg)
                                    .stroke(Stroke::new(1.0, theme.border))
                                    .rounding(8.0)
                                    .inner_margin(egui::Margin::symmetric(14.0, 10.0));

                                item_frame.show(ui, |ui| {
                                    let avail_w = ui.available_width();
                                    let actions_w = 236.0;
                                    let info_w = (avail_w - actions_w - 12.0).max(120.0);

                                    ui.horizontal(|ui| {
                                        // Left details: bounded width so it never pushes or bugs right buttons
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(info_w, 0.0),
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                ui.set_width(info_w);
                                                ui.horizontal(|ui| {
                                                    let smart_title = get_smart_passage_title(&passage);
                                                    ui.label(
                                                        RichText::new(smart_title)
                                                            .color(theme.text_active)
                                                            .strong()
                                                            .monospace()
                                                            .size(12.0),
                                                    );

                                                    let cat_label = if passage.category.trim().is_empty()
                                                        || passage.category.eq_ignore_ascii_case("custom")
                                                        || passage.category.eq_ignore_ascii_case("seed")
                                                    {
                                                        format!("#{}", passage.id.abs())
                                                    } else {
                                                        passage.category.to_uppercase()
                                                    };

                                                    Frame::none()
                                                        .fill(theme.bg_surface)
                                                        .stroke(Stroke::new(1.0, theme.border))
                                                        .rounding(4.0)
                                                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                RichText::new(cat_label)
                                                                    .color(theme.accent)
                                                                    .monospace()
                                                                    .size(9.5),
                                                            );
                                                        });
                                                });

                                                let preview_snippet = if passage.text_content.len() > 80 {
                                                    format!("{}…", &passage.text_content[..80])
                                                } else {
                                                    passage.text_content.clone()
                                                };
                                                ui.label(
                                                    RichText::new(preview_snippet)
                                                        .color(theme.text_dim)
                                                        .size(11.0)
                                                        .monospace(),
                                                );

                                                ui.label(
                                                    RichText::new(format!("{} words", passage.word_count))
                                                        .color(theme.text_dim)
                                                        .size(10.0)
                                                        .monospace(),
                                                );
                                            },
                                        );

                                        // Right action buttons: Practice | Edit | Delete
                                        // All 3 buttons look identical (same variant, same width, same height)
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

                                            // 1. Delete (furthest right)
                                            if UnifiedButton::show(ui, "Delete", ButtonVariant::Secondary, theme, 72.0).clicked() {
                                                state.text_to_delete = Some((passage.id, format!("Passage #{}", passage.id.abs())));
                                            }

                                            // 2. Edit (middle)
                                            if UnifiedButton::show(ui, "Edit", ButtonVariant::Secondary, theme, 72.0).clicked() {
                                                state.editing_passage_id = Some(passage.id);
                                                state.title_input = passage.category.clone();
                                                state.content_input = passage.text_content.clone();
                                                if passage.word_count <= 20 {
                                                    state.mode = PassageCreationMode::Words;
                                                    state.word_target = 20;
                                                } else {
                                                    state.mode = PassageCreationMode::Words;
                                                    state.word_target = 40;
                                                }
                                                state.limit_notice = None;
                                            }

                                            // 3. Practice (leftmost of the 3 buttons)
                                            if UnifiedButton::show(ui, "Practice", ButtonVariant::Secondary, theme, 72.0).clicked() {
                                                let wt = if passage.word_count >= 30 {
                                                    WordCountTarget::Words40
                                                } else {
                                                    WordCountTarget::Words25
                                                };

                                                *passage_to_load = Some(CustomPassageRequest {
                                                    text: passage.text_content.clone(),
                                                    mode: GameMode::Words,
                                                    word_target: Some(wt),
                                                    duration: None,
                                                });
                                            }
                                        });
                                    });
                                });
                                ui.add_space(8.0);
                            }
                        }
                    }
                    Err(e) => {
                        ui.label(
                            RichText::new(format!("Failed to load passages: {e}"))
                                .color(Color32::from_rgb(255, 92, 92)),
                        );
                    }
                }
            });

            // Delete passage modal confirmation
            if let Some((passage_id, ref title)) = state.text_to_delete.clone() {
                if let Some(confirmed) = ConfirmModal::show(
                    ui.ctx(),
                    theme,
                    "DELETE CUSTOM PASSAGE",
                    title,
                    "Are you sure you want to permanently delete this passage from your library?",
                    "Delete Passage",
                ) {
                    if confirmed {
                        let _ = DbQueries::delete_passage(db, passage_id);
                        state.toast_message = Some((format!("Deleted {title}."), current_time + 3.0));
                    }
                    state.text_to_delete = None;
                }
            }
        });
    }
}
