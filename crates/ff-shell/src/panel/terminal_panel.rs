//! Terminal Panel — dockable panel hosting tabbed interactive terminal sessions.
//!
//! Provides the UI-facing panel implementation for interactive terminal
//! sessions, with tab management and keyboard focus routing.

use ff_layout::{DockState, DockZone, DockablePanel};

use crate::terminal::manager::SessionId;

/// Terminal Panel — hosts tabbed interactive terminal sessions.
///
/// Registered as a `DockablePanel` with ID `"shell.terminal"` in the
/// `ff-layout` system, defaulting to the Bottom dock zone.
#[derive(Debug)]
pub struct TerminalPanel {
    /// The currently active (focused) tab.
    active_tab: Option<SessionId>,
    /// Whether this panel currently has keyboard focus.
    has_focus: bool,
    /// Current working directory display string.
    working_directory_display: String,
}

impl TerminalPanel {
    /// Creates a new terminal panel.
    pub fn new() -> Self {
        Self {
            active_tab: None,
            has_focus: false,
            working_directory_display: String::new(),
        }
    }

    /// Returns the currently active tab's session ID, if any.
    pub fn active_tab(&self) -> Option<SessionId> {
        self.active_tab
    }

    /// Sets the active tab to the given session ID.
    pub fn set_active_tab(&mut self, session_id: SessionId) {
        self.active_tab = Some(session_id);
    }

    /// Returns whether this panel currently has keyboard focus.
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    /// Sets the focus state of this panel.
    pub fn set_focus(&mut self, focused: bool) {
        self.has_focus = focused;
    }

    /// Updates the working directory display string.
    pub fn set_working_directory_display(&mut self, display: String) {
        self.working_directory_display = display;
    }

    /// Returns the working directory display string.
    pub fn working_directory_display(&self) -> &str {
        &self.working_directory_display
    }
}

impl Default for TerminalPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DockablePanel for TerminalPanel {
    fn panel_id(&self) -> &str {
        "shell.terminal"
    }

    fn default_dock_zone(&self) -> DockZone {
        DockZone::Bottom
    }

    fn title(&self) -> &str {
        "Terminal"
    }

    fn on_dock_state_changed(&mut self, state: DockState) {
        if state == DockState::Hidden {
            self.has_focus = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.2
    #[test]
    fn terminal_panel_has_correct_panel_id() {
        let panel = TerminalPanel::new();
        assert_eq!(panel.panel_id(), "shell.terminal");
        assert_eq!(panel.default_dock_zone(), DockZone::Bottom);
        assert_eq!(panel.title(), "Terminal");
    }

    // Validates: Requirement 7.4
    #[test]
    fn focus_management() {
        let mut panel = TerminalPanel::new();
        assert!(!panel.has_focus());
        panel.set_focus(true);
        assert!(panel.has_focus());
    }

    // Validates: Requirement 7.3
    #[test]
    fn hidden_state_clears_focus() {
        let mut panel = TerminalPanel::new();
        panel.set_focus(true);
        panel.on_dock_state_changed(DockState::Hidden);
        assert!(!panel.has_focus());
    }

    // Validates: Requirement 11.5
    #[test]
    fn working_directory_display() {
        let mut panel = TerminalPanel::new();
        panel.set_working_directory_display("/home/user/project".to_string());
        assert_eq!(panel.working_directory_display(), "/home/user/project");
    }

    // Validates: Requirement 7.7
    #[test]
    fn active_tab_management() {
        let mut panel = TerminalPanel::new();
        assert!(panel.active_tab().is_none());
        let id = SessionId::new();
        panel.set_active_tab(id);
        assert_eq!(panel.active_tab(), Some(id));
    }
}
