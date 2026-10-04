use crate::game::VelocityPoint;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};

pub struct VelocityGraphWidget;

/// Cached resampled graph geometry. The Catmull-Rom curve and instant polyline
/// only change when the data or canvas geometry changes, so they are recomputed
/// at most once per new sample rather than every frame.
#[derive(Clone)]
struct GraphCache {
    key: u64,
    instant: Vec<Pos2>,
    spline: Vec<Pos2>,
}

fn graph_cache_key(history: &[VelocityPoint], max_wpm: f32, total_time: f32, rect: Rect) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    history.len().hash(&mut hasher);
    if let Some(p) = history.first() {
        p.time_secs.to_bits().hash(&mut hasher);
        p.net_wpm.to_bits().hash(&mut hasher);
    }
    if let Some(p) = history.last() {
        p.time_secs.to_bits().hash(&mut hasher);
        p.net_wpm.to_bits().hash(&mut hasher);
        p.instant_wpm.to_bits().hash(&mut hasher);
    }
    max_wpm.to_bits().hash(&mut hasher);
    total_time.to_bits().hash(&mut hasher);
    rect.min.x.to_bits().hash(&mut hasher);
    rect.min.y.to_bits().hash(&mut hasher);
    rect.width().to_bits().hash(&mut hasher);
    rect.height().to_bits().hash(&mut hasher);
    hasher.finish()
}

impl VelocityGraphWidget {
    pub fn draw(
        ui: &mut egui::Ui,
        history: &[VelocityPoint],
        theme: &Theme,
        _current_instant: f32,
        _current_net: f32,
        height: f32,
        _show_labels: bool,
    ) {
        let available_w = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(available_w, height), Sense::hover());
        let painter = ui.painter_at(rect);

        // Pure flat canvas: no boxy card layers or borders on top of the background

