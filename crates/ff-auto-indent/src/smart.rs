//! Smart-indent engine — pattern-based increase/decrease logic.
//!
//! Applies language-specific indent patterns to compute the correct
//! indentation for new lines based on the reference line content.
//!
//! Handles:
//! - Indent increase (line matches increase_pattern)
//! - Net cancellation (line matches both increase and decrease)
//! - Statement continuation (line matches statement_pattern)
//! - Fallback to maintain-indent when no patterns exist

use crate::config::IndentConfig;
use crate::decision::IndentDecision;
use crate::maintain::{compute_maintain_indent, parse_line_indent};
use crate::patterns::IndentPatterns;

/// Context for smart-indent computation.
#[derive(Debug, Clone)]
pub struct IndentContext {
    /// Content of the reference line (where Enter was pressed).
    pub reference_line: String,
    /// Column position of the caret on the reference line.
    pub caret_column: u64,
    /// Whether the caret is inside a comment.
    pub in_comment: bool,
    /// Whether the caret is inside a block comment.
    pub in_block_comment: bool,
    /// Whether the previous continuation line was empty (for double-Enter break-out).
    pub is_empty_comment_continuation: bool,
    /// Text before the caret on the reference line.
    pub text_before_caret: String,
    /// Text after the caret on the reference line.
    pub text_after_caret: String,
}

impl IndentContext {
    /// Create a simple context for testing with just the reference line and caret.
    pub fn simple(reference_line: &str, caret_column: u64) -> Self {
        let caret_col = (caret_column as usize).min(reference_line.len());
        Self {
            reference_line: reference_line.to_string(),
            caret_column,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: reference_line[..caret_col].to_string(),
            text_after_caret: reference_line[caret_col..].to_string(),
        }
    }
}

/// Compute the net indent adjustment based on pattern matching.
///
/// Returns the change in indent levels:
/// - +1 if only increase matches
/// - -1 if only decrease matches
/// - 0 if both match (cancel) or neither matches
pub fn compute_net_adjustment(patterns: &IndentPatterns, line_content: &str) -> i32 {
    let increases = patterns.matches_increase(line_content);
    let decreases = patterns.matches_decrease(line_content);

    match (increases, decreases) {
        (true, true) => 0,   // Net cancellation
        (true, false) => 1,  // Increase
        (false, true) => -1, // Decrease (unusual for reference line, but handled)
        (false, false) => 0, // No change
    }
}

/// Compute smart-indent for a newline.
///
/// Examines the reference line against indent patterns and computes
/// the appropriate indentation for the new line.
///
/// Priority:
/// 1. If no increase_pattern is defined, fall back to maintain-indent.
/// 2. If reference line matches increase_pattern (and not decrease), indent +1.
/// 3. If both increase and decrease match, net effect is 0 (maintain).
/// 4. Statement pattern: indent +1 for immediate next line only.
pub fn compute_smart_indent(
    context: &IndentContext,
    patterns: &IndentPatterns,
    config: &IndentConfig,
) -> IndentDecision {
    // Caret at column 0 → no indent regardless of patterns
    if context.caret_column == 0 {
        return IndentDecision::no_indent();
    }

    // No increase pattern defined → fall back to maintain
    if patterns.increase_pattern.is_none() && patterns.statement_pattern.is_none() {
        return compute_maintain_indent(&context.reference_line, context.caret_column, config);
    }

    let indent_info = parse_line_indent(&context.reference_line, config.tab_size());
    let reference_level = config.columns_to_level(indent_info.column_width);

    // Compute net adjustment from patterns
    let net = compute_net_adjustment(patterns, &context.reference_line);

    // Check for statement continuation
    let is_statement = patterns.matches_statement(&context.reference_line);

    let new_level = if net > 0 {
        // Increase pattern matched (and decrease did not)
        reference_level.increment()
    } else if is_statement {
        // Statement pattern matched — indent next line by one
        reference_level.increment()
    } else {
        // No increase, no statement — maintain reference level
        reference_level
    };

    let whitespace = config.whitespace_for_level(new_level);
    IndentDecision::smart(whitespace, new_level.value())
}

