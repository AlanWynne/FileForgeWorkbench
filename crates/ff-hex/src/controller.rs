//! Hex mode controller.
//!
//! Top-level orchestrator for hex display mode. Owns all hex
//! sub-components and coordinates state transitions, input handling,
//! and view model generation.

use crate::config::HexConfig;
use crate::cursor::HexCursor;
use crate::editing::{HexEditAction, HexEditState};
use crate::error::HexError;
use crate::layout::HexLayout;
use crate::modified_tracker::ModifiedByteTracker;
use crate::search::HexSearchBridge;
use crate::session::HexSessionState;
use crate::types::{ArrowDirection, BytesPerRow, HexInput, HexMode, HexPane};
use crate::view_model::{HexByteMetadata, HexCursorRenderState, HexRow, HexViewModel};
use crate::viewport_adapter::HexViewportAdapter;

/// Trait for read-only byte-level access to the document buffer.
///
/// Implemented by the document model's buffer type. Enables hex mode
/// to read raw bytes without owning the document.
pub trait ByteReader {
    /// Read a single byte at the given offset.
    /// Returns None if offset >= document length.
    fn byte_at(&self, offset: u64) -> Option<u8>;

    /// Read a contiguous range of bytes.
    /// Returns a Vec with actual bytes read (may be shorter at EOF).
    fn bytes_in_range(&self, start: u64, end: u64) -> Vec<u8>;

    /// Total byte length of the document.
    fn byte_length(&self) -> u64;
}

/// A simple ByteReader backed by a Vec<u8>.
///
/// Useful for testing and for hex dump operations.
#[derive(Debug, Clone)]
pub struct VecByteReader {
    data: Vec<u8>,
}

impl VecByteReader {
    /// Create a new reader from a byte vector.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl ByteReader for VecByteReader {
    fn byte_at(&self, offset: u64) -> Option<u8> {
        self.data.get(offset as usize).copied()
    }

    fn bytes_in_range(&self, start: u64, end: u64) -> Vec<u8> {
        let start = start as usize;
        let end = (end as usize).min(self.data.len());
        if start >= self.data.len() {
            Vec::new()
        } else {
            self.data[start..end].to_vec()
        }
    }

    fn byte_length(&self) -> u64 {
        self.data.len() as u64
    }
}

/// Top-level orchestrator for hex display mode.
///
/// Owns all hex sub-components and coordinates state transitions,
/// input handling, and view model generation.
#[derive(Debug, Clone)]
pub struct HexModeController {
    mode: HexMode,
    layout: HexLayout,
    cursor: HexCursor,
    edit_state: HexEditState,
    search_bridge: HexSearchBridge,
    viewport: HexViewportAdapter,
    modified_tracker: ModifiedByteTracker,
    config: HexConfig,
}

impl HexModeController {
    /// Create a new controller with default configuration.
    pub fn new(config: HexConfig) -> Self {
        let layout = HexLayout::new(0, config.bytes_per_row);
        let viewport = HexViewportAdapter::new(1, 20);
        Self {
            mode: HexMode::Off,
            layout,
            cursor: HexCursor::new(),
            edit_state: HexEditState::new(),
            search_bridge: HexSearchBridge::new(),
            viewport,
            modified_tracker: ModifiedByteTracker::new(),
            config,
        }
    }

    // --- Mode Control ---

    /// Activate hex mode. Maps current text cursor to hex position.
    ///
    /// Returns error if already active (with status message).
    pub fn activate(
        &mut self,
        text_cursor_byte_offset: u64,
        document_byte_length: u64,
    ) -> Result<(), HexError> {
        if self.mode.is_active() {
            return Err(HexError::AlreadyActive);
        }

        self.mode = HexMode::On;
        self.layout = HexLayout::new(document_byte_length, self.config.bytes_per_row);
        self.layout.set_digit_case(self.config.digit_case);
        self.cursor.set_from_text_position(text_cursor_byte_offset);
        self.viewport
            .recalculate(document_byte_length, self.config.bytes_per_row);

        // Scroll to make cursor visible
        let cursor_row = self.layout.row_for_offset(text_cursor_byte_offset);
        self.viewport.ensure_row_visible(cursor_row);

        Ok(())
    }

