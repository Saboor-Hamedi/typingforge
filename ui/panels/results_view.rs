use crate::game::stats::SessionStats;
use crate::ui::components::{ButtonVariant, UnifiedButton, VelocityGraphWidget};
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Vec2};

pub struct ResultsView;

impl ResultsView {
    pub fn show(
        ui: &mut egui::Ui,
        stats: &SessionStats,
        theme: &Theme,
        is_personal_best: bool,
        pb_banner_timer: f32,
        on_restart: &mut bool,
        on_back: &mut bool,
        on_dismiss_pb: &mut bool,
    ) {
        ui.vertical(|ui| {
            ui.add_space(8.0);

            // 1. Brief, skippable, non-blocking Personal Best celebration banner (Part 7)
            if is_personal_best && pb_banner_timer > 0.0 {
                ui.horizontal_centered(|ui| {
                    let banner_btn = ui.add(
                        egui::Button::new(
                            RichText::new("★ NEW PERSONAL BEST ACHIEVED ★  (Click to dismiss)")
                                .color(Color32::from_rgb(251, 191, 36))
                                .monospace()
                                .strong()
                                .size(12.5),
                        )
                        .fill(Color32::from_rgba_unmultiplied(251, 191, 36, 25))
                        .rounding(6.0),
                    );

                    if banner_btn.clicked() {
                        *on_dismiss_pb = true;
                    }
                });
                ui.add_space(8.0);
            }

            // 2. Hero Metrics Row (Pure flat typography, zero card background layers)
            ui.horizontal(|ui| {
                Self::metric_item(ui, "wpm", &format!("{:.0}", stats.net_wpm), theme.accent, 54.0);
                ui.add_space(44.0);

                Self::metric_item(ui, "acc", &format!("{:.1}%", stats.accuracy), theme.text_correct, 54.0);
                ui.add_space(44.0);

                Self::metric_item(ui, "raw", &format!("{:.0}", stats.raw_wpm), theme.graph_instant, 32.0);
                ui.add_space(32.0);

                Self::metric_item(ui, "streak", &format!("{}", stats.max_streak), theme.streak_fire, 32.0);
            });

            ui.add_space(10.0);

            // 3. Secondary Metrics Line
            ui.horizontal(|ui| {
                let info = format!(
                    "time: {:.1}s    keystrokes: {}    errors: {}    consistency: {:.0}%",
                    stats.elapsed_time,
                    stats.total_keystrokes,
                    stats.incorrect_keystrokes,
                    stats.consistency
                );
                ui.label(
                    RichText::new(info)
                        .color(theme.text_dim)
                        .monospace()
                        .size(11.5),
                );
            });

            ui.add_space(14.0);

            // 4. Spline Velocity Graph
            VelocityGraphWidget::draw(
                ui,
                &stats.velocity_history,
                theme,
                stats.velocity_history.last().map(|p| p.instant_wpm).unwrap_or(0.0),
                stats.net_wpm,
                135.0,
                false,
            );

            ui.add_space(12.0);

            // 5. Key Mistakes Heatmap
            if !stats.key_mistakes.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("mistakes:").color(theme.text_dim).monospace().size(11.0));

                    let mut sorted: Vec<(&char, &usize)> = stats.key_mistakes.iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(a.1));

                    for (c, count) in sorted.into_iter().take(6) {
                        let display_c = if *c == ' ' { "space".to_string() } else { c.to_string() };
                        let label_text = format!("{display_c}: {count}");
                        ui.label(
                            RichText::new(label_text)
                                .color(Color32::from_rgb(255, 92, 92))
                                .monospace()
                                .size(11.5),
                        );
                        ui.add_space(12.0);
                    }
                });
                ui.add_space(12.0);
            }

            // 6. Action Buttons - Centered Back to Typing and Play Again
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);

                if UnifiedButton::show(ui, "← Back to Typing", ButtonVariant::Secondary, theme, 140.0).clicked() {
                    *on_back = true;
                }

                if UnifiedButton::show(ui, "▶ Play Again (Tab)", ButtonVariant::Primary, theme, 160.0).clicked() {
                    *on_restart = true;
                }
            });
        });
    }

    fn metric_item(ui: &mut egui::Ui, label: &str, val: &str, color: Color32, font_size: f32) {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(val)
                    .color(color)
                    .monospace()
                    .strong()
                    .size(font_size),
            );
            ui.label(
                RichText::new(label)
                    .color(Color32::from_rgb(112, 112, 128))
                    .monospace()
                    .size(11.0),
            );
        });
    }
}
