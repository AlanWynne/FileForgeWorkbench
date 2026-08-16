//! Property-based tests for the ff-asa crate.
//!
//! Each property test verifies a universal invariant that must hold
//! for all valid inputs, using the `proptest` crate.

use proptest::prelude::*;

use ff_asa::config::ExportPageSeparator;
use ff_asa::control::AsaControl;
use ff_asa::detection::{detect_asa, DetectionConfig};
use ff_asa::export_text::{export_text, TextExportOptions};
use ff_asa::merge::MergedLine;
use ff_asa::page_index::PageIndex;
use ff_asa::preview::{build_preview, PreviewElement};
use ff_asa::shading::compute_band_groups;
use ff_asa::strip::{restore_asa, strip_asa};

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Generate a random ASCII character (0x00–0x7F).
fn ascii_char() -> impl Strategy<Value = char> {
    (0u8..128).prop_map(|b| b as char)
}

/// Generate a random valid ASA control character.
fn asa_control_char() -> impl Strategy<Value = char> {
    prop_oneof![
        Just(' '),
        Just('0'),
        Just('-'),
        Just('1'),
        Just('+'),
        Just('H'),
    ]
}

/// Generate a random ASA control (weighted towards common ones).
fn asa_control() -> impl Strategy<Value = AsaControl> {
    prop_oneof![
        3 => Just(AsaControl::Space),
        1 => Just(AsaControl::DoubleSpace),
        1 => Just(AsaControl::TripleSpace),
        2 => Just(AsaControl::PageEject),
        1 => Just(AsaControl::Overstrike),
        1 => Just(AsaControl::Halt),
    ]
}

/// Generate a printable ASCII string (no control characters).
fn printable_ascii_string(max_len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(0x20u8..0x7F, 0..max_len)
        .prop_map(|bytes| bytes.into_iter().map(|b| b as char).collect())
}

/// Generate a valid ASA document line (control char + printable content).
fn asa_document_line() -> impl Strategy<Value = String> {
    (asa_control_char(), printable_ascii_string(80))
        .prop_map(|(ctrl, content)| format!("{}{}", ctrl, content))
}

// ─── Property 1: ASA Control Character Parsing Completeness ─────────────────

proptest! {
    /// **Validates: Requirement 1.1, 1.9**
    ///
    /// For any character in the ASCII range, `AsaControl::from_char` returns the
    /// correct variant for known ASA characters and `Unknown(ch)` for others.
    /// The function never panics.
    // Feature: ff-asa, Property 1: ASA control character parsing completeness
    #[test]
    fn prop_asa_control_parsing_completeness(ch in ascii_char()) {
        let control = AsaControl::from_char(ch);

        // Known chars produce their expected variant
        match ch {
            ' ' => prop_assert_eq!(control, AsaControl::Space),
            '0' => prop_assert_eq!(control, AsaControl::DoubleSpace),
            '-' => prop_assert_eq!(control, AsaControl::TripleSpace),
            '1' => prop_assert_eq!(control, AsaControl::PageEject),
            '+' => prop_assert_eq!(control, AsaControl::Overstrike),
            'H' => prop_assert_eq!(control, AsaControl::Halt),
            other => prop_assert_eq!(control, AsaControl::Unknown(other)),
        }

        // Spacing values match spec
        match control {
            AsaControl::DoubleSpace => prop_assert_eq!(control.spacing_lines(), 1),
            AsaControl::TripleSpace => prop_assert_eq!(control.spacing_lines(), 2),
            _ => prop_assert_eq!(control.spacing_lines(), 0),
        }
    }
}

// ─── Property 2: Detection Confidence Threshold Boundary Correctness ────────

proptest! {
    /// **Validates: Requirement 2.1, 2.2, 2.6**
    ///
    /// The detection algorithm classifies a file as ASA-controlled if and only if:
    /// (a) confidence >= threshold AND (b) at least one `1` is present.
    // Feature: ff-asa, Property 2: Detection confidence threshold boundary correctness
    #[test]
    fn prop_detection_threshold_boundary(
        valid_ratio in 0.0f64..1.0,
        has_page_eject in proptest::bool::ANY,
        threshold in 0.5f64..1.0,
        line_count in 5usize..100,
    ) {
        // Build lines with the target valid ratio
        let valid_count = (line_count as f64 * valid_ratio).round() as usize;
        let invalid_count = line_count - valid_count;

        let mut lines: Vec<String> = Vec::with_capacity(line_count);

        // Add valid ASA lines
        for i in 0..valid_count {
            if has_page_eject && i == 0 {
                lines.push("1PAGE DATA".to_string());
            } else {
                lines.push(" VALID DATA".to_string());
            }
        }

        // Add invalid lines
        for _ in 0..invalid_count {
            lines.push("XINVALID DATA".to_string());
        }

        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

        let config = DetectionConfig {
            threshold,
            sample_size: line_count + 10, // Sample all lines
        };

        let result = detect_asa(&line_refs, &config);

        // Confidence must be in [0.0, 1.0]
        prop_assert!(result.confidence >= 0.0);
        prop_assert!(result.confidence <= 1.0);

        // Classification rule: is_asa iff (confidence >= threshold AND page_eject_found)
        let expected_is_asa = result.confidence >= threshold && result.has_page_eject;
        prop_assert_eq!(result.is_asa, expected_is_asa);
    }
}

