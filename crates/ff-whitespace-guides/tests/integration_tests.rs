//! Integration tests for the ff-whitespace-guides crate.
//!
//! These tests verify end-to-end behaviour across multiple subsystem components.

use ff_whitespace_guides::commands::{toggle_edge_column, toggle_indent_guides, toggle_whitespace};
use ff_whitespace_guides::modes::{
    EdgeMode, IndentGuideMode, TabDrawMode, WhitespaceVisibility, WrapIndentMode, WrapVisualFlag,
    WrapVisualLocation,
};
use ff_whitespace_guides::query::edge::{compute_edge_indicator, EdgeConfig};
use ff_whitespace_guides::query::indent_guides::{
    compute_look_both_guides, compute_look_forward_guides, compute_real_guides,
};
use ff_whitespace_guides::query::whitespace::compute_whitespace_glyphs;
use ff_whitespace_guides::query::wrap_markers::{
    compute_continuation_indent, compute_wrap_markers,
};
use ff_whitespace_guides::settings::WhitespaceSettings;
use ff_whitespace_guides::types::{ColourRGBA, EdgeProperties, WhitespaceGlyph};

/// Integration test: full lifecycle — construct settings, toggle whitespace,
/// verify glyph output changes.
///
/// Validates: Requirements 1, 8
#[test]
fn full_lifecycle_toggle_whitespace_changes_glyph_output() {
    // Start with invisible mode (default)
    let settings = WhitespaceSettings::default();
    assert_eq!(settings.visibility, WhitespaceVisibility::Invisible);

    let line = b"  hello  world  ";

    // No glyphs in invisible mode
    let glyphs = compute_whitespace_glyphs(line, 4, settings.visibility, settings.tab_draw_mode);
    assert!(glyphs.is_empty());

    // Toggle to VisibleAlways
    let new_vis = toggle_whitespace(settings.visibility);
    assert_eq!(new_vis, WhitespaceVisibility::VisibleAlways);

    // Now glyphs should appear for all whitespace
    let glyphs = compute_whitespace_glyphs(line, 4, new_vis, settings.tab_draw_mode);
    assert_eq!(glyphs.len(), 6); // 2 leading + 2 middle + 2 trailing spaces

    // Toggle to VisibleAfterIndent
    let next_vis = toggle_whitespace(new_vis);
    assert_eq!(next_vis, WhitespaceVisibility::VisibleAfterIndent);

    // Only non-leading spaces visible
    let glyphs = compute_whitespace_glyphs(line, 4, next_vis, settings.tab_draw_mode);
    assert_eq!(glyphs.len(), 4); // 2 middle + 2 trailing
}

/// Integration test: indent guide spanning — multi-line document with blank lines,
/// verify LookBoth guides extend through blanks.
///
/// Validates: Requirement 3
#[test]
fn indent_guide_spanning_through_blank_lines() {
    let doc: Vec<&[u8]> = vec![
        b"        if condition {", // indent 8
        b"            statement;", // indent 12
        b"",                       // blank
        b"",                       // blank
        b"            more_code;", // indent 12
        b"        }",              // indent 8
    ];

    // Real mode: blank lines get no guides
    let real_at_blank = compute_real_guides(doc[2], 4);
    assert!(real_at_blank.is_empty());

    // LookBoth mode: blank lines inherit guides from surrounding context
    let look_both_at_blank = compute_look_both_guides(&doc, 2, 4);
    // Surrounding non-blank lines have indent 12, so guides at 4, 8, 12
    assert_eq!(look_both_at_blank, vec![4, 8, 12]);

    // LookForward mode: blank lines inherit from next non-blank
    let look_forward_at_blank = compute_look_forward_guides(&doc, 2, 4);
    assert_eq!(look_forward_at_blank, vec![4, 8, 12]);
}

/// Integration test: edge column multi-line — configure multiple edges,
/// verify all are returned.
///
/// Validates: Requirement 5
#[test]
fn edge_column_multi_line_returns_all_configured_edges() {
    let edges = vec![
        EdgeProperties {
            column: 80,
            colour: ColourRGBA {
                r: 255,
                g: 0,
                b: 0,
                a: 128,
            },
        },
        EdgeProperties {
            column: 100,
            colour: ColourRGBA {
                r: 0,
                g: 255,
                b: 0,
                a: 128,
            },
        },
        EdgeProperties {
            column: 120,
            colour: ColourRGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 128,
            },
        },
    ];

    let config = EdgeConfig {
        mode: EdgeMode::MultiLine,
        column: 80,
        colour: ColourRGBA::default(),
        multi_edges: edges.clone(),
    };

    let result = compute_edge_indicator(&config).unwrap();
    match result {
        ff_whitespace_guides::types::EdgeInfo::MultiLine { edges: returned } => {
            assert_eq!(returned.len(), 3);
            assert_eq!(returned, edges);
        }
        _ => panic!("Expected MultiLine variant"),
    }

    // Clear multi-edges by creating config with empty list
    let cleared_config = EdgeConfig {
        mode: EdgeMode::MultiLine,
        multi_edges: Vec::new(),
        ..config
    };
    let result = compute_edge_indicator(&cleared_config).unwrap();
    match result {
        ff_whitespace_guides::types::EdgeInfo::MultiLine { edges: returned } => {
            assert!(returned.is_empty());
        }
        _ => panic!("Expected MultiLine variant"),
    }
}

