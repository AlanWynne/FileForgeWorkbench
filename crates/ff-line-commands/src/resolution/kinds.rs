//! Per-kind resolution helpers for `ResolutionEngine`.
//!
//! Immediate commands, block pairs, and source+target resolution.

use crate::block_pair::BlockPairValidator;
use crate::command::{
    BlockCommandKind, ExecutableCommand, LineCommandKind, SourceOperation, SourceTarget,
    TargetPosition,
};
use crate::config::LineCommandConfig;
use crate::error::LineCommandError;
use crate::pending::PendingCommandStore;

use super::ResolutionEngine;

impl ResolutionEngine {
    pub(super) fn resolve_immediate(
        kind: &LineCommandKind,
        line: u64,
        config: &LineCommandConfig,
    ) -> Option<ExecutableCommand> {
        match kind {
            LineCommandKind::Delete => Some(ExecutableCommand::Delete {
                start_line: line,
                count: 1,
            }),
            LineCommandKind::DeleteCount(n) => Some(ExecutableCommand::Delete {
                start_line: line,
                count: u64::from(*n),
            }),
            LineCommandKind::Insert => Some(ExecutableCommand::Insert {
                after_line: line,
                count: 1,
            }),
            LineCommandKind::InsertCount(n) => Some(ExecutableCommand::Insert {
                after_line: line,
                count: *n,
            }),
            LineCommandKind::Repeat => Some(ExecutableCommand::Repeat {
                start_line: line,
                count: 1,
            }),
            LineCommandKind::RepeatCount(n) => Some(ExecutableCommand::Repeat {
                start_line: line,
                count: *n,
            }),
            LineCommandKind::Exclude => Some(ExecutableCommand::Exclude {
                start_line: line,
                count: 1,
            }),
            LineCommandKind::ExcludeCount(n) => Some(ExecutableCommand::Exclude {
                start_line: line,
                count: u64::from(*n),
            }),
            LineCommandKind::Tag => Some(ExecutableCommand::Tag {
                start_line: line,
                end_line: line,
            }),
            LineCommandKind::Untag => Some(ExecutableCommand::Untag {
                start_line: line,
                end_line: line,
            }),
            LineCommandKind::ShiftRight => Some(ExecutableCommand::ShiftRight {
                start_line: line,
                end_line: line,
                columns: config.shift_width,
            }),
            LineCommandKind::ShiftRightCount(n) => Some(ExecutableCommand::ShiftRight {
                start_line: line,
                end_line: line,
                columns: *n,
            }),
            LineCommandKind::ShiftLeft => Some(ExecutableCommand::ShiftLeft {
                start_line: line,
                end_line: line,
                columns: config.shift_width,
            }),
            LineCommandKind::ShiftLeftCount(n) => Some(ExecutableCommand::ShiftLeft {
                start_line: line,
                end_line: line,
                columns: *n,
            }),
            LineCommandKind::BoundsShiftRight => Some(ExecutableCommand::BoundsShiftRight {
                start_line: line,
                end_line: line,
            }),
            LineCommandKind::BoundsShiftLeft => Some(ExecutableCommand::BoundsShiftLeft {
                start_line: line,
                end_line: line,
            }),
            LineCommandKind::ClipboardCopy => Some(ExecutableCommand::ClipboardCopy {
                start_line: line,
                end_line: line,
            }),
            LineCommandKind::ShowFirst => Some(ExecutableCommand::ShowFirst {
                block_start: line,
                block_end: line,
            }),
            LineCommandKind::ShowLast => Some(ExecutableCommand::ShowLast {
                block_start: line,
                block_end: line,
            }),
            LineCommandKind::ShowLine => Some(ExecutableCommand::ShowLine {
                block_start: line,
                block_end: line,
            }),
            LineCommandKind::ShiftRightOne => Some(ExecutableCommand::ShiftRight {
                start_line: line,
                end_line: line,
                columns: 1,
            }),
            _ => None,
        }
    }

