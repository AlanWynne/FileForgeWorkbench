//! # ff-global-search -- Cross-file Search and Replace Engine
//!
//! Provides global search across workspace roots or catalog paths,
//! streaming results incrementally via an async channel.
//! Reuses `ff-find-and-replace` for per-file matching logic.
//!
//! ## Key types
//!
//! - [`GlobalSearchRequest`] -- query, mode, scope, and options
//! - [`GlobalSearchEngine`] -- async search orchestrator
//! - [`GlobalReplaceEngine`] -- cross-file replace
//! - [`SearchEvent`] -- streamed result events
//! - [`SearchResult`] / [`FileMatches`] -- match data

pub mod error;
pub mod replace;
pub mod result;
pub mod search;

pub use error::GlobalSearchError;
pub use replace::{ConflictList, GlobalReplaceEngine, ReplaceSummary};
pub use result::{FileMatches, SearchEvent, SearchResult};
pub use search::{CancellationToken, GlobalSearchEngine, GlobalSearchRequest};
