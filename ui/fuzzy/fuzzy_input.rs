use crate::ui::theme::Theme;
use egui::{Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

pub struct FuzzyInput;

impl FuzzyInput {
    pub fn show(
        ui: &mut egui::Ui,
        query: &mut String,
        theme: &Theme,
        request_focus: bool,
    ) -> bool {
        let mut text_changed = false;
        // Sleek, compact command-bar size (48px)
        let height = 48.0;
        let width = ui.available_width();

        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let painter = ui.painter_at(rect);

        // Right side indicators: Magnifying glass and/or Ctrl+P badge, plus clear button when query not empty
        let right_padding = if !query.is_empty() { 68.0 } else { 88.0 };

        // ─────────────────────────────────────────────────────────────────
        // Vertically Centered Search Input Field (Sleek command bar)
        // ─────────────────────────────────────────────────────────────────
        let text_h = 24.0;
        let text_y = rect.center().y - text_h / 2.0;
        let text_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 16.0, text_y),
            Pos2::new(rect.max.x - right_padding, text_y + text_h),
        );

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(text_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let id = ui.id().with("fuzzy_palette_input");

        let response = child_ui.add_sized(
            text_rect.size(),
            egui::TextEdit::singleline(query)
                .id(id)
                .hint_text(
                    RichText::new("Search passages...")
                        .color(Color32::from_white_alpha(75))
                        .italics(),
                )
                .font(FontId::proportional(15.0))
                .text_color(theme.text_active)
                .margin(Vec2::ZERO)
                .frame(false),
        );

        if request_focus {
            response.request_focus();
        }

        if response.changed() {
            text_changed = true;
        }

        // ─────────────────────────────────────────────────────────────────
        // Right side: Subtle, muted magnifying glass icon and Ctrl+P hint
        // ─────────────────────────────────────────────────────────────────
        if query.is_empty() {
            // Subtle keyboard shortcut hint "Ctrl P"
            let badge_rect = Rect::from_center_size(
                Pos2::new(rect.max.x - 52.0, rect.center().y),
                Vec2::new(44.0, 20.0),
            );
            painter.rect_stroke(badge_rect, 4.0, Stroke::new(1.0, Color32::from_white_alpha(22)));
            painter.text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Ctrl P",
                FontId::monospace(10.0),
                theme.text_dim.linear_multiply(0.7),
            );

            // Subtle magnifying glass icon on the far right
            let icon_center = Pos2::new(rect.max.x - 16.0, rect.center().y - 1.0);
            let glass_radius = 5.0;
            let icon_color = theme.text_dim.linear_multiply(0.65);
            painter.circle_stroke(icon_center, glass_radius, Stroke::new(1.3, icon_color));
            let handle_start = icon_center + Vec2::new(glass_radius * 0.707, glass_radius * 0.707);
            let handle_end = handle_start + Vec2::new(3.5, 3.5);
            painter.line_segment([handle_start, handle_end], Stroke::new(1.4, icon_color));
        } else {
            // Clear "✕" button
            let clear_rect = Rect::from_center_size(
                Pos2::new(rect.max.x - 42.0, rect.center().y),
                Vec2::new(20.0, 20.0),
            );
            let clear_resp = ui.interact(clear_rect, ui.id().with("fuzzy_clear_btn"), Sense::click());
            if clear_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if clear_resp.clicked() {
                query.clear();
                text_changed = true;
            }
            let clear_col = if clear_resp.hovered() { theme.accent } else { theme.text_dim };
            painter.text(
                clear_rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                FontId::monospace(12.0),
                clear_col,
            );

            // Subtle magnifying glass icon on the far right
            let icon_center = Pos2::new(rect.max.x - 16.0, rect.center().y - 1.0);
            let glass_radius = 5.0;
            let icon_color = theme.text_dim.linear_multiply(0.65);
            painter.circle_stroke(icon_center, glass_radius, Stroke::new(1.3, icon_color));
            let handle_start = icon_center + Vec2::new(glass_radius * 0.707, glass_radius * 0.707);
            let handle_end = handle_start + Vec2::new(3.5, 3.5);
            painter.line_segment([handle_start, handle_end], Stroke::new(1.4, icon_color));
        }

        text_changed
    }
}
