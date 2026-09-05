//! Shell-side Automation Registry implementation for ff-desktop.
//!
//! `ShellAutomationRegistry` wraps `ff_fftest::automation::InMemoryAutomationRegistry`
//! and is owned by `WorkbenchShell`. The shell calls `begin_frame()` at the top
//! of each egui frame and `register()` for each rendered widget.
//!
//! Validates: Requirement 2.1, 2.4, 2.5 (automated-dialog-testing)

pub mod ids;

use ff_fftest::automation::InMemoryAutomationRegistry;
pub use ff_fftest::{AutomationId, AutomationRegistry, ControlState};

// === ShellAutomationRegistry ================================================

/// The concrete automation registry used by `WorkbenchShell`.
///
/// Thin wrapper around `InMemoryAutomationRegistry` that adds a convenience
/// `register_str` method so call sites can pass `&str` IDs directly.
///
/// Validates: Requirement 2.1, 2.5
#[derive(Debug, Default)]
pub struct ShellAutomationRegistry {
    inner: InMemoryAutomationRegistry,
}

impl ShellAutomationRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a control using a `&str` ID -- convenience wrapper.
    pub fn register_str(&mut self, id: &str, state: ControlState) {
        self.inner.register(AutomationId::new(id), state);
    }

    /// Query a control by `&str` ID -- convenience wrapper.
    #[allow(dead_code)]
    pub fn query_str(&self, id: &str) -> Option<&ControlState> {
        self.inner.query(&AutomationId::new(id))
    }
}

impl AutomationRegistry for ShellAutomationRegistry {
    fn begin_frame(&mut self) {
        self.inner.begin_frame();
    }

    fn register(&mut self, id: AutomationId, state: ControlState) {
        self.inner.register(id, state);
    }

    fn query(&self, id: &AutomationId) -> Option<&ControlState> {
        self.inner.query(id)
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.1 -- shell registry registers and queries controls
    #[test]
    fn shell_registry_register_and_query() {
        let mut reg = ShellAutomationRegistry::new();
        reg.register_str(ids::COMMAND_FIELD, ControlState::with_value("FIND HELLO"));
        let state = reg.query_str(ids::COMMAND_FIELD).expect("state present");
        assert_eq!(state.value.as_deref(), Some("FIND HELLO"));
    }

    // Validates: Requirement 2.5 -- begin_frame clears stale entries
    #[test]
    fn shell_registry_begin_frame_clears_entries() {
        let mut reg = ShellAutomationRegistry::new();
        reg.register_str(ids::STATUSBAR_MESSAGE, ControlState::with_value("OK"));
        assert!(reg.query_str(ids::STATUSBAR_MESSAGE).is_some());
        reg.begin_frame();
        assert!(reg.query_str(ids::STATUSBAR_MESSAGE).is_none());
    }

    // Validates: Requirement 2.4 -- all key ID constants are non-empty strings
    #[test]
    fn all_id_constants_are_non_empty() {
        let constants = [
            ids::COMMAND_FIELD,
            ids::SCROLL_FIELD,
            ids::STATUSBAR_MESSAGE,
            ids::STATUSBAR_LINE_COL,
            ids::STATUSBAR_ENCODING,
            ids::STATUSBAR_MODIFIED,
            ids::TAB_POM,
            ids::TAB_FILES_PANEL,
            ids::TAB_SETTINGS,
            ids::TAB_FILE_EXPLORER,
            ids::POM_EXIT,
            ids::POM_CALENDAR_PREV,
            ids::POM_CALENDAR_NEXT,
            ids::MENU_FILE_OPEN,
            ids::MENU_FILE_SAVE,
            ids::MENU_FILE_CLOSE,
            ids::MENU_HELP_ABOUT,
            ids::DIALOG_CATALOG_NAME,
            ids::DIALOG_CATALOG_CONFIRM,
            ids::DIALOG_CATALOG_CANCEL,
            ids::DIALOG_ALLOC_DSN,
            ids::DIALOG_ALLOC_CONFIRM,
            ids::DIALOG_ALLOC_CANCEL,
            ids::SETTINGS_FILTER,
            ids::DIALOG_KEYS_SAVE,
            ids::DIALOG_KEYS_CANCEL,
            ids::DIALOG_ABOUT_CLOSE,
            ids::EXPLORER_SIDEBAR,
            ids::EDITOR_CONTENT,
        ];
        for id in &constants {
            assert!(!id.is_empty(), "ID constant must not be empty: {id}");
        }
    }

    // Validates: Requirement 2.2 -- all ID constants follow dot-separated convention
    #[test]
    fn all_id_constants_contain_at_least_one_dot() {
        let constants = [
            ids::COMMAND_FIELD,
            ids::STATUSBAR_MESSAGE,
            ids::TAB_POM,
            ids::MENU_FILE_OPEN,
            ids::DIALOG_CATALOG_NAME,
            ids::EDITOR_CONTENT,
        ];
        for id in &constants {
            assert!(id.contains('.'), "ID '{id}' must contain at least one dot");
        }
    }

    // Validates: Requirement 2.4 -- prefix constants end with a dot
    #[test]
    fn prefix_constants_end_with_dot() {
        let prefixes = [
            ids::TAB_HEADER_PREFIX,
            ids::POM_OPTION_PREFIX,
            ids::MENU_BAR_PREFIX,
            ids::EXPLORER_CATALOG_PREFIX,
        ];
        for prefix in &prefixes {
            assert!(
                prefix.ends_with('.'),
                "Prefix '{prefix}' must end with a dot"
            );
        }
    }

    // Validates: Requirement 2.5 -- multiple controls registered and queried
    #[test]
    fn multiple_controls_registered_and_queried() {
        let mut reg = ShellAutomationRegistry::new();
        reg.register_str(ids::COMMAND_FIELD, ControlState::with_value("FIND"));
        reg.register_str(ids::STATUSBAR_MESSAGE, ControlState::with_value("Ready"));
        reg.register_str(ids::EDITOR_CONTENT, ControlState::active());

        assert_eq!(
            reg.query_str(ids::COMMAND_FIELD)
                .and_then(|s| s.value.as_deref()),
            Some("FIND")
        );
        assert_eq!(
            reg.query_str(ids::STATUSBAR_MESSAGE)
                .and_then(|s| s.value.as_deref()),
            Some("Ready")
        );
        assert!(reg.query_str(ids::EDITOR_CONTENT).is_some());
    }
}
