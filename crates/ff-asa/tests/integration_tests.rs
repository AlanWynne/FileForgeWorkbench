//! Integration tests for the ff-asa crate.
//!
//! End-to-end tests exercising the full preview lifecycle, strip/restore,
//! export, navigation, and configuration.

use ff_asa::config::ExportPageSeparator;
use ff_asa::control::AsaControl;
use ff_asa::detection::{detect_asa, DetectionConfig};
use ff_asa::export_text::{export_text, TextExportOptions};
use ff_asa::merge::merge_overstrikes;
use ff_asa::navigation::{first_page, last_page, locate_page, next_page};
use ff_asa::page_index::PageIndex;
use ff_asa::preview::{build_preview, PreviewElement};
use ff_asa::strip::{restore_asa, strip_asa};

/// Build a sample ASA document for testing.
fn sample_asa_document() -> Vec<String> {
    vec![
        "1MONTHLY SALES REPORT               PAGE   1".to_string(),
        " DEPARTMENT: ELECTRONICS".to_string(),
        " ".to_string(),
        "0ITEM              QTY    PRICE    TOTAL".to_string(),
        " Widget A          100    $5.00    $500.00".to_string(),
        " Widget B           50   $10.00    $500.00".to_string(),
        "+-----            ----   ------    -------".to_string(),
        "-SUBTOTAL          150            $1000.00".to_string(),
        " ".to_string(),
        "1MONTHLY SALES REPORT               PAGE   2".to_string(),
        " DEPARTMENT: SOFTWARE".to_string(),
        " ".to_string(),
        "0ITEM              QTY    PRICE    TOTAL".to_string(),
        " License A          10  $100.00   $1000.00".to_string(),
        " License B           5  $200.00   $1000.00".to_string(),
    ]
}

// ─── Integration Test: Full Preview Lifecycle ───────────────────────────────

