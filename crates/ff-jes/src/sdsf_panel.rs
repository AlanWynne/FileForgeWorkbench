//! SDSF panel chrome state: title line, SCROLL field, message area,
//! COMMAND INPUT field, and main panel command list.
//!
//! Validates: Requirement 16 AC 1, 2, 3, 13, 14, 15, 16, 17, 21, 22

// === Scroll Amount ==========================================================

/// The value stored in the SCROLL ===> field.
///
/// Validates: Requirement 16.3
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScrollAmount {
    /// Scroll by a fixed number of rows/columns.
    Lines(usize),
    /// Scroll by half the visible area.
    Half,
    /// Scroll to the cursor position.
    Csr,
    /// Scroll to the maximum extent.
    Max,
    /// Scroll by one page.
    #[default]
    Page,
}

impl ScrollAmount {
    /// Parses a SCROLL field value string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "HALF" => Some(Self::Half),
            "CSR" => Some(Self::Csr),
            "MAX" => Some(Self::Max),
            "PAGE" => Some(Self::Page),
            other => other.parse::<usize>().ok().map(Self::Lines),
        }
    }

    /// Returns the canonical string representation.
    pub fn display(&self) -> String {
        match self {
            Self::Lines(n) => n.to_string(),
            Self::Half => "HALF".to_string(),
            Self::Csr => "CSR".to_string(),
            Self::Max => "MAX".to_string(),
            Self::Page => "PAGE".to_string(),
        }
    }
}

// === Main Panel Commands ====================================================

/// A command group in the main panel.
///
/// Validates: Requirement 16.14
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandGroup {
    Jobs,
    Output,
    Jes,
    Log,
    Memory,
    Other,
}

impl CommandGroup {
    pub fn label(self) -> &'static str {
        match self {
            Self::Jobs => "Jobs",
            Self::Output => "Output",
            Self::Jes => "JES",
            Self::Log => "Log",
            Self::Memory => "Memory",
            Self::Other => "Other",
        }
    }
}

/// A single entry in the main panel command list.
///
/// Validates: Requirement 16.13
#[derive(Debug, Clone)]
pub struct MainPanelCommand {
    /// Short command name (e.g. "ST", "I", "O").
    pub name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Which group this command belongs to.
    pub group: CommandGroup,
}

/// Returns the full list of main panel commands.
///
/// Validates: Requirement 16.13, 16.14
pub fn main_panel_commands() -> &'static [MainPanelCommand] {
    &[
        MainPanelCommand {
            name: "ST",
            description: "Status -- all jobs",
            group: CommandGroup::Jobs,
        },
        MainPanelCommand {
            name: "I",
            description: "Input queue",
            group: CommandGroup::Jobs,
        },
        MainPanelCommand {
            name: "O",
            description: "Output queue",
            group: CommandGroup::Output,
        },
        MainPanelCommand {
            name: "H",
            description: "Held output queue",
            group: CommandGroup::Output,
        },
        MainPanelCommand {
            name: "JES",
            description: "JES spool datasets",
            group: CommandGroup::Jes,
        },
        MainPanelCommand {
            name: "LOG",
            description: "System log",
            group: CommandGroup::Log,
        },
        MainPanelCommand {
            name: "ULOG",
            description: "User log",
            group: CommandGroup::Log,
        },
        MainPanelCommand {
            name: "MEM",
            description: "Memory map",
            group: CommandGroup::Memory,
        },
        MainPanelCommand {
            name: "WHO",
            description: "Session information",
            group: CommandGroup::Other,
        },
        MainPanelCommand {
            name: "INIT",
            description: "Initiator pool status",
            group: CommandGroup::Other,
        },
    ]
}

// === Panel Chrome State =====================================================

/// The name of the currently active SDSF sub-panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivePanel {
    /// The main panel (MENU).
    Main,
    /// A named sub-panel (e.g. "ST", "I", "O").
    Sub(String),
}

impl Default for ActivePanel {
    fn default() -> Self {
        Self::Sub("I".to_string())
    }
}

/// Full chrome state for the SDSF Job Monitor panel.
///
/// Validates: Requirement 16 AC 1-4, 13-17, 21-22
#[derive(Debug, Clone)]
pub struct SdsfPanelState {
    /// Currently active sub-panel.
    pub active_panel: ActivePanel,
    /// The panel that MENU returns to (SET MAIN default).
    pub main_default: String,
    /// Current SCROLL ===> field value.
    pub scroll_amount: ScrollAmount,
    /// Most recent message shown in the title line message area.
    pub message: Option<String>,
    /// Current text in the COMMAND INPUT ===> field.
    pub command_input: String,
    /// Whether the main panel command groups are in grouped display mode.
    pub main_grouped: bool,
    /// Expanded state per group name (for grouped display).
    pub group_expanded: std::collections::HashMap<String, bool>,
    /// Total visible rows (for title line row range display).
    pub total_rows: usize,
    /// First visible row index (0-based, for title line).
    pub first_visible_row: usize,
    /// Visible row count (page size).
    pub page_size: usize,
}

