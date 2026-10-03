use egui::Vec2;

#[derive(Debug, Clone)]
pub struct ScreenShake {
    pub timer: f32,
    pub duration: f32,
    pub intensity: f32,
    pub enabled: bool,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            timer: 0.0,
            duration: 0.22,
            intensity: 4.0,
            enabled: true,
        }
    }
}

impl ScreenShake {
    pub fn trigger(&mut self) {
        if self.enabled {
            self.timer = self.duration;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
        }
    }

    pub fn current_offset(&self) -> Vec2 {
        if self.timer <= 0.0 || !self.enabled {
            Vec2::ZERO
        } else {
            let progress = 1.0 - (self.timer / self.duration);
            let decay = (1.0 - progress).powi(2);
            let angle = progress * std::f32::consts::PI * 14.0;
            let x = angle.sin() * self.intensity * decay;
            let y = (angle * 0.7).cos() * (self.intensity * 0.5) * decay;
            Vec2::new(x, y)
        }
    }
}
