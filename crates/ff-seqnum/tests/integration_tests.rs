//! Integration tests for the ff-seqnum crate.
//!
//! End-to-end tests exercising the complete sequence number lifecycle.

use ff_seqnum::*;

// ─── Test Helpers ───────────────────────────────────────────────────────────

struct TestDoc {
    lines: Vec<String>,
}

impl TestDoc {
    fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl DocumentAccess for TestDoc {
    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn line_content(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }
}

impl DocumentMutate for TestDoc {
    fn replace_columns(&mut self, line_index: usize, range: &ColumnRange, content: &str) {
        if let Some(line) = self.lines.get_mut(line_index) {
            let start = range.start_offset();
            let end = range.end_offset();
            if line.len() <= start {
                let padding = " ".repeat(start - line.len());
                line.push_str(&padding);
                line.push_str(content);
                return;
            }
            let actual_end = end.min(line.len());
            let mut new_line = String::with_capacity(line.len().max(end));
            new_line.push_str(&line[..start]);
            new_line.push_str(content);
            if actual_end < line.len() {
                new_line.push_str(&line[actual_end..]);
            }
            *line = new_line;
        }
    }
}

struct TestProfile {
    front: Option<ColumnRange>,
    back: Option<ColumnRange>,
    auto_unnum_val: bool,
    lang_id: String,
}

impl TestProfile {
    fn cobol() -> Self {
        Self {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum_val: true,
            lang_id: "cobol".to_string(),
        }
    }

    fn jcl() -> Self {
        Self {
            front: None,
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum_val: true,
            lang_id: "jcl".to_string(),
        }
    }

