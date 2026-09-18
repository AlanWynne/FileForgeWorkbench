//! Command-line history and RETRIEVE recall, owned by the command processor.
//!
//! CR-NR-084 (Option B): the raw command-line recall ring and the Retrieve
//! pointer live here, in the command-processor layer, rather than in the GUI
//! shell. `CommandLineHistory` is the single owner the shell forwards every
//! submitted command line to; it exposes record / retrieve / reset / list.
//!
//! This is DISTINCT from `crate::history::CommandHistory` (the CommandId +
//! params + timestamp audit log). This module is the raw-string recall ring:
//! entries are stored most-recent-first, de-duplicated with ISPF/B049 semantics
//! (case-insensitive command name; arguments case-insensitive outside quotes,
//! case-sensitive inside quotes), and browsed by the RETRIEVE command.
//!
//! Behaviour is byte-for-byte the same as the previous `ff_keys::CommandHistory`
//! + `RetrieveState` (which now re-export the types defined here); only the
//! ownership moved. No acceptance criterion changed (function-keys-and-history
//! Requirements 5-10, 19).

use std::collections::VecDeque;

/// A single entry in the command-line recall ring.
///
/// Addresses: function-keys-and-history Requirement 6, Requirement 7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLineEntry {
    /// The full command string as entered/dispatched.
    command: String,
}

impl CommandLineEntry {
    /// Create a new history entry.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }

    /// The full command string.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Extract the command name (first token) for deduplication comparison.
    ///
    /// Returns the first whitespace-delimited token.
    pub fn command_name(&self) -> &str {
        self.command.split_whitespace().next().unwrap_or("")
    }

    /// Extract the arguments portion (everything after the first token).
    pub fn arguments(&self) -> &str {
        let name = self.command_name();
        if name.is_empty() {
            return "";
        }
        let after_name = &self.command[name.len()..];
        // Skip leading whitespace after the command name
        after_name.trim_start()
    }

    /// Check if this entry is a duplicate of another using the deduplication rules:
    /// - Case-insensitive on the command name (first token).
    /// - Arguments compared case-insensitively OUTSIDE quotes, but
    ///   case-sensitively INSIDE single or double quotes.
    ///
    /// This matches ISPF command semantics and the owner's expectation: a
    /// command whose arguments are case-insensitive (e.g. `THEME LEGACY` vs
    /// `theme legacy`) de-duplicates as the same command, while quoted operands
    /// that ARE case-sensitive (e.g. a search string `FIND 'ERROR'` vs
    /// `FIND 'error'`) remain distinct. (B049)
    pub fn is_duplicate_of(&self, other: &CommandLineEntry) -> bool {
        self.command_name()
            .eq_ignore_ascii_case(other.command_name())
            && normalise_args_for_dedup(self.arguments())
                == normalise_args_for_dedup(other.arguments())
    }
}

/// Build a comparison key for an argument string: characters outside quotes are
/// lowercased (case-insensitive), characters inside single or double quotes are
/// preserved verbatim (case-sensitive). A quote toggles the in-quote state; the
/// quote characters themselves are kept so mismatched quoting stays distinct.
///
/// Validates: B049 -- quote-aware, ISPF-style argument de-duplication.
fn normalise_args_for_dedup(args: &str) -> String {
    let mut out = String::with_capacity(args.len());
    let mut quote: Option<char> = None;
    for c in args.chars() {
        match quote {
            None => {
                if c == '\'' || c == '"' {
                    quote = Some(c);
                    out.push(c);
                } else {
                    out.extend(c.to_lowercase());
                }
            }
            Some(q) => {
                out.push(c);
                if c == q {
                    quote = None;
                }
            }
        }
    }
    out
}

/// Bounded, deduplicated, ordered command-line recall ring.
///
/// Entries are stored most-recent-first.
///
/// Addresses: function-keys-and-history Requirement 6, Requirement 7, Requirement 9
#[derive(Debug, Clone)]
pub struct CommandLineRing {
    /// The history entries, most-recent-first.
    entries: VecDeque<CommandLineEntry>,
    /// Maximum number of entries.
    max_entries: usize,
}

impl CommandLineRing {
    /// The default maximum number of entries.
    pub const DEFAULT_MAX_ENTRIES: usize = 200;

