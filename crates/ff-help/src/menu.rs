//! Help Menu model — defines menu items for the Help menu bar entry.
//!
//! Provides menu item definitions and the About dialog information.

use crate::topic_key::TopicKey;

/// Information for the About dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct AboutInfo {
    /// Application name.
    pub app_name: String,
    /// Version string.
    pub version: String,
    /// Build date string.
    pub build_date: String,
    /// Rust compiler version used.
    pub rust_version: String,
    /// License name.
    pub license: String,
}

impl Default for AboutInfo {
    fn default() -> Self {
        Self {
            app_name: "FileForgeWorkbench".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_date: "unknown".to_string(),
            rust_version: "unknown".to_string(),
            license: "MIT".to_string(),
        }
    }
}

/// A Help menu item action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpMenuAction {
    /// Open Help Panel with Help Index.
    HelpIndex,
    /// Open Help Panel with command reference.
    CommandReference,
    /// Open Help Panel with line command reference.
    LineCommandReference,
    /// Open Help Panel with key bindings reference.
    KeyBindings,
    /// Show the About dialog.
    About,
}

impl HelpMenuAction {
    /// Resolve the action into a TopicKey for the Help Panel.
    ///
    /// Returns `None` for the About action (modal dialog, not a panel topic).
    pub fn topic_key(&self) -> Option<TopicKey> {
        match self {
            Self::HelpIndex => Some(TopicKey::index()),
            Self::CommandReference => Some(TopicKey::index()), // Commands section of index
            Self::LineCommandReference => Some(TopicKey::line_index()),
            Self::KeyBindings => Some(TopicKey::feature("function_keys")),
            Self::About => None,
        }
    }
}

/// A single item in the Help menu.
#[derive(Debug, Clone)]
pub struct HelpMenuItem {
    /// Display label.
    pub label: String,
    /// The action to perform when selected.
    pub action: HelpMenuAction,
    /// Whether this is a separator (label is ignored).
    pub is_separator: bool,
}

/// Returns the list of Help menu items per Requirement 14.1.
///
/// Items: Help Index, Command Reference, Line Command Reference,
/// Key Bindings, separator, About FileForgeWorkbench.
pub fn help_menu_items() -> Vec<HelpMenuItem> {
    vec![
        HelpMenuItem {
            label: "Help Index".to_string(),
            action: HelpMenuAction::HelpIndex,
            is_separator: false,
        },
        HelpMenuItem {
            label: "Command Reference".to_string(),
            action: HelpMenuAction::CommandReference,
            is_separator: false,
        },
        HelpMenuItem {
            label: "Line Command Reference".to_string(),
            action: HelpMenuAction::LineCommandReference,
            is_separator: false,
        },
        HelpMenuItem {
            label: "Key Bindings".to_string(),
            action: HelpMenuAction::KeyBindings,
            is_separator: false,
        },
        HelpMenuItem {
            label: String::new(),
            action: HelpMenuAction::About, // unused for separator
            is_separator: true,
        },
        HelpMenuItem {
            label: "About FileForgeWorkbench".to_string(),
            action: HelpMenuAction::About,
            is_separator: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 14.1 — Menu items list completeness
    #[test]
    fn help_menu_has_all_required_items() {
        let items = help_menu_items();
        // 5 real items + 1 separator = 6 total
        assert_eq!(items.len(), 6);
        assert_eq!(items[0].label, "Help Index");
        assert_eq!(items[1].label, "Command Reference");
        assert_eq!(items[2].label, "Line Command Reference");
        assert_eq!(items[3].label, "Key Bindings");
        assert!(items[4].is_separator);
        assert_eq!(items[5].label, "About FileForgeWorkbench");
    }

    // Validates: Requirement 14.2 — Help Index action resolves to index topic
    #[test]
    fn help_index_action_resolves_to_index() {
        assert_eq!(
            HelpMenuAction::HelpIndex.topic_key(),
            Some(TopicKey::index())
        );
    }

    // Validates: Requirement 14.4 — Line Command Reference action
    #[test]
    fn line_command_reference_resolves_to_line_index() {
        assert_eq!(
            HelpMenuAction::LineCommandReference.topic_key(),
            Some(TopicKey::line_index())
        );
    }

    // Validates: Requirement 14.5 — Key Bindings action
    #[test]
    fn key_bindings_resolves_to_function_keys() {
        assert_eq!(
            HelpMenuAction::KeyBindings.topic_key(),
            Some(TopicKey::feature("function_keys"))
        );
    }

    // Validates: Requirement 14.6 — About action has no topic key (modal dialog)
    #[test]
    fn about_action_has_no_topic_key() {
        assert_eq!(HelpMenuAction::About.topic_key(), None);
    }

    // Validates: Requirement 14.6 — About info populated
    #[test]
    fn about_info_has_app_details() {
        let info = AboutInfo::default();
        assert_eq!(info.app_name, "FileForgeWorkbench");
        assert_eq!(info.license, "MIT");
        assert!(!info.version.is_empty());
    }
}
