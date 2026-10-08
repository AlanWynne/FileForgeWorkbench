//! Core newtypes and shared types for the document model.
//!
//! Provides `BytePosition`, `LineNumber`, `CharacterExtracted`, `Direction`,
//! and related types used throughout the crate.

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A byte offset within the document buffer. Uses u64 to support >2 GB documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BytePosition(pub u64);

impl BytePosition {
    /// The zero position (start of document).
    pub const ZERO: Self = Self(0);

    /// Returns the inner u64 value.
    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for BytePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<u64> for BytePosition {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for BytePosition {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Sub<u64> for BytePosition {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self {
        Self(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<u64> for BytePosition {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_sub(rhs);
    }
}

impl Sub<BytePosition> for BytePosition {
    type Output = u64;
    fn sub(self, rhs: BytePosition) -> u64 {
        self.0.saturating_sub(rhs.0)
    }
}

impl From<u64> for BytePosition {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<BytePosition> for u64 {
    fn from(pos: BytePosition) -> u64 {
        pos.0
    }
}

/// A 0-based line number within the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineNumber(pub u64);

impl LineNumber {
    /// The first line (line 0).
    pub const ZERO: Self = Self(0);

    /// Convert to 1-based display number.
    pub fn to_display(self) -> u64 {
        self.0 + 1
    }

    /// Create from a 1-based display number.
    pub fn from_display(display: u64) -> Self {
        Self(display.saturating_sub(1))
    }

    /// Returns the inner u64 value.
    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}", self.to_display())
    }
}

impl Add<u64> for LineNumber {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for LineNumber {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Sub<u64> for LineNumber {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self {
        Self(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<u64> for LineNumber {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_sub(rhs);
    }
}

impl From<u64> for LineNumber {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<LineNumber> for u64 {
    fn from(ln: LineNumber) -> u64 {
        ln.0
    }
}

/// A 0-based record number within the document.
///
/// A record is the universal editable unit (CR-CH-058). For a Delimited
/// document a record IS a line, so `RecordNumber` and `LineNumber` are
/// interchangeable for Delimited documents and the line API delegates to the
/// record API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RecordNumber(pub u64);

impl RecordNumber {
    /// The first record (record 0).
    pub const ZERO: Self = Self(0);

    /// Returns the inner u64 value.
    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for RecordNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "record {}", self.0)
    }
}

impl Add<u64> for RecordNumber {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for RecordNumber {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Sub<u64> for RecordNumber {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self {
        Self(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<u64> for RecordNumber {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_sub(rhs);
    }
}

impl From<u64> for RecordNumber {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<RecordNumber> for u64 {
    fn from(rn: RecordNumber) -> u64 {
        rn.0
    }
}

impl From<LineNumber> for RecordNumber {
    fn from(ln: LineNumber) -> Self {
        Self(ln.0)
    }
}

impl From<RecordNumber> for LineNumber {
    fn from(rn: RecordNumber) -> Self {
        Self(rn.0)
    }
}

/// A range of bytes within the Append_Buffer: a `(start, len)` pair.
///
/// Used by `Piece::Edited` to reference the appended bytes of an edited or
/// inserted record. `u64` to support large edit volumes without a type ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BufRange {
    /// Start offset of the appended bytes within the Append_Buffer.
    pub start: u64,
    /// Length in bytes of the appended span.
    pub len: u64,
}

impl BufRange {
    /// Create a new buffer range.
    pub fn new(start: u64, len: u64) -> Self {
        Self { start, len }
    }

    /// One past the last byte of this range.
    pub fn end(self) -> u64 {
        self.start + self.len
    }
}

/// A Unicode code point extracted from the buffer with its byte width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterExtracted {
    /// The Unicode code point (or U+FFFD for invalid bytes).
    pub character: char,
    /// Number of bytes this character occupies in UTF-8.
    pub byte_width: u8,
}

/// Direction for character navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Navigate toward higher byte positions.
    Forward,
    /// Navigate toward lower byte positions.
    Backward,
}

/// Two-segment read-only view over gap buffer content.
/// Segment 1 = bytes before the gap; Segment 2 = bytes after the gap.
#[derive(Debug, Clone)]
pub struct SplitView {
    /// Bytes before the gap.
    pub before_gap: Vec<u8>,
    /// Bytes after the gap.
    pub after_gap: Vec<u8>,
}

impl SplitView {
    /// Total content length across both segments.
    pub fn length(&self) -> u64 {
        (self.before_gap.len() + self.after_gap.len()) as u64
    }

    /// Get byte at a logical content position.
    pub fn byte_at(&self, position: u64) -> Option<u8> {
        let before_len = self.before_gap.len() as u64;
        if position < before_len {
            Some(self.before_gap[position as usize])
        } else {
            let offset = position - before_len;
            self.after_gap.get(offset as usize).copied()
        }
    }

    /// Iterate over all content bytes in order.
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.before_gap.iter().chain(self.after_gap.iter()).copied()
    }
}

/// Result of an insertion operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertResult {
    /// Number of lines added by the insertion.
    pub lines_added: u64,
    /// Byte length of inserted content.
    pub bytes_inserted: u64,
}

/// Result of a deletion operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteResult {
    /// Number of lines removed by the deletion.
    pub lines_removed: u64,
    /// Byte length of deleted content.
    pub bytes_deleted: u64,
}

/// Current state of a streaming file load operation.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingProgress {
    /// Load has not started.
    NotStarted,
    /// Load is in progress.
    InProgress {
        /// Bytes loaded so far.
        bytes_loaded: u64,
        /// Estimated total bytes (from VFS metadata).
        estimated_total: Option<u64>,
    },
    /// Load completed successfully.
    Complete {
        /// Total bytes loaded.
        total_bytes: u64,
        /// Total lines in the document.
        total_lines: u64,
    },
    /// Load failed with an error.
    Failed {
        /// Error description.
        reason: String,
        /// Bytes loaded before failure.
        bytes_loaded: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_position_arithmetic() {
        let pos = BytePosition(10);
        assert_eq!(pos + 5, BytePosition(15));
        assert_eq!(pos - 3, BytePosition(7));
        assert_eq!(pos - 20, BytePosition(0)); // saturating
        assert_eq!(BytePosition(20) - BytePosition(5), 15u64);
    }

    #[test]
    fn byte_position_display() {
        assert_eq!(BytePosition(42).to_string(), "42");
    }

    #[test]
    fn line_number_display_is_one_based() {
        let ln = LineNumber(0);
        assert_eq!(ln.to_display(), 1);
        assert_eq!(ln.to_string(), "line 1");

        let ln2 = LineNumber(9);
        assert_eq!(ln2.to_display(), 10);
        assert_eq!(ln2.to_string(), "line 10");
    }

    #[test]
    fn line_number_from_display() {
        assert_eq!(LineNumber::from_display(1), LineNumber(0));
        assert_eq!(LineNumber::from_display(5), LineNumber(4));
        assert_eq!(LineNumber::from_display(0), LineNumber(0)); // saturating
    }

    #[test]
    fn line_number_arithmetic() {
        let ln = LineNumber(5);
        assert_eq!(ln + 3, LineNumber(8));
        assert_eq!(ln - 2, LineNumber(3));
        assert_eq!(ln - 10, LineNumber(0)); // saturating
    }

    #[test]
    fn split_view_length_and_access() {
        let view = SplitView {
            before_gap: vec![1, 2, 3],
            after_gap: vec![4, 5],
        };
        assert_eq!(view.length(), 5);
        assert_eq!(view.byte_at(0), Some(1));
        assert_eq!(view.byte_at(2), Some(3));
        assert_eq!(view.byte_at(3), Some(4));
        assert_eq!(view.byte_at(4), Some(5));
        assert_eq!(view.byte_at(5), None);
    }

    #[test]
    fn split_view_iter() {
        let view = SplitView {
            before_gap: vec![10, 20],
            after_gap: vec![30, 40, 50],
        };
        let collected: Vec<u8> = view.iter().collect();
        assert_eq!(collected, vec![10, 20, 30, 40, 50]);
    }

    #[test]
    fn record_number_arithmetic_and_display() {
        // Validates: Requirement 11.4
        let rn = RecordNumber(5);
        assert_eq!(rn + 3, RecordNumber(8));
        assert_eq!(rn - 2, RecordNumber(3));
        assert_eq!(rn - 10, RecordNumber(0)); // saturating
        assert_eq!(rn.value(), 5);
        assert_eq!(RecordNumber(7).to_string(), "record 7");
        assert_eq!(RecordNumber::ZERO, RecordNumber(0));
    }

    #[test]
    fn record_number_line_number_interchangeable() {
        // Validates: Requirement 11.3
        let ln = LineNumber(42);
        let rn: RecordNumber = ln.into();
        assert_eq!(rn, RecordNumber(42));
        let back: LineNumber = rn.into();
        assert_eq!(back, LineNumber(42));
    }

    #[test]
    fn buf_range_end() {
        // Validates: Requirement 12.1
        let r = BufRange::new(10, 5);
        assert_eq!(r.start, 10);
        assert_eq!(r.len, 5);
        assert_eq!(r.end(), 15);
    }

    #[test]
    fn character_extracted_equality() {
        let a = CharacterExtracted {
            character: 'A',
            byte_width: 1,
        };
        let b = CharacterExtracted {
            character: 'A',
            byte_width: 1,
        };
        assert_eq!(a, b);
    }
}
