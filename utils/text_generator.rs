//! Public-domain literature sentence generator & normalizer.
//! Ported and upgraded from Python `generateText.py` into native, high-performance Rust.
//!
//! Features:
//! - Full Unicode normalization of quotes (“ ” „ ‟ « » ″ → ") and apostrophes (‘ ’ ‚ ‛ ′ → ')
//! - Cleans Gutenberg headers/footers, formatting artifacts, and zero-width spaces
//! - Sentence segmentation with strict length, letter-ratio, and punctuation filters
//! - Slices into exact 25-word and 40-word practice chunks
//! - Ultra-fast batch streaming into SQLite with incremental transaction commits
//! - Supports any batch size from 1 to 10,000,000 with live atomic progress and cancellation

use crate::db::DatabaseConnection;
use crate::utils::text::sanitize_text;
use chrono::Utc;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub const SOURCES: &[&str] = &[
    "https://www.gutenberg.org/files/1342/1342-0.txt", // Pride and Prejudice
    "https://www.gutenberg.org/files/2701/2701-0.txt", // Moby Dick
    "https://www.gutenberg.org/files/84/84-0.txt",     // Frankenstein
    "https://www.gutenberg.org/files/1661/1661-0.txt", // Sherlock Holmes
    "https://www.gutenberg.org/files/98/98-0.txt",     // A Tale of Two Cities
    "https://www.gutenberg.org/files/2600/2600-0.txt", // War and Peace
    "https://www.gutenberg.org/files/1400/1400-0.txt", // Great Expectations
    "https://www.gutenberg.org/files/345/345-0.txt",   // Dracula
    "https://www.gutenberg.org/files/11/11-0.txt",     // Alice in Wonderland
    "https://www.gutenberg.org/files/74/74-0.txt",     // Tom Sawyer
];

/// Rich built-in public-domain classic literature paragraphs.
/// Ensures zero-dependency, instantaneous, and offline generation of authentic text.
pub const CLASSIC_CORPUS: &str = r#"
It was the best of times, it was the worst of times, it was the age of wisdom,
it was the age of foolishness, it was the epoch of belief, it was the epoch of
incredulity, it was the season of Light, it was the season of Darkness, it was
the spring of hope, it was the winter of despair. We had everything before us,
we had nothing before us, we were all going direct to Heaven, we were all going
direct the other way. In short, the period was so far like the present period,
that some of its noisiest authorities insisted on its being received, for good
or for evil, in the superlative degree of comparison only.

Call me Ishmael. Some years ago, never mind how long precisely, having little
or no money in my purse, and nothing particular to interest me on shore, I
thought I would sail about a little and see the watery part of the world. It is
a way I have of driving off the spleen and regulating the circulation.
Whenever I find myself growing grim about the mouth; whenever it is a damp,
drizzly November in my soul; whenever I find myself involuntarily pausing before
coffin warehouses, and bringing up the rear of every funeral I meet; and
especially whenever my hypos get such an upper hand of me, that it requires a
strong moral principle to prevent me from deliberately stepping into the street,
and methodically knocking people's hats off, then, I account it high time to get
to sea as soon as I can.

It is a truth universally acknowledged, that a single man in possession of a
good fortune, must be in want of a wife. However little known the feelings or
views of such a man may be on his first entering a neighbourhood, this truth is
so well fixed in the minds of the surrounding families, that he is considered
the rightful property of some one or other of their daughters. My dear Mr.
Bennet, said his lady to him one day, have you heard that Netherfield Park is let
at last? Mr. Bennet replied that he had not. But it is, returned she; for Mrs.
Long has just been here, and she told me all about it. Mr. Bennet made no
answer. Do you not want to know who has taken it? cried his wife impatiently.
You want to tell me, and I have no objection to hearing it.

To Sherlock Holmes she is always the woman. I have seldom heard him mention her
under any other name. In his eyes she eclipses and predominates the whole of her
sex. It was not that he felt any emotion akin to love for Irene Adler. All
emotions, and that one particularly, were abhorrent to his cold, precise but
admirably balanced mind. He was, I take it, the most perfect reasoning and
observing machine that the world has seen, but as a lover he would have placed
himself in a false position. He never spoke of the softer passions, save with a
gibe and a sneer. They were admirable things for the observer, excellent for
drawing the veil from men's motives and actions.

