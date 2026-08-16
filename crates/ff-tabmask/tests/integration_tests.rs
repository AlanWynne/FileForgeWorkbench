//! Integration tests for ff-tabmask.
//!
//! End-to-end scenario tests verifying the interaction of multiple components.

use ff_tabmask::traits::{ConfigProvider, LanguageDefinitionRef};
use ff_tabmask::{
    compute_shift_left, compute_shift_right, compute_tab_action, execute_line_command,
    execute_mask_command, execute_reset_tabs, execute_tabs_command, handle_reset, ArtifactKind,
    ArtifactPosition, DefaultsLoader, DisplayArtifactManager, EditorMode, MaskCommandResult,
    MaskLine, MaskManager, MaskState, TabKeyAction, TabStopList, TabStopSource, TabsCommandResult,
    TabsMaskState, TabsState,
};

/// A test config provider for integration tests.
struct TestConfig {
    tab_stops: Vec<u32>,
    tab_size: u32,
}

impl ConfigProvider for TestConfig {
    fn get_tab_stops(&self) -> Vec<u32> {
        self.tab_stops.clone()
    }

    fn get_tab_size(&self) -> u32 {
        self.tab_size
    }
}

// ─── Integration Test 18.1: Session initialization with COBOL profile ───────

#[test]
fn session_initialization_with_cobol_language_profile() {
    // Validates: Requirements 4.3, 10.1
    let config = TestConfig {
        tab_stops: vec![9, 17, 25],
        tab_size: 8,
    };

    let toml_str = r#"
        default_tab_stops = [7, 12, 72]
        default_mask = "      *"
    "#;
    let toml_val: toml::Value = toml::from_str(toml_str).unwrap();
    let lang_def = LanguageDefinitionRef::new(&toml_val);

    let state = DefaultsLoader::init_session(&config, Some(&lang_def), 80);

    // Language def tab stops take precedence
    assert_eq!(
        state.tabs().tab_stops(),
        &TabStopList::from_columns(vec![7, 12, 72])
    );
    assert_eq!(state.tabs().source(), &TabStopSource::LanguageDefinition);

    // Mask loaded from language definition
    assert!(state.mask().is_active());
    assert_eq!(state.mask().mask().unwrap().content(), "      *");
    assert!(state.mask().from_language());
}

// ─── Integration Test 18.2: TABS command then Tab key ───────────────────────

#[test]
fn tabs_command_then_tab_key_in_insert_mode() {
    // Validates: Requirements 2.1, 5.1, 5.5
    let mut state = TabsMaskState::new(
        TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
        MaskState::empty(),
    );

    // Set tab stops via TABS command
    let result = execute_tabs_command(&mut state, &["7", "12", "72"], Some(0), 80).unwrap();
    assert!(matches!(result, TabsCommandResult::StopsUpdated { .. }));

    // Press Tab at column 3 in Insert mode
    let action = compute_tab_action(
        state.tabs().tab_stops(),
        3,
        EditorMode::Insert,
        false,
        8,
        80,
    );
    assert_eq!(action, TabKeyAction::InsertSpacesTo { target_column: 7 });

    // Press Tab at column 7
    let action = compute_tab_action(
        state.tabs().tab_stops(),
        7,
        EditorMode::Insert,
        false,
        8,
        80,
    );
    assert_eq!(action, TabKeyAction::InsertSpacesTo { target_column: 12 });
}

// ─── Integration Test 18.3: TABS command then Tab key in Overstrike mode ────

#[test]
fn tabs_command_then_tab_key_in_overstrike_mode() {
    // Validates: Requirements 5.6
    let state = TabsMaskState::new(
        TabsState::new(
            TabStopList::from_columns(vec![5, 10, 15]),
            TabStopSource::BuiltIn,
        ),
        MaskState::empty(),
    );

    let action = compute_tab_action(
        state.tabs().tab_stops(),
        3,
        EditorMode::Overstrike,
        false,
        8,
        80,
    );
    assert_eq!(action, TabKeyAction::MoveCursorTo { target_column: 5 });

    // Tab does NOT insert characters in Overstrike mode — just moves cursor
    let action = compute_tab_action(
        state.tabs().tab_stops(),
        5,
        EditorMode::Overstrike,
        false,
        8,
        80,
    );
    assert_eq!(action, TabKeyAction::MoveCursorTo { target_column: 10 });
}