/// Check if a typed character completes a decrease pattern match.
///
/// Returns Some(new_indent_string) if the line should be re-indented,
/// or None if no adjustment is needed.
pub fn compute_decrease_on_type(
    current_line_content: &str,
    caret_column: u64,
    patterns: &IndentPatterns,
    config: &IndentConfig,
) -> Option<String> {
    // No decrease pattern → no adjustment
    let decrease_pattern = patterns.decrease_pattern.as_ref()?;

    // Guard: only trigger when content before the typed character is only whitespace.
    // The caret_column points AFTER the just-typed character, so we check up to caret-1.
    let caret_col = caret_column as usize;
    let check_col = caret_col.saturating_sub(1);
    let before_typed_char = if check_col <= current_line_content.len() {
        &current_line_content[..check_col]
    } else {
        current_line_content
    };

    // Check if there's non-whitespace content before the typed character
    if before_typed_char.chars().any(|ch| !ch.is_whitespace()) {
        return None;
    }

    // Check if the line matches the decrease pattern
    if !decrease_pattern.is_match(current_line_content) {
        return None;
    }

    // Compute the decreased indent level
    let indent_info = parse_line_indent(current_line_content, config.tab_size());
    let current_level = config.columns_to_level(indent_info.column_width);
    let new_level = current_level.decrement();

    // Floor clamping: never go below zero
    let new_whitespace = config.whitespace_for_level(new_level);

    // Only adjust if the new whitespace differs from current
    if config.column_width_of(&new_whitespace) < indent_info.column_width {
        Some(new_whitespace)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndentStyle;
    use crate::patterns::c_like_patterns;

    fn make_config() -> IndentConfig {
        IndentConfig::new(4, 4, IndentStyle::Spaces)
    }

    #[test]
    fn smart_indent_increase_on_brace() {
        // Validates: Requirement 3.1 — increase pattern adds one level
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext::simple("    if (true) {", 15);
        let result = compute_smart_indent(&ctx, &patterns, &config);
        assert_eq!(result.indent_text, "        "); // 8 spaces = level 2
        assert_eq!(result.indent_level, 2);
    }

    #[test]
    fn smart_indent_no_increase_maintains_level() {
        // Validates: Requirement 3.3 — no pattern match → maintain
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext::simple("    let x = 5;", 14);
        let result = compute_smart_indent(&ctx, &patterns, &config);
        assert_eq!(result.indent_text, "    "); // 4 spaces = level 1
        assert_eq!(result.indent_level, 1);
    }

    #[test]
    fn smart_indent_net_cancellation() {
        // Validates: Requirement 3.5 — both increase and decrease cancel
        let config = make_config();
        let patterns = c_like_patterns();
        // "} else {" matches both increase AND decrease
        let ctx = IndentContext::simple("    } else {", 12);
        let result = compute_smart_indent(&ctx, &patterns, &config);
        // Net effect is 0, maintains reference level
        assert_eq!(result.indent_text, "    "); // level 1 maintained
        assert_eq!(result.indent_level, 1);
    }

    #[test]
    fn smart_indent_fallback_to_maintain_when_no_patterns() {
        // Validates: Requirement 3.3 — fallback to maintain
        let config = make_config();
        let patterns = IndentPatterns::empty();
        let ctx = IndentContext::simple("    hello world", 15);
        let result = compute_smart_indent(&ctx, &patterns, &config);
        assert_eq!(result.indent_text, "    ");
    }

    #[test]
    fn smart_indent_caret_at_column_zero() {
        // Validates: Requirement 2.5 — column 0 always no indent
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext::simple("    if (true) {", 0);
        let result = compute_smart_indent(&ctx, &patterns, &config);
        assert_eq!(result.indent_text, "");
        assert_eq!(result.indent_level, 0);
    }

    #[test]
    fn smart_indent_statement_continuation() {
        // Validates: Requirement 3.6 — statement pattern indents next line
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext::simple("    if (condition)", 18);
        let result = compute_smart_indent(&ctx, &patterns, &config);
        assert_eq!(result.indent_text, "        "); // level 2
        assert_eq!(result.indent_level, 2);
    }

    #[test]
    fn smart_indent_from_zero_level() {
        // Validates: Requirement 3.1
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext::simple("fn main() {", 12);
        let result = compute_smart_indent(&ctx, &patterns, &config);
        assert_eq!(result.indent_text, "    "); // level 0 → level 1
        assert_eq!(result.indent_level, 1);
    }

    #[test]
    fn decrease_on_type_triggers_on_closing_brace() {
        // Validates: Requirement 4.1 — decrease triggers on }
        let config = make_config();
        let patterns = c_like_patterns();
        // Line is "        }" (8 spaces + }), caret at col 9
        let result = compute_decrease_on_type("        }", 9, &patterns, &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "    "); // decreased from level 2 to level 1
    }

    #[test]
    fn decrease_on_type_no_trigger_with_content_before() {
        // Validates: Requirement 4.7 — no decrease when content before caret
        let config = make_config();
        let patterns = c_like_patterns();
        // Line has content before the brace
        let result = compute_decrease_on_type("x = }", 5, &patterns, &config);
        assert!(result.is_none());
    }

    #[test]
    fn decrease_on_type_floor_at_zero() {
        // Validates: Requirement 4.6 — never goes below zero
        let config = make_config();
        let patterns = c_like_patterns();
        // Line at level 0 with closing brace
        let result = compute_decrease_on_type("}", 1, &patterns, &config);
        // Already at level 0, cannot decrease further
        assert!(result.is_none());
    }

    #[test]
    fn decrease_on_type_no_pattern() {
        // Validates: Requirement 4.3 — no decrease when pattern undefined
        let config = make_config();
        let patterns = IndentPatterns::empty();
        let result = compute_decrease_on_type("        }", 9, &patterns, &config);
        assert!(result.is_none());
    }

    #[test]
    fn decrease_on_type_no_match() {
        // Validates: Requirement 4.1
        let config = make_config();
        let patterns = c_like_patterns();
        // Line doesn't match decrease pattern
        let result = compute_decrease_on_type("        x", 9, &patterns, &config);
        assert!(result.is_none());
    }

    #[test]
    fn net_adjustment_increase_only() {
        let patterns = c_like_patterns();
        assert_eq!(compute_net_adjustment(&patterns, "if (true) {"), 1);
    }

    #[test]
    fn net_adjustment_decrease_only() {
        let patterns = c_like_patterns();
        assert_eq!(compute_net_adjustment(&patterns, "    }"), -1);
    }

    #[test]
    fn net_adjustment_both_cancel() {
        let patterns = c_like_patterns();
        assert_eq!(compute_net_adjustment(&patterns, "    } else {"), 0);
    }

    #[test]
    fn net_adjustment_neither() {
        let patterns = c_like_patterns();
        assert_eq!(compute_net_adjustment(&patterns, "let x = 5;"), 0);
    }
}