    /// Create an empty history with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
        }
    }

    /// Create an empty history with the default capacity (200).
    pub fn with_default_capacity() -> Self {
        Self::new(Self::DEFAULT_MAX_ENTRIES)
    }

    /// Add a command to history.
    ///
    /// Applies deduplication and capacity rules:
    /// - If a duplicate exists, removes it and inserts the new entry at front.
    /// - If at capacity after dedup removal, removes the oldest entry before inserting.
    pub fn add(&mut self, command: impl Into<String>) {
        let new_entry = CommandLineEntry::new(command);

        if new_entry.command().trim().is_empty() {
            return;
        }

        // Remove existing duplicate (if any)
        self.entries.retain(|e| !e.is_duplicate_of(&new_entry));

        // Enforce capacity (remove oldest if at limit)
        if self.entries.len() >= self.max_entries {
            self.entries.pop_back();
        }

        // Insert at front (most recent)
        self.entries.push_front(new_entry);
    }

    /// Get the entry at the given index (0 = most recent).
    pub fn get(&self, index: usize) -> Option<&CommandLineEntry> {
        self.entries.get(index)
    }

    /// Number of entries currently in history.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether history is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the maximum capacity.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Update the maximum capacity.
    ///
    /// If the new max is smaller than the current length, oldest entries are trimmed.
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        while self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
    }

    /// Iterate over all entries, most-recent-first.
    pub fn iter(&self) -> impl Iterator<Item = &CommandLineEntry> {
        self.entries.iter()
    }

    /// Get the entries as a slice-like view (for serialization).
    pub fn entries(&self) -> &VecDeque<CommandLineEntry> {
        &self.entries
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Export entries as a Vec of command strings for serialization.
    pub fn to_command_strings(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|e| e.command().to_string())
            .collect()
    }

    /// Import entries from a Vec of command strings (most-recent-first order expected).
    ///
    /// Truncates to `max_entries` if the input exceeds capacity.
    pub fn from_command_strings(commands: Vec<String>, max_entries: usize) -> Self {
        let mut entries = VecDeque::new();
        for cmd in commands.into_iter().take(max_entries) {
            if !cmd.trim().is_empty() {
                entries.push_back(CommandLineEntry::new(cmd));
            }
        }
        Self {
            entries,
            max_entries,
        }
    }
}

// === RETRIEVE pointer =======================================================

/// Result of a RETRIEVE command invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrieveResult {
    /// Successfully recalled a command. Place it in the command field.
    Recalled {
        /// The recalled command string.
        command: String,
    },
    /// History is empty; nothing to recall.
    HistoryEmpty,
    /// Already at the oldest entry; no older history exists.
    NoOlderHistory,
    /// The command field contained "LIST" -- show the full history as a
    /// selectable list.
    ///
    /// Validates: Requirement 19.1, 19.2
    ShowList {
        /// All history entries in most-recent-first order.
        entries: Vec<String>,
    },
}

/// The state of the RETRIEVE pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PointerState {
    /// No retrieval cycle active. Next RETRIEVE starts from most recent.
    Initial,
    /// Currently pointing at a specific index in the ring.
    AtIndex(usize),
}

/// Manages the Retrieve Pointer, cycling backward through history on successive
/// calls.
#[derive(Debug)]
pub struct RetrieveState {
    /// Current pointer state.
    state: PointerState,
}

impl RetrieveState {
    /// Create a new retrieve state at the initial position.
    pub fn new() -> Self {
        Self {
            state: PointerState::Initial,
        }
    }

