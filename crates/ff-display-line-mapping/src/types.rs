//! Core newtypes for the display-line-mapping crate.
//!
//! These newtypes prevent accidental misuse of raw `usize` values by
//! distinguishing document lines, display lines, and sub-line offsets
//! at the type level.

/// A zero-based document line index.
///
/// Addresses: Requirement 1 AC 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocLine(pub usize);

/// A zero-based display line index (contiguous across visible content).
///
/// Addresses: Requirement 1 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayLine(pub usize);

/// A zero-based sub-line offset within a wrapped document line.
/// Sub-line 0 is the first visual line of a wrapped document line.
///
/// Addresses: Requirement 4 AC 8
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubLine(pub usize);

/// Result of a display-to-document lookup, including the sub-line offset.
///
/// Addresses: Requirement 1 AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocPosition {
    /// The document line containing this display line.
    pub doc_line: DocLine,
    /// The sub-line offset within the document line (0 for unwrapped).
    pub sub_line: SubLine,
}

/// Notification payload when display line count changes.
///
/// Addresses: Requirement 7 AC 9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayLineCountChange {
    /// Previous total display lines.
    pub old_count: usize,
    /// New total display lines.
    pub new_count: usize,
}

/// Handle for a registered change listener.
///
/// Addresses: Requirement 7 AC 9
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerHandle(pub u64);
