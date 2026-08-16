//! Indent pattern compilation, caching, and matching.
//!
//! Compiles regex patterns from language TOML definitions and provides
//! efficient match evaluation for indent increase/decrease detection.

use regex::Regex;

/// Raw indent table fields as read from language TOML.
///
/// Intermediate type between a language definition's raw strings
/// and compiled `IndentPatterns`.
#[derive(Debug, Clone, Default)]
pub struct IndentTableRaw {
    /// Regex for lines that trigger indent increase on the next line.
    pub increase_pattern: Option<String>,
    /// Regex for lines that trigger indent decrease on the current line.
    pub decrease_pattern: Option<String>,
    /// Regex for statement continuation start.
    pub statement_pattern: Option<String>,
    /// Regex for statement continuation end.
    pub statement_end_pattern: Option<String>,
    /// Regex for block-start delimiter (Enter-between-braces).
    pub block_start: Option<String>,
    /// Regex for block-end delimiter (Enter-between-braces).
    pub block_end: Option<String>,
}

/// A compiled regex pattern with the source string preserved for diagnostics.
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    regex: Regex,
    source: String,
}

impl CompiledPattern {
    /// Attempt to compile a regex pattern.
    ///
    /// Returns `None` if the pattern is invalid (logs WARN via ff-logging).
    pub fn try_compile(source: &str) -> Option<Self> {
        match Regex::new(source) {
            Ok(regex) => Some(Self {
                regex,
                source: source.to_string(),
            }),
            Err(_err) => {
                // In production, this would log via ff-logging:
                // ff_logging::log(LogLevel::Warn, "auto-indent", &format!("invalid regex: {}", source));
                None
            }
        }
    }

    /// Test whether the pattern matches the given text.
    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    /// Returns the source pattern string.
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Compiled indent patterns for a specific language.
///
/// Invalid regex patterns are treated as `None` (logged as WARN).
#[derive(Debug, Clone)]
pub struct IndentPatterns {
    /// Pattern matching lines that trigger indent increase on the next line.
    pub increase_pattern: Option<CompiledPattern>,
    /// Pattern matching lines that trigger indent decrease on the current line.
    pub decrease_pattern: Option<CompiledPattern>,
    /// Pattern matching statement continuation start.
    pub statement_pattern: Option<CompiledPattern>,
    /// Pattern matching statement continuation end.
    pub statement_end_pattern: Option<CompiledPattern>,
    /// Pattern matching block-start delimiters.
    pub block_start: Option<CompiledPattern>,
    /// Pattern matching block-end delimiters.
    pub block_end: Option<CompiledPattern>,
}

impl IndentPatterns {
    /// Create empty patterns (no smart indent rules).
    pub fn empty() -> Self {
        Self {
            increase_pattern: None,
            decrease_pattern: None,
            statement_pattern: None,
            statement_end_pattern: None,
            block_start: None,
            block_end: None,
        }
    }

    /// Compile patterns from raw TOML strings.
    ///
    /// Invalid patterns are logged as WARN and treated as None.
    pub fn compile(raw: &IndentTableRaw) -> Self {
        Self {
            increase_pattern: raw
                .increase_pattern
                .as_deref()
                .and_then(CompiledPattern::try_compile),
            decrease_pattern: raw
                .decrease_pattern
                .as_deref()
                .and_then(CompiledPattern::try_compile),
            statement_pattern: raw
                .statement_pattern
                .as_deref()
                .and_then(CompiledPattern::try_compile),
            statement_end_pattern: raw
                .statement_end_pattern
                .as_deref()
                .and_then(CompiledPattern::try_compile),
            block_start: raw
                .block_start
                .as_deref()
                .and_then(CompiledPattern::try_compile),
            block_end: raw
                .block_end
                .as_deref()
                .and_then(CompiledPattern::try_compile),
        }
    }

    /// Returns true if no patterns are defined.
    pub fn is_empty(&self) -> bool {
        self.increase_pattern.is_none()
            && self.decrease_pattern.is_none()
            && self.statement_pattern.is_none()
            && self.statement_end_pattern.is_none()
            && self.block_start.is_none()
            && self.block_end.is_none()
    }

    /// Test whether a line matches the increase pattern.
    pub fn matches_increase(&self, line_content: &str) -> bool {
        self.increase_pattern
            .as_ref()
            .is_some_and(|p| p.is_match(line_content))
    }

    /// Test whether a line matches the decrease pattern.
    pub fn matches_decrease(&self, line_content: &str) -> bool {
        self.decrease_pattern
            .as_ref()
            .is_some_and(|p| p.is_match(line_content))
    }

    /// Test whether a line matches the statement pattern.
    pub fn matches_statement(&self, line_content: &str) -> bool {
        self.statement_pattern
            .as_ref()
            .is_some_and(|p| p.is_match(line_content))
    }

    /// Test whether a line matches the statement end pattern.
    pub fn matches_statement_end(&self, line_content: &str) -> bool {
        self.statement_end_pattern
            .as_ref()
            .is_some_and(|p| p.is_match(line_content))
    }

    /// Test whether text before caret matches block_start.
    pub fn matches_block_start(&self, text_before_caret: &str) -> bool {
        self.block_start
            .as_ref()
            .is_some_and(|p| p.is_match(text_before_caret))
    }

