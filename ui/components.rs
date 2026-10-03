use crate::ui::theme::Theme;
use egui::{Color32, FontId, Frame, Margin, Pos2, Rect, Response, RichText, Stroke, Ui, Vec2};

pub use crate::ui::velocity_graph::VelocityGraphWidget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,   // Accent fill, dark text
    Secondary, // Neutral surface fill, theme border, active text
    Danger,    // Transparent fill, soft red border & text
    Ghost,     // Transparent fill, no border, dim text
}

pub struct UnifiedButton;

impl UnifiedButton {
    pub const HEIGHT: f32 = 32.0;
    pub const ROUNDING: f32 = 5.0;
    pub const FONT_SIZE: f32 = 11.5;

    pub fn show(
        ui: &mut Ui,
        text: &str,
        variant: ButtonVariant,
        theme: &Theme,
        min_width: f32,
    ) -> Response {
        let (fill_col, stroke, text_col, strong) = match variant {
            ButtonVariant::Primary => (
                theme.accent,
                Stroke::NONE,
                Color32::from_rgb(10, 14, 22),
                true,
            ),
            ButtonVariant::Secondary => (
                theme.bg_surface,
                Stroke::new(1.0, theme.border),
                theme.text_active,
                false,
            ),
            ButtonVariant::Danger => (
                Color32::TRANSPARENT,
                Stroke::new(1.0, Color32::from_rgb(180, 50, 50).linear_multiply(0.6)),
                Color32::from_rgb(255, 92, 92),
                false,
            ),
            ButtonVariant::Ghost => (
                Color32::TRANSPARENT,
                Stroke::NONE,
                theme.text_dim,
                false,
            ),
        };

        let mut rich = RichText::new(text)
            .color(text_col)
            .monospace()
            .size(Self::FONT_SIZE);
        if strong {
            rich = rich.strong();
        }

        let btn = egui::Button::new(rich)
            .fill(fill_col)
            .stroke(stroke)
            .rounding(Self::ROUNDING)
            .min_size(Vec2::new(min_width, Self::HEIGHT));

        ui.add(btn)
    }
}

pub struct UnifiedInput;

impl UnifiedInput {
    pub const ROUNDING: f32 = 6.0;
    pub const FONT_SIZE: f32 = 13.5;

    pub fn singleline(
        ui: &mut Ui,
        text: &mut String,
        hint: &str,
        theme: &Theme,
        width: f32,
    ) -> Response {
        let frame = Frame::none()
            .fill(theme.bg)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Self::ROUNDING)
            .inner_margin(Margin::symmetric(14.0, 8.0));

        frame
            .show(ui, |ui| {
                ui.set_width(width);
                let edit = egui::TextEdit::singleline(text)
                    .hint_text(RichText::new(hint).color(theme.text_dim).monospace().size(13.0))
                    .text_color(theme.text_active)
                    .font(FontId::monospace(Self::FONT_SIZE))
                    .frame(false)
                    .desired_width(width);
                ui.add(edit)
            })
            .inner
    }

    pub fn password(
        ui: &mut Ui,
        text: &mut String,
        hint: &str,
        theme: &Theme,
        width: f32,
    ) -> Response {
        let frame = Frame::none()
            .fill(theme.bg)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Self::ROUNDING)
            .inner_margin(Margin::symmetric(14.0, 8.0));

        frame
            .show(ui, |ui| {
                ui.set_width(width);
                let edit = egui::TextEdit::singleline(text)
                    .password(true)
                    .hint_text(RichText::new(hint).color(theme.text_dim).monospace().size(13.0))
                    .text_color(theme.text_active)
                    .font(FontId::monospace(Self::FONT_SIZE))
                    .frame(false)
                    .desired_width(width);
                ui.add(edit)
            })
            .inner
    }

    pub fn multiline(
        ui: &mut Ui,
        text: &mut String,
        hint: &str,
        theme: &Theme,
        width: f32,
        rows: usize,
    ) -> Response {
        let frame = Frame::none()
            .fill(theme.bg)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Self::ROUNDING)
            .inner_margin(Margin::symmetric(14.0, 10.0));

        frame
            .show(ui, |ui| {
                ui.set_width(width);
                let edit = egui::TextEdit::multiline(text)
                    .hint_text(RichText::new(hint).color(theme.text_dim).monospace().size(13.0))
                    .text_color(theme.text_active)
                    .font(FontId::monospace(Self::FONT_SIZE))
                    .frame(false)
                    .desired_rows(rows)
                    .desired_width(width);
                ui.add(edit)
            })
            .inner
    }
}