    /// Execute one RETRIEVE step.
    ///
    /// `command_field_text` is the current text in the Primary_Command_Field.
    /// If it equals "LIST" (case-insensitive, trimmed), returns `ShowList` with
    /// all history entries instead of performing single-step recall.
    ///
    /// Validates: Requirement 5.1-5.4, 5.7, 19.1-19.2
    pub fn retrieve(
        &mut self,
        history: &CommandLineRing,
        command_field_text: &str,
    ) -> RetrieveResult {
        // LIST trigger -- Validates: Requirement 19.1
        if command_field_text.trim().eq_ignore_ascii_case("LIST") {
            let entries = history.iter().map(|e| e.command().to_string()).collect();
            return RetrieveResult::ShowList { entries };
        }

        if history.is_empty() {
            return RetrieveResult::HistoryEmpty;
        }

        match &self.state {
            PointerState::Initial => {
                self.state = PointerState::AtIndex(0);
                RetrieveResult::Recalled {
                    command: history.get(0).unwrap().command().to_string(),
                }
            }
            PointerState::AtIndex(current) => {
                let next = current + 1;
                if next >= history.len() {
                    RetrieveResult::NoOlderHistory
                } else {
                    self.state = PointerState::AtIndex(next);
                    RetrieveResult::Recalled {
                        command: history.get(next).unwrap().command().to_string(),
                    }
                }
            }
        }
    }

    /// Reset the pointer to initial position.
    ///
    /// Called when any non-RETRIEVE command is submitted.
    pub fn reset(&mut self) {
        self.state = PointerState::Initial;
    }

    /// Set the pointer to a specific index (used by History_Dropdown selection).
    pub fn set_position(&mut self, index: usize) {
        self.state = PointerState::AtIndex(index);
    }

    /// Whether the pointer is at the initial (no retrieval) position.
    pub fn is_at_initial(&self) -> bool {
        matches!(self.state, PointerState::Initial)
    }

    /// The current pointer index, if active.
    pub fn current_index(&self) -> Option<usize> {
        match &self.state {
            PointerState::Initial => None,
            PointerState::AtIndex(i) => Some(*i),
        }
    }
}

impl Default for RetrieveState {
    fn default() -> Self {
        Self::new()
    }
}

// === CommandLineHistory: the processor-layer owner (CR-NR-084) ==============

/// Whether a submitted command line is the RETRIEVE command (the recall action),
/// which is never recorded and does not reset the pointer. Matches the VERB, so
/// the B066/B067 merged form `RETRIEVE <field>` is also recognised.
fn is_retrieve_command(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    upper == "RETRIEVE" || upper.starts_with("RETRIEVE ")
}

/// The single command-processor-owned command-line history: the recall ring plus
/// the RETRIEVE pointer. The GUI shell holds one of these and FORWARDS every
/// submitted command line to it, so command-line history ownership lives in the
/// processor layer (CR-NR-084, Option B). Behaviour is identical to the previous
/// shell-owned `CommandHistory` + `RetrieveState`.
#[derive(Debug)]
pub struct CommandLineHistory {
    ring: CommandLineRing,
    pointer: RetrieveState,
}

impl CommandLineHistory {
    /// Create a new command-line history with the given ring capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            ring: CommandLineRing::new(max_entries),
            pointer: RetrieveState::new(),
        }
    }

    /// Record a submitted command line.
    ///
    /// - The RETRIEVE command (any form, incl. the merged `RETRIEVE <field>`) is
    ///   NOT recorded and does NOT touch the pointer (Requirement 8.2, 19.8).
    /// - Any OTHER submitted line resets the retrieve pointer (Requirement 19.5)
    ///   and is added with dedup-promote + capacity eviction (Requirement 7, 9).
    /// - Empty/whitespace lines are ignored by the ring.
    pub fn record(&mut self, line: &str) {
        if is_retrieve_command(line) {
            return;
        }
        // A non-RETRIEVE submission ends any active retrieve cycle (Req 19.5).
        self.pointer.reset();
        self.ring.add(line);
    }

    /// Execute one RETRIEVE step against the current command-field text.
    ///
    /// `field_text` is the current Primary_Command_Field content -- the source of
    /// truth for the `LIST` trigger and the empty check (Requirement 19.1).
    pub fn retrieve(&mut self, field_text: &str) -> RetrieveResult {
        self.pointer.retrieve(&self.ring, field_text)
    }

    /// Reset the retrieve pointer to its initial position.
    pub fn reset(&mut self) {
        self.pointer.reset();
    }

    /// Point the retrieve pointer at a specific index (0 = most recent), used
    /// when an entry is chosen from the history list (Requirement 10.4).
    pub fn set_position(&mut self, index: usize) {
        self.pointer.set_position(index);
    }

    /// All history entries, most-recent-first (for the `RETRIEVE LIST` overlay).
    pub fn list(&self) -> Vec<String> {
        self.ring.to_command_strings()
    }

    /// Number of entries in the ring.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Whether the ring is empty.
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// The most-recent entry's command string, if any (index 0).
    pub fn most_recent(&self) -> Option<&str> {
        self.ring.get(0).map(|e| e.command())
    }

    /// Borrow the underlying ring (read-only), e.g. for persistence export.
    pub fn ring(&self) -> &CommandLineRing {
        &self.ring
    }

    /// The configured maximum number of entries (ring capacity).
    pub fn max_entries(&self) -> usize {
        self.ring.max_entries()
    }

    /// Replace the ring contents from persisted command strings (Requirement 6),
    /// preserving the configured capacity, and reset the pointer.
    pub fn load_command_strings(&mut self, commands: Vec<String>) {
        let max = self.ring.max_entries();
        self.ring = CommandLineRing::from_command_strings(commands, max);
        self.pointer.reset();
    }
}

