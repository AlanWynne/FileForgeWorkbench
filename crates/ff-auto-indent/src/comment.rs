//! Comment continuation (block and line comments).
//!
//! Handles automatic insertion of comment continuation markers when
//! Enter is pressed inside block or line comments.

use crate::config::IndentConfig;
use crate::decision::{CommentContinuation, CommentKind, IndentDecision};
use crate::maintain::parse_line_indent;
use crate::smart::IndentContext;

/// Language-specific comment configuration for auto-continuation.
///
/// Loaded from the language TOML `[comment]` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentConfig {
    /// Block comment start delimiter (e.g., "/*").
    pub block_start: Option<String>,
    /// Block comment end delimiter (e.g., "*/").
    pub block_end: Option<String>,
    /// Block comment continuation marker (e.g., " * ").
    pub block_continue: Option<String>,
    /// Line comment prefix (e.g., "//").
    pub line_prefix: Option<String>,
    /// Whether to continue line comments on Enter.
    pub continue_line: bool,
}

impl CommentConfig {
    /// Create an empty comment config (no continuation).
    pub fn empty() -> Self {
        Self {
            block_start: None,
            block_end: None,
            block_continue: None,
            line_prefix: None,
            continue_line: false,
        }
    }

    /// Create a C-style comment config for testing.
    pub fn c_style() -> Self {
        Self {
            block_start: Some("/*".to_string()),
            block_end: Some("*/".to_string()),
            block_continue: Some(" * ".to_string()),
            line_prefix: Some("//".to_string()),
            continue_line: true,
        }
    }

    /// Returns true if block comment continuation is configured.
    pub fn has_block_continuation(&self) -> bool {
        self.block_start.is_some() && self.block_end.is_some() && self.block_continue.is_some()
    }

    /// Returns true if line comment continuation is configured and enabled.
    pub fn has_line_continuation(&self) -> bool {
        self.continue_line && self.line_prefix.is_some()
    }
}

impl Default for CommentConfig {
    fn default() -> Self {
        Self::empty()
    }
}

/// Raw comment table fields as read from language TOML.
#[derive(Debug, Clone, Default)]
pub struct CommentTableRaw {
    /// Block comment start delimiter.
    pub block_start: Option<String>,
    /// Block comment end delimiter.
    pub block_end: Option<String>,
    /// Block comment continuation marker.
    pub block_continue: Option<String>,
    /// Line comment prefix.
    pub line_prefix: Option<String>,
    /// Whether to continue line comments on Enter.
    pub continue_line: Option<bool>,
}

impl CommentConfig {
    /// Build a `CommentConfig` from raw TOML table values.
    pub fn from_raw(raw: &CommentTableRaw) -> Self {
        Self {
            block_start: raw.block_start.clone(),
            block_end: raw.block_end.clone(),
            block_continue: raw.block_continue.clone(),
            line_prefix: raw.line_prefix.clone(),
            continue_line: raw.continue_line.unwrap_or(false),
        }
    }
}

/// Detect if a line consists only of whitespace + a continuation marker (no content after).
///
/// Used for the "double-Enter break-out" logic.
pub fn is_empty_continuation(line_text: &str, comment_config: &CommentConfig) -> bool {
    let trimmed = line_text.trim();

    // Check block continue marker
    if let Some(block_continue) = &comment_config.block_continue {
        let marker_trimmed = block_continue.trim();
        if trimmed == marker_trimmed {
            return true;
        }
        // Also check with asterisk only (common case: " * " → "*")
        if trimmed == "*" && marker_trimmed.contains('*') {
            return true;
        }
    }

    // Check line comment prefix
    if let Some(line_prefix) = &comment_config.line_prefix {
        let prefix_trimmed = line_prefix.trim();
        if trimmed == prefix_trimmed {
            return true;
        }
    }

    false
}

