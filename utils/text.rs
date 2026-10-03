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
            // Strip zero-width and byte-order-mark characters
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{2060}' | '\u{00AD}' => {
                // Skip completely
            }
            // Normalize single smart quotes / backticks / apostrophes
            '\u{2018}' | '\u{2019}' | '\u{201B}' | '`' | '´' => {
                cleaned.push('\'');
            }
            // Normalize double smart quotes / guillemets
            '\u{201C}' | '\u{201D}' | '\u{201F}' | '«' | '»' => {
                cleaned.push('"');
            }
            // Normalize typographic dashes to standard hyphens
            '—' | '–' | '―' => {
                cleaned.push('-');
            }
            // Normalize line breaks and tabs to spaces
            '\r' | '\n' | '\t' => {
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
    fn test_escape_fts5_query() {
        let q = "speed \"test\"* OR AND";
        let escaped = escape_fts5_query(q);
        assert!(escaped.contains("\"speed\"*"));
    }
}
