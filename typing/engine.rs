//! Core typing simulation engine and real-time kinetic telemetry.
//!
//! The [`GameEngine`] is the central authority on:
//! - Keystroke validation against target text ([`CharStatus`]: Pending, Correct, Incorrect).
//! - Caret cursor positioning (`current_word`, `current_char`).
//! - High-precision latency tracking and micro-second keystroke logging.
//! - Instantaneous rolling velocity calculation (WPM over a 1.2-second sliding window).
//! - Continuous exponential moving average (EMA) metric smoothing via [`LiveMetrics`].
//! - Perfect streak milestones (e.g. 10, 25, 50, 100, 200).
//! - Graceful typo advance and backspace unwinding (including `Ctrl+Backspace`).
//! - Automatic instantaneous test completion when the final character is pressed.

use super::metrics::LiveMetrics;
use crate::game::stats::{KeystrokeRecord, SessionStats, VelocityPoint};
use crate::game::text::{CharStatus, DisplayChar, TextGenerator};
use std::collections::VecDeque;

/// Supported gameplay evaluation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameMode {
    /// Test ends when the countdown timer reaches zero seconds.
    Timed,
    /// Test ends when all target words have been typed.
    Words,
}

/// Duration preset options for [`GameMode::Timed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimedDuration {
    /// 25-second sprint duration.
    Sec25 = 25,
    /// 40-second endurance duration.
    Sec40 = 40,
    /// 60-second long-form duration.
    Sec60 = 60,
}

/// Target word count options for [`GameMode::Words`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WordCountTarget {
    /// Standard 25-word speed test.
    Words25 = 25,
    /// Extended 40-word accuracy test.
    Words40 = 40,
    /// Marathon 120-word endurance test.
    Words120 = 120,
}

/// High-level lifecycle state machine for an active typing session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Waiting for the user's first keystroke. Timer is halted at 0.0s.
    Idle,
    /// Active typing session in progress. Clock is advancing and metrics are updating.
    Running,
    /// Session has reached its end condition (words completed or timer expired).
    Completed,
}

/// Configuration settings defining the target session rules.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Active evaluation mode (Timed vs Words).
    pub mode: GameMode,
    /// Total duration in seconds if running in [`GameMode::Timed`].
    pub timed_duration: TimedDuration,
    /// Total word target count if running in [`GameMode::Words`].
    pub word_target: WordCountTarget,
    /// Whether punctuation symbols (.,!?;: etc.) should be generated.
    pub include_punctuation: bool,
    /// Whether numeric digits (0-9) should be generated.
    pub include_numbers: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            mode: GameMode::Timed,
            timed_duration: TimedDuration::Sec25,
            word_target: WordCountTarget::Words25,
            include_punctuation: false,
            include_numbers: false,
        }
    }
}

/// The core kinetic typing game engine.
///
/// Encapsulates all word layout grids, cursor indexes, time-series velocity telemetry,
/// mistake records, and milestone streak counters.
pub struct GameEngine {
    /// Session configuration defining target rules and mode.
    pub config: SessionConfig,
    /// Current lifecycle phase (`Idle`, `Running`, or `Completed`).
    pub state: GameState,
    /// 2D word grid: outer vector is words, inner vector is individual display characters.
    pub words: Vec<Vec<DisplayChar>>,
    /// 0-indexed index of the current word being typed.
    pub current_word: usize,
    /// 0-indexed character index within the current word.
    pub current_char: usize,

    /// Absolute wall-clock epoch start timestamp (seconds), if started.
    pub start_time: Option<f64>,
    /// Elapsed active typing duration in seconds.
    pub elapsed_time: f32,
    /// Remaining countdown time in seconds (for timed mode).
    pub remaining_time: f32,

    /// Consecutive error-free keystrokes in the current streak.
    pub streak: usize,
    /// Highest consecutive error-free keystrokes achieved during this session.
    pub max_streak: usize,
    /// Temporary flag set when hitting streak thresholds (10, 25, 50, 100, 200).
    pub streak_milestone_hit: Option<usize>,

    /// Live telemetry calculators and smoothed metrics for HUD rendering.
    pub live_metrics: LiveMetrics,

    /// Keystroke timestamp queue for instant velocity calculation (1.2s rolling window).
    recent_keystroke_times: VecDeque<f32>,
    /// Timestamp of the preceding keystroke (used to compute inter-key latency).
    pub last_keystroke_time: f32,

