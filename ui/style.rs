//! Central UI style constants and helpers — the source of truth for corner
//! radii and card painting, so every surface in the app looks consistent.

use egui::{Color32, Painter, Rect};

/// Outer borderless-window corner radius (kept tiny for a crisp frameless look).
pub const RADIUS_WINDOW: f32 = 2.0;
/// Large card / panel container radius.
pub const RADIUS_CARD: f32 = 10.0;
/// Interactive control radius (buttons, inputs, segmented rows).
pub const RADIUS_CONTROL: f32 = 6.0;
/// Small chip / badge / swatch radius.
pub const RADIUS_SMALL: f32 = 4.0;

/// Paints a filled card with a 1px border drawn *inside* the shape.
///
/// egui's `rect_stroke` renders the outline centred on (and for rect shapes,
/// outside of) the fill path, which can leave a hard corner peeking out from
/// under a rounded fill. Drawing the border as a slightly larger rounded rect
/// and inset-filling the surface avoids that artifact entirely.
pub fn paint_card(painter: &Painter, rect: Rect, rounding: f32, fill: Color32, border: Color32) {
    paint_card_bordered(painter, rect, rounding, fill, border, 1.0);
}

/// Same as [`paint_card`] with an explicit border thickness.
pub fn paint_card_bordered(
    painter: &Painter,
    rect: Rect,
    rounding: f32,
    fill: Color32,
    border: Color32,
    border_w: f32,
) {
    painter.rect_filled(rect, rounding, border);
    let inner = rect.shrink(border_w);
    painter.rect_filled(inner, (rounding - border_w).max(0.0), fill);
}
