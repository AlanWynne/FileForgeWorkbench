//! Line command types, enums, and classification logic.
//!
//! Defines `ParsedLineCommand`, `LineCommandKind`, `LineCommandCategory`,
//! `BlockCommandKind`, `BlockPair`, `SourceTarget`, and `ExecutableCommand`.

/// A line command parsed from a prefix-area input string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLineCommand {
    /// The line number where this command was entered (0-based document line).
    pub line: u64,
    /// The kind of line command parsed.
    pub kind: LineCommandKind,
}

/// All possible line command types with their parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LineCommandKind {
    // --- Delete ---
    /// Delete a single line.
    Delete,
    /// Delete n consecutive lines starting at this line.
    DeleteCount(u32),
    /// Block delete marker (DD). Requires matching pair.
    DeleteBlock,

    // --- Insert ---
    /// Insert one blank line after this line.
    Insert,
    /// Insert n blank lines after this line.
    InsertCount(u32),

    // --- Repeat ---
    /// Duplicate this line once.
    Repeat,
    /// Duplicate this line n times.
    RepeatCount(u32),
    /// Block repeat marker (RR). Requires matching pair.
    RepeatBlock,

    // --- Copy ---
    /// Single-line copy source marker.
    Copy,
    /// Block copy source marker (CC). Requires matching pair.
    CopyBlock,

    // --- Move ---
    /// Single-line move source marker.
    Move,
    /// Block move source marker (MM). Requires matching pair.
    MoveBlock,

    // --- Target ---
    /// After-insertion target.
    After,
    /// Before-insertion target.
    Before,

    // --- Exclude ---
    /// Exclude a single line from the viewport.
    Exclude,
    /// Exclude n consecutive lines.
    ExcludeCount(u32),
    /// Block exclude marker (XX). Requires matching pair.
    ExcludeBlock,

    // --- Tag/Untag ---
    /// Tag a single line.
    Tag,
    /// Block tag marker (TT). Requires matching pair.
    TagBlock,
    /// Untag a single line.
    Untag,
    /// Block untag marker (UU). Requires matching pair.
    UntagBlock,

    // --- Shift Right ---
    /// Shift right by default ShiftWidth.
    ShiftRight,
    /// Shift right by n columns.
    ShiftRightCount(u32),
    /// Block shift right marker (>>). Requires matching pair.
    ShiftRightBlock,

    // --- Shift Left ---
    /// Shift left by default ShiftWidth.
    ShiftLeft,
    /// Shift left by n columns.
    ShiftLeftCount(u32),
    /// Block shift left marker (<<). Requires matching pair.
    ShiftLeftBlock,

    // --- Bounds-Aware Shift ---
    /// Bounds-aware shift right by one position.
    BoundsShiftRight,
    /// Block bounds-aware shift right marker ()). Requires matching pair.
    BoundsShiftRightBlock,
    /// Bounds-aware shift left by one position.
    BoundsShiftLeft,
    /// Block bounds-aware shift left marker (((). Requires matching pair.
    BoundsShiftLeftBlock,
}

/// Classification of line commands for resolution and compatibility logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineCommandCategory {
    /// Commands that execute immediately without a partner or target.
    Immediate,
    /// Block markers that require exactly one matching pair to execute.
    Block,
    /// Source markers that require a target (A or B) to resolve.
    Source,
    /// Target markers that resolve pending source markers.
    Target,
}

/// Block command kinds (the paired variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCommandKind {
    Delete,
    Repeat,
    Exclude,
    Tag,
    Untag,
    ShiftRight,
    ShiftLeft,
    BoundsRight,
    BoundsLeft,
    Copy,
    Move,
}

impl std::fmt::Display for BlockCommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Delete => "DD",
            Self::Repeat => "RR",
            Self::Exclude => "XX",
            Self::Tag => "TT",
            Self::Untag => "UU",
            Self::ShiftRight => ">>",
            Self::ShiftLeft => "<<",
            Self::BoundsRight => "))",
            Self::BoundsLeft => "((",
            Self::Copy => "CC",
            Self::Move => "MM",
        };
        write!(f, "{}", s)
    }
}