    /// Deactivate hex mode. Returns the byte offset for text cursor restore.
    ///
    /// Returns error if already inactive.
    pub fn deactivate(&mut self) -> Result<u64, HexError> {
        if !self.mode.is_active() {
            return Err(HexError::AlreadyInactive);
        }

        let byte_offset = self.cursor.to_text_position();
        self.mode = HexMode::Off;
        Ok(byte_offset)
    }

    /// Toggle hex mode.
    ///
    /// Returns `Ok(None)` if activated, `Ok(Some(byte_offset))` if deactivated.
    pub fn toggle(
        &mut self,
        text_cursor_byte_offset: u64,
        document_byte_length: u64,
    ) -> Result<Option<u64>, HexError> {
        if self.mode.is_active() {
            let offset = self.deactivate()?;
            Ok(Some(offset))
        } else {
            self.activate(text_cursor_byte_offset, document_byte_length)?;
            Ok(None)
        }
    }

    /// Whether hex mode is currently active.
    pub fn is_active(&self) -> bool {
        self.mode.is_active()
    }

    /// Current hex mode state.
    pub fn mode(&self) -> HexMode {
        self.mode
    }

    // --- Cursor Access ---

    /// Get a reference to the hex cursor.
    pub fn cursor(&self) -> &HexCursor {
        &self.cursor
    }

    /// Get mutable access to the hex cursor (for navigation).
    pub fn cursor_mut(&mut self) -> &mut HexCursor {
        &mut self.cursor
    }

    // --- Layout Access ---

    /// Get the current layout configuration.
    pub fn layout(&self) -> &HexLayout {
        &self.layout
    }

    /// Change bytes per row. Preserves cursor byte offset.
    pub fn set_bytes_per_row(&mut self, bpr: BytesPerRow, document_byte_length: u64) {
        self.config.bytes_per_row = bpr;
        self.layout.set_bytes_per_row(bpr, document_byte_length);
        self.viewport.recalculate(document_byte_length, bpr);

        // Ensure cursor is still visible
        let cursor_row = self.layout.row_for_offset(self.cursor.byte_offset());
        self.viewport.ensure_row_visible(cursor_row);
    }

    // --- Editing ---

    /// Process a key input in hex mode.
    ///
    /// Returns the edit action if a byte was modified, or None for
    /// navigation-only inputs.
    pub fn handle_input(
        &mut self,
        input: HexInput,
        document: &dyn ByteReader,
    ) -> Result<Option<HexEditAction>, HexError> {
        match input {
            HexInput::HexDigit(ch) => {
                let current_byte = document.byte_at(self.cursor.byte_offset()).unwrap_or(0);
                let action = self
                    .edit_state
                    .input_hex_digit(ch, &self.cursor, current_byte)?;
                self.modified_tracker.mark_modified(action.byte_offset);
                self.cursor.advance_after_hex_edit(document.byte_length());
                Ok(Some(action))
            }
            HexInput::AsciiChar(ch) => {
                let current_byte = document.byte_at(self.cursor.byte_offset()).unwrap_or(0);
                let action = self
                    .edit_state
                    .input_ascii_char(ch, &self.cursor, current_byte)?;
                self.modified_tracker.mark_modified(action.byte_offset);
                self.cursor.advance_after_ascii_edit(document.byte_length());
                Ok(Some(action))
            }
            HexInput::Arrow(direction) => {
                let doc_len = document.byte_length();
                match direction {
                    ArrowDirection::Up => self.cursor.move_up(&self.layout),
                    ArrowDirection::Down => self.cursor.move_down(&self.layout, doc_len),
                    ArrowDirection::Left => self.cursor.move_left(&self.layout),
                    ArrowDirection::Right => self.cursor.move_right(&self.layout, doc_len),
                }
                let cursor_row = self.layout.row_for_offset(self.cursor.byte_offset());
                self.viewport.ensure_row_visible(cursor_row);
                Ok(None)
            }
            HexInput::SwitchPane => {
                self.cursor.switch_pane();
                Ok(None)
            }
            HexInput::PageUp => {
                self.viewport.page_up();
                Ok(None)
            }
            HexInput::PageDown => {
                self.viewport.page_down();
                Ok(None)
            }
        }
    }

