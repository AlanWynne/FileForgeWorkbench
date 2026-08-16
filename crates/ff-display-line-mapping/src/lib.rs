//! # ff-display-line-mapping — Display Line Mapping for FileForgeWorkbench
//!
//! This crate maintains the bidirectional relationship between document lines
//! (logical lines in the buffer) and display lines (visual lines rendered in
//! the viewport). It supports:
//!
//! - **Line exclusion**: hiding ranges of document lines (ISPF EXCLUDE/SHOW)
//! - **Code folding**: collapsing regions behind fold headers
//! - **Word wrap**: mapping one document line to multiple display sub-lines
//! - **Lazy allocation**: zero heap overhead when no folding/exclusion/wrapping is active
//! - **Large documents**: 64-bit indexing for documents exceeding 2^31 lines
//! - **O(log n) performance**: Fenwick tree-based prefix-sum queries
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_display_line_mapping::{ContractionState, DisplayLineMapping, DocLine, DisplayLine, SubLine};
//!
//! let mut state = ContractionState::new(100);
//! assert!(state.is_one_to_one());
//!
//! // Hide lines 10-19
//! state.set_visible(DocLine(10), DocLine(19), false);
//! assert!(!state.is_one_to_one());
//! assert_eq!(state.lines_displayed(), 90);
//!
//! // Set word wrap height for line 5
//! state.set_height(DocLine(5), 3);
//! assert_eq!(state.lines_displayed(), 92); // gained 2 extra display lines
//!
//! // Reset to one-to-one mode
//! state.show_all();
//! assert!(state.is_one_to_one());
//! assert_eq!(state.lines_displayed(), 100);
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Core newtype definitions for document lines, display lines, and sub-lines.
pub mod types;

/// Error types for the display-line-mapping crate.
pub mod error;

/// The `DisplayLineMapping` trait defining the full public API.
pub mod traits;

/// The `ContractionState` implementation.
pub mod contraction_state;

/// Partitioning data structures (Fenwick tree).
pub mod partitioning;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use contraction_state::ContractionState;
pub use error::DisplayMappingError;
pub use traits::DisplayLineMapping;
pub use types::{
    DisplayLine, DisplayLineCountChange, DocLine, DocPosition, ListenerHandle, SubLine,
};
