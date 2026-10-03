use crate::data::AppConfig;
use crate::fx::CaretStyle;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, Rect, RichText, Stroke, Vec2};

pub struct SettingPreviewTab;

impl SettingPreviewTab {
    pub fn show(ui: &mut egui::Ui, config: &AppConfig, theme: &Theme) {
        ui.vertical(|ui| {
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("LIVE WORKSPACE PREVIEW")
                        .color(theme.accent)
                        .strong()
                        .monospace()
                        .size(13.0),
                );
                ui.add_space(12.0);
                ui.label(
                    RichText::new("• Real-time synthesis of theme, caret kinetics & typography")
                        .color(theme.text_dim)
                        .monospace()
                        .size(11.0),
                );
            });
            ui.add_space(10.0);

            // Full Height Canvas
            let preview_h = (ui.available_height() - 16.0).max(280.0);
            let preview_w = ui.available_width();
            let (rect, _resp) = ui.allocate_exact_size(Vec2::new(preview_w, preview_h), egui::Sense::hover());
            let painter = ui.painter_at(rect);

            // Canvas Background & Border
            painter.rect_filled(rect, 8.0, theme.bg);
            painter.rect_stroke(rect, 8.0, Stroke::new(1.0, theme.border));

            let font_id = egui::FontId::monospace(config.font_size);

            let line_1 = [
                ("velotype", theme.text_correct),
                ("is", theme.text_correct),
                ("engineered", theme.text_correct),
                ("for", theme.text_correct),
                ("blazing", theme.text_active),
                ("speed", theme.text_dim),
                ("and", theme.text_dim),
            ];

            let line_2 = [
                ("flawless", theme.text_dim),
                ("accuracy.", theme.text_dim),
                ("every", theme.text_dim),
                ("keystroke", theme.text_dim),
                ("flows", theme.text_dim),
                ("seamlessly", theme.text_dim),
            ];

            let line_3 = [
                ("without", theme.text_dim),
                ("interruptions", theme.text_dim),
                ("or", theme.text_dim),
                ("friction.", theme.text_dim),
            ];

            let start_x = rect.min.x + 36.0;
            let line_height = config.font_size * 1.7;
            let start_y = rect.min.y + 40.0;

            let mut caret_pos = Pos2::ZERO;

            // Render Line 1
            let mut cur_x = start_x;
            for (idx, (word, color)) in line_1.iter().enumerate() {
                painter.text(
                    Pos2::new(cur_x, start_y),
                    egui::Align2::LEFT_TOP,
                    *word,
                    font_id.clone(),
                    *color,
                );

                let word_w = ui.fonts(|f| {
                    f.layout_no_wrap(word.to_string(), font_id.clone(), Color32::WHITE).size().x
                });

                if idx == 4 {
                    // Caret is 60% through "blazing"
                    caret_pos = Pos2::new(cur_x + word_w * 0.6, start_y);
                }

                let space_w = ui.fonts(|f| {
                    f.layout_no_wrap(" ".to_string(), font_id.clone(), Color32::WHITE).size().x
                });
                cur_x += word_w + space_w;
            }

            // Render Line 2
            let mut cur_x2 = start_x;
            for (word, color) in line_2.iter() {
                painter.text(
                    Pos2::new(cur_x2, start_y + line_height),
                    egui::Align2::LEFT_TOP,
                    *word,
                    font_id.clone(),
                    *color,
                );

                let word_w = ui.fonts(|f| {
                    f.layout_no_wrap(word.to_string(), font_id.clone(), Color32::WHITE).size().x
                });
                let space_w = ui.fonts(|f| {
                    f.layout_no_wrap(" ".to_string(), font_id.clone(), Color32::WHITE).size().x
                });
                cur_x2 += word_w + space_w;
            }

            // Render Line 3
            let mut cur_x3 = start_x;
            for (word, color) in line_3.iter() {
                painter.text(
                    Pos2::new(cur_x3, start_y + line_height * 2.0),
                    egui::Align2::LEFT_TOP,
                    *word,
                    font_id.clone(),
                    *color,
                );

                let word_w = ui.fonts(|f| {
                    f.layout_no_wrap(word.to_string(), font_id.clone(), Color32::WHITE).size().x
                });
                let space_w = ui.fonts(|f| {
                    f.layout_no_wrap(" ".to_string(), font_id.clone(), Color32::WHITE).size().x
                });
                cur_x3 += word_w + space_w;
            }

            // Draw Caret
            if caret_pos != Pos2::ZERO {
                let time = ui.ctx().input(|i| i.time);
                let alpha = ((time * 4.0).sin().abs() as f32).clamp(0.25, 1.0);
                let caret_color = Color32::from_rgba_unmultiplied(
                    theme.caret.r(),
                    theme.caret.g(),
                    theme.caret.b(),
                    (255.0 * alpha) as u8,
                );

                let char_h = config.font_size * 1.15;
                let char_w = config.font_size * 0.6;

                match config.caret_style {
                    CaretStyle::Line | CaretStyle::Bar => {
                        let w = config.caret_width;
                        let caret_rect = Rect::from_min_size(caret_pos, Vec2::new(w, char_h));
                        if config.caret_glow > 0.05 {
                            let glow_color = Color32::from_rgba_unmultiplied(
                                theme.caret.r(),
                                theme.caret.g(),
                                theme.caret.b(),
                                (70.0 * config.caret_glow * alpha) as u8,
                            );
                            painter.rect_filled(caret_rect.expand(2.5 * config.caret_glow), 2.0, glow_color);
                        }
                        painter.rect_filled(caret_rect, 1.5, caret_color);
                    }
                    CaretStyle::Block => {
                        let caret_rect = Rect::from_min_size(caret_pos, Vec2::new(char_w, char_h));
                        let block_color = Color32::from_rgba_unmultiplied(
                            theme.caret.r(),
                            theme.caret.g(),
                            theme.caret.b(),
                            (140.0 * alpha) as u8,
                        );
                        painter.rect_filled(caret_rect, 2.0, block_color);
                    }
                    CaretStyle::Underline => {
                        let caret_rect = Rect::from_min_size(
                            Pos2::new(caret_pos.x, caret_pos.y + char_h - 2.5),
                            Vec2::new(char_w, config.caret_width.max(2.0)),
                        );
                        painter.rect_filled(caret_rect, 1.0, caret_color);
                    }
                }
            }

            // Bottom telemetry indicator pill inside preview
            let pill_y = rect.max.y - 32.0;
            let pill_pos = Pos2::new(rect.min.x + 36.0, pill_y);
            painter.text(
                pill_pos,
                egui::Align2::LEFT_CENTER,
                format!("THEME: {}  |  CARET: {}  |  SCALE: {:.0}px", theme.name, config.caret_style.display_name(), config.font_size),
                egui::FontId::monospace(10.5),
                theme.text_dim,
            );
        });
    }
}
