//! Individual menu item types — action items, toggles, and command bindings.
//!
//! Every menu item is bound to a `CommandId` in the command framework. The item
//! delegates activation to `execute_command` and never mutates state directly.

/// An individual menu item bound to a command in the command framework.
///
/// Each item carries:
/// - A unique `id` for targeting (plugin contributions, testing)
/// - A display `label` shown in the menu
/// - An `access_key` for keyboard navigation
/// - A `command_id` linking to the command registry
/// - Shortcut text, enabled/visible state, and toggle semantics
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Unique identifier for this menu item (for contribution targeting).
    pub id: String,
    /// Display label (from command metadata or explicit override).
    pub label: String,
    /// Access key character for keyboard navigation.
    pub access_key: Option<char>,
    /// The Command_ID this item invokes when activated.
    pub command_id: String,
    /// Keyboard shortcut display text (read from ShortcutRegistry).
    pub shortcut_text: Option<String>,
    /// Whether the item is currently enabled (from command enabled predicate).
    pub is_enabled: bool,
    /// Whether the item is currently visible (from command visibility predicate).
    pub is_visible: bool,
    /// Whether this item represents a toggle (checkbox-style display).
    pub is_toggle: bool,
    /// Current toggle state (only meaningful if `is_toggle` is true).
    pub is_checked: bool,
    /// Contributing plugin name (None for built-in items).
    pub contributed_by: Option<String>,
}

impl MenuItem {
    /// Creates a new menu item with the given id, label, and command binding.
    ///
    /// The item is enabled, visible, and non-toggle by default.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        command_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            access_key: None,
            command_id: command_id.into(),
            shortcut_text: None,
            is_enabled: true,
            is_visible: true,
            is_toggle: false,
            is_checked: false,
            contributed_by: None,
        }
    }

    /// Sets the access key for keyboard navigation.
    pub fn with_access_key(mut self, key: char) -> Self {
        self.access_key = Some(key);
        self
    }

    /// Sets the shortcut display text.
    pub fn with_shortcut_text(mut self, text: impl Into<String>) -> Self {
        self.shortcut_text = Some(text.into());
        self
    }

    /// Marks this item as a toggle with the given initial checked state.
    pub fn as_toggle(mut self, checked: bool) -> Self {
        self.is_toggle = true;
        self.is_checked = checked;
        self
    }

    /// Sets the contributing plugin name.
    pub fn with_plugin(mut self, plugin: impl Into<String>) -> Self {
        self.contributed_by = Some(plugin.into());
        self
    }
}

/// Binding between a menu item and a command ID.
///
/// This struct captures the association and provides shortcut resolution
/// and predicate evaluation against the command registry.
#[derive(Debug, Clone)]
pub struct MenuCommandBinding {
    /// The menu item ID.
    pub item_id: String,
    /// The command ID in the command registry.
    pub command_id: String,
    /// Display name resolved from command metadata.
    pub display_name: String,
    /// Shortcut text resolved from the shortcut registry.
    pub shortcut_text: Option<String>,
    /// Optional icon identifier.
    pub icon: Option<String>,
    /// Access key character.
    pub access_key: Option<char>,
}

/// A toggle menu item binding that extends `MenuCommandBinding` with
/// a checked-state evaluation mechanism.
#[derive(Debug, Clone)]
pub struct ToggleBinding {
    /// The underlying command binding.
    pub binding: MenuCommandBinding,
    /// Whether the toggle is currently checked.
    pub is_checked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_item_new_has_correct_defaults() {
        let item = MenuItem::new("test_id", "Test Label", "test.command");
        assert_eq!(item.id, "test_id");
        assert_eq!(item.label, "Test Label");
        assert_eq!(item.command_id, "test.command");
        assert_eq!(item.access_key, None);
        assert_eq!(item.shortcut_text, None);
        assert!(item.is_enabled);
        assert!(item.is_visible);
        assert!(!item.is_toggle);
        assert!(!item.is_checked);
        assert_eq!(item.contributed_by, None);
    }

    #[test]
    fn menu_item_builder_methods_work() {
        let item = MenuItem::new("save", "Save", "file.save")
            .with_access_key('S')
            .with_shortcut_text("Ctrl+S")
            .as_toggle(true)
            .with_plugin("my-plugin");

        assert_eq!(item.access_key, Some('S'));
        assert_eq!(item.shortcut_text, Some("Ctrl+S".to_string()));
        assert!(item.is_toggle);
        assert!(item.is_checked);
        assert_eq!(item.contributed_by, Some("my-plugin".to_string()));
    }

    #[test]
    fn toggle_binding_carries_checked_state() {
        let binding = MenuCommandBinding {
            item_id: "view_wrap".to_string(),
            command_id: "view.word_wrap".to_string(),
            display_name: "Word Wrap".to_string(),
            shortcut_text: None,
            icon: None,
            access_key: Some('W'),
        };

        let toggle = ToggleBinding {
            binding,
            is_checked: true,
        };

        assert!(toggle.is_checked);
        assert_eq!(toggle.binding.command_id, "view.word_wrap");
    }
}
