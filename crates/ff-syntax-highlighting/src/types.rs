//! Shared types used throughout the syntax-highlighting crate.

/// A style-slot index (0–255) assigned to character positions by the lexer.
/// The theme system resolves each index to visual attributes.
/// Addresses: Requirement 2, criterion 2.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StyleSlotIndex(pub u8);

impl StyleSlotIndex {
    /// The default/unstyled index.
    pub const DEFAULT: Self = Self(0);

    /// Maximum valid index.
    pub const MAX: Self = Self(255);

    /// Get the raw u8 value.
    pub fn value(self) -> u8 {
        self.0
    }
}

impl Default for StyleSlotIndex {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// An opaque integer representing the lexer's parsing state at a position.
/// Stored per-line for incremental re-highlighting.
/// Addresses: Requirement 3, criterion 3.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LexerState(pub i32);

impl LexerState {
    /// Initial state for the beginning of a document or unknown state.
    pub const INITIAL: Self = Self(0);
}

impl Default for LexerState {
    fn default() -> Self {
        Self::INITIAL
    }
}

/// A byte offset into the document text buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytePosition(pub usize);

/// A zero-based line index into the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineNumber(pub usize);

/// A contiguous range of characters sharing the same style-slot index.
/// Produced by styled_spans() for the viewport renderer.
/// Addresses: Requirement 2, criterion 2.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Byte offset of the span start.
    pub start: BytePosition,
    /// Byte offset of the span end (exclusive).
    pub end: BytePosition,
    /// The style-slot index for this span.
    pub style: StyleSlotIndex,
}

/// Flags associated with a line's fold level.
/// Addresses: Requirement 8, criterion 8.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FoldFlags(pub u8);

impl FoldFlags {
    /// No flags set.
    pub const NONE: Self = Self(0);
    /// Line is a fold header (begins a foldable region).
    pub const FOLD_HEADER: Self = Self(1 << 0);
    /// Line contains only whitespace.
    pub const FOLD_WHITESPACE: Self = Self(1 << 1);

    /// Check if this flags value contains the specified flag.
    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }

    /// Insert a flag into this flags value.
    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    /// Remove a flag from this flags value.
    pub fn remove(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }
}

impl Default for FoldFlags {
    fn default() -> Self {
        Self::NONE
    }
}

/// A 12-bit fold level (0–4095) representing nesting depth at end of line.
/// Addresses: Requirement 8, criterion 8.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FoldLevel(u16);

impl FoldLevel {
    /// Minimum fold level.
    pub const MIN: Self = Self(0);
    /// Maximum fold level (12-bit).
    pub const MAX: Self = Self(4095);

    /// Create a fold level, clamping to [0, 4095].
    pub fn new(level: u16) -> Self {
        Self(level.min(4095))
    }

    /// Get the raw u16 value.
    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for FoldLevel {
    fn default() -> Self {
        Self::MIN
    }
}

/// Metadata about a keyword set supported by a lexer.
/// Addresses: Requirement 1, criterion 1.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordSetDescriptor {
    /// Set index (0–8).
    pub index: u8,
    /// Human-readable name (e.g., "Primary keywords", "Type names").
    pub name: String,
    /// Description of what this keyword set represents.
    pub description: String,
}

/// Index identifying which keyword set (0–8) matched.
/// Addresses: Requirement 5, criterion 5.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeywordSetIndex(pub u8);

impl KeywordSetIndex {
    /// Maximum supported keyword set index.
    pub const MAX: u8 = 8;

    /// Create a new keyword set index, returning None if out of range.
    pub fn new(index: u8) -> Option<Self> {
        if index <= Self::MAX {
            Some(Self(index))
        } else {
            None
        }
    }

    /// Get the raw u8 value.
    pub fn value(self) -> u8 {
        self.0
    }
}

/// Metadata about a lexer property for auto-discovery.
/// Addresses: Requirement 10, criterion 10.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDescriptor {
    /// Property key (e.g., "fold.comment").
    pub name: String,
    /// Property type hint.
    pub property_type: PropertyType,
    /// Human-readable description.
    pub description: String,
    /// Default value as string.
    pub default_value: String,
}

/// Type hint for lexer properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyType {
    /// String value.
    String,
    /// Integer value.
    Integer,
    /// Boolean value ("0"/"1" or "true"/"false").
    Boolean,
}

