//! Edit operation types for the undo/redo system.
//!
//! Each [`EditOperation`] represents a single atomic change to the document buffer.
//! Operations store position and length metadata; actual text content lives in the
//! [`ScrapStack`](crate::scrap::ScrapStack).

use serde::{Deserialize, Serialize};

/// A single atomic change to the document buffer.
///
/// Operations reference text data by offset/length into the `ScrapStack` rather than
/// storing text inline — this keeps the operation metadata compact and cache-friendly.
///
/// # Variants
///
/// - `Insert` — text was inserted at a byte position
/// - `Delete` — text was removed at a byte position
/// - `Replace` — text at a position was atomically replaced with different text
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditOperation {
    /// Insert text at a byte position.
    Insert {
        /// Byte position in the document where text was inserted.
        position: u64,
        /// Length of inserted text in bytes (actual data in ScrapStack).
        length: u32,
        /// Offset into the ScrapStack where the inserted text is stored.
        scrap_offset: u64,
    },
    /// Delete text at a byte position.
    Delete {
        /// Byte position in the document where deletion starts.
        position: u64,
        /// Length of deleted text in bytes (actual data in ScrapStack).
        length: u32,
        /// Offset into the ScrapStack where the deleted text is stored.
        scrap_offset: u64,
    },
    /// Replace text at a byte position (atomic delete + insert).
    Replace {
        /// Byte position in the document where replacement starts.
        position: u64,
        /// Length of the old (removed) text.
        old_length: u32,
        /// Length of the new (inserted) text.
        new_length: u32,
        /// Offset into ScrapStack for the old text.
        old_scrap_offset: u64,
        /// Offset into ScrapStack for the new text.
        new_scrap_offset: u64,
    },
}

impl EditOperation {
    /// Returns the byte position in the document where this operation occurs.
    pub fn position(&self) -> u64 {
        match self {
            Self::Insert { position, .. }
            | Self::Delete { position, .. }
            | Self::Replace { position, .. } => *position,
        }
    }

    /// Returns the net document size change caused by this operation.
    ///
    /// Positive means the document grew; negative means it shrank.
    pub fn size_delta(&self) -> i64 {
        match self {
            Self::Insert { length, .. } => i64::from(*length),
            Self::Delete { length, .. } => -i64::from(*length),
            Self::Replace {
                old_length,
                new_length,
                ..
            } => i64::from(*new_length) - i64::from(*old_length),
        }
    }

    /// Returns the inverse of this operation (for undo).
    ///
    /// - Insert becomes Delete
    /// - Delete becomes Insert
    /// - Replace swaps old/new
    pub fn inverse(&self) -> Self {
        match self {
            Self::Insert {
                position,
                length,
                scrap_offset,
            } => Self::Delete {
                position: *position,
                length: *length,
                scrap_offset: *scrap_offset,
            },
            Self::Delete {
                position,
                length,
                scrap_offset,
            } => Self::Insert {
                position: *position,
                length: *length,
                scrap_offset: *scrap_offset,
            },
            Self::Replace {
                position,
                old_length,
                new_length,
                old_scrap_offset,
                new_scrap_offset,
            } => Self::Replace {
                position: *position,
                old_length: *new_length,
                new_length: *old_length,
                old_scrap_offset: *new_scrap_offset,
                new_scrap_offset: *old_scrap_offset,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_position_returns_correct_value() {
        let op = EditOperation::Insert {
            position: 42,
            length: 5,
            scrap_offset: 0,
        };
        assert_eq!(op.position(), 42);
    }

    #[test]
    fn insert_size_delta_is_positive() {
        let op = EditOperation::Insert {
            position: 0,
            length: 10,
            scrap_offset: 0,
        };
        assert_eq!(op.size_delta(), 10);
    }

    #[test]
    fn delete_size_delta_is_negative() {
        let op = EditOperation::Delete {
            position: 0,
            length: 7,
            scrap_offset: 0,
        };
        assert_eq!(op.size_delta(), -7);
    }

    #[test]
    fn replace_size_delta_is_difference() {
        let op = EditOperation::Replace {
            position: 0,
            old_length: 5,
            new_length: 8,
            old_scrap_offset: 0,
            new_scrap_offset: 5,
        };
        assert_eq!(op.size_delta(), 3);
    }

    #[test]
    fn insert_inverse_is_delete() {
        let op = EditOperation::Insert {
            position: 10,
            length: 3,
            scrap_offset: 5,
        };
        let inv = op.inverse();
        assert_eq!(
            inv,
            EditOperation::Delete {
                position: 10,
                length: 3,
                scrap_offset: 5,
            }
        );
    }

    #[test]
    fn delete_inverse_is_insert() {
        let op = EditOperation::Delete {
            position: 10,
            length: 3,
            scrap_offset: 5,
        };
        let inv = op.inverse();
        assert_eq!(
            inv,
            EditOperation::Insert {
                position: 10,
                length: 3,
                scrap_offset: 5,
            }
        );
    }

    #[test]
    fn replace_inverse_swaps_old_and_new() {
        let op = EditOperation::Replace {
            position: 10,
            old_length: 3,
            new_length: 5,
            old_scrap_offset: 0,
            new_scrap_offset: 3,
        };
        let inv = op.inverse();
        assert_eq!(
            inv,
            EditOperation::Replace {
                position: 10,
                old_length: 5,
                new_length: 3,
                old_scrap_offset: 3,
                new_scrap_offset: 0,
            }
        );
    }
}
