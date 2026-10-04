use crate::data::AppConfig;
use crate::ui::components::UnifiedToggle;
use crate::ui::theme::Theme;
use egui::{RichText, Stroke};

pub struct MotionHudTab;

impl MotionHudTab {
    pub fn show(ui: &mut egui::Ui, config: &mut AppConfig, theme: &Theme) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("MOTION, PARTICLES & HUD TELEMETRY")
                    .color(theme.accent)
                    .strong()
                    .monospace()
                    .size(13.0),
            );
            ui.label(
                RichText::new("Configure HUD telemetry overlay and real-time kinetic effects.")
                    .color(theme.text_dim)
                    .monospace()
                    .size(11.5),
            );
            ui.add_space(16.0);

            let card_w = ui.available_width();
            let pad = 20.0;
            let inner_w = card_w - pad * 2.0;

            let frame = egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(pad));

            frame.show(ui, |ui| {
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);

                UnifiedToggle::row(
                    ui,
                    &mut config.reduced_motion,
                    "Reduced Motion",
                    "Minimal-motion mode: disables particles, screen shake and kinetic caret glide",
                    theme,
                );

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                UnifiedToggle::row(
                    ui,
                    &mut config.particles_enabled,
                    "Keypress Particle Sparks",
                    "Emit glowing kinetic micro-particles on stroke impacts",
                    theme,
                );

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                UnifiedToggle::row(
                    ui,
                    &mut config.screen_shake_enabled,
                    "Haptic Screen Shake",
                    "Gentle physical frame impulse and impact rumble on error",
                    theme,
                );

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                UnifiedToggle::row(
                    ui,
                    &mut config.show_live_wpm,
                    "Live Velocity Counter",
                    "Smoothed EMA speed and accuracy telemetry while typing",
                    theme,
                );

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                UnifiedToggle::row(
                    ui,
                    &mut config.show_velocity_graph,
                    "Velocity Telemetry Graph",
                    "Real-time kinetic velocity waveform in the footer bar",
                    theme,
                );
            });

            ui.add_space(16.0);

            ui.label(
                RichText::new("GENERATED PRACTICE TEXT")
                    .color(theme.accent)
                    .strong()
                    .monospace()
                    .size(12.0),
            );
            ui.add_space(6.0);

            let frame = egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(pad));

            frame.show(ui, |ui| {
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);

                UnifiedToggle::row(
                    ui,
                    &mut config.include_punctuation,
                    "Include Punctuation",
                    "Add punctuation marks (. , ; ! ? : -) to non-passage generated drills",
                    theme,
                );

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                UnifiedToggle::row(
                    ui,
                    &mut config.include_numbers,
                    "Include Numbers",
                    "Mix numeric tokens into non-passage generated drills",
                    theme,
                );
            });
        });
    }
}
