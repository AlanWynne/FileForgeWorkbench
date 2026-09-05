//! SDSF help system: HELP, ACTH, COLH, CMDH, SEARCH commands.
//!
//! Implements Requirement 18 AC 18.5-18.9:
//!   - HELP / PF1: context-sensitive panel help (AC 18.5)
//!   - ACTH: action character help (AC 18.6)
//!   - COLH: column help (AC 18.7)
//!   - CMDH: primary command help (AC 18.8)
//!   - SEARCH <text> within help panel (AC 18.9)

// === Help Entry Types ========================================================

/// A single action character entry for ACTH.
///
/// Addresses: Requirement 18 AC 18.6
#[derive(Debug, Clone)]
pub struct ActionCharHelp {
    /// The action character (e.g. "S", "C", "H").
    pub character: &'static str,
    /// One-line description.
    pub description: &'static str,
}

/// A single column entry for COLH.
///
/// Addresses: Requirement 18 AC 18.7
#[derive(Debug, Clone)]
pub struct ColumnHelp {
    /// Column name (uppercase).
    pub name: &'static str,
    /// Data type description.
    pub data_type: &'static str,
    /// Display width in characters.
    pub width: usize,
    /// One-line description.
    pub description: &'static str,
}

/// A single primary command entry for CMDH.
///
/// Addresses: Requirement 18 AC 18.8
#[derive(Debug, Clone)]
pub struct CommandHelp {
    /// Command name (uppercase).
    pub name: &'static str,
    /// Syntax string (e.g. "SORT colname [A|D]").
    pub syntax: &'static str,
    /// One-line description.
    pub description: &'static str,
}

// === HelpPanel ===============================================================

/// The kind of help content being displayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpKind {
    /// General panel help (HELP / PF1).
    Panel,
    /// Action character help (ACTH).
    ActionChars,
    /// Column help (COLH).
    Columns,
    /// Primary command help (CMDH).
    Commands,
}

/// A rendered help panel with searchable content.
///
/// Addresses: Requirement 18 AC 18.5-18.9
#[derive(Debug, Clone)]
pub struct HelpPanel {
    /// What kind of help this panel shows.
    pub kind: HelpKind,
    /// The panel name this help is for (e.g. "ST", "I").
    pub panel_name: String,
    /// All lines of help content.
    pub lines: Vec<String>,
    /// Current search term (empty = no search active).
    pub search_term: String,
    /// Index of the first matching line (None = no match or no search).
    pub search_match: Option<usize>,
}

impl HelpPanel {
    /// Build a general panel help panel.
    ///
    /// Addresses: Requirement 18 AC 18.5
    pub fn panel_help(
        panel_name: &str,
        purpose: &str,
        commands: &[&str],
        columns: &[&str],
    ) -> Self {
        let mut lines = vec![
            format!("SDSF HELP -- Panel: {panel_name}"),
            String::new(),
            format!("Purpose: {purpose}"),
            String::new(),
            "Available Commands:".to_string(),
        ];
        for cmd in commands {
            lines.push(format!("  {cmd}"));
        }
        lines.push(String::new());
        lines.push("Columns:".to_string());
        for col in columns {
            lines.push(format!("  {col}"));
        }
        Self {
            kind: HelpKind::Panel,
            panel_name: panel_name.to_string(),
            lines,
            search_term: String::new(),
            search_match: None,
        }
    }

    /// Build an ACTH help panel.
    ///
    /// Addresses: Requirement 18 AC 18.6
    pub fn acth(panel_name: &str, actions: &[ActionCharHelp]) -> Self {
        let mut lines = vec![
            format!("SDSF ACTH -- Action Characters for Panel: {panel_name}"),
            String::new(),
        ];
        for a in actions {
            lines.push(format!("  {:4} -- {}", a.character, a.description));
        }
        Self {
            kind: HelpKind::ActionChars,
            panel_name: panel_name.to_string(),
            lines,
            search_term: String::new(),
            search_match: None,
        }
    }

    /// Build a COLH help panel.
    ///
    /// Addresses: Requirement 18 AC 18.7
    pub fn colh(panel_name: &str, columns: &[ColumnHelp]) -> Self {
        let mut lines = vec![
            format!("SDSF COLH -- Columns for Panel: {panel_name}"),
            String::new(),
            format!(
                "  {:<12} {:<10} {:>5}  Description",
                "Name", "Type", "Width"
            ),
            format!("  {}", "-".repeat(60)),
        ];
        for c in columns {
            lines.push(format!(
                "  {:<12} {:<10} {:>5}  {}",
                c.name, c.data_type, c.width, c.description
            ));
        }
        Self {
            kind: HelpKind::Columns,
            panel_name: panel_name.to_string(),
            lines,
            search_term: String::new(),
            search_match: None,
        }
    }

