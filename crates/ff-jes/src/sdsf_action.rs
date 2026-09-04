//! SDSF NP column action character system.
//!
//! Validates: Requirement 16 AC 5, 7, 8, 9, 10, 11, 12, 23

use crate::model::JobId;

// === Action Characters ======================================================

/// A single SDSF action character.
///
/// Validates: Requirement 16.8, 16.23
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChar {
    /// S -- select/view job output.
    Select,
    /// ? -- display valid actions for this job.
    QueryActions,
    /// C -- cancel job.
    Cancel,
    /// H -- hold job.
    Hold,
    /// A -- release (free) held job.
    Release,
    /// P -- purge job output.
    Purge,
    /// D -- delete output dataset.
    DeleteOutput,
    /// E -- edit JCL.
    EditJcl,
    /// J -- view JCL (browse).
    ViewJcl,
    /// W -- who has job (ownership info).
    Who,
}

impl ActionChar {
    /// Parses a single character into an ActionChar.
    pub fn parse(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'S' => Some(Self::Select),
            '?' => Some(Self::QueryActions),
            'C' => Some(Self::Cancel),
            'H' => Some(Self::Hold),
            'A' => Some(Self::Release),
            'P' => Some(Self::Purge),
            'D' => Some(Self::DeleteOutput),
            'E' => Some(Self::EditJcl),
            'J' => Some(Self::ViewJcl),
            'W' => Some(Self::Who),
            _ => None,
        }
    }

    /// Returns the character representation.
    pub fn as_char(self) -> char {
        match self {
            Self::Select => 'S',
            Self::QueryActions => '?',
            Self::Cancel => 'C',
            Self::Hold => 'H',
            Self::Release => 'A',
            Self::Purge => 'P',
            Self::DeleteOutput => 'D',
            Self::EditJcl => 'E',
            Self::ViewJcl => 'J',
            Self::Who => 'W',
        }
    }

    /// Returns a short description of the action.
    pub fn description(self) -> &'static str {
        match self {
            Self::Select => "Select/view job output",
            Self::QueryActions => "Display valid actions for this job",
            Self::Cancel => "Cancel job",
            Self::Hold => "Hold job",
            Self::Release => "Release held job",
            Self::Purge => "Purge job output",
            Self::DeleteOutput => "Delete output dataset",
            Self::EditJcl => "Edit JCL",
            Self::ViewJcl => "View JCL (browse)",
            Self::Who => "Who has job",
        }
    }
}

// === NP Column Entry ========================================================

/// What the user has typed into the NP column for a given row.
///
/// Validates: Requirement 16.5, 16.9, 16.10
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpEntry {
    /// A single action character.
    Action(ActionChar),
    /// Repeat previous action (=).
    Repeat,
    /// Block start/end marker (//).
    BlockMark,
    /// Empty (no entry).
    Empty,
}

impl NpEntry {
    /// Parses the text typed into an NP column cell.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if s.is_empty() {
            return Self::Empty;
        }
        if s == "=" {
            return Self::Repeat;
        }
        if s == "//" {
            return Self::BlockMark;
        }
        if s.len() == 1 {
            if let Some(ac) = ActionChar::parse(s.chars().next().unwrap()) {
                return Self::Action(ac);
            }
        }
        Self::Empty
    }
}

// === NP Action Dispatch =====================================================

/// A resolved action to execute on a specific job row.
///
/// Validates: Requirement 16.7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpDispatch {
    pub job_id: JobId,
    pub action: ActionChar,
}

/// State for the NP column across all visible rows.
///
/// Validates: Requirement 16.5, 16.9, 16.10
#[derive(Debug, Clone, Default)]
pub struct NpColumnState {
    /// The last action character used (for = repeat).
    pub last_action: Option<ActionChar>,
    /// Whether SET ROWNUM ON is active.
    pub show_row_numbers: bool,
}

