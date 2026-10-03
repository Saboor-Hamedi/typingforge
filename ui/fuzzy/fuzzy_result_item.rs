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
        let item_height = 44.0;
        let item_width = ui.available_width();

        let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_width, item_height), Sense::click());
        let is_hovered = resp.hovered();

        if resp.clicked() {
            action = ResultItemAction::Select;
        }

        let painter = ui.painter_at(rect);
        let rounding = 6.0;

        // Subtle, lightweight selection & hover states
        if is_selected {
            let sel_bg = theme.accent.linear_multiply(0.12);
            painter.rect_filled(rect, rounding, sel_bg);
        } else if is_hovered {
            painter.rect_filled(rect, rounding, Color32::from_white_alpha(10));
        }

        let pad_x = 14.0;
        let right_pad = 14.0;
        let center_y = rect.center().y;
        let badge_font = FontId::proportional(11.5);
        let subtle_border = Stroke::new(1.0, Color32::from_white_alpha(25)); // 10% opacity subtle border

        // ─────────────────────────────────────────────────────────────────
        // RIGHT SIDE: BADGES (Word count, Mode, Edit)
        // STRICTLY TRANSPARENT BACKGROUNDS (No creamy fills!)
        // ─────────────────────────────────────────────────────────────────

        // 1. "Edit" Badge Button (Accent text, transparent background)
        let edit_label = "Edit";
        let edit_text_w = ui.painter().layout_no_wrap(edit_label.to_string(), badge_font.clone(), Color32::WHITE).size().x;
        let edit_w = edit_text_w + 14.0;
        let edit_h = 22.0;
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

        let edit_border = if edit_resp.hovered() {
            Stroke::new(1.0, theme.accent.linear_multiply(0.55))
        } else {
            subtle_border
        };
        // Transparent background: NO rect_filled!
        painter.rect_stroke(edit_rect, 4.0, edit_border);
        painter.text(
            edit_rect.center(),
            egui::Align2::CENTER_CENTER,
            edit_label,
            badge_font.clone(),
            theme.accent,
        );

        // 2. Mode Badge: EITHER "Word" OR "Time"
        let mode_label = if passage.category.to_lowercase().contains("time") {
            "Time"
        } else if passage.word_count > 30 {
            "Time"
        } else {
            "Word"
        };
        let mode_text_w = ui.painter().layout_no_wrap(mode_label.to_string(), badge_font.clone(), Color32::WHITE).size().x;
        let mode_w = mode_text_w + 14.0;
        let mode_rect = Rect::from_center_size(
            Pos2::new(edit_rect.min.x - 6.0 - mode_w / 2.0, center_y),
            Vec2::new(mode_w, 22.0),
        );

        // Transparent background: NO rect_filled!
        painter.rect_stroke(mode_rect, 4.0, subtle_border);
        painter.text(
            mode_rect.center(),
            egui::Align2::CENTER_CENTER,
            mode_label,
            badge_font.clone(),
            theme.text_dim,
        );

        // 3. Word Count Badge: "{w} words"
        let words_label = format!("{} words", passage.word_count);
        let words_text_w = ui.painter().layout_no_wrap(words_label.clone(), badge_font.clone(), Color32::WHITE).size().x;
        let words_w = words_text_w + 14.0;
        let words_rect = Rect::from_center_size(
            Pos2::new(mode_rect.min.x - 6.0 - words_w / 2.0, center_y),
            Vec2::new(words_w, 22.0),
        );

        // Transparent background: NO rect_filled!
        painter.rect_stroke(words_rect, 4.0, subtle_border);
        painter.text(
            words_rect.center(),
            egui::Align2::CENTER_CENTER,
            words_label,
            badge_font,
            theme.text_dim,
        );

        // ─────────────────────────────────────────────────────────────────
        // LEFT SIDE: SMART TITLE / CONTENT PREVIEW (Never generic "custom")
        // Truncate cleanly so it never overlaps the badges
        // ─────────────────────────────────────────────────────────────────
        let available_title_w = (words_rect.min.x - rect.min.x - pad_x - 10.0).max(100.0);
        let raw_title = get_smart_title(passage);
        let title_font = FontId::proportional(13.5);

        let title_color = if is_selected {
            theme.accent
        } else if is_hovered {
            theme.text_active
        } else {
            Color32::from_rgb(220, 225, 232)
        };

        // Layout with clipping or ellipsis if exceeding available width
        let mut display_title = raw_title;
        while display_title.chars().count() > 8 && ui.painter().layout_no_wrap(display_title.clone(), title_font.clone(), Color32::WHITE).size().x > available_title_w {
            let len = display_title.chars().count();
            display_title = format!("{}...", display_title.chars().take(len.saturating_sub(5)).collect::<String>());
        }

        painter.text(
            Pos2::new(rect.min.x + pad_x, center_y),
            egui::Align2::LEFT_CENTER,
            display_title,
            title_font,
            title_color,
        );

        ui.add_space(2.0);
        action
    }
}

/// Smart title logic:
/// If title length < 10 chars OR title is generic ("custom", "test", etc.),
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