    /// Comprehensive statistical breakdown and velocity history of the session.
    pub stats: SessionStats,
    /// Flag indicating that the last keystroke was an error (triggers subtle UI shake).
    pub has_error_shake: bool,
    /// Indicates whether the active test is using a custom/literature passage.
    pub custom_passage_active: bool,
}

impl GameEngine {
    /// Creates a new [`GameEngine`] instance initialized with the given session config.
    pub fn new(config: SessionConfig) -> Self {
        let mut engine = Self {
            config: config.clone(),
            state: GameState::Idle,
            words: Vec::new(),
            current_word: 0,
            current_char: 0,
            start_time: None,
            elapsed_time: 0.0,
            remaining_time: config.timed_duration as usize as f32,
            streak: 0,
            max_streak: 0,
            streak_milestone_hit: None,
            live_metrics: LiveMetrics::new(),
            recent_keystroke_times: VecDeque::with_capacity(32),
            last_keystroke_time: 0.0,
            stats: SessionStats::default(),
            has_error_shake: false,
            custom_passage_active: false,
        };
        engine.reset();
        engine
    }


    /// Resets all session metrics, resets cursors to index 0, clears keystroke telemetry,
    /// and generates a fresh text corpus based on active session configuration.
    pub fn reset(&mut self) {
        self.state = GameState::Idle;
        self.current_word = 0;
        self.current_char = 0;
        self.start_time = None;
        self.elapsed_time = 0.0;
        self.remaining_time = match self.config.mode {
            GameMode::Timed => self.config.timed_duration as usize as f32,
            _ => 0.0,
        };
        self.streak = 0;
        self.max_streak = 0;
        self.streak_milestone_hit = None;
        self.live_metrics.reset();
        self.recent_keystroke_times.clear();
        self.last_keystroke_time = 0.0;
        self.stats = SessionStats::default();
        self.has_error_shake = false;
        self.custom_passage_active = false;

        let word_count = match self.config.mode {
            GameMode::Words => self.config.word_target as usize,
            // Timed mode generates enough words to comfortably fill the duration
            // even at very high typing speeds (roughly 4 words per second).
            GameMode::Timed => (self.config.timed_duration as usize * 4).max(120),
        };

        let generated = TextGenerator::generate_words(
            word_count,
            self.config.include_punctuation,
            self.config.include_numbers,
        );

        self.words = generated
            .into_iter()
            .map(|w| w.chars().map(DisplayChar::new).collect())
            .collect();
    }

    /// Loads custom or database-provided passage text into the active session.
    ///
    /// Splits text by whitespace into words and individual characters with initial [`CharStatus::Pending`].
    pub fn load_passage_text(&mut self, text: &str, is_custom: bool) {
        self.reset();
        self.custom_passage_active = is_custom;

        let words: Vec<Vec<DisplayChar>> = text
            .split_whitespace()
            .map(|word| word.chars().map(DisplayChar::new).collect())
            .collect();

        if !words.is_empty() {
            self.words = words;
        }
    }

    /// Loads custom passage text (e.g. from clipboard or user input editor) and marks it as custom.
    pub fn load_passage(&mut self, text: &str) {
        self.load_passage_text(text, true);
    }

    /// Loads a passage while simultaneously overriding the target mode, word count, or duration.
    pub fn load_passage_with_mode(
        &mut self,
        text: &str,
        mode: GameMode,
        word_target: Option<WordCountTarget>,
        timed_duration: Option<TimedDuration>,
    ) {
        self.config.mode = mode;
        if let Some(wt) = word_target {
            self.config.word_target = wt;
        }
        if let Some(td) = timed_duration {
            self.config.timed_duration = td;
        }
        self.load_passage(text);
    }

