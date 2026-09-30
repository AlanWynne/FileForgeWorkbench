//! The SCRM data model: collections of screen captures.
//!
//! Validates: screen-snapshot-scrm Requirement 17.1, 17.2, 15.1, 15.2.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ff_screen_model::ScreenModel;
use serde::{Deserialize, Serialize};

/// A single captured screen within a collection.
///
/// The logical `ScreenModel` is the authoritative, copy/paste-able text content
/// (screen-snapshot-scrm Requirement 1). `image` is an OPTIONAL secondary
/// artefact only (Requirement 1.3); it is never required.
///
/// Validates: Requirement 17.2, 1.3, 15.1, 15.2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenCapture {
    /// Stable unique id within the collection.
    pub capture_id: String,
    /// 1-based sequence number within the collection.
    pub sequence_number: u32,
    /// When the capture was taken.
    pub timestamp: DateTime<Utc>,
    /// The logical screen name (e.g. the Context/panel name), if known.
    pub screen_name: Option<String>,
    /// The program/command that produced the screen, if known.
    pub program_name: Option<String>,
    /// The DIDL dialog-state identifier, if a dialog state was available.
    /// Validates: Requirement 15.1, 15.2.
    pub dialog_state: Option<String>,
    /// The authoritative logical screen content (text-first).
    pub screen: ScreenModel,
    /// Optional secondary bitmap artefact (PNG bytes); never authoritative.
    pub image: Option<Vec<u8>>,
    /// Free-form notes attached to this capture.
    pub notes: String,
    /// Arbitrary metadata key/value pairs.
    pub metadata: BTreeMap<String, String>,
}

impl ScreenCapture {
    /// Construct a capture around a screen model with the given id/sequence.
    pub fn new(
        capture_id: impl Into<String>,
        sequence_number: u32,
        timestamp: DateTime<Utc>,
        screen: ScreenModel,
    ) -> Self {
        Self {
            capture_id: capture_id.into(),
            sequence_number,
            timestamp,
            screen_name: None,
            program_name: None,
            dialog_state: None,
            screen,
            image: None,
            notes: String::new(),
            metadata: BTreeMap::new(),
        }
    }
}

/// A named, ordered, persisted group of screen captures.
///
/// Validates: Requirement 17.1, 7.1-7.4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenCollection {
    /// Stable unique id.
    pub collection_id: String,
    /// Human-readable name.
    pub name: String,
    /// Optional descriptive notes.
    pub description: String,
    /// Who created it.
    pub created_by: String,
    /// When it was created.
    pub created_timestamp: DateTime<Utc>,
    /// The session that created it, if known.
    pub session_id: Option<String>,
    /// The captures, in sequence order.
    pub screens: Vec<ScreenCapture>,
}

impl ScreenCollection {
    /// Create a new, empty collection.
    pub fn new(
        collection_id: impl Into<String>,
        name: impl Into<String>,
        created_by: impl Into<String>,
        created_timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            collection_id: collection_id.into(),
            name: name.into(),
            description: String::new(),
            created_by: created_by.into(),
            created_timestamp,
            session_id: None,
            screens: Vec::new(),
        }
    }

    /// Append a screen model as the next capture, assigning the next sequence
    /// number (1-based) and the given id/timestamp. Returns the sequence number.
    ///
    /// Validates: Requirement 8.2, 8.3.
    pub fn append_capture(
        &mut self,
        capture_id: impl Into<String>,
        timestamp: DateTime<Utc>,
        screen: ScreenModel,
    ) -> u32 {
        let seq = self.screens.len() as u32 + 1;
        self.screens
            .push(ScreenCapture::new(capture_id, seq, timestamp, screen));
        seq
    }

    /// Number of captures.
    pub fn len(&self) -> usize {
        self.screens.len()
    }

    /// Whether the collection has no captures.
    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn ts() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("valid timestamp")
    }

    // Validates: Requirement 8.2, 8.3 -- append assigns sequential numbers.
    #[test]
    fn append_capture_assigns_sequential_numbers() {
        let mut c = ScreenCollection::new("c1", "Test", "alan", ts());
        assert!(c.is_empty());
        let s1 = c.append_capture("a", ts(), ScreenModel::new("One"));
        let s2 = c.append_capture("b", ts(), ScreenModel::new("Two"));
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
        assert_eq!(c.len(), 2);
        assert_eq!(c.screens[1].sequence_number, 2);
    }

    // Validates: Requirement 15.1, 15.2 -- a capture can carry a DIDL state id.
    #[test]
    fn capture_carries_dialog_state() {
        let mut cap = ScreenCapture::new("a", 1, ts(), ScreenModel::new("S"));
        cap.dialog_state = Some("PANEL.EDIT".to_string());
        assert_eq!(cap.dialog_state.as_deref(), Some("PANEL.EDIT"));
        // image is optional and defaults to None (Requirement 1.3).
        assert!(cap.image.is_none());
    }
}