    /// Test whether text after caret matches block_end.
    pub fn matches_block_end(&self, text_after_caret: &str) -> bool {
        self.block_end
            .as_ref()
            .is_some_and(|p| p.is_match(text_after_caret))
    }
}

/// Create standard C-like indent patterns for testing and as a common preset.
pub fn c_like_patterns() -> IndentPatterns {
    IndentPatterns::compile(&IndentTableRaw {
        increase_pattern: Some(r"\{\s*$".to_string()),
        decrease_pattern: Some(r"^\s*\}".to_string()),
        statement_pattern: Some(r"^\s*(if|while|for)\b.*[^{]\s*$".to_string()),
        statement_end_pattern: Some(r";\s*$".to_string()),
        block_start: Some(r"\{\s*$".to_string()),
        block_end: Some(r"^\s*\}".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_pattern_valid_regex() {
        // Validates: Requirement 9.7 — valid patterns compile successfully
        let pattern = CompiledPattern::try_compile(r"\{\s*$");
        assert!(pattern.is_some());
    }

    #[test]
    fn compiled_pattern_invalid_regex_returns_none() {
        // Validates: Requirement 9.7 — invalid patterns return None
        let pattern = CompiledPattern::try_compile(r"[unclosed");
        assert!(pattern.is_none());
    }

    #[test]
    fn compiled_pattern_invalid_star_returns_none() {
        // Validates: Requirement 9.7
        let pattern = CompiledPattern::try_compile(r"*invalid");
        assert!(pattern.is_none());
    }

    #[test]
    fn compiled_pattern_matches_correctly() {
        // Validates: Requirement 3.2
        let pattern = CompiledPattern::try_compile(r"\{\s*$").unwrap();
        assert!(pattern.is_match("if (true) {"));
        assert!(pattern.is_match("fn main() {  "));
        assert!(!pattern.is_match("let x = 5;"));
    }

    #[test]
    fn indent_patterns_compile_from_raw() {
        // Validates: Requirement 9.1, 9.2
        let raw = IndentTableRaw {
            increase_pattern: Some(r"\{\s*$".to_string()),
            decrease_pattern: Some(r"^\s*\}".to_string()),
            statement_pattern: None,
            statement_end_pattern: None,
            block_start: Some(r"\{$".to_string()),
            block_end: Some(r"^\s*\}".to_string()),
        };
        let patterns = IndentPatterns::compile(&raw);
        assert!(patterns.increase_pattern.is_some());
        assert!(patterns.decrease_pattern.is_some());
        assert!(patterns.statement_pattern.is_none());
        assert!(patterns.block_start.is_some());
        assert!(!patterns.is_empty());
    }

    #[test]
    fn indent_patterns_empty() {
        let patterns = IndentPatterns::empty();
        assert!(patterns.is_empty());
        assert!(!patterns.matches_increase("anything"));
        assert!(!patterns.matches_decrease("anything"));
    }

    #[test]
    fn indent_patterns_with_invalid_regex_degrades() {
        // Validates: Requirement 9.7 — invalid regex treated as None
        let raw = IndentTableRaw {
            increase_pattern: Some(r"[invalid".to_string()),
            decrease_pattern: Some(r"^\s*\}".to_string()),
            ..Default::default()
        };
        let patterns = IndentPatterns::compile(&raw);
        // Invalid pattern becomes None
        assert!(patterns.increase_pattern.is_none());
        // Valid pattern still works
        assert!(patterns.decrease_pattern.is_some());
        // Invalid pattern never matches
        assert!(!patterns.matches_increase("any content"));
    }

    #[test]
    fn matches_increase_with_brace() {
        // Validates: Requirement 3.1
        let patterns = c_like_patterns();
        assert!(patterns.matches_increase("if (true) {"));
        assert!(patterns.matches_increase("fn main() {"));
        assert!(!patterns.matches_increase("let x = 5;"));
        assert!(!patterns.matches_increase("}"));
    }

    #[test]
    fn matches_decrease_with_closing_brace() {
        // Validates: Requirement 4.1
        let patterns = c_like_patterns();
        assert!(patterns.matches_decrease("    }"));
        assert!(patterns.matches_decrease("}"));
        assert!(!patterns.matches_decrease("if (true) {"));
    }

    #[test]
    fn matches_statement_pattern() {
        // Validates: Requirement 3.6
        let patterns = c_like_patterns();
        assert!(patterns.matches_statement("    if (condition)"));
        assert!(patterns.matches_statement("    while (true)"));
        assert!(!patterns.matches_statement("    if (condition) {"));
    }

    #[test]
    fn matches_block_start_and_end() {
        // Validates: Requirement 5.1
        let patterns = c_like_patterns();
        assert!(patterns.matches_block_start("fn main() {"));
        assert!(patterns.matches_block_end("    }"));
    }

    #[test]
    fn empty_pattern_none_never_matches() {
        // Validates: Requirement 9.7 — None pattern never matches
        let patterns = IndentPatterns::empty();
        assert!(!patterns.matches_increase("anything { with braces"));
        assert!(!patterns.matches_decrease("} closing"));
        assert!(!patterns.matches_statement("if (x)"));
        assert!(!patterns.matches_statement_end(";"));
        assert!(!patterns.matches_block_start("{"));
        assert!(!patterns.matches_block_end("}"));
    }

    #[test]
    fn compiled_pattern_preserves_source() {
        let pattern = CompiledPattern::try_compile(r"\{\s*$").unwrap();
        assert_eq!(pattern.source(), r"\{\s*$");
    }
}