pub struct UnifiedToggle;

impl UnifiedToggle {
    pub const WIDTH: f32 = 36.0;
    pub const HEIGHT: f32 = 20.0;

    /// Renders a standalone sleek pill toggle switch
    pub fn show(ui: &mut Ui, value: &mut bool, theme: &Theme) -> Response {
        let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(Self::WIDTH, Self::HEIGHT), egui::Sense::click());
        resp.surrender_focus();

        if resp.clicked() {
            *value = !*value;
            resp.mark_changed();
        }

        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let painter = ui.painter_at(rect);
        let rounding = Self::HEIGHT / 2.0;
        let knob_radius = (Self::HEIGHT - 4.0) / 2.0;
        let center_y = rect.center().y;

        let bg_color = if *value {
            if resp.hovered() {
                theme.accent.linear_multiply(0.9)
            } else {
                theme.accent
            }
        } else {
            if resp.hovered() {
                Color32::from_white_alpha(32)
            } else {
                Color32::from_white_alpha(16)
            }
        };

        painter.rect_filled(rect, rounding, bg_color);
        if !*value {
            painter.rect_stroke(rect, rounding, Stroke::new(1.0, theme.border));
        }

        let knob_x = if *value {
            rect.max.x - 2.0 - knob_radius
        } else {
            rect.min.x + 2.0 + knob_radius
        };

        let knob_color = if *value {
            Color32::from_rgb(10, 14, 22)
        } else {
            if resp.hovered() { theme.text_active } else { theme.text_dim }
        };

        painter.circle_filled(Pos2::new(knob_x, center_y), knob_radius, knob_color);

        resp
    }

    /// Renders a full-width interactive setting row with title, description, and toggle
    pub fn row(
        ui: &mut Ui,
        value: &mut bool,
        title: &str,
        desc: &str,
        theme: &Theme,
    ) -> Response {
        let row_w = ui.available_width();
        let row_h = 44.0;
        let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(row_w, row_h), egui::Sense::click());
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
            painter.rect_filled(rect, 6.0, Color32::from_white_alpha(10));
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

        // Right toggle switch
        let toggle_rect = Rect::from_center_size(
            Pos2::new(rect.max.x - Self::WIDTH / 2.0 - 8.0, rect.center().y),
            Vec2::new(Self::WIDTH, Self::HEIGHT),
        );

        let rounding = Self::HEIGHT / 2.0;
        let knob_radius = (Self::HEIGHT - 4.0) / 2.0;
        let center_y = toggle_rect.center().y;

        let bg_color = if *value {
            if hovered {
                theme.accent.linear_multiply(0.9)
            } else {
                theme.accent
            }
        } else {
            if hovered {
                Color32::from_white_alpha(32)
            } else {
                Color32::from_white_alpha(16)
            }
        };

        painter.rect_filled(toggle_rect, rounding, bg_color);
        if !*value {
            painter.rect_stroke(toggle_rect, rounding, Stroke::new(1.0, theme.border));
        }

        let knob_x = if *value {
            toggle_rect.max.x - 2.0 - knob_radius
        } else {
            toggle_rect.min.x + 2.0 + knob_radius
        };

        let knob_color = if *value {
            Color32::from_rgb(10, 14, 22)
        } else {
            if hovered { theme.text_active } else { theme.text_dim }
        };

        painter.circle_filled(Pos2::new(knob_x, center_y), knob_radius, knob_color);

        resp
    }
}
