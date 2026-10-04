use crate::data::AppConfig;
use crate::fx::CaretStyle;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, Rect, RichText, Stroke, Vec2};

pub struct CaretPhysicsTab;

impl CaretPhysicsTab {
    pub fn show(ui: &mut egui::Ui, config: &mut AppConfig, theme: &Theme) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("CARET DYNAMICS & KINETICS")
                    .color(theme.accent)
                    .strong()
                    .monospace()
                    .size(13.0),
            );
            ui.label(
                RichText::new("Tune the responsive spring physics, bloom glow, and rendering geometry.")
                    .color(theme.text_dim)
                    .monospace()
                    .size(11.5),
            );
            ui.add_space(16.0);

            // ─────────────────────────────────────────────────────────────
            // 1. LIVE CARET PLACEHOLDER PREVIEW (Restored!)
            // ─────────────────────────────────────────────────────────────
            ui.label(
                RichText::new("LIVE CARET PREVIEW")
                    .color(theme.text_dim)
                    .size(10.5)
                    .monospace(),
            );
            ui.add_space(6.0);

            let preview_h = 70.0;
            let preview_w = ui.available_width();
            let (rect, _resp) = ui.allocate_exact_size(Vec2::new(preview_w, preview_h), egui::Sense::hover());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 8.0, theme.bg_surface);
            painter.rect_stroke(rect, 8.0, Stroke::new(1.0, theme.border));

            let sample_text = "the quick brown fox jumps";
            let font_id = egui::FontId::monospace(22.0);
            let text_pos = Pos2::new(rect.min.x + 24.0, rect.center().y - 12.0);

            // Render sample text
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                sample_text,
                font_id.clone(),
                theme.text_active,
            );

            // Measure position after "brown " to place the caret
            let prefix = "the quick brown ";
            let prefix_w = ui.fonts(|f| {
                f.layout_no_wrap(prefix.to_string(), font_id.clone(), Color32::WHITE).size().x
            });

            // Caret pulse animation
            let time = ui.ctx().input(|i| i.time);
            let alpha_pulse = ((time * 3.5).sin().abs() as f32).clamp(0.25, 1.0);
            let caret_color = Color32::from_rgba_unmultiplied(
                theme.caret.r(),
                theme.caret.g(),
                theme.caret.b(),
                (255.0 * alpha_pulse) as u8,
            );

            let caret_x = text_pos.x + prefix_w;
            let caret_y = text_pos.y;
            let char_h = 24.0;
            let char_w = 13.0;

            match config.caret_style {
                CaretStyle::Line => {
                    let w = config.caret_width;
                    let caret_rect = Rect::from_min_size(Pos2::new(caret_x, caret_y), Vec2::new(w, char_h));
                    if config.caret_glow > 0.1 {
                        let glow_color = Color32::from_rgba_unmultiplied(
                            theme.caret.r(),
                            theme.caret.g(),
                            theme.caret.b(),
                            (60.0 * config.caret_glow * alpha_pulse) as u8,
                        );
                        painter.rect_filled(caret_rect.expand(2.5 * config.caret_glow), 3.0, glow_color);
                    }
                    painter.rect_filled(caret_rect, 1.5, caret_color);
                }
                CaretStyle::Bar => {
                    let w = config.caret_width.max(2.0);
                    let caret_rect = Rect::from_min_size(Pos2::new(caret_x, caret_y), Vec2::new(w, char_h));
                    painter.rect_filled(caret_rect, 2.0, caret_color);
                }
                CaretStyle::Block => {
                    let caret_rect = Rect::from_min_size(Pos2::new(caret_x, caret_y), Vec2::new(char_w, char_h));
                    let block_color = Color32::from_rgba_unmultiplied(
                        theme.caret.r(),
                        theme.caret.g(),
                        theme.caret.b(),
                        (140.0 * alpha_pulse) as u8,
                    );
                    painter.rect_filled(caret_rect, 2.0, block_color);
                }
                CaretStyle::Underline => {
                    let caret_rect = Rect::from_min_size(
                        Pos2::new(caret_x, caret_y + char_h - 3.0),
                        Vec2::new(char_w, config.caret_width.max(2.0)),
                    );
                    painter.rect_filled(caret_rect, 1.0, caret_color);
                }
            }

            ui.add_space(20.0);

            // ─────────────────────────────────────────────────────────────
            // 2. CARET CONTROLS
            // ─────────────────────────────────────────────────────────────
            let controls_frame = egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(16.0));

            controls_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Caret Style:").color(theme.text_active).monospace().size(12.5));
                    ui.add_space(16.0);
                    for &style in CaretStyle::ALL {
                        let selected = config.caret_style == style;
                        let btn_fill = if selected { theme.border } else { Color32::TRANSPARENT };
                        let btn_text = if selected { theme.accent } else { theme.text_dim };

                        if ui.add(
                            egui::Button::new(RichText::new(style.display_name()).color(btn_text).monospace().size(12.0))
                                .fill(btn_fill)
                                .rounding(4.0),
                        ).clicked() {
                            config.caret_style = style;
                        }
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(14.0);

                // Width slider
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Caret Width:").color(theme.text_dim).monospace().size(12.0));
                    ui.add(egui::Slider::new(&mut config.caret_width, 1.0..=6.0).suffix(" px"));
                });

                ui.add_space(10.0);

                // Smoothness slider
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Spring Smoothness:").color(theme.text_dim).monospace().size(12.0));
                    ui.add(egui::Slider::new(&mut config.caret_smoothness, 0.05..=0.40).suffix(" s"));
                });

                ui.add_space(10.0);

                // Bloom glow slider
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Bloom Glow Intensity:").color(theme.text_dim).monospace().size(12.0));
                    ui.add(egui::Slider::new(&mut config.caret_glow, 0.0..=1.0));
                });
            });
        });
    }
}
