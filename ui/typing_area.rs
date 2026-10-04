use crate::fx::{CaretController, ParticleSystem};
use crate::game::{CharStatus, DisplayChar};
use crate::ui::theme::Theme;
use egui::{Color32, FontId, Pos2, Sense, Vec2};

pub struct TypingAreaWidget;

/// Cached word-grid layout for the typing canvas.
///
/// Layout (line wrapping, glyph metrics and per-word origins) depends only on the
/// passage's word-length sequence, the available width and the font size — not on
/// per-frame typing status. Caching it removes the per-frame wrapping pass and the
/// `layout_no_wrap("M")` font measurement, leaving only the lightweight paint pass.
#[derive(Clone)]
struct LayoutCache {
    key: u64,
    char_w: f32,
    char_h: f32,
    padding_y: f32,
    area_height: f32,
    start_x: f32,
    /// Horizontal offset (relative to the canvas origin) of each word's first glyph.
    word_start_x: Vec<f32>,
    /// Vertical offset (relative to the canvas origin) of each word's baseline line.
    word_line_y: Vec<f32>,
}

fn layout_key(words: &[Vec<DisplayChar>], available_w: f32, font_size: f32) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    words.len().hash(&mut hasher);
    for word in words {
        word.len().hash(&mut hasher);
    }
    available_w.to_bits().hash(&mut hasher);
    font_size.to_bits().hash(&mut hasher);
    hasher.finish()
}

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
        let key = layout_key(words, available_w, font_size);

        let cache_id = egui::Id::new("typing_area_layout_cache");
        let cached: Option<LayoutCache> = ui.ctx().data_mut(|d| d.get_temp(cache_id));

        let layout = match cached {
            Some(c) if c.key == key => c,
            _ => {
                // Measure EXACT monospace font metrics with a sample glyph
                let (exact_char_w, exact_char_h) = ui.fonts(|f| {
                    let galley = f.layout_no_wrap("M".to_string(), font_id.clone(), Color32::WHITE);
                    (galley.size().x, galley.size().y)
                });

                let line_spacing = font_size * 0.85;
                let line_height = exact_char_h + line_spacing;
                let padding_y = 20.0;
                let space_w = exact_char_w;

                let target_block_w = (available_w - 48.0).min(840.0).max(200.0);
                let max_line_w = target_block_w;

                // 1. Group words into wrapped lines
                let mut lines: Vec<Vec<usize>> = Vec::new();
                let mut cur_line_words: Vec<usize> = Vec::new();
                let mut cur_line_w = 0.0;
                for (w_idx, word) in words.iter().enumerate() {
                    let word_w = (word.len() as f32) * exact_char_w;
                    if cur_line_w + word_w > max_line_w && !cur_line_words.is_empty() {
                        lines.push(std::mem::take(&mut cur_line_words));
                        cur_line_w = 0.0;
                    }
                    cur_line_words.push(w_idx);
                    cur_line_w += word_w + space_w;
                }
                if !cur_line_words.is_empty() {
                    lines.push(cur_line_words);
                }
                if lines.is_empty() {
                    lines.push(Vec::new());
                }

                let start_x = ((available_w - target_block_w) / 2.0).max(24.0);

                let mut word_start_x = vec![start_x; words.len()];
                let mut word_line_y = vec![padding_y; words.len()];

                for (line_idx, line) in lines.iter().enumerate() {
                    let line_y = padding_y + (line_idx as f32) * line_height;
                    let n_words = line.len();
                    let is_last_line = line_idx == lines.len() - 1;

                    // Full justification: distribute extra space evenly among gaps
                    let gap_w = if !is_last_line && n_words > 1 {
                        let words_w: f32 = line
                            .iter()
                            .map(|&w| (words[w].len() as f32) * exact_char_w)
                            .sum();
                        let extra_w = (target_block_w - words_w).max(0.0);
                        (extra_w / ((n_words - 1) as f32)).min(space_w * 3.0)
                    } else {
                        space_w
                    };

                    let mut cur_x = start_x;
                    for &w_idx in line {
                        word_start_x[w_idx] = cur_x;
                        word_line_y[w_idx] = line_y;
                        cur_x += (words[w_idx].len() as f32) * exact_char_w + gap_w;
                    }
                }

                let line_count = lines.len().max(2);
                let area_height = line_height * (line_count as f32) + padding_y * 2.0;

                let cache = LayoutCache {
                    key,
                    char_w: exact_char_w,
                    char_h: exact_char_h,
                    padding_y,
                    area_height,
                    start_x,
                    word_start_x,
                    word_line_y,
                };
                ui.ctx().data_mut(|d| d.insert_temp(cache_id, cache.clone()));
                cache
            }
        };

        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(available_w, layout.area_height), Sense::click());
        let painter = ui.painter_at(rect);

        let origin = rect.min;
        let mut target_caret_pos = Pos2::new(
            origin.x + layout.start_x,
            origin.y + layout.padding_y,
        );

        // 2. Paint characters using the cached layout and current per-char status
        for (w_idx, word) in words.iter().enumerate() {
            let word_x = origin.x + layout.word_start_x[w_idx];
            let line_y = origin.y + layout.word_line_y[w_idx];
            let is_cur_word = w_idx == current_word_idx;

            for (c_idx, ch) in word.iter().enumerate() {
                let char_pos = Pos2::new(word_x + (c_idx as f32) * layout.char_w, line_y);

                if is_cur_word && c_idx == current_char_idx {
                    target_caret_pos = char_pos;
                }

                let char_color = match ch.status {
                    CharStatus::Correct => theme.accent,
                    CharStatus::Incorrect => Color32::from_rgb(235, 87, 87),
                    CharStatus::Pending => Color32::from_rgba_unmultiplied(
                        theme.text_dim.r(),
                        theme.text_dim.g(),
                        theme.text_dim.b(),
                        115,
                    ),
                };

                let effective_font = if ch.pop_anim > 0.0 {
                    FontId::monospace(font_size * (1.0 + ch.pop_anim * 0.12))
                } else {
                    font_id.clone()
                };

                // Render the actual typed character if incorrect so the user sees what they typed
                let display_text = match ch.status {
                    CharStatus::Incorrect => ch.typed.unwrap_or(ch.expected).to_string(),
                    _ => ch.expected.to_string(),
                };

                painter.text(
                    char_pos,
                    egui::Align2::LEFT_TOP,
                    display_text,
                    effective_font,
                    char_color,
                );
            }

            if is_cur_word && current_char_idx >= word.len() {
                target_caret_pos = Pos2::new(word_x + (word.len() as f32) * layout.char_w, line_y);
            }
        }

        // Caret stability at completion: park at the end of the last word
        if current_word_idx >= words.len() {
            if let Some(last) = words.len().checked_sub(1) {
                target_caret_pos = Pos2::new(
                    origin.x + layout.word_start_x[last] + (words[last].len() as f32) * layout.char_w,
                    origin.y + layout.word_line_y[last],
                );
            }
        }

        caret.set_target(target_caret_pos, layout.char_w, layout.char_h);

        // Render micro-particles and the kinetic caret
        particles.draw(&painter);
        caret.draw(&painter, theme.caret);
    }
}
