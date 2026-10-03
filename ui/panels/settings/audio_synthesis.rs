use crate::audio::{AudioManager, SoundPreset};
use crate::data::AppConfig;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Stroke, Vec2};

pub struct AudioSynthesisTab;

impl AudioSynthesisTab {
    pub fn show(ui: &mut egui::Ui, config: &mut AppConfig, audio: &AudioManager, theme: &Theme) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("AUDIO SYNTHESIS & ACOUSTICS")
                    .color(theme.accent)
                    .strong()
                    .monospace()
                    .size(13.0),
            );
            ui.label(
                RichText::new("Synthesized mechanical switch acoustic feedback on every keystroke.")
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

                // Master Enable Toggle
                crate::ui::components::UnifiedToggle::row(
                    ui,
                    &mut config.sound_enabled,
                    "Keystroke Audio Feedback",
                    "Synthesizes responsive mechanical switch acoustics on keystrokes",
                    theme,
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(14.0);

                // Volume Slider
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Master Volume:").color(theme.text_dim).monospace().size(12.0));
                    let mut vol_pct = (config.sound_volume * 100.0).round();
                    if ui.add(egui::Slider::new(&mut vol_pct, 0.0..=100.0).suffix(" %")).changed() {
                        config.sound_volume = (vol_pct / 100.0).clamp(0.0, 1.0);
                    }
                });

                ui.add_space(14.0);

                // Sound Preset
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Switch Acoustic Profile:").color(theme.text_dim).monospace().size(12.0));
                });
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);
                    let presets = [
                        (SoundPreset::Mechanical, "Mechanical"),
                        (SoundPreset::DeepThock, "Deep Thock"),
                        (SoundPreset::BubblePop, "Bubble Pop"),
                        (SoundPreset::Typewriter, "Typewriter"),
                        (SoundPreset::CyberBlip, "Cyber Blip"),
                    ];

                    for (preset, label) in presets {
                        let selected = config.sound_preset == preset;
                        let btn_fill = if selected { theme.border } else { Color32::TRANSPARENT };
                        let btn_color = if selected { theme.accent } else { theme.text_dim };

                        if ui.add(
                            egui::Button::new(RichText::new(label).color(btn_color).monospace().size(11.5))
                                .fill(btn_fill)
                                .rounding(4.0),
                        ).clicked() {
                            config.sound_preset = preset;
                            audio.play_click();
                        }
                    }
                });

                ui.add_space(16.0);

                // Test Keystroke Button
                ui.horizontal(|ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("♫  Test Keystroke Click").color(theme.text_active).monospace().size(12.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .rounding(4.0)
                            .min_size(Vec2::new(180.0, 28.0)),
                    ).clicked() {
                        audio.play_click();
                    }
                });
            });
        });
    }
}
