use crate::ui::theme::Theme;
use egui::{Color32, FontId, Frame, Margin, Response, RichText, Stroke, Ui, Vec2};

pub use crate::ui::toggle::UnifiedToggle;
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
    pub const ROUNDING: f32 = crate::ui::style::RADIUS_CONTROL;
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