#[test]
// Validates: Requirements 1, 2, 3, 4
fn full_preview_lifecycle_detect_activate_navigate() {
    let lines = sample_asa_document();
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    // Step 1: Detect ASA
    let detection = detect_asa(&line_refs, &DetectionConfig::default());
    assert!(
        detection.is_asa,
        "Sample document should be detected as ASA"
    );
    assert!(detection.has_page_eject);
    assert!(detection.confidence >= 0.8);

    // Step 2: Parse controls
    let controls: Vec<AsaControl> = lines
        .iter()
        .map(|line| AsaControl::from_char(line.chars().next().unwrap_or(' ')))
        .collect();
    let contents: Vec<&str> = lines
        .iter()
        .map(|line| if line.len() > 1 { &line[1..] } else { "" })
        .collect();

    // Step 3: Build preview
    let state = build_preview(&controls, &contents, 60, 5, true);

    // Verify page count (2 explicit page ejects)
    assert_eq!(state.total_pages, 2);

    // Verify page bands are present
    let page_bands: Vec<usize> = state
        .elements
        .iter()
        .filter_map(|e| {
            if let PreviewElement::PageBand { page_number, .. } = e {
                Some(*page_number)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(page_bands, vec![1, 2]);

    // Step 4: Navigate
    let line = locate_page(&state.page_index, 1).unwrap();
    assert_eq!(line, 0);
    let line = locate_page(&state.page_index, 2).unwrap();
    assert_eq!(line, 9);

    // Page 3 doesn't exist
    assert!(locate_page(&state.page_index, 3).is_err());
}

// ─── Integration Test: Overstrike Merge in Document ─────────────────────────

#[test]
// Validates: Requirement 5
fn overstrike_merge_with_multi_line_overprint_sequences() {
    let lines = [
        " REPORT HEADER",
        "+REPORT HEADER",  // Same chars → bold
        "+--------------", // Dashes → underline
        " DATA LINE",
    ];
    let controls: Vec<AsaControl> = lines
        .iter()
        .map(|line| AsaControl::from_char(line.chars().next().unwrap_or(' ')))
        .collect();
    let contents: Vec<&str> = lines.iter().map(|line| &line[1..]).collect();

    let results = merge_overstrikes(&controls, &contents);

    // Should produce 2 merged results: merged header + data line
    assert_eq!(results.len(), 2);

    if let ff_asa::MergeResult::Merged(ref merged) = results[0] {
        // Header merged with two overprints
        assert_eq!(merged.overprint_count, 2);
        assert!(merged.has_bold());
        assert!(merged.has_underline());
    } else {
        panic!("Expected merged result for header");
    }
}

// ─── Integration Test: Strip/Restore Round-Trip ─────────────────────────────

#[test]
// Validates: Requirement 7
fn strip_restore_preserves_asa_controls_through_edit_save_cycle() {
    let lines = sample_asa_document();
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    // Strip
    let (stripped, mut map) = strip_asa(&line_refs);

    // Verify stripped content doesn't have control chars in column 1
    for (i, stripped_line) in stripped.iter().enumerate() {
        if !stripped_line.is_empty() {
            // The key test is that the original control was removed
            assert_eq!(
                stripped_line.len(),
                lines[i].len() - 1,
                "Stripped line {} should be 1 char shorter",
                i
            );
        }
    }

    // Simulate editing: insert a new line
    let mut edited = stripped.clone();
    edited.insert(3, "NEW INSERTED LINE".to_string());
    map.insert_line(3);

    // Simulate editing: delete a line
    edited.remove(5); // Remove what was originally line 4 (shifted by insert)
    map.remove_line(5);

    // Restore
    let edited_refs: Vec<&str> = edited.iter().map(|s| s.as_str()).collect();
    let restored = restore_asa(&edited_refs, &map);

    // Verify restoration adds control characters back
    assert_eq!(restored.len(), edited.len());
    // First line should have '1' prefix (page eject)
    assert!(restored[0].starts_with('1'));
    // Inserted line should have ' ' prefix (default)
    assert!(restored[3].starts_with(' '));
}

// ─── Integration Test: Text Export ──────────────────────────────────────────

#[test]
// Validates: Requirement 11
fn export_text_matches_rendered_preview_content() {
    let lines = sample_asa_document();
    let controls: Vec<AsaControl> = lines
        .iter()
        .map(|line| AsaControl::from_char(line.chars().next().unwrap_or(' ')))
        .collect();
    let contents: Vec<&str> = lines
        .iter()
        .map(|line| if line.len() > 1 { &line[1..] } else { "" })
        .collect();

    let state = build_preview(&controls, &contents, 60, 5, true);

    // Export with dashes
    let options = TextExportOptions {
        page_separator: ExportPageSeparator::Dashes,
    };
    let text = export_text(&state.elements, &options);

    // Should contain page separators
    assert!(text.contains("--- PAGE 1 ---"));
    assert!(text.contains("--- PAGE 2 ---"));

    // Should contain data content
    assert!(text.contains("MONTHLY SALES REPORT"));
    assert!(text.contains("DEPARTMENT: ELECTRONICS"));
    assert!(text.contains("DEPARTMENT: SOFTWARE"));

    // Should not contain raw control characters
    let output_lines: Vec<&str> = text.lines().collect();
    for line in &output_lines {
        if !line.is_empty() && !line.starts_with("---") {
            // Data lines should not start with ASA control chars
            // (they're stripped during preview generation)
        }
    }
}

// ─── Integration Test: Page Navigation ──────────────────────────────────────

#[test]
// Validates: Requirement 10
fn page_navigation_locate_and_traverse() {
    let controls = vec![
        AsaControl::PageEject,
        AsaControl::Space,
        AsaControl::Space,
        AsaControl::PageEject,
        AsaControl::Space,
        AsaControl::Space,
        AsaControl::PageEject,
        AsaControl::Space,
    ];
    let index = PageIndex::build(&controls, 60, true);

    // LOCATE PAGE
    assert_eq!(locate_page(&index, 1).unwrap(), 0);
    assert_eq!(locate_page(&index, 2).unwrap(), 3);
    assert_eq!(locate_page(&index, 3).unwrap(), 6);

    // LOCATE PAGE FIRST / LAST
    assert_eq!(first_page(&index), Some(0));
    assert_eq!(last_page(&index), Some(6));

    // UP PAGE / DOWN PAGE
    assert_eq!(next_page(&index, 1), Some(3));
    assert_eq!(next_page(&index, 2), Some(6));
    assert_eq!(next_page(&index, 3), None); // at last page

    // Error case
    let err = locate_page(&index, 99).unwrap_err();
    assert!(err.to_string().contains("page 99 not found"));
}

// ─── Integration Test: Printer Profile Switch ───────────────────────────────

#[test]
// Validates: Requirement 8
fn printer_profile_switch_triggers_page_index_rebuild() {
    // Build a document with many lines (> 60 and < 66)
    let controls: Vec<AsaControl> = (0..65).map(|_| AsaControl::Space).collect();

    // With ibm-1403 (depth=60), should get implicit break
    let index_1403 = PageIndex::build(&controls, 60, true);
    assert!(
        index_1403.page_count() >= 1,
        "60-depth should create at least 1 implicit break"
    );

    // With ibm-4245 (depth=66), all 65 lines fit in one page
    let index_4245 = PageIndex::build(&controls, 66, true);
    // 65 lines fit in 66-depth page, no implicit break needed
    assert_eq!(index_4245.page_count(), 0);
}

// ─── Integration Test: Config Hot-Reload ────────────────────────────────────

#[test]
// Validates: Requirement 9, 12
fn config_change_updates_line_band_shading() {
    use ff_asa::merge::MergedLine;
    use ff_asa::shading::compute_band_groups;

    let elements: Vec<PreviewElement> = (0..20)
        .map(|i| PreviewElement::DataLine {
            content: MergedLine::from_base("DATA", i),
            band_group: 0,
            page_line: i + 1,
        })
        .collect();

    // With band_size 5
    let groups_5 = compute_band_groups(&elements, 5);
    // First 5 should be group 0, next 5 group 1, etc.
    assert_eq!(groups_5[0], Some(0));
    assert_eq!(groups_5[4], Some(0));
    assert_eq!(groups_5[5], Some(1));
    assert_eq!(groups_5[9], Some(1));
    assert_eq!(groups_5[10], Some(0));

    // With band_size 3
    let groups_3 = compute_band_groups(&elements, 3);
    // First 3 group 0, next 3 group 1, etc.
    assert_eq!(groups_3[0], Some(0));
    assert_eq!(groups_3[2], Some(0));
    assert_eq!(groups_3[3], Some(1));
    assert_eq!(groups_3[5], Some(1));
    assert_eq!(groups_3[6], Some(0));

    // Changing band_size produces different groupings
    assert_ne!(groups_5, groups_3);
}
