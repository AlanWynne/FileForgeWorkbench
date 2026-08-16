//! Integration tests for ff-caret-selection.
//!
//! End-to-end validation across Requirements 1–12.

use ff_caret_selection::{
    BlinkState, CaretColours, CaretLineConfig, CaretLineMode, CaretSelectionConfig, CaretShape,
    CaretStyle, CaretWidth, ColourRGBA, FocusState, ModifiedMarkerConfig, MultiCaretDisplay,
    RectangularSelectionDisplay, SelectionColourSet, SelectionContext, VirtualSpaceRenderer,
};
use ff_edit_operations::{
    EditMode, ModifiedLineTracker, SelectionContainer, SelectionPosition, SelectionRange,
};

// ─── Integration Test 16.1 ──────────────────────────────────────────────────
// Full caret configuration load from theme → render info generation lifecycle

#[test]
fn full_caret_config_to_render_info_lifecycle() {
    // Validates: Requirement 11 — theme integration lifecycle

    // Step 1: Create configuration (simulating theme load)
    let config = CaretSelectionConfig::new();

    // Step 2: Verify all defaults are sane
    assert_eq!(
        config.shape.effective_style(EditMode::Insert),
        CaretStyle::Line
    );
    assert_eq!(config.shape.effective_width(), 1);
    assert_eq!(config.blink.period_ms(), 530);
    assert_eq!(config.caret_line.mode(), CaretLineMode::Frame);
    assert!(config.selection_display.is_visible());

    // Step 3: Use the config to produce render info via MultiCaretDisplay
    let selection = SelectionContainer::new();
    let display = MultiCaretDisplay::new(&selection, &config.shape, &config.colours);
    let carets = display.caret_render_list(EditMode::Insert);

    assert_eq!(carets.len(), 1);
    assert!(carets[0].is_primary);
    assert_eq!(carets[0].style, CaretStyle::Line);
    assert_eq!(carets[0].width, 1);
    assert_eq!(carets[0].colour, ColourRGBA::rgb(0, 0, 0));

    // Step 4: Verify blink integration
    let mut blink = config.blink.clone();
    blink.reset(0);
    assert!(blink.is_visible(100));
    assert!(!blink.is_visible(300));
}

// ─── Integration Test 16.2 ──────────────────────────────────────────────────
// Multi-caret scenario with mixed primary/additional colours and selections

#[test]
fn multi_caret_with_mixed_colours_and_selections() {
    // Validates: Requirements 2, 9 — multi-caret with colour assignment

    let mut selection = SelectionContainer::with_range(SelectionRange::new(
        SelectionPosition::new(1, 0),
        SelectionPosition::new(1, 10),
    ));
    selection.add(SelectionRange::new(
        SelectionPosition::new(3, 5),
        SelectionPosition::new(3, 15),
    ));
    selection.add(SelectionRange::collapsed(SelectionPosition::new(7, 0)));

    let shape = CaretShape::new(CaretStyle::Line, CaretWidth::new(2));
    let colours = CaretColours::new(
        ColourRGBA::rgb(255, 0, 0), // red primary
        ColourRGBA::rgb(0, 0, 255), // blue additional
    );

    let display = MultiCaretDisplay::new(&selection, &shape, &colours);

    // Verify caret list
    let carets = display.caret_render_list(EditMode::Insert);
    assert_eq!(carets.len(), 3);

    // Primary caret (index 0 is main by default)
    assert!(carets[0].is_primary);
    assert_eq!(carets[0].colour, ColourRGBA::rgb(255, 0, 0));
    assert_eq!(carets[0].width, 2);

    // Additional carets
    assert!(!carets[1].is_primary);
    assert_eq!(carets[1].colour, ColourRGBA::rgb(0, 0, 255));
    assert!(!carets[2].is_primary);
    assert_eq!(carets[2].colour, ColourRGBA::rgb(0, 0, 255));

    // Verify selection list
    let selections = display.selection_render_list();
    assert_eq!(selections.len(), 2); // Only non-collapsed ranges
    assert_eq!(selections[0].colour_context, SelectionContext::Primary);
    assert_eq!(selections[1].colour_context, SelectionContext::Additional);
}

// ─── Integration Test 16.3 ──────────────────────────────────────────────────
// Rectangular selection spanning lines with virtual space extension

