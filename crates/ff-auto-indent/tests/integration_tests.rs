//! Integration tests for ff-auto-indent.
//!
//! End-to-end validation of the auto-indent engine across multiple scenarios.

use ff_auto_indent::config::IndentStyle;
use ff_auto_indent::patterns::c_like_patterns;
use ff_auto_indent::{
    indent_lines, unindent_lines, AutoIndentMode, AutoIndentService, CommentConfig, IndentConfig,
    IndentContext, IndentPatterns, IndentTableRaw,
};

fn c_service() -> AutoIndentService {
    AutoIndentService::new(
        IndentConfig::new(4, 4, IndentStyle::Spaces),
        AutoIndentMode::Smart,
    )
}

// ─── Integration Test 14.1 ──────────────────────────────────────────────────

#[test]
fn full_newline_indent_cycle_c_like() {
    // Validates: Requirement 3.1 — full cycle: configure, insert newline after {, verify indent
    let service = c_service();
    let patterns = c_like_patterns();
    let comment = CommentConfig::empty();

    let ctx = IndentContext::simple("    if (true) {", 15);
    let result = service.compute_newline_indent(&ctx, &patterns, &comment);

    assert_eq!(result.indent_text, "        "); // Level 2
    assert_eq!(result.indent_level, 2);
}

// ─── Integration Test 14.2 ──────────────────────────────────────────────────

#[test]
fn decrease_on_closing_brace() {
    // Validates: Requirement 4.1 — type } on indented blank line, verify decrease
    let service = c_service();
    let patterns = c_like_patterns();

    // User typed "}" on a line with 8 spaces of indent
    let result = service.compute_char_indent("        }", 9, &patterns);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "    "); // Decreased from level 2 to level 1
}

// ─── Integration Test 14.3 ──────────────────────────────────────────────────

#[test]
fn enter_between_braces_expansion() {
    // Validates: Requirement 5.1 — press Enter between {}, verify 3-line expansion
    let service = c_service();
    let patterns = c_like_patterns();
    let comment = CommentConfig::empty();

    let ctx = IndentContext {
        reference_line: "fn main() {}".to_string(),
        caret_column: 11,
        in_comment: false,
        in_block_comment: false,
        is_empty_comment_continuation: false,
        text_before_caret: "fn main() {".to_string(),
        text_after_caret: "}".to_string(),
    };

    let result = service.compute_newline_indent(&ctx, &patterns, &comment);
    assert!(result.block_expansion.is_some());
    assert_eq!(result.indent_text, "    "); // Middle line at level 1
    let expansion = result.block_expansion.unwrap();
    assert_eq!(expansion.closing_indent, ""); // Closing at level 0
    assert_eq!(expansion.closing_text, "}");
}

// ─── Integration Test 14.4 ──────────────────────────────────────────────────

#[test]
fn block_comment_continuation() {
    // Validates: Requirement 6.1 — press Enter inside /* */, verify * marker
    let service = c_service();
    let patterns = c_like_patterns();
    let comment = CommentConfig::c_style();

    let ctx = IndentContext {
        reference_line: "     * Documentation line".to_string(),
        caret_column: 25,
        in_comment: true,
        in_block_comment: true,
        is_empty_comment_continuation: false,
        text_before_caret: "     * Documentation line".to_string(),
        text_after_caret: String::new(),
    };

    let result = service.compute_newline_indent(&ctx, &patterns, &comment);
    assert!(result.comment_continuation.is_some());
    let cont = result.comment_continuation.unwrap();
    assert_eq!(cont.marker, " * ");
    assert_eq!(cont.kind, ff_auto_indent::CommentKind::Block);
}

// ─── Integration Test 14.5 ──────────────────────────────────────────────────

#[test]
fn line_comment_continuation() {
    // Validates: Requirement 6.2 — press Enter after // comment, verify // prefix
    let service = c_service();
    let patterns = c_like_patterns();
    let comment = CommentConfig::c_style();

    let ctx = IndentContext {
        reference_line: "    // some comment".to_string(),
        caret_column: 19,
        in_comment: false,
        in_block_comment: false,
        is_empty_comment_continuation: false,
        text_before_caret: "    // some comment".to_string(),
        text_after_caret: String::new(),
    };

    let result = service.compute_newline_indent(&ctx, &patterns, &comment);
    assert!(result.comment_continuation.is_some());
    let cont = result.comment_continuation.unwrap();
    assert_eq!(cont.marker, "// ");
    assert_eq!(cont.kind, ff_auto_indent::CommentKind::Line);
}

// ─── Integration Test 14.6 ──────────────────────────────────────────────────

#[test]
fn double_enter_comment_break_out() {
    // Validates: Requirement 6.6 — double-Enter on empty continuation removes marker
    let service = c_service();
    let patterns = c_like_patterns();
    let comment = CommentConfig::c_style();

    let ctx = IndentContext {
        reference_line: "     * ".to_string(),
        caret_column: 7,
        in_comment: true,
        in_block_comment: true,
        is_empty_comment_continuation: true,
        text_before_caret: "     * ".to_string(),
        text_after_caret: String::new(),
    };

    let result = service.compute_newline_indent(&ctx, &patterns, &comment);
    // When break-out is signaled, comment continuation should NOT be added
    // The result falls through to smart indent (no continuation marker)
    assert!(result.comment_continuation.is_none());
}