You will rejoice to hear that no disaster has accompanied the commencement of
an enterprise which you have regarded with such evil forebodings. I arrived here
yesterday, and my first task is to assure my dear sister of my welfare and
increasing confidence in the success of my undertaking. I am already far north
of London, and as I walk in the streets of Petersburgh, I feel a cold northern
breeze play upon my cheeks, which braces my nerves and fills me with delight. Do
you understand this feeling? This breeze, which has travelled from the regions
towards which I am advancing, gives me a foretaste of those icy climes.

Alice was beginning to get very tired of sitting by her sister on the bank, and
of having nothing to do: once or twice she had peeped into the book her sister
was reading, but it had no pictures or conversations in it, and what is the use
of a book, thought Alice, without pictures or conversations? So she was
considering in her own mind, as well as she could, for the hot day made her feel
very sleepy and stupid, whether the pleasure of making a daisy-chain would be
worth the trouble of getting up and picking the daisies, when suddenly a White
Rabbit with pink eyes ran close by her.

Thirty years ago, Marseilles lay burning in the sun, one day. A blazing sun upon
a fierce August day was no greater rarity in southern France then, than at any
other time, before or since. Everything in Marseilles, and about Marseilles, had
stared at the scorching sun, and had staring been so long; and had lost all
strength of staring more, that it lay down in the heat, in a faint, and gave it
up for lost. The street was as hot as an oven, and the wall as heated as a
furnace.

There was no possibility of taking a walk that day. We had been wandering,
indeed, in the leafless shrubbery an hour in the morning; but since dinner the
cold winter wind had brought with it clouds so sombre, and a rain so
penetrating, that further outdoor exercise was now out of the question. I was
glad of it: I never liked long walks, especially on chilly afternoons: dreadful
to me was the coming home in the raw twilight, with nipped fingers and toes, and
a heart saddened by the chidings of Bessie, the nurse, and humbled by the
consciousness of my physical inferiority.

It is a curious thing, the death of a loved one. We all know that our time in this
world is limited, and that eventually all of us will go on any journey that
lies beyond, and yet we are always surprised and deeply grieved when it happens
to someone we know. It is like walking up the stairs to your bedroom in the
dark, and thinking there is one more stair than there is. Your foot falls down,
through the air, and there is a sickly moment of dark surprise as you try and
readjust the way you thought of things.
"#;

/// Represents an exported JSON sentence record matching the schema expected by `velotype_sentences.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneratedPassageRecord {
    /// Sequential passage identifier (1-indexed).
    pub id: usize,
    /// Fully normalized passage text with standard ASCII quotes and punctuation.
    pub text: String,
    /// Category classification ("prose", "quotes", "custom").
    pub category: String,
    /// Number of whitespace-separated words in the text.
    pub word_count: usize,
    /// ISO 8601 UTC creation timestamp string.
    pub date: String,
}

/// Native engine for fetching, splitting, normalizing, and slicing public-domain literature.
pub struct TextGenerator;

impl TextGenerator {
    /// Strips Project Gutenberg header/footer markers, metadata, and raw carriage returns.
    /// Removes Project Gutenberg licensing preambles, header blocks, and legal footers.
    ///
    /// Detects markers such as `*** START OF THE PROJECT GUTENBERG EBOOK` and
    /// `*** END OF THE PROJECT GUTENBERG EBOOK`, returning only the core literary content.
    pub fn clean_gutenberg(text: &str) -> String {
        let mut cleaned = text.to_string();

        // Strip START header
        if let Some(start_pos) = cleaned.find("*** START OF") {
            if let Some(end_header) = cleaned[start_pos..].find("***\n") {
                cleaned = cleaned[start_pos + end_header + 4..].to_string();
            } else if let Some(end_header) = cleaned[start_pos..].find("***\r\n") {
                cleaned = cleaned[start_pos + end_header + 5..].to_string();
            }
        }

        // Strip END footer
        if let Some(end_pos) = cleaned.find("*** END OF") {
            cleaned = cleaned[..end_pos].to_string();
        }

        // Normalize carriage returns
        cleaned = cleaned.replace("\r\n", "\n").replace('\r', "\n");
        cleaned
    }

