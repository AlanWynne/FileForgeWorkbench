//! Find/replace event system for plugins and UI.
//!
//! Addresses: Requirement 17 AC 7

use crate::result::FindResult;
use crate::search_mode::SearchMode;

/// Events emitted by the FindEngine for plugins and UI.
///
/// Addresses: Requirement 17 AC 7
#[derive(Debug, Clone)]
pub enum FindEvent {
    /// A find operation has started.
    FindStarted { term: String, mode: SearchMode },
    /// A match was found.
    MatchFound { result: FindResult },
    /// A find operation completed.
    FindCompleted {
        term: String,
        total_matches: u64,
        elapsed_ms: u64,
    },
    /// A replace operation completed.
    ReplaceCompleted {
        term: String,
        replacement_count: u64,
        elapsed_ms: u64,
    },
    /// Progress update during long operations.
    Progress {
        matches_so_far: u64,
        lines_scanned: u64,
    },
}

/// Trait for receiving find/replace events.
pub trait FindEventListener: Send + Sync {
    /// Called when a find/replace event occurs.
    fn on_event(&self, event: &FindEvent);
}