#[test]
fn rectangular_selection_with_virtual_space() {
    // Validates: Requirements 7, 8 — rectangular + virtual space

    let rect_display = RectangularSelectionDisplay;
    let vs_renderer = VirtualSpaceRenderer;

    // Column band from col 5 to col 20, line content only 10 chars
    let (left, right) = rect_display.column_band_for_line(5, 20, 10, 8.0);
    assert_eq!(left, 40.0); // 5 * 8
    assert_eq!(right, 160.0); // 20 * 8 — extends into virtual space

    // Virtual space caret positioning
    let caret_x = vs_renderer.horizontal_offset(80.0, 10, 8.0);
    assert_eq!(caret_x, 160.0); // 80 + 10*8

    // Virtual space selection rect
    let rect = vs_renderer.selection_rect_in_virtual_space(80.0, 0, 10, 8.0, 20.0);
    assert_eq!(rect.x, 80.0);
    assert_eq!(rect.width, 80.0); // 10 * 8
    assert_eq!(rect.height, 20.0);

    // Thin selection at column 15
    let thin_x = rect_display.thin_selection_x(15, 8.0);
    assert_eq!(thin_x, 120.0);
}

// ─── Integration Test 16.4 ──────────────────────────────────────────────────
// Caret-line highlight mode switching (None → Frame → Fill) with hot-reload

#[test]
fn caret_line_mode_switching_with_hot_reload() {
    // Validates: Requirements 4, 11 — mode switching and hot-reload

    let mut config = CaretLineConfig::new();
    assert_eq!(config.mode(), CaretLineMode::Frame);
    assert!(config.should_show(true));
    assert!(!config.should_show(false));

    // Switch to None
    config.set_mode(CaretLineMode::None);
    assert!(!config.should_show(true));
    assert!(!config.should_show(false));

    // Switch to Fill
    config.set_mode(CaretLineMode::Fill);
    assert!(config.should_show(true));

    // Simulate hot-reload: set always_show true
    config.set_always_show(true);
    assert!(config.should_show(false)); // now shows when unfocused

    // Verify frame width clamping still works after mode changes
    config.set_mode(CaretLineMode::Frame);
    config.set_frame_width(50);
    assert_eq!(config.effective_frame_width(30), 10); // 30/3 = 10
}

// ─── Integration Test 16.5 ──────────────────────────────────────────────────
// Focus gain/loss cycle with blink reset and inactive selection colour switch

#[test]
fn focus_cycle_with_blink_and_selection_colours() {
    // Validates: Requirements 6, 12 — focus integration

    let mut focus = FocusState::new();
    let mut blink = BlinkState::new(500);
    let colours = SelectionColourSet::default();

    // Initially unfocused
    assert!(!focus.is_caret_visible());

    // Focus gained at time 0
    focus.on_focus_gained(&mut blink, 0);
    assert!(focus.is_caret_visible());
    assert!(blink.is_visible(0));

    // While focused, use primary selection colours
    let (text, back) = colours.colours_for_context(SelectionContext::Primary);
    assert_eq!(back, ColourRGBA::rgb(192, 192, 192));
    assert_eq!(text, None);

    // Time advances to hidden phase
    assert!(!blink.is_visible(300));

    // Caret moves — reset to visible
    focus.on_caret_moved(&mut blink, 300);
    assert!(blink.is_visible(300));

    // Focus lost
    focus.on_focus_lost();
    assert!(!focus.is_caret_visible());

    // When unfocused, use inactive selection colours
    let (inactive_text, inactive_back) = colours.colours_for_context(SelectionContext::Inactive);
    assert_eq!(inactive_back, ColourRGBA::rgba(128, 128, 128, 0x3F));
    assert_eq!(inactive_text, None);
}

// ─── Integration Test 16.6 ──────────────────────────────────────────────────
// Modified line markers with save-clear cycle

#[test]
fn modified_markers_with_save_clear_cycle() {
    // Validates: Requirement 10 — modified markers lifecycle

    let marker_config = ModifiedMarkerConfig::default();
    let mut tracker = ModifiedLineTracker::new();

    // No markers initially
    assert!(!marker_config.should_render(0, &tracker));
    assert!(!marker_config.should_render(5, &tracker));

    // Modify some lines
    tracker.mark_modified(1);
    tracker.mark_modified(5);
    tracker.mark_modified(10);

    // Markers visible for modified lines
    assert!(marker_config.should_render(1, &tracker));
    assert!(marker_config.should_render(5, &tracker));
    assert!(marker_config.should_render(10, &tracker));
    assert!(!marker_config.should_render(2, &tracker));

    // Marker character is always '*'
    assert_eq!(marker_config.render_char(), '*');

    // Simulate save — clears all markers
    tracker.clear_all();
    assert!(!marker_config.should_render(1, &tracker));
    assert!(!marker_config.should_render(5, &tracker));
    assert!(!marker_config.should_render(10, &tracker));

    // New modifications after save
    tracker.mark_modified(3);
    assert!(marker_config.should_render(3, &tracker));
}