    // --- Viewport ---

    /// Get the viewport adapter.
    pub fn viewport(&self) -> &HexViewportAdapter {
        &self.viewport
    }

    /// Get mutable viewport adapter (for scroll operations).
    pub fn viewport_mut(&mut self) -> &mut HexViewportAdapter {
        &mut self.viewport
    }

    // --- View Model ---

    /// Build the view model for the currently visible rows.
    pub fn build_view_model(&self, document: &dyn ByteReader) -> HexViewModel {
        let doc_len = document.byte_length();
        let total_rows = self.layout.total_rows(doc_len);
        let top_row = self.viewport.top_row();
        let visible_count = self.viewport.visible_rows().min(total_rows - top_row);
        let bpr = self.layout.bytes_per_row().as_u64();

        let cursor_row = self.layout.row_for_offset(self.cursor.byte_offset());
        let cursor_byte_in_row = self.layout.byte_index_in_row(self.cursor.byte_offset());

        let mut visible_rows = Vec::with_capacity(visible_count as usize);

        for row_idx in top_row..(top_row + visible_count) {
            let row_start = row_idx * bpr;
            let row_end = (row_start + bpr).min(doc_len);
            let bytes = document.bytes_in_range(row_start, row_end);

            let offset_text = self.layout.format_offset(row_start);
            let hex_text = self.layout.format_hex_pane(&bytes);
            let ascii_text = self.layout.format_ascii_pane(&bytes);

            let byte_metadata: Vec<HexByteMetadata> = (0..bytes.len())
                .map(|i| {
                    let abs_offset = row_start + i as u64;
                    HexByteMetadata {
                        is_modified: self.modified_tracker.is_modified(abs_offset),
                        is_search_match: self.search_bridge.is_highlighted(abs_offset),
                        is_cursor: row_idx == cursor_row && i == cursor_byte_in_row,
                        is_selected: false,
                        is_field_boundary: false,
                    }
                })
                .collect();

            visible_rows.push(HexRow {
                row_index: row_idx,
                offset_text,
                hex_text,
                ascii_text,
                byte_metadata,
            });
        }

        HexViewModel {
            visible_rows,
            total_rows,
            top_row,
            cursor: HexCursorRenderState {
                row: cursor_row,
                byte_in_row: cursor_byte_in_row,
                nibble: self.cursor.nibble(),
                pane: self.cursor.active_pane(),
            },
            active_pane: self.cursor.active_pane(),
            mode: self.mode,
        }
    }

    // --- Modified Tracking ---

    /// Get the modified byte tracker.
    pub fn modified_tracker(&self) -> &ModifiedByteTracker {
        &self.modified_tracker
    }

    /// Get mutable modified byte tracker.
    pub fn modified_tracker_mut(&mut self) -> &mut ModifiedByteTracker {
        &mut self.modified_tracker
    }

    /// Notify that the document was saved.
    pub fn on_document_saved(&mut self) {
        self.modified_tracker.on_save();
    }

    // --- Session ---

    /// Capture current state for session persistence.
    pub fn capture_session(&self) -> HexSessionState {
        HexSessionState::capture(
            self.mode,
            self.config.bytes_per_row,
            self.cursor.byte_offset(),
            self.viewport.top_row(),
            self.cursor.active_pane(),
        )
    }

    /// Restore from a saved session.
    pub fn restore_session(
        &mut self,
        state: &HexSessionState,
        document_byte_length: u64,
    ) -> Result<(), HexError> {
        if state.was_active() {
            let bpr = state.saved_bytes_per_row().unwrap_or_default();
            self.config.bytes_per_row = bpr;
            self.activate(
                state
                    .cursor_offset
                    .min(document_byte_length.saturating_sub(1)),
                document_byte_length,
            )?;
            self.viewport_mut().set_top_row(state.viewport_top_row);

            // Restore active pane
            if state.active_pane == HexPane::Ascii && self.cursor.active_pane() == HexPane::Hex {
                self.cursor.switch_pane();
            }
        }
        Ok(())
    }

    // --- Search Bridge ---

    /// Get the search bridge for hex search integration.
    pub fn search_bridge(&self) -> &HexSearchBridge {
        &self.search_bridge
    }

