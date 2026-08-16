//! `CommandMetadata` — descriptive information attached to registered commands.
//!
//! Used by menus, keybinding UI, help systems, and command palettes.

use crate::shortcut::ShortcutBinding;

/// Descriptive information attached to a registered command.
///
/// Provides display name, description, category, optional shortcut, and
/// optional icon reference for runtime inspection by UIs.
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Human-readable display name (localizable).
    pub display_name: String,
    /// One-sentence description of what the command does.
    pub description: String,
    /// Category derived from Command_ID prefix (e.g., "file", "edit").
    pub category: String,
    /// Optional default keyboard shortcut binding.
    pub default_shortcut: Option<ShortcutBinding>,
    /// Optional icon asset reference string.
    pub icon: Option<String>,
}

impl CommandMetadata {
    /// Creates a new metadata builder.
    pub fn builder(
        display_name: impl Into<String>,
        description: impl Into<String>,
    ) -> CommandMetadataBuilder {
        CommandMetadataBuilder {
            display_name: display_name.into(),
            description: description.into(),
            category: String::new(),
            default_shortcut: None,
            icon: None,
        }
    }
}

/// Builder for constructing `CommandMetadata`.
pub struct CommandMetadataBuilder {
    display_name: String,
    description: String,
    category: String,
    default_shortcut: Option<ShortcutBinding>,
    icon: Option<String>,
}

impl CommandMetadataBuilder {
    /// Sets the category string.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Sets the default keyboard shortcut binding.
    pub fn default_shortcut(mut self, binding: ShortcutBinding) -> Self {
        self.default_shortcut = Some(binding);
        self
    }

    /// Sets the icon reference.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Builds the `CommandMetadata`.
    pub fn build(self) -> CommandMetadata {
        CommandMetadata {
            display_name: self.display_name,
            description: self.description,
            category: self.category,
            default_shortcut: self.default_shortcut,
            icon: self.icon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.1
    #[test]
    fn metadata_builder_sets_display_name_and_description() {
        let meta = CommandMetadata::builder("Save File", "Saves the current file to disk")
            .category("file")
            .build();
        assert_eq!(meta.display_name, "Save File");
        assert_eq!(meta.description, "Saves the current file to disk");
        assert_eq!(meta.category, "file");
    }

    // Validates: Requirement 3.2
    #[test]
    fn metadata_default_shortcut_is_none_when_not_set() {
        let meta = CommandMetadata::builder("Test", "A test command")
            .category("test")
            .build();
        assert!(meta.default_shortcut.is_none());
    }

    // Validates: Requirement 3.3
    #[test]
    fn metadata_icon_is_none_when_not_set() {
        let meta = CommandMetadata::builder("Test", "A test command")
            .category("test")
            .build();
        assert!(meta.icon.is_none());
    }

    // Validates: Requirement 3.3
    #[test]
    fn metadata_icon_can_be_set() {
        let meta = CommandMetadata::builder("Save", "Save file")
            .category("file")
            .icon("icon_save")
            .build();
        assert_eq!(meta.icon.as_deref(), Some("icon_save"));
    }
}