/// A validated and normalized block command pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPair {
    /// The block command kind.
    pub kind: BlockCommandKind,
    /// The start line of the block (inclusive, normalized to min).
    pub start_line: u64,
    /// The end line of the block (inclusive, normalized to max).
    pub end_line: u64,
}

/// A resolved source + target combination for copy/move operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTarget {
    /// The operation type.
    pub operation: SourceOperation,
    /// Source start line (inclusive).
    pub source_start: u64,
    /// Source end line (inclusive).
    pub source_end: u64,
    /// Target insertion point.
    pub target_line: u64,
    /// Whether to insert after (A) or before (B) the target line.
    pub target_position: TargetPosition,
}

/// Whether the source operation is copy or move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceOperation {
    Copy,
    Move,
}

/// Insertion position relative to the target line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPosition {
    After,
    Before,
}

/// A command that has been fully resolved and is ready for execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableCommand {
    /// Delete lines (D, Dn, or resolved DD pair).
    Delete { start_line: u64, count: u64 },
    /// Insert blank lines after a line.
    Insert { after_line: u64, count: u32 },
    /// Repeat (duplicate) a single line.
    Repeat { start_line: u64, count: u32 },
    /// Repeat block (duplicate a range).
    RepeatBlock { start_line: u64, end_line: u64 },
    /// Copy lines to a target position.
    CopyToTarget(SourceTarget),
    /// Move lines to a target position.
    MoveToTarget(SourceTarget),
    /// Exclude lines from viewport.
    Exclude { start_line: u64, count: u64 },
    /// Tag lines.
    Tag { start_line: u64, end_line: u64 },
    /// Untag lines.
    Untag { start_line: u64, end_line: u64 },
    /// Shift lines right.
    ShiftRight {
        start_line: u64,
        end_line: u64,
        columns: u32,
    },
    /// Shift lines left.
    ShiftLeft {
        start_line: u64,
        end_line: u64,
        columns: u32,
    },
    /// Bounds-aware shift right.
    BoundsShiftRight { start_line: u64, end_line: u64 },
    /// Bounds-aware shift left.
    BoundsShiftLeft { start_line: u64, end_line: u64 },
}

/// Classify a `LineCommandKind` into its category.
pub fn classify(kind: &LineCommandKind) -> LineCommandCategory {
    match kind {
        // Immediate commands — execute without partner or target
        LineCommandKind::Delete
        | LineCommandKind::DeleteCount(_)
        | LineCommandKind::Insert
        | LineCommandKind::InsertCount(_)
        | LineCommandKind::Repeat
        | LineCommandKind::RepeatCount(_)
        | LineCommandKind::Exclude
        | LineCommandKind::ExcludeCount(_)
        | LineCommandKind::Tag
        | LineCommandKind::Untag
        | LineCommandKind::ShiftRight
        | LineCommandKind::ShiftRightCount(_)
        | LineCommandKind::ShiftLeft
        | LineCommandKind::ShiftLeftCount(_)
        | LineCommandKind::BoundsShiftRight
        | LineCommandKind::BoundsShiftLeft => LineCommandCategory::Immediate,

        // Block markers — require exactly one matching pair
        LineCommandKind::DeleteBlock
        | LineCommandKind::RepeatBlock
        | LineCommandKind::ExcludeBlock
        | LineCommandKind::TagBlock
        | LineCommandKind::UntagBlock
        | LineCommandKind::ShiftRightBlock
        | LineCommandKind::ShiftLeftBlock
        | LineCommandKind::BoundsShiftRightBlock
        | LineCommandKind::BoundsShiftLeftBlock
        | LineCommandKind::CopyBlock
        | LineCommandKind::MoveBlock => LineCommandCategory::Block,

        // Source markers — need target to resolve
        LineCommandKind::Copy | LineCommandKind::Move => LineCommandCategory::Source,

        // Target markers — resolve pending source markers
        LineCommandKind::After | LineCommandKind::Before => LineCommandCategory::Target,
    }
}