/// Integration test: wrap marker end-to-end — enable wrap, set flags,
/// compute markers for wrapped line.
///
/// Validates: Requirements 6, 7
#[test]
fn wrap_marker_end_to_end() {
    // Simulate a line wrapped into 4 sub-lines
    let sub_line_count = 4;
    let flags = WrapVisualFlag::END.union(WrapVisualFlag::START);
    let location = WrapVisualLocation::EndByText;

    let markers = compute_wrap_markers(sub_line_count, flags, location).unwrap();

    // End markers on sub-lines 0, 1, 2 (not last)
    assert_eq!(markers.end_markers, vec![0, 1, 2]);
    // Start markers on sub-lines 1, 2, 3 (continuations)
    assert_eq!(markers.start_markers, vec![1, 2, 3]);
    assert!(!markers.margin_marker);
    assert_eq!(markers.location, WrapVisualLocation::EndByText);

    // Verify continuation indent computation
    let indent_info = compute_continuation_indent(
        8, // first sub-line indent
        4, // tab_size
        WrapIndentMode::Indent,
        0,  // start_indent (not used for Indent mode)
        80, // viewport_width
    );
    assert_eq!(indent_info.indent_chars, 12); // 8 + 4
    assert!(!indent_info.clamped);
}

/// Integration test: wrap markers when wrap is not active produce no results.
///
/// Validates: Requirement 6 AC 6.9
#[test]
fn wrap_markers_inactive_when_single_subline() {
    // Even with all flags set, a single sub-line means no wrapping occurred
    let flags = WrapVisualFlag::END
        .union(WrapVisualFlag::START)
        .union(WrapVisualFlag::MARGIN);
    let result = compute_wrap_markers(1, flags, WrapVisualLocation::Default);
    assert!(result.is_none());
}

/// Integration test: headless testability — all queries work without
/// windowing system.
///
/// Validates: Requirement 9 AC 9.5
#[test]
fn headless_testability_all_queries_work_without_display() {
    // WhitespaceSettings is fully constructable without any windowing system
    let settings = WhitespaceSettings::default();
    assert!(!settings.is_whitespace_visible());

    // Whitespace query works without display
    let glyphs = compute_whitespace_glyphs(
        b"\thello world ",
        4,
        WhitespaceVisibility::VisibleAlways,
        TabDrawMode::Strikeout,
    );
    assert!(!glyphs.is_empty());
    assert_eq!(
        glyphs[0].glyph,
        WhitespaceGlyph::TabStrikeout { width_chars: 4 }
    );

    // Indent guide query works without display
    let guides = compute_real_guides(b"        code", 4);
    assert_eq!(guides, vec![4, 8]);

    // Edge query works without display
    let config = EdgeConfig {
        mode: EdgeMode::Line,
        column: 80,
        colour: ColourRGBA {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        },
        multi_edges: Vec::new(),
    };
    let edge = compute_edge_indicator(&config);
    assert!(edge.is_some());

    // Wrap marker query works without display
    let markers = compute_wrap_markers(3, WrapVisualFlag::END, WrapVisualLocation::Default);
    assert!(markers.is_some());

    // Toggle commands work without display
    let v = toggle_whitespace(WhitespaceVisibility::Invisible);
    assert_eq!(v, WhitespaceVisibility::VisibleAlways);
    let ig = toggle_indent_guides(IndentGuideMode::None);
    assert_eq!(ig, IndentGuideMode::Real);
    let em = toggle_edge_column(EdgeMode::None, None);
    assert_eq!(em, EdgeMode::Line);
}

/// Integration test: settings construction from defaults matches spec.
///
/// Validates: Requirement 9 AC 9.2
#[test]
fn settings_construction_matches_defaults() {
    let settings = WhitespaceSettings::default();

    // Verify all defaults match the specification
    assert_eq!(settings.visibility, WhitespaceVisibility::Invisible);
    assert_eq!(settings.tab_draw_mode, TabDrawMode::LongArrow);
    assert_eq!(settings.whitespace_size, 1);
    assert_eq!(settings.indent_guide_mode, IndentGuideMode::None);
    assert_eq!(settings.active_guide_column, None);
    assert_eq!(settings.edge_mode, EdgeMode::None);
    assert_eq!(settings.edge_column, 80);
    assert!(settings.edge_columns.is_empty());
    assert_eq!(settings.wrap_visual_flags, WrapVisualFlag::NONE);
    assert_eq!(settings.wrap_visual_location, WrapVisualLocation::Default);
    assert_eq!(settings.wrap_indent_mode, WrapIndentMode::Fixed);
    assert_eq!(settings.wrap_start_indent, 0);
    assert!(!settings.wrap_active);
    assert_eq!(settings.tab_size, 4);
    assert_eq!(settings.indent_size, 4);
}
