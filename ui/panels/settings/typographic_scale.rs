use crate::data::AppConfig;
use crate::ui::theme::Theme;
use egui::{Pos2, RichText, Stroke, Vec2};

pub struct TypographicScaleTab;

impl TypographicScaleTab {
    pub fn show(ui: &mut egui::Ui, config: &mut AppConfig, theme: &Theme) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("TYPOGRAPHIC SCALE & READABILITY")
                    .color(theme.accent)
                    .strong()
                    .monospace()
                    .size(13.0),
            );
            ui.label(
                RichText::new("Adjust base monospace font scale and reading comfort.")
                    .color(theme.text_dim)
                    .monospace()
                    .size(11.5),
            );
            ui.add_space(16.0);

            let frame = egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(16.0));

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Font Size Scale:").color(theme.text_dim).monospace().size(12.0));
                    ui.add(egui::Slider::new(&mut config.font_size, 18.0..=34.0).suffix(" px"));
                });

                ui.add_space(20.0);
                ui.label(RichText::new("TYPOGRAPHY PREVIEW").color(theme.text_dim).monospace().size(10.5));
                ui.add_space(8.0);

                let (rect, _resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 80.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 6.0, theme.bg);
                painter.rect_stroke(rect, 6.0, Stroke::new(1.0, theme.border));

                painter.text(
                    Pos2::new(rect.min.x + 16.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Sphinx of black quartz, judge my vow.",
                    egui::FontId::monospace(config.font_size),
                    theme.text_active,
                );
            });
        });
    }
}