    /// Get mutable search bridge.
    pub fn search_bridge_mut(&mut self) -> &mut HexSearchBridge {
        &mut self.search_bridge
    }

    // --- Config ---

    /// Get the current configuration.
    pub fn config(&self) -> &HexConfig {
        &self.config
    }

    /// Update the digit case setting, applying immediately to the layout.
    pub fn set_digit_case(&mut self, case: crate::types::HexDigitCase) {
        self.config.digit_case = case;
        self.layout.set_digit_case(case);
    }

    /// Set editing enabled or disabled.
    pub fn set_editing_enabled(&mut self, enabled: bool) {
        self.edit_state.set_editing_enabled(enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn default_controller() -> HexModeController {
        HexModeController::new(HexConfig::default())
    }

    fn test_document() -> VecByteReader {
        VecByteReader::new((0..256).map(|i| i as u8).collect())
    }

    // Validates: Requirement 1 AC 1
    #[test]
    fn activate_transitions_to_hex_mode() {
        let mut ctrl = default_controller();
        assert!(!ctrl.is_active());

        ctrl.activate(0, 256).unwrap();
        assert!(ctrl.is_active());
        assert_eq!(ctrl.mode(), HexMode::On);
    }

    // Validates: Requirement 1 AC 4
    #[test]
    fn activate_when_already_active_returns_error() {
        let mut ctrl = default_controller();
        ctrl.activate(0, 256).unwrap();

        let result = ctrl.activate(0, 256);
        assert_eq!(result.unwrap_err(), HexError::AlreadyActive);
    }

    // Validates: Requirement 1 AC 2
    #[test]
    fn deactivate_transitions_to_text_mode() {
        let mut ctrl = default_controller();
        ctrl.activate(42, 256).unwrap();

        let offset = ctrl.deactivate().unwrap();
        assert!(!ctrl.is_active());
        assert_eq!(offset, 42);
    }

    // Validates: Requirement 1 AC 5
    #[test]
    fn deactivate_when_already_inactive_returns_error() {
        let mut ctrl = default_controller();
        let result = ctrl.deactivate();
        assert_eq!(result.unwrap_err(), HexError::AlreadyInactive);
    }

    // Validates: Requirement 1 AC 3
    #[test]
    fn toggle_activates_when_off() {
        let mut ctrl = default_controller();
        let result = ctrl.toggle(0, 256).unwrap();
        assert_eq!(result, None); // Activated, no offset returned
        assert!(ctrl.is_active());
    }

    // Validates: Requirement 1 AC 3
    #[test]
    fn toggle_deactivates_when_on() {
        let mut ctrl = default_controller();
        ctrl.activate(50, 256).unwrap();
        let result = ctrl.toggle(50, 256).unwrap();
        assert_eq!(result, Some(50)); // Deactivated, offset returned
        assert!(!ctrl.is_active());
    }

    // Validates: Requirement 1 AC 9
    #[test]
    fn activate_maps_text_cursor_to_hex_position() {
        let mut ctrl = default_controller();
        ctrl.activate(33, 256).unwrap();
        assert_eq!(ctrl.cursor().byte_offset(), 33);
    }

    // Validates: Requirement 1 AC 10
    #[test]
    fn deactivate_returns_byte_offset_for_text_restore() {
        let mut ctrl = default_controller();
        ctrl.activate(100, 256).unwrap();
        let offset = ctrl.deactivate().unwrap();
        assert_eq!(offset, 100);
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn handle_hex_digit_input_produces_edit_action() {
        let mut ctrl = default_controller();
        let doc = test_document();
        ctrl.activate(0, 256).unwrap();

        let result = ctrl.handle_input(HexInput::HexDigit('A'), &doc).unwrap();
        assert!(result.is_some());
        let action = result.unwrap();
        assert_eq!(action.byte_offset, 0);
        assert_eq!(action.new_value, 0xA0); // A in high nibble, 0 in low
        assert_eq!(action.old_value, 0x00);
    }

    // Validates: Requirement 4 AC 3
    #[test]
    fn handle_ascii_char_input_produces_edit_action() {
        let mut ctrl = default_controller();
        let doc = test_document();
        ctrl.activate(5, 256).unwrap();
        ctrl.cursor_mut().switch_pane(); // ASCII pane

        let result = ctrl.handle_input(HexInput::AsciiChar('X'), &doc).unwrap();
        assert!(result.is_some());
        let action = result.unwrap();
        assert_eq!(action.byte_offset, 5);
        assert_eq!(action.new_value, 0x58); // 'X'
        assert_eq!(action.old_value, 0x05);
    }

    // Validates: Requirement 6 AC 1-2
    #[test]
    fn handle_arrow_navigation_moves_cursor() {
        let mut ctrl = default_controller();
        let doc = test_document();
        ctrl.activate(0, 256).unwrap();

        let result = ctrl
            .handle_input(HexInput::Arrow(ArrowDirection::Down), &doc)
            .unwrap();
        assert!(result.is_none()); // navigation only
        assert_eq!(ctrl.cursor().byte_offset(), 16);
    }

    // Validates: Requirement 6 AC 3
    #[test]
    fn handle_switch_pane_toggles_focus() {
        let mut ctrl = default_controller();
        let doc = test_document();
        ctrl.activate(0, 256).unwrap();

        ctrl.handle_input(HexInput::SwitchPane, &doc).unwrap();
        assert_eq!(ctrl.cursor().active_pane(), HexPane::Ascii);
    }

    // Validates: Requirement 8 AC 1-2
    #[test]
    fn editing_marks_byte_as_modified() {
        let mut ctrl = default_controller();
        let doc = test_document();
        ctrl.activate(5, 256).unwrap();

        ctrl.handle_input(HexInput::HexDigit('F'), &doc).unwrap();
        assert!(ctrl.modified_tracker().is_modified(5));
    }

    // Validates: Requirement 8 AC 3
    #[test]
    fn on_document_saved_clears_modified_indicators() {
        let mut ctrl = default_controller();
        let doc = test_document();
        ctrl.activate(5, 256).unwrap();
        ctrl.handle_input(HexInput::HexDigit('F'), &doc).unwrap();
        assert!(ctrl.modified_tracker().has_modifications());

        ctrl.on_document_saved();
        assert!(!ctrl.modified_tracker().has_modifications());
    }

    // Validates: Requirement 2
    #[test]
    fn build_view_model_produces_correct_row_data() {
        let mut ctrl = default_controller();
        let doc = test_document();
        ctrl.activate(0, 256).unwrap();

        let vm = ctrl.build_view_model(&doc);
        assert_eq!(vm.mode, HexMode::On);
        assert_eq!(vm.total_rows, 16);
        assert_eq!(vm.top_row, 0);
        assert!(!vm.visible_rows.is_empty());

        let first_row = &vm.visible_rows[0];
        assert_eq!(first_row.row_index, 0);
        assert_eq!(first_row.offset_text, "00000000");
    }

    // Validates: Requirement 3 AC 3
    #[test]
    fn set_bytes_per_row_recalculates_layout() {
        let mut ctrl = default_controller();
        ctrl.activate(100, 256).unwrap();

        ctrl.set_bytes_per_row(BytesPerRow::ThirtyTwo, 256);
        assert_eq!(ctrl.layout().bytes_per_row(), BytesPerRow::ThirtyTwo);
        // Cursor byte offset preserved
        assert_eq!(ctrl.cursor().byte_offset(), 100);
    }

    // Validates: Requirement 15
    #[test]
    fn capture_and_restore_session() {
        let mut ctrl = default_controller();
        ctrl.activate(50, 256).unwrap();
        ctrl.cursor_mut().switch_pane();

        let session = ctrl.capture_session();
        assert_eq!(session.mode, HexMode::On);
        assert_eq!(session.cursor_offset, 50);
        assert_eq!(session.active_pane, HexPane::Ascii);

        // Restore into a fresh controller
        let mut new_ctrl = default_controller();
        new_ctrl.restore_session(&session, 256).unwrap();
        assert!(new_ctrl.is_active());
        assert_eq!(new_ctrl.cursor().byte_offset(), 50);
        assert_eq!(new_ctrl.cursor().active_pane(), HexPane::Ascii);
    }
}
