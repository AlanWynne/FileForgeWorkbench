//! Block pair validator — pair matching, normalization, and overlap detection.

use crate::command::{BlockCommandKind, BlockPair, LineCommandKind};
use crate::error::LineCommandError;
use crate::pending::PendingCommandStore;

/// Validates and normalizes block command pairs from the pending store.
pub struct BlockPairValidator;

impl BlockPairValidator {
    /// Normalize a pair so start_line <= end_line regardless of entry order.
    pub fn normalize(line1: u64, line2: u64) -> (u64, u64) {
        if line1 <= line2 {
            (line1, line2)
        } else {
            (line2, line1)
        }
    }

    /// Attempt to form a valid block pair from pending block markers.
    ///
    /// Returns `Ok(BlockPair)` if exactly two matching markers exist.
    /// Returns `Err` if 0, 1, or >2 markers exist for a block type.
    pub fn validate_pair(
        pending: &PendingCommandStore,
        block_kind: BlockCommandKind,
    ) -> Result<BlockPair, LineCommandError> {
        let line_cmd_kind = Self::block_kind_to_line_cmd_kind(block_kind);
        let markers = pending.pending_blocks(&line_cmd_kind);

        match markers.len() {
            0 | 1 => Err(LineCommandError::AwaitingPair {
                kind: block_kind.to_string(),
            }),
            2 => {
                let line1 = markers[0].command.line;
                let line2 = markers[1].command.line;
                let (start_line, end_line) = Self::normalize(line1, line2);
                Ok(BlockPair {
                    kind: block_kind,
                    start_line,
                    end_line,
                })
            }
            _ => Err(LineCommandError::TooManyMarkers {
                kind: block_kind.to_string(),
            }),
        }
    }

    /// Check for overlapping block ranges across different block types.
    pub fn check_overlaps(pairs: &[BlockPair]) -> Result<(), LineCommandError> {
        for i in 0..pairs.len() {
            for j in (i + 1)..pairs.len() {
                if Self::ranges_overlap(
                    pairs[i].start_line,
                    pairs[i].end_line,
                    pairs[j].start_line,
                    pairs[j].end_line,
                ) {
                    return Err(LineCommandError::OverlappingBlocks {
                        kind1: pairs[i].kind.to_string(),
                        kind2: pairs[j].kind.to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn ranges_overlap(start1: u64, end1: u64, start2: u64, end2: u64) -> bool {
        start1 <= end2 && start2 <= end1
    }

    fn block_kind_to_line_cmd_kind(kind: BlockCommandKind) -> LineCommandKind {
        match kind {
            BlockCommandKind::Delete => LineCommandKind::DeleteBlock,
            BlockCommandKind::Repeat => LineCommandKind::RepeatBlock,
            BlockCommandKind::Exclude => LineCommandKind::ExcludeBlock,
            BlockCommandKind::Tag => LineCommandKind::TagBlock,
            BlockCommandKind::Untag => LineCommandKind::UntagBlock,
            BlockCommandKind::ShiftRight => LineCommandKind::ShiftRightBlock,
            BlockCommandKind::ShiftLeft => LineCommandKind::ShiftLeftBlock,
            BlockCommandKind::BoundsRight => LineCommandKind::BoundsShiftRightBlock,
            BlockCommandKind::BoundsLeft => LineCommandKind::BoundsShiftLeftBlock,
            BlockCommandKind::Copy => LineCommandKind::CopyBlock,
            BlockCommandKind::Move => LineCommandKind::MoveBlock,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::ParsedLineCommand;
    use crate::pending::PendingReason;

    fn add_block(store: &mut PendingCommandStore, line: u64, kind: LineCommandKind) {
        store.add(
            ParsedLineCommand { line, kind },
            PendingReason::AwaitingPair,
        );
    }

    #[test]
    fn normalize_already_ordered() {
        assert_eq!(BlockPairValidator::normalize(3, 7), (3, 7));
    }

    #[test]
    fn normalize_reversed_order() {
        assert_eq!(BlockPairValidator::normalize(10, 2), (2, 10));
    }

    #[test]
    fn normalize_same_line() {
        assert_eq!(BlockPairValidator::normalize(5, 5), (5, 5));
    }

    #[test]
    fn validate_pair_with_two_markers_succeeds() {
        let mut store = PendingCommandStore::new();
        add_block(&mut store, 3, LineCommandKind::DeleteBlock);
        add_block(&mut store, 7, LineCommandKind::DeleteBlock);

        let pair = BlockPairValidator::validate_pair(&store, BlockCommandKind::Delete).unwrap();
        assert_eq!(pair.kind, BlockCommandKind::Delete);
        assert_eq!(pair.start_line, 3);
        assert_eq!(pair.end_line, 7);
    }

    #[test]
    fn validate_pair_normalizes_reversed_markers() {
        let mut store = PendingCommandStore::new();
        add_block(&mut store, 10, LineCommandKind::DeleteBlock);
        add_block(&mut store, 2, LineCommandKind::DeleteBlock);

        let pair = BlockPairValidator::validate_pair(&store, BlockCommandKind::Delete).unwrap();
        assert_eq!(pair.start_line, 2);
        assert_eq!(pair.end_line, 10);
    }

    #[test]
    fn validate_pair_single_marker_returns_awaiting_pair() {
        let mut store = PendingCommandStore::new();
        add_block(&mut store, 5, LineCommandKind::DeleteBlock);

        let result = BlockPairValidator::validate_pair(&store, BlockCommandKind::Delete);
        assert!(matches!(result, Err(LineCommandError::AwaitingPair { .. })));
    }

    #[test]
    fn validate_pair_no_markers_returns_awaiting_pair() {
        let store = PendingCommandStore::new();
        let result = BlockPairValidator::validate_pair(&store, BlockCommandKind::Delete);
        assert!(matches!(result, Err(LineCommandError::AwaitingPair { .. })));
    }

    #[test]
    fn validate_pair_too_many_markers_returns_error() {
        let mut store = PendingCommandStore::new();
        add_block(&mut store, 1, LineCommandKind::DeleteBlock);
        add_block(&mut store, 3, LineCommandKind::DeleteBlock);
        add_block(&mut store, 5, LineCommandKind::DeleteBlock);

        let result = BlockPairValidator::validate_pair(&store, BlockCommandKind::Delete);
        assert!(matches!(
            result,
            Err(LineCommandError::TooManyMarkers { .. })
        ));
    }

    #[test]
    fn check_overlaps_no_overlap_succeeds() {
        let pairs = vec![
            BlockPair {
                kind: BlockCommandKind::Delete,
                start_line: 1,
                end_line: 5,
            },
            BlockPair {
                kind: BlockCommandKind::ShiftRight,
                start_line: 6,
                end_line: 10,
            },
        ];
        assert!(BlockPairValidator::check_overlaps(&pairs).is_ok());
    }

    #[test]
    fn check_overlaps_with_overlap_returns_error() {
        let pairs = vec![
            BlockPair {
                kind: BlockCommandKind::Delete,
                start_line: 1,
                end_line: 5,
            },
            BlockPair {
                kind: BlockCommandKind::ShiftRight,
                start_line: 3,
                end_line: 8,
            },
        ];
        let result = BlockPairValidator::check_overlaps(&pairs);
        assert!(matches!(
            result,
            Err(LineCommandError::OverlappingBlocks { .. })
        ));
    }
}
