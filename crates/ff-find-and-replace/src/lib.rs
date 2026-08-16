//! # ff-find-and-replace — Search and Replacement Engine
//!
//! This crate provides the search and replacement engine for FileForgeWorkbench.
//! It implements ISPF-style FIND/RFIND/CHANGE/RCHANGE commands with:
//!
//! - **Literal search** with forward/backward/first/last directions
//! - **Regular expression search** with NFA-based engine, group capture, backreferences
//! - **Hex byte search** for raw byte pattern matching
//! - **Unicode Full Case Folding** for case-insensitive search across all scripts
//! - **Whole-word and word-start matching** with multi-byte boundary detection
//! - **Scope filtering** (TAGGED, EXCLUDED, VISIBLE, NONTAGGED) and column bounds
//! - **Incremental search** (search-as-you-type) with debouncing and cancellation
//! - **Highlight-all-matches** mode for live viewport feedback
//! - **Session state persistence** for RFIND/RCHANGE repetition
//! - **Command framework integration** with undo-wrapped CHANGE transactions
//!
//! ## Architecture
//!
//! The crate is GUI-independent — it contains no rendering or UI framework
//! dependencies. Search results are emitted as byte ranges for consumption
//! by the text-decorations layer.
//!
//! ## Position in Architecture
//!
//! This is a Wave 5 (Command Engine) crate depending on:
//! - `ff-document-model` for buffer access via `CharacterIndexer` trait
//! - `ff-command` for command registration
//! - `ff-logging` for diagnostic output

// ─── Public Modules ─────────────────────────────────────────────────────────

pub mod case_folder;
pub mod direction;
pub mod engine;
pub mod error;
pub mod events;
pub mod hex_search;
pub mod highlight_all;
pub mod incremental;
pub mod indexer;
pub mod literal;
pub mod regex;
pub mod request;
pub mod result;
pub mod scope;
pub mod search_mode;
pub mod state;
pub mod substitution;
pub mod types;
pub mod word_boundary;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use case_folder::CaseFolder;
pub use direction::SearchDirection;
pub use engine::{FindEngine, FindEngineConfig};
pub use error::FindReplaceError;
pub use events::{FindEvent, FindEventListener};
pub use highlight_all::HighlightAllResult;
pub use indexer::{CharacterIndexer, CharacterIndexerMut};
pub use request::{ChangeRequest, FindRequest, WordMatchMode};
pub use result::{ChangeOutcome, ChangeResult, FindOutcome, FindResult};
pub use scope::{Bounds, ColumnRange, ScopeFilterProvider, ScopeModifier};
pub use search_mode::SearchMode;
pub use state::FindState;
pub use substitution::{SubstitutionEngine, SubstitutionTemplate};
pub use types::MatchRange;
