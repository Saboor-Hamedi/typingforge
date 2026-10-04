use crate::ui::theme::Theme;
use crate::ui::updater::{AppUpdater, UpdateStatus};
use egui::{Color32, Frame, Margin, RichText, Stroke};

pub struct UpdatesTab;

impl UpdatesTab {
    pub fn show(ui: &mut egui::Ui, theme: &Theme, updater: &AppUpdater) {
        let card_w = 480.0;
        let pad = 20.0;
        let inner_w = card_w - pad * 2.0;

        let status = updater.get_status();
        let current_version = &updater.current_version;

        ui.vertical_centered(|ui| {
            // Card 1: Update Status & Details
            Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(10.0)
                .inner_margin(Margin::same(pad))
                .show(ui, |ui| {
                    ui.set_min_width(inner_w);
                    ui.set_max_width(inner_w);

                    ui.label(
                        RichText::new("APPLICATION UPDATES")
                            .color(theme.accent)
                            .strong()
                            .monospace()
                            .size(14.0),
                    );

                    ui.add_space(4.0);

                    ui.label(
                        RichText::new(format!("Installed Version: v{current_version}"))
                            .color(theme.text_active)
                            .monospace()
                            .size(12.5),
                    );

                    ui.label(
                        RichText::new("Channel: Official GitHub Releases (Stable)")
                            .color(theme.text_dim)
                            .monospace()
                            .size(11.0),
                    );

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Dynamic Status Display
                    match status {
                        UpdateStatus::Idle => {
                            ui.label(
                                RichText::new("Velotype automatically checks for updates in the background. You can also manually trigger a check below.")
                                    .color(theme.text_dim)
                                    .size(11.5),
                            );
                        }
                        UpdateStatus::Checking => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new("Checking GitHub for the latest release payload...")
                                        .color(theme.text_active)
                                        .monospace()
                                        .size(12.0),
                                );
                            });
                        }
                        UpdateStatus::UpToDate => {
                            ui.label(
                                RichText::new(format!("✔ Velotype v{current_version} is running the latest version."))
                                    .color(Color32::from_rgb(52, 211, 153))
                                    .strong()
                                    .monospace()
                                    .size(12.0),
                            );
                        }
                        UpdateStatus::UpdateAvailable(ref new_ver) => {
                            ui.label(
                                RichText::new(format!("★ New Version Available: v{new_ver}"))
                                    .color(theme.accent)
                                    .strong()
                                    .monospace()
                                    .size(12.5),
                            );
                        }
                        UpdateStatus::Downloading {
                            ref version,
                            progress_pct,
                            downloaded_mb,
                            total_mb,
                        } => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("Downloading Velotype v{version}..."))
                                        .color(theme.accent)
                                        .strong()
                                        .size(12.5),
                                );
                            });
                            ui.add_space(8.0);

                            let fraction = (progress_pct / 100.0).clamp(0.0, 1.0);
                            ui.add(
                                egui::ProgressBar::new(fraction)
                                    .show_percentage()
                                    .animate(true),
                            );
                            ui.add_space(6.0);

                            ui.label(
                                RichText::new(format!(
                                    "{downloaded_mb:.1} MB / {total_mb:.1} MB ({progress_pct:.0}%)"
                                ))
                                .color(theme.text_dim)
                                .monospace()
                                .size(11.0),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Downloading silently in background. Once complete, click Restart below to apply.")
                                    .color(theme.text_dim)
                                    .size(11.0),
                            );
                        }
                        UpdateStatus::ReadyToInstall { ref version, .. } => {
                            Frame::none()
                                .fill(theme.bg)
                                .stroke(Stroke::new(1.0, theme.accent))
                                .rounding(6.0)
                                .inner_margin(Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.set_width(inner_w - 24.0);
                                    ui.label(
                                        RichText::new(format!("🎉 Release v{version} Downloaded & Ready"))
                                            .color(theme.accent)
                                            .strong()
                                            .size(12.5),
                                    );
                                    ui.label(
                                        RichText::new("Click the Restart button below to run the setup installer and update Velotype.")
                                            .color(theme.text_active)
                                            .size(11.5),
                                    );
                                });
                        }
                        UpdateStatus::Error(ref err) => {
                            ui.label(
                                RichText::new(format!("✖ Notice: {err}"))
                                    .color(Color32::from_rgb(255, 92, 92))
                                    .size(11.5),
                            );
                        }
                    }
                });

            // Card 2: Release Information & Notes
            if let Some(ref rel) = *updater.release_info.lock().unwrap_or_else(|p| p.into_inner()) {
                ui.add_space(16.0);
                Frame::none()
                    .fill(theme.bg_surface)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(10.0)
                    .inner_margin(Margin::same(pad))
                    .show(ui, |ui| {
                        ui.set_min_width(inner_w);
                        ui.set_max_width(inner_w);

                        ui.label(
                            RichText::new(format!("RELEASE NOTES: v{}", rel.version))
                                .color(theme.text_active)
                                .strong()
                                .monospace()
                                .size(13.0),
                        );
                        ui.add_space(8.0);

                        let notes_preview = if rel.release_notes.trim().is_empty() {
                            "Performance improvements, kinetic physics tuning, and bug fixes."
                        } else {
                            &rel.release_notes
                        };

                        ui.label(
                            RichText::new(notes_preview)
                                .color(theme.text_dim)
                                .monospace()
                                .size(11.0),
                        );
                    });
            }

            // Interactive Update Button prominently positioned at the very bottom
            ui.add_space(24.0);
            updater.render_button(ui, theme, false);
        });
    }
}
