//! Automation ID types and registry trait.
//!
//! Validates: Requirement 2.1, 2.2, 2.3, 2.5 (automated-dialog-testing)

use std::collections::HashMap;

// === AutomationId ===========================================================

/// A stable dot-separated identifier for a UI control.
///
/// IDs follow the convention `<panel>.<group>.<control>`, e.g.:
/// - `menu.file.open`
/// - `button.save`
/// - `textbox.command_field`
/// - `dialog.catalog_manager.name_field`
///
/// Validates: Requirement 2.2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutomationId(String);

impl AutomationId {
    /// Create a new `AutomationId` from a static string.
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    /// Return the raw string value of this ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AutomationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// === ControlState ===========================================================

/// The observable state of a UI control at query time.
///
/// Validates: Requirement 2.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlState {
    /// Whether the control is currently rendered and visible.
    pub visible: bool,
    /// Whether the control accepts user interaction.
    pub enabled: bool,
    /// The current text value or selected item, if applicable.
    pub value: Option<String>,
    /// The display label of the control, if applicable.
    pub label: Option<String>,
}

impl ControlState {
    /// Construct a simple visible+enabled control with no value or label.
    pub fn active() -> Self {
        Self {
            visible: true,
            enabled: true,
            value: None,
            label: None,
        }
    }

    /// Construct a visible+enabled control with a text value.
    pub fn with_value(value: impl Into<String>) -> Self {
        Self {
            visible: true,
            enabled: true,
            value: Some(value.into()),
            label: None,
        }
    }

    /// Construct a visible+enabled control with a label.
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            visible: true,
            enabled: true,
            value: None,
            label: Some(label.into()),
        }
    }
}

// === AutomationRegistry trait ===============================================

/// Trait implemented by the shell to expose UI control state to the FFTest runner.
///
/// The shell calls `begin_frame()` at the start of each egui frame to clear
/// stale entries, then calls `register()` for each rendered widget.
/// The runner calls `query()` between frames to read control state.
///
/// This trait has no egui dependency -- it operates on plain Rust types only.
///
/// Validates: Requirement 2.1, 2.5
pub trait AutomationRegistry: Send + Sync {
    /// Clear all registered controls from the previous frame.
    ///
    /// Must be called at the start of each egui frame before any `register()` calls.
    fn begin_frame(&mut self);

    /// Register a control's current state under the given Automation ID.
    ///
    /// Called once per rendered widget per frame.
    fn register(&mut self, id: AutomationId, state: ControlState);

    /// Query the current state of a control by Automation ID.
    ///
    /// Returns `None` if the control was not registered in the current frame
    /// (i.e., it is not currently rendered).
    fn query(&self, id: &AutomationId) -> Option<&ControlState>;

    /// Returns true if the given Automation ID was registered in the current frame.
    fn is_present(&self, id: &AutomationId) -> bool {
        self.query(id).is_some()
    }
}

// === InMemoryAutomationRegistry =============================================

/// A simple in-memory implementation of `AutomationRegistry` for use in
/// `ff-desktop` and in tests.
///
/// Validates: Requirement 2.1, 2.5
#[derive(Debug, Default)]
pub struct InMemoryAutomationRegistry {
    controls: HashMap<AutomationId, ControlState>,
}

impl InMemoryAutomationRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }
}

impl AutomationRegistry for InMemoryAutomationRegistry {
    fn begin_frame(&mut self) {
        self.controls.clear();
    }

    fn register(&mut self, id: AutomationId, state: ControlState) {
        self.controls.insert(id, state);
    }

    fn query(&self, id: &AutomationId) -> Option<&ControlState> {
        self.controls.get(id)
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.1 -- every control gets a stable Automation ID
    #[test]
    fn automation_id_stores_and_returns_string() {
        let id = AutomationId::new("menu.file.open");
        assert_eq!(id.as_str(), "menu.file.open");
    }

    // Validates: Requirement 2.2 -- dot-separated hierarchical naming
    #[test]
    fn automation_id_display_matches_inner_string() {
        let id = AutomationId::new("button.save");
        assert_eq!(id.to_string(), "button.save");
    }

    // Validates: Requirement 2.5 -- query returns None before registration
    #[test]
    fn query_returns_none_for_unregistered_id() {
        let registry = InMemoryAutomationRegistry::new();
        let id = AutomationId::new("button.save");
        assert!(registry.query(&id).is_none());
    }

    // Validates: Requirement 2.5 -- query returns state after registration
    #[test]
    fn register_then_query_returns_state() {
        let mut registry = InMemoryAutomationRegistry::new();
        let id = AutomationId::new("button.save");
        registry.register(id.clone(), ControlState::active());
        let state = registry.query(&id).expect("state present");
        assert!(state.visible);
        assert!(state.enabled);
    }

    // Validates: Requirement 2.5 -- begin_frame clears stale entries
    #[test]
    fn begin_frame_clears_previous_registrations() {
        let mut registry = InMemoryAutomationRegistry::new();
        let id = AutomationId::new("textbox.command_field");
        registry.register(id.clone(), ControlState::active());
        assert!(registry.is_present(&id));
        registry.begin_frame();
        assert!(!registry.is_present(&id));
    }

    // Validates: Requirement 2.5 -- value round-trips through ControlState
    #[test]
    fn control_state_with_value_stores_text() {
        let state = ControlState::with_value("FIND HELLO");
        assert_eq!(state.value.as_deref(), Some("FIND HELLO"));
        assert!(state.visible);
        assert!(state.enabled);
    }

    // Validates: Requirement 2.5 -- label round-trips through ControlState
    #[test]
    fn control_state_with_label_stores_label() {
        let state = ControlState::with_label("Save");
        assert_eq!(state.label.as_deref(), Some("Save"));
    }

    // Validates: Requirement 2.3 -- is_present helper works correctly
    #[test]
    fn is_present_returns_false_for_absent_id() {
        let registry = InMemoryAutomationRegistry::new();
        assert!(!registry.is_present(&AutomationId::new("menu.file.open")));
    }

    // Validates: Requirement 2.5 -- multiple controls registered in same frame
    #[test]
    fn multiple_controls_registered_in_same_frame() {
        let mut registry = InMemoryAutomationRegistry::new();
        let ids = [
            "menu.file.open",
            "button.save",
            "textbox.command_field",
            "statusbar.message",
        ];
        for id_str in &ids {
            registry.register(AutomationId::new(id_str), ControlState::active());
        }
        for id_str in &ids {
            assert!(registry.is_present(&AutomationId::new(id_str)));
        }
    }

    // Validates: Requirement 2.5 -- re-registering same ID in same frame overwrites
    #[test]
    fn re_register_same_id_overwrites_state() {
        let mut registry = InMemoryAutomationRegistry::new();
        let id = AutomationId::new("textbox.command_field");
        registry.register(id.clone(), ControlState::with_value("FIND"));
        registry.register(id.clone(), ControlState::with_value("CHANGE"));
        let state = registry.query(&id).expect("state present");
        assert_eq!(state.value.as_deref(), Some("CHANGE"));
    }
}