    /// Advances the engine simulation clock by `dt` seconds.
    ///
    /// Performs per-frame tasks:
    /// - Decays character pop/scale bounce animations.
    /// - Advances elapsed time and slides the 1.2s rolling velocity window.
    /// - Computes instantaneous and cumulative net/raw WPM and accuracy.
    /// - Feeds telemetry into exponential moving average (EMA) HUD filters.
    /// - Periodically samples velocity points (every 150ms) for results graph plotting.
    /// - Evaluates session completion triggers (countdown expiry or word exhaustion).
    pub fn update(&mut self, dt: f32) {
        // Decay character animations
        for word in &mut self.words {
            for ch in word.iter_mut() {
                if ch.pop_anim > 0.0 {
                    ch.pop_anim = (ch.pop_anim - dt * 6.0).max(0.0);
                }
            }
        }

        if self.state != GameState::Running {
            return;
        }

        self.elapsed_time += dt;

        // Drain older keystrokes from rolling window (> 1.2s)
        while let Some(&t) = self.recent_keystroke_times.front() {
            if self.elapsed_time - t > 1.2 {
                self.recent_keystroke_times.pop_front();
            } else {
                break;
            }
        }

        // Calculate instantaneous velocity
        let mut instant_wpm = 0.0;
        if self.recent_keystroke_times.len() >= 2 {
            let count = self.recent_keystroke_times.len();
            let window_duration = (self.elapsed_time - self.recent_keystroke_times[0]).max(0.1);
            let instant_words = (count as f32) / 5.0;
            instant_wpm = (instant_words / (window_duration / 60.0)).clamp(0.0, 300.0);
        }

        // Calculate cumulative WPM & accuracy
        let (net, raw) = SessionStats::calculate_wpm(
            self.stats.correct_keystrokes,
            self.stats.total_keystrokes,
            self.elapsed_time,
        );
        let acc = SessionStats::calculate_accuracy(
            self.stats.correct_keystrokes,
            self.stats.total_keystrokes,
        );

        // Update live metrics & EMA smoothing throttled every 150ms
        self.live_metrics.update_keystroke(net, raw, acc, instant_wpm);
        self.live_metrics.update_frame(self.elapsed_time, dt);

        // Record velocity data point every 0.15s for the real-time velocity graph
        let should_record = self
            .stats
            .velocity_history
            .last()
            .map(|p| self.elapsed_time - p.time_secs >= 0.15)
            .unwrap_or(true);

        if should_record {
            // Cap velocity history to prevent memory explosion during marathon tests
            const MAX_VELOCITY_POINTS: usize = 1200;
            if self.stats.velocity_history.len() >= MAX_VELOCITY_POINTS {
                let mut downsampled = Vec::with_capacity(MAX_VELOCITY_POINTS / 2 + 1);
                for (i, pt) in self.stats.velocity_history.drain(..).enumerate() {
                    if i % 2 == 0 {
                        downsampled.push(pt);
                    }
                }
                self.stats.velocity_history = downsampled;
            }

            self.stats.velocity_history.push(VelocityPoint {
                time_secs: self.elapsed_time,
                net_wpm: net,
                raw_wpm: raw,
                instant_wpm,
                is_error: self.has_error_shake,
            });
            self.has_error_shake = false;
        }

        // Handle Mode End Conditions
        let is_last_word_done = !self.words.is_empty()
            && self.current_word == self.words.len() - 1
            && self.current_char >= self.words[self.current_word].len();

        if self.custom_passage_active {
            if self.current_word >= self.words.len() || is_last_word_done {
                self.complete();
            } else if self.config.mode == GameMode::Timed {
                self.remaining_time = (self.config.timed_duration as usize as f32 - self.elapsed_time).max(0.0);
                if self.remaining_time <= 0.0 {
                    self.complete();
                }
            }
        } else {
            match self.config.mode {
                GameMode::Timed => {
                    self.remaining_time = (self.config.timed_duration as usize as f32 - self.elapsed_time).max(0.0);
                    if self.remaining_time <= 0.0 {
                        self.complete();
                    }
                }
                GameMode::Words => {
                    if self.current_word >= self.words.len() || is_last_word_done {
                        self.complete();
                    }
                }
            }
        }

    }

    /// Computes the linear 0-indexed character offset across the entire text passage.
    ///
    /// Useful for mapping keystroke logs to exact document coordinates.
    pub fn calculate_current_position(&self) -> usize {
        let mut pos = 0;
        for w in 0..self.current_word.min(self.words.len()) {
            pos += self.words[w].len() + 1;
        }
        pos + self.current_char
    }

