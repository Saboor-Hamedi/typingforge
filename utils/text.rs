/// Text sanitization and utility functions.

/// Sanitizes pasted or edited text:
/// - Strips zero-width and invisible characters (\u{200B}, \u{200C}, \u{200D}, \u{FEFF})
/// - Normalizes smart single/double quotes to straight quotes
/// - Normalizes typographic dashes
/// - Collapses multiple whitespace characters into single spaces
/// - Trims leading and trailing whitespace
pub fn sanitize_text(input: &str) -> String {
    let mut cleaned = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            // Strip zero-width, byte-order-mark, and directional characters
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{2060}' | '\u{00AD}' | '\u{200E}' | '\u{200F}' => {
                // Skip completely
            }
            // Normalize single smart quotes / backticks / apostrophes / primes
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '`' | '´' | '\u{2032}' | '\u{2039}' | '\u{203A}' | '\u{FF07}' | '\u{055A}' | '\u{275B}' | '\u{275C}' => {
                cleaned.push('\'');
            }
            // Normalize double smart quotes / guillemets / primes / low quotes
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '«' | '»' | '\u{2033}' | '\u{02DD}' | '\u{FF02}' | '\u{275D}' | '\u{275E}' => {
                cleaned.push('"');
            }
            // Normalize typographic dashes to standard hyphens
            '—' | '–' | '―' | '‒' | '−' => {
                cleaned.push('-');
            }
            // Normalize typographic ellipsis
            '…' => {
                cleaned.push_str("...");
            }
            // Normalize non-standard spaces and line breaks to standard space
            '\r' | '\n' | '\t' | '\u{00A0}' | '\u{202F}' | '\u{205F}' | '\u{3000}' | '\u{2000}'..='\u{200A}' => {
                cleaned.push(' ');
            }
            other => {
                cleaned.push(other);
            }
        }
    }

    // Collapse multiple consecutive spaces into a single space and trim
    let mut result = String::with_capacity(cleaned.len());
    let mut in_space = false;

    for ch in cleaned.trim().chars() {
        if ch == ' ' {
            if !in_space {
                result.push(' ');
                in_space = true;
            }
        } else {
            result.push(ch);
            in_space = false;
        }
    }

    result
}

/// Escapes a query string for safe SQLite FTS5 MATCH queries
pub fn escape_fts5_query(query: &str) -> String {
    let sanitized = sanitize_text(query);
    // Wrap words in quotes or escape special characters
    let mut tokens = Vec::new();
    for word in sanitized.split_whitespace() {
        let clean_word: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
        if !clean_word.is_empty() {
            tokens.push(format!("\"{}\"*", clean_word));
        }
    }
    tokens.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_smart_quotes_and_zero_width() {
        let raw = "\u{FEFF}“Hello\u{200B} ‘world’!”   How’s it—going?   ";
        let cleaned = sanitize_text(raw);
        assert_eq!(cleaned, "\"Hello 'world'!\" How's it-going?");
    }

    #[test]
    fn test_sanitize_various_quotes_and_symbols() {
        let raw = "„German quotes“ and «French quotes» and ″primes″ and ‚single low‛…";
        let cleaned = sanitize_text(raw);
        assert_eq!(cleaned, "\"German quotes\" and \"French quotes\" and \"primes\" and 'single low'...");
    }

    #[test]
    fn test_escape_fts5_query() {
        let q = "speed \"test\"* OR AND";
        let escaped = escape_fts5_query(q);
        assert!(escaped.contains("\"speed\"*"));
    }
}
