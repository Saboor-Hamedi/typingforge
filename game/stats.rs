use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VelocityPoint {
    pub time_secs: f32,
    pub net_wpm: f32,
    pub raw_wpm: f32,
    pub instant_wpm: f32,
    pub is_error: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeystrokeRecord {
    pub expected_char: char,
    pub actual_char: char,
    pub is_correct: bool,
    pub latency_ms: i64,
    pub position: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionStats {
    pub net_wpm: f32,
    pub raw_wpm: f32,
    pub accuracy: f32,
    pub consistency: f32,
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,
    pub incorrect_keystrokes: usize,
    pub elapsed_time: f32,
    pub max_streak: usize,
    pub velocity_history: Vec<VelocityPoint>,
    pub key_mistakes: HashMap<char, usize>,
    pub keystroke_log: Vec<KeystrokeRecord>,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            net_wpm: 0.0,
            raw_wpm: 0.0,
            accuracy: 100.0,
            consistency: 100.0,
            total_keystrokes: 0,
            correct_keystrokes: 0,
            incorrect_keystrokes: 0,
            elapsed_time: 0.0,
            max_streak: 0,
            velocity_history: Vec::new(),
            key_mistakes: HashMap::new(),
            keystroke_log: Vec::new(),
        }
    }
}

impl SessionStats {
    pub fn calculate_wpm(correct_chars: usize, total_chars: usize, elapsed_secs: f32) -> (f32, f32) {
        if elapsed_secs <= 0.2 {
            return (0.0, 0.0);
        }
        let minutes = elapsed_secs / 60.0;
        let words_typed_raw = (total_chars as f32) / 5.0;
        let words_typed_net = (correct_chars as f32) / 5.0;

        let raw = words_typed_raw / minutes;
        let net = words_typed_net / minutes;
        (net.max(0.0), raw.max(0.0))
    }

    pub fn calculate_accuracy(correct_chars: usize, total_chars: usize) -> f32 {
        if total_chars == 0 {
            100.0
        } else {
            ((correct_chars as f32) / (total_chars as f32) * 100.0).clamp(0.0, 100.0)
        }
    }

    pub fn calculate_consistency(history: &[VelocityPoint]) -> f32 {
        if history.len() < 3 {
            return 100.0;
        }
        let speeds: Vec<f32> = history.iter().map(|p| p.instant_wpm).filter(|&s| s > 5.0).collect();
        if speeds.len() < 2 {
            return 100.0;
        }

        let mean: f32 = speeds.iter().sum::<f32>() / speeds.len() as f32;
        if mean <= 0.0 {
            return 100.0;
        }

        let variance: f32 = speeds.iter().map(|&s| (s - mean).powi(2)).sum::<f32>() / (speeds.len() - 1) as f32;
        let std_dev = variance.sqrt();
        let cov = std_dev / mean; // coefficient of variation

        // Higher cov means lower consistency: 100% when cov == 0, dropping as variance increases
        ((1.0 - (cov * 0.75)) * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_calculation() {
        // 50 characters in 12 seconds = 10 words in 0.2 minutes = 50 WPM
        let (net, raw) = SessionStats::calculate_wpm(50, 50, 12.0);
        assert!((net - 50.0).abs() < 0.1);
        assert!((raw - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_accuracy_calculation() {
        assert_eq!(SessionStats::calculate_accuracy(100, 100), 100.0);
        assert_eq!(SessionStats::calculate_accuracy(90, 100), 90.0);
        assert_eq!(SessionStats::calculate_accuracy(0, 0), 100.0);
    }
}