    fn no_seq_cols() -> Self {
        Self {
            front: None,
            back: None,
            auto_unnum_val: true,
            lang_id: "text".to_string(),
        }
    }
}

impl LanguageProfile for TestProfile {
    fn sequence_cols_front(&self) -> Option<ColumnRange> {
        self.front
    }
    fn sequence_cols_back(&self) -> Option<ColumnRange> {
        self.back
    }
    fn auto_unnum(&self) -> bool {
        self.auto_unnum_val
    }
    fn language_id(&self) -> &str {
        &self.lang_id
    }
}

fn make_cobol_line(seq_front: &str, body: &str, seq_back: &str) -> String {
    let front = format!("{:<6}", seq_front);
    let body_padded = format!("{:<66}", body);
    let back = format!("{:<8}", seq_back);
    format!("{}{}{}", &front[..6], &body_padded[..66], &back[..8])
}

fn make_jcl_line(body: &str, seq_back: &str) -> String {
    let body_padded = format!("{:<72}", body);
    let back = format!("{:<8}", seq_back);
    format!("{}{}", &body_padded[..72], &back[..8])
}

// ─── Integration Tests ──────────────────────────────────────────────────────

#[test]
fn cobol_file_open_auto_detect_and_strip() {
    // Validates: Requirements 2.1, 2.2, 3.1, 3.4, 3.9
    let lines: Vec<String> = (1..=20)
        .map(|i| {
            make_cobol_line(
                &format!("{:06}", i * 100),
                &format!(" MOVE LINE{i} TO DEST."),
                &format!("{:08}", i * 100),
            )
        })
        .collect();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile::cobol();
    let config = SeqNumConfig::default();
    let mut state = SeqNumState::new();

    let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

    // Verify stripping occurred
    match result {
        AutoStripResult::Stripped {
            front,
            back,
            message,
        } => {
            assert!(front.is_some());
            assert!(back.is_some());
            assert!(message.contains("SEQUENCE NUMBERS REMOVED"));
            assert!(message.contains("1-6"));
            assert!(message.contains("73-80"));
        }
        _ => panic!("Expected Stripped result, got {:?}", result),
    }

    // Verify edit buffer is clean (front stripped)
    for i in 0..20 {
        let line = doc.line_content(i).unwrap();
        assert_eq!(&line[..6], "      ", "Line {} front not stripped", i);
        assert_eq!(&line[72..80], "        ", "Line {} back not stripped", i);
    }

    // Verify side-table is populated
    assert!(!state.side_table.is_empty());
    let entry = state.side_table.get_original_values(0).unwrap();
    assert_eq!(entry.front_content.as_deref(), Some("000100"));
    assert_eq!(entry.back_content.as_deref(), Some("00000100"));

    // Verify status indicator
    assert_eq!(
        state.status_indicator(),
        SeqNumStatusIndicator::Stripped {
            has_front: true,
            has_back: true
        }
    );
}

#[test]
fn jcl_file_open_back_only_strip() {
    // Validates: Requirements 1.7, 2.4, 3.1
    let lines: Vec<String> = (1..=10)
        .map(|i| {
            make_jcl_line(
                &format!("//STEP{i}  EXEC PGM=PROG{i}"),
                &format!("{:08}", i * 100),
            )
        })
        .collect();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile::jcl();
    let config = SeqNumConfig::default();
    let mut state = SeqNumState::new();

    let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

    match result {
        AutoStripResult::Stripped {
            front,
            back,
            message,
        } => {
            assert!(front.is_none()); // JCL has no front columns
            assert!(back.is_some());
            assert!(message.contains("73-80"));
            assert!(!message.contains("1-6")); // No front in message
        }
        _ => panic!("Expected Stripped result"),
    }

    // Verify back stripped, front content untouched
    for i in 0..10 {
        let line = doc.line_content(i).unwrap();
        assert!(
            line.starts_with("//STEP"),
            "Line {} front should be untouched",
            i
        );
        assert_eq!(&line[72..80], "        ", "Line {} back not stripped", i);
    }
}

#[test]
fn unnum_and_undo_cycle() {
    // Validates: Requirements 5.2, 5.9, 5.10, 9.1, 9.5
    let lines: Vec<String> = (1..=10)
        .map(|i| {
            make_cobol_line(
                &format!("{:06}", i * 100),
                " MOVE A TO B.",
                &format!("{:08}", i * 100),
            )
        })
        .collect();
    let original_lines: Vec<String> = lines.clone();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile::cobol();
    let mut state = SeqNumState::new();

    // Execute UNNUM
    let result =
        execute_unnum(&mut doc, &profile, &UnnumVariant::Default, None, &mut state).unwrap();

    assert_eq!(result.lines_modified, 10);
    assert!(result.message.contains("10 lines modified"));

    // Verify stripped
    for i in 0..10 {
        assert_eq!(&doc.line_content(i).unwrap()[..6], "      ");
    }

    // Simulate UNDO via restore_from_side_table
    let front = ColumnRange::new(1, 6).unwrap();
    let back = ColumnRange::new(73, 80).unwrap();
    let restored = restore_from_side_table(&mut doc, &state.side_table, Some(&front), Some(&back));
    assert_eq!(restored, 10);

    // Verify restored matches original
    for (i, original) in original_lines.iter().enumerate() {
        assert_eq!(doc.line_content(i).unwrap(), original.as_str());
    }
}

#[test]
fn number_std_and_undo_cycle() {
    // Validates: Requirements 6.4, 6.6, 9.2, 9.6
    let lines: Vec<String> = (1..=5)
        .map(|_| make_cobol_line("      ", " MOVE A TO B.", "        "))
        .collect();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile::cobol();
    let mut state = SeqNumState::new();

    // Execute NUMBER STD
    let result = execute_number(
        &mut doc,
        &profile,
        &NumberVariant::Std {
            start_value: 100,
            increment: 100,
        },
        None,
        &mut state,
        &SequenceFormat::Numeric,
    );

    match result {
        NumberCommandResult::Completed { result, .. } => {
            assert_eq!(result.lines_numbered, 5);
            assert!(!result.overflow_occurred);
        }
        _ => panic!("Expected Completed"),
    }

    // Verify sequential values in back cols (73-80)
    assert!(doc.line_content(0).unwrap()[72..80].contains("00000100"));
    assert!(doc.line_content(1).unwrap()[72..80].contains("00000200"));
    assert!(doc.line_content(4).unwrap()[72..80].contains("00000500"));

    // NUMBER STD uses back columns — front should be unchanged (spaces)
    for i in 0..5 {
        assert_eq!(&doc.line_content(i).unwrap()[..6], "      ");
    }
}

#[test]
fn number_show_overlay() {
    // Validates: Requirements 8.1, 8.2, 8.4, 8.7
    let lines: Vec<String> = (1..=5)
        .map(|i| {
            make_cobol_line(
                &format!("{:06}", i * 100),
                " CODE.",
                &format!("{:08}", i * 100),
            )
        })
        .collect();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile::cobol();
    let config = SeqNumConfig::default();
    let mut state = SeqNumState::new();

    // Strip on open
    auto_strip_on_open(&mut doc, &profile, &config, &mut state);

    // Verify overlay is None when mode is off
    assert!(get_overlay_content(&state, 0).is_none());

    // Toggle NUMBER SHOW on
    let active = toggle_show_mode(&mut state);
    assert!(active);

    // Verify overlay returns original values
    let overlay = get_overlay_content(&state, 0).unwrap();
    assert_eq!(overlay.front_text.as_deref(), Some("000100"));
    assert_eq!(overlay.back_text.as_deref(), Some("00000100"));

    // Toggle off — overlay returns None
    toggle_show_mode(&mut state);
    assert!(get_overlay_content(&state, 0).is_none());
}

#[test]
fn number_on_auto_numbering() {
    // Validates: Requirements 6.7, 6.8
    let lines: Vec<String> = (1..=3)
        .map(|_| {
            "      CODE LINE HERE                                                            "
                .to_string()
        })
        .collect();
    let mut doc = TestDoc::new(lines);
    let range = ColumnRange::new(1, 6).unwrap();

    let mut auto_state = AutoNumberState {
        next_value: 100,
        increment: 100,
        target_columns: range,
        format: SequenceFormat::Numeric,
    };

    // Simulate auto-numbering on line insert
    auto_number_line(&mut doc, 0, &mut auto_state).unwrap();
    assert!(doc.line_content(0).unwrap().starts_with("000100"));
    assert_eq!(auto_state.next_value, 200);

    auto_number_line(&mut doc, 1, &mut auto_state).unwrap();
    assert!(doc.line_content(1).unwrap().starts_with("000200"));
    assert_eq!(auto_state.next_value, 300);
}

#[test]
fn restore_on_save() {
    // Validates: Requirement 11.5
    let lines: Vec<String> = (1..=5)
        .map(|i| {
            make_cobol_line(
                &format!("{:06}", i * 100),
                " CODE.",
                &format!("{:08}", i * 100),
            )
        })
        .collect();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile::cobol();
    let config = SeqNumConfig::default();
    let mut state = SeqNumState::new();

    // Strip on open
    auto_strip_on_open(&mut doc, &profile, &config, &mut state);

    // Enable restore_on_save
    let save_config = SeqNumConfig {
        restore_on_save: true,
        ..SeqNumConfig::default()
    };

    let decision = prepare_save_content(&doc, &state, &save_config);
    match decision {
        SaveContentDecision::RestoreAndSave { restorations } => {
            assert!(!restorations.is_empty());
            // Verify restoration entries have original content
            let first = restorations.iter().find(|r| r.line_index == 0).unwrap();
            assert_eq!(first.front_content.as_deref(), Some("000100"));
            assert_eq!(first.back_content.as_deref(), Some("00000100"));
        }
        _ => panic!("Expected RestoreAndSave"),
    }
}

#[test]
fn no_sequence_columns_language() {
    // Validates: Requirement 1.9
    let lines: Vec<String> = (1..=10)
        .map(|i| format!("{:06} body content line {}", i * 100, i))
        .collect();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile::no_seq_cols();
    let config = SeqNumConfig::default();
    let mut state = SeqNumState::new();

    let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

    assert_eq!(result, AutoStripResult::NoColumnsConfigured);
    // No stripping occurred
    assert!(doc.line_content(0).unwrap().starts_with("000100"));
}

#[test]
fn configuration_override_disables_auto_strip() {
    // Validates: Requirement 12.4
    let lines: Vec<String> = (1..=10)
        .map(|i| {
            make_cobol_line(
                &format!("{:06}", i * 100),
                " CODE.",
                &format!("{:08}", i * 100),
            )
        })
        .collect();
    let mut doc = TestDoc::new(lines);
    let profile = TestProfile {
        front: Some(ColumnRange::new(1, 6).unwrap()),
        back: Some(ColumnRange::new(73, 80).unwrap()),
        auto_unnum_val: false, // Disabled
        lang_id: "cobol".to_string(),
    };
    let config = SeqNumConfig::default();
    let mut state = SeqNumState::new();

    let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

    match result {
        AutoStripResult::Detected { message } => {
            assert!(message.contains("not removed"));
        }
        _ => panic!("Expected Detected result"),
    }

    // Content should be unchanged
    assert!(doc.line_content(0).unwrap().starts_with("000100"));

    // Status indicator shows SEQNUM?
    assert_eq!(
        state.status_indicator(),
        SeqNumStatusIndicator::DetectedNotStripped
    );
}

#[test]
fn grid_edit_mode_rejection() {
    // Validates: Requirements 13.1, 13.2
    let result = validate_mode(UNNUM_COMMAND_ID, true, true);
    assert!(result.is_some());
    assert!(result.unwrap().contains("Grid Edit Mode"));

    let result = validate_mode(NUMBER_COMMAND_ID, true, true);
    assert!(result.is_some());
    assert!(result.unwrap().contains("Grid Edit Mode"));

    let result = validate_mode(NUMBER_SHOW_COMMAND_ID, true, true);
    assert!(result.is_some());
    assert!(result.unwrap().contains("Grid Edit Mode"));
}
