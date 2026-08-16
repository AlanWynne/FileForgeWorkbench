//! Text matching for EXCLUDE and SHOW operations.
//!
//! Provides literal (case-sensitive and case-insensitive) and regex matching
//! against individual line content strings.

use regex::Regex;

use crate::error::ExcludeFilterError;
use crate::types::TextMatchMode;

/// A compiled text matcher that can test whether a line matches a search pattern.
///
/// Addresses: Requirement 2 AC 1, 2 AC 3, Requirement 3 AC 4–5
#[derive(Debug, Clone)]
pub struct TextMatcher {
    pattern: String,
    mode: TextMatchMode,
    /// Lowercased pattern for case-insensitive literal matching.
    lower_pattern: String,
    /// Compiled regex for regex matching.
    compiled_regex: Option<Regex>,
}

impl TextMatcher {
    /// Create a new text matcher for literal (case-insensitive) matching.
    pub fn literal(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            mode: TextMatchMode::Literal,
            lower_pattern: pattern.to_lowercase(),
            compiled_regex: None,
        }
    }

    /// Create a new text matcher for literal case-sensitive matching.
    pub fn literal_case_sensitive(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            mode: TextMatchMode::LiteralCaseSensitive,
            lower_pattern: pattern.to_lowercase(),
            compiled_regex: None,
        }
    }

    /// Create a new text matcher for regex matching.
    ///
    /// Validates the regex pattern and returns an error if invalid.
    pub fn regex(pattern: &str, command: &str) -> Result<Self, ExcludeFilterError> {
        let compiled = Regex::new(pattern).map_err(|e| ExcludeFilterError::InvalidRegex {
            command: command.to_string(),
            detail: e.to_string(),
        })?;
        Ok(Self {
            pattern: pattern.to_string(),
            mode: TextMatchMode::Regex,
            lower_pattern: String::new(),
            compiled_regex: Some(compiled),
        })
    }

    /// Test whether a line's content matches this matcher's pattern.
    pub fn matches_line(&self, line_content: &str) -> bool {
        match self.mode {
            TextMatchMode::Literal => line_content.to_lowercase().contains(&self.lower_pattern),
            TextMatchMode::LiteralCaseSensitive => line_content.contains(&self.pattern),
            TextMatchMode::Regex => {
                if let Some(ref re) = self.compiled_regex {
                    re.is_match(line_content)
                } else {
                    false
                }
            }
        }
    }

    /// Get the pattern string.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Get the match mode.
    pub fn mode(&self) -> TextMatchMode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Literal Matching ───────────────────────────────────────────────

    #[test]
    fn literal_match_case_insensitive_finds_substring() {
        let matcher = TextMatcher::literal("hello");
        assert!(matcher.matches_line("say HELLO world"));
        assert!(matcher.matches_line("Hello there"));
        assert!(matcher.matches_line("hello"));
    }

    #[test]
    fn literal_match_case_insensitive_no_match() {
        let matcher = TextMatcher::literal("xyz");
        assert!(!matcher.matches_line("hello world"));
        assert!(!matcher.matches_line(""));
    }

    #[test]
    fn literal_match_case_sensitive_exact() {
        let matcher = TextMatcher::literal_case_sensitive("Hello");
        assert!(matcher.matches_line("Hello world"));
        assert!(!matcher.matches_line("hello world"));
        assert!(!matcher.matches_line("HELLO world"));
    }

    #[test]
    fn literal_match_empty_pattern_matches_everything() {
        let matcher = TextMatcher::literal("");
        assert!(matcher.matches_line("anything"));
        assert!(matcher.matches_line(""));
    }

    // ─── Regex Matching ─────────────────────────────────────────────────

    #[test]
    fn regex_match_simple_literal() {
        let matcher = TextMatcher::regex("hello", "exclude").unwrap();
        assert!(matcher.matches_line("say hello there"));
        assert!(!matcher.matches_line("goodbye"));
    }

    #[test]
    fn regex_match_dot_wildcard() {
        let matcher = TextMatcher::regex("h.llo", "exclude").unwrap();
        assert!(matcher.matches_line("hello"));
        assert!(matcher.matches_line("hallo"));
        assert!(!matcher.matches_line("hllo"));
    }

    #[test]
    fn regex_match_star_quantifier() {
        let matcher = TextMatcher::regex("ab*c", "exclude").unwrap();
        assert!(matcher.matches_line("ac")); // zero b's
        assert!(matcher.matches_line("abc")); // one b
        assert!(matcher.matches_line("abbc")); // two b's
    }

    #[test]
    fn regex_match_plus_quantifier() {
        let matcher = TextMatcher::regex("ab+c", "exclude").unwrap();
        assert!(!matcher.matches_line("ac")); // zero b's - no match
        assert!(matcher.matches_line("abc")); // one b
        assert!(matcher.matches_line("abbc")); // two b's
    }

    #[test]
    fn regex_match_anchored_start() {
        let matcher = TextMatcher::regex("^hello", "exclude").unwrap();
        assert!(matcher.matches_line("hello world"));
        assert!(!matcher.matches_line("say hello"));
    }

    #[test]
    fn regex_match_anchored_end() {
        let matcher = TextMatcher::regex("world$", "exclude").unwrap();
        assert!(matcher.matches_line("hello world"));
        assert!(!matcher.matches_line("world hello"));
    }

    #[test]
    fn regex_match_anchored_both() {
        let matcher = TextMatcher::regex("^hello$", "exclude").unwrap();
        assert!(matcher.matches_line("hello"));
        assert!(!matcher.matches_line("hello world"));
        assert!(!matcher.matches_line("say hello"));
    }

    #[test]
    fn regex_match_dot_star() {
        let matcher = TextMatcher::regex("h.*o", "exclude").unwrap();
        assert!(matcher.matches_line("hello"));
        assert!(matcher.matches_line("ho"));
        assert!(matcher.matches_line("h---o"));
    }

    #[test]
    fn regex_match_character_class() {
        let matcher = TextMatcher::regex("[0-9]+", "exclude").unwrap();
        assert!(matcher.matches_line("line 123 here"));
        assert!(!matcher.matches_line("no digits"));
    }

    #[test]
    fn regex_match_word_boundary() {
        let matcher = TextMatcher::regex(r"\bfoo\b", "exclude").unwrap();
        assert!(matcher.matches_line("foo bar"));
        assert!(matcher.matches_line("bar foo"));
        assert!(!matcher.matches_line("foobar"));
    }

    // ─── Regex Validation ───────────────────────────────────────────────

    #[test]
    fn regex_invalid_empty_pattern() {
        // Empty string is valid regex (matches everything), but we can test invalid patterns
        let result = TextMatcher::regex("(unclosed", "exclude");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid regex pattern"));
    }

    #[test]
    fn regex_invalid_unmatched_paren() {
        let result = TextMatcher::regex("(abc", "exclude");
        assert!(result.is_err());
    }

    #[test]
    fn regex_invalid_unmatched_bracket() {
        let result = TextMatcher::regex("[abc", "show");
        assert!(result.is_err());
    }

    #[test]
    fn regex_invalid_bad_quantifier() {
        let result = TextMatcher::regex("*abc", "exclude");
        assert!(result.is_err());
    }

    #[test]
    fn regex_valid_complex_pattern() {
        let matcher = TextMatcher::regex(r"^\s*//.*TODO", "exclude").unwrap();
        assert!(matcher.matches_line("  // TODO: fix this"));
        assert!(!matcher.matches_line("let x = 5; // not a TODO comment at start"));
    }
}
