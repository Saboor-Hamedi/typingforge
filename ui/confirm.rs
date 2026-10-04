use crate::ui::theme::Theme;
use egui::{Color32, Frame, Id, Key, Order, Pos2, Rect, RichText, Sense, Stroke, Vec2};

pub struct ConfirmModal;

impl ConfirmModal {
    /// Shows a modal confirmation dialog overlay centered on screen.
    /// Returns:
    /// - `Some(true)` if user clicked "OK" / Confirm
    /// - `Some(false)` if user clicked "Cancel", pressed Escape, or clicked backdrop
    /// - `None` if the modal is still open waiting for user choice
    pub fn show(
        ctx: &egui::Context,
        theme: &Theme,
        title: &str,
        username: &str,
        warning_body: &str,
        confirm_btn_text: &str,
    ) -> Option<bool> {
        let screen_rect = ctx.screen_rect();
        let mut result = None;

        // Escape cancels modal
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            return Some(false);
        }

        egui::Area::new(Id::new("confirm_modal_overlay"))
            .order(Order::Foreground)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                // 1. Semi-transparent dark backdrop
                let (backdrop_rect, backdrop_response) =
                    ui.allocate_exact_size(screen_rect.size(), Sense::click());
                ui.painter().rect_filled(
                    backdrop_rect,
                    0.0,
                    Color32::from_black_alpha(190),
                );

                if backdrop_response.clicked() {
                    result = Some(false);
                }

                // 2. Centered sleek modal window
                let modal_width = 440.0;
                let modal_pos = Pos2::new(
                    screen_rect.center().x - modal_width / 2.0,
                    screen_rect.center().y - 130.0,
                );

                ui.allocate_new_ui(
                    egui::UiBuilder::new().max_rect(Rect::from_min_size(modal_pos, Vec2::new(modal_width, 260.0))),
                    |ui| {
                        Frame::none()
                            .fill(theme.bg_surface)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(220, 70, 70).linear_multiply(0.7)))
                            .rounding(10.0)
                            .inner_margin(egui::Margin::symmetric(24.0, 22.0))
                            .shadow(egui::epaint::Shadow {
                                offset: Vec2::new(0.0, 8.0),
                                blur: 24.0,
                                spread: 0.0,
                                color: Color32::from_black_alpha(210),
                            })
                            .show(ui, |ui| {
                                ui.set_width(modal_width - 48.0);

                                ui.vertical_centered(|ui| {
                                    // Warning Title
                                    ui.label(
                                        RichText::new(format!("⚠  {title}"))
                                            .color(Color32::from_rgb(255, 92, 92))
                                            .strong()
                                            .monospace()
                                            .size(13.0),
                                    );
                                    ui.add_space(12.0);

                                    // Highlighted User Name Chip
                                    Frame::none()
                                        .fill(theme.bg)
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .rounding(6.0)
                                        .inner_margin(egui::Margin::symmetric(14.0, 6.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new("USER:")
                                                        .color(theme.text_dim)
                                                        .monospace()
                                                        .size(11.0),
                                                );
                                                ui.label(
                                                    RichText::new(username)
                                                        .color(theme.accent)
                                                        .strong()
                                                        .monospace()
                                                        .size(13.5),
                                                );
                                            });
                                        });

                                    ui.add_space(12.0);

                                    // Warning Description
                                    ui.label(
                                        RichText::new(warning_body)
                                            .color(theme.text_dim)
                                            .monospace()
                                            .size(11.5),
                                    );

                                    ui.add_space(20.0);

                                    // Centered Action Buttons
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);

                                        let cancel_btn = egui::Button::new(
                                            RichText::new("Cancel")
                                                .color(theme.text_active)
                                                .monospace()
                                                .size(11.5),
                                        )
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .rounding(6.0)
                                        .min_size(Vec2::new(100.0, 30.0));

                                        if ui.add(cancel_btn).clicked() {
                                            result = Some(false);
                                        }

                                        let confirm_btn = egui::Button::new(
                                            RichText::new(confirm_btn_text)
                                                .color(Color32::WHITE)
                                                .strong()
                                                .monospace()
                                                .size(11.5),
                                        )
                                        .fill(Color32::from_rgb(210, 50, 50))
                                        .rounding(6.0)
                                        .min_size(Vec2::new(140.0, 30.0));

                                        if ui.add(confirm_btn).clicked() {
                                            result = Some(true);
                                        }
                                    });
                                });
                            });
                    },
                );
            });

        result
    }
}
