//! Shared ISPF Table Editor -- the PURE, egui-free BLOCK line-command pass
//! (CR-CH-076, ispf-table-editor Requirement 9).
//!
//! Split out of [`super::line_commands`] to keep each file under the 400-line
//! rule (rust-standards.md). This is the DOUBLED-marker (`RR` / `DD` / `CC` /
//! `MM`) path: a host whose [`LineCommandSet::blocks`] is set recognises the
//! block forms typed into the Act cells of TWO rows and treats them as the
//! inclusive, order-independent endpoints of a range (Requirement 9.1).
//!
//! It REUSES the `ff-line-commands` engine's EXISTING block model with NO
//! public-API change (Requirement 9.6): [`PendingCommandStore`] accumulates the
//! parsed `*Block` markers, [`BlockPairValidator::validate_pair`] forms the
//! inclusive order-normalised [`BlockPair`] (returning `AwaitingPair` /
//! `TooManyMarkers` for a lone / extra marker), and
//! [`BlockPairValidator::check_overlaps`] guards overlapping ranges.
//!
//! A malformed block (unterminated, too many markers, or overlapping) makes NO
//! structural change for THAT block and surfaces a non-blocking notice, with the
//! offending Act cells RETAINED so the user can supply the partner on the next
//! cycle; the rest of the batch still applies (Requirement 9.4).

use ff_line_commands::block_pair::BlockPairValidator;
use ff_line_commands::command::{to_block_kind, BlockCommandKind, BlockPair, LineCommandKind};
use ff_line_commands::pending::{PendingCommandStore, PendingReason};
use ff_line_commands::ParsedLineCommand;

use super::line_commands::ApplyResult;
use super::state::{IspfTableHost, LineCommandSet};

/// One collected block marker, keyed by its BACKING index into the full row list.
pub(super) struct BlockMarker {
    /// Backing index of the marked row (used as the engine's `line`).
    pub index: usize,
    /// The parsed block kind (`DeleteBlock` / `RepeatBlock` / `CopyBlock` /
    /// `MoveBlock`).
    pub kind: LineCommandKind,
}

/// The structural plan a resolved block pair contributes to the rebuild, in
/// terms of STABLE original indices (mirrors the single-command before/after/
/// removed model in [`super::line_commands`]).
pub(super) struct BlockPlan {
    /// Rows to REMOVE (a delete block, or the source range of a move block).
    pub removed: Vec<usize>,
    /// `(target_index, before, rows_to_insert)` groups: the rows cloned/moved
    /// into place either BEFORE (`before = true`) or AFTER the target index.
    pub inserts: Vec<BlockInsert>,
    /// Source Act cells to clear (the marked endpoints of a consumed block).
    pub clear: Vec<usize>,
}

/// A group of rows to insert at a target position (before/after a target index).
pub(super) struct BlockInsert {
    pub target: usize,
    pub before: bool,
    pub rows: Vec<usize>,
}