    /// Build a CMDH help panel.
    ///
    /// Addresses: Requirement 18 AC 18.8
    pub fn cmdh(panel_name: &str, commands: &[CommandHelp]) -> Self {
        let mut lines = vec![
            format!("SDSF CMDH -- Primary Commands for Panel: {panel_name}"),
            String::new(),
        ];
        for c in commands {
            lines.push(format!("  {:<20} -- {}", c.syntax, c.description));
        }
        Self {
            kind: HelpKind::Commands,
            panel_name: panel_name.to_string(),
            lines,
            search_term: String::new(),
            search_match: None,
        }
    }

    /// Search the help content for the given text; scroll to first match.
    ///
    /// Returns the line index of the first match, or None if not found.
    ///
    /// Addresses: Requirement 18 AC 18.9
    pub fn search(&mut self, text: &str) -> Option<usize> {
        self.search_term = text.to_string();
        if text.is_empty() {
            self.search_match = None;
            return None;
        }
        let lower = text.to_lowercase();
        self.search_match = self
            .lines
            .iter()
            .position(|line| line.to_lowercase().contains(&lower));
        self.search_match
    }

    /// Returns the total number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns whether a search is active with a match.
    pub fn has_match(&self) -> bool {
        self.search_match.is_some()
    }
}

// === HelpRegistry ============================================================

/// Provides help content for a named SDSF panel.
///
/// In the full implementation this is populated by each panel's metadata.
/// Here we provide a default set for the standard panels.
///
/// Addresses: Requirement 18 AC 18.5-18.8
pub struct HelpRegistry;

impl HelpRegistry {
    /// Standard action characters available on job list panels.
    pub fn standard_actions() -> Vec<ActionCharHelp> {
        vec![
            ActionCharHelp {
                character: "S",
                description: "Select / display output",
            },
            ActionCharHelp {
                character: "?",
                description: "Display job details",
            },
            ActionCharHelp {
                character: "C",
                description: "Cancel job",
            },
            ActionCharHelp {
                character: "H",
                description: "Hold job",
            },
            ActionCharHelp {
                character: "A",
                description: "Release (free) held job",
            },
            ActionCharHelp {
                character: "P",
                description: "Purge job and output",
            },
            ActionCharHelp {
                character: "D",
                description: "Delete job",
            },
            ActionCharHelp {
                character: "E",
                description: "Edit JCL",
            },
            ActionCharHelp {
                character: "J",
                description: "Display JCL",
            },
            ActionCharHelp {
                character: "W",
                description: "Who -- display job owner info",
            },
        ]
    }

    /// Standard columns for the ST / input queue panels.
    pub fn standard_columns() -> Vec<ColumnHelp> {
        vec![
            ColumnHelp {
                name: "JOBNAME",
                data_type: "CHAR",
                width: 8,
                description: "Job name",
            },
            ColumnHelp {
                name: "JOBID",
                data_type: "CHAR",
                width: 8,
                description: "Job identifier",
            },
            ColumnHelp {
                name: "OWNER",
                data_type: "CHAR",
                width: 8,
                description: "Job owner",
            },
            ColumnHelp {
                name: "STATUS",
                data_type: "CHAR",
                width: 8,
                description: "Current job status",
            },
            ColumnHelp {
                name: "CLASS",
                data_type: "CHAR",
                width: 1,
                description: "Job class",
            },
            ColumnHelp {
                name: "PRTY",
                data_type: "NUM",
                width: 3,
                description: "Job priority",
            },
            ColumnHelp {
                name: "RC",
                data_type: "NUM",
                width: 4,
                description: "Return code",
            },
        ]
    }

