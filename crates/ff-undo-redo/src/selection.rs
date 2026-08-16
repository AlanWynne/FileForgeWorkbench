//! Selection state types for undo/redo cursor restoration.
//!
//! Each transaction captures the selection state before and after the operation
//! so that undo restores the before-state and redo restores the after-state.

use serde::{Deserialize, Serialize};

/// The cursor/selection state at a point in time.
///
/// Stored with each transaction for restoration on undo/redo.
/// Supports multi-caret editing scenarios.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionState {
    /// All active caret positions (supports multi-caret).
    pub carets: Vec<CaretPosition>,
    /// The selection type at this point.
    pub selection_type: SelectionType,
}

impl SelectionState {
    /// Creates a simple single-caret state with no selection.
    pub fn single_caret(position: u64) -> Self {
        Self {
            carets: vec![CaretPosition {
                position,
                anchor: position,
                virtual_space: 0,
                anchor_virtual_space: 0,
            }],
            selection_type: SelectionType::None,
        }
    }

    /// Creates a single-caret state with a stream selection.
    pub fn with_selection(position: u64, anchor: u64) -> Self {
        Self {
            carets: vec![CaretPosition {
                position,
                anchor,
                virtual_space: 0,
                anchor_virtual_space: 0,
            }],
            selection_type: if position == anchor {
                SelectionType::None
            } else {
                SelectionType::Stream
            },
        }
    }
}

/// A single caret position with optional anchor for selection range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaretPosition {
    /// Caret byte position in document.
    pub position: u64,
    /// Anchor byte position (for selection range; equals position if no selection).
    pub anchor: u64,
    /// Virtual space offset beyond line end.
    pub virtual_space: u32,
    /// Anchor virtual space.
    pub anchor_virtual_space: u32,
}

/// Type of selection active at capture time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionType {
    /// Character/stream selection.
    Stream,
    /// Rectangular/column selection.
    Rectangular,
    /// Line-based selection.
    Line,
    /// No active selection (just a caret).
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_caret_creates_no_selection() {
        let state = SelectionState::single_caret(42);
        assert_eq!(state.carets.len(), 1);
        assert_eq!(state.carets[0].position, 42);
        assert_eq!(state.carets[0].anchor, 42);
        assert_eq!(state.selection_type, SelectionType::None);
    }

    #[test]
    fn with_selection_creates_stream_selection() {
        let state = SelectionState::with_selection(10, 5);
        assert_eq!(state.carets[0].position, 10);
        assert_eq!(state.carets[0].anchor, 5);
        assert_eq!(state.selection_type, SelectionType::Stream);
    }

    #[test]
    fn with_selection_same_position_creates_no_selection() {
        let state = SelectionState::with_selection(10, 10);
        assert_eq!(state.selection_type, SelectionType::None);
    }
}
