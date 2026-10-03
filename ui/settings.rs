use crate::audio::{AudioManager, SoundPreset};
use crate::fx::CaretStyle;
use crate::storage::AppConfig;
use crate::ui::theme::{Theme, ThemeId};
use egui::{Color32, Pos2, Sense, Vec2};

pub struct SettingsScreen;

impl SettingsScreen {
    pub fn show(
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        theme: &Theme,
        audio: &AudioManager,
        _is_open: &mut bool,
    ) {
        ui.vertical(|ui| {
            ui.add_space(10.0);

            // Full-window Vertical Scroll Area
            let max_h = ui.available_height();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(max_h)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let available_w = ui.available_width();

                        // ─────────────────────────────────────────────────────────────
                        // 1. THEME GALLERY (Spacious 2-column grid, no cramped text)
                        // ─────────────────────────────────────────────────────────────
                        ui.label(
                            egui::RichText::new("AESTHETIC THEMES")
                                .color(theme.accent)
                                .strong()
                                .monospace(),
                        );
                        ui.add_space(8.0);

                        let col_count = 2;
                        let spacing = 12.0;
                        let card_w = (available_w - spacing) / col_count as f32;
                        let card_h = 52.0;

                        let total_themes = ThemeId::ALL.len();
                        let row_count = (total_themes + col_count - 1) / col_count;

                        for row in 0..row_count {
                            ui.horizontal(|ui| {
                                for col in 0..col_count {
                                    let idx = row * col_count + col;
                                    if idx < total_themes {
                                        let tid = ThemeId::ALL[idx];
                                        let t = Theme::get(tid);
                                        let selected = config.theme == tid;

                                        if Self::theme_card(ui, &t, selected, card_w, card_h) {
                                            config.theme = tid;
                                        }

                                        if col < col_count - 1 {
                                            ui.add_space(spacing);
                                        }
                                    }
                                }
                            });
                            ui.add_space(8.0);
                        }

                        ui.add_space(20.0);

                        // ─────────────────────────────────────────────────────────────
                        // 2. CARET & SPRING DYNAMICS (Flat, clean controls)
                        // ─────────────────────────────────────────────────────────────
                        ui.label(
                            egui::RichText::new("CARET & SPRING PHYSICS")
                                .color(theme.accent)
                                .strong()
                                .monospace(),
                        );
                        ui.add_space(8.0);

                        // Caret style picker row
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Style:").color(theme.text_active).monospace());
                            ui.add_space(8.0);
                            for style in CaretStyle::ALL {
                                let selected = config.caret_style == *style;
                                let btn = egui::Button::new(
                                    egui::RichText::new(style.display_name())
                                        .color(if selected { theme.accent } else { theme.text_dim })
                                        .monospace(),
                                )
                                .fill(if selected { theme.bg_surface_hover } else { Color32::TRANSPARENT })
                                .rounding(6.0);

                                if ui.add(btn).clicked() {
                                    config.caret_style = *style;
                                }
                            }
                        });
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Width:").color(theme.text_active).monospace());
                            ui.add(egui::Slider::new(&mut config.caret_width, 1.0..=10.0).suffix("px"));
                        });
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Spring Smoothness:").color(theme.text_active).monospace());
                            ui.add(egui::Slider::new(&mut config.caret_smoothness, 0.05..=0.35));
                        });
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Bloom Glow:").color(theme.text_active).monospace());
                            ui.add(egui::Slider::new(&mut config.caret_glow, 0.0..=1.0));
                        });

                        ui.add_space(20.0);

                        // ─────────────────────────────────────────────────────────────
                        // 3. TACTILE AUDIO SYNTHESIS
                        // ─────────────────────────────────────────────────────────────
                        ui.label(
                            egui::RichText::new("KEYSTROKE AUDIO SYNTHESIS")
                                .color(theme.accent)
                                .strong()
                                .monospace(),
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            Self::toggle_switch(ui, &mut config.sound_enabled, theme);
                            ui.label(egui::RichText::new("Enable Audio").color(theme.text_active).monospace());

                            if config.sound_enabled {
                                ui.add_space(16.0);
                                if ui.button(egui::RichText::new("▶ Test Audio").monospace()).clicked() {
                                    audio.play_click();
                                }
                            }
                        });

                        if config.sound_enabled {
                            ui.add_space(10.0);
                            ui.horizontal_wrapped(|ui| {
                                for preset in SoundPreset::ALL {
                                    let selected = config.sound_preset == *preset;
                                    let btn = egui::Button::new(
                                        egui::RichText::new(preset.display_name())
                                            .color(if selected { theme.accent } else { theme.text_dim })
                                            .monospace(),
                                    )
                                    .fill(if selected { theme.bg_surface_hover } else { Color32::TRANSPARENT })
                                    .rounding(6.0);

                                    if ui.add(btn).clicked() {
                                        config.sound_preset = *preset;
                                        audio.play_click();
                                    }
                                }
                            });

                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Volume:").color(theme.text_active).monospace());
                                ui.add(egui::Slider::new(&mut config.sound_volume, 0.0..=1.0));
                            });
                        }

                        ui.add_space(20.0);

                        // ─────────────────────────────────────────────────────────────
                        // 4. MOTION DYNAMICS & HUD
                        // ─────────────────────────────────────────────────────────────
                        ui.label(
                            egui::RichText::new("MOTION DYNAMICS & HUD")
                                .color(theme.accent)
                                .strong()
                                .monospace(),
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            Self::toggle_switch(ui, &mut config.show_live_wpm, theme);
                            ui.label(egui::RichText::new("Show Live Velocity HUD in Typing View").color(theme.text_active).monospace());
                        });
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            Self::toggle_switch(ui, &mut config.show_velocity_graph, theme);
                            ui.label(egui::RichText::new("Show Velocity Telemetry Waveform Graph").color(theme.text_active).monospace());
                        });
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            Self::toggle_switch(ui, &mut config.particles_enabled, theme);
                            ui.label(egui::RichText::new("Keystroke Particle Sparks").color(theme.text_active).monospace());
                        });
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            Self::toggle_switch(ui, &mut config.screen_shake_enabled, theme);
                            ui.label(egui::RichText::new("Screen Shake Impulse on Typing Errors").color(theme.text_active).monospace());
                        });

                        ui.add_space(20.0);

                        // ─────────────────────────────────────────────────────────────
                        // 5. TYPOGRAPHY
                        // ─────────────────────────────────────────────────────────────
                        ui.label(
                            egui::RichText::new("TYPOGRAPHIC SCALE")
                                .color(theme.accent)
                                .strong()
                                .monospace(),
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Editor Font Size:").color(theme.text_active).monospace());
                            ui.add(egui::Slider::new(&mut config.font_size, 20.0..=36.0).suffix("px"));
                        });

                        ui.add_space(24.0);
                    });
                });
        });
    }

    /// Flat, borderless Theme Card with generous spacing
    fn theme_card(ui: &mut egui::Ui, theme_item: &Theme, selected: bool, width: f32, height: f32) -> bool {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        let p = ui.painter_at(rect);

        // Flat clean styling with no harsh borders or top edge lines
        let bg_color = if selected {
            theme_item.bg_surface_hover
        } else if response.hovered() {
            theme_item.bg_surface
        } else {
            Color32::from_rgba_premultiplied(
                theme_item.bg_surface.r(),
                theme_item.bg_surface.g(),
                theme_item.bg_surface.b(),
                80,
            )
        };

        p.rect_filled(rect, 8.0, bg_color);

        // Theme Name
        let name_col = if selected {
            theme_item.accent
        } else {
            theme_item.text_active
        };

        p.text(
            Pos2::new(rect.min.x + 16.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            theme_item.name,
            egui::FontId::monospace(13.0),
            name_col,
        );

        // Preview Palette Circles: [Canvas, Surface, Accent, Caret]
        let dot_y = rect.center().y;
        let r = 5.5;
        let right_x = if selected { rect.max.x - 70.0 } else { rect.max.x - 16.0 };

        let dot4 = Pos2::new(right_x, dot_y);
        let dot3 = Pos2::new(right_x - 15.0, dot_y);
        let dot2 = Pos2::new(right_x - 30.0, dot_y);
        let dot1 = Pos2::new(right_x - 45.0, dot_y);

        p.circle_filled(dot1, r, theme_item.bg);
        p.circle_filled(dot2, r, theme_item.bg_surface);
        p.circle_filled(dot3, r, theme_item.accent);
        p.circle_filled(dot4, r, theme_item.caret);

        // Active indicator tag
        if selected {
            p.text(
                Pos2::new(rect.max.x - 14.0, dot_y),
                egui::Align2::RIGHT_CENTER,
                "✓ ACTIVE",
                egui::FontId::monospace(10.0),
                theme_item.accent,
            );
        }

        response.clicked()
    }

    /// Animated sliding pill toggle switch
    pub fn toggle_switch(ui: &mut egui::Ui, value: &mut bool, theme: &Theme) -> egui::Response {
        let desired_size = Vec2::new(36.0, 18.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *value = !*value;
            response.mark_changed();
        }

        let how_on = ui.ctx().animate_bool(response.id, *value);

        let track_color = if *value {
            theme.accent
        } else {
            theme.border
        };

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 9.0, track_color);

        let knob_x = egui::lerp((rect.min.x + 9.0)..=(rect.max.x - 9.0), how_on);
        let knob_center = Pos2::new(knob_x, rect.center().y);
        painter.circle_filled(knob_center, 6.5, Color32::WHITE);

        response
    }
}
