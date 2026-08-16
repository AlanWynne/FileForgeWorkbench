//! Session persistence for zoom state.
//!
//! [`ZoomSessionEntry`] captures the zoom offset for a document at save time
//! and restores it on session reload, clamping to the current configuration
//! range if limits have changed between sessions.

use serde::{Deserialize, Serialize};

use crate::config::ZoomConfig;
use crate::state::ZoomState;

/// Serialisable zoom state for session persistence.
///
/// Stored alongside cursor position and scroll state per document URI
/// in the session store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoomSessionEntry {
    /// The document's resource URI.
    pub resource_uri: String,
    /// The zoom offset at time of snapshot.
    pub zoom_offset: i32,
}

impl ZoomSessionEntry {
    /// Capture the current zoom state for a document.
    ///
    /// # Arguments
    ///
    /// * `uri` — The document's resource URI.
    /// * `state` — The current zoom state to persist.
    pub fn from_state(uri: &str, state: &ZoomState) -> Self {
        Self {
            resource_uri: uri.to_string(),
            zoom_offset: state.offset().value(),
        }
    }

    /// Restore a [`ZoomState`] from this persisted entry.
    ///
    /// The offset is clamped to the current configuration range, handling
    /// the case where configuration limits changed between sessions.
    pub fn restore(&self, config: &ZoomConfig) -> ZoomState {
        ZoomState::from_persisted(self.zoom_offset, config)
    }
}

/// Serialise a batch of zoom session entries to JSON bytes.
pub fn persist_all(entries: &[ZoomSessionEntry]) -> Vec<u8> {
    serde_json::to_vec(entries).unwrap_or_default()
}

/// Deserialise a batch of zoom session entries from JSON bytes.
///
/// Returns an empty vector if parsing fails.
pub fn restore_all(data: &[u8]) -> Vec<ZoomSessionEntry> {
    serde_json::from_slice(data).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ZoomConfig {
        ZoomConfig::default()
    }

    // Validates: Requirement 6.1 — persist captures current offset
    #[test]
    fn from_state_captures_offset() {
        let config = default_config();
        let state = ZoomState::from_persisted(7, &config);
        let entry = ZoomSessionEntry::from_state("file:///test.rs", &state);
        assert_eq!(entry.resource_uri, "file:///test.rs");
        assert_eq!(entry.zoom_offset, 7);
    }

    // Validates: Requirement 6.2 — restore creates state with correct offset
    #[test]
    fn restore_creates_state_with_offset() {
        let config = default_config();
        let entry = ZoomSessionEntry {
            resource_uri: "file:///test.rs".to_string(),
            zoom_offset: 5,
        };
        let state = entry.restore(&config);
        assert_eq!(state.offset().value(), 5);
    }

    // Validates: Requirement 6.3 — restore clamps when config changed
    #[test]
    fn restore_clamps_when_config_narrower() {
        let config = ZoomConfig {
            max_offset: 30,
            ..Default::default()
        };
        let entry = ZoomSessionEntry {
            resource_uri: "file:///test.rs".to_string(),
            zoom_offset: 50,
        };
        let state = entry.restore(&config);
        assert_eq!(state.offset().value(), 30);
    }

    // Validates: Requirement 6.4 — serialisation round-trip
    #[test]
    fn persist_and_restore_round_trip() {
        let entries = vec![
            ZoomSessionEntry {
                resource_uri: "file:///a.rs".to_string(),
                zoom_offset: 3,
            },
            ZoomSessionEntry {
                resource_uri: "file:///b.rs".to_string(),
                zoom_offset: -2,
            },
        ];
        let data = persist_all(&entries);
        let restored = restore_all(&data);
        assert_eq!(restored, entries);
    }

    // Validates: Requirement 6.2 — restore_all handles invalid data
    #[test]
    fn restore_all_returns_empty_on_invalid_data() {
        let data = b"not valid json";
        let restored = restore_all(data);
        assert!(restored.is_empty());
    }

    // Validates: Requirement 6.2 — missing entry defaults to default_offset
    #[test]
    fn missing_entry_uses_default_offset() {
        let config = ZoomConfig {
            default_offset: 2,
            ..Default::default()
        };
        // When no persisted entry exists, the caller creates a new state
        let state = ZoomState::new(&config);
        assert_eq!(state.offset().value(), 2);
    }
}