impl Default for SdsfPanelState {
    fn default() -> Self {
        let mut group_expanded = std::collections::HashMap::new();
        for cmd in main_panel_commands() {
            group_expanded
                .entry(cmd.group.label().to_string())
                .or_insert(true);
        }
        Self {
            active_panel: ActivePanel::default(),
            main_default: "I".to_string(),
            scroll_amount: ScrollAmount::default(),
            message: None,
            command_input: String::new(),
            main_grouped: false,
            group_expanded,
            total_rows: 0,
            first_visible_row: 0,
            page_size: 25,
        }
    }
}

impl SdsfPanelState {
    /// Builds the title line string.
    ///
    /// Validates: Requirement 16.2
    pub fn title_line(&self) -> String {
        let panel_name = match &self.active_panel {
            ActivePanel::Main => "SDSF MAIN PANEL",
            ActivePanel::Sub(name) => name.as_str(),
        };
        let last = (self.first_visible_row + self.page_size).min(self.total_rows);
        if self.total_rows == 0 {
            format!("{panel_name} -- Row 0 to 0 of 0")
        } else {
            format!(
                "{panel_name} -- Row {} to {} of {}",
                self.first_visible_row + 1,
                last,
                self.total_rows
            )
        }
    }

    /// Sets the message area text.
    ///
    /// Validates: Requirement 16.21
    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    /// Clears the message area.
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    /// Navigates to the main panel (MENU command).
    ///
    /// Validates: Requirement 16.17
    pub fn navigate_to_main(&mut self) {
        self.active_panel = ActivePanel::Main;
        self.clear_message();
    }

    /// Navigates to a named sub-panel.
    ///
    /// Validates: Requirement 16.15
    pub fn navigate_to(&mut self, panel: &str) {
        self.active_panel = ActivePanel::Sub(panel.to_uppercase());
        self.clear_message();
    }

    /// Handles the SET MAIN GROUP command.
    ///
    /// Validates: Requirement 16.16
    pub fn set_main_grouped(&mut self, grouped: bool) {
        self.main_grouped = grouped;
    }

    /// Toggles a command group's expanded state.
    pub fn toggle_group(&mut self, group_label: &str) {
        let entry = self
            .group_expanded
            .entry(group_label.to_string())
            .or_insert(true);
        *entry = !*entry;
    }