    /// Splits continuous literary text into well-formed, complete sentences.
    ///
    /// Applies strict quality filters:
    /// - Character length between 25 and 600 characters.
    /// - Minimum 80% alphabetical character ratio (eliminates tables, ascii art, and raw numbers).
    /// - Ensures terminating sentence punctuation (`.`, `!`, `?`).
    pub fn split_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let normalized = sanitize_text(text);

        // Simple and robust sentence splitting
        let mut current = String::new();
        let chars: Vec<char> = normalized.chars().collect();
        let len = chars.len();

        let mut i = 0;
        while i < len {
            let ch = chars[i];
            current.push(ch);

            // Check if sentence boundary
            if ch == '.' || ch == '!' || ch == '?' {
                let mut is_boundary = false;
                if i + 1 >= len {
                    is_boundary = true;
                } else if chars[i + 1].is_whitespace() {
                    // Peek ahead to next non-space char
                    let mut j = i + 1;
                    while j < len && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j >= len || chars[j].is_uppercase() || chars[j] == '"' || chars[j] == '\'' {
                        is_boundary = true;
                    }
                }

                if is_boundary {
                    let candidate = current.trim();
                    if candidate.len() >= 25 && candidate.len() <= 600 {
                        let letter_count = candidate.chars().filter(|c| c.is_alphabetic() || c.is_whitespace()).count();
                        let ratio = letter_count as f32 / candidate.len().max(1) as f32;
                        if ratio >= 0.80 {
                            let mut s = candidate.to_string();
                            if !s.ends_with('.') && !s.ends_with('!') && !s.ends_with('?') && !s.ends_with('"') {
                                s.push('.');
                            }
                            sentences.push(s);
                        }
                    }
                    current.clear();
                }
            }
            i += 1;
        }

        if !current.trim().is_empty() {
            let candidate = current.trim();
            if candidate.len() >= 25 && candidate.len() <= 600 {
                let mut s = candidate.to_string();
                if !s.ends_with('.') && !s.ends_with('!') && !s.ends_with('?') {
                    s.push('.');
                }
                sentences.push(s);
            }
        }

