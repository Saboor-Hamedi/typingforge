use super::fuzzy_input::FuzzyInput;
use super::fuzzy_result_item::{FuzzyResultItem, ResultItemAction};
use crate::db::{DatabaseConnection, DbPassage, DbQueries};
use crate::ui::theme::Theme;
use egui::{Color32, FontId, Key, Pos2, Rect, RichText, Sense, Stroke, Vec2};

#[derive(Debug, Clone)]
pub enum PaletteAction {
    SelectPassage(DbPassage),
    EditPassage(DbPassage),
}

pub struct FuzzyPalette {
    pub is_open: bool,
    pub query: String,
    pub results: Vec<DbPassage>,
    pub selected_index: usize,
    pub last_searched_query: String,
    pub request_focus_input: bool,
}

impl Default for FuzzyPalette {
    fn default() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            last_searched_query: "\x00".to_string(),
            request_focus_input: false,
        }
    }
}

impl FuzzyPalette {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, _db: &DatabaseConnection) {
        self.is_open = true;
        self.request_focus_input = true;
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
        self.last_searched_query.clear();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
    }

    pub fn toggle(&mut self, db: &DatabaseConnection) {
        if self.is_open {
            self.close();
        } else {
            self.open(db);
        }
    }

    fn refresh_results(&mut self, db: &DatabaseConnection) {
        let trimmed = self.query.trim();
        if trimmed.is_empty() {
            self.results.clear();
            self.selected_index = 0;
        } else if let Ok(res) = DbQueries::search_passages(db, trimmed) {
            self.results = res;
            if self.selected_index >= self.results.len() {
                self.selected_index = 0;
            }
        }
        self.last_searched_query = self.query.clone();
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        db: &DatabaseConnection,
        theme: &Theme,
    ) -> Option<PaletteAction> {
        if !self.is_open {
            return None;
        }

        let mut emitted_action = None;
        let screen_rect = ctx.screen_rect();

        // 1. Semi-transparent dark backdrop overlay (macOS Spotlight style)
        let backdrop_id = egui::Id::new("fuzzy_palette_backdrop");
        let backdrop_resp = egui::Area::new(backdrop_id)
            .order(egui::Order::Foreground)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                ui.painter().rect_filled(rect, 0.0, Color32::from_black_alpha(150));
                resp
            })
            .inner;

        if backdrop_resp.clicked() {
            self.close();
            return None;
        }

        // 2. Keyboard shortcuts inside command palette
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.close();
            return None;
        }

        let result_count = self.results.len();
        if result_count > 0 {
            if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                self.selected_index = (self.selected_index + 1) % result_count;
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                if self.selected_index == 0 {
                    self.selected_index = result_count - 1;
                } else {
                    self.selected_index -= 1;
                }
            }
            if ctx.input(|i| i.key_pressed(Key::Enter)) {
                if let Some(passage) = self.results.get(self.selected_index) {
                    emitted_action = Some(PaletteAction::SelectPassage(passage.clone()));
                    self.close();
                    return emitted_action;
                }
            }
        }

        // 3. Spotlight Centered Modal Window
        // Fixed position (centered on screen horizontally, fixed top offset).
        // Fixed maximum height: max-h-[60vh].
        // Prevents any jumping when typing. Internal list scrolls smoothly.
        let has_results = !self.query.trim().is_empty();
        let modal_w = 620.0_f32.min(screen_rect.width() - 40.0);
        let max_modal_h = (screen_rect.height() * 0.60).min(560.0);

        let modal_top = (screen_rect.height() * 0.18).max(50.0); // Constant top: NEVER jumps!
        let modal_left = (screen_rect.width() - modal_w) * 0.5;

        let overhead_h = 48.0 + 24.0 + 13.0 + 30.0; // Input(48) + Pad(24) + Divider(13) + Footer(30)
        let modal_h = if !has_results {
            72.0_f32 // Compact sleek command bar
        } else if self.results.is_empty() {
            135.0_f32 // Input + "No matching passages" message
        } else {
            let needed_h = overhead_h + (result_count as f32 * 46.0);
            needed_h.min(max_modal_h)
        };

        let modal_rect = Rect::from_min_size(Pos2::new(modal_left, modal_top), Vec2::new(modal_w, modal_h));

        egui::Area::new(egui::Id::new("fuzzy_palette_modal"))
            .order(egui::Order::Tooltip)
            .fixed_pos(modal_rect.min)
            .show(ctx, |ui| {
                let p = ui.painter();
                // Outer soft shadow & sleek modal card background
                p.rect_filled(modal_rect.expand(4.0), 12.0, Color32::from_black_alpha(60));
                p.rect_filled(modal_rect, 12.0, theme.bg);
                p.rect_stroke(modal_rect, 12.0, Stroke::new(1.0, theme.border.linear_multiply(0.85)));

                let pad = 12.0;
                let content_rect = modal_rect.shrink(pad);

                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );

                // 3a. Sleek Floating Borderless Search Input (48px)
                let request_focus = self.request_focus_input;
                self.request_focus_input = false;

                let changed = FuzzyInput::show(&mut child_ui, &mut self.query, theme, request_focus);
                if changed || self.query != self.last_searched_query {
                    self.refresh_results(db);
                }

                // 3b. Results Body with Internal Scrolling
                if has_results {
                    child_ui.add_space(4.0);
                    // 1px subtle divider
                    let (div_rect, _) = child_ui.allocate_exact_size(Vec2::new(child_ui.available_width(), 1.0), Sense::hover());
                    child_ui.painter().rect_filled(div_rect, 0.0, Color32::from_white_alpha(18));
                    child_ui.add_space(8.0);

                    if self.results.is_empty() {
                        child_ui.add_space(14.0);
                        child_ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("No matching passages found")
                                    .color(theme.text_dim)
                                    .font(FontId::proportional(13.5)),
                            );
                        });
                    } else {
                        let scroll_h = (modal_h - overhead_h).max(46.0);
                        egui::ScrollArea::vertical()
                            .max_height(scroll_h)
                            .auto_shrink([false, false])
                            .show(&mut child_ui, |scroll_ui| {
                                for (idx, passage) in self.results.iter().enumerate() {
                                    let is_selected = idx == self.selected_index;
                                    match FuzzyResultItem::show(scroll_ui, passage, is_selected, theme) {
                                        ResultItemAction::Select => {
                                            emitted_action = Some(PaletteAction::SelectPassage(passage.clone()));
                                            self.close();
                                            break;
                                        }
                                        ResultItemAction::Edit => {
                                            emitted_action = Some(PaletteAction::EditPassage(passage.clone()));
                                            self.close();
                                            break;
                                        }
                                        ResultItemAction::None => {}
                                    }
                                }
                            });

                        // 3c. Minimal Footer with keyboard hints
                        child_ui.add_space(4.0);
                        let (footer_rect, _) = child_ui.allocate_exact_size(Vec2::new(child_ui.available_width(), 20.0), Sense::hover());
                        let fp = child_ui.painter_at(footer_rect);
                        fp.line_segment(
                            [Pos2::new(footer_rect.min.x, footer_rect.min.y), Pos2::new(footer_rect.max.x, footer_rect.min.y)],
                            Stroke::new(1.0, Color32::from_white_alpha(12)),
                        );

                        let footer_text = "↑↓ Navigate   •   ↵ Select & Type   •   Click 'Edit'   •   Esc Close";
                        fp.text(
                            Pos2::new(footer_rect.min.x + 4.0, footer_rect.center().y + 2.0),
                            egui::Align2::LEFT_CENTER,
                            footer_text,
                            FontId::monospace(10.0),
                            theme.text_dim.linear_multiply(0.75),
                        );
                    }
                }
            });

        emitted_action
    }
}
