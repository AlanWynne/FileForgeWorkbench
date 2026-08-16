//! Integration tests for ff-hex.
//!
//! Validates end-to-end workflows across multiple components.

use ff_hex::{
    ArrowDirection, ByteReader, BytesPerRow, HexConfig, HexDigitCase, HexDumpExporter,
    HexGotoHandler, HexInput, HexMode, HexModeController, HexPane, HexSearchBridge, NibblePosition,
    VecByteReader,
};

/// Helper to create a controller with 256-byte sequential document.
fn setup_256() -> (HexModeController, VecByteReader) {
    let data: Vec<u8> = (0..=255u8).collect();
    let doc = VecByteReader::new(data);
    let ctrl = HexModeController::new(HexConfig::default());
    (ctrl, doc)
}

// Validates: Requirement 1 (full lifecycle)
#[test]
fn full_hex_mode_lifecycle() {
    let (mut ctrl, doc) = setup_256();
    let doc_len = doc.byte_length();

    // Start inactive
    assert!(!ctrl.is_active());

    // Activate
    ctrl.activate(0, doc_len).unwrap();
    assert!(ctrl.is_active());
    assert_eq!(ctrl.mode(), HexMode::On);
    assert_eq!(ctrl.cursor().byte_offset(), 0);

    // Navigate
    ctrl.handle_input(HexInput::Arrow(ArrowDirection::Down), &doc)
        .unwrap();
    assert_eq!(ctrl.cursor().byte_offset(), 16);

    ctrl.handle_input(HexInput::Arrow(ArrowDirection::Right), &doc)
        .unwrap();
    // In hex pane, right moves nibble: High → Low on same byte
    assert_eq!(ctrl.cursor().byte_offset(), 16);
    assert_eq!(ctrl.cursor().nibble(), NibblePosition::Low);

    // Edit a byte
    let action = ctrl
        .handle_input(HexInput::HexDigit('F'), &doc)
        .unwrap()
        .unwrap();
    assert_eq!(action.byte_offset, 16);
    assert_eq!(action.new_value, 0x1F); // low nibble = F, high nibble preserved = 1
    assert!(ctrl.modified_tracker().is_modified(16));

    // Deactivate
    let offset = ctrl.deactivate().unwrap();
    assert!(!ctrl.is_active());
    // After the edit cursor advanced: was at 16 low nibble, after edit moved to 17
    assert_eq!(offset, 17);
}

// Validates: Requirement 5 (hex search workflow)
#[test]
fn hex_search_with_auto_activation() {
    let (mut ctrl, _doc) = setup_256();

    // Search for pattern in hex mode
    let pattern = HexSearchBridge::validate_hex_pattern("0D0E").unwrap();
    assert_eq!(pattern, vec![0x0D, 0x0E]);

    // Simulate finding a match while hex mode is inactive
    let needs_activate = ctrl.search_bridge_mut().on_hex_match_found(13, 15, false);
    assert!(needs_activate); // Should trigger hex mode activation

    // After activating, highlights should be present
    let highlights = ctrl.search_bridge().active_highlights();
    assert_eq!(highlights.len(), 1);
    assert_eq!(highlights[0].start, 13);
    assert_eq!(highlights[0].end, 15);
}

// Validates: Requirement 12 (goto offset)
#[test]
fn goto_offset_navigation() {
    let (mut ctrl, doc) = setup_256();
    let doc_len = doc.byte_length();

    ctrl.activate(0, doc_len).unwrap();

    // Parse and navigate to hex offset
    let parsed = HexGotoHandler::parse_offset("X'80'").unwrap();
    assert_eq!(parsed.value, 0x80);

    HexGotoHandler::validate_bounds(parsed.value, doc_len).unwrap();
    ctrl.cursor_mut().goto_offset(parsed.value, doc_len);
    assert_eq!(ctrl.cursor().byte_offset(), 0x80);

    // Parse decimal offset
    let parsed = HexGotoHandler::parse_offset("100").unwrap();
    assert_eq!(parsed.value, 100);

    // Out of range
    let result = HexGotoHandler::validate_bounds(256, doc_len);
    assert!(result.is_err());
}

// Validates: Requirement 11 (hex dump export)
#[test]
fn hex_dump_export_full_document() {
    let data: Vec<u8> = (0..48).collect();
    let layout = ff_hex::HexLayout::new(48, BytesPerRow::Sixteen);
    let dump = HexDumpExporter::export(&data, None, &layout);

    let lines: Vec<&str> = dump.lines().collect();
    assert_eq!(lines.len(), 3); // 48 bytes / 16 = 3 rows

    // First line starts with offset 00000000
    assert!(lines[0].starts_with("00000000"));
    // Second line starts with offset 00000010
    assert!(lines[1].starts_with("00000010"));
    // Third line starts with offset 00000020
    assert!(lines[2].starts_with("00000020"));

    // Round trip
    let parsed = HexDumpExporter::parse_hex_dump(&dump, &layout);
    assert_eq!(parsed, data);
}

