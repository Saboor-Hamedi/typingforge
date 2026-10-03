use egui::{Color32, Pos2, Vec2};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Pos2,
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color32,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    pub enabled: bool,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self {
            particles: Vec::with_capacity(128),
            enabled: true,
        }
    }
}

impl ParticleSystem {
    pub fn emit(&mut self, pos: Pos2, color: Color32, count: usize) {
        if !self.enabled {
            return;
        }
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(20.0..120.0);
            let vx = angle.cos() * speed - 15.0;
            let vy = angle.sin() * speed - 35.0; // slight upward bias
            let max_life = rng.gen_range(0.2..0.45);
            let size = rng.gen_range(1.5..3.5);

            self.particles.push(Particle {
                pos,
                velocity: Vec2::new(vx, vy),
                life: max_life,
                max_life,
                size,
                color,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.pos += p.velocity * dt;
            p.velocity.y += 180.0 * dt; // gravity
            p.life -= dt;
        }

        // Retain only live particles
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn draw(&self, painter: &egui::Painter) {
        for p in &self.particles {
            let life_ratio = (p.life / p.max_life).clamp(0.0, 1.0);
            let alpha = (life_ratio * 255.0) as u8;
            let color = Color32::from_rgba_premultiplied(
                ((p.color.r() as f32) * life_ratio) as u8,
                ((p.color.g() as f32) * life_ratio) as u8,
                ((p.color.b() as f32) * life_ratio) as u8,
                alpha,
            );
            painter.circle_filled(p.pos, p.size * life_ratio, color);
        }
    }
}
