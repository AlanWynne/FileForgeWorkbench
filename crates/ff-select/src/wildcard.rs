//! Glob-style wildcard pattern matching for string criteria values.
//!
//! Supports `*` (match zero or more characters) and `?` (match exactly one character)
//! with backslash escape support (`\*` matches literal asterisk, `\?` matches literal
//! question mark).

/// Glob-style wildcard pattern matching for string criteria values.
///
/// Addresses: Requirement 4
pub struct WildcardMatcher;

impl WildcardMatcher {
    /// Check whether a criterion value contains unescaped wildcard characters.
    ///
    /// Returns `true` if the value contains `*` or `?` that are not preceded
    /// by a backslash escape.
    ///
    /// Addresses: Requirement 4 AC 4
    pub fn has_wildcards(value: &str) -> bool {
        let chars: Vec<char> = value.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' {
                // Skip escaped character
                i += 2;
            } else if chars[i] == '*' || chars[i] == '?' {
                return true;
            } else {
                i += 1;
            }
        }
        false
    }

    /// Test whether a value matches a wildcard pattern.
    ///
    /// `*` matches zero or more characters; `?` matches exactly one character.
    /// Backslash escapes: `\*` matches literal `*`, `\?` matches literal `?`.
    ///
    /// When `case_sensitive` is `false`, comparison is case-insensitive.
    ///
    /// Addresses: Requirement 4 AC 1, 3, 6
    pub fn matches(value: &str, pattern: &str, case_sensitive: bool) -> bool {
        let value_str: Vec<char> = if case_sensitive {
            value.chars().collect()
        } else {
            value.to_lowercase().chars().collect()
        };
        let pattern_str: Vec<char> = if case_sensitive {
            pattern.chars().collect()
        } else {
            pattern.to_lowercase().chars().collect()
        };

        Self::match_recursive(&value_str, 0, &pattern_str, 0)
    }

    /// Recursive wildcard matching implementation.
    fn match_recursive(value: &[char], vi: usize, pattern: &[char], pi: usize) -> bool {
        // Both exhausted — match
        if pi >= pattern.len() && vi >= value.len() {
            return true;
        }

        // Pattern exhausted but value remains — no match
        if pi >= pattern.len() {
            return false;
        }

        // Handle escape sequences
        if pattern[pi] == '\\' && pi + 1 < pattern.len() {
            // Escaped character — must match literally
            let literal = pattern[pi + 1];
            if vi < value.len() && value[vi] == literal {
                return Self::match_recursive(value, vi + 1, pattern, pi + 2);
            }
            return false;
        }

        // Handle * — match zero or more characters
        if pattern[pi] == '*' {
            // Try matching zero characters (advance pattern past *)
            if Self::match_recursive(value, vi, pattern, pi + 1) {
                return true;
            }
            // Try matching one or more characters (advance value)
            if vi < value.len() {
                return Self::match_recursive(value, vi + 1, pattern, pi);
            }
            return false;
        }

        // Handle ? — match exactly one character
        if pattern[pi] == '?' {
            if vi < value.len() {
                return Self::match_recursive(value, vi + 1, pattern, pi + 1);
            }
            return false;
        }

        // Regular character — must match exactly
        if vi < value.len() && value[vi] == pattern[pi] {
            return Self::match_recursive(value, vi + 1, pattern, pi + 1);
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- has_wildcards tests ---

    #[test]
    fn has_wildcards_detects_star() {
        assert!(WildcardMatcher::has_wildcards("abc*def"));
    }

    #[test]
    fn has_wildcards_detects_question_mark() {
        assert!(WildcardMatcher::has_wildcards("abc?def"));
    }

    #[test]
    fn has_wildcards_ignores_escaped_star() {
        assert!(!WildcardMatcher::has_wildcards(r"abc\*def"));
    }

    #[test]
    fn has_wildcards_ignores_escaped_question_mark() {
        assert!(!WildcardMatcher::has_wildcards(r"abc\?def"));
    }

    #[test]
    fn has_wildcards_returns_false_for_no_wildcards() {
        assert!(!WildcardMatcher::has_wildcards("plain text"));
    }

    #[test]
    fn has_wildcards_detects_star_after_escaped_backslash() {
        // \\* means escaped backslash followed by literal star
        // But in our simple scheme, \\ escapes the second \, then * is unescaped
        assert!(WildcardMatcher::has_wildcards("abc\\\\*def"));
    }

    // --- matches tests ---

    #[test]
    fn matches_exact_equality() {
        assert!(WildcardMatcher::matches("hello", "hello", true));
        assert!(!WildcardMatcher::matches("hello", "world", true));
    }

    #[test]
    fn matches_star_at_end() {
        assert!(WildcardMatcher::matches("hello world", "hello*", true));
    }

    #[test]
    fn matches_star_at_beginning() {
        assert!(WildcardMatcher::matches("hello world", "*world", true));
    }

    #[test]
    fn matches_star_in_middle() {
        assert!(WildcardMatcher::matches("hello world", "hel*rld", true));
    }

    #[test]
    fn matches_star_matches_empty_string() {
        assert!(WildcardMatcher::matches("hello", "hello*", true));
    }

    #[test]
    fn matches_question_mark_exactly_one_char() {
        assert!(WildcardMatcher::matches("hello", "hell?", true));
        assert!(!WildcardMatcher::matches("hell", "hell?", true));
    }

    #[test]
    fn matches_multiple_question_marks() {
        assert!(WildcardMatcher::matches("abc", "???", true));
        assert!(!WildcardMatcher::matches("ab", "???", true));
        assert!(!WildcardMatcher::matches("abcd", "???", true));
    }

    #[test]
    fn matches_combined_star_and_question() {
        assert!(WildcardMatcher::matches("abcdef", "a?c*f", true));
        assert!(WildcardMatcher::matches("axcf", "a?c*f", true));
    }

    #[test]
    fn matches_escaped_star_matches_literal() {
        assert!(WildcardMatcher::matches("a*b", r"a\*b", true));
        assert!(!WildcardMatcher::matches("axb", r"a\*b", true));
    }

    #[test]
    fn matches_escaped_question_matches_literal() {
        assert!(WildcardMatcher::matches("a?b", r"a\?b", true));
        assert!(!WildcardMatcher::matches("axb", r"a\?b", true));
    }

    #[test]
    fn matches_case_insensitive() {
        assert!(WildcardMatcher::matches("Hello", "hello", false));
        assert!(WildcardMatcher::matches("HELLO", "h*o", false));
    }

    #[test]
    fn matches_case_sensitive_rejects_mismatch() {
        assert!(!WildcardMatcher::matches("Hello", "hello", true));
    }

    #[test]
    fn matches_empty_pattern_matches_empty_value() {
        assert!(WildcardMatcher::matches("", "", true));
    }

    #[test]
    fn matches_star_only_matches_anything() {
        assert!(WildcardMatcher::matches("anything", "*", true));
        assert!(WildcardMatcher::matches("", "*", true));
    }
}
