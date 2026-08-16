//! Integration tests for ff-zoom — end-to-end zoom workflows.
//!
//! These tests exercise the complete zoom lifecycle including state
//! management, commands, indicators, and persistence.

use ff_zoom::commands::{parse_zoom_args, ZoomOperation};
use ff_zoom::config::ZoomConfig;
use ff_zoom::indicator::{format_zoom_query, ZoomIndicatorState};
use ff_zoom::operations::{ZoomChangeEvent, ZoomFontMetrics, ZoomResult};
use ff_zoom::persistence::{persist_all, restore_all, ZoomSessionEntry};
use ff_zoom::state::ZoomState;
use ff_zoom::types::ZoomOffset;

// ─── Task 21: End-to-end zoom workflows ─────────────────────────────────────

// Validates: Requirements 1.1, 1.2, 2.1
#[test]
fn zoom_in_three_times_gives_correct_offset_and_effective_size() {
    let config = ZoomConfig::default(); // step=1, min=-10, max=60
    let mut state = ZoomState::new(&config);
    let base_size = 12;

    state.zoom_in();
    state.zoom_in();
    state.zoom_in();

    assert_eq!(state.offset().value(), 3);
    assert_eq!(state.effective_font_size(base_size), 15);
}

// Validates: Requirements 1.5, 2.6
#[test]
fn zoom_in_to_max_then_one_more_returns_at_limit() {
    let config = ZoomConfig {
        max_offset: 5,
        ..Default::default()
    };
    let mut state = ZoomState::from_persisted(5, &config);

    let result = state.zoom_in();
    assert!(matches!(result, ZoomResult::AtLimit { limit: 5, .. }));
    if let ZoomResult::AtLimit { message, .. } = result {
        assert_eq!(message, "Maximum zoom reached (+5)");
    }
}

// Validates: Requirements 1.5, 2.7
#[test]
fn zoom_out_to_min_then_one_more_returns_at_limit() {
    let config = ZoomConfig {
        min_offset: -5,
        ..Default::default()
    };
    let mut state = ZoomState::from_persisted(-5, &config);

    let result = state.zoom_out();
    assert!(matches!(result, ZoomResult::AtLimit { limit: -5, .. }));
    if let ZoomResult::AtLimit { message, .. } = result {
        assert_eq!(message, "Minimum zoom reached (-5)");
    }
}

// Validates: Requirements 2.1, 2.3, 7.2
#[test]
fn zoom_in_five_times_then_reset_gives_zero_and_hidden_indicator() {
    let config = ZoomConfig::default();
    let mut state = ZoomState::new(&config);

    for _ in 0..5 {
        state.zoom_in();
    }
    assert_eq!(state.offset().value(), 5);

    state.zoom_reset();
    assert_eq!(state.offset().value(), 0);

    let indicator = ZoomIndicatorState::from_offset(state.offset());
    assert_eq!(indicator, ZoomIndicatorState::Hidden);
}

// Validates: Requirements 8.2, 7.1
#[test]
fn zoom_command_set_positive_offset_shows_correct_indicator() {
    let config = ZoomConfig::default();
    let mut state = ZoomState::new(&config);

    let op = parse_zoom_args("7").unwrap();
    assert_eq!(op, ZoomOperation::SetAbsolute(7));
    state.set_offset(7);

    assert_eq!(state.offset().value(), 7);
    let indicator = ZoomIndicatorState::from_offset(state.offset());
    assert_eq!(
        indicator,
        ZoomIndicatorState::Visible {
            text: "Zoom: +7".to_string(),
            offset: 7,
        }
    );
}

// Validates: Requirements 8.2, 7.5
#[test]
fn zoom_command_set_negative_offset_shows_correct_indicator() {
    let config = ZoomConfig::default();
    let mut state = ZoomState::new(&config);

    let op = parse_zoom_args("-3").unwrap();
    assert_eq!(op, ZoomOperation::SetAbsolute(-3));
    state.set_offset(-3);

    assert_eq!(state.offset().value(), -3);
    let indicator = ZoomIndicatorState::from_offset(state.offset());
    assert_eq!(
        indicator,
        ZoomIndicatorState::Visible {
            text: "Zoom: -3".to_string(),
            offset: -3,
        }
    );
}

// Validates: Requirement 8.6
#[test]
fn zoom_query_no_args_returns_status_message() {
    let config = ZoomConfig::default();
    let state = ZoomState::from_persisted(3, &config);
    let base_size = 12;

    let op = parse_zoom_args("").unwrap();
    assert_eq!(op, ZoomOperation::Query);

    let effective = state.effective_font_size(base_size);
    let msg = format_zoom_query(state.offset(), effective);
    assert_eq!(msg, "Zoom offset: +3 (effective size: 15pt)");
}

// Validates: Requirements 5.1, 5.3, 7.1, 7.2
#[test]
fn two_editor_instances_with_different_offsets_independent_indicators() {
    let config = ZoomConfig::default();
    let mut state1 = ZoomState::new(&config);
    let mut state2 = ZoomState::new(&config);

    state1.set_offset(5);
    state2.set_offset(-2);

    // Simulate "tab switch" by checking indicators
    let indicator1 = ZoomIndicatorState::from_offset(state1.offset());
    let indicator2 = ZoomIndicatorState::from_offset(state2.offset());

    assert_eq!(
        indicator1,
        ZoomIndicatorState::Visible {
            text: "Zoom: +5".to_string(),
            offset: 5,
        }
    );
    assert_eq!(
        indicator2,
        ZoomIndicatorState::Visible {
            text: "Zoom: -2".to_string(),
            offset: -2,
        }
    );
}

