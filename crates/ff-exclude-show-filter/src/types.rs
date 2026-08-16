//! Core types for the exclude-show-filter crate.
//!
//! Defines enums and structs for command arguments, operation results,
//! exclusion blocks, and line commands.

use std::fmt;

// ─── Command Argument Types ─────────────────────────────────────────────────

/// Scope modifier for EXCLUDE text/regex operations.
///
/// Addresses: Requirement 2 AC 1–2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExcludeScope {
    /// Search only visible lines (default for EXCLUDE without ALL modifier).
    #[default]
    Visible,
    /// Search all lines regardless of current visibility state.
    All,
}

/// Text matching mode for EXCLUDE and SHOW operations.
///
/// Addresses: Requirement 2 AC 1, 2 AC 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextMatchMode {
    /// Case-insensitive literal substring match (default).
    #[default]
    Literal,
    /// Case-sensitive literal substring match.
    LiteralCaseSensitive,
    /// Regular expression match.
    Regex,
}

/// Arguments parsed from an EXCLUDE primary command.
///
/// Addresses: Requirement 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeArgs {
    /// EXCLUDE 'text' — literal text match on lines in scope.
    Text {
        pattern: String,
        scope: ExcludeScope,
    },
    /// EXCLUDE REGEX 'pattern' — regex match on lines in scope.
    Regex {
        pattern: String,
        scope: ExcludeScope,
    },
    /// EXCLUDE ALL — exclude every line in the document.
    All,
    /// EXCLUDE TAGGED — exclude lines with tag flag set.
    Tagged,
    /// EXCLUDE n m — exclude a specific line range (1-based inclusive).
    Range { start_line: usize, end_line: usize },
}

/// Arguments parsed from a SHOW/INCLUDE primary command.
///
/// Addresses: Requirement 3
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShowArgs {
    /// SHOW ALL — make all lines visible.
    All,
    /// SHOW EXCLUDED — make all excluded lines visible.
    Excluded,
    /// SHOW NONEXCLUDED — no-op, confirms current state.
    NonExcluded,
    /// SHOW 'text' — show excluded lines matching literal text.
    Text { pattern: String },
    /// SHOW REGEX 'pattern' — show excluded lines matching regex.
    Regex { pattern: String },
}

/// Variants of the RESET command relevant to exclusion.
///
/// Addresses: Requirement 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetVariant {
    /// RESET (no args) — clear exclusion state and delegate to other subsystems.
    Default,
    /// RESET EXCLUDED — clear only exclusion state.
    Excluded,
    /// RESET ALL — clear exclusion as part of full session reset.
    All,
}

// ─── Exclusion Block ────────────────────────────────────────────────────────

/// A contiguous range of excluded document lines.
/// Used by the viewport renderer to display placeholder lines.
///
/// Addresses: Requirement 6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExclusionBlock {
    /// First excluded document line in this block (0-based).
    pub start_line: usize,
    /// Last excluded document line in this block (0-based, inclusive).
    pub end_line: usize,
}

impl ExclusionBlock {
    /// Create a new exclusion block.
    pub fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
        }
    }

    /// Number of excluded lines in this block.
    pub fn line_count(&self) -> usize {
        self.end_line - self.start_line + 1
    }

    /// Generate placeholder text for viewport display.
    /// Format: "-- N line(s) excluded --"
    ///
    /// Addresses: Requirement 6 AC 2
    pub fn placeholder_text(&self) -> String {
        let count = self.line_count();
        format!("-- {count} line(s) excluded --")
    }

    /// Check if a document line falls within this block.
    pub fn contains(&self, doc_line: usize) -> bool {
        doc_line >= self.start_line && doc_line <= self.end_line
    }
}

impl fmt::Display for ExclusionBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ExclusionBlock[{}..={}] ({})",
            self.start_line,
            self.end_line,
            self.placeholder_text()
        )
    }
}

// ─── Operation Results ──────────────────────────────────────────────────────

/// Result of an EXCLUDE operation.
///
/// Addresses: Requirement 2 AC 8–9
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcludeResult {
    /// Number of lines whose visibility state was changed to excluded.
    pub lines_affected: usize,
    /// Status message for display in the status bar.
    pub message: String,
}

impl ExcludeResult {
    /// Create a result for an exclude operation.
    pub fn new(lines_affected: usize) -> Self {
        let message = if lines_affected > 0 {
            format!("{lines_affected} line(s) excluded")
        } else {
            "No lines matched".to_string()
        };
        Self {
            lines_affected,
            message,
        }
    }
}

impl fmt::Display for ExcludeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Result of a SHOW operation.
///
/// Addresses: Requirement 3 AC 7–8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowResult {
    /// Number of lines whose visibility state was changed to visible.
    pub lines_shown: usize,
    /// Status message for display in the status bar.
    pub message: String,
}