// ─── Integration Test 14.7 ──────────────────────────────────────────────────

#[test]
fn indent_unindent_multi_line_roundtrip() {
    // Validates: Requirements 7.1, 8.1 — 5 lines indent then unindent
    let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
    let lines: Vec<u64> = (0..5).collect();
    let contents = vec![
        "    line1",
        "        line2",
        "    line3",
        "            line4",
        "line5",
    ];

    // Indent
    let indented = indent_lines(&lines, &contents, &config);
    assert_eq!(indented.len(), 5);

    // Create indented content
    let indented_contents: Vec<String> = contents
        .iter()
        .zip(indented.iter())
        .map(|(orig, edit)| {
            let after_ws = orig.trim_start();
            format!("{}{}", edit.new_indent, after_ws)
        })
        .collect();
    let indented_refs: Vec<&str> = indented_contents.iter().map(|s| s.as_str()).collect();

    // Unindent
    let restored = unindent_lines(&lines, &indented_refs, &config);

    // Verify roundtrip
    for (i, (original, edit)) in contents.iter().zip(restored.iter()).enumerate() {
        let original_ws_cols = config.column_width_of(
            &original
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>(),
        );
        let restored_cols = config.column_width_of(&edit.new_indent);
        assert_eq!(
            restored_cols, original_ws_cols,
            "Line {} roundtrip failed: expected {} cols, got {}",
            i, original_ws_cols, restored_cols
        );
    }
}

// ─── Integration Test 14.8 ──────────────────────────────────────────────────

#[test]
fn language_change_updates_patterns() {
    // Validates: Requirement 9.5 — switch language, verify new rules apply
    let service = c_service();

    // Load C-like patterns
    service.set_language_patterns("c", c_like_patterns());

    // Load Python-like patterns (different increase pattern)
    let python_raw = IndentTableRaw {
        increase_pattern: Some(r":\s*$".to_string()), // Python: colon at end
        decrease_pattern: None,
        statement_pattern: None,
        statement_end_pattern: None,
        block_start: None,
        block_end: None,
    };
    let python_patterns = IndentPatterns::compile(&python_raw);
    service.set_language_patterns("python", python_patterns);

    // Test with C patterns
    let c_patterns = service.get_patterns("c").unwrap();
    let ctx = IndentContext::simple("    def foo():", 14);
    let comment = CommentConfig::empty();
    let result = service.compute_newline_indent(&ctx, &c_patterns, &comment);
    // C patterns don't match Python colon, so maintain
    assert_eq!(result.indent_text, "    ");

    // Test with Python patterns
    let py_patterns = service.get_patterns("python").unwrap();
    let result = service.compute_newline_indent(&ctx, &py_patterns, &comment);
    // Python pattern matches ":", so increase
    assert_eq!(result.indent_text, "        ");
}

// ─── Integration Test 14.9 ──────────────────────────────────────────────────

#[test]
fn hot_reload_indent_size_change() {
    // Validates: Requirement 1.4 — change indent_size, verify subsequent indents use new size
    let service = c_service();
    let patterns = c_like_patterns();
    let comment = CommentConfig::empty();

    // Initial: indent_size=4
    let ctx = IndentContext::simple("fn main() {", 12);
    let result = service.compute_newline_indent(&ctx, &patterns, &comment);
    assert_eq!(result.indent_text, "    "); // 4 spaces

    // Hot-reload: change to indent_size=2
    service.update_config(IndentConfig::new(2, 4, IndentStyle::Spaces));
    let result = service.compute_newline_indent(&ctx, &patterns, &comment);
    assert_eq!(result.indent_text, "  "); // 2 spaces
}

// ─── Integration Test 14.10 ─────────────────────────────────────────────────

#[test]
fn multi_caret_independent_indent() {
    // Validates: Requirement 10.5 — two carets get independent correct indent
    let service = c_service();
    let patterns = c_like_patterns();
    let comment = CommentConfig::empty();

    // Caret 1: after a brace → increase
    let ctx1 = IndentContext::simple("fn main() {", 12);
    let result1 = service.compute_newline_indent(&ctx1, &patterns, &comment);

    // Caret 2: on a normal line → maintain
    let ctx2 = IndentContext::simple("    let x = 5;", 14);
    let result2 = service.compute_newline_indent(&ctx2, &patterns, &comment);

    assert_eq!(result1.indent_text, "    "); // Increased to level 1
    assert_eq!(result2.indent_text, "    "); // Maintained at level 1
                                             // They're independent computations
    assert_eq!(result1.indent_level, 1);
    assert_eq!(result2.indent_level, 1);
}

// ─── Integration Test 14.11 ─────────────────────────────────────────────────

#[test]
fn none_mode_produces_column_zero() {
    // Validates: Requirement 10.3 — None mode, Enter produces column 0
    let service = AutoIndentService::new(
        IndentConfig::new(4, 4, IndentStyle::Spaces),
        AutoIndentMode::None,
    );
    let patterns = c_like_patterns();
    let comment = CommentConfig::c_style();

    // Even with patterns that would normally trigger increase
    let ctx = IndentContext::simple("    if (true) {", 15);
    let result = service.compute_newline_indent(&ctx, &patterns, &comment);
    assert_eq!(result.indent_text, "");
    assert_eq!(result.indent_level, 0);
    assert!(result.block_expansion.is_none());
    assert!(result.comment_continuation.is_none());
}
