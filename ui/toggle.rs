//! Unified toggle/switch component — the single source of truth for all
//! on/off controls in the application.
//!
//! Both the standalone pill ([`UnifiedToggle::show`]) and the full settings row
//! ([`UnifiedToggle::row`]) render through [`UnifiedToggle::draw_switch`], so the
//! knob geometry, colors and animation are guaranteed to be identical everywhere.

use crate::ui::theme::Theme;
use egui::{Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

pub struct UnifiedToggle;

impl UnifiedToggle {
    /// Track (pill) width in pixels.
    pub const WIDTH: f32 = 36.0;
    /// Track (pill) height in pixels.
    pub const HEIGHT: f32 = 20.0;
    /// Inner padding between the knob and the track edge.
    const KNOB_PADDING: f32 = 2.0;
    /// Corner radius used by the surrounding setting row.
    pub const ROW_ROUNDING: f32 = 6.0;

    /// Draws the switch track + knob into `rect`. This is the single place that
    /// defines how a Velotype toggle looks.
    pub fn draw_switch(
        painter: &egui::Painter,
        rect: Rect,
        value: bool,
        hovered: bool,
        theme: &Theme,
    ) {
        let rounding = Self::HEIGHT / 2.0;
        let knob_radius = (Self::HEIGHT - Self::KNOB_PADDING * 2.0) / 2.0;
        let center_y = rect.center().y;

        let track_color = if value {
            if hovered {
                theme.accent.linear_multiply(0.9)
            } else {
                theme.accent
            }
        } else if hovered {
            Color32::from_white_alpha(32)
        } else {
            Color32::from_white_alpha(16)
        };

        painter.rect_filled(rect, rounding, track_color);
        if !value {
            painter.rect_stroke(rect, rounding, Stroke::new(1.0, theme.border));
        }

        let knob_x = if value {
            rect.max.x - Self::KNOB_PADDING - knob_radius
        } else {
            rect.min.x + Self::KNOB_PADDING + knob_radius
        };

        // Knob is identical in size/shape for both states; only the color adapts.
        let knob_color = if value {
            Color32::from_rgb(10, 14, 22)
        } else if hovered {
            theme.text_active
        } else {
            theme.text_dim
        };

        painter.circle_filled(Pos2::new(knob_x, center_y), knob_radius, knob_color);
    }

    /// Standalone switch with no label. Returns the click response.
    pub fn show(ui: &mut Ui, value: &mut bool, theme: &Theme) -> Response {
        let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(Self::WIDTH, Self::HEIGHT), Sense::click());
        resp.surrender_focus();

        if resp.clicked() {
            *value = !*value;
            resp.mark_changed();
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        Self::draw_switch(&ui.painter_at(rect), rect, *value, resp.hovered(), theme);
        resp
    }

    /// Full-width interactive setting row with title, description and switch.
    pub fn row(ui: &mut Ui, value: &mut bool, title: &str, desc: &str, theme: &Theme) -> Response {
        let row_w = ui.available_width();
        let row_h = 44.0;
        let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(row_w, row_h), Sense::click());
        resp.surrender_focus();

        if resp.clicked() {
            *value = !*value;
            resp.mark_changed();
        }

        let hovered = resp.hovered();
        if hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let painter = ui.painter_at(rect);
        if hovered {
            painter.rect_filled(rect, Self::ROW_ROUNDING, Color32::from_white_alpha(10));
        }

        // Left text block
        let text_left = rect.min.x + 8.0;
        painter.text(
            Pos2::new(text_left, rect.min.y + 13.0),
            egui::Align2::LEFT_CENTER,
            title,
            FontId::monospace(12.0),
            if hovered { theme.text_active } else { theme.text_active.linear_multiply(0.95) },
        );
        painter.text(
            Pos2::new(text_left, rect.min.y + 31.0),
            egui::Align2::LEFT_CENTER,
            desc,
            FontId::monospace(10.5),
            theme.text_dim,
        );

        // Right switch — same geometry as `show`
        let toggle_rect = Rect::from_center_size(
            Pos2::new(rect.max.x - Self::WIDTH / 2.0 - 8.0, rect.center().y),
            Vec2::new(Self::WIDTH, Self::HEIGHT),
        );
        Self::draw_switch(&painter, toggle_rect, *value, hovered, theme);

        resp
    }
}
