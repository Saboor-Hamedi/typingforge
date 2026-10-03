use crate::data::AppConfig;
use crate::fx::{CaretController, ParticleSystem};
use crate::game::stats::VelocityPoint;
use crate::game::text::DisplayChar;
use crate::typing::{GameMode, GameState, LiveMetrics, TypingRenderer};
use crate::ui::components::VelocityGraphWidget;
use crate::ui::theme::Theme;
use egui::RichText;

pub struct TypingView;

impl TypingView {
    pub fn show(
        ui: &mut egui::Ui,
        words: &[Vec<DisplayChar>],
        current_word: usize,
        current_char: usize,
        caret: &mut CaretController,
        particles: &ParticleSystem,
        theme: &Theme,
        config: &AppConfig,
        mode: GameMode,
        remaining_time: f32,
        streak: usize,
        state: GameState,
        metrics: &LiveMetrics,
        velocity_history: &[VelocityPoint],
        instant_wpm: f32,
        net_wpm: f32,
    ) {
        ui.add_space(20.0);

        // Dim the counters to 40% opacity while actively typing to pull focus to text (Part 4)
        let is_actively_typing = state == GameState::Running;
        let counter_alpha = if is_actively_typing { 0.40 } else { 1.0 };

        ui.scope(|ui| {
            ui.set_opacity(counter_alpha);

            // Floating live telemetry HUD (no card boxes, tabular numerals, smooth EMA)
            ui.horizontal(|ui| {
                if mode == GameMode::Timed {
                    ui.label(
                        RichText::new(format!("{:>2.0}", remaining_time.ceil()))
                            .color(theme.accent)
                            .monospace()
                            .size(28.0),
                    );
                    ui.add_space(20.0);
                } else {
                    let total = words.len();
                    let current = current_word.min(total);
                    ui.label(
                        RichText::new(format!("{current}/{total}"))
                            .color(theme.accent)
                            .monospace()
                            .size(24.0),
                    );
                    ui.add_space(20.0);
                }

                if config.show_live_wpm && state != GameState::Idle {
                    // EMA smoothed WPM with tabular numerals
                    ui.label(
                        RichText::new(format!("{} wpm", metrics.format_wpm()))
                            .color(theme.text_dim)
                            .monospace()
                            .size(18.0),
                    );
                    ui.add_space(20.0);
                }

                // Burst WPM: Only displayed if it exceeds threshold (> 100 WPM) (Part 4)
                if let Some(burst) = metrics.burst_wpm {
                    ui.label(
                        RichText::new(format!("⚡ {:.0} burst", burst))
                            .color(theme.graph_instant)
                            .monospace()
                            .size(18.0),
                    );
                    ui.add_space(20.0);
                }

                if streak >= 10 {
                    ui.label(
                        RichText::new(format!("🔥 {streak}"))
                            .color(theme.streak_fire)
                            .monospace()
                            .size(18.0),
                    );
                }
            });
        });

        ui.add_space(14.0);

        // Centered Typing Canvas (Uniform line margins, soft crimson typo colors, spring caret)
        TypingRenderer::draw(
            ui,
            words,
            current_word,
            current_char,
            caret,
            particles,
            theme,
            config.font_size,
        );

        ui.add_space(18.0);

        // Flat Spline Velocity Telemetry Waveform
        if config.show_velocity_graph {
            VelocityGraphWidget::draw(
                ui,
                velocity_history,
                theme,
                instant_wpm,
                net_wpm,
                90.0,
                true,
            );
        }
    }
}