    /// Standard primary commands for job list panels.
    pub fn standard_commands() -> Vec<CommandHelp> {
        vec![
            CommandHelp {
                name: "PREFIX",
                syntax: "PREFIX <pattern>",
                description: "Filter by job name prefix",
            },
            CommandHelp {
                name: "OWNER",
                syntax: "OWNER <pattern>",
                description: "Filter by job owner",
            },
            CommandHelp {
                name: "SORT",
                syntax: "SORT colname [A|D]",
                description: "Sort by column",
            },
            CommandHelp {
                name: "FIND",
                syntax: "FIND <text>",
                description: "Search panel data",
            },
            CommandHelp {
                name: "LOCATE",
                syntax: "LOCATE <jobname>",
                description: "Scroll to job name",
            },
            CommandHelp {
                name: "FILTER",
                syntax: "FILTER <expr>",
                description: "Advanced filter expression",
            },
        ]
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 18.5
    #[test]
    fn panel_help_contains_purpose_and_sections() {
        let panel = HelpPanel::panel_help(
            "ST",
            "Display all jobs",
            &["PREFIX", "SORT", "FIND"],
            &["JOBNAME", "STATUS"],
        );
        assert_eq!(panel.kind, HelpKind::Panel);
        assert!(panel.lines.iter().any(|l| l.contains("Display all jobs")));
        assert!(panel.lines.iter().any(|l| l.contains("PREFIX")));
        assert!(panel.lines.iter().any(|l| l.contains("JOBNAME")));
    }

    // Validates: Requirement 18.5
    #[test]
    fn panel_help_includes_panel_name() {
        let panel = HelpPanel::panel_help("I", "Input queue", &[], &[]);
        assert!(panel.lines[0].contains("I"));
    }

    // Validates: Requirement 18.6
    #[test]
    fn acth_lists_all_action_characters() {
        let actions = HelpRegistry::standard_actions();
        let panel = HelpPanel::acth("ST", &actions);
        assert_eq!(panel.kind, HelpKind::ActionChars);
        assert!(panel.lines.iter().any(|l| l.contains("Cancel")));
        assert!(panel.lines.iter().any(|l| l.contains("Hold")));
    }

    // Validates: Requirement 18.6
    #[test]
    fn acth_has_entry_for_each_action() {
        let actions = HelpRegistry::standard_actions();
        let panel = HelpPanel::acth("ST", &actions);
        for action in &actions {
            assert!(
                panel.lines.iter().any(|l| l.contains(action.character)),
                "missing action: {}",
                action.character
            );
        }
    }

    // Validates: Requirement 18.7
    #[test]
    fn colh_lists_columns_with_type_and_width() {
        let cols = HelpRegistry::standard_columns();
        let panel = HelpPanel::colh("ST", &cols);
        assert_eq!(panel.kind, HelpKind::Columns);
        assert!(panel.lines.iter().any(|l| l.contains("JOBNAME")));
        assert!(panel.lines.iter().any(|l| l.contains("CHAR")));
    }

    // Validates: Requirement 18.7
    #[test]
    fn colh_has_entry_for_each_column() {
        let cols = HelpRegistry::standard_columns();
        let panel = HelpPanel::colh("ST", &cols);
        for col in &cols {
            assert!(
                panel.lines.iter().any(|l| l.contains(col.name)),
                "missing column: {}",
                col.name
            );
        }
    }

    // Validates: Requirement 18.8
    #[test]
    fn cmdh_lists_commands_with_syntax() {
        let cmds = HelpRegistry::standard_commands();
        let panel = HelpPanel::cmdh("ST", &cmds);
        assert_eq!(panel.kind, HelpKind::Commands);
        assert!(panel.lines.iter().any(|l| l.contains("PREFIX")));
        assert!(panel.lines.iter().any(|l| l.contains("Filter by job name")));
    }

    // Validates: Requirement 18.8
    #[test]
    fn cmdh_has_entry_for_each_command() {
        let cmds = HelpRegistry::standard_commands();
        let panel = HelpPanel::cmdh("ST", &cmds);
        for cmd in &cmds {
            assert!(
                panel.lines.iter().any(|l| l.contains(cmd.name)),
                "missing command: {}",
                cmd.name
            );
        }
    }

    // Validates: Requirement 18.9
    #[test]
    fn search_finds_matching_line() {
        let cmds = HelpRegistry::standard_commands();
        let mut panel = HelpPanel::cmdh("ST", &cmds);
        let idx = panel.search("PREFIX");
        assert!(idx.is_some());
        assert!(panel.has_match());
        assert!(panel.lines[idx.unwrap()].contains("PREFIX"));
    }

    // Validates: Requirement 18.9
    #[test]
    fn search_is_case_insensitive() {
        let cmds = HelpRegistry::standard_commands();
        let mut panel = HelpPanel::cmdh("ST", &cmds);
        let idx = panel.search("prefix");
        assert!(idx.is_some());
    }

    // Validates: Requirement 18.9
    #[test]
    fn search_returns_none_for_no_match() {
        let mut panel = HelpPanel::panel_help("ST", "purpose", &[], &[]);
        let idx = panel.search("XYZZY_NOT_PRESENT");
        assert!(idx.is_none());
        assert!(!panel.has_match());
    }

    // Validates: Requirement 18.9
    #[test]
    fn search_empty_string_clears_match() {
        let cmds = HelpRegistry::standard_commands();
        let mut panel = HelpPanel::cmdh("ST", &cmds);
        panel.search("PREFIX");
        assert!(panel.has_match());
        panel.search("");
        assert!(!panel.has_match());
    }
}