    /// Handles an incoming typed character keypress event.
    ///
    /// Flow:
    /// - Transitions engine from `Idle` to `Running` upon first keystroke.
    /// - If space (`' '`), validates or marks untyped characters in current word as incorrect,
    ///   then advances cursor to the beginning of the next word.
    /// - If regular character, compares against expected char. On match: marks correct,
    ///   increments streak counter, triggers milestone alert if applicable, and advances caret.
    ///   On mismatch: marks character as incorrect, logs mistake, resets streak, and advances
    ///   caret so typing flow remains natural and uninterrupted.
    /// - Immediately completes the session if the very last character of the passage is typed.
    ///
    /// Returns `true` if keystroke was correct, `false` otherwise.
    pub fn handle_char(&mut self, c: char) -> bool {
        if self.state == GameState::Completed {
            return false;
        }
        if self.state == GameState::Idle {
            self.state = GameState::Running;
            self.last_keystroke_time = self.elapsed_time;
        }

        let latency_ms = if self.stats.total_keystrokes == 0 {
            0
        } else {
            ((self.elapsed_time - self.last_keystroke_time) * 1000.0).max(0.0) as i64
        };
        self.last_keystroke_time = self.elapsed_time;

        let mut is_correct = false;
        self.stats.total_keystrokes += 1;
        self.recent_keystroke_times.push_back(self.elapsed_time);
        let pos = self.calculate_current_position();

        if c == ' ' {
            // Space advances to next word
            if self.current_word < self.words.len() {
                let word = &mut self.words[self.current_word];
                for ch_idx in self.current_char..word.len() {
                    if word[ch_idx].status == CharStatus::Pending {
                        word[ch_idx].status = CharStatus::Incorrect;
                        self.stats.incorrect_keystrokes += 1;
                        self.stats.keystroke_log.push(KeystrokeRecord {
                            expected_char: word[ch_idx].expected,
                            actual_char: ' ',
                            is_correct: false,
                            latency_ms,
                            position: pos + (ch_idx - self.current_char),
                        });
                    }
                }
                self.current_word += 1;
                self.current_char = 0;
                self.stats.correct_keystrokes += 1;
                is_correct = true;
                self.stats.keystroke_log.push(KeystrokeRecord {
                    expected_char: ' ',
                    actual_char: ' ',
                    is_correct: true,
                    latency_ms,
                    position: pos,
                });
            }
        } else if self.current_word < self.words.len() {
            let word = &mut self.words[self.current_word];
            if self.current_char < word.len() {
                let expected = word[self.current_char].expected;

                if expected == c {
                    word[self.current_char].status = CharStatus::Correct;
                    word[self.current_char].typed = Some(c);
                    word[self.current_char].pop_anim = 1.0;
                    self.stats.correct_keystrokes += 1;
                    self.streak += 1;
                    if self.streak > self.max_streak {
                        self.max_streak = self.streak;
                    }
                    if self.streak == 10 || self.streak == 25 || self.streak == 50 || self.streak == 100 || self.streak == 200 {
                        self.streak_milestone_hit = Some(self.streak);
                    }
                    is_correct = true;

                    self.stats.keystroke_log.push(KeystrokeRecord {
                        expected_char: expected,
                        actual_char: c,
                        is_correct: true,
                        latency_ms,
                        position: pos,
                    });

                    // Advance caret on correct keystroke
                    self.current_char += 1;
                } else {
                    word[self.current_char].status = CharStatus::Incorrect;
                    word[self.current_char].typed = Some(c);
                    self.stats.incorrect_keystrokes += 1;
                    self.on_mistake(expected);

                    self.stats.keystroke_log.push(KeystrokeRecord {
                        expected_char: expected,
                        actual_char: c,
                        is_correct: false,
                        latency_ms,
                        position: pos,
                    });

                    // Advance caret on typo so the typing flow keeps moving naturally
                    self.current_char += 1;
                }
            }
        }

        // Check if finished:
        // Automatically complete the test immediately once the last letter of the last word
        // is typed, without requiring the user to press space!
        let is_last_word_done = !self.words.is_empty()
            && self.current_word == self.words.len() - 1
            && self.current_char >= self.words[self.current_word].len();

        if self.current_word >= self.words.len() || is_last_word_done {
            self.complete();
        }

        is_correct
    }