/// Resolve the collected block markers into a [`BlockPlan`] over `rows`, pairing
/// via the `ff-line-commands` engine and honouring the host's enabled set.
///
/// `targets` is the shared single-command target list `(index, kind)` for A/B
/// markers; a `CopyBlock` / `MoveBlock` consumes the first unconsumed target the
/// same way a single `C` / `M` does (Requirement 9.3). `consumed_targets` is the
/// set of target indices already taken by single-command copy/move sources; this
/// pass appends the indices it consumes so the caller clears them.
///
/// Markers of a block kind that fails to pair (lone / too many) are RETAINED
/// (their indices are NOT added to `plan.clear`) and a non-blocking notice is
/// pushed (Requirement 9.4). Overlapping resolved pairs are dropped with a
/// notice and their markers retained (Requirement 9.4, 9.6). A `MoveBlock` when
/// `set.move_` is false is dropped with a "not enabled" notice (Requirement 7.12).
///
/// Validates: ispf-table-editor Requirement 9.1, 9.2, 9.3, 9.4, 9.6
pub(super) fn resolve_blocks<H: IspfTableHost>(
    rows: &[H::Row],
    set: &LineCommandSet,
    markers: &[BlockMarker],
    targets: &[(usize, LineCommandKind)],
    consumed_targets: &mut Vec<usize>,
    result: &mut ApplyResult,
) -> BlockPlan {
    let mut plan = BlockPlan {
        removed: Vec::new(),
        inserts: Vec::new(),
        clear: Vec::new(),
    };
    if markers.is_empty() {
        return plan;
    }

    // Accumulate every block marker into the engine's pending store, keyed by
    // backing index as the `line` (Requirement 9.6).
    let mut store = PendingCommandStore::new();
    for m in markers {
        store.add(
            ParsedLineCommand {
                line: m.index as u64,
                kind: m.kind.clone(),
            },
            PendingReason::AwaitingPair,
        );
    }

    // Pair each DISTINCT block kind present. A kind that fails to pair is a
    // "block not closed" (lone) or "too many markers" notice; its markers are
    // retained (not cleared) so the user can fix them next cycle (Req 9.4).
    let mut resolved: Vec<(BlockCommandKind, BlockPair)> = Vec::new();
    let mut seen: Vec<BlockCommandKind> = Vec::new();
    for m in markers {
        let Some(bk) = to_block_kind(&m.kind) else {
            continue;
        };
        if seen.contains(&bk) {
            continue;
        }
        seen.push(bk);
        match BlockPairValidator::validate_pair(&store, bk) {
            Ok(pair) => resolved.push((bk, pair)),
            Err(_) => {
                result.push(format!("block {bk} not closed (needs a matching pair)"));
            }
        }
    }

    // Guard overlapping ranges across the resolved pairs: on overlap drop EVERY
    // resolved pair (markers retained) with a notice (Req 9.4, 9.6).
    let pairs_only: Vec<BlockPair> = resolved.iter().map(|(_, p)| p.clone()).collect();
    if BlockPairValidator::check_overlaps(&pairs_only).is_err() {
        result.push("overlapping block ranges rejected".to_string());
        return plan;
    }

    // Translate each resolved pair into the structural plan.
    for (bk, pair) in &resolved {
        let start = pair.start_line as usize;
        let end = pair.end_line as usize;
        let range: Vec<usize> = (start..=end).filter(|i| *i < rows.len()).collect();
        match bk {
            BlockCommandKind::Delete => {
                plan.removed.extend(range.iter().copied());
                plan.clear.extend(range.iter().copied());
            }
            BlockCommandKind::Repeat => {
                // Clone every row in the range into its own after[i] slot so the
                // duplicated block lands immediately after the block.
                plan.inserts.push(BlockInsert {
                    target: end,
                    before: false,
                    rows: range.clone(),
                });
                plan.clear.extend(range.iter().copied());
            }
            BlockCommandKind::Copy | BlockCommandKind::Move => {
                let is_move = matches!(bk, BlockCommandKind::Move);
                if is_move && !set.move_ {
                    // MM is gated on the host's `move_` flag (Alias: off) -- drop
                    // it with a not-enabled notice and retain the markers.
                    result.push("ignored block line command 'MM' (not enabled)".to_string());
                    continue;
                }
                // Consume the first unconsumed A/B target, exactly as a single
                // C / M does (Requirement 9.3).
                let Some(&(t_index, ref t_kind)) = targets
                    .iter()
                    .find(|(ti, _)| !consumed_targets.contains(ti))
                else {
                    result.push("copy/move block source pending a target (A/B)".to_string());
                    continue;
                };
                consumed_targets.push(t_index);
                let before = matches!(t_kind, LineCommandKind::Before);
                plan.inserts.push(BlockInsert {
                    target: t_index,
                    before,
                    rows: range.clone(),
                });
                plan.clear.extend(range.iter().copied());
                // Clear the consumed A/B target's Act cell too (as a single
                // C / M does for its target).
                plan.clear.push(t_index);
                if is_move {
                    plan.removed.extend(range.iter().copied());
                }
            }
            _ => {
                // No other block kind is enabled by the table editor set
                // (Requirement 9.1 restricts the block set to RR/DD/CC/MM).
            }
        }
    }

    plan
}
