//! Property-based tests for the ff-whitespace-guides crate.
//!
//! Uses the `proptest` crate with a minimum of 256 cases per property.

use proptest::prelude::*;

use ff_whitespace_guides::modes::{
    EdgeMode, IndentGuideMode, TabDrawMode, WhitespaceVisibility, WrapIndentMode,
};
use ff_whitespace_guides::query::edge::{compute_edge_indicator, EdgeConfig};
use ff_whitespace_guides::query::indent_guides::{compute_look_both_guides, compute_real_guides};
use ff_whitespace_guides::query::whitespace::compute_whitespace_glyphs;
use ff_whitespace_guides::query::wrap_markers::compute_continuation_indent;
use ff_whitespace_guides::types::ColourRGBA;

/// Strategy to generate a line of printable ASCII mixed with spaces and tabs.
fn line_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(
        prop_oneof![
            3 => Just(b' '),
            1 => Just(b'\t'),
            6 => 0x21u8..0x7Fu8, // printable non-whitespace ASCII
        ],
        0..=200,
    )
}

/// Strategy to generate a line with leading whitespace followed by non-whitespace.
fn indented_line_strategy() -> impl Strategy<Value = Vec<u8>> {
    (
        prop::collection::vec(prop_oneof![Just(b' '), Just(b'\t')], 0..=50),
        prop::collection::vec(0x41u8..0x5Bu8, 1..=20), // some letters
    )
        .prop_map(|(indent, content)| {
            let mut line = indent;
            line.extend(content);
            line
        })
}

/// Strategy to generate a document (vec of lines) with random indentation.
fn document_strategy() -> impl Strategy<Value = Vec<Vec<u8>>> {
    prop::collection::vec(
        prop_oneof![
            3 => indented_line_strategy(),
            1 => Just(Vec::new()), // blank line
            1 => prop::collection::vec(Just(b' '), 0..=40), // all-whitespace line
        ],
        1..=20,
    )
}

// ─── Property 1: Whitespace Glyph Completeness ─────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 1.3**
    ///
    /// For any line and VisibleAlways mode, every space/tab character produces
    /// exactly one glyph.
    ///
    /// Feature: whitespace-and-guides, Property 1: whitespace glyph completeness
    #[test]
    fn whitespace_glyph_completeness(
        line in line_strategy(),
        tab_size in 1u32..=8u32,
    ) {
        let glyphs = compute_whitespace_glyphs(
            &line,
            tab_size,
            WhitespaceVisibility::VisibleAlways,
            TabDrawMode::LongArrow,
        );

        let ws_count = line.iter().filter(|&&b| b == b' ' || b == b'\t').count();
        prop_assert_eq!(
            glyphs.len(),
            ws_count,
            "Expected {} glyphs for {} whitespace chars in line of len {}",
            ws_count,
            ws_count,
            line.len()
        );
    }
}

// ─── Property 2: Indent Guide Column Alignment ─────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 3.3**
    ///
    /// All guide columns returned by `compute_real_guides` are exact multiples
    /// of tab_size.
    ///
    /// Feature: whitespace-and-guides, Property 2: indent guide column alignment
    #[test]
    fn indent_guide_columns_are_tab_stop_aligned(
        line in indented_line_strategy(),
        tab_size in 1u32..=8u32,
    ) {
        let guides = compute_real_guides(&line, tab_size);
        for col in &guides {
            prop_assert_eq!(
                col % tab_size,
                0,
                "Guide column {} is not a multiple of tab_size {}",
                col,
                tab_size
            );
        }
    }
}

// ─── Property 3: LookBoth Superset of Real Guides ──────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 3.4, 3.5**
    ///
    /// For any document and line index, guides in LookBoth mode are a superset
    /// of guides in Real mode for that line.
    ///
    /// Feature: whitespace-and-guides, Property 3: LookBoth superset of Real
    #[test]
    fn look_both_produces_superset_of_real_guides(
        doc in document_strategy(),
        tab_size in 1u32..=8u32,
    ) {
        prop_assume!(!doc.is_empty());

        for line_idx in 0..doc.len() {
            let line_refs: Vec<&[u8]> = doc.iter().map(|l| l.as_slice()).collect();

            let real_guides = compute_real_guides(&doc[line_idx], tab_size);
            let look_both_guides = compute_look_both_guides(&line_refs, line_idx, tab_size);

            for col in &real_guides {
                prop_assert!(
                    look_both_guides.contains(col),
                    "Real guide at column {} not found in LookBoth guides {:?} for line {}",
                    col,
                    look_both_guides,
                    line_idx
                );
            }
        }
    }
}