impl NpColumnState {
    /// Resolves NP entries for a list of (row_index, job_id, np_text) tuples.
    ///
    /// Handles = repeat and // block syntax.
    /// Returns the list of dispatches to execute.
    ///
    /// Validates: Requirement 16.7, 16.9, 16.10
    pub fn resolve_entries(
        &mut self,
        rows: &[(usize, JobId, String)],
    ) -> Result<Vec<NpDispatch>, String> {
        let mut dispatches = Vec::new();

        // First pass: find block markers and resolve entries
        let entries: Vec<(usize, JobId, NpEntry)> = rows
            .iter()
            .map(|(idx, id, text)| (*idx, *id, NpEntry::parse(text)))
            .collect();

        // Find block marker positions
        let block_marks: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter(|(_, (_, _, e))| *e == NpEntry::BlockMark)
            .map(|(i, _)| i)
            .collect();

        // Determine block range (first and last // markers)
        let block_range = if block_marks.len() >= 2 {
            Some((block_marks[0], *block_marks.last().unwrap()))
        } else {
            None
        };

        for (i, (_, job_id, entry)) in entries.iter().enumerate() {
            match entry {
                NpEntry::Action(ac) => {
                    self.last_action = Some(*ac);
                    dispatches.push(NpDispatch {
                        job_id: *job_id,
                        action: *ac,
                    });
                }
                NpEntry::Repeat => {
                    if let Some(ac) = self.last_action {
                        dispatches.push(NpDispatch {
                            job_id: *job_id,
                            action: ac,
                        });
                    } else {
                        return Err("No previous action to repeat".to_string());
                    }
                }
                NpEntry::BlockMark => {
                    // Block marks themselves don't dispatch; the range does
                    if let Some((start, end)) = block_range {
                        if i == start {
                            // Need an action -- look for one in the block
                            // Block action requires a prior last_action
                            if let Some(ac) = self.last_action {
                                // Dispatch all rows in range
                                for (_, entry_job_id, _) in &entries[start..=end] {
                                    dispatches.push(NpDispatch {
                                        job_id: *entry_job_id,
                                        action: ac,
                                    });
                                }
                            }
                        }
                    }
                }
                NpEntry::Empty => {}
            }
        }

        Ok(dispatches)
    }
}

// === Command-Line Action Syntax =============================================

/// Parsed result of a command-line action like "2 C".
///
/// Validates: Requirement 16.11
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLineAction {
    /// 1-based row number.
    pub row: usize,
    /// The action to apply.
    pub action: ActionChar,
}

impl CommandLineAction {
    /// Parses "N ACTION" from the command input field.
    /// Returns None if the input does not match this syntax.
    pub fn parse(input: &str) -> Option<Self> {
        let mut parts = input.split_whitespace();
        let row_str = parts.next()?;
        let action_str = parts.next()?;
        if parts.next().is_some() {
            return None; // too many tokens
        }
        let row: usize = row_str.parse().ok()?;
        if row == 0 {
            return None;
        }
        if action_str.len() != 1 {
            return None;
        }
        let action = ActionChar::parse(action_str.chars().next().unwrap())?;
        Some(Self { row, action })
    }
}

// === SET ROWNUM =============================================================

