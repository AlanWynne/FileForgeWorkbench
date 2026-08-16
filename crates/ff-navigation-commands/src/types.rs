//! Core types for the navigation-commands crate.
//!
//! Defines data structures shared across multiple command implementations.

/// Active column boundaries for column-sensitive operations.
///
/// Bounds restrict operations like SORT, FIND, CHANGE, and shift
/// to a specific column range within each line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveBounds {
    /// Left column boundary (1-based, inclusive).
    pub left: u64,
    /// Right column boundary (1-based, inclusive).
    pub right: u64,
}

impl ActiveBounds {
    /// Create validated bounds.
    ///
    /// Returns `None` if `left < 1` or `right <= left`.
    pub fn new(left: u64, right: u64) -> Option<Self> {
        if left >= 1 && right > left {
            Some(Self { left, right })
        } else {
            None
        }
    }

    /// Compute the intersection of these bounds with an explicit column range.
    ///
    /// Returns `None` if the intersection is empty (effective_start > effective_end).
    pub fn intersect(&self, col1: u64, col2: u64) -> Option<(u64, u64)> {
        let effective_left = self.left.max(col1);
        let effective_right = self.right.min(col2);
        if effective_left <= effective_right {
            Some((effective_left, effective_right))
        } else {
            None
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Sort in ascending order (A–Z, 0–9).
    #[default]
    Ascending,
    /// Sort in descending order (Z–A, 9–0).
    Descending,
}

/// Sort scope qualifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortScope {
    /// Sort all visible lines (default).
    AllVisible,
    /// Sort only tagged lines.
    Tagged,
    /// Sort only currently visible (non-excluded) lines.
    Visible,
    /// Sort lines within a pending CC block.
    Block {
        /// First line of the block (1-based).
        start_line: u64,
        /// Last line of the block (1-based).
        end_line: u64,
    },
}

/// Parsed SORT command parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortParams {
    /// Optional explicit column range for the sort key (1-based, inclusive).
    pub column_range: Option<(u64, u64)>,
    /// Sort direction (A or D).
    pub direction: SortDirection,
    /// Scope qualifier.
    pub scope: SortScope,
}

/// Modifier indicating whether a navigation operation should extend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionModifier {
    /// Move caret without changing selection (collapse).
    Move,
    /// Extend selection from anchor to new caret position.
    Extend,
}

/// Direction for word/word-part navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordDirection {
    /// Move towards the beginning of the document.
    Left,
    /// Move towards the end of the document.
    Right,
}

/// Word navigation variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordNavKind {
    /// Move to start of previous/next word.
    WordStart,
    /// Move to end of current/next word.
    WordEnd,
}

/// Sub-word boundary detection result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordPartBoundary {
    /// Lowercase to uppercase transition (camelCase).
    LowerToUpper,
    /// End of uppercase run before lowercase (XMLParser → XML|Parser).
    UpperRunBeforeLower,
    /// Alphanumeric to non-alphanumeric transition.
    AlphaToNonAlpha,
    /// Digit to alpha or alpha to digit transition.
    DigitAlphaTransition,
    /// Start or end of word (no internal boundary found).
    WordEdge,
}

/// A single COLS_Line display artifact, anchored to a document position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColsLine {
    /// The document line number this COLS_Line is anchored above.
    pub anchor_line: u64,
    /// Unique identifier for this COLS_Line instance.
    pub id: u64,
}

/// Result of a COLS toggle operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColsToggleResult {
    /// A new COLS_Line was inserted.
    Inserted(ColsLine),
    /// An existing COLS_Line was removed.
    Removed(u64),
}

/// Configuration values for navigation commands.
#[derive(Debug, Clone, PartialEq)]
pub struct NavigationConfig {
    /// Columns to scroll for LEFT/RIGHT without explicit count.
    pub horizontal_scroll_columns: u64,
    /// Lines of overlap to retain when scrolling by page.
    pub page_overlap_lines: u64,
    /// Whether active Bounds restrict FIND operations.
    pub bounds_affect_find: bool,
    /// Additional characters to treat as word characters.
    pub extra_word_characters: String,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            horizontal_scroll_columns: 8,
            page_overlap_lines: 2,
            bounds_affect_find: false,
            extra_word_characters: String::new(),
        }
    }
}
