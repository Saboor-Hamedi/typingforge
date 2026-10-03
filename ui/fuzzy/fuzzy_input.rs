use crate::ui::theme::Theme;
use egui::{FontId, Pos2, Rect, Sense, Stroke, Vec2};

pub struct FuzzyInput;

impl FuzzyInput {
    pub fn show(
        ui: &mut egui::Ui,
        query: &mut String,
        theme: &Theme,
        request_focus: bool,
    ) -> bool {
        let mut text_changed = false;
        // Generous vertical padding for a spacious, minimal macOS Spotlight feel
        let height = 52.0;
        let width = ui.available_width();

        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let painter = ui.painter_at(rect);

        // Borderless: NO inner box border or heavy fill. Text floats cleanly in modal.

        // ─────────────────────────────────────────────────────────────────
        // Sleek Vector Magnifying Glass Icon (Subtle & aligned)
        // ─────────────────────────────────────────────────────────────────
        let icon_center = Pos2::new(rect.min.x + 24.0, rect.center().y - 1.0);
        let glass_radius = 6.0;
        let icon_color = theme.accent.linear_multiply(0.9);
        painter.circle_stroke(icon_center, glass_radius, Stroke::new(1.6, icon_color));
        let handle_start = icon_center + Vec2::new(glass_radius * 0.707, glass_radius * 0.707);
        let handle_end = handle_start + Vec2::new(4.5, 4.5);
        painter.line_segment([handle_start, handle_end], Stroke::new(1.8, icon_color));

        // ─────────────────────────────────────────────────────────────────
        // Vertically Centered Search Input Field (Generous padding, floating)
        // ─────────────────────────────────────────────────────────────────
        let text_h = 28.0;
        let text_y = rect.center().y - text_h / 2.0;
        let text_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 50.0, text_y),
            Pos2::new(rect.max.x - 44.0, text_y + text_h),
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
                .hint_text("Search passages by title or content...")
                .font(FontId::proportional(15.5))
                .text_color(theme.text_active)
                .frame(false),
        );

        if request_focus {
            response.request_focus();
        }

        if response.changed() {
            text_changed = true;
        }

        // Clear "✕" button when query is not empty
        if !query.is_empty() {
            let clear_rect = Rect::from_center_size(
                Pos2::new(rect.max.x - 24.0, rect.center().y),
                Vec2::new(22.0, 22.0),
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
        }

        text_changed
    }
}
