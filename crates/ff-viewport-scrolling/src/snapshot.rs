//! Viewport state persistence (snapshot and restore).
//!
//! Provides a serialisable snapshot of viewport state for session management.
//! On restore, all values are clamped to current document boundaries.

use serde::{Deserialize, Serialize};

/// Serialisable viewport state for session persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportSnapshot {
    /// Top line at time of snapshot.
    pub top_line: u64,
    /// Cursor line at time of snapshot.
    pub cursor_line: u64,
    /// Cursor column at time of snapshot.
    pub cursor_column: u64,
    /// Horizontal offset at time of snapshot.
    pub horizontal_offset: u64,
    /// Column affinity at time of snapshot.
    pub column_affinity: u64,
}
