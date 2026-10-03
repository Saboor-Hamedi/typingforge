use egui::{Color32, Pos2, Rect, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CaretStyle {
    Line,
    Bar,
    Block,
    Underline,
}

impl CaretStyle {
    pub const ALL: &'static [CaretStyle] = &[
        CaretStyle::Line,
        CaretStyle::Bar,
        CaretStyle::Block,
        CaretStyle::Underline,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            CaretStyle::Line => "Line",
            CaretStyle::Bar => "Bar",
            CaretStyle::Block => "Block",
            CaretStyle::Underline => "Underline",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CaretController {
    pub current_pos: Pos2,
    pub target_pos: Pos2,
    pub velocity: Vec2,

    pub target_width: f32,
    pub target_height: f32,
    pub current_width: f32,
    pub current_height: f32,

    pub idle_time: f32,
    pub is_typing: bool,

    pub style: CaretStyle,
    pub base_width: f32,
    pub smoothness: f32,
    pub glow_intensity: f32,
}

impl Default for CaretController {
    fn default() -> Self {
        Self {
            current_pos: Pos2::ZERO,
            target_pos: Pos2::ZERO,
            velocity: Vec2::ZERO,
            target_width: 3.0,
            target_height: 28.0,
            current_width: 3.0,
            current_height: 28.0,
            idle_time: 0.0,
            is_typing: false,
            style: CaretStyle::Line,
            base_width: 3.0,
            smoothness: 0.18,
            glow_intensity: 0.65,
        }
    }
}

impl CaretController {
    pub fn update(&mut self, dt: f32) {
        if self.is_typing {
            self.idle_time = 0.0;
            self.is_typing = false;
        } else {
            self.idle_time += dt;
        }

        // Substep physics simulation with max 0.008s steps to eliminate numerical
        // instability and spring explosion when alt-tabbing or encountering large dt
        let sub_dt = 0.008_f32;
        let mut remaining = dt.min(0.08);
        while remaining > 0.0001 {
            let step = remaining.min(sub_dt);
            remaining -= step;

            let stiffness = 240.0 * (1.0 / self.smoothness.clamp(0.05, 0.4));
            let damping = 2.0 * stiffness.sqrt();

            let delta_x = self.target_pos.x - self.current_pos.x;
            let delta_y = self.target_pos.y - self.current_pos.y;

            let force_x = delta_x * stiffness - self.velocity.x * damping;
            let force_y = delta_y * stiffness - self.velocity.y * damping;

            self.velocity.x += force_x * step;
            self.velocity.y += force_y * step;

            // Clamping velocity to prevent runaway values
            self.velocity.x = self.velocity.x.clamp(-3000.0, 3000.0);
            self.velocity.y = self.velocity.y.clamp(-3000.0, 3000.0);

            self.current_pos.x += self.velocity.x * step;
            self.current_pos.y += self.velocity.y * step;
        }

        // Snap to target if within sub-pixel threshold
        if (self.target_pos.x - self.current_pos.x).abs() < 0.25 && (self.target_pos.y - self.current_pos.y).abs() < 0.25 {
            self.current_pos = self.target_pos;
            self.velocity = Vec2::ZERO;
        }

        self.current_width += (self.target_width - self.current_width) * (dt * 18.0).min(1.0);
        self.current_height += (self.target_height - self.current_height) * (dt * 18.0).min(1.0);
    }

    pub fn set_target(&mut self, pos: Pos2, char_width: f32, char_height: f32) {
        if self.current_pos == Pos2::ZERO {
            self.current_pos = pos;
            self.velocity = Vec2::ZERO;
        }
        if (self.target_pos.x - pos.x).abs() > 0.5 || (self.target_pos.y - pos.y).abs() > 0.5 {
            self.target_pos = pos;
            self.idle_time = 0.0;
        }
        self.target_width = char_width;
        self.target_height = char_height;
    }

    pub fn snap_to(&mut self, pos: Pos2) {
        self.current_pos = pos;
        self.target_pos = pos;
        self.velocity = Vec2::ZERO;
        self.idle_time = 0.0;
    }

    pub fn blink_alpha(&self) -> f32 {
        if self.idle_time < 0.4 {
            1.0
        } else {
            // Eased sine fade
            0.45 + 0.55 * ((self.idle_time - 0.4) * 4.5).cos()
        }
    }

    pub fn draw(&self, painter: &egui::Painter, primary_color: Color32) {
        let alpha = self.blink_alpha();
        if alpha <= 0.02 {
            return;
        }

        let base_alpha = (primary_color.a() as f32 / 255.0) * alpha;

        // Caret stretch effect based on horizontal speed:
        let speed_x = self.velocity.x;
        let stretch = (speed_x.abs() * 0.014).min(14.0);

        let base_color = Color32::from_rgba_premultiplied(
            ((primary_color.r() as f32) * base_alpha) as u8,
            ((primary_color.g() as f32) * base_alpha) as u8,
            ((primary_color.b() as f32) * base_alpha) as u8,
            (255.0 * base_alpha) as u8,
        );

        match self.style {
            CaretStyle::Line | CaretStyle::Bar => {
                let width = if self.style == CaretStyle::Bar {
                    (self.base_width * 1.6).max(5.0)
                } else {
                    self.base_width
                };
                let dynamic_width = width + stretch;
                // Center the line on the exact position so wide widths expand cleanly
                let draw_x = self.current_pos.x - (width * 0.5);

                let rect = Rect::from_min_size(
                    Pos2::new(draw_x, self.current_pos.y),
                    Vec2::new(dynamic_width, self.current_height),
                );

                // Layered Bloom Glow
                if self.glow_intensity > 0.0 {
                    for i in 1..=3 {
                        let glow_spread = (i as f32) * 2.5 * self.glow_intensity;
                        let glow_alpha = (0.22 / (i as f32)) * base_alpha * self.glow_intensity;
                        let glow_color = Color32::from_rgba_premultiplied(
                            ((primary_color.r() as f32) * glow_alpha) as u8,
                            ((primary_color.g() as f32) * glow_alpha) as u8,
                            ((primary_color.b() as f32) * glow_alpha) as u8,
                            (255.0 * glow_alpha) as u8,
                        );
                        let glow_rect = rect.expand(glow_spread);
                        painter.rect_filled(glow_rect, (dynamic_width + glow_spread) / 2.0, glow_color);
                    }
                }

                painter.rect_filled(rect, dynamic_width / 2.0, base_color);
            }
            CaretStyle::Block => {
                let rect = Rect::from_min_size(
                    self.current_pos,
                    Vec2::new(self.target_width.max(12.0), self.current_height),
                );
                let block_fill = Color32::from_rgba_premultiplied(
                    ((primary_color.r() as f32) * 0.35 * base_alpha) as u8,
                    ((primary_color.g() as f32) * 0.35 * base_alpha) as u8,
                    ((primary_color.b() as f32) * 0.35 * base_alpha) as u8,
                    (80.0 * base_alpha) as u8,
                );
                painter.rect_filled(rect, 3.0, block_fill);
                painter.rect_stroke(rect, 3.0, Stroke::new(1.5, base_color));
            }
            CaretStyle::Underline => {
                let underline_h = (self.base_width).clamp(2.5, 6.0);
                let rect = Rect::from_min_size(
                    Pos2::new(self.current_pos.x, self.current_pos.y + self.current_height - underline_h),
                    Vec2::new(self.target_width.max(12.0), underline_h),
                );
                painter.rect_filled(rect, 1.5, base_color);
            }
        }
    }
}
