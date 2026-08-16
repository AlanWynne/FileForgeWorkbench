//! Per-document session state for RFIND/RCHANGE repetition.
//!
//! Addresses: Requirements 5, 9, 13

use std::collections::VecDeque;

use crate::error::FindReplaceError;
use crate::request::{ChangeRequest, FindRequest};
use crate::types::BytePosition;

/// Per-document session state for RFIND/RCHANGE repetition.
///
/// Addresses: Requirements 5, 9, 13
#[derive(Debug, Clone)]
pub struct FindState {
    /// The most recent FindRequest (for RFIND).
    pub last_find: Option<FindRequest>,
    /// The most recent ChangeRequest (for RCHANGE).
    pub last_change: Option<ChangeRequest>,
    /// Position of the last match (for advancing RFIND/RCHANGE).
    pub last_match_position: Option<BytePosition>,
    /// Ring buffer of recent search terms.
    pub search_history: VecDeque<String>,
    /// Ring buffer of recent replacement texts.
    pub replacement_history: VecDeque<String>,
    /// Maximum history size.
    pub history_capacity: usize,
}

impl FindState {
    /// Create a new empty FindState with the given history capacity.
    pub fn new(history_capacity: usize) -> Self {
        Self {
            last_find: None,
            last_change: None,
            last_match_position: None,
            search_history: VecDeque::with_capacity(history_capacity),
            replacement_history: VecDeque::with_capacity(history_capacity),
            history_capacity,
        }
    }

    /// Record a new find request as the last search.
    ///
    /// Addresses: Requirement 5 AC 3
    pub fn record_find(&mut self, request: &FindRequest, match_pos: BytePosition) {
        // Add to search history if term is new/different from most recent
        if self.search_history.front() != Some(&request.term) {
            if self.search_history.len() >= self.history_capacity {
                self.search_history.pop_back();
            }
            self.search_history.push_front(request.term.clone());
        }
        self.last_find = Some(request.clone());
        self.last_match_position = Some(match_pos);
    }

    /// Record a new change request.
    ///
    /// Addresses: Requirement 9 AC 3
    pub fn record_change(&mut self, request: &ChangeRequest, final_pos: BytePosition) {
        // Record find part
        self.record_find(&request.find, final_pos);
        // Add to replacement history
        if self.replacement_history.front() != Some(&request.replacement) {
            if self.replacement_history.len() >= self.history_capacity {
                self.replacement_history.pop_back();
            }
            self.replacement_history
                .push_front(request.replacement.clone());
        }
        self.last_change = Some(request.clone());
    }

    /// Clear highlights and incremental state (RESET without ALL).
    ///
    /// Retains search/change parameters for RFIND/RCHANGE.
    ///
    /// Addresses: Requirement 13 AC 4
    pub fn reset(&mut self) {
        // Keep last_find, last_change, history — just clear transient state
        // In a full implementation this would also clear highlight decorations
    }

    /// Clear last-search/change params (RESET ALL).
    ///
    /// RFIND/RCHANGE will report "No previous FIND/CHANGE" after this.
    /// Retains history list.
    ///
    /// Addresses: Requirement 13 AC 5
    pub fn reset_all(&mut self) {
        self.last_find = None;
        self.last_change = None;
        self.last_match_position = None;
    }

    /// Serialise for session persistence.
    ///
    /// Addresses: Requirement 13 AC 7
    pub fn serialize(&self) -> Result<Vec<u8>, FindReplaceError> {
        serde_json::to_vec(&SerializableFindState::from(self))
            .map_err(|e| FindReplaceError::Serialization(e.to_string()))
    }

    /// Deserialise from session data.
    pub fn deserialize(data: &[u8]) -> Result<Self, FindReplaceError> {
        let s: SerializableFindState = serde_json::from_slice(data)
            .map_err(|e| FindReplaceError::Serialization(e.to_string()))?;
        Ok(s.into())
    }
}

/// Serialisable representation of FindState (history only).
#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableFindState {
    search_history: Vec<String>,
    replacement_history: Vec<String>,
    history_capacity: usize,
}

impl From<&FindState> for SerializableFindState {
    fn from(state: &FindState) -> Self {
        Self {
            search_history: state.search_history.iter().cloned().collect(),
            replacement_history: state.replacement_history.iter().cloned().collect(),
            history_capacity: state.history_capacity,
        }
    }
}

impl From<SerializableFindState> for FindState {
    fn from(s: SerializableFindState) -> Self {
        Self {
            last_find: None,
            last_change: None,
            last_match_position: None,
            search_history: s.search_history.into_iter().collect(),
            replacement_history: s.replacement_history.into_iter().collect(),
            history_capacity: s.history_capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_find_stores_request_and_position() {
        let mut state = FindState::new(20);
        let req = FindRequest::literal("hello");
        state.record_find(&req, BytePosition(5));
        assert!(state.last_find.is_some());
        assert_eq!(state.last_match_position, Some(BytePosition(5)));
    }

    #[test]
    fn record_find_adds_to_search_history() {
        let mut state = FindState::new(20);
        state.record_find(&FindRequest::literal("hello"), BytePosition(0));
        state.record_find(&FindRequest::literal("world"), BytePosition(5));
        assert_eq!(state.search_history.len(), 2);
        assert_eq!(state.search_history[0], "world");
        assert_eq!(state.search_history[1], "hello");
    }

    #[test]
    fn search_history_does_not_duplicate_consecutive_same_term() {
        let mut state = FindState::new(20);
        state.record_find(&FindRequest::literal("hello"), BytePosition(0));
        state.record_find(&FindRequest::literal("hello"), BytePosition(5));
        assert_eq!(state.search_history.len(), 1);
    }

    #[test]
    fn search_history_overflow_drops_oldest() {
        let mut state = FindState::new(3);
        state.record_find(&FindRequest::literal("a"), BytePosition(0));
        state.record_find(&FindRequest::literal("b"), BytePosition(0));
        state.record_find(&FindRequest::literal("c"), BytePosition(0));
        state.record_find(&FindRequest::literal("d"), BytePosition(0));
        assert_eq!(state.search_history.len(), 3);
        assert_eq!(state.search_history[0], "d");
        assert_eq!(state.search_history[2], "b");
    }

    #[test]
    fn reset_preserves_last_find_and_history() {
        let mut state = FindState::new(20);
        state.record_find(&FindRequest::literal("hello"), BytePosition(0));
        state.reset();
        assert!(state.last_find.is_some());
        assert!(!state.search_history.is_empty());
    }

    #[test]
    fn reset_all_clears_last_find_but_keeps_history() {
        let mut state = FindState::new(20);
        state.record_find(&FindRequest::literal("hello"), BytePosition(0));
        state.reset_all();
        assert!(state.last_find.is_none());
        assert!(state.last_change.is_none());
        assert!(!state.search_history.is_empty());
    }

    #[test]
    fn serialize_and_deserialize_roundtrip() {
        let mut state = FindState::new(20);
        state.record_find(&FindRequest::literal("hello"), BytePosition(0));
        state.record_find(&FindRequest::literal("world"), BytePosition(5));

        let data = state.serialize().unwrap();
        let restored = FindState::deserialize(&data).unwrap();

        assert_eq!(restored.search_history.len(), 2);
        assert_eq!(restored.history_capacity, 20);
        // last_find is not serialised (it's session-transient)
        assert!(restored.last_find.is_none());
    }
}