/// Returns true if the given kind is a block marker requiring a pair.
pub fn is_block_marker(kind: &LineCommandKind) -> bool {
    classify(kind) == LineCommandCategory::Block
}

/// Map a `LineCommandKind` block variant to its `BlockCommandKind`.
/// Returns `None` for non-block kinds.
pub fn to_block_kind(kind: &LineCommandKind) -> Option<BlockCommandKind> {
    match kind {
        LineCommandKind::DeleteBlock => Some(BlockCommandKind::Delete),
        LineCommandKind::RepeatBlock => Some(BlockCommandKind::Repeat),
        LineCommandKind::ExcludeBlock => Some(BlockCommandKind::Exclude),
        LineCommandKind::TagBlock => Some(BlockCommandKind::Tag),
        LineCommandKind::UntagBlock => Some(BlockCommandKind::Untag),
        LineCommandKind::ShiftRightBlock => Some(BlockCommandKind::ShiftRight),
        LineCommandKind::ShiftLeftBlock => Some(BlockCommandKind::ShiftLeft),
        LineCommandKind::BoundsShiftRightBlock => Some(BlockCommandKind::BoundsRight),
        LineCommandKind::BoundsShiftLeftBlock => Some(BlockCommandKind::BoundsLeft),
        LineCommandKind::CopyBlock => Some(BlockCommandKind::Copy),
        LineCommandKind::MoveBlock => Some(BlockCommandKind::Move),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_delete_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::Delete),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_delete_count_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::DeleteCount(5)),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_delete_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::DeleteBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_insert_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::Insert),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_insert_count_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::InsertCount(3)),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_repeat_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::Repeat),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_repeat_count_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::RepeatCount(2)),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_repeat_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::RepeatBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_copy_is_source() {
        assert_eq!(
            classify(&LineCommandKind::Copy),
            LineCommandCategory::Source
        );
    }

    #[test]
    fn classify_copy_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::CopyBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_move_is_source() {
        assert_eq!(
            classify(&LineCommandKind::Move),
            LineCommandCategory::Source
        );
    }

    #[test]
    fn classify_move_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::MoveBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_after_is_target() {
        assert_eq!(
            classify(&LineCommandKind::After),
            LineCommandCategory::Target
        );
    }

    #[test]
    fn classify_before_is_target() {
        assert_eq!(
            classify(&LineCommandKind::Before),
            LineCommandCategory::Target
        );
    }

    #[test]
    fn classify_exclude_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::Exclude),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_exclude_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::ExcludeBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_tag_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::Tag),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_tag_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::TagBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_untag_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::Untag),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_untag_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::UntagBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_shift_right_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::ShiftRight),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_shift_right_count_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::ShiftRightCount(4)),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_shift_right_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::ShiftRightBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_shift_left_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::ShiftLeft),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_shift_left_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::ShiftLeftBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_bounds_shift_right_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::BoundsShiftRight),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_bounds_shift_right_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::BoundsShiftRightBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn classify_bounds_shift_left_is_immediate() {
        assert_eq!(
            classify(&LineCommandKind::BoundsShiftLeft),
            LineCommandCategory::Immediate
        );
    }

    #[test]
    fn classify_bounds_shift_left_block_is_block() {
        assert_eq!(
            classify(&LineCommandKind::BoundsShiftLeftBlock),
            LineCommandCategory::Block
        );
    }

    #[test]
    fn is_block_marker_returns_true_for_blocks() {
        assert!(is_block_marker(&LineCommandKind::DeleteBlock));
        assert!(is_block_marker(&LineCommandKind::RepeatBlock));
        assert!(is_block_marker(&LineCommandKind::CopyBlock));
        assert!(is_block_marker(&LineCommandKind::MoveBlock));
    }

    #[test]
    fn is_block_marker_returns_false_for_non_blocks() {
        assert!(!is_block_marker(&LineCommandKind::Delete));
        assert!(!is_block_marker(&LineCommandKind::Copy));
        assert!(!is_block_marker(&LineCommandKind::After));
    }
}
