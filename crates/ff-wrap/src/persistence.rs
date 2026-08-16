//! Session persistence for wrap state.
//!
//! Provides `WrapSnapshot` for serialising and restoring per-document
//! wrap settings across editor sessions.

use crate::boundary::{WrapBoundary, WrapColumn};
use crate::config::WrapConfig;
use crate::mode::WrapMode;
use crate::state::WrapState;

/// Serialisable wrap state for session persistence.
///
/// Stored alongside cursor position, scroll state, and zoom offset
/// per document URI.
///
/// Addresses: Requirement 11 (Wrap Persistence in Session State)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WrapSnapshot {
    /// The wrap mode at time of snapshot: "none", "word", or "character".
    pub mode: String,

    /// The wrap boundary: "viewport" or a column number as string.
    pub boundary: String,
}

impl WrapSnapshot {
    /// Create a snapshot from the current wrap state.
    ///
    /// Addresses: Requirement 11 AC 1
    pub fn from_state(state: &WrapState) -> Self {
        let mode = match state.mode() {
            WrapMode::None => "none".to_string(),
            WrapMode::Word => "word".to_string(),
            WrapMode::Character => "character".to_string(),
        };

        let boundary = match state.boundary() {
            WrapBoundary::Viewport => "viewport".to_string(),
            WrapBoundary::Column(col) => col.value().to_string(),
        };

        Self { mode, boundary }
    }

    /// Restore a WrapState from this snapshot, falling back to config defaults
    /// for unrecognised values.
    ///
    /// Addresses: Requirement 11 AC 2, AC 3
    pub fn restore(&self, config: &WrapConfig) -> WrapState {
        let mode = match self.mode.to_lowercase().as_str() {
            "none" => WrapMode::None,
            "word" => WrapMode::Word,
            "character" => WrapMode::Character,
            _ => WrapMode::None, // Fallback for unrecognised variants
        };

        let boundary = match self.boundary.to_lowercase().as_str() {
            "viewport" => WrapBoundary::Viewport,
            other => {
                if let Ok(n) = other.parse::<u16>() {
                    match WrapColumn::new(n) {
                        Some(col) => WrapBoundary::Column(col),
                        Option::None => config.wrap_column,
                    }
                } else {
                    config.wrap_column
                }
            }
        };

        let mut state = WrapState::from_config(config);
        state.set_mode(mode);
        state.set_boundary(boundary);
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_from_state_none_mode() {
        // Validates: Requirement 11.1
        let state = WrapState::from_config(&WrapConfig::default());
        let snapshot = WrapSnapshot::from_state(&state);
        assert_eq!(snapshot.mode, "none");
        assert_eq!(snapshot.boundary, "viewport");
    }

    #[test]
    fn snapshot_from_state_word_mode_with_column() {
        let config = WrapConfig {
            default_mode: WrapMode::Word,
            wrap_column: WrapBoundary::Column(WrapColumn::new(80).unwrap()),
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        let snapshot = WrapSnapshot::from_state(&state);
        assert_eq!(snapshot.mode, "word");
        assert_eq!(snapshot.boundary, "80");
    }

    #[test]
    fn restore_valid_snapshot() {
        // Validates: Requirement 11.2
        let snapshot = WrapSnapshot {
            mode: "word".to_string(),
            boundary: "viewport".to_string(),
        };
        let config = WrapConfig::default();
        let state = snapshot.restore(&config);
        assert_eq!(state.mode(), WrapMode::Word);
        assert_eq!(state.boundary(), WrapBoundary::Viewport);
    }

    #[test]
    fn restore_unrecognised_mode_falls_back_to_none() {
        // Validates: Requirement 11.3
        let snapshot = WrapSnapshot {
            mode: "turbo_wrap".to_string(),
            boundary: "viewport".to_string(),
        };
        let config = WrapConfig::default();
        let state = snapshot.restore(&config);
        assert_eq!(state.mode(), WrapMode::None);
    }

    #[test]
    fn restore_column_boundary() {
        // Validates: Requirement 11.5
        let snapshot = WrapSnapshot {
            mode: "character".to_string(),
            boundary: "120".to_string(),
        };
        let config = WrapConfig::default();
        let state = snapshot.restore(&config);
        assert_eq!(state.mode(), WrapMode::Character);
        assert_eq!(
            state.boundary(),
            WrapBoundary::Column(WrapColumn::new(120).unwrap())
        );
    }

    #[test]
    fn restore_invalid_boundary_uses_config_default() {
        let snapshot = WrapSnapshot {
            mode: "word".to_string(),
            boundary: "not_a_number".to_string(),
        };
        let config = WrapConfig::default();
        let state = snapshot.restore(&config);
        assert_eq!(state.boundary(), WrapBoundary::Viewport);
    }

    #[test]
    fn roundtrip_preserves_state() {
        // Validates: Requirement 11.1, 11.2
        let config = WrapConfig {
            default_mode: WrapMode::Character,
            wrap_column: WrapBoundary::Column(WrapColumn::new(100).unwrap()),
            ..WrapConfig::default()
        };
        let original_state = WrapState::from_config(&config);
        let snapshot = WrapSnapshot::from_state(&original_state);
        let restored_state = snapshot.restore(&config);
        assert_eq!(original_state.mode(), restored_state.mode());
        assert_eq!(original_state.boundary(), restored_state.boundary());
    }

    #[test]
    fn serialization_roundtrip() {
        let snapshot = WrapSnapshot {
            mode: "word".to_string(),
            boundary: "80".to_string(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: WrapSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, deserialized);
    }
}
