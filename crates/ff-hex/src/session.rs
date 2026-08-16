//! Hex mode session state persistence.
//!
//! Serialisable per-file hex mode session state, stored in the
//! session history system for restore on reopen.

use crate::types::{BytesPerRow, HexMode, HexPane};

/// Serialisable per-file hex mode session state.
///
/// Stored in the session history entry for each file to enable
/// restoring hex mode state when a file is reopened.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HexSessionState {
    /// Whether hex mode was active when the file was last closed.
    pub mode: HexMode,
    /// Bytes per row setting.
    pub bytes_per_row: u32,
    /// Cursor byte offset.
    pub cursor_offset: u64,
    /// Top visible row (viewport).
    pub viewport_top_row: u64,
    /// Which pane had focus.
    pub active_pane: HexPane,
}

impl Default for HexSessionState {
    fn default() -> Self {
        Self {
            mode: HexMode::Off,
            bytes_per_row: 16,
            cursor_offset: 0,
            viewport_top_row: 0,
            active_pane: HexPane::Hex,
        }
    }
}

impl HexSessionState {
    /// Create a session state capturing the current hex mode configuration.
    pub fn capture(
        mode: HexMode,
        bytes_per_row: BytesPerRow,
        cursor_offset: u64,
        viewport_top_row: u64,
        active_pane: HexPane,
    ) -> Self {
        Self {
            mode,
            bytes_per_row: bytes_per_row.as_usize() as u32,
            cursor_offset,
            viewport_top_row,
            active_pane,
        }
    }

    /// Whether hex mode was previously active for this file.
    pub fn was_active(&self) -> bool {
        self.mode.is_active()
    }

    /// Get the saved bytes per row value.
    pub fn saved_bytes_per_row(&self) -> Option<BytesPerRow> {
        BytesPerRow::from_value(self.bytes_per_row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 15 AC 3
    #[test]
    fn session_state_captures_all_fields() {
        let state =
            HexSessionState::capture(HexMode::On, BytesPerRow::ThirtyTwo, 1024, 5, HexPane::Ascii);

        assert_eq!(state.mode, HexMode::On);
        assert_eq!(state.bytes_per_row, 32);
        assert_eq!(state.cursor_offset, 1024);
        assert_eq!(state.viewport_top_row, 5);
        assert_eq!(state.active_pane, HexPane::Ascii);
    }

    // Validates: Requirement 15 AC 1-2
    #[test]
    fn session_state_serialization_round_trip() {
        let state =
            HexSessionState::capture(HexMode::On, BytesPerRow::Sixteen, 512, 3, HexPane::Hex);

        let json = serde_json::to_string(&state).unwrap();
        let restored: HexSessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, restored);
    }

    // Validates: Requirement 15 AC 2
    #[test]
    fn was_active_reflects_mode_state() {
        let active =
            HexSessionState::capture(HexMode::On, BytesPerRow::Sixteen, 0, 0, HexPane::Hex);
        assert!(active.was_active());

        let inactive = HexSessionState::default();
        assert!(!inactive.was_active());
    }

    // Validates: Requirement 15 AC 3
    #[test]
    fn saved_bytes_per_row_returns_valid_value() {
        let state =
            HexSessionState::capture(HexMode::On, BytesPerRow::SixtyFour, 0, 0, HexPane::Hex);
        assert_eq!(state.saved_bytes_per_row(), Some(BytesPerRow::SixtyFour));
    }
}