        if history.len() < 2 {
            let center = rect.center();
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "Type to engage kinetic telemetry curve...",
                egui::FontId::monospace(11.0),
                theme.text_dim,
            );
            return;
        }

        let padding_x = 20.0;
        let padding_y = 14.0;
        let draw_rect = rect.shrink2(Vec2::new(padding_x, padding_y));

        let mut max_wpm: f32 = 60.0;
        for p in history {
            if p.net_wpm > max_wpm {
                max_wpm = p.net_wpm;
            }
            if p.instant_wpm > max_wpm {
                max_wpm = p.instant_wpm;
            }
        }
        max_wpm = ((max_wpm + 15.0) / 20.0).ceil() * 20.0;
        let total_time = history.last().map(|p| p.time_secs).unwrap_or(1.0).max(1.0);

        // Telemetry horizontal grid lines with baseline markers
        for fraction in [0.25, 0.5, 0.75] {
            let y = draw_rect.max.y - draw_rect.height() * fraction;
            painter.line_segment(
                [Pos2::new(draw_rect.min.x, y), Pos2::new(draw_rect.max.x, y)],
                Stroke::new(0.8, theme.graph_grid),
            );

            let label_wpm = (max_wpm * fraction).round() as usize;
            painter.text(
                Pos2::new(draw_rect.min.x + 2.0, y - 2.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{label_wpm}"),
                egui::FontId::monospace(9.0),
                theme.text_dim,
            );
        }

        let map_point = |time: f32, wpm: f32| -> Pos2 {
            let x = draw_rect.min.x + (time / total_time) * draw_rect.width();
            let y = draw_rect.max.y - (wpm / max_wpm).clamp(0.0, 1.0) * draw_rect.height();
            Pos2::new(x, y)
        };

        // Resample the instant polyline and Net WPM spline, cached between frames
        let cache_id = egui::Id::new("velocity_graph_geometry_cache");
        let key = graph_cache_key(history, max_wpm, total_time, draw_rect);
        let cached: Option<GraphCache> = ui.ctx().data_mut(|d| d.get_temp(cache_id));
        let cache = match cached {
            Some(c) if c.key == key => c,
            _ => {
                let instant: Vec<Pos2> = history
                    .iter()
                    .map(|p| map_point(p.time_secs, p.instant_wpm))
                    .collect();
                let raw_net_points: Vec<Pos2> = history
                    .iter()
                    .map(|p| map_point(p.time_secs, p.net_wpm))
                    .collect();
                let steps = if raw_net_points.len() > 150 { 2 } else { 4 };
                let spline = Self::catmull_rom_spline(&raw_net_points, steps);
                let cache = GraphCache { key, instant, spline };
                ui.ctx().data_mut(|d| d.insert_temp(cache_id, cache.clone()));
                cache
            }
        };

        // Instant burst raw points
        let instant_points = &cache.instant;
        for i in 0..instant_points.len().saturating_sub(1) {
            painter.line_segment(
                [instant_points[i], instant_points[i + 1]],
                Stroke::new(1.0, theme.graph_instant),
            );
        }

        let spline_points = &cache.spline;

        if spline_points.len() >= 2 {
            // Hardware vertical linear gradient fill under spline curve
            let base_y = draw_rect.max.y;
            let top_alpha = (theme.graph_stroke.a() as f32 * 0.22) as u8;
            let top_color = Color32::from_rgba_premultiplied(
                ((theme.graph_stroke.r() as f32) * 0.22) as u8,
                ((theme.graph_stroke.g() as f32) * 0.22) as u8,
                ((theme.graph_stroke.b() as f32) * 0.22) as u8,
                top_alpha,
            );
            let bot_color = Color32::TRANSPARENT;

            let mut mesh = egui::Mesh::default();
            for i in 0..spline_points.len() - 1 {
                let pt_a = spline_points[i];
                let pt_b = spline_points[i + 1];

                let idx = mesh.vertices.len() as u32;
                mesh.vertices.push(egui::epaint::Vertex { pos: pt_a, uv: egui::epaint::WHITE_UV, color: top_color });
                mesh.vertices.push(egui::epaint::Vertex { pos: pt_b, uv: egui::epaint::WHITE_UV, color: top_color });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(pt_b.x, base_y), uv: egui::epaint::WHITE_UV, color: bot_color });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(pt_a.x, base_y), uv: egui::epaint::WHITE_UV, color: bot_color });

                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx, idx + 2, idx + 3);
            }
            painter.add(egui::Shape::mesh(mesh));

            // Layered Bloom Glow & Sharp Spline Stroke
            for i in 0..spline_points.len() - 1 {
                let p1 = spline_points[i];
                let p2 = spline_points[i + 1];

                // Wide diffuse glow
                let glow_color = Color32::from_rgba_premultiplied(
                    theme.graph_stroke.r() / 3,
                    theme.graph_stroke.g() / 3,
                    theme.graph_stroke.b() / 3,
                    40,
                );
                painter.line_segment([p1, p2], Stroke::new(6.0, glow_color));

                // Core sharp curve
                painter.line_segment([p1, p2], Stroke::new(2.4, theme.graph_stroke));
            }
        }

        // Draw error event pins with subtle pulse markers
        for p in history {
            if p.is_error {
                let err_pos = map_point(p.time_secs, p.net_wpm);
                painter.circle_filled(err_pos, 3.0, theme.text_error);
                painter.line_segment(
                    [err_pos, Pos2::new(err_pos.x, draw_rect.max.y)],
                    Stroke::new(1.0, theme.text_error_bg),
                );
            }
        }

        // Animated glowing head at latest point
        if let Some(last_p) = history.last() {
            let head_pos = map_point(last_p.time_secs, last_p.net_wpm);
            painter.circle_filled(head_pos, 4.5, theme.graph_stroke);
            painter.circle_stroke(head_pos, 8.5, Stroke::new(1.2, theme.graph_stroke));
        }

        // Interactive hover crosshair and tooltip inspection
        if let Some(mouse_pos) = response.hover_pos() {
            if draw_rect.contains(mouse_pos) {
                // Find closest historical point in time
                let rel_x = (mouse_pos.x - draw_rect.min.x) / draw_rect.width();
                let hover_time = (rel_x * total_time).clamp(0.0, total_time);

                if let Some(closest) = history.iter().min_by(|a, b| {
                    (a.time_secs - hover_time).abs().total_cmp(&(b.time_secs - hover_time).abs())
                }) {
                    let pt_net = map_point(closest.time_secs, closest.net_wpm);
                    let pt_inst = map_point(closest.time_secs, closest.instant_wpm);

                    // Vertical guide line
                    painter.line_segment(
                        [Pos2::new(pt_net.x, draw_rect.min.y), Pos2::new(pt_net.x, draw_rect.max.y)],
                        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 45)),
                    );

                    // Marker highlights
                    painter.circle_stroke(pt_net, 4.5, Stroke::new(2.0, theme.graph_stroke));
                    painter.circle_stroke(pt_inst, 3.5, Stroke::new(1.5, theme.graph_instant));

                    // Floating micro-tooltip badge
                    let tooltip_text = format!(
                        "{:.1}s · Net: {:.0} · Burst: {:.0}",
                        closest.time_secs, closest.net_wpm, closest.instant_wpm
                    );
                    let font_id = egui::FontId::monospace(10.0);
                    let text_w = 170.0;
                    let text_h = 22.0;

                    let mut tooltip_pos = Pos2::new(pt_net.x - text_w * 0.5, draw_rect.min.y + 4.0);
                    tooltip_pos.x = tooltip_pos.x.clamp(draw_rect.min.x + 2.0, draw_rect.max.x - text_w - 2.0);

                    let tooltip_rect = Rect::from_min_size(tooltip_pos, Vec2::new(text_w, text_h));
                    painter.rect_filled(tooltip_rect, 5.0, theme.bg);
                    painter.rect_stroke(tooltip_rect, 5.0, Stroke::new(1.0, theme.border));
                    painter.text(
                        tooltip_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        tooltip_text,
                        font_id,
                        theme.text_active,
                    );
                }
            }
        }
    }

    /// Catmull-Rom spline interpolation through a list of control points
    fn catmull_rom_spline(points: &[Pos2], subdivisions: usize) -> Vec<Pos2> {
        if points.len() < 2 {
            return points.to_vec();
        }

        let mut result = Vec::with_capacity(points.len() * subdivisions);
        let n = points.len();

        for i in 0..n - 1 {
            let p0 = if i == 0 { points[0] } else { points[i - 1] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < n { points[i + 2] } else { points[n - 1] };

            for step in 0..subdivisions {
                let t = (step as f32) / (subdivisions as f32);
                let t2 = t * t;
                let t3 = t2 * t;

                let x = 0.5
                    * ((2.0 * p1.x)
                        + (-p0.x + p2.x) * t
                        + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
                        + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);

                let y = 0.5
                    * ((2.0 * p1.y)
                        + (-p0.y + p2.y) * t
                        + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
                        + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);

                result.push(Pos2::new(x, y));
            }
        }

        if let Some(&last) = points.last() {
            result.push(last);
        }

        result
    }
}
