//! Core newtypes and enums for the large-file-performance subsystem.

/// A style slot index identifying a font/style combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StyleSlot(pub u16);

/// A monotonic clock value for cache eviction ordering.
/// Wraps at u16::MAX and resets all entries to prevent stale comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClockValue(pub u16);

impl ClockValue {
    /// Increment the clock, wrapping at u16::MAX.
    pub fn increment(self) -> Self {
        if self.0 == u16::MAX {
            Self(1) // Reset to 1 (not 0) to distinguish from "never accessed"
        } else {
            Self(self.0 + 1)
        }
    }
}

/// A character offset within a line (0-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharOffset(pub u64);

/// An x-position in fractional pixels from the left margin of a line.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct XPosition(pub f64);

/// A range of characters within a line for chunked measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRange {
    /// Start character offset (inclusive).
    pub start: CharOffset,
    /// End character offset (exclusive).
    pub end: CharOffset,
}

impl ChunkRange {
    /// Create a new chunk range.
    pub fn new(start: u64, end: u64) -> Self {
        Self {
            start: CharOffset(start),
            end: CharOffset(end),
        }
    }

    /// Length of the range in characters.
    pub fn len(&self) -> u64 {
        self.end.0.saturating_sub(self.start.0)
    }

    /// Returns true if the range is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if this range overlaps with another.
    pub fn overlaps(&self, other: &ChunkRange) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// The render chunk size limit for text drawing calls.
/// Clamped to [50, 1000]. Default: 300.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderChunkSize(pub u32);

impl RenderChunkSize {
    pub const MIN: u32 = 50;
    pub const MAX: u32 = 1000;
    pub const DEFAULT: u32 = 300;

    /// Create a RenderChunkSize, clamping to valid range.
    pub fn new(chars: u32) -> Self {
        Self(chars.clamp(Self::MIN, Self::MAX))
    }
}

impl Default for RenderChunkSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// The long-line threshold in characters.
/// Clamped to [1_000, 100_000]. Default: 10_000.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LongLineThreshold(pub u32);

impl LongLineThreshold {
    pub const MIN: u32 = 1_000;
    pub const MAX: u32 = 100_000;
    pub const DEFAULT: u32 = 10_000;

    /// Create a LongLineThreshold, clamping to valid range.
    pub fn new(chars: u32) -> Self {
        Self(chars.clamp(Self::MIN, Self::MAX))
    }
}

impl Default for LongLineThreshold {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// Cache scoping level — determines how many lines are cached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLevel {
    /// Cache only visible viewport lines (for files > 1M lines).
    Viewport,
    /// Cache visible + overscan buffer (default for files < 1M lines).
    Page,
    /// Cache all lines (only for files < 10,000 lines).
    Document,
}

/// Validity levels for a LineLayout entry.
///
/// Determines what must be recomputed before the entry can be reused.
/// Adapted from Scintilla's `LineLayout::ValidLevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidLevel {
    /// Completely stale — must remeasure from scratch.
    Invalid = 0,
    /// Text or style may have changed — verify before reuse.
    CheckTextAndStyle = 1,
    /// Positions valid but sub-line breaks need recalculation (e.g., after resize).
    Positions = 2,
    /// Fully valid — positions and sub-line breaks are current.
    Lines = 3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_chunk_size_clamped() {
        // Validates: Requirement 1 AC 6
        assert_eq!(RenderChunkSize::new(0).0, RenderChunkSize::MIN);
        assert_eq!(RenderChunkSize::new(10000).0, RenderChunkSize::MAX);
        assert_eq!(RenderChunkSize::new(300).0, 300);
    }

    #[test]
    fn long_line_threshold_clamped() {
        // Validates: Requirement 1 AC 5
        assert_eq!(LongLineThreshold::new(0).0, LongLineThreshold::MIN);
        assert_eq!(LongLineThreshold::new(200_000).0, LongLineThreshold::MAX);
        assert_eq!(LongLineThreshold::new(10_000).0, 10_000);
    }

    #[test]
    fn chunk_range_len() {
        let r = ChunkRange::new(10, 20);
        assert_eq!(r.len(), 10);
    }

    #[test]
    fn chunk_range_empty() {
        let r = ChunkRange::new(5, 5);
        assert!(r.is_empty());
    }

    #[test]
    fn chunk_range_overlaps() {
        let a = ChunkRange::new(0, 10);
        let b = ChunkRange::new(5, 15);
        let c = ChunkRange::new(10, 20);
        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c)); // [0,10) and [10,20) don't overlap
    }

    #[test]
    fn clock_value_wraps_at_max() {
        // Validates: Requirement 2 AC 7
        let max = ClockValue(u16::MAX);
        let wrapped = max.increment();
        assert_eq!(wrapped.0, 1);
    }

    #[test]
    fn valid_level_ordering() {
        // Validates: Requirement 3 AC 5
        assert!(ValidLevel::Invalid < ValidLevel::CheckTextAndStyle);
        assert!(ValidLevel::CheckTextAndStyle < ValidLevel::Positions);
        assert!(ValidLevel::Positions < ValidLevel::Lines);
    }
}