    /// Returns commands visible in the main panel, respecting group expansion.
    ///
    /// Validates: Requirement 16.13, 16.14
    pub fn visible_main_commands(&self) -> Vec<&'static MainPanelCommand> {
        if !self.main_grouped {
            return main_panel_commands().iter().collect();
        }
        main_panel_commands()
            .iter()
            .filter(|cmd| {
                self.group_expanded
                    .get(cmd.group.label())
                    .copied()
                    .unwrap_or(true)
            })
            .collect()
    }

    /// Updates the SCROLL ===> field value.
    ///
    /// Validates: Requirement 16.3
    pub fn set_scroll_amount(&mut self, value: &str) -> bool {
        if let Some(amount) = ScrollAmount::parse(value) {
            self.scroll_amount = amount;
            true
        } else {
            false
        }
    }

    /// Returns the action bar menu names.
    ///
    /// Validates: Requirement 16.1
    pub fn action_bar_menus() -> &'static [&'static str] {
        &["File", "View", "Help"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ScrollAmount tests ---

    #[test]
    fn scroll_amount_parse_page() {
        // Validates: Requirement 16.3
        assert_eq!(ScrollAmount::parse("PAGE"), Some(ScrollAmount::Page));
    }

    #[test]
    fn scroll_amount_parse_half() {
        assert_eq!(ScrollAmount::parse("HALF"), Some(ScrollAmount::Half));
    }

    #[test]
    fn scroll_amount_parse_numeric() {
        assert_eq!(ScrollAmount::parse("10"), Some(ScrollAmount::Lines(10)));
    }

    #[test]
    fn scroll_amount_parse_invalid() {
        assert_eq!(ScrollAmount::parse("BOGUS"), None);
    }

    #[test]
    fn scroll_amount_display_round_trip() {
        // Validates: Requirement 16.3 -- retained across interactions
        let amounts = [
            ScrollAmount::Page,
            ScrollAmount::Half,
            ScrollAmount::Max,
            ScrollAmount::Lines(5),
        ];
        for a in &amounts {
            let s = a.display();
            assert_eq!(ScrollAmount::parse(&s).as_ref(), Some(a));
        }
    }

    // --- Main panel command tests ---

    #[test]
    fn main_panel_has_all_groups() {
        // Validates: Requirement 16.14
        let cmds = main_panel_commands();
        let groups: std::collections::HashSet<_> = cmds.iter().map(|c| c.group).collect();
        assert!(groups.contains(&CommandGroup::Jobs));
        assert!(groups.contains(&CommandGroup::Output));
        assert!(groups.contains(&CommandGroup::Jes));
        assert!(groups.contains(&CommandGroup::Log));
        assert!(groups.contains(&CommandGroup::Memory));
        assert!(groups.contains(&CommandGroup::Other));
    }

    #[test]
    fn main_panel_has_st_command() {
        // Validates: Requirement 16.13
        let cmds = main_panel_commands();
        assert!(cmds.iter().any(|c| c.name == "ST"));
    }

    #[test]
    fn main_panel_commands_have_descriptions() {
        // Validates: Requirement 16.13
        for cmd in main_panel_commands() {
            assert!(
                !cmd.description.is_empty(),
                "command {} has no description",
                cmd.name
            );
        }
    }

    // --- SdsfPanelState tests ---

    #[test]
    fn title_line_format_with_rows() {
        // Validates: Requirement 16.2
        let mut state = SdsfPanelState::default();
        state.total_rows = 47;
        state.first_visible_row = 0;
        state.page_size = 25;
        let title = state.title_line();
        assert!(title.contains("Row 1 to 25 of 47"), "got: {title}");
    }

    #[test]
    fn title_line_format_empty() {
        // Validates: Requirement 16.2
        let state = SdsfPanelState::default();
        let title = state.title_line();
        assert!(title.contains("Row 0 to 0 of 0"));
    }

    #[test]
    fn title_line_clamps_last_row() {
        // Validates: Requirement 16.2
        let mut state = SdsfPanelState::default();
        state.total_rows = 10;
        state.first_visible_row = 8;
        state.page_size = 25;
        let title = state.title_line();
        assert!(title.contains("Row 9 to 10 of 10"), "got: {title}");
    }

    #[test]
    fn message_area_set_and_clear() {
        // Validates: Requirement 16.21
        let mut state = SdsfPanelState::default();
        assert!(state.message.is_none());
        state.set_message("Command not valid");
        assert_eq!(state.message.as_deref(), Some("Command not valid"));
        state.clear_message();
        assert!(state.message.is_none());
    }

    #[test]
    fn navigate_to_main_sets_active_panel() {
        // Validates: Requirement 16.17
        let mut state = SdsfPanelState::default();
        state.navigate_to("ST");
        state.set_message("some message");
        state.navigate_to_main();
        assert_eq!(state.active_panel, ActivePanel::Main);
        assert!(state.message.is_none());
    }

    #[test]
    fn navigate_to_sub_panel() {
        // Validates: Requirement 16.15
        let mut state = SdsfPanelState::default();
        state.navigate_to_main();
        state.navigate_to("ST");
        assert_eq!(state.active_panel, ActivePanel::Sub("ST".to_string()));
    }

    #[test]
    fn set_main_grouped_toggles_display_mode() {
        // Validates: Requirement 16.16
        let mut state = SdsfPanelState::default();
        assert!(!state.main_grouped);
        state.set_main_grouped(true);
        assert!(state.main_grouped);
    }

    #[test]
    fn visible_main_commands_ungrouped_returns_all() {
        // Validates: Requirement 16.13
        let state = SdsfPanelState::default();
        assert!(!state.main_grouped);
        let visible = state.visible_main_commands();
        assert_eq!(visible.len(), main_panel_commands().len());
    }

    #[test]
    fn visible_main_commands_grouped_respects_collapse() {
        // Validates: Requirement 16.14
        let mut state = SdsfPanelState::default();
        state.set_main_grouped(true);
        // Collapse the Jobs group
        state.toggle_group("Jobs");
        let visible = state.visible_main_commands();
        assert!(visible.iter().all(|c| c.group != CommandGroup::Jobs));
    }

    #[test]
    fn scroll_amount_retained_across_interactions() {
        // Validates: Requirement 16.3
        let mut state = SdsfPanelState::default();
        assert!(state.set_scroll_amount("HALF"));
        assert_eq!(state.scroll_amount, ScrollAmount::Half);
        // Simulate another interaction -- value persists
        assert_eq!(state.scroll_amount, ScrollAmount::Half);
    }

    #[test]
    fn scroll_amount_invalid_not_applied() {
        // Validates: Requirement 16.3
        let mut state = SdsfPanelState::default();
        let before = state.scroll_amount.display();
        assert!(!state.set_scroll_amount("GARBAGE"));
        assert_eq!(state.scroll_amount.display(), before);
    }

    #[test]
    fn action_bar_has_file_view_help() {
        // Validates: Requirement 16.1
        let menus = SdsfPanelState::action_bar_menus();
        assert!(menus.contains(&"File"));
        assert!(menus.contains(&"View"));
        assert!(menus.contains(&"Help"));
    }

    #[test]
    fn command_input_field_is_writable() {
        // Validates: Requirement 16.22
        let mut state = SdsfPanelState::default();
        state.command_input = "PREFIX PAY".to_string();
        assert_eq!(state.command_input, "PREFIX PAY");
    }
}