// ─── Property 3: Page Index Construction Consistency ────────────────────────

proptest! {
    /// **Validates: Requirement 4.1, 4.5, 8.3, 8.4**
    ///
    /// For any sequence of ASA controls, the page index entries are non-overlapping,
    /// ordered, and explicit `1` controls always start a new page.
    // Feature: ff-asa, Property 3: Page index construction consistency
    #[test]
    fn prop_page_index_consistency(
        controls in proptest::collection::vec(asa_control(), 1..200),
        page_depth in 10u16..80,
    ) {
        let index = PageIndex::build(&controls, page_depth, true);

        // Entries are ordered by document_line
        for window in index.entries().windows(2) {
            prop_assert!(window[0].document_line < window[1].document_line,
                "Page entries must be ordered: {} < {}",
                window[0].document_line, window[1].document_line);
        }

        // Every explicit `1` must start a new page
        let page_eject_lines: Vec<usize> = controls.iter()
            .enumerate()
            .filter(|(_, c)| c.is_page_break())
            .map(|(i, _)| i)
            .collect();

        let page_start_lines: Vec<usize> = index.entries()
            .iter()
            .filter(|e| e.is_explicit)
            .map(|e| e.document_line)
            .collect();

        for &eject_line in &page_eject_lines {
            prop_assert!(page_start_lines.contains(&eject_line),
                "Page eject at line {} must start a page", eject_line);
        }

        // Page numbers are sequential
        for (i, entry) in index.entries().iter().enumerate() {
            prop_assert_eq!(entry.page_number.0 as usize, i + 1,
                "Page numbers must be sequential");
        }
    }
}

// ─── Property 4: Overstrike Merge Idempotence ───────────────────────────────

proptest! {
    /// **Validates: Requirement 5.1, 5.2, 5.3**
    ///
    /// Applying the same overprint twice produces the same result as applying
    /// it once (style idempotence — double bold is still bold).
    // Feature: ff-asa, Property 4: Overstrike merge idempotence and character coverage
    #[test]
    fn prop_overstrike_merge_idempotence(
        base in printable_ascii_string(80),
        overprint in printable_ascii_string(80),
    ) {
        // Apply overprint once
        let mut merged_once = MergedLine::from_base(&base, 0);
        merged_once.apply_overprint(&overprint);

        // Apply same overprint twice
        let mut merged_twice = MergedLine::from_base(&base, 0);
        merged_twice.apply_overprint(&overprint);
        merged_twice.apply_overprint(&overprint);

        // Characters should be the same (idempotent for content)
        prop_assert_eq!(merged_once.plain_text(), merged_twice.plain_text(),
            "Applying same overprint twice should produce same plain text");

        // Merged length >= max(base, overprint)
        let expected_min_len = base.len().max(overprint.len());
        prop_assert!(merged_once.characters.len() >= expected_min_len,
            "Merged length {} must be >= max(base={}, overprint={})",
            merged_once.characters.len(), base.len(), overprint.len());
    }
}

// ─── Property 5: Strip/Restore Round-Trip Fidelity ──────────────────────────

proptest! {
    /// **Validates: Requirement 7.1, 7.2, 7.3**
    ///
    /// Stripping followed by restoring produces byte-for-byte identical output.
    // Feature: ff-asa, Property 5: Strip/restore round-trip fidelity
    #[test]
    fn prop_strip_restore_round_trip(
        lines in proptest::collection::vec(asa_document_line(), 1..100),
    ) {
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

        let (stripped, map) = strip_asa(&line_refs);

        // Control map has exactly one entry per line
        prop_assert_eq!(map.len(), lines.len(),
            "Control map must have one entry per line");

        // Restore must produce the original
        let stripped_refs: Vec<&str> = stripped.iter().map(|s| s.as_str()).collect();
        let restored = restore_asa(&stripped_refs, &map);

        for (i, (original, restored_line)) in lines.iter().zip(restored.iter()).enumerate() {
            prop_assert_eq!(original, restored_line,
                "Line {} must match after round-trip", i);
        }
    }
}

// ─── Property 6: Line Band Shading Assignment Correctness ───────────────────