impl ShowResult {
    /// Create a result for a show operation.
    pub fn new(lines_shown: usize) -> Self {
        let message = if lines_shown > 0 {
            format!("{lines_shown} line(s) shown")
        } else {
            "No excluded lines matched".to_string()
        };
        Self {
            lines_shown,
            message,
        }
    }

    /// Create a result for SHOW NONEXCLUDED (no-op).
    pub fn non_excluded_noop() -> Self {
        Self {
            lines_shown: 0,
            message: "No excluded lines were modified".to_string(),
        }
    }
}

impl fmt::Display for ShowResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Result of a RESET operation.
///
/// Addresses: Requirement 4 AC 7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetResult {
    /// Number of lines restored to visibility.
    pub lines_restored: usize,
    /// Status message for display in the status bar.
    pub message: String,
}

impl ResetResult {
    /// Create a result for a reset operation.
    pub fn new(lines_restored: usize) -> Self {
        Self {
            lines_restored,
            message: format!("RESET: {lines_restored} line(s) restored to view"),
        }
    }
}

impl fmt::Display for ResetResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// ─── Line Command Types ─────────────────────────────────────────────────────

/// A resolved X/Xn/XX line command ready for execution.
///
/// Addresses: Requirement 5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCommandExclude {
    /// X — exclude a single line (0-based).
    Single { line: usize },
    /// Xn — exclude n consecutive lines starting at line (0-based).
    Count { line: usize, count: usize },
    /// XX...XX — exclude a block of lines (inclusive range, 0-based).
    Block { start: usize, end: usize },
}

// ─── Change Notification ────────────────────────────────────────────────────

/// Event emitted when exclusion state changes.
/// Consumed by viewport/scrollbar for synchronization.
///
/// Addresses: Requirement 7 AC 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusionChanged {
    /// Total number of currently excluded lines after the change.
    pub total_excluded: usize,
    /// Total number of exclusion blocks after the change.
    pub block_count: usize,
    /// Number of lines whose state changed in this operation.
    pub lines_changed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusion_block_line_count_single_line() {
        let block = ExclusionBlock::new(5, 5);
        assert_eq!(block.line_count(), 1);
    }

    #[test]
    fn exclusion_block_line_count_multiple_lines() {
        let block = ExclusionBlock::new(3, 7);
        assert_eq!(block.line_count(), 5);
    }

    #[test]
    fn exclusion_block_placeholder_text_single() {
        let block = ExclusionBlock::new(10, 10);
        assert_eq!(block.placeholder_text(), "-- 1 line(s) excluded --");
    }

    #[test]
    fn exclusion_block_placeholder_text_multiple() {
        let block = ExclusionBlock::new(0, 9);
        assert_eq!(block.placeholder_text(), "-- 10 line(s) excluded --");
    }

    #[test]
    fn exclusion_block_contains_line_within_range() {
        let block = ExclusionBlock::new(3, 7);
        assert!(block.contains(3));
        assert!(block.contains(5));
        assert!(block.contains(7));
        assert!(!block.contains(2));
        assert!(!block.contains(8));
    }

    #[test]
    fn exclusion_block_display_impl() {
        let block = ExclusionBlock::new(2, 4);
        let s = format!("{block}");
        assert!(s.contains("ExclusionBlock[2..=4]"));
        assert!(s.contains("3 line(s) excluded"));
    }

    #[test]
    fn exclude_result_with_matches() {
        let result = ExcludeResult::new(5);
        assert_eq!(result.lines_affected, 5);
        assert_eq!(result.message, "5 line(s) excluded");
    }

    #[test]
    fn exclude_result_no_matches() {
        let result = ExcludeResult::new(0);
        assert_eq!(result.message, "No lines matched");
    }

    #[test]
    fn show_result_with_matches() {
        let result = ShowResult::new(3);
        assert_eq!(result.lines_shown, 3);
        assert_eq!(result.message, "3 line(s) shown");
    }

    #[test]
    fn show_result_no_matches() {
        let result = ShowResult::new(0);
        assert_eq!(result.message, "No excluded lines matched");
    }

    #[test]
    fn show_result_nonexcluded_noop() {
        let result = ShowResult::non_excluded_noop();
        assert_eq!(result.lines_shown, 0);
        assert_eq!(result.message, "No excluded lines were modified");
    }

    #[test]
    fn reset_result_formats_correctly() {
        let result = ResetResult::new(42);
        assert_eq!(result.message, "RESET: 42 line(s) restored to view");
    }

    #[test]
    fn exclude_scope_default_is_visible() {
        assert_eq!(ExcludeScope::default(), ExcludeScope::Visible);
    }

    #[test]
    fn text_match_mode_default_is_literal() {
        assert_eq!(TextMatchMode::default(), TextMatchMode::Literal);
    }
}
