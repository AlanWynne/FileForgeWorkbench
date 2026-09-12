//! FindEngine: top-level search orchestrator.
//!
//! Coordinates scope filtering, delegates to literal/regex/hex searchers,
//! manages state transitions, and emits events.
//!
//! Addresses: Requirements 1-20

use crate::case_folder::CaseFolder;
use crate::direction::SearchDirection;
use crate::error::FindReplaceError;
use crate::highlight_all::HighlightAllResult;
use crate::indexer::{CharacterIndexer, CharacterIndexerMut};
use crate::regex::RegexEngine;
use crate::request::{ChangeRequest, FindRequest, WordMatchMode};
use crate::result::{ChangeOutcome, ChangeResult, FindOutcome};
use crate::scope::{Bounds, ScopeFilterProvider, ScopeModifier};
use crate::search_mode::SearchMode;
use crate::state::FindState;
use crate::types::{BytePosition, MatchRange};

/// Configuration for the FindEngine.
#[derive(Debug, Clone)]
pub struct FindEngineConfig {
    /// Whether BOUNDS affect FIND (default: true).
    pub bounds_affect_find: bool,
    /// Maximum matches for highlight-all (default: 1000).
    pub highlight_all_max: u64,
    /// Incremental search time budget in ms (default: 50).
    pub incremental_time_budget_ms: u64,
    /// Regex match-attempt limit per position (default: 10_000).
    pub regex_step_limit: u64,
    /// Search history capacity (default: 20).
    pub history_capacity: usize,
    /// Progress report interval (matches between events).
    pub progress_interval: u64,
}

impl Default for FindEngineConfig {
    fn default() -> Self {
        Self {
            bounds_affect_find: true,
            highlight_all_max: 1000,
            incremental_time_budget_ms: 50,
            regex_step_limit: 10_000,
            history_capacity: 20,
            progress_interval: 100,
        }
    }
}

/// The top-level search and replacement engine.
///
/// Addresses: Requirements 1-20
pub struct FindEngine {
    pub(super) config: FindEngineConfig,
    pub(super) state: FindState,
    pub(super) case_folder: CaseFolder,
    pub(super) regex_engine: RegexEngine,
}

