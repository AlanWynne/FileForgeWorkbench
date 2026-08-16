//! Command compatibility matrix — validates primary command + line command combinations.

use crate::command::{classify, LineCommandCategory};
use crate::error::LineCommandError;
use crate::pending::PendingCommandStore;

/// Defines which primary commands are compatible with which line commands.
pub struct CommandCompatibilityMatrix;

impl CommandCompatibilityMatrix {
    /// Check if a primary command is compatible with the set of pending line commands.
    ///
    /// Returns `Ok(())` if compatible, `Err` with description if not.
    pub fn check_compatibility(
        primary_command: Option<&str>,
        pending: &PendingCommandStore,
    ) -> Result<(), LineCommandError> {
        if pending.is_empty() {
            return Ok(());
        }

        let primary = match primary_command {
            Some(cmd) => cmd.to_uppercase(),
            None => {
                // No primary command — only immediate commands may execute
                if Self::all_immediate(pending) {
                    return Ok(());
                }
                // Non-immediate commands pending without a primary command is valid
                // (they stay pending)
                return Ok(());
            }
        };

        // Check for file-path COPY/MOVE incompatibility with source markers
        if (primary.starts_with("COPY ") || primary.starts_with("MOVE "))
            && !pending.pending_sources().is_empty()
        {
            return Err(LineCommandError::SourceWithFilePath);
        }

        // COPY primary is compatible with C/CC + A/B
        if primary == "COPY" {
            let sources = pending.pending_sources();
            let has_copy_sources = sources.iter().any(|s| {
                matches!(
                    s.command.kind,
                    crate::command::LineCommandKind::Copy
                        | crate::command::LineCommandKind::CopyBlock
                )
            });
            let has_move_sources = sources.iter().any(|s| {
                matches!(
                    s.command.kind,
                    crate::command::LineCommandKind::Move
                        | crate::command::LineCommandKind::MoveBlock
                )
            });
            if has_move_sources {
                return Err(LineCommandError::IncompatibleCommands {
                    primary: "COPY".to_string(),
                    line_cmd: "M/MM".to_string(),
                });
            }
            if has_copy_sources {
                return Ok(());
            }
        }

        // MOVE primary is compatible with M/MM + A/B
        if primary == "MOVE" {
            let sources = pending.pending_sources();
            let has_copy_sources = sources.iter().any(|s| {
                matches!(
                    s.command.kind,
                    crate::command::LineCommandKind::Copy
                        | crate::command::LineCommandKind::CopyBlock
                )
            });
            let has_move_sources = sources.iter().any(|s| {
                matches!(
                    s.command.kind,
                    crate::command::LineCommandKind::Move
                        | crate::command::LineCommandKind::MoveBlock
                )
            });
            if has_copy_sources {
                return Err(LineCommandError::IncompatibleCommands {
                    primary: "MOVE".to_string(),
                    line_cmd: "C/CC".to_string(),
                });
            }
            if has_move_sources {
                return Ok(());
            }
        }

        Ok(())
    }

    /// Returns true if all pending commands are immediate commands.
    pub fn all_immediate(pending: &PendingCommandStore) -> bool {
        pending
            .all_pending()
            .all(|(_, pc)| classify(&pc.command.kind) == LineCommandCategory::Immediate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{LineCommandKind, ParsedLineCommand};
    use crate::pending::PendingReason;

    fn make_store_with(entries: Vec<(u64, LineCommandKind, PendingReason)>) -> PendingCommandStore {
        let mut store = PendingCommandStore::new();
        for (line, kind, reason) in entries {
            store.add(ParsedLineCommand { line, kind }, reason);
        }
        store
    }

    #[test]
    fn empty_pending_is_always_compatible() {
        let store = PendingCommandStore::new();
        assert!(CommandCompatibilityMatrix::check_compatibility(Some("COPY"), &store).is_ok());
    }

    #[test]
    fn copy_primary_with_copy_sources_is_compatible() {
        let store = make_store_with(vec![
            (0, LineCommandKind::Copy, PendingReason::AwaitingTarget),
            (5, LineCommandKind::After, PendingReason::AwaitingSource),
        ]);
        assert!(CommandCompatibilityMatrix::check_compatibility(Some("COPY"), &store).is_ok());
    }

    #[test]
    fn copy_primary_with_move_sources_is_incompatible() {
        let store = make_store_with(vec![(
            0,
            LineCommandKind::Move,
            PendingReason::AwaitingTarget,
        )]);
        let result = CommandCompatibilityMatrix::check_compatibility(Some("COPY"), &store);
        assert!(matches!(
            result,
            Err(LineCommandError::IncompatibleCommands { .. })
        ));
    }

    #[test]
    fn move_primary_with_copy_sources_is_incompatible() {
        let store = make_store_with(vec![(
            0,
            LineCommandKind::Copy,
            PendingReason::AwaitingTarget,
        )]);
        let result = CommandCompatibilityMatrix::check_compatibility(Some("MOVE"), &store);
        assert!(matches!(
            result,
            Err(LineCommandError::IncompatibleCommands { .. })
        ));
    }

    #[test]
    fn copy_path_with_source_markers_is_incompatible() {
        let store = make_store_with(vec![(
            0,
            LineCommandKind::Copy,
            PendingReason::AwaitingTarget,
        )]);
        let result =
            CommandCompatibilityMatrix::check_compatibility(Some("COPY /path/file.txt"), &store);
        assert!(matches!(result, Err(LineCommandError::SourceWithFilePath)));
    }

    #[test]
    fn no_primary_with_immediate_commands_is_compatible() {
        let store = make_store_with(vec![
            (0, LineCommandKind::Delete, PendingReason::AwaitingPair),
            (1, LineCommandKind::ShiftRight, PendingReason::AwaitingPair),
        ]);
        assert!(CommandCompatibilityMatrix::check_compatibility(None, &store).is_ok());
    }

    #[test]
    fn all_immediate_returns_true_for_immediate_commands() {
        let store = make_store_with(vec![
            (0, LineCommandKind::Delete, PendingReason::AwaitingPair),
            (1, LineCommandKind::Insert, PendingReason::AwaitingPair),
        ]);
        assert!(CommandCompatibilityMatrix::all_immediate(&store));
    }

    #[test]
    fn all_immediate_returns_false_when_source_present() {
        let store = make_store_with(vec![(
            0,
            LineCommandKind::Copy,
            PendingReason::AwaitingTarget,
        )]);
        assert!(!CommandCompatibilityMatrix::all_immediate(&store));
    }
}
