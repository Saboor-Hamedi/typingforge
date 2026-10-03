use crate::game::SessionStats;
use crate::ui::theme::Theme;
use crate::ui::velocity_graph::VelocityGraphWidget;
use egui::Color32;

pub struct ResultsScreen;

impl ResultsScreen {
    pub fn show(
        ui: &mut egui::Ui,
        stats: &SessionStats,
        theme: &Theme,
        is_personal_best: bool,
        on_restart: &mut bool,
    ) {
        ui.vertical(|ui| {
            ui.add_space(10.0);

            // 1. Personal Best banner (clean text, no heavy card)
            if is_personal_best {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        egui::RichText::new("★ NEW PERSONAL BEST ACHIEVED ★")
                            .color(Color32::from_rgb(251, 191, 36))
                            .monospace()
                            .strong()
                            .size(13.0),
                    );
                });
                ui.add_space(10.0);
            }

            // 2. Hero Metrics Row (Pure flat typography floating on canvas, ZERO card background layers)
            ui.horizontal(|ui| {
                Self::metric_item(ui, "wpm", &format!("{:.0}", stats.net_wpm), theme.accent, 56.0);
                ui.add_space(48.0);

                Self::metric_item(ui, "acc", &format!("{:.1}%", stats.accuracy), theme.text_correct, 56.0);
                ui.add_space(48.0);

                Self::metric_item(ui, "raw", &format!("{:.0}", stats.raw_wpm), theme.graph_instant, 34.0);
                ui.add_space(36.0);

                Self::metric_item(ui, "streak", &format!("{}", stats.max_streak), theme.streak_fire, 34.0);
            });

            ui.add_space(12.0);

            // 3. Secondary Metrics Line (Flat clean text)
            ui.horizontal(|ui| {
                let info = format!(
                    "time: {:.1}s    keystrokes: {}    errors: {}    consistency: {:.0}%",
                    stats.elapsed_time,
                    stats.total_keystrokes,
                    stats.incorrect_keystrokes,
                    stats.consistency
                );
                ui.label(
                    egui::RichText::new(info)
                        .color(theme.text_dim)
                        .monospace()
                        .size(11.5),
                );
            });

            ui.add_space(16.0);

            // 4. Spline Velocity Graph (Floating curve on canvas, no boxy card layers)
            VelocityGraphWidget::draw(
                ui,
                &stats.velocity_history,
                theme,
                stats.velocity_history.last().map(|p| p.instant_wpm).unwrap_or(0.0),
                stats.net_wpm,
                140.0,
                false,
            );

            ui.add_space(14.0);

            // 5. Mistake Keys (Flat clean text, no cards)
            if !stats.key_mistakes.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("mistakes:")
                            .color(theme.text_dim)
                            .monospace()
                            .size(11.0),
                    );

                    let mut sorted: Vec<(&char, &usize)> = stats.key_mistakes.iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(a.1));

                    for (c, count) in sorted.into_iter().take(6) {
                        let display_c = if *c == ' ' { "space".to_string() } else { c.to_string() };
                        let label_text = format!("{display_c}: {count}");
                        ui.label(
                            egui::RichText::new(label_text)
                                .color(Color32::from_rgb(248, 113, 113))
                                .monospace()
                                .size(11.5),
                        );
                        ui.add_space(12.0);
                    }
                });
                ui.add_space(14.0);
            }

            // 6. Action Button (Centered)
            ui.horizontal_centered(|ui| {
                let btn = egui::Button::new(
                    egui::RichText::new("  ▶ Play Again (Tab + Enter)  ")
                        .color(Color32::from_rgb(10, 14, 22))
                        .monospace()
                        .strong(),
                )
                .fill(theme.accent)
                .rounding(8.0);

                if ui.add(btn).clicked() {
                    *on_restart = true;
                }
            });
        });
    }

    fn metric_item(ui: &mut egui::Ui, label: &str, value: &str, val_col: Color32, val_size: f32) {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .color(Color32::from_rgb(140, 140, 152))
                    .monospace()
                    .size(11.0),
            );
            ui.label(
                egui::RichText::new(value)
                    .color(val_col)
                    .monospace()
                    .strong()
                    .size(val_size),
            );
        });
    }
}