impl FindEngine {
    /// Create a new FindEngine with default configuration.
    pub fn new() -> Self {
        Self::with_config(FindEngineConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: FindEngineConfig) -> Self {
        let state = FindState::new(config.history_capacity);
        let regex_engine = RegexEngine::with_limits(10_000, config.regex_step_limit);
        Self {
            config,
            state,
            case_folder: CaseFolder::new(),
            regex_engine,
        }
    }

    /// Get the current FindState.
    pub fn state(&self) -> &FindState {
        &self.state
    }

    /// Get mutable access to FindState (for RESET operations).
    pub fn state_mut(&mut self) -> &mut FindState {
        &mut self.state
    }

    /// Get the case folder reference.
    pub fn case_folder(&self) -> &CaseFolder {
        &self.case_folder
    }

    /// Execute a FIND operation.
    ///
    /// Addresses: Requirements 1-4
    pub fn find(
        &mut self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<FindOutcome, FindReplaceError> {
        // Empty document short-circuit
        if indexer.length() == 0 {
            return Ok(FindOutcome::NotFound {
                term: request.term.clone(),
            });
        }

        // Empty term handling
        if request.term.is_empty() {
            return Err(FindReplaceError::NoSearchTerm);
        }

        let outcome = self.execute_find(request, indexer, scope_filter, bounds)?;

        // Record state on success
        if let Some(result) = outcome.first_result() {
            self.state.record_find(request, result.match_range.end);
        }

        Ok(outcome)
    }

    /// Execute an RFIND (repeat previous find).
    ///
    /// Addresses: Requirement 5
    pub fn rfind(
        &mut self,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<FindOutcome, FindReplaceError> {
        let last_find = self
            .state
            .last_find
            .clone()
            .ok_or(FindReplaceError::NoPreviousFind)?;

        // Normalise direction: FIRST->NEXT, LAST->PREV
        let mut request = last_find;
        request.direction = request.direction.normalise_for_repeat();

        // Advance cursor from last match position
        if let Some(pos) = self.state.last_match_position {
            request.cursor_position = pos;
        }

        self.find(&request, indexer, scope_filter, bounds)
    }

    /// Execute a CHANGE operation.
    ///
    /// Addresses: Requirements 6-8
    pub fn change(
        &mut self,
        request: &ChangeRequest,
        indexer: &mut dyn CharacterIndexerMut,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<ChangeOutcome, FindReplaceError> {
        // Read-only check
        if indexer.is_read_only() {
            return Ok(ChangeOutcome::ReadOnly);
        }

        // Empty term check
        if request.find.term.is_empty() {
            return Err(FindReplaceError::NoSearchTerm);
        }

        // Empty doc check
        if indexer.length() == 0 {
            return Ok(ChangeOutcome::NotFound {
                term: request.find.term.clone(),
            });
        }

        // For non-ALL, find single match and replace
        let outcome = self.execute_find(&request.find, indexer, scope_filter, bounds)?;
        match outcome {
            FindOutcome::NotFound { term } => Ok(ChangeOutcome::NotFound { term }),
            FindOutcome::Found(result) => {
                let replacement_bytes = self.compute_replacement(request, &result, indexer)?;
                indexer.replace_range(
                    result.match_range.start,
                    result.match_range.end,
                    &replacement_bytes,
                )?;
                let final_pos =
                    BytePosition(result.match_range.start.0 + replacement_bytes.len() as u64);
                let final_line = indexer.line_from_position(final_pos);

                self.state.record_change(request, final_pos);
                Ok(ChangeOutcome::Changed(ChangeResult {
                    replacement_count: 1,
                    final_position: final_pos,
                    final_line,
                }))
            }
            FindOutcome::FoundAll { .. } => {
                // CHANGE ALL: replace all matches
                let result = self.execute_change_all(request, indexer, scope_filter, bounds)?;
                Ok(result)
            }
        }
    }

    /// Execute a CHANGE ALL operation (replace all occurrences).
    ///
    /// Addresses: Requirement 6 AC 2, Requirement 7 AC 8
    pub fn change_all(
        &mut self,
        request: &ChangeRequest,
        indexer: &mut dyn CharacterIndexerMut,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<ChangeOutcome, FindReplaceError> {
        if indexer.is_read_only() {
            return Ok(ChangeOutcome::ReadOnly);
        }
        if request.find.term.is_empty() {
            return Err(FindReplaceError::NoSearchTerm);
        }
        if indexer.length() == 0 {
            return Ok(ChangeOutcome::NotFound {
                term: request.find.term.clone(),
            });
        }
        self.execute_change_all(request, indexer, scope_filter, bounds)
    }

    /// Execute an RCHANGE (repeat previous change).
    ///
    /// Addresses: Requirement 9
    pub fn rchange(
        &mut self,
        indexer: &mut dyn CharacterIndexerMut,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<ChangeOutcome, FindReplaceError> {
        let last_change = self
            .state
            .last_change
            .clone()
            .ok_or(FindReplaceError::NoPreviousChange)?;

        // Normalise direction
        let mut request = last_change;
        request.find.direction = request.find.direction.normalise_for_repeat();

        // Advance cursor
        if let Some(pos) = self.state.last_match_position {
            request.find.cursor_position = pos;
        }

        self.change(&request, indexer, scope_filter, bounds)
    }

    /// Execute a find for EXCLUDE/SHOW delegation (does NOT update FindState).
    ///
    /// Addresses: Requirement 16 AC 1-4
    pub fn find_for_filter(
        &self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<FindOutcome, FindReplaceError> {
        if request.term.is_empty() {
            return Err(FindReplaceError::NoSearchTerm);
        }
        if indexer.length() == 0 {
            return Ok(FindOutcome::NotFound {
                term: request.term.clone(),
            });
        }
        // Execute without state update -- use a temporary engine state
        execute_find_stateless(
            request,
            indexer,
            scope_filter,
            bounds,
            &self.config,
            &self.case_folder,
        )
    }

    /// Compute all matches within a viewport range for highlight-all.
    ///
    /// Addresses: Requirement 15
    pub fn highlight_all(
        &self,
        term: &str,
        mode: SearchMode,
        case_sensitive: bool,
        viewport_start: BytePosition,
        viewport_end: BytePosition,
        indexer: &dyn CharacterIndexer,
    ) -> Result<HighlightAllResult, FindReplaceError> {
        if term.is_empty() {
            return Ok(HighlightAllResult::empty());
        }

        let request = FindRequest {
            term: term.to_string(),
            mode,
            direction: SearchDirection::Next,
            scope: ScopeModifier::All,
            case_sensitive,
            word_match: WordMatchMode::None,
            column_range: None,
            cursor_position: viewport_start,
        };

        let matches = self.find_all_in_range(&request, indexer, viewport_start, viewport_end)?;
        let match_ranges: Vec<MatchRange> = matches.iter().map(|r| r.match_range).collect();
        Ok(HighlightAllResult::from_matches(
            match_ranges,
            self.config.highlight_all_max,
        ))
    }
}

impl Default for FindEngine {
    fn default() -> Self {
        Self::new()
    }
}

mod finders;
mod stateless;

use stateless::execute_find_stateless;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::{MutableSliceIndexer, SliceIndexer};
    use crate::scope::AllLinesFilter;

    #[test]
    fn find_literal_forward_from_start() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello world hello");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("hello");
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        match outcome {
            FindOutcome::Found(r) => {
                assert_eq!(r.match_range.start, BytePosition(0));
                assert_eq!(r.match_range.end, BytePosition(5));
            }
            _ => panic!("Expected Found"),
        }
    }

    #[test]
    fn find_returns_not_found_for_missing_term() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello world");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("xyz");
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        assert!(matches!(outcome, FindOutcome::NotFound { .. }));
    }

    #[test]
    fn find_empty_term_returns_error() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("");
        let err = engine.find(&req, &indexer, &filter, None).unwrap_err();
        assert!(matches!(err, FindReplaceError::NoSearchTerm));
    }

    #[test]
    fn find_empty_document_returns_not_found() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("hello");
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        assert!(matches!(outcome, FindOutcome::NotFound { .. }));
    }

    #[test]
    fn find_case_insensitive_matches() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("Hello World");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("hello").with_case_sensitive(false);
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        assert!(outcome.is_found());
    }

    #[test]
    fn rfind_without_previous_find_returns_error() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello");
        let filter = AllLinesFilter;
        let err = engine.rfind(&indexer, &filter, None).unwrap_err();
        assert!(matches!(err, FindReplaceError::NoPreviousFind));
    }

