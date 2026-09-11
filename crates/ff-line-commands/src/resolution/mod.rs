//! Resolution engine -- determines which pending line commands can be executed.
//!
//! Resolves immediate commands, block pairs, and source+target combinations.

use crate::block_pair::BlockPairValidator;
use crate::command::{classify, BlockCommandKind, ExecutableCommand, LineCommandCategory};
use crate::compatibility::CommandCompatibilityMatrix;
use crate::config::LineCommandConfig;
use crate::error::LineCommandError;
use crate::parser::LineCommandParser;
use crate::pending::{PendingCommand, PendingCommandStore, PendingReason};

/// The outcome of resolution: either executable operations or errors.
#[derive(Debug)]
pub struct ResolutionResult {
    /// Commands ready to execute.
    pub executable: Vec<ExecutableCommand>,
    /// Errors encountered during resolution.
    pub errors: Vec<LineCommandError>,
    /// Commands that remain pending after this cycle.
    pub still_pending: Vec<PendingCommand>,
}

/// Determines which pending line commands can be executed in the current command cycle.
pub struct ResolutionEngine;

impl ResolutionEngine {
    /// Process all newly entered line commands and existing pending commands.
    ///
    /// # Resolution Order
    /// 1. Parse new prefix-area inputs -> add to pending store or reject
    /// 2. Validate block pairs -> form BlockPair if two markers present
    /// 3. Check source+target resolution -> form SourceTarget if both present
    /// 4. Verify compatibility with primary command (if any)
    /// 5. Return resolved operations for execution
    pub fn resolve(
        new_inputs: &[(u64, String)],
        pending: &mut PendingCommandStore,
        primary_command: Option<&str>,
        config: &LineCommandConfig,
    ) -> ResolutionResult {
        let mut errors = Vec::new();
        let mut executable = Vec::new();

        // Step 1: Parse new inputs and add to pending store
        for (line, input) in new_inputs {
            match LineCommandParser::parse(input, *line) {
                Ok(parsed) => {
                    let category = classify(&parsed.kind);
                    let reason = match category {
                        LineCommandCategory::Immediate => PendingReason::AwaitingPair, // placeholder
                        LineCommandCategory::Block => PendingReason::AwaitingPair,
                        LineCommandCategory::Source => PendingReason::AwaitingTarget,
                        LineCommandCategory::Target => PendingReason::AwaitingSource,
                    };
                    pending.add(parsed, reason);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        // Step 4: Compatibility check
        if let Err(e) = CommandCompatibilityMatrix::check_compatibility(primary_command, pending) {
            errors.push(e);
            let still_pending = pending.all_pending().map(|(_, pc)| pc.clone()).collect();
            return ResolutionResult {
                executable,
                errors,
                still_pending,
            };
        }

        // Step 2 + 5: Resolve immediate commands
        let mut lines_to_remove = Vec::new();
        for (line, pc) in pending.all_pending() {
            let category = classify(&pc.command.kind);
            if category == LineCommandCategory::Immediate {
                if let Some(cmd) = Self::resolve_immediate(&pc.command.kind, *line, config) {
                    executable.push(cmd);
                    lines_to_remove.push(*line);
                }
            }
        }

        for line in &lines_to_remove {
            pending.remove(*line);
        }

        // Step 2: Resolve block pairs
        let block_kinds = [
            BlockCommandKind::Delete,
            BlockCommandKind::Repeat,
            BlockCommandKind::Exclude,
            BlockCommandKind::Tag,
            BlockCommandKind::Untag,
            BlockCommandKind::ShiftRight,
            BlockCommandKind::ShiftLeft,
            BlockCommandKind::BoundsRight,
            BlockCommandKind::BoundsLeft,
            BlockCommandKind::ClipboardCopy,
            BlockCommandKind::ShiftRightOne,
        ];

        for block_kind in &block_kinds {
            match BlockPairValidator::validate_pair(pending, *block_kind) {
                Ok(pair) => {
                    if let Some(cmd) = Self::resolve_block_pair(&pair, config) {
                        executable.push(cmd);
                        // Remove the two markers from pending
                        let line_cmd_kind = Self::block_kind_to_line_cmd_kind(*block_kind);
                        let markers: Vec<u64> = pending
                            .pending_blocks(&line_cmd_kind)
                            .iter()
                            .map(|pc| pc.command.line)
                            .collect();
                        for line in markers {
                            pending.remove(line);
                        }
                    }
                }
                Err(LineCommandError::AwaitingPair { .. }) => {
                    // Single marker stays pending, no error for user
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        // Step 3: Resolve source + target
        Self::resolve_source_target(pending, &mut executable, &mut errors);

        let still_pending = pending.all_pending().map(|(_, pc)| pc.clone()).collect();

        ResolutionResult {
            executable,
            errors,
            still_pending,
        }
    }
}

mod kinds;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{
        LineCommandKind, ParsedLineCommand, SourceOperation, SourceTarget, TargetPosition,
    };

    fn default_config() -> LineCommandConfig {
        LineCommandConfig::default()
    }

    #[test]
    fn resolve_immediate_delete_command() {
        let mut pending = PendingCommandStore::new();
        let config = default_config();
        let result =
            ResolutionEngine::resolve(&[(5, "D3".to_string())], &mut pending, None, &config);
        assert_eq!(result.executable.len(), 1);
        assert!(matches!(
            result.executable[0],
            ExecutableCommand::Delete {
                start_line: 5,
                count: 3
            }
        ));
        assert!(result.errors.is_empty());
    }

    #[test]
    fn resolve_block_pair_delete() {
        let mut pending = PendingCommandStore::new();
        let config = default_config();
        let result = ResolutionEngine::resolve(
            &[(2, "DD".to_string()), (5, "DD".to_string())],
            &mut pending,
            None,
            &config,
        );
        assert!(result.executable.iter().any(|cmd| matches!(
            cmd,
            ExecutableCommand::Delete {
                start_line: 2,
                count: 4
            }
        )));
    }

    #[test]
    fn resolve_single_block_marker_stays_pending() {
        let mut pending = PendingCommandStore::new();
        let config = default_config();
        let result =
            ResolutionEngine::resolve(&[(3, "DD".to_string())], &mut pending, None, &config);
        assert!(result.executable.is_empty());
        assert!(!pending.is_empty());
    }

    #[test]
    fn resolve_copy_source_and_target() {
        let mut pending = PendingCommandStore::new();
        let config = default_config();
        let result = ResolutionEngine::resolve(
            &[(2, "C".to_string()), (8, "A".to_string())],
            &mut pending,
            None,
            &config,
        );
        assert!(result.executable.iter().any(|cmd| matches!(
            cmd,
            ExecutableCommand::CopyToTarget(SourceTarget {
                operation: SourceOperation::Copy,
                source_start: 2,
                source_end: 2,
                target_line: 8,
                target_position: TargetPosition::After,
            })
        )));
    }

    #[test]
    fn resolve_invalid_input_produces_error() {
        let mut pending = PendingCommandStore::new();
        let config = default_config();
        let result =
            ResolutionEngine::resolve(&[(0, "ZZZ".to_string())], &mut pending, None, &config);
        assert!(!result.errors.is_empty());
        assert!(result.executable.is_empty());
    }

    #[test]
    fn resolve_no_inputs_no_primary_pending_source_stays_pending() {
        let mut pending = PendingCommandStore::new();
        pending.add(
            ParsedLineCommand {
                line: 3,
                kind: LineCommandKind::Copy,
            },
            PendingReason::AwaitingTarget,
        );
        let config = default_config();
        let result = ResolutionEngine::resolve(&[], &mut pending, None, &config);
        assert!(result.executable.is_empty());
        assert!(!pending.is_empty());
    }

    #[test]
    fn resolve_shift_right_uses_config_width() {
        let mut pending = PendingCommandStore::new();
        let config = LineCommandConfig::new(4);
        let result =
            ResolutionEngine::resolve(&[(0, ">".to_string())], &mut pending, None, &config);
        assert!(matches!(
            &result.executable[0],
            ExecutableCommand::ShiftRight { columns: 4, .. }
        ));
    }
}