/// The public trait exposed to consumers (viewport renderer, minimap, export).
/// Consumers depend on this trait rather than the concrete HighlightEngine.
/// Addresses: Requirement 11, criterion 11.4
pub trait SyntaxHighlighter: Send + Sync {
    /// Guarantee all text up to `position` has valid style data.
    /// Addresses: Requirement 4, criterion 4.1
    fn ensure_styled_to(&mut self, position: BytePosition);

    /// Returns the current end-of-styled-text position.
    /// Addresses: Requirement 4, criterion 4.4
    fn styling_position(&self) -> BytePosition;

    /// Get the style index at a specific byte position. O(1).
    /// Addresses: Requirement 2, criterion 2.3
    fn style_at(&self, position: BytePosition) -> StyleSlotIndex;

    /// Get contiguous styled spans within a range.
    /// Addresses: Requirement 2, criterion 2.4
    fn styled_spans(&self, start: BytePosition, end: BytePosition) -> Vec<HighlightSpan>;

    /// Get the fold level and flags for a specific line.
    /// Addresses: Requirement 8, criterion 8.5
    fn fold_level_at(&self, line: LineNumber) -> (FoldLevel, FoldFlags);

    /// Get fold levels for a range of lines (bulk query).
    /// Addresses: Requirement 15, criterion 15.6
    fn fold_level_range(
        &self,
        start_line: LineNumber,
        end_line: LineNumber,
    ) -> Vec<(LineNumber, FoldLevel, FoldFlags)>;

    /// Get the number of base style slots the active lexer uses.
    /// Addresses: Requirement 12, criterion 12.4
    fn style_slot_count(&self) -> u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_slot_index_default_is_zero() {
        assert_eq!(StyleSlotIndex::default(), StyleSlotIndex(0));
    }

    #[test]
    fn style_slot_index_value_roundtrip() {
        let idx = StyleSlotIndex(42);
        assert_eq!(idx.value(), 42);
    }

    #[test]
    fn lexer_state_default_is_initial() {
        assert_eq!(LexerState::default(), LexerState::INITIAL);
        assert_eq!(LexerState::INITIAL.0, 0);
    }

    #[test]
    fn fold_level_clamps_to_4095() {
        assert_eq!(FoldLevel::new(5000).value(), 4095);
        assert_eq!(FoldLevel::new(4095).value(), 4095);
        assert_eq!(FoldLevel::new(0).value(), 0);
        assert_eq!(FoldLevel::new(100).value(), 100);
    }

    #[test]
    fn fold_flags_contains_and_insert() {
        let mut flags = FoldFlags::NONE;
        assert!(!flags.contains(FoldFlags::FOLD_HEADER));
        flags.insert(FoldFlags::FOLD_HEADER);
        assert!(flags.contains(FoldFlags::FOLD_HEADER));
        assert!(!flags.contains(FoldFlags::FOLD_WHITESPACE));
        flags.insert(FoldFlags::FOLD_WHITESPACE);
        assert!(flags.contains(FoldFlags::FOLD_HEADER));
        assert!(flags.contains(FoldFlags::FOLD_WHITESPACE));
    }

    #[test]
    fn fold_flags_remove() {
        let mut flags = FoldFlags::NONE;
        flags.insert(FoldFlags::FOLD_HEADER);
        flags.insert(FoldFlags::FOLD_WHITESPACE);
        flags.remove(FoldFlags::FOLD_HEADER);
        assert!(!flags.contains(FoldFlags::FOLD_HEADER));
        assert!(flags.contains(FoldFlags::FOLD_WHITESPACE));
    }

    #[test]
    fn keyword_set_index_valid_range() {
        assert!(KeywordSetIndex::new(0).is_some());
        assert!(KeywordSetIndex::new(8).is_some());
        assert!(KeywordSetIndex::new(9).is_none());
        assert!(KeywordSetIndex::new(255).is_none());
    }

    #[test]
    fn byte_position_ordering() {
        assert!(BytePosition(0) < BytePosition(10));
        assert!(BytePosition(10) == BytePosition(10));
    }

    #[test]
    fn line_number_ordering() {
        assert!(LineNumber(0) < LineNumber(1));
    }
}