// ─── Integration Test 18.4: MASK command then I line command ────────────────

#[test]
fn mask_command_then_insert_line_uses_mask_content() {
    // Validates: Requirements 6.1, 9.1, 9.5
    let mut state = TabsMaskState::new(
        TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
        MaskState::with_mask(MaskLine::new("      *"), false),
    );

    // Display the mask
    let result = execute_mask_command(&mut state, &[], Some(5), 80).unwrap();
    assert_eq!(result, MaskCommandResult::LinesAdded { count: 1 });

    // Insert a line — mask should be applied
    let content = MaskManager::apply_mask(state.mask(), 72);
    assert!(content.is_some());
    let line = content.unwrap();
    assert!(line.starts_with("      *"));
    assert_eq!(line.len(), 72);
}

// ─── Integration Test 18.5: MASK edit in place ──────────────────────────────

#[test]
fn mask_edit_in_place_updates_subsequent_inserts() {
    // Validates: Requirements 6.4, 9.1
    let mut state = TabsMaskState::new(
        TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
        MaskState::with_mask(MaskLine::new("OLD MASK"), false),
    );

    // Edit the mask content (simulating in-place MASK_Line editing)
    state.mask_mut().update_mask("NEW MASK CONTENT".to_string());

    // Subsequent insert uses updated mask
    let content = MaskManager::apply_mask(state.mask(), 20);
    assert_eq!(content, Some("NEW MASK CONTENT    ".to_string()));
}

// ─── Integration Test 18.6: MASK OFF then I ─────────────────────────────────

#[test]
fn mask_off_then_insert_produces_blank_line() {
    // Validates: Requirements 7.1, 9.3
    let mut state = TabsMaskState::new(
        TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
        MaskState::with_mask(MaskLine::new("      *"), true),
    );

    // Clear mask with MASK OFF
    let result = execute_mask_command(&mut state, &["OFF"], None, 80).unwrap();
    assert_eq!(result, MaskCommandResult::MaskCleared);

    // Insert line — no mask applied
    let content = MaskManager::apply_mask(state.mask(), 72);
    assert!(content.is_none());
}

// ─── Integration Test 18.7: RESET clears display but preserves state ────────

#[test]
fn reset_clears_display_but_preserves_state() {
    // Validates: Requirements 11.1, 11.3, 11.4
    let stops = TabStopList::from_columns(vec![7, 12, 72]);
    let mut state = TabsMaskState::new(
        TabsState::new(stops.clone(), TabStopSource::LanguageDefinition),
        MaskState::with_mask(MaskLine::new("      *"), true),
    );

    // Add TABS and MASK display lines
    state.add_tabs_line(ArtifactPosition {
        anchor_line: 3,
        from_line_command: false,
    });
    state.add_mask_line(ArtifactPosition {
        anchor_line: 7,
        from_line_command: false,
    });

    // RESET
    handle_reset(&mut state);

    // Display lines removed
    assert!(!state.has_tabs_lines());
    assert!(!state.has_mask_lines());

    // State preserved
    assert_eq!(state.tabs().tab_stops(), &stops);
    assert!(state.mask().is_active());
    assert_eq!(state.mask().mask().unwrap().content(), "      *");
}

// ─── Integration Test 18.8: RESET TABS restores language defaults ───────────

#[test]
fn reset_tabs_restores_language_defaults() {
    // Validates: Requirements 12.1, 12.2
    let defaults = TabStopList::from_columns(vec![7, 12, 72]);
    let mut state = TabsMaskState::new(
        TabsState::new(defaults.clone(), TabStopSource::LanguageDefinition),
        MaskState::empty(),
    );

    // Override with custom stops
    execute_tabs_command(&mut state, &["5", "10", "15"], Some(0), 80).unwrap();
    assert_eq!(
        state.tabs().tab_stops(),
        &TabStopList::from_columns(vec![5, 10, 15])
    );

    // RESET TABS restores defaults
    execute_reset_tabs(&mut state, 80).unwrap();
    assert_eq!(state.tabs().tab_stops(), &defaults);
}

// ─── Integration Test 18.9: Shift right/left with tab stops ─────────────────