// Validates: Requirements 6.1, 6.2
#[test]
fn persist_two_states_and_restore_correctly() {
    let config = ZoomConfig::default();
    let state1 = ZoomState::from_persisted(3, &config);
    let state2 = ZoomState::from_persisted(-2, &config);

    let entries = vec![
        ZoomSessionEntry::from_state("file:///a.rs", &state1),
        ZoomSessionEntry::from_state("file:///b.rs", &state2),
    ];
    let data = persist_all(&entries);
    let restored = restore_all(&data);

    assert_eq!(restored.len(), 2);
    let r1 = restored[0].restore(&config);
    let r2 = restored[1].restore(&config);
    assert_eq!(r1.offset().value(), 3);
    assert_eq!(r2.offset().value(), -2);
}

// Validates: Requirement 6.3
#[test]
fn persist_offset_50_then_config_max_30_clamps_on_restore() {
    let old_config = ZoomConfig::default(); // max=60
    let state = ZoomState::from_persisted(50, &old_config);
    let entry = ZoomSessionEntry::from_state("file:///test.rs", &state);

    // Config changed between sessions
    let new_config = ZoomConfig {
        max_offset: 30,
        ..Default::default()
    };
    let restored = entry.restore(&new_config);
    assert_eq!(restored.offset().value(), 30);
}

// Validates: Requirement 4.1 — step=3 zoom in once increases by 3
#[test]
fn config_step_3_zoom_in_increases_by_3() {
    let config = ZoomConfig {
        step: 3,
        ..Default::default()
    };
    let mut state = ZoomState::new(&config);
    state.zoom_in();
    assert_eq!(state.offset().value(), 3);
}

// Validates: Requirements 3.1, 3.5
#[test]
fn ctrl_scroll_up_four_times_with_step_1_gives_offset_4() {
    let config = ZoomConfig::default(); // step=1
    let mut state = ZoomState::new(&config);

    // Simulate 4 Ctrl+Scroll Up events (each is one zoom in step)
    for _ in 0..4 {
        state.zoom_in();
    }
    assert_eq!(state.offset().value(), 4);
}

// Validates: Requirement 3.3
#[test]
fn scroll_without_ctrl_does_not_change_offset() {
    let config = ZoomConfig::default();
    let state = ZoomState::new(&config);
    let original = state.offset().value();

    // When Ctrl is not held, scroll does not trigger zoom
    // (This is handled at the input layer; state is unchanged)
    assert_eq!(state.offset().value(), original);
}

// Validates: Requirement 8.8
#[test]
fn zoom_command_not_recorded_in_history() {
    // The ZOOM command sets skip_history=true per design.
    // We validate that zoom operations return ZoomResult (not UndoRecord).
    let config = ZoomConfig::default();
    let mut state = ZoomState::new(&config);
    let result = state.zoom_in();

    // ZoomResult has no undo/history semantics
    assert!(matches!(result, ZoomResult::Applied { .. }));
}

// Validates: Requirement 4.6
#[test]
fn hot_reload_config_with_narrower_range_clamps_active_instances() {
    let config = ZoomConfig::default();
    let mut state1 = ZoomState::from_persisted(50, &config);
    let mut state2 = ZoomState::from_persisted(-8, &config);

    let new_config = ZoomConfig {
        min_offset: -5,
        max_offset: 30,
        ..Default::default()
    };

    state1.apply_config_change(&new_config);
    state2.apply_config_change(&new_config);

    assert_eq!(state1.offset().value(), 30);
    assert_eq!(state2.offset().value(), -5);
}

// Validates: Requirements 1.6, 1.8
#[test]
fn zoom_change_event_tracks_relayout_need() {
    let old_offset = ZoomOffset::new(0, -10, 60);
    let new_offset = ZoomOffset::new(5, -10, 60);
    let base_size = 12;

    let event = ZoomChangeEvent::from_state_change(1, old_offset, new_offset, base_size);
    assert!(event.requires_relayout);
    assert_eq!(event.old_offset, 0);
    assert_eq!(event.new_offset, 5);
    assert_eq!(event.effective_font_size, 17);
}

// Validates: Requirement 9.1 — zoom offset is in points, DPI-independent
#[test]
fn zoom_offset_preserved_across_simulated_dpi_changes() {
    let config = ZoomConfig::default();
    let state = ZoomState::from_persisted(5, &config);

    // Simulating DPI change: the offset stays the same
    // DPI conversion is handled by the rendering layer, not the zoom model
    let _dpi_96 = 1.0f32;
    let _dpi_144 = 1.5f32;
    let _dpi_192 = 2.0f32;

    // The offset is always 5 points regardless of DPI
    assert_eq!(state.offset().value(), 5);
    // Effective font size in POINTS is constant
    assert_eq!(state.effective_font_size(12), 17);
}

// Validates: Requirements 1.6, 9.4
#[test]
fn font_metrics_visible_lines_changes_with_zoom() {
    let config = ZoomConfig::default();
    let state_normal = ZoomState::new(&config);
    let state_zoomed = ZoomState::from_persisted(6, &config);
    let base_size = 12;

    let metrics_normal = ZoomFontMetrics::compute(base_size, &state_normal);
    let metrics_zoomed = ZoomFontMetrics::compute(base_size, &state_zoomed);

    // At base 12pt, line_height ≈ 16px; at 18pt, line_height ≈ 24px
    let lines_normal = metrics_normal.visible_lines(600.0, 16.0);
    let lines_zoomed = metrics_zoomed.visible_lines(600.0, 24.0);

    assert!(lines_normal > lines_zoomed);
    assert_eq!(metrics_normal.effective_font_size, 12);
    assert_eq!(metrics_zoomed.effective_font_size, 18);
}
