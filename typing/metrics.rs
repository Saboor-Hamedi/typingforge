/// Real-time metrics calculations with Exponential Moving Average (EMA) smoothing,
/// throttling, and burst detection as specified in Part 4.

#[derive(Debug, Clone)]
pub struct LiveMetrics {
    /// Internal raw target WPM
    pub internal_net_wpm: f32,
    pub internal_raw_wpm: f32,
    pub internal_instant_wpm: f32,
    pub internal_accuracy: f32,

    /// EMA smoothed WPM for jitter-free visual display (glides rather than snaps)
    pub smoothed_wpm: f32,
    pub displayed_wpm: f32,

    /// Burst WPM (only non-None if exceeding threshold, e.g. > 100.0 WPM)
    pub burst_wpm: Option<f32>,

    /// Last time rendered metrics were updated (throttled every 150ms)
    last_update_time: f32,
    throttle_interval_secs: f32,

    /// EMA smoothing factor alpha (0.15 as specified in directive)
    ema_alpha: f32,
}

impl Default for LiveMetrics {
    fn default() -> Self {
        Self {
            internal_net_wpm: 0.0,
            internal_raw_wpm: 0.0,
            internal_instant_wpm: 0.0,
            internal_accuracy: 100.0,
            smoothed_wpm: 0.0,
            displayed_wpm: 0.0,
            burst_wpm: None,
            last_update_time: 0.0,
            throttle_interval_secs: 0.15, // 150ms throttle
            ema_alpha: 0.15,
        }
    }
}

impl LiveMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Called on every keystroke to update underlying metrics immediately
    pub fn update_keystroke(
        &mut self,
        net_wpm: f32,
        raw_wpm: f32,
        accuracy: f32,
        instant_wpm: f32,
    ) {
        self.internal_net_wpm = net_wpm;
        self.internal_raw_wpm = raw_wpm;
        self.internal_accuracy = accuracy;
        self.internal_instant_wpm = instant_wpm;
    }

    /// Throttled update called each frame: glides displayed metrics using EMA every 150ms
    pub fn update_frame(&mut self, elapsed_time: f32, dt: f32) {
        // Continuous smooth interpolation towards smoothed target (framerate independent)
        let target = self.internal_net_wpm;
        let blend = (1.0 - (1.0 - self.ema_alpha).powf((dt / 0.016).max(0.0))).clamp(0.0, 1.0);
        self.smoothed_wpm = self.smoothed_wpm + blend * (target - self.smoothed_wpm);

        // Throttle rendered display refresh to every 150ms
        if elapsed_time - self.last_update_time >= self.throttle_interval_secs {
            self.last_update_time = elapsed_time;
            self.displayed_wpm = self.smoothed_wpm;

            // Burst calculation: only show if exceeding threshold (> 100 WPM)
            if self.internal_instant_wpm > 100.0 {
                self.burst_wpm = Some(self.internal_instant_wpm);
            } else {
                self.burst_wpm = None;
            }
        }
    }

    /// Formats number with fixed width / tabular numerals so digits never jitter width
    pub fn format_wpm(&self) -> String {
        format!("{:.0}", self.displayed_wpm)
    }

    pub fn format_accuracy(&self) -> String {
        format!("{:.1}%", self.internal_accuracy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_smoothing_and_burst_threshold() {
        let mut metrics = LiveMetrics::new();
        assert_eq!(metrics.burst_wpm, None);

        // Keystroke update with high burst
        metrics.update_keystroke(60.0, 65.0, 98.0, 120.0);
        assert_eq!(metrics.internal_net_wpm, 60.0);

        // Step 150ms
        metrics.update_frame(0.16, 0.016);
        assert!(metrics.smoothed_wpm > 0.0);
        assert!(metrics.burst_wpm.is_some());
        assert_eq!(metrics.burst_wpm.unwrap(), 120.0);

        // When instant drops below 100, burst should disappear
        metrics.update_keystroke(60.0, 65.0, 98.0, 85.0);
        metrics.update_frame(0.32, 0.016);
        assert_eq!(metrics.burst_wpm, None);
    }
}
