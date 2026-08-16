//! Block expansion logic (Enter between braces).
//!
//! Detects when Enter is pressed between matching block-start and block-end
//! patterns and generates the three-line expansion with correct indentation.
//!
//! Example: pressing Enter between `{` and `}` produces:
//! ```text
//! {
//!     |  ← caret here, indented one level
//! }
//! ```

use crate::config::IndentConfig;
use crate::decision::{BlockExpansion, IndentDecision};
use crate::maintain::parse_line_indent;
use crate::patterns::IndentPatterns;
use crate::smart::IndentContext;

/// Attempt to expand a block when Enter is pressed between block-start and block-end.
///
/// Returns `Some(IndentDecision)` with block expansion if the caret is between
/// matching block delimiters, or `None` if no expansion applies.
///
/// The expansion produces:
/// - Middle line: indented one level deeper than the reference line
/// - Closing line: at the same indent level as the reference line
pub fn try_block_expansion(
    context: &IndentContext,
    patterns: &IndentPatterns,
    config: &IndentConfig,
) -> Option<IndentDecision> {
    // Both block_start and block_end must be defined
    if patterns.block_start.is_none() || patterns.block_end.is_none() {
        return None;
    }

    // Check: text before caret matches block_start, text after caret matches block_end
    if !patterns.matches_block_start(&context.text_before_caret) {
        return None;
    }
    if !patterns.matches_block_end(&context.text_after_caret) {
        return None;
    }

    // Compute indentation levels
    let indent_info = parse_line_indent(&context.reference_line, config.tab_size());
    let reference_level = config.columns_to_level(indent_info.column_width);

    // Middle line: one level deeper
    let middle_level = reference_level.increment();
    let middle_whitespace = config.whitespace_for_level(middle_level);

    // Closing line: same level as reference (the opening line)
    let closing_whitespace = config.whitespace_for_level(reference_level);

    // The closing text is derived from text_after_caret (trimmed leading whitespace)
    let closing_text = context.text_after_caret.trim_start().to_string();

    let expansion = BlockExpansion {
        closing_text,
        closing_indent: closing_whitespace,
    };

    Some(IndentDecision::block_expand(
        middle_whitespace,
        middle_level.value(),
        expansion,
    ))
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
    fn block_expansion_between_braces() {
        // Validates: Requirement 5.1 — Enter between {} produces 3-line expansion
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext {
            reference_line: "    fn main() {}".to_string(),
            caret_column: 15,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "    fn main() {".to_string(),
            text_after_caret: "}".to_string(),
        };

        let result = try_block_expansion(&ctx, &patterns, &config);
        assert!(result.is_some());
        let decision = result.unwrap();
        // Middle line: level 2 (reference is level 1)
        assert_eq!(decision.indent_text, "        ");
        assert_eq!(decision.indent_level, 2);
        // Block expansion present
        let expansion = decision.block_expansion.unwrap();
        assert_eq!(expansion.closing_indent, "    "); // Same as reference
        assert_eq!(expansion.closing_text, "}");
    }

    #[test]
    fn block_expansion_at_zero_indent() {
        // Validates: Requirement 5.1
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext {
            reference_line: "fn main() {}".to_string(),
            caret_column: 11,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "fn main() {".to_string(),
            text_after_caret: "}".to_string(),
        };

        let result = try_block_expansion(&ctx, &patterns, &config);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.indent_text, "    "); // level 1
        assert_eq!(decision.indent_level, 1);
        let expansion = decision.block_expansion.unwrap();
        assert_eq!(expansion.closing_indent, ""); // level 0
    }

    #[test]
    fn no_expansion_when_patterns_undefined() {
        // Validates: Requirement 5.4 — no expansion when patterns missing
        let config = make_config();
        let patterns = IndentPatterns::empty();
        let ctx = IndentContext::simple("fn main() {}", 11);

        let result = try_block_expansion(&ctx, &patterns, &config);
        assert!(result.is_none());
    }

    #[test]
    fn no_expansion_when_caret_not_between_delimiters() {
        // Validates: Requirement 5.1
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext {
            reference_line: "    let x = 5;".to_string(),
            caret_column: 14,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "    let x = 5;".to_string(),
            text_after_caret: "".to_string(),
        };

        let result = try_block_expansion(&ctx, &patterns, &config);
        assert!(result.is_none());
    }

    #[test]
    fn no_expansion_when_only_block_start_matches() {
        // Validates: Requirement 5.1
        let config = make_config();
        let patterns = c_like_patterns();
        let ctx = IndentContext {
            reference_line: "    if (true) { x".to_string(),
            caret_column: 15,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "    if (true) {".to_string(),
            text_after_caret: " x".to_string(),
        };

        let result = try_block_expansion(&ctx, &patterns, &config);
        assert!(result.is_none());
    }

    #[test]
    fn block_expansion_with_tabs() {
        // Validates: Requirement 5.1 with tab indentation
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        let patterns = c_like_patterns();
        let ctx = IndentContext {
            reference_line: "\tfn main() {}".to_string(),
            caret_column: 12,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "\tfn main() {".to_string(),
            text_after_caret: "}".to_string(),
        };

        let result = try_block_expansion(&ctx, &patterns, &config);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.indent_text, "\t\t"); // level 2 tabs
        let expansion = decision.block_expansion.unwrap();
        assert_eq!(expansion.closing_indent, "\t"); // level 1 tab
    }
}
