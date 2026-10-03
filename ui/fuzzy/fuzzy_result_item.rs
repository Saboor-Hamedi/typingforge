use crate::db::DbPassage;
use crate::ui::theme::Theme;
use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultItemAction {
    None,
    Select,
    Edit,
}

pub struct FuzzyResultItem;

impl FuzzyResultItem {
    pub fn show(
        ui: &mut egui::Ui,
        passage: &DbPassage,
        is_selected: bool,
        theme: &Theme,
    ) -> ResultItemAction {
        let mut action = ResultItemAction::None;
        let item_height = 46.0;
        let item_width = ui.available_width();

        let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_width, item_height), Sense::click());
        let is_hovered = resp.hovered();

        if resp.clicked() {
            action = ResultItemAction::Select;
        }

        let painter = ui.painter_at(rect);
        let rounding = 8.0;

        // Subtle, lightweight selection & hover states (low opacity, never heavy or harsh)
        if is_selected {
            let sel_bg = theme.accent.linear_multiply(0.12);
            painter.rect_filled(rect, rounding, sel_bg);
        } else if is_hovered {
            painter.rect_filled(rect, rounding, Color32::from_white_alpha(10));
        }

        let pad_x = 16.0;
        let right_pad = 16.0;
        let center_y = rect.center().y;

        // ─────────────────────────────────────────────────────────────────
        // RIGHT SIDE: SUBTLY STYLED BADGES
        // Thin 1px border (20-30% opacity), low-opacity fill (5-10%), rounded corners
        // ─────────────────────────────────────────────────────────────────

        // 1. "Edit" Badge: Interactive button with accent color text
        let edit_label = "Edit";
        let badge_font = FontId::proportional(12.0);
        let edit_text_w = ui.painter().layout_no_wrap(edit_label.to_string(), badge_font.clone(), Color32::WHITE).size().x;
        let edit_w = edit_text_w + 16.0; // px-2
        let edit_h = 24.0;              // py-1
        let edit_rect = Rect::from_center_size(
            Pos2::new(rect.max.x - right_pad - edit_w / 2.0, center_y),
            Vec2::new(edit_w, edit_h),
        );

        let edit_resp = ui.interact(
            edit_rect,
            ui.id().with("edit_btn").with(passage.id),
            Sense::click(),
        );

        if edit_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if edit_resp.clicked() {
            action = ResultItemAction::Edit;
        }

        let edit_bg = if edit_resp.hovered() {
            theme.accent.linear_multiply(0.18)
        } else {
            theme.accent.linear_multiply(0.08)
        };
        let edit_border = if edit_resp.hovered() {
            Stroke::new(1.0, theme.accent.linear_multiply(0.65))
        } else {
            Stroke::new(1.0, theme.accent.linear_multiply(0.28))
        };
        painter.rect_filled(edit_rect, 5.0, edit_bg);
        painter.rect_stroke(edit_rect, 5.0, edit_border);
        painter.text(
            edit_rect.center(),
            egui::Align2::CENTER_CENTER,
            edit_label,
            badge_font.clone(),
            theme.accent,
        );

        // 2. Mode Badge: EITHER "Word" OR "Time" (never both)
        let mode_label = if passage.category.to_lowercase().contains("time") {
            "Time"
        } else if passage.word_count > 30 {
            "Time"
        } else {
            "Word"
        };
        let mode_text_w = ui.painter().layout_no_wrap(mode_label.to_string(), badge_font.clone(), Color32::WHITE).size().x;
        let mode_w = mode_text_w + 16.0;
        let mode_rect = Rect::from_center_size(
            Pos2::new(edit_rect.min.x - 8.0 - mode_w / 2.0, center_y),
            Vec2::new(mode_w, 24.0),
        );

        let badge_bg = Color32::from_white_alpha(8);
        let badge_border = Stroke::new(1.0, Color32::from_white_alpha(32));
        painter.rect_filled(mode_rect, 5.0, badge_bg);
        painter.rect_stroke(mode_rect, 5.0, badge_border);
        painter.text(
            mode_rect.center(),
            egui::Align2::CENTER_CENTER,
            mode_label,
            badge_font,
            theme.text_dim,
        );

        // ─────────────────────────────────────────────────────────────────
        // LEFT SIDE: SMART TITLE / CONTENT PREVIEW (Never generic "custom")
        // If title length < 10 or generic, show first few words of content
        // ─────────────────────────────────────────────────────────────────
        let title_text = get_smart_title(passage);

        let title_color = if is_selected {
            theme.accent
        } else if is_hovered {
            theme.text_active
        } else {
            Color32::from_rgb(230, 234, 240)
        };

        painter.text(
            Pos2::new(rect.min.x + pad_x, center_y),
            egui::Align2::LEFT_CENTER,
            title_text,
            FontId::proportional(14.0),
            title_color,
        );

        ui.add_space(4.0); // py-3 to py-4 vertical spacing between items
        action
    }
}

/// Smart title logic from prompt:
/// If title length < 10 chars OR title is in generic list ["custom", "test", "passage", etc.],
/// use the first few words of the passage content (~40 characters) as context!
fn get_smart_title(passage: &DbPassage) -> String {
    let raw = passage.category.trim();
    let lower = raw.to_lowercase();
    let generic_names = [
        "custom", "test", "passage", "prose", "quotes", "quote", "code", "untitled", "sentence",
    ];

    let is_generic = raw.chars().count() < 10
        || generic_names.iter().any(|g| lower == *g || lower.starts_with(g));

    if !is_generic {
        raw.to_string()
    } else {
        let clean = passage.text_content.trim().replace('\n', " ");
        if clean.chars().count() > 42 {
            format!("{}...", clean.chars().take(40).collect::<String>())
        } else {
            clean
        }
    }
}