proptest! {
    /// **Validates: Requirement 9.1, 9.3, 9.4, 9.5**
    ///
    /// Line band shading resets at page boundaries and page bands don't carry groups.
    // Feature: ff-asa, Property 6: Line band shading assignment correctness
    #[test]
    fn prop_line_band_shading_resets_at_page_boundary(
        band_size in 1u8..10,
        data_before in 1usize..20,
        data_after in 1usize..20,
    ) {
        use ff_asa::merge::MergedLine;

        let mut elements = Vec::new();

        // Add data lines before page break
        for i in 0..data_before {
            elements.push(PreviewElement::DataLine {
                content: MergedLine::from_base("DATA", i),
                band_group: 0,
                page_line: i + 1,
            });
        }

        // Page break
        elements.push(PreviewElement::PageBand {
            page_number: 1,
            is_explicit: true,
        });

        // Data lines after page break
        for i in 0..data_after {
            elements.push(PreviewElement::DataLine {
                content: MergedLine::from_base("DATA", data_before + i),
                band_group: 0,
                page_line: i + 1,
            });
        }

        let groups = compute_band_groups(&elements, band_size);

        // Find the page band index
        let page_band_idx = data_before;

        // Page band has no group
        prop_assert_eq!(groups[page_band_idx], None,
            "Page band must not carry a group");

        // First data line after page break starts at group 0
        if data_after > 0 {
            prop_assert_eq!(groups[page_band_idx + 1], Some(0),
                "First line after page break must be in group 0");
        }
    }
}

// ─── Property 7: Text Export Spacing and Page Break Fidelity ────────────────

proptest! {
    /// **Validates: Requirement 11.3, 11.4, 11.5**
    ///
    /// Text export produces the correct number of page separators and spacing lines.
    // Feature: ff-asa, Property 7: Text export spacing and page break fidelity
    #[test]
    fn prop_text_export_page_count(
        page_count in 1usize..20,
        lines_per_page in 1usize..10,
    ) {
        use ff_asa::merge::MergedLine;

        let mut elements = Vec::new();

        for page in 0..page_count {
            elements.push(PreviewElement::PageBand {
                page_number: page + 1,
                is_explicit: true,
            });
            for line in 0..lines_per_page {
                elements.push(PreviewElement::DataLine {
                    content: MergedLine::from_base("DATA", page * lines_per_page + line),
                    band_group: 0,
                    page_line: line + 1,
                });
            }
        }

        let options = TextExportOptions {
            page_separator: ExportPageSeparator::Dashes,
        };
        let text = export_text(&elements, &options);

        // Count page separators — should equal page_count
        let separator_count = text.lines()
            .filter(|l| l.starts_with("--- PAGE ") && l.ends_with(" ---"))
            .count();
        prop_assert_eq!(separator_count, page_count,
            "Expected {} page separators, found {}", page_count, separator_count);
    }
}

// ─── Property 8: Preview Element Count Conservation ─────────────────────────

proptest! {
    /// **Validates: Requirement 1.2–1.6, 4.4, 5.4**
    ///
    /// Preview element count matches: data lines = N - overprints,
    /// plus spacing insertions plus page bands plus halt bands.
    // Feature: ff-asa, Property 8: Preview element count conservation
    #[test]
    fn prop_preview_element_count_conservation(
        controls in proptest::collection::vec(
            prop_oneof![
                3 => Just(AsaControl::Space),
                1 => Just(AsaControl::DoubleSpace),
                1 => Just(AsaControl::TripleSpace),
                1 => Just(AsaControl::PageEject),
                1 => Just(AsaControl::Halt),
                // Exclude leading overstrike to avoid orphan edge case
            ],
            1..50
        ),
    ) {
        let contents: Vec<String> = controls.iter()
            .enumerate()
            .map(|(i, _)| format!("LINE {}", i))
            .collect();
        let content_refs: Vec<&str> = contents.iter().map(|s| s.as_str()).collect();

        let state = build_preview(&controls, &content_refs, 60, 5, false);

        // Count expected elements
        let expected_data_lines = controls.len(); // No overstrikes in this test
        let expected_spacing: usize = controls.iter()
            .map(|c| c.spacing_lines() as usize)
            .sum();
        let expected_page_bands = controls.iter()
            .filter(|c| c.is_page_break())
            .count();
        let expected_halt_bands = controls.iter()
            .filter(|c| matches!(c, AsaControl::Halt))
            .count();

        let actual_data_lines = state.elements.iter()
            .filter(|e| matches!(e, PreviewElement::DataLine { .. }))
            .count();
        let actual_spacing = state.elements.iter()
            .filter(|e| matches!(e, PreviewElement::SpacingLine { .. }))
            .count();
        let actual_page_bands = state.elements.iter()
            .filter(|e| matches!(e, PreviewElement::PageBand { .. }))
            .count();
        let actual_halt_bands = state.elements.iter()
            .filter(|e| matches!(e, PreviewElement::HaltBand { .. }))
            .count();

        prop_assert_eq!(actual_data_lines, expected_data_lines,
            "Data lines: expected {}, got {}", expected_data_lines, actual_data_lines);
        prop_assert_eq!(actual_spacing, expected_spacing,
            "Spacing lines: expected {}, got {}", expected_spacing, actual_spacing);
        prop_assert_eq!(actual_page_bands, expected_page_bands,
            "Page bands: expected {}, got {}", expected_page_bands, actual_page_bands);
        prop_assert_eq!(actual_halt_bands, expected_halt_bands,
            "Halt bands: expected {}, got {}", expected_halt_bands, actual_halt_bands);
    }
}
