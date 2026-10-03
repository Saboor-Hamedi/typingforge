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
            if self.selected_index >= self.results.len().min(5) {
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

        let displayed_count = self.results.len().min(5);
        if displayed_count > 0 {
            if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                self.selected_index = (self.selected_index + 1) % displayed_count;
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                if self.selected_index == 0 {
                    self.selected_index = displayed_count - 1;
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
        // When query is empty: ONLY sleek input appears! (~68px)
        // When searching: 5 results appear in body of the fuzzy modal!
        let has_results = !self.query.trim().is_empty();
        let modal_w = 600.0_f32.min(screen_rect.width() - 40.0);

        let modal_h = if !has_results {
            68.0_f32 // Just the sleek input pill floating
        } else if self.results.is_empty() {
            135.0_f32 // Input + "No matching passages" message
        } else {
            // Input (56) + divider (1) + padding + 5 results (5 * 50) + footer (32)
            (68.0 + (displayed_count as f32 * 50.0) + 38.0).min(screen_rect.height() - 80.0)
        };

        let modal_top = (screen_rect.height() - modal_h) * 0.28; // Spotlight placement (top-center)
        let modal_left = (screen_rect.width() - modal_w) * 0.5;
        let modal_rect = Rect::from_min_size(Pos2::new(modal_left, modal_top), Vec2::new(modal_w, modal_h));

        egui::Area::new(egui::Id::new("fuzzy_palette_modal"))
            .order(egui::Order::Tooltip)
            .fixed_pos(modal_rect.min)
            .show(ctx, |ui| {
                let p = ui.painter();
                // Outer soft shadow & sleek modal card background
                p.rect_filled(modal_rect.expand(4.0), 14.0, Color32::from_black_alpha(50));
                p.rect_filled(modal_rect, 14.0, theme.bg);
                p.rect_stroke(modal_rect, 14.0, Stroke::new(1.0, theme.border.linear_multiply(0.85)));

                let pad = 12.0;
                let content_rect = modal_rect.shrink(pad);

                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );

                // 3a. Sleek Floating Borderless Search Input
                let request_focus = self.request_focus_input;
                self.request_focus_input = false;

                let changed = FuzzyInput::show(&mut child_ui, &mut self.query, theme, request_focus);
                if changed || self.query != self.last_searched_query {
                    self.refresh_results(db);
                }

                // 3b. Results Body (ONLY appears when searching!)
                if has_results {
                    child_ui.add_space(4.0);
                    // Very subtle 1px divider between input and results
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
                        // Take up to 5 results only as requested
                        for (idx, passage) in self.results.iter().take(5).enumerate() {
                            let is_selected = idx == self.selected_index;
                            match FuzzyResultItem::show(&mut child_ui, passage, is_selected, theme) {
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

                        // 3c. Minimal Footer with keyboard hints
                        child_ui.add_space(6.0);
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
                            theme.text_dim.linear_multiply(0.8),
                        );
                    }
                }
            });

        emitted_action
    }
}
