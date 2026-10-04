use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharStatus {
    Pending,
    Correct,
    Incorrect,
}

#[derive(Debug, Clone)]
pub struct DisplayChar {
    pub expected: char,
    pub typed: Option<char>,
    pub status: CharStatus,
    pub pop_anim: f32, // 0.0 to 1.0 (ease-out bounce for correct keystroke)
}

impl DisplayChar {
    pub fn new(expected: char) -> Self {
        Self {
            expected,
            typed: None,
            status: CharStatus::Pending,
            pop_anim: 0.0,
        }
    }
}

pub struct TextGenerator;

pub const COMMON_WORDS: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not", "on", "with",
    "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say", "her",
    "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so", "up",
    "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can", "like", "time",
    "no", "just", "him", "know", "take", "people", "into", "year", "your", "good", "some", "could",
    "them", "see", "other", "than", "then", "now", "look", "only", "come", "its", "over", "think",
    "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way", "even",
    "new", "want", "because", "any", "these", "give", "day", "most", "us", "great", "between", "need",
    "large", "under", "system", "program", "state", "point", "number", "world", "never", "high",
    "stream", "light", "code", "fast", "build", "rust", "memory", "thread", "vector", "async", "cache",
    "value", "string", "struct", "logic", "smooth", "typing", "focus", "speed", "rhythm", "cursor",
    "frame", "render", "canvas", "motion", "sound", "energy", "spark", "pulse", "spring", "drift",
    "clean", "sharp", "stable", "buffer", "future", "signal", "power", "syntax", "matrix", "pixel",
    "device", "design", "engine", "flow", "orbit", "prism", "glitch", "charge", "kinetic", "burst",
    "simple", "steady", "rapid", "fluid", "hyper", "cyber", "neural", "atomic", "quantum", "zenith",
    "silent", "breath", "shadow", "flight", "portal", "silver", "golden", "streak", "impact", "blaze",
    "echo", "source", "module", "kernel", "packet", "switch", "client", "server", "beacon", "horizon",
];

impl TextGenerator {
    pub fn generate_words(count: usize, include_punctuation: bool, include_numbers: bool) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut words = Vec::with_capacity(count);
        let punctuation_marks = [".", ",", ";", "!", "?", ":", "-"];

        for _ in 0..count {
            let is_number = include_numbers && rng.gen_bool(0.12);
            let mut word = if is_number {
                if rng.gen_bool(0.4) {
                    rng.gen_range(0..100).to_string()
                } else if rng.gen_bool(0.7) {
                    rng.gen_range(1980..2030).to_string()
                } else {
                    format!("{:.1}", rng.gen_range(1.0..99.0))
                }
            } else {
                let base = COMMON_WORDS.choose(&mut rng).unwrap_or(&"code").to_string();
                if include_punctuation && rng.gen_bool(0.15) {
                    let mut chars = base.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                } else {
                    base
                }
            };

            if include_punctuation && !is_number && rng.gen_bool(0.18) {
                let mark = punctuation_marks.choose(&mut rng).unwrap_or(&",");
                word.push_str(mark);
            }

            words.push(word);
        }

        words
    }
}