/// Compute comment continuation for a newline inside a comment.
///
/// Returns `Some(IndentDecision)` with the continuation marker if the caret
/// is inside a comment and continuation is configured, or `None` otherwise.
pub fn compute_comment_continuation(
    context: &IndentContext,
    comment_config: &CommentConfig,
    config: &IndentConfig,
) -> Option<IndentDecision> {
    // Must be inside a comment (syntax-state based detection)
    if !context.in_comment && !context.in_block_comment {
        // Fallback: check if reference line looks like a comment
        if !is_reference_line_comment(&context.reference_line, comment_config) {
            return None;
        }
    }

    // Double-Enter break-out: if the reference line is an empty continuation,
    // signal removal rather than adding another marker
    if context.is_empty_comment_continuation {
        // Return no indent — the caller should remove the previous marker
        return None;
    }

    let indent_info = parse_line_indent(&context.reference_line, config.tab_size());

    // Block comment continuation
    if context.in_block_comment || is_in_block_comment(&context.reference_line, comment_config) {
        if let Some(block_end) = &comment_config.block_end {
            // Don't continue on the closing line
            if context.reference_line.contains(block_end.as_str()) {
                return None;
            }
        }

        if let Some(block_continue) = &comment_config.block_continue {
            let whitespace = config.whitespace_for_columns(indent_info.column_width);
            let level = config.columns_to_level(indent_info.column_width).value();
            let mut decision = IndentDecision::maintain(whitespace, level);
            decision.comment_continuation = Some(CommentContinuation {
                marker: block_continue.clone(),
                kind: CommentKind::Block,
            });
            return Some(decision);
        }
    }

    // Line comment continuation
    if comment_config.has_line_continuation()
        && is_line_comment(&context.reference_line, comment_config)
    {
        if let Some(line_prefix) = &comment_config.line_prefix {
            let whitespace = config.whitespace_for_columns(indent_info.column_width);
            let level = config.columns_to_level(indent_info.column_width).value();
            let marker = format!("{} ", line_prefix);
            let mut decision = IndentDecision::maintain(whitespace, level);
            decision.comment_continuation = Some(CommentContinuation {
                marker,
                kind: CommentKind::Line,
            });
            return Some(decision);
        }
    }

    None
}

/// Check if a reference line is inside a block comment (text-based heuristic).
fn is_in_block_comment(line: &str, config: &CommentConfig) -> bool {
    let trimmed = line.trim();
    if let Some(block_continue) = &config.block_continue {
        let marker = block_continue.trim();
        if trimmed.starts_with(marker) {
            return true;
        }
    }
    if let Some(block_start) = &config.block_start {
        if trimmed.starts_with(block_start.as_str())
            && !trimmed.contains(config.block_end.as_deref().unwrap_or(""))
        {
            return true;
        }
    }
    false
}

/// Check if a reference line is a line comment.
fn is_line_comment(line: &str, config: &CommentConfig) -> bool {
    if let Some(prefix) = &config.line_prefix {
        let trimmed = line.trim_start();
        return trimmed.starts_with(prefix.as_str());
    }
    false
}

/// Check if the reference line looks like any kind of comment.
fn is_reference_line_comment(line: &str, config: &CommentConfig) -> bool {
    is_in_block_comment(line, config) || is_line_comment(line, config)
}

