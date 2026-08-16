//! Error types for the ff-line-commands crate.
//!
//! All errors follow the `[line-cmd] operation: description` format
//! per cross-cutting error message standards.

/// Errors produced by the ff-line-commands crate.
///
/// Formatted per Error Message Standards: `[line-cmd] operation: description`
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LineCommandError {
    /// Unrecognised line command string entered in prefix area.
    #[error("[line-cmd] parse: unrecognised command '{input}'")]
    InvalidCommand { input: String },

    /// Block command has only one marker — awaiting matching pair.
    #[error("[line-cmd] pair: {kind} requires a matching pair")]
    AwaitingPair { kind: String },

    /// More than two markers of the same block type present.
    #[error("[line-cmd] pair: only two {kind} markers are permitted")]
    TooManyMarkers { kind: String },

    /// Overlapping block ranges from different block command types.
    #[error("[line-cmd] pair: overlapping block ranges for {kind1} and {kind2}")]
    OverlappingBlocks { kind1: String, kind2: String },

    /// Move target falls inside the source block.
    #[error("[line-cmd] move: target cannot be inside the source block")]
    TargetInsideSource,

    /// More than one A or B target marker pending.
    #[error("[line-cmd] target: only one target marker is permitted per operation")]
    DuplicateTarget,

    /// Primary command is incompatible with pending line commands.
    #[error("[line-cmd] compatibility: '{primary}' cannot be used with pending {line_cmd}")]
    IncompatibleCommands { primary: String, line_cmd: String },

    /// Source line commands combined with a file path argument on COPY/MOVE.
    #[error("[line-cmd] compatibility: source line commands cannot be combined with a file path argument")]
    SourceWithFilePath,

    /// Bounds-aware shift attempted without active BOUNDS.
    #[error("[line-cmd] bounds_shift: bounds-aware shift requires active BOUNDS")]
    NoBoundsActive,

    /// Line number is out of range for the document.
    #[error("[line-cmd] {operation}: line {line} is out of range (document has {total} lines)")]
    LineOutOfRange {
        operation: String,
        line: u64,
        total: u64,
    },

    /// Source markers awaiting target.
    #[error("[line-cmd] resolve: waiting for A or B target")]
    AwaitingTarget,

    /// Target markers awaiting source.
    #[error("[line-cmd] resolve: A/B target entered with no pending source")]
    AwaitingSource,

    /// Document mutation failed.
    #[error("[line-cmd] {operation}: document error — {description}")]
    DocumentError {
        operation: String,
        description: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_command_display_follows_format() {
        let err = LineCommandError::InvalidCommand {
            input: "ZZZ".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[line-cmd] parse: unrecognised command 'ZZZ'"
        );
    }

    #[test]
    fn awaiting_pair_display_follows_format() {
        let err = LineCommandError::AwaitingPair {
            kind: "DD".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[line-cmd] pair: DD requires a matching pair"
        );
    }

    #[test]
    fn too_many_markers_display_follows_format() {
        let err = LineCommandError::TooManyMarkers {
            kind: "CC".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[line-cmd] pair: only two CC markers are permitted"
        );
    }

    #[test]
    fn overlapping_blocks_display_follows_format() {
        let err = LineCommandError::OverlappingBlocks {
            kind1: "DD".to_string(),
            kind2: "XX".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[line-cmd] pair: overlapping block ranges for DD and XX"
        );
    }

    #[test]
    fn target_inside_source_display_follows_format() {
        let err = LineCommandError::TargetInsideSource;
        assert_eq!(
            err.to_string(),
            "[line-cmd] move: target cannot be inside the source block"
        );
    }

    #[test]
    fn line_out_of_range_display_follows_format() {
        let err = LineCommandError::LineOutOfRange {
            operation: "delete".to_string(),
            line: 100,
            total: 50,
        };
        assert_eq!(
            err.to_string(),
            "[line-cmd] delete: line 100 is out of range (document has 50 lines)"
        );
    }

    #[test]
    fn no_bounds_active_display_follows_format() {
        let err = LineCommandError::NoBoundsActive;
        assert_eq!(
            err.to_string(),
            "[line-cmd] bounds_shift: bounds-aware shift requires active BOUNDS"
        );
    }
}
