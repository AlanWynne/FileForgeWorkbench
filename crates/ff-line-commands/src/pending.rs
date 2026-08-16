//! Pending command store — per-session storage for unresolved line commands.
//!
//! Provides query, insertion, removal, and reset operations.

use std::collections::HashMap;

use crate::command::{classify, LineCommandCategory, LineCommandKind, ParsedLineCommand};

/// Reason a command is pending (for display in prefix area and status).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingReason {
    /// Block marker waiting for its matching pair.
    AwaitingPair,
    /// Source marker (C/CC/M/MM) waiting for a target (A/B).
    AwaitingTarget,
    /// Target marker (A/B) waiting for a source (C/CC/M/MM).
    AwaitingSource,
    /// Invalid command text retained for user correction.
    InvalidCommand(String),
}

/// A line command that has been entered but not yet resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCommand {
    /// The parsed command.
    pub command: ParsedLineCommand,
    /// A message describing why this command is pending.
    pub reason: PendingReason,
    /// Timestamp when the command was entered (monotonic, for ordering).
    pub entered_at: u64,
}

/// Per-session storage for unresolved line commands.
///
/// Provides query, insertion, removal, and reset operations.
pub struct PendingCommandStore {
    /// All pending commands, indexed by line number for O(1) lookup.
    commands: HashMap<u64, PendingCommand>,
    /// Monotonically increasing counter for ordering.
    next_id: u64,
}

impl PendingCommandStore {
    /// Creates a new empty pending command store.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a pending command for a line.
    pub fn add(&mut self, command: ParsedLineCommand, reason: PendingReason) {
        let entered_at = self.next_id;
        self.next_id += 1;
        let line = command.line;
        self.commands.insert(
            line,
            PendingCommand {
                command,
                reason,
                entered_at,
            },
        );
    }

    /// Remove a pending command at a line, return it if it existed.
    pub fn remove(&mut self, line: u64) -> Option<PendingCommand> {
        self.commands.remove(&line)
    }

    /// Get the pending command for a specific line, if any.
    pub fn get(&self, line: u64) -> Option<&PendingCommand> {
        self.commands.get(&line)
    }

    /// Query all pending commands of a given category.
    pub fn by_category(&self, category: LineCommandCategory) -> Vec<&PendingCommand> {
        self.commands
            .values()
            .filter(|pc| classify(&pc.command.kind) == category)
            .collect()
    }

    /// Query all pending source markers (C, CC, M, MM).
    pub fn pending_sources(&self) -> Vec<&PendingCommand> {
        self.commands
            .values()
            .filter(|pc| {
                matches!(
                    pc.command.kind,
                    LineCommandKind::Copy
                        | LineCommandKind::CopyBlock
                        | LineCommandKind::Move
                        | LineCommandKind::MoveBlock
                )
            })
            .collect()
    }

    /// Query all pending target markers (A, B).
    pub fn pending_targets(&self) -> Vec<&PendingCommand> {
        self.commands
            .values()
            .filter(|pc| {
                matches!(
                    pc.command.kind,
                    LineCommandKind::After | LineCommandKind::Before
                )
            })
            .collect()
    }

    /// Query all pending block markers of a specific kind.
    pub fn pending_blocks(&self, kind: &LineCommandKind) -> Vec<&PendingCommand> {
        self.commands
            .values()
            .filter(|pc| pc.command.kind == *kind)
            .collect()
    }

    /// Clear all pending commands (RESET COMMANDS / RESET ALL).
    pub fn clear_all(&mut self) {
        self.commands.clear();
    }

    /// Returns all pending commands as an iterator (for prefix area display).
    pub fn all_pending(&self) -> impl Iterator<Item = (&u64, &PendingCommand)> {
        self.commands.iter()
    }

    /// Returns the number of pending commands.
    pub fn count(&self) -> usize {
        self.commands.len()
    }