// ─── Property 4: Continuation Indent Clamping ───────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 7.6**
    ///
    /// Effective wrap indent never exceeds 3/4 of viewport width.
    ///
    /// Feature: whitespace-and-guides, Property 4: continuation indent clamping
    #[test]
    fn continuation_indent_never_exceeds_three_quarters_viewport(
        first_subline_indent in 0u32..=200u32,
        tab_size in 1u32..=8u32,
        mode_idx in 0usize..4usize,
        start_indent in 0u32..=50u32,
        viewport_width in 20u32..=300u32,
    ) {
        let modes = [
            WrapIndentMode::Fixed,
            WrapIndentMode::Same,
            WrapIndentMode::Indent,
            WrapIndentMode::DeepIndent,
        ];
        let mode = modes[mode_idx];

        let result = compute_continuation_indent(
            first_subline_indent,
            tab_size,
            mode,
            start_indent,
            viewport_width,
        );

        let max_allowed = viewport_width * 3 / 4;
        prop_assert!(
            result.indent_chars <= max_allowed,
            "Indent {} exceeds max {} (3/4 of viewport {})",
            result.indent_chars,
            max_allowed,
            viewport_width
        );
    }
}

// ─── Property 5: Toggle Command Cycling ─────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 8.1, 8.2**
    ///
    /// Applying toggle N times returns to the original state (N = variant count).
    ///
    /// Feature: whitespace-and-guides, Property 5: toggle command cycling
    #[test]
    fn toggle_whitespace_cycling_returns_to_start(start_idx in 0usize..4usize) {
        let variants = WhitespaceVisibility::variants();
        let start = variants[start_idx];

        let mut current = start;
        for _ in 0..4 {
            current = current.next();
        }
        prop_assert_eq!(current, start);
    }

    /// Feature: whitespace-and-guides, Property 5: toggle indent guide cycling
    #[test]
    fn toggle_indent_guides_cycling_returns_to_start(start_idx in 0usize..4usize) {
        let variants = IndentGuideMode::variants();
        let start = variants[start_idx];

        let mut current = start;
        for _ in 0..4 {
            current = current.next();
        }
        prop_assert_eq!(current, start);
    }
}

// ─── Property 6: Edge Indicator Mode Consistency ────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 5.1, 5.2**
    ///
    /// EdgeMode::None always yields no indicator; non-None always yields an indicator.
    ///
    /// Feature: whitespace-and-guides, Property 6: edge indicator mode consistency
    #[test]
    fn edge_indicator_mode_consistency(
        mode_idx in 0usize..4usize,
        column in 1u32..=300u32,
        r in 0u8..=255u8,
        g in 0u8..=255u8,
        b in 0u8..=255u8,
        a in 0u8..=255u8,
        num_edges in 0usize..=5usize,
    ) {
        let modes = [EdgeMode::None, EdgeMode::Line, EdgeMode::Background, EdgeMode::MultiLine];
        let mode = modes[mode_idx];
        let colour = ColourRGBA { r, g, b, a };

        let multi_edges: Vec<_> = (0..num_edges)
            .map(|i| ff_whitespace_guides::types::EdgeProperties {
                column: column + i as u32,
                colour,
            })
            .collect();

        let config = EdgeConfig {
            mode,
            column,
            colour,
            multi_edges,
        };

        let result = compute_edge_indicator(&config);

        match mode {
            EdgeMode::None => prop_assert!(result.is_none(), "None mode should yield None"),
            _ => prop_assert!(result.is_some(), "Non-None mode {:?} should yield Some", mode),
        }
    }
}