impl Default for CommandLineHistory {
    fn default() -> Self {
        Self::new(CommandLineRing::DEFAULT_MAX_ENTRIES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === CommandLineRing (moved from ff-keys, behaviour unchanged) ==========

    #[test]
    fn history_entry_command_name_extraction() {
        let entry = CommandLineEntry::new("FIND 'ERROR' ALL");
        assert_eq!(entry.command_name(), "FIND");
        assert_eq!(entry.arguments(), "'ERROR' ALL");
    }

    #[test]
    fn history_entry_dedup_case_insensitive_name() {
        // Validates: Requirement 7.2 -- case-insensitive on command name
        let a = CommandLineEntry::new("find 'ERROR'");
        let b = CommandLineEntry::new("FIND 'ERROR'");
        assert!(a.is_duplicate_of(&b));
    }

    #[test]
    fn history_entry_dedup_case_sensitive_inside_quotes() {
        // Validates: Requirement 7.2 + B049
        let a = CommandLineEntry::new("FIND 'ERROR'");
        let b = CommandLineEntry::new("FIND 'error'");
        assert!(!a.is_duplicate_of(&b));
    }

    #[test]
    fn history_entry_dedup_case_insensitive_unquoted_args() {
        // Validates: B049
        let a = CommandLineEntry::new("THEME LEGACY");
        let b = CommandLineEntry::new("theme legacy");
        assert!(a.is_duplicate_of(&b));
        let c = CommandLineEntry::new("THEME dark");
        assert!(!a.is_duplicate_of(&c));
    }

    #[test]
    fn add_deduplicates_and_promotes() {
        // Validates: Requirement 7.1
        let mut ring = CommandLineRing::new(10);
        ring.add("SAVE");
        ring.add("FIND 'ERROR'");
        ring.add("SAVE"); // duplicate -- promoted to front
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.get(0).unwrap().command(), "SAVE");
        assert_eq!(ring.get(1).unwrap().command(), "FIND 'ERROR'");
    }

    #[test]
    fn capacity_enforcement_evicts_oldest() {
        // Validates: Requirement 9.3
        let mut ring = CommandLineRing::new(3);
        ring.add("CMD1");
        ring.add("CMD2");
        ring.add("CMD3");
        ring.add("CMD4"); // evicts CMD1
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.get(0).unwrap().command(), "CMD4");
        assert_eq!(ring.get(2).unwrap().command(), "CMD2");
    }

    #[test]
    fn empty_command_not_added() {
        let mut ring = CommandLineRing::new(10);
        ring.add("");
        ring.add("   ");
        assert!(ring.is_empty());
    }

    #[test]
    fn to_from_command_strings_round_trip() {
        let mut ring = CommandLineRing::new(10);
        ring.add("CMD1");
        ring.add("CMD2");
        let restored = CommandLineRing::from_command_strings(ring.to_command_strings(), 10);
        assert_eq!(restored.len(), ring.len());
        assert_eq!(restored.get(0).unwrap().command(), "CMD2");
    }

    // === RetrieveState (moved from ff-keys, behaviour unchanged) ============

    fn make_ring(commands: &[&str]) -> CommandLineRing {
        let mut ring = CommandLineRing::new(200);
        for &cmd in commands.iter().rev() {
            ring.add(cmd);
        }
        ring
    }