        sentences
    }

    /// Slices complete sentences into exact target word chunks (e.g. 25 or 40 words).
    ///
    /// Preserves grammatical continuity, normalizes internal quote characters, ensures
    /// valid terminal punctuation, and skips duplicate chunks via the `used` set.
    pub fn slice_to_words(
        sentences: &[String],
        target_words: usize,
        needed: usize,
        used: &mut HashSet<String>,
    ) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut buffer: Vec<&str> = Vec::new();
        let mut buf_count = 0;

        for s in sentences {
            let words: Vec<&str> = s.split_whitespace().collect();
            let count = words.len();

            if count > target_words {
                // If sentence alone is larger than target, take an exact target slice
                let slice = words[..target_words].join(" ");
                let mut clean_slice = sanitize_text(&slice);
                if !clean_slice.ends_with('.') && !clean_slice.ends_with('!') && !clean_slice.ends_with('?') {
                    clean_slice.push('.');
                }
                if !used.contains(&clean_slice) {
                    used.insert(clean_slice.clone());
                    chunks.push(clean_slice);
                    if chunks.len() >= needed {
                        return chunks;
                    }
                }
                continue;
            }

            if buf_count + count > target_words {
                buffer.clear();
                buf_count = 0;
            }

            buffer.push(s);
            buf_count += count;

            if buf_count == target_words {
                let chunk = buffer.join(" ");
                let clean_chunk = sanitize_text(&chunk);
                if !used.contains(&clean_chunk) {
                    used.insert(clean_chunk.clone());
                    chunks.push(clean_chunk);
                    if chunks.len() >= needed {
                        return chunks;
                    }
                }
                buffer.clear();
                buf_count = 0;
            }
        }

        // If not enough exact chunks, pad with truncated sentences to meet target
        if chunks.len() < needed {
            for s in sentences {
                let words: Vec<&str> = s.split_whitespace().collect();
                if words.len() >= target_words {
                    let slice = words[..target_words].join(" ");
                    let mut clean_slice = sanitize_text(&slice);
                    if !clean_slice.ends_with('.') && !clean_slice.ends_with('!') && !clean_slice.ends_with('?') {
                        clean_slice.push('.');
                    }
                    if !used.contains(&clean_slice) {
                        used.insert(clean_slice.clone());
                        chunks.push(clean_slice);
                        if chunks.len() >= needed {
                            return chunks;
                        }
                    }
                }
            }
        }

        chunks
    }

    /// Fetches texts from public-domain sources or falls back to the embedded classic corpus.
    ///
    /// When online, attempts background HTTP downloads via curl without opening console windows.
    /// If offline or if requests timeout, automatically uses the rich offline [`CLASSIC_CORPUS`].
    pub fn fetch_corpus() -> String {
        let mut corpus = Vec::new();

        // Attempt curl on Windows/Unix if online
        for url in SOURCES {
            let curl_prog = if cfg!(target_os = "windows") { "curl.exe" } else { "curl" };
            let mut cmd = std::process::Command::new(curl_prog);
            cmd.stdin(std::process::Stdio::null());
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            if let Ok(out) = cmd
                .args(["-s", "--connect-timeout", "4", "--max-time", "8", url])
                .output()
            {
                if out.status.success() && !out.stdout.is_empty() {
                    if let Ok(raw) = String::from_utf8(out.stdout) {
                        let cleaned = Self::clean_gutenberg(&raw);
                        if cleaned.len() > 1000 {
                            corpus.push(cleaned);
                            if corpus.len() >= 3 {
                                break; // 3 books is plenty for hundreds of thousands of passages
                            }
                        }
                    }
                }
            }
        }

        // Always merge with the high-quality built-in classic corpus
        corpus.push(CLASSIC_CORPUS.to_string());
        corpus.join("\n\n")
    }

    /// Generates structured records and streams them directly into the SQLite database.
    ///
    /// Architecture:
    /// - Operates in incremental chunks of 500-1,000 records per SQLite transaction to prevent
    ///   memory spikes and locking delays.
    /// - Emits real-time progress to an atomic counter (`progress`) for UI progress bar rendering.
    /// - Checks an atomic cancellation token (`cancel`) before every batch to permit instant aborts.
    /// - Fully sanitizes and normalizes every passage (`sanitize_text`), ensuring zero weird quotes.
    ///
    /// Supports scaling from 10 to 10,000,000 records safely.
    pub fn generate_into_database(
        db: &DatabaseConnection,
        total_requested: usize,
        progress: Option<Arc<AtomicUsize>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<usize, String> {
        if total_requested == 0 {
            return Err("Requested count must be greater than zero.".to_string());
        }

        let raw_corpus = Self::fetch_corpus();
        let mut sentences = Self::split_sentences(&raw_corpus);

        if sentences.len() < 10 {
            sentences = Self::split_sentences(CLASSIC_CORPUS);
        }

        let mut rng = rand::rngs::StdRng::seed_from_u64(2026);
        let mut used = HashSet::new();
        let mut total_inserted = 0usize;

        let batch_size = 500.min(total_requested);
        let categories = ["prose", "quotes"];

        while total_inserted < total_requested {
            if let Some(c) = cancel.as_ref() {
                if c.load(Ordering::Relaxed) {
                    break;
                }
            }

            let remaining = total_requested - total_inserted;
            let current_batch_size = batch_size.min(remaining);
            let mut batch_records: Vec<(String, String, bool)> = Vec::with_capacity(current_batch_size);

            let word_targets = [25, 40];
            let mut target_idx = 0;

            while batch_records.len() < current_batch_size {
                if let Some(c) = cancel.as_ref() {
                    if c.load(Ordering::Relaxed) {
                        break;
                    }
                }

                sentences.shuffle(&mut rng);
                let wt = word_targets[target_idx % word_targets.len()];
                let cat = categories[(total_inserted + batch_records.len()) % categories.len()];
                target_idx += 1;

                let chunks = Self::slice_to_words(&sentences, wt, current_batch_size - batch_records.len(), &mut used);
                if chunks.is_empty() {
                    // Clear used set if exhausted and reshuffle
                    used.clear();
                    sentences.shuffle(&mut rng);
                    continue;
                }

                for chunk in chunks {
                    // Absolute normalization: ensures all quotes are standard ASCII " and '
                    let normalized = sanitize_text(&chunk);
                    batch_records.push((normalized, cat.to_string(), true));
                    if batch_records.len() >= current_batch_size {
                        break;
                    }
                }
            }

            if batch_records.is_empty() {
                break;
            }

            let borrowed: Vec<(&str, &str, bool)> = batch_records
                .iter()
                .map(|(t, c, is_c)| (t.as_str(), c.as_str(), *is_c))
                .collect();

            match crate::db::DbQueries::insert_passages_batch(db, &borrowed) {
                Ok(n) => {
                    total_inserted += n;
                    if let Some(p) = progress.as_ref() {
                        p.store(total_inserted, Ordering::Relaxed);
                    }
                }
                Err(e) => return Err(format!("Failed to insert batch: {e}")),
            }
        }

        Ok(total_inserted)
    }

    /// Generates structured records and outputs to a formatted JSON file.
    ///
    /// Preserves exact compatibility with `velotype_sentences.json` / Python output.
    pub fn generate_to_json_file(path: &Path, count: usize) -> Result<usize, String> {
        let raw_corpus = Self::fetch_corpus();
        let mut sentences = Self::split_sentences(&raw_corpus);

        if sentences.len() < 10 {
            sentences = Self::split_sentences(CLASSIC_CORPUS);
        }

        let mut rng = rand::rngs::StdRng::seed_from_u64(2026);
        let mut used = HashSet::new();
        let mut records = Vec::with_capacity(count);
        let now = Utc::now().to_rfc3339();

        let buckets = [
            ("prose", 25),
            ("quotes", 25),
            ("prose", 40),
            ("quotes", 40),
        ];

        let per_bucket = (count / buckets.len()).max(1);

        for &(cat, wc) in &buckets {
            let mut collected = 0;
            let mut safety = 0;

            while collected < per_bucket && records.len() < count && safety < 100 {
                safety += 1;
                sentences.shuffle(&mut rng);
                let needed = per_bucket - collected;
                let chunks = Self::slice_to_words(&sentences, wc, needed, &mut used);
                for c in chunks {
                    let normalized = sanitize_text(&c);
                    let actual_wc = normalized.split_whitespace().count();
                    records.push(GeneratedPassageRecord {
                        id: records.len() + 1,
                        text: normalized,
                        category: cat.to_string(),
                        word_count: actual_wc,
                        date: now.clone(),
                    });
                    collected += 1;
                    if records.len() >= count {
                        break;
                    }
                }
            }
        }

        let output = serde_json::json!({
            "records": records
        });

        let json_str = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("Failed to serialize JSON: {e}"))?;

        std::fs::write(path, json_str)
            .map_err(|e| format!("Failed to write file {}: {e}", path.display()))?;

        Ok(records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences_and_clean() {
        let text = "“Call me Ishmael,” he said. Some years ago, never mind how long precisely, having little or no money in my purse! Is this true?";
        let sentences = TextGenerator::split_sentences(text);
        assert!(!sentences.is_empty());
        for s in &sentences {
            assert!(s.ends_with('.') || s.ends_with('!') || s.ends_with('?') || s.ends_with('"'));
            // Check that curly quotes are normalized to straight quotes
            assert!(!s.contains('“'));
            assert!(!s.contains('”'));
        }
    }

    #[test]
    fn test_slice_to_words() {
        let sentences = TextGenerator::split_sentences(CLASSIC_CORPUS);
        let mut used = HashSet::new();
        let chunks_25 = TextGenerator::slice_to_words(&sentences, 25, 4, &mut used);
        assert!(!chunks_25.is_empty());
        for chunk in &chunks_25 {
            let wc = chunk.split_whitespace().count();
            assert_eq!(wc, 25);
            assert!(!chunk.contains('“'));
            assert!(!chunk.contains('”'));
        }
    }
}