    pub(super) fn resolve_block_pair(
        pair: &crate::command::BlockPair,
        config: &LineCommandConfig,
    ) -> Option<ExecutableCommand> {
        match pair.kind {
            BlockCommandKind::Delete => Some(ExecutableCommand::Delete {
                start_line: pair.start_line,
                count: pair.end_line - pair.start_line + 1,
            }),
            BlockCommandKind::Repeat => Some(ExecutableCommand::RepeatBlock {
                start_line: pair.start_line,
                end_line: pair.end_line,
            }),
            BlockCommandKind::Exclude => Some(ExecutableCommand::Exclude {
                start_line: pair.start_line,
                count: pair.end_line - pair.start_line + 1,
            }),
            BlockCommandKind::Tag => Some(ExecutableCommand::Tag {
                start_line: pair.start_line,
                end_line: pair.end_line,
            }),
            BlockCommandKind::Untag => Some(ExecutableCommand::Untag {
                start_line: pair.start_line,
                end_line: pair.end_line,
            }),
            BlockCommandKind::ShiftRight => Some(ExecutableCommand::ShiftRight {
                start_line: pair.start_line,
                end_line: pair.end_line,
                columns: config.shift_width,
            }),
            BlockCommandKind::ShiftLeft => Some(ExecutableCommand::ShiftLeft {
                start_line: pair.start_line,
                end_line: pair.end_line,
                columns: config.shift_width,
            }),
            BlockCommandKind::BoundsRight => Some(ExecutableCommand::BoundsShiftRight {
                start_line: pair.start_line,
                end_line: pair.end_line,
            }),
            BlockCommandKind::BoundsLeft => Some(ExecutableCommand::BoundsShiftLeft {
                start_line: pair.start_line,
                end_line: pair.end_line,
            }),
            // Copy/Move blocks are handled via source+target resolution
            BlockCommandKind::Copy | BlockCommandKind::Move => None,
            // ClipboardCopy block (WW) -- collect text, no document mutation
            BlockCommandKind::ClipboardCopy => Some(ExecutableCommand::ClipboardCopy {
                start_line: pair.start_line,
                end_line: pair.end_line,
            }),
            // ShiftRightOne block (]]) -- shift all lines by 1 column
            BlockCommandKind::ShiftRightOne => Some(ExecutableCommand::ShiftRight {
                start_line: pair.start_line,
                end_line: pair.end_line,
                columns: 1,
            }),
        }
    }

    pub(super) fn resolve_source_target(
        pending: &mut PendingCommandStore,
        executable: &mut Vec<ExecutableCommand>,
        errors: &mut Vec<LineCommandError>,
    ) {
        let targets = pending.pending_targets();
        if targets.is_empty() {
            return;
        }

        // Check for duplicate targets
        if targets.len() > 1 {
            errors.push(LineCommandError::DuplicateTarget);
            return;
        }

        let target_cmd = targets[0].clone();
        let target_line = target_cmd.command.line;
        let target_position = match target_cmd.command.kind {
            LineCommandKind::After => TargetPosition::After,
            LineCommandKind::Before => TargetPosition::Before,
            _ => return,
        };

        let sources = pending.pending_sources();
        if sources.is_empty() {
            // Target with no source stays pending
            return;
        }

        // Determine the source operation and range
        let first_source = sources[0].clone();
        let operation = match first_source.command.kind {
            LineCommandKind::Copy => SourceOperation::Copy,
            LineCommandKind::CopyBlock => SourceOperation::Copy,
            LineCommandKind::Move => SourceOperation::Move,
            LineCommandKind::MoveBlock => SourceOperation::Move,
            _ => return,
        };

        // Check if it's a block source (CC or MM)
        let is_block = matches!(
            first_source.command.kind,
            LineCommandKind::CopyBlock | LineCommandKind::MoveBlock
        );

        let (source_start, source_end, source_lines_to_remove) = if is_block {
            // Need exactly two block markers
            let block_kind = match operation {
                SourceOperation::Copy => BlockCommandKind::Copy,
                SourceOperation::Move => BlockCommandKind::Move,
            };
            match BlockPairValidator::validate_pair(pending, block_kind) {
                Ok(pair) => {
                    let line_cmd_kind = Self::block_kind_to_line_cmd_kind(block_kind);
                    let markers: Vec<u64> = pending
                        .pending_blocks(&line_cmd_kind)
                        .iter()
                        .map(|pc| pc.command.line)
                        .collect();
                    (pair.start_line, pair.end_line, markers)
                }
                Err(_) => {
                    // Block pair not complete yet -- leave pending
                    return;
                }
            }
        } else {
            // Single line source
            (
                first_source.command.line,
                first_source.command.line,
                vec![first_source.command.line],
            )
        };

        let source_target = SourceTarget {
            operation,
            source_start,
            source_end,
            target_line,
            target_position,
        };

        let cmd = match operation {
            SourceOperation::Copy => ExecutableCommand::CopyToTarget(source_target),
            SourceOperation::Move => ExecutableCommand::MoveToTarget(source_target),
        };

        executable.push(cmd);

        // Remove resolved markers
        for line in source_lines_to_remove {
            pending.remove(line);
        }
        pending.remove(target_line);
    }

    pub(super) fn block_kind_to_line_cmd_kind(kind: BlockCommandKind) -> LineCommandKind {
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
            BlockCommandKind::ClipboardCopy => LineCommandKind::ClipboardCopyBlock,
            BlockCommandKind::ShiftRightOne => LineCommandKind::ShiftRightOneBlock,
        }
    }
}