    /// Returns true if there are no pending commands.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for PendingCommandStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cmd(line: u64, kind: LineCommandKind) -> ParsedLineCommand {
        ParsedLineCommand { line, kind }
    }

    #[test]
    fn new_store_is_empty() {
        let store = PendingCommandStore::new();
        assert!(store.is_empty());
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn add_increments_count() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(0, LineCommandKind::Copy),
            PendingReason::AwaitingTarget,
        );
        assert_eq!(store.count(), 1);
        store.add(
            make_cmd(5, LineCommandKind::After),
            PendingReason::AwaitingSource,
        );
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn get_returns_added_command() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(3, LineCommandKind::DeleteBlock),
            PendingReason::AwaitingPair,
        );
        let pc = store.get(3).unwrap();
        assert_eq!(pc.command.kind, LineCommandKind::DeleteBlock);
        assert_eq!(pc.reason, PendingReason::AwaitingPair);
    }

    #[test]
    fn get_returns_none_for_missing_line() {
        let store = PendingCommandStore::new();
        assert!(store.get(99).is_none());
    }

    #[test]
    fn remove_returns_command_and_decrements_count() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(7, LineCommandKind::Copy),
            PendingReason::AwaitingTarget,
        );
        assert_eq!(store.count(), 1);

        let removed = store.remove(7);
        assert!(removed.is_some());
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn remove_returns_none_for_missing_line() {
        let mut store = PendingCommandStore::new();
        assert!(store.remove(42).is_none());
    }

    #[test]
    fn clear_all_empties_store() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(0, LineCommandKind::Copy),
            PendingReason::AwaitingTarget,
        );
        store.add(
            make_cmd(1, LineCommandKind::Move),
            PendingReason::AwaitingTarget,
        );
        store.add(
            make_cmd(2, LineCommandKind::DeleteBlock),
            PendingReason::AwaitingPair,
        );

        store.clear_all();
        assert!(store.is_empty());
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn by_category_filters_correctly() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(0, LineCommandKind::Copy),
            PendingReason::AwaitingTarget,
        );
        store.add(
            make_cmd(1, LineCommandKind::After),
            PendingReason::AwaitingSource,
        );
        store.add(
            make_cmd(2, LineCommandKind::DeleteBlock),
            PendingReason::AwaitingPair,
        );

        let sources = store.by_category(LineCommandCategory::Source);
        assert_eq!(sources.len(), 1);

        let targets = store.by_category(LineCommandCategory::Target);
        assert_eq!(targets.len(), 1);

        let blocks = store.by_category(LineCommandCategory::Block);
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn pending_sources_returns_copy_and_move_markers() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(0, LineCommandKind::Copy),
            PendingReason::AwaitingTarget,
        );
        store.add(
            make_cmd(1, LineCommandKind::Move),
            PendingReason::AwaitingTarget,
        );
        store.add(
            make_cmd(2, LineCommandKind::CopyBlock),
            PendingReason::AwaitingPair,
        );
        store.add(
            make_cmd(3, LineCommandKind::After),
            PendingReason::AwaitingSource,
        );

        let sources = store.pending_sources();
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn pending_targets_returns_a_and_b_markers() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(0, LineCommandKind::After),
            PendingReason::AwaitingSource,
        );
        store.add(
            make_cmd(1, LineCommandKind::Before),
            PendingReason::AwaitingSource,
        );
        store.add(
            make_cmd(2, LineCommandKind::Copy),
            PendingReason::AwaitingTarget,
        );

        let targets = store.pending_targets();
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn monotonic_entered_at_ordering() {
        let mut store = PendingCommandStore::new();
        store.add(
            make_cmd(0, LineCommandKind::Copy),
            PendingReason::AwaitingTarget,
        );
        store.add(
            make_cmd(1, LineCommandKind::Move),
            PendingReason::AwaitingTarget,
        );

        let first = store.get(0).unwrap();
        let second = store.get(1).unwrap();
        assert!(first.entered_at < second.entered_at);
    }
}
