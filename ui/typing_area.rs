use crate::fx::{CaretController, ParticleSystem};
use crate::game::{CharStatus, DisplayChar};
use crate::ui::theme::Theme;
use egui::{Color32, FontId, Pos2, Sense, Vec2};

pub struct TypingAreaWidget;

impl TypingAreaWidget {
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
        let available_w = ui.available_width();
        let font_id = FontId::monospace(font_size);

        // Measure EXACT monospace font metrics with a sample glyph to ensure 100% pixel-proper caret placement
        let (exact_char_w, exact_char_h) = ui.fonts(|f| {
            let galley = f.layout_no_wrap("M".to_string(), font_id.clone(), Color32::WHITE);
            (galley.size().x, galley.size().y)
        });

        // Increase line spacing for airy, comfortable typography (~1.85x line height)
        let line_spacing = font_size * 0.85;
        let line_height = exact_char_h + line_spacing;

        let padding_y = 20.0;
        let space_w = exact_char_w;

        // 1. Group words into wrapped lines using centered max block width
        let target_block_w = (available_w - 48.0).min(840.0).max(200.0);
        let max_line_w = target_block_w;

        struct LineSpan {
            word_indices: Vec<usize>,
        }

        let mut lines: Vec<LineSpan> = Vec::new();
        let mut cur_line_words: Vec<usize> = Vec::new();
        let mut cur_line_w = 0.0;

        for (w_idx, word) in words.iter().enumerate() {
            let word_w = (word.len() as f32) * exact_char_w;
            if cur_line_w + word_w > max_line_w && !cur_line_words.is_empty() {
                lines.push(LineSpan {
                    word_indices: std::mem::take(&mut cur_line_words),
                });
                cur_line_w = 0.0;
            }
            cur_line_words.push(w_idx);
            cur_line_w += word_w + space_w;
        }
        if !cur_line_words.is_empty() {
            lines.push(LineSpan {
                word_indices: cur_line_words,
            });
        }

        // Dynamic height: expand naturally based on total lines (MonkeyType style, no cutting off)
        let total_lines = lines.len().max(2);
        let area_height = line_height * (total_lines as f32) + padding_y * 2.0;

        let (rect, _response) = ui.allocate_exact_size(Vec2::new(available_w, area_height), Sense::click());
        let painter = ui.painter_at(rect);

        // 2. Lay out characters with fully justified text (no ragged edges, no right push)
        let uniform_start_x = rect.min.x + ((available_w - target_block_w) / 2.0).max(24.0);
        let mut char_draw_list: Vec<(Pos2, &DisplayChar, bool)> = Vec::new();
        let mut target_caret_pos = Pos2::new(uniform_start_x, rect.min.y + padding_y);

        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = rect.min.y + padding_y + (line_idx as f32) * line_height;
            let n_words = line.word_indices.len();
            let is_last_line = line_idx == lines.len() - 1;

            // Full justification: distribute extra line space evenly among gaps (except last line)
            let gap_w = if !is_last_line && n_words > 1 {
                let words_w: f32 = line.word_indices.iter().map(|&w| (words[w].len() as f32) * exact_char_w).sum();
                let extra_w = (target_block_w - words_w).max(0.0);
                (extra_w / ((n_words - 1) as f32)).min(space_w * 3.0)
            } else {
                space_w
            };

            let mut cur_x = uniform_start_x;

            for &w_idx in &line.word_indices {
                let word = &words[w_idx];
                let is_cur_word = w_idx == current_word_idx;

                let word_start_x = cur_x;
                for (c_idx, ch) in word.iter().enumerate() {
                    let char_pos = Pos2::new(word_start_x + (c_idx as f32) * exact_char_w, line_y);
                    char_draw_list.push((char_pos, ch, is_cur_word));

                    if is_cur_word && c_idx == current_char_idx {
                        target_caret_pos = char_pos;
                    }
                }

                if is_cur_word && current_char_idx >= word.len() {
                    target_caret_pos = Pos2::new(word_start_x + (word.len() as f32) * exact_char_w, line_y);
                }

                cur_x += (word.len() as f32) * exact_char_w + gap_w;
            }
        }

        // Caret stability at Word=100 (or milestone/completion): park caret at the end of the last word
        if current_word_idx >= words.len() {
            if let Some(last_line_idx) = lines.len().checked_sub(1) {
                let last_line = &lines[last_line_idx];
                let line_y = rect.min.y + padding_y + (last_line_idx as f32) * line_height;
                let mut cur_x = uniform_start_x;
                for &w_idx in &last_line.word_indices {
                    let ww = (words[w_idx].len() as f32) * exact_char_w;
                    if w_idx == words.len() - 1 {
                        target_caret_pos = Pos2::new(cur_x + ww, line_y);
                    }
                    cur_x += ww + space_w;
                }
            }
        }

        // Container expands naturally (MonkeyType style, scroll_y is 0)
        let scroll_y = 0.0;

        // Proper Caret Placement
        caret.set_target(
            Pos2::new(target_caret_pos.x, target_caret_pos.y - scroll_y),
            exact_char_w,
            exact_char_h,
        );

        // Draw characters with proper alignment
        for (pos, ch, _is_cur_word) in char_draw_list {
            let scrolled_y = pos.y - scroll_y;
            if scrolled_y < rect.min.y - 10.0 || scrolled_y > rect.max.y + 10.0 {
                continue;
            }

            let char_color = match ch.status {
                // Typed Text Accent: Primary accent color pops distinctly
                CharStatus::Correct => theme.accent,
                // Error Text: Muted red with no background
                CharStatus::Incorrect => Color32::from_rgb(235, 87, 87),
                // Placeholder Styling: Untyped/upcoming text styled at ~45% muted opacity
                CharStatus::Pending => Color32::from_rgba_unmultiplied(
                    theme.text_dim.r(),
                    theme.text_dim.g(),
                    theme.text_dim.b(),
                    115,
                ),
            };

            let draw_pos = Pos2::new(pos.x, scrolled_y);
            let effective_font = if ch.pop_anim > 0.0 {
                FontId::monospace(font_size * (1.0 + ch.pop_anim * 0.12))
            } else {
                font_id.clone()
            };

            // Typo Visuals: No background highlight, no shake, no sound change. Just red text color + caret advance.
            let display_text = ch.expected.to_string();

            painter.text(
                draw_pos,
                egui::Align2::LEFT_TOP,
                display_text,
                effective_font,
                char_color,
            );
        }

        // Render micro-particles
        particles.draw(&painter);

        // Render kinetic caret
        caret.draw(&painter, theme.caret);
    }
}