    #[test]
    fn rfind_repeats_last_find_advancing() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("abc abc abc");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("abc");
        engine.find(&req, &indexer, &filter, None).unwrap();
        let outcome = engine.rfind(&indexer, &filter, None).unwrap();
        match outcome {
            FindOutcome::Found(r) => {
                assert_eq!(r.match_range.start, BytePosition(4));
            }
            _ => panic!("Expected Found"),
        }
    }

    #[test]
    fn change_replaces_first_occurrence() {
        let mut engine = FindEngine::new();
        let mut indexer = MutableSliceIndexer::new("hello world");
        let filter = AllLinesFilter;
        let req = ChangeRequest::new(FindRequest::literal("hello"), "goodbye");
        let outcome = engine.change(&req, &mut indexer, &filter, None).unwrap();
        assert!(outcome.is_changed());
        assert_eq!(indexer.content_str(), Some("goodbye world"));
    }

    #[test]
    fn change_read_only_returns_read_only_outcome() {
        let mut engine = FindEngine::new();
        let mut indexer = MutableSliceIndexer::read_only("hello");
        let filter = AllLinesFilter;
        let req = ChangeRequest::new(FindRequest::literal("hello"), "bye");
        let outcome = engine.change(&req, &mut indexer, &filter, None).unwrap();
        assert!(matches!(outcome, ChangeOutcome::ReadOnly));
    }

    #[test]
    fn change_all_replaces_all_occurrences() {
        let mut engine = FindEngine::new();
        let mut indexer = MutableSliceIndexer::new("aaa bbb aaa");
        let filter = AllLinesFilter;
        let find_req = FindRequest::literal("aaa");
        let req = ChangeRequest::new(find_req, "x");
        let outcome = engine
            .change_all(&req, &mut indexer, &filter, None)
            .unwrap();
        match outcome {
            ChangeOutcome::Changed(r) => {
                assert_eq!(r.replacement_count, 2);
            }
            _ => panic!("Expected Changed"),
        }
        assert_eq!(indexer.content_str(), Some("x bbb x"));
    }

    #[test]
    fn rchange_without_previous_change_returns_error() {
        let mut engine = FindEngine::new();
        let mut indexer = MutableSliceIndexer::new("hello");
        let filter = AllLinesFilter;
        let err = engine.rchange(&mut indexer, &filter, None).unwrap_err();
        assert!(matches!(err, FindReplaceError::NoPreviousChange));
    }

    #[test]
    fn find_hex_pattern_matches_raw_bytes() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::new(b"hello\x4A\x5Bworld");
        let filter = AllLinesFilter;
        let req = FindRequest::hex("4A5B");
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        match outcome {
            FindOutcome::Found(r) => {
                assert_eq!(r.match_range.start, BytePosition(5));
                assert_eq!(r.match_range.end, BytePosition(7));
            }
            _ => panic!("Expected Found"),
        }
    }

    #[test]
    fn find_regex_simple_pattern() {
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello world 123");
        let filter = AllLinesFilter;
        let req = FindRequest::regex("\\d+");
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        match outcome {
            FindOutcome::Found(r) => {
                assert_eq!(r.match_range.start, BytePosition(12));
                assert_eq!(r.match_range.end, BytePosition(15));
            }
            _ => panic!("Expected Found"),
        }
    }
}