#[test]
fn shift_right_left_with_tab_stops() {
    // Validates: Requirements 14.1, 14.2
    let stops = TabStopList::from_columns(vec![5, 10, 15, 20]);

    // Shift right from column 5
    let right = compute_shift_right(&stops, 5, 1);
    assert_eq!(right.target_column, 10);
    assert_eq!(right.delta, 5);

    // Shift left from column 10
    let left = compute_shift_left(&stops, 10, 1);
    assert_eq!(left.target_column, 5);
    assert_eq!(left.delta, -5);

    // Shift right by 2
    let right2 = compute_shift_right(&stops, 5, 2);
    assert_eq!(right2.target_column, 15);
    assert_eq!(right2.delta, 10);
}

// ─── Integration Test 18.10: Multiple TABS_Lines at different positions ─────

#[test]
fn multiple_tabs_lines_toggle_removes_all() {
    // Validates: Requirements 1.7, 1.4
    let mut state = TabsMaskState::new(
        TabsState::new(
            TabStopList::from_columns(vec![5, 10, 15]),
            TabStopSource::BuiltIn,
        ),
        MaskState::empty(),
    );

    // Insert TABS_Line at cursor position 3
    execute_tabs_command(&mut state, &[], Some(3), 80).unwrap();
    // Insert another via line command at position 10
    execute_line_command(&mut state, ArtifactKind::TabsLine, 10, 80).unwrap();
    assert_eq!(state.tabs_lines().len(), 2);

    // Toggle off removes ALL
    execute_tabs_command(&mut state, &[], Some(0), 80).unwrap();
    assert!(!state.has_tabs_lines());
}

// ─── Integration Test 18.11: Tab key with empty stop list ───────────────────

#[test]
fn tab_key_with_empty_stop_list_uses_tab_size() {
    // Validates: Requirement 5.3
    let stops = TabStopList::empty();
    let action = compute_tab_action(&stops, 5, EditorMode::Insert, false, 4, 80);
    assert_eq!(action, TabKeyAction::AdvanceBySize { spaces: 4 });
}

// ─── Integration Test 18.12: Language definition precedence ─────────────────

#[test]
fn language_definition_precedence_over_global_config() {
    // Validates: Requirements 4.3, 13.6
    let config = TestConfig {
        tab_stops: vec![9, 17, 25, 33],
        tab_size: 8,
    };

    let toml_str = r#"default_tab_stops = [7, 12, 72]"#;
    let toml_val: toml::Value = toml::from_str(toml_str).unwrap();
    let lang_def = LanguageDefinitionRef::new(&toml_val);

    let (stops, source) = DefaultsLoader::load_tab_stops(&config, Some(&lang_def), 80);
    assert_eq!(stops, TabStopList::from_columns(vec![7, 12, 72]));
    assert_eq!(source, TabStopSource::LanguageDefinition);
}

// ─── Integration Test 18.13: MASK in Browse mode ────────────────────────────

#[test]
fn mask_in_browse_mode_display_only() {
    // Validates: Requirement 6.11
    // In browse mode, MASK is displayed but not editable.
    // The render function still works (display is permitted).
    let mask = MaskLine::new("      *");
    let rendered = DisplayArtifactManager::render_mask_line(&mask, 72);
    assert_eq!(rendered.len(), 72);
    assert!(rendered.starts_with("      *"));

    // Metadata confirms Browse mode is valid for MASK
    let meta = DisplayArtifactManager::artifact_metadata(ArtifactKind::MaskLine);
    assert!(meta.applicable_modes.contains(&EditorMode::Browse));
}

// ─── Integration Test 18.14: In command with active mask (n lines) ──────────

#[test]
fn in_command_with_active_mask_produces_n_identical_lines() {
    // Validates: Requirements 9.2, 9.4
    let mask_state = MaskState::with_mask(MaskLine::new("      *"), false);

    // Insert 3 lines — all should have mask content
    let lines = MaskManager::apply_mask_to_n_lines(&mask_state, 72, 3);
    assert_eq!(lines.len(), 3);
    for line in &lines {
        assert_eq!(line.len(), 72);
        assert!(line.starts_with("      *"));
    }

    // All lines are identical (same mask applied to each)
    assert_eq!(lines[0], lines[1]);
    assert_eq!(lines[1], lines[2]);
}