    #[test]
    fn retrieve_from_empty_history() {
        // Validates: Requirement 5.7
        let ring = CommandLineRing::new(200);
        let mut state = RetrieveState::new();
        assert_eq!(state.retrieve(&ring, ""), RetrieveResult::HistoryEmpty);
    }

    #[test]
    fn successive_retrieves_cycle_backward() {
        // Validates: Requirement 5.2, 5.3
        let ring = make_ring(&["CMD1", "CMD2", "CMD3"]);
        let mut state = RetrieveState::new();
        assert_eq!(
            state.retrieve(&ring, ""),
            RetrieveResult::Recalled {
                command: "CMD1".to_string()
            }
        );
        assert_eq!(
            state.retrieve(&ring, ""),
            RetrieveResult::Recalled {
                command: "CMD2".to_string()
            }
        );
    }

    #[test]
    fn list_trigger_returns_show_list() {
        // Validates: Requirement 19.1, 19.2
        let ring = make_ring(&["CMD1", "CMD2"]);
        let mut state = RetrieveState::new();
        assert_eq!(
            state.retrieve(&ring, "LIST"),
            RetrieveResult::ShowList {
                entries: vec!["CMD1".to_string(), "CMD2".to_string()]
            }
        );
    }

    // === CommandLineHistory owner (CR-NR-084) ===============================

    #[test]
    fn owner_records_and_recalls_most_recent() {
        // Validates: Requirement 5.1, 5.2, 19.1
        let mut h = CommandLineHistory::new(200);
        h.record("LOCATE 1");
        h.record("THEME legacy");
        assert_eq!(
            h.retrieve(""),
            RetrieveResult::Recalled {
                command: "THEME legacy".to_string()
            }
        );
        // A second retrieve steps older.
        assert_eq!(
            h.retrieve(""),
            RetrieveResult::Recalled {
                command: "LOCATE 1".to_string()
            }
        );
    }

    #[test]
    fn owner_excludes_retrieve_from_history() {
        // Validates: Requirement 8.2, 19.8 -- RETRIEVE (bare or merged) is not
        // recorded and does not reset the pointer.
        let mut h = CommandLineHistory::new(200);
        h.record("LOCATE 1");
        h.record("RETRIEVE");
        h.record("RETRIEVE 1"); // the B066/B067 merged form
        h.record("RETRIEVE LIST");
        assert_eq!(h.len(), 1, "only LOCATE 1 is recorded");
        assert_eq!(h.most_recent(), Some("LOCATE 1"));
    }

    #[test]
    fn owner_non_retrieve_resets_pointer() {
        // Validates: Requirement 19.5 -- a non-RETRIEVE submission resets the
        // pointer so the next RETRIEVE starts at the most recent again.
        let mut h = CommandLineHistory::new(200);
        h.record("A");
        h.record("B");
        assert_eq!(
            h.retrieve(""),
            RetrieveResult::Recalled {
                command: "B".to_string()
            }
        );
        // Submit a new command; the pointer resets and B is promoted.
        h.record("C");
        assert_eq!(
            h.retrieve(""),
            RetrieveResult::Recalled {
                command: "C".to_string()
            }
        );
    }

    #[test]
    fn owner_list_returns_all_entries_most_recent_first() {
        // Validates: Requirement 19.2, 19.3
        let mut h = CommandLineHistory::new(200);
        h.record("A");
        h.record("B");
        assert_eq!(h.list(), vec!["B".to_string(), "A".to_string()]);
    }

    #[test]
    fn owner_load_command_strings_replaces_and_resets() {
        // Validates: Requirement 6 (persistence seam)
        let mut h = CommandLineHistory::new(200);
        h.record("OLD");
        h.load_command_strings(vec!["X".to_string(), "Y".to_string()]);
        assert_eq!(h.list(), vec!["X".to_string(), "Y".to_string()]);
        assert!(h.retrieve("").command_is("X"));
    }

    // Small helper for the assertion above.
    impl RetrieveResult {
        fn command_is(&self, expected: &str) -> bool {
            matches!(self, RetrieveResult::Recalled { command } if command == expected)
        }
    }
}
