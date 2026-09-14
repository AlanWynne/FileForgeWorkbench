//! Command History — bounded, deduplicated, ordered ring of past commands.
//!
//! Entries are stored most-recent-first. Deduplication uses case-insensitive
//! comparison on the command name (first token). Arguments are compared
//! case-insensitively outside quotes and case-sensitively inside single/double
//! quotes, so `THEME LEGACY` and `theme legacy` de-duplicate while
//! `FIND 'ERROR'` and `FIND 'error'` remain distinct (ISPF semantics, B049).

use std::collections::VecDeque;

/// A single entry in the Command_History.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// The full command string as entered/dispatched.
    command: String,
}

impl HistoryEntry {
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
    pub fn is_duplicate_of(&self, other: &HistoryEntry) -> bool {
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

/// Bounded, deduplicated, ordered command history ring.
///
/// Entries are stored most-recent-first.
#[derive(Debug, Clone)]
pub struct CommandHistory {
    /// The history entries, most-recent-first.
    entries: VecDeque<HistoryEntry>,
    /// Maximum number of entries.
    max_entries: usize,
}

impl CommandHistory {
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
        let new_entry = HistoryEntry::new(command);

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
    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
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
    pub fn iter(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter()
    }

    /// Get the entries as a slice-like view (for serialization).
    pub fn entries(&self) -> &VecDeque<HistoryEntry> {
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
                entries.push_back(HistoryEntry::new(cmd));
            }
        }
        Self {
            entries,
            max_entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_entry_command_name_extraction() {
        let entry = HistoryEntry::new("FIND 'ERROR' ALL");
        assert_eq!(entry.command_name(), "FIND");
        assert_eq!(entry.arguments(), "'ERROR' ALL");
    }

    #[test]
    fn history_entry_single_word_command() {
        let entry = HistoryEntry::new("SAVE");
        assert_eq!(entry.command_name(), "SAVE");
        assert_eq!(entry.arguments(), "");
    }

    #[test]
    fn history_entry_dedup_case_insensitive_name() {
        // Validates: Requirement 7.2 — case-insensitive on command name
        let a = HistoryEntry::new("find 'ERROR'");
        let b = HistoryEntry::new("FIND 'ERROR'");
        assert!(a.is_duplicate_of(&b));
        assert!(b.is_duplicate_of(&a));
    }

    #[test]
    fn history_entry_dedup_case_sensitive_inside_quotes() {
        // Validates: Requirement 7.2 + B049 -- quoted operands stay case-sensitive,
        // so two searches for differently-cased strings remain distinct.
        let a = HistoryEntry::new("FIND 'ERROR'");
        let b = HistoryEntry::new("FIND 'error'");
        assert!(!a.is_duplicate_of(&b));
    }

    #[test]
    fn history_entry_dedup_case_insensitive_unquoted_args() {
        // Validates: B049 -- unquoted arguments (e.g. a theme mode name) are
        // compared case-insensitively, so `THEME LEGACY` and `theme legacy`
        // de-duplicate as the same command.
        let a = HistoryEntry::new("THEME LEGACY");
        let b = HistoryEntry::new("theme legacy");
        assert!(a.is_duplicate_of(&b));
        assert!(b.is_duplicate_of(&a));

        // And a genuinely different unquoted arg is still distinct.
        let c = HistoryEntry::new("THEME dark");
        assert!(!a.is_duplicate_of(&c));
    }

    #[test]
    fn history_add_deduplicates_theme_regardless_of_case() {
        // Validates: B049 end-to-end -- adding both casings yields one entry.
        let mut history = CommandHistory::new(10);
        history.add("THEME legacy");
        history.add("THEME LEGACY");
        assert_eq!(history.len(), 1, "case-only difference must de-duplicate");
    }

    #[test]
    fn history_entry_different_names_not_duplicate() {
        let a = HistoryEntry::new("FIND 'ERROR'");
        let b = HistoryEntry::new("CHANGE 'ERROR'");
        assert!(!a.is_duplicate_of(&b));
    }

    #[test]
    fn add_inserts_at_front() {
        // Validates: Requirement 7.3
        let mut history = CommandHistory::new(10);
        history.add("SAVE");
        history.add("FIND 'ERROR'");

        assert_eq!(history.get(0).unwrap().command(), "FIND 'ERROR'");
        assert_eq!(history.get(1).unwrap().command(), "SAVE");
    }

    #[test]
    fn add_deduplicates_and_promotes() {
        // Validates: Requirement 7.1
        let mut history = CommandHistory::new(10);
        history.add("SAVE");
        history.add("FIND 'ERROR'");
        history.add("SAVE"); // duplicate — should be promoted to front

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0).unwrap().command(), "SAVE");
        assert_eq!(history.get(1).unwrap().command(), "FIND 'ERROR'");
    }

    #[test]
    fn add_case_insensitive_dedup_promotes_new_form() {
        // Validates: Requirement 7.1, 7.2
        let mut history = CommandHistory::new(10);
        history.add("find 'ERROR'");
        history.add("FIND 'ERROR'"); // same command (case-insensitive) + same args

        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap().command(), "FIND 'ERROR'");
    }

    #[test]
    fn capacity_enforcement_evicts_oldest() {
        // Validates: Requirement 9.3
        let mut history = CommandHistory::new(3);
        history.add("CMD1");
        history.add("CMD2");
        history.add("CMD3");
        assert_eq!(history.len(), 3);

        history.add("CMD4"); // should evict CMD1
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().command(), "CMD4");
        assert_eq!(history.get(2).unwrap().command(), "CMD2");
    }

    #[test]
    fn set_max_entries_trims_oldest() {
        // Validates: Requirement 9.3, 11.7
        let mut history = CommandHistory::new(10);
        history.add("CMD1");
        history.add("CMD2");
        history.add("CMD3");
        history.add("CMD4");

        history.set_max_entries(2);
        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0).unwrap().command(), "CMD4");
        assert_eq!(history.get(1).unwrap().command(), "CMD3");
    }

    #[test]
    fn empty_command_not_added() {
        let mut history = CommandHistory::new(10);
        history.add("");
        history.add("   ");
        assert!(history.is_empty());
    }

    #[test]
    fn from_command_strings_preserves_order() {
        let commands = vec!["CMD3".to_string(), "CMD2".to_string(), "CMD1".to_string()];
        let history = CommandHistory::from_command_strings(commands, 200);
        assert_eq!(history.get(0).unwrap().command(), "CMD3");
        assert_eq!(history.get(2).unwrap().command(), "CMD1");
    }

    #[test]
    fn from_command_strings_truncates_to_max() {
        let commands: Vec<String> = (0..50).map(|i| format!("CMD{}", i)).collect();
        let history = CommandHistory::from_command_strings(commands, 10);
        assert_eq!(history.len(), 10);
    }

    #[test]
    fn to_command_strings_round_trip() {
        let mut history = CommandHistory::new(10);
        history.add("CMD1");
        history.add("CMD2");
        history.add("CMD3");

        let strings = history.to_command_strings();
        let restored = CommandHistory::from_command_strings(strings, 10);
        assert_eq!(restored.len(), history.len());
        for i in 0..history.len() {
            assert_eq!(
                restored.get(i).unwrap().command(),
                history.get(i).unwrap().command()
            );
        }
    }

    #[test]
    fn clear_removes_all_entries() {
        let mut history = CommandHistory::new(10);
        history.add("CMD1");
        history.add("CMD2");
        history.clear();
        assert!(history.is_empty());
    }
}
