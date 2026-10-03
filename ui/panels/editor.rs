use crate::db::{DatabaseConnection, DbQueries};
use crate::ui::theme::Theme;
use crate::utils::sanitize_text;
use egui::{Color32, RichText, Vec2};

pub struct EditorPanel {
    pub buffer: String,
    pub title_buffer: String,
    pub status_message: Option<(String, f32)>, // message, display timer
}

impl Default for EditorPanel {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            title_buffer: "Custom Passage".to_string(),
            status_message: None,
        }
    }
}

impl EditorPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_with_text(&mut self, text: &str) {
        self.buffer = text.to_string();
        self.status_message = None;
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        db: &DatabaseConnection,
        user_id: Option<i64>,
        on_apply: &mut Option<String>,
        on_close: &mut bool,
        dt: f32,
    ) {
        if let Some((_, ref mut timer)) = self.status_message {
            *timer -= dt;
            if *timer <= 0.0 {
                self.status_message = None;
            }
        }

        ui.vertical(|ui| {
            ui.add_space(8.0);

            // Title & Instruction
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("PASSAGE EDITOR & CUSTOM TEXT")
                        .color(theme.accent)
                        .monospace()
                        .strong()
                        .size(13.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("✕ Close (Esc)").monospace().size(11.0)).clicked() {
                        *on_close = true;
                    }

                    if ui.button(RichText::new("🎲 Load Random Passage").monospace().size(11.0)).clicked() {
                        if let Ok(Some(passage)) = DbQueries::get_random_passage(db, None) {
                            self.title_buffer = format!("Passage ({})", passage.category);
                            self.buffer = passage.text_content;
                            self.status_message = Some(("Loaded passage from database".to_string(), 2.0));
                        }
                    }

                    if ui.button(RichText::new("✨ Auto-Sanitize").monospace().size(11.0)).clicked() {
                        self.buffer = sanitize_text(&self.buffer);
                        self.status_message = Some(("Sanitized quotes & spacing".to_string(), 2.0));
                    }
                });
            });

            ui.add_space(8.0);

            // Title Input
            ui.horizontal(|ui| {
                ui.label(RichText::new("Title:").color(theme.text_dim).monospace().size(11.0));
                ui.add(
                    egui::TextEdit::singleline(&mut self.title_buffer)
                        .hint_text("Passage title...")
                        .desired_width(260.0),
                );

                let char_count = self.buffer.chars().count();
                ui.label(
                    RichText::new(format!("Chars: {}  ·  Words: {}", char_count, self.buffer.split_whitespace().count()))
                        .color(theme.text_dim)
                        .monospace()
                        .size(11.0),
                );

                if let Some((msg, _)) = &self.status_message {
                    ui.label(RichText::new(msg).color(Color32::from_rgb(16, 185, 129)).monospace().size(11.0));
                }
            });

            ui.add_space(10.0);

            // Editor Box
            let available_h = (ui.available_height() - 56.0).max(180.0);
            let available_w = ui.available_width();

            ui.add(
                egui::TextEdit::multiline(&mut self.buffer)
                    .font(egui::FontId::monospace(14.0))
                    .desired_width(available_w)
                    .desired_rows(10)
                    .min_size(Vec2::new(available_w, available_h))
                    .hint_text("Paste or compose your custom typing passage here...\nZero-width spaces, smart quotes, and multiple spaces will be automatically normalized."),
            );

            ui.add_space(12.0);

            // Bottom Actions
            ui.horizontal(|ui| {
                let apply_btn = egui::Button::new(
                    RichText::new("  ▶ Apply & Start Practice  ")
                        .color(Color32::from_rgb(10, 14, 22))
                        .monospace()
                        .strong(),
                )
                .fill(theme.accent)
                .rounding(6.0);

                if ui.add(apply_btn).clicked() {
                    let sanitized = sanitize_text(&self.buffer);
                    if !sanitized.is_empty() {
                        let title = if self.title_buffer.trim().is_empty() {
                            "Custom Passage"
                        } else {
                            self.title_buffer.trim()
                        };

                        // Save to database passages library & FTS5 index
                        let _ = DbQueries::insert_passage(db, &sanitized, title, true);
                        let _ = DbQueries::insert_text(db, title, &sanitized, "user_edit", user_id);

                        *on_apply = Some(sanitized);
                        *on_close = true;
                    }
                }

                ui.add_space(12.0);

                if ui.button(RichText::new("Clear Buffer").monospace().size(11.0)).clicked() {
                    self.buffer.clear();
                }
            });
        });
    }
}
