use crate::data::AppConfig;
use crate::ui::theme::{Theme, ThemeId};
use egui::{Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};

pub struct ThemeTab;

impl ThemeTab {
    pub fn show(ui: &mut egui::Ui, config: &mut AppConfig, current_theme: &Theme) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("AESTHETIC THEMES")
                    .color(current_theme.accent)
                    .strong()
                    .monospace()
                    .size(13.0),
            );
            ui.label(
                RichText::new("Select a curated color palette for your typing workspace.")
                    .color(current_theme.text_dim)
                    .monospace()
                    .size(11.5),
            );
            ui.add_space(14.0);

            let available_w = ui.available_width();
            let col_count = 2;
            let spacing = 12.0;
            let card_w = (available_w - spacing) / col_count as f32;
            let card_h = 56.0;

            let total_themes = ThemeId::ALL.len();
            let row_count = (total_themes + col_count - 1) / col_count;

            for row in 0..row_count {
                ui.horizontal(|ui| {
                    for col in 0..col_count {
                        let idx = row * col_count + col;
                        if idx < total_themes {
                            let tid = ThemeId::ALL[idx];
                            let t = Theme::get(tid);
                            let is_selected = config.theme == tid;

                            if Self::theme_card(ui, &t, is_selected, card_w, card_h) {
                                config.theme = tid;
                            }

                            if col < col_count - 1 {
                                ui.add_space(spacing);
                            }
                        }
                    }
                });
                ui.add_space(10.0);
            }
        });
    }

    fn theme_card(
        ui: &mut egui::Ui,
        theme: &Theme,
        selected: bool,
        width: f32,
        height: f32,
    ) -> bool {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        let border_color = if selected {
            theme.accent
        } else if response.hovered() {
            theme.text_dim
        } else {
            theme.border
        };

        let painter = ui.painter_at(rect);

        // Card container background
        painter.rect_filled(rect, 8.0, theme.bg_surface);
        painter.rect_stroke(rect, 8.0, Stroke::new(if selected { 1.5 } else { 1.0 }, border_color));

        // Color swatches (bg, accent, text_correct, text_active)
        let swatches = [theme.bg, theme.accent, theme.text_correct, theme.text_active];
        let swatch_size = 14.0;
        let swatch_spacing = 6.0;
        let swatches_x = rect.min.x + 14.0;
        let swatches_y = rect.center().y - swatch_size / 2.0;

        for (i, &color) in swatches.iter().enumerate() {
            let s_rect = Rect::from_min_size(
                Pos2::new(swatches_x + i as f32 * (swatch_size + swatch_spacing), swatches_y),
                Vec2::splat(swatch_size),
            );
            painter.rect_filled(s_rect, 3.0, color);
            painter.rect_stroke(s_rect, 3.0, Stroke::new(0.5, Color32::from_black_alpha(80)));
        }

        // Theme name label
        let name_x = swatches_x + swatches.len() as f32 * (swatch_size + swatch_spacing) + 12.0;
        painter.text(
            Pos2::new(name_x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            theme.name,
            egui::FontId::monospace(12.5),
            if selected { theme.text_active } else { theme.text_dim },
        );

        // Active indicator badge
        if selected {
            let badge_text = "ACTIVE";
            let badge_pos = Pos2::new(rect.max.x - 14.0, rect.center().y);
            painter.text(
                badge_pos,
                egui::Align2::RIGHT_CENTER,
                badge_text,
                egui::FontId::monospace(10.0),
                theme.accent,
            );
        }

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        response.clicked()
    }
}