// Validates: Requirement 15 (session state save/restore)
#[test]
fn session_state_save_and_restore() {
    let (mut ctrl, doc) = setup_256();
    let doc_len = doc.byte_length();

    // Set up a specific state
    ctrl.activate(100, doc_len).unwrap();
    ctrl.cursor_mut().switch_pane(); // Switch to ASCII

    // Capture session
    let session = ctrl.capture_session();
    assert_eq!(session.mode, HexMode::On);
    assert_eq!(session.cursor_offset, 100);
    assert_eq!(session.active_pane, HexPane::Ascii);

    // Serialise and deserialise
    let json = serde_json::to_string(&session).unwrap();
    let restored: ff_hex::HexSessionState = serde_json::from_str(&json).unwrap();

    // Restore into a fresh controller
    let mut new_ctrl = HexModeController::new(HexConfig::default());
    new_ctrl.restore_session(&restored, doc_len).unwrap();
    assert!(new_ctrl.is_active());
    assert_eq!(new_ctrl.cursor().byte_offset(), 100);
    assert_eq!(new_ctrl.cursor().active_pane(), HexPane::Ascii);
}

// Validates: Requirement 16 (command compatibility)
#[test]
fn commands_operate_while_hex_active() {
    let (mut ctrl, doc) = setup_256();
    let doc_len = doc.byte_length();

    ctrl.activate(0, doc_len).unwrap();

    // Navigation commands work in hex mode
    ctrl.handle_input(HexInput::PageDown, &doc).unwrap();
    // Viewport should have scrolled
    assert!(
        ctrl.viewport().top_row() > 0
            || ctrl.viewport().visible_rows() >= ctrl.viewport().total_rows()
    );

    // Switch pane works
    ctrl.handle_input(HexInput::SwitchPane, &doc).unwrap();
    assert_eq!(ctrl.cursor().active_pane(), HexPane::Ascii);
}

// Validates: Requirement 3 (bytes per row change)
#[test]
fn bytes_per_row_change_preserves_cursor() {
    let (mut ctrl, doc) = setup_256();
    let doc_len = doc.byte_length();

    ctrl.activate(50, doc_len).unwrap();
    assert_eq!(ctrl.cursor().byte_offset(), 50);

    ctrl.set_bytes_per_row(BytesPerRow::ThirtyTwo, doc_len);
    // Cursor byte offset should be preserved
    assert_eq!(ctrl.cursor().byte_offset(), 50);
    assert_eq!(ctrl.layout().bytes_per_row(), BytesPerRow::ThirtyTwo);
}

// Validates: Requirement 13 (digit case change)
#[test]
fn digit_case_change_updates_display() {
    let (mut ctrl, doc) = setup_256();
    let doc_len = doc.byte_length();

    ctrl.activate(0, doc_len).unwrap();

    // Default is uppercase
    let vm = ctrl.build_view_model(&doc);
    let first_row_hex = &vm.visible_rows[0].hex_text;
    assert!(first_row_hex.contains("0A") || first_row_hex.contains("0B"));

    // Switch to lowercase
    ctrl.set_digit_case(HexDigitCase::Lowercase);
    let vm = ctrl.build_view_model(&doc);
    let first_row_hex = &vm.visible_rows[0].hex_text;
    assert!(first_row_hex.contains("0a") || first_row_hex.contains("0b"));
}

// Validates: Requirement 2 AC 10 (empty document)
#[test]
fn empty_document_displays_single_row() {
    let doc = VecByteReader::new(vec![]);
    let mut ctrl = HexModeController::new(HexConfig::default());
    ctrl.activate(0, 0).unwrap();

    let vm = ctrl.build_view_model(&doc);
    assert_eq!(vm.total_rows, 1);
    assert_eq!(vm.visible_rows.len(), 1);
    assert_eq!(vm.visible_rows[0].offset_text, "00000000");
}

// Validates: Requirement 8 (modified byte tracking across edit cycle)
#[test]
fn modified_bytes_tracked_through_edit_and_save() {
    let (mut ctrl, doc) = setup_256();
    let doc_len = doc.byte_length();

    ctrl.activate(5, doc_len).unwrap();

    // Edit byte at offset 5
    ctrl.handle_input(HexInput::HexDigit('A'), &doc).unwrap();
    assert!(ctrl.modified_tracker().is_modified(5));

    // Save clears all
    ctrl.on_document_saved();
    assert!(!ctrl.modified_tracker().is_modified(5));
    assert!(!ctrl.modified_tracker().has_modifications());
}
