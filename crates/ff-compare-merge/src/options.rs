//! Comparison options: whitespace mode, case sensitivity, algorithm selection.

/// Controls how whitespace differences are treated during comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum WhitespaceMode {
    /// All whitespace is significant (default).
    #[default]
    None,
    /// Ignore leading and trailing whitespace only.
    LeadingTrailing,
    /// Ignore all whitespace differences including internal.
    All,
}

/// The algorithm used for line-level diff computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum DiffAlgorithm {
    /// Myers' greedy LCS-based algorithm — produces minimal edit script.
    #[default]
    Myers,
    /// Patience diff — anchors on unique matching lines for improved readability.
    Patience,
}

/// How comparison results are displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ViewMode {
    /// Split panel: left and right resources in separate panes.
    #[default]
    SideBySide,
    /// Single panel: unified view with interleaved changes.
    Inline,
}

/// Configuration for a comparison operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CompareOptions {
    /// How whitespace is handled during comparison.
    pub whitespace_mode: WhitespaceMode,
    /// Whether to use case-insensitive comparison.
    pub ignore_case: bool,
    /// Which diff algorithm to use.
    pub algorithm: DiffAlgorithm,
    /// Number of context lines for diff export (default: 3).
    pub context_lines: usize,
    /// View mode for displaying results.
    pub view_mode: ViewMode,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            whitespace_mode: WhitespaceMode::None,
            ignore_case: false,
            algorithm: DiffAlgorithm::Myers,
            context_lines: 3,
            view_mode: ViewMode::SideBySide,
        }
    }
}

impl CompareOptions {
    /// Normalise a line according to the current options.
    pub fn normalise<'a>(&self, line: &'a str) -> std::borrow::Cow<'a, str> {
        let s: std::borrow::Cow<str> = match self.whitespace_mode {
            WhitespaceMode::None => std::borrow::Cow::Borrowed(line),
            WhitespaceMode::LeadingTrailing => std::borrow::Cow::Owned(line.trim().to_string()),
            WhitespaceMode::All => {
                std::borrow::Cow::Owned(line.split_whitespace().collect::<Vec<_>>().join(" "))
            }
        };
        if self.ignore_case {
            std::borrow::Cow::Owned(s.to_ascii_lowercase())
        } else {
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_sensible() {
        // Validates: Requirement 11 — default options
        let opts = CompareOptions::default();
        assert_eq!(opts.whitespace_mode, WhitespaceMode::None);
        assert!(!opts.ignore_case);
        assert_eq!(opts.algorithm, DiffAlgorithm::Myers);
        assert_eq!(opts.context_lines, 3);
        assert_eq!(opts.view_mode, ViewMode::SideBySide);
    }

    #[test]
    fn normalise_none_preserves_whitespace() {
        // Validates: Requirement 11.1 — None mode preserves all whitespace
        let opts = CompareOptions::default();
        assert_eq!(opts.normalise("  hello  ").as_ref(), "  hello  ");
    }

    #[test]
    fn normalise_leading_trailing_trims() {
        // Validates: Requirement 11.2 — LeadingTrailing trims edges
        let opts = CompareOptions {
            whitespace_mode: WhitespaceMode::LeadingTrailing,
            ..Default::default()
        };
        assert_eq!(opts.normalise("  hello  ").as_ref(), "hello");
        assert_eq!(opts.normalise("  hello world  ").as_ref(), "hello world");
    }

    #[test]
    fn normalise_all_collapses_internal_whitespace() {
        // Validates: Requirement 11.2 — All mode collapses internal whitespace
        let opts = CompareOptions {
            whitespace_mode: WhitespaceMode::All,
            ..Default::default()
        };
        assert_eq!(opts.normalise("  hello   world  ").as_ref(), "hello world");
    }

    #[test]
    fn normalise_ignore_case_lowercases() {
        // Validates: Requirement 11.3 — ignore_case lowercases
        let opts = CompareOptions {
            ignore_case: true,
            ..Default::default()
        };
        assert_eq!(opts.normalise("Hello WORLD").as_ref(), "hello world");
    }

    #[test]
    fn normalise_combined_options() {
        // Validates: Requirement 11.1 + 11.3 — combined whitespace + case
        let opts = CompareOptions {
            whitespace_mode: WhitespaceMode::LeadingTrailing,
            ignore_case: true,
            ..Default::default()
        };
        assert_eq!(opts.normalise("  HELLO  ").as_ref(), "hello");
    }
}