/// Parses a SET ROWNUM ON/OFF command.
///
/// Validates: Requirement 16.12
pub fn parse_set_rownum(operands: &str) -> Option<bool> {
    match operands.trim().to_uppercase().as_str() {
        "ON" => Some(true),
        "OFF" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::JobId;

    fn jid(n: u64) -> JobId {
        JobId::new(n)
    }

    // --- ActionChar tests ---

    #[test]
    fn action_char_parse_all_valid() {
        // Validates: Requirement 16.8
        for (c, expected) in [
            ('S', ActionChar::Select),
            ('?', ActionChar::QueryActions),
            ('C', ActionChar::Cancel),
            ('H', ActionChar::Hold),
            ('A', ActionChar::Release),
            ('P', ActionChar::Purge),
            ('D', ActionChar::DeleteOutput),
            ('E', ActionChar::EditJcl),
            ('J', ActionChar::ViewJcl),
            ('W', ActionChar::Who),
        ] {
            assert_eq!(ActionChar::parse(c), Some(expected));
        }
    }

    #[test]
    fn action_char_parse_lowercase() {
        // Validates: Requirement 16.8
        assert_eq!(ActionChar::parse('s'), Some(ActionChar::Select));
        assert_eq!(ActionChar::parse('c'), Some(ActionChar::Cancel));
    }

    #[test]
    fn action_char_parse_invalid_returns_none() {
        // Validates: Requirement 16.23
        assert_eq!(ActionChar::parse('Z'), None);
        assert_eq!(ActionChar::parse('1'), None);
    }

    #[test]
    fn action_char_round_trip() {
        let ac = ActionChar::Cancel;
        assert_eq!(ActionChar::parse(ac.as_char()), Some(ac));
    }

    // --- NpEntry tests ---

    #[test]
    fn np_entry_parse_action() {
        assert_eq!(NpEntry::parse("S"), NpEntry::Action(ActionChar::Select));
        assert_eq!(NpEntry::parse("c"), NpEntry::Action(ActionChar::Cancel));
    }

    #[test]
    fn np_entry_parse_repeat() {
        // Validates: Requirement 16.9
        assert_eq!(NpEntry::parse("="), NpEntry::Repeat);
    }

    #[test]
    fn np_entry_parse_block_mark() {
        // Validates: Requirement 16.10
        assert_eq!(NpEntry::parse("//"), NpEntry::BlockMark);
    }

    #[test]
    fn np_entry_parse_empty() {
        assert_eq!(NpEntry::parse(""), NpEntry::Empty);
        assert_eq!(NpEntry::parse("  "), NpEntry::Empty);
    }

    #[test]
    fn np_entry_parse_invalid_returns_empty() {
        // Validates: Requirement 16.23 -- invalid chars produce Empty (rejected)
        assert_eq!(NpEntry::parse("Z"), NpEntry::Empty);
    }

    // --- NpColumnState dispatch tests ---

    #[test]
    fn np_dispatch_single_action() {
        // Validates: Requirement 16.7
        let mut state = NpColumnState::default();
        let rows = vec![(0, jid(1), "S".to_string())];
        let dispatches = state.resolve_entries(&rows).unwrap();
        assert_eq!(dispatches.len(), 1);
        assert_eq!(dispatches[0].action, ActionChar::Select);
        assert_eq!(dispatches[0].job_id, jid(1));
    }

    #[test]
    fn np_dispatch_repeat_uses_last_action() {
        // Validates: Requirement 16.9
        let mut state = NpColumnState::default();
        let rows = vec![(0, jid(1), "C".to_string()), (1, jid(2), "=".to_string())];
        let dispatches = state.resolve_entries(&rows).unwrap();
        assert_eq!(dispatches.len(), 2);
        assert_eq!(dispatches[1].action, ActionChar::Cancel);
        assert_eq!(dispatches[1].job_id, jid(2));
    }

    #[test]
    fn np_dispatch_repeat_with_no_prior_action_errors() {
        // Validates: Requirement 16.9
        let mut state = NpColumnState::default();
        let rows = vec![(0, jid(1), "=".to_string())];
        assert!(state.resolve_entries(&rows).is_err());
    }

    #[test]
    fn np_dispatch_block_applies_to_range() {
        // Validates: Requirement 16.10
        let mut state = NpColumnState {
            last_action: Some(ActionChar::Hold),
            show_row_numbers: false,
        };
        let rows = vec![
            (0, jid(1), "//".to_string()),
            (1, jid(2), "".to_string()),
            (2, jid(3), "//".to_string()),
        ];
        let dispatches = state.resolve_entries(&rows).unwrap();
        // All 3 rows in block should be dispatched with Hold
        assert_eq!(dispatches.len(), 3);
        assert!(dispatches.iter().all(|d| d.action == ActionChar::Hold));
    }

    #[test]
    fn np_dispatch_empty_rows_produce_no_dispatches() {
        let mut state = NpColumnState::default();
        let rows = vec![(0, jid(1), "".to_string()), (1, jid(2), "".to_string())];
        let dispatches = state.resolve_entries(&rows).unwrap();
        assert!(dispatches.is_empty());
    }

    // --- CommandLineAction tests ---

    #[test]
    fn command_line_action_parse_valid() {
        // Validates: Requirement 16.11
        let cla = CommandLineAction::parse("2 C").unwrap();
        assert_eq!(cla.row, 2);
        assert_eq!(cla.action, ActionChar::Cancel);
    }

    #[test]
    fn command_line_action_parse_lowercase() {
        let cla = CommandLineAction::parse("3 s").unwrap();
        assert_eq!(cla.row, 3);
        assert_eq!(cla.action, ActionChar::Select);
    }

    #[test]
    fn command_line_action_parse_row_zero_invalid() {
        assert!(CommandLineAction::parse("0 C").is_none());
    }

    #[test]
    fn command_line_action_parse_too_many_tokens() {
        assert!(CommandLineAction::parse("2 C extra").is_none());
    }

    #[test]
    fn command_line_action_parse_non_numeric_row() {
        assert!(CommandLineAction::parse("X C").is_none());
    }

    #[test]
    fn command_line_action_parse_invalid_action() {
        // Validates: Requirement 16.23
        assert!(CommandLineAction::parse("2 Z").is_none());
    }

    // --- SET ROWNUM tests ---

    #[test]
    fn set_rownum_on() {
        // Validates: Requirement 16.12
        assert_eq!(parse_set_rownum("ON"), Some(true));
        assert_eq!(parse_set_rownum("on"), Some(true));
    }

    #[test]
    fn set_rownum_off() {
        assert_eq!(parse_set_rownum("OFF"), Some(false));
    }

    #[test]
    fn set_rownum_invalid() {
        assert_eq!(parse_set_rownum("YES"), None);
    }
}
