use crate::fx::{CaretController, ParticleSystem};
use crate::game::text::DisplayChar;
use crate::ui::theme::Theme;
use crate::ui::typing_area::TypingAreaWidget;

pub struct TypingRenderer;

impl TypingRenderer {
    pub fn draw(
        ui: &mut egui::Ui,
        words: &[Vec<DisplayChar>],
        current_word_idx: usize,
        current_char_idx: usize,
        caret: &mut CaretController,
        particles: &ParticleSystem,
        theme: &Theme,
        font_size: f32,
    ) {
        TypingAreaWidget::draw(
            ui,
            words,
            current_word_idx,
            current_char_idx,
            caret,
            particles,
            theme,
            font_size,
        );
    }
}