/// Determine if the "double-Enter break-out" should apply.
///
/// Returns true if the previous line contained only whitespace + continuation marker.
pub fn should_break_comment_continuation(
    previous_line: &str,
    comment_config: &CommentConfig,
) -> bool {
    is_empty_continuation(previous_line, comment_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndentStyle;

    fn make_config() -> IndentConfig {
        IndentConfig::new(4, 4, IndentStyle::Spaces)
    }

    fn make_context(reference_line: &str, in_block_comment: bool) -> IndentContext {
        IndentContext {
            reference_line: reference_line.to_string(),
            caret_column: reference_line.len() as u64,
            in_comment: in_block_comment,
            in_block_comment,
            is_empty_comment_continuation: false,
            text_before_caret: reference_line.to_string(),
            text_after_caret: String::new(),
        }
    }

    #[test]
    fn block_comment_continuation() {
        // Validates: Requirement 6.1 — block comment continues with marker
        let config = make_config();
        let comment_config = CommentConfig::c_style();
        let ctx = make_context("     * some comment text", true);
        let result = compute_comment_continuation(&ctx, &comment_config, &config);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(
            decision.comment_continuation.as_ref().unwrap().marker,
            " * "
        );
        assert_eq!(
            decision.comment_continuation.as_ref().unwrap().kind,
            CommentKind::Block
        );
    }

    #[test]
    fn line_comment_continuation() {
        // Validates: Requirement 6.2 — line comment continues with prefix
        let config = make_config();
        let comment_config = CommentConfig::c_style();
        let ctx = IndentContext {
            reference_line: "    // some comment".to_string(),
            caret_column: 19,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "    // some comment".to_string(),
            text_after_caret: String::new(),
        };
        let result = compute_comment_continuation(&ctx, &comment_config, &config);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(
            decision.comment_continuation.as_ref().unwrap().marker,
            "// "
        );
        assert_eq!(
            decision.comment_continuation.as_ref().unwrap().kind,
            CommentKind::Line
        );
    }

    #[test]
    fn no_continuation_when_not_in_comment() {
        // Validates: Requirement 6.7
        let config = make_config();
        let comment_config = CommentConfig::c_style();
        let ctx = make_context("    let x = 5;", false);
        let result = compute_comment_continuation(&ctx, &comment_config, &config);
        assert!(result.is_none());
    }

    #[test]
    fn no_continuation_on_block_end_line() {
        // Validates: Requirement 6.4 — no continue on closing line
        let config = make_config();
        let comment_config = CommentConfig::c_style();
        let ctx = make_context("     */", true);
        let result = compute_comment_continuation(&ctx, &comment_config, &config);
        assert!(result.is_none());
    }

    #[test]
    fn no_continuation_when_config_empty() {
        // Validates: Requirement 6.3
        let config = make_config();
        let comment_config = CommentConfig::empty();
        let ctx = make_context("    // comment", false);
        let result = compute_comment_continuation(&ctx, &comment_config, &config);
        assert!(result.is_none());
    }

    #[test]
    fn no_continuation_when_continue_line_disabled() {
        // Validates: Requirement 6.2
        let config = make_config();
        let comment_config = CommentConfig {
            line_prefix: Some("//".to_string()),
            continue_line: false,
            ..CommentConfig::empty()
        };
        let ctx = IndentContext {
            reference_line: "    // comment".to_string(),
            caret_column: 14,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "    // comment".to_string(),
            text_after_caret: String::new(),
        };
        let result = compute_comment_continuation(&ctx, &comment_config, &config);
        assert!(result.is_none());
    }

    #[test]
    fn double_enter_break_out() {
        // Validates: Requirement 6.6 — double-Enter removes continuation
        let config = make_config();
        let comment_config = CommentConfig::c_style();
        let ctx = IndentContext {
            reference_line: "     * ".to_string(),
            caret_column: 7,
            in_comment: true,
            in_block_comment: true,
            is_empty_comment_continuation: true,
            text_before_caret: "     * ".to_string(),
            text_after_caret: String::new(),
        };
        let result = compute_comment_continuation(&ctx, &comment_config, &config);
        // Should return None to signal break-out
        assert!(result.is_none());
    }

    #[test]
    fn is_empty_continuation_block() {
        let config = CommentConfig::c_style();
        assert!(is_empty_continuation("     * ", &config));
        assert!(is_empty_continuation("  *", &config));
        assert!(!is_empty_continuation("     * some text", &config));
    }

    #[test]
    fn is_empty_continuation_line() {
        let config = CommentConfig::c_style();
        assert!(is_empty_continuation("    //", &config));
        assert!(!is_empty_continuation("    // text", &config));
    }

    #[test]
    fn should_break_returns_true_for_empty_continuation() {
        // Validates: Requirement 6.6
        let config = CommentConfig::c_style();
        assert!(should_break_comment_continuation("     * ", &config));
        assert!(!should_break_comment_continuation("     * text", &config));
    }
}