    /// Handles a backspace keystroke, supporting single-character unwind and `Ctrl+Backspace` whole-word clearing.
    ///
    /// Allows moving backward across word boundaries if at the start of the current word.
    pub fn handle_backspace(&mut self, ctrl: bool) {
        if self.state != GameState::Running {
            return;
        }

        if ctrl {
            // Delete whole word back to start of word
            if self.current_word < self.words.len() {
                let word = &mut self.words[self.current_word];
                let limit = self.current_char.min(word.len());
                for i in 0..limit {
                    if word[i].status == CharStatus::Correct {
                        self.streak = self.streak.saturating_sub(1);
                    }
                    word[i].status = CharStatus::Pending;
                    word[i].typed = None;
                }
                self.current_char = 0;
            }
        } else if self.current_word < self.words.len() {
            let word = &mut self.words[self.current_word];
            if self.current_char > 0 {
                self.current_char -= 1;
                if word[self.current_char].status == CharStatus::Correct {
                    self.streak = self.streak.saturating_sub(1);
                }
                word[self.current_char].status = CharStatus::Pending;
                word[self.current_char].typed = None;
            } else if self.current_word > 0 {
                // Step back into previous word
                self.current_word -= 1;
                self.current_char = self.words[self.current_word].len();
                if self.current_char > 0 {
                    self.current_char -= 1;
                    if self.words[self.current_word][self.current_char].status == CharStatus::Correct {
                        self.streak = self.streak.saturating_sub(1);
                    }
                    self.words[self.current_word][self.current_char].status = CharStatus::Pending;
                    self.words[self.current_word][self.current_char].typed = None;
                }
            }
        }
    }

    /// Internal error handler triggered whenever a typed character disagrees with expected text.
    fn on_mistake(&mut self, expected: char) {
        self.streak = 0;
        self.has_error_shake = true;
        *self.stats.key_mistakes.entry(expected).or_insert(0) += 1;
    }

    /// Finalizes the typing session and computes official aggregate benchmarks.
    ///
    /// Computes Net WPM, Raw WPM, Accuracy, Consistency, and saves session metrics.
    pub fn complete(&mut self) {
        self.state = GameState::Completed;
        let (net, raw) = SessionStats::calculate_wpm(
            self.stats.correct_keystrokes,
            self.stats.total_keystrokes,
            self.elapsed_time,
        );
        self.stats.net_wpm = net;
        self.stats.raw_wpm = raw;
        self.stats.accuracy = SessionStats::calculate_accuracy(
            self.stats.correct_keystrokes,
            self.stats.total_keystrokes,
        );
        self.stats.elapsed_time = self.elapsed_time;
        self.stats.max_streak = self.max_streak;
        self.stats.consistency = SessionStats::calculate_consistency(&self.stats.velocity_history);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typos_advance_caret_and_backspace_recovers() {
        let config = SessionConfig::default();
        let mut engine = GameEngine::new(config);
        engine.load_passage("the quick");

        // 1. Type incorrect character 'x' for expected 't'
        let correct = engine.handle_char('x');
        assert!(!correct);
        // Caret MUST advance on typo to index 1!
        assert_eq!(engine.current_char, 1);
        assert_eq!(engine.words[0][0].status, CharStatus::Incorrect);

        // 2. Backspace moves caret back to index 0 and clears the typo
        engine.handle_backspace(false);
        assert_eq!(engine.current_char, 0);
        assert_eq!(engine.words[0][0].status, CharStatus::Pending);

        // 3. Typing correct key 't' succeeds and advances to index 1
        let correct_fixed = engine.handle_char('t');
        assert!(correct_fixed);
        assert_eq!(engine.current_char, 1);
        assert_eq!(engine.words[0][0].status, CharStatus::Correct);

        // 4. Typing typo 'z' for expected 'h' advances to index 2
        let correct2 = engine.handle_char('z');
        assert!(!correct2);
        assert_eq!(engine.current_char, 2);
        assert_eq!(engine.words[0][1].status, CharStatus::Incorrect);

        // 5. Typing correct 'e' advances to index 3 (end of word "the")
        let correct3 = engine.handle_char('e');
        assert!(correct3);
        assert_eq!(engine.current_char, 3);
        assert_eq!(engine.words[0][2].status, CharStatus::Correct);
    }

    #[test]
    fn test_completes_immediately_on_last_letter_without_space() {
        let config = SessionConfig::default();
        let mut engine = GameEngine::new(config);
        engine.load_passage("hi");

        assert_eq!(engine.state, GameState::Idle);
        engine.handle_char('h');
        assert_eq!(engine.state, GameState::Running);
        // Type the very last letter 'i'
        engine.handle_char('i');
        // Game must complete IMMEDIATELY without space!
        assert_eq!(engine.state, GameState::Completed);
    }
}
