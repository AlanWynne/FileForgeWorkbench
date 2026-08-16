//! Panel registry — tracks registered panel types and their default assignments.

use std::collections::HashMap;

use crate::dock::zone::DockZone;
use crate::error::LayoutError;

/// Information stored for each registered panel type.
#[derive(Debug, Clone)]
pub struct PanelRegistration {
    /// The unique panel identifier.
    pub panel_id: String,
    /// Default dock zone assignment.
    pub default_zone: DockZone,
    /// Display title.
    pub title: String,
}

/// Registry of all known panel types and their default assignments.
///
/// Plugins register panels here during initialization. The registry validates
/// panel_id format, checks for duplicates, and records default zone assignments.
#[derive(Debug, Clone)]
pub struct PanelRegistry {
    /// Map of panel_id → PanelRegistration.
    panels: HashMap<String, PanelRegistration>,
}

impl PanelRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
        }
    }

    /// Registers a panel type with the given ID, title, and default zone.
    ///
    /// # Errors
    ///
    /// Returns `LayoutError::InvalidPanelId` if the panel_id format is invalid.
    /// Returns `LayoutError::DuplicatePanelId` if the panel_id is already registered.
    pub fn register(
        &mut self,
        panel_id: &str,
        title: &str,
        default_zone: DockZone,
    ) -> Result<(), LayoutError> {
        // Validate panel_id format: 1–64 ASCII alphanumeric/underscore
        Self::validate_panel_id(panel_id)?;

        // Check for duplicate
        if self.panels.contains_key(panel_id) {
            return Err(LayoutError::DuplicatePanelId {
                panel_id: panel_id.to_string(),
            });
        }

        self.panels.insert(
            panel_id.to_string(),
            PanelRegistration {
                panel_id: panel_id.to_string(),
                default_zone,
                title: title.to_string(),
            },
        );

        Ok(())
    }

    /// Deregisters a panel (plugin unload). Returns true if the panel was found.
    pub fn deregister(&mut self, panel_id: &str) -> bool {
        self.panels.remove(panel_id).is_some()
    }

    /// Looks up a panel registration by ID.
    pub fn get(&self, panel_id: &str) -> Option<&PanelRegistration> {
        self.panels.get(panel_id)
    }

    /// Returns all registered panel IDs.
    pub fn list_all(&self) -> Vec<&str> {
        self.panels.keys().map(|s| s.as_str()).collect()
    }

    /// Returns whether a panel_id is currently registered.
    pub fn is_registered(&self, panel_id: &str) -> bool {
        self.panels.contains_key(panel_id)
    }

    /// Returns the number of registered panels.
    pub fn count(&self) -> usize {
        self.panels.len()
    }

    /// Validates a panel_id: must be 1–64 ASCII alphanumeric or underscore characters.
    fn validate_panel_id(panel_id: &str) -> Result<(), LayoutError> {
        if panel_id.is_empty() {
            return Err(LayoutError::InvalidPanelId {
                panel_id: panel_id.to_string(),
                reason: "panel_id must not be empty".to_string(),
            });
        }
        if panel_id.len() > 64 {
            return Err(LayoutError::InvalidPanelId {
                panel_id: panel_id.to_string(),
                reason: format!(
                    "panel_id must be at most 64 characters, got {}",
                    panel_id.len()
                ),
            });
        }
        if !panel_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(LayoutError::InvalidPanelId {
                panel_id: panel_id.to_string(),
                reason: "panel_id must contain only ASCII alphanumeric or underscore characters"
                    .to_string(),
            });
        }
        Ok(())
    }
}

impl Default for PanelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_valid_panel_succeeds() {
        // Validates: Requirement 1 criteria 2, 3
        let mut registry = PanelRegistry::new();
        let result = registry.register("file_tree", "File Tree", DockZone::Left);
        assert!(result.is_ok());
        assert!(registry.is_registered("file_tree"));
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn register_duplicate_panel_id_returns_error() {
        // Validates: Requirement 1 criterion 10
        let mut registry = PanelRegistry::new();
        registry
            .register("file_tree", "File Tree", DockZone::Left)
            .unwrap();
        let result = registry.register("file_tree", "File Tree 2", DockZone::Right);
        assert!(matches!(result, Err(LayoutError::DuplicatePanelId { .. })));
        // Registry unchanged
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn register_empty_panel_id_returns_error() {
        // Validates: Requirement 1 criterion 4
        let mut registry = PanelRegistry::new();
        let result = registry.register("", "Empty", DockZone::Left);
        assert!(matches!(result, Err(LayoutError::InvalidPanelId { .. })));
    }

    #[test]
    fn register_panel_id_too_long_returns_error() {
        // Validates: Requirement 1 criterion 4
        let mut registry = PanelRegistry::new();
        let long_id = "a".repeat(65);
        let result = registry.register(&long_id, "Too Long", DockZone::Left);
        assert!(matches!(result, Err(LayoutError::InvalidPanelId { .. })));
    }

    #[test]
    fn register_panel_id_with_invalid_chars_returns_error() {
        // Validates: Requirement 1 criterion 4
        let mut registry = PanelRegistry::new();
        let result = registry.register("file-tree", "File Tree", DockZone::Left);
        assert!(matches!(result, Err(LayoutError::InvalidPanelId { .. })));

        let result = registry.register("file tree", "File Tree", DockZone::Left);
        assert!(matches!(result, Err(LayoutError::InvalidPanelId { .. })));

        let result = registry.register("file.tree", "File Tree", DockZone::Left);
        assert!(matches!(result, Err(LayoutError::InvalidPanelId { .. })));
    }

    #[test]
    fn register_panel_id_at_max_length_succeeds() {
        let mut registry = PanelRegistry::new();
        let max_id = "a".repeat(64);
        let result = registry.register(&max_id, "Max Length", DockZone::Left);
        assert!(result.is_ok());
    }

    #[test]
    fn deregister_existing_panel_returns_true() {
        // Validates: Requirement 1 criterion 14
        let mut registry = PanelRegistry::new();
        registry
            .register("file_tree", "File Tree", DockZone::Left)
            .unwrap();
        assert!(registry.deregister("file_tree"));
        assert!(!registry.is_registered("file_tree"));
    }

    #[test]
    fn deregister_nonexistent_panel_returns_false() {
        let mut registry = PanelRegistry::new();
        assert!(!registry.deregister("nonexistent"));
    }

    #[test]
    fn get_returns_registration_details() {
        let mut registry = PanelRegistry::new();
        registry
            .register("output", "Output", DockZone::Bottom)
            .unwrap();
        let reg = registry.get("output").unwrap();
        assert_eq!(reg.panel_id, "output");
        assert_eq!(reg.title, "Output");
        assert_eq!(reg.default_zone, DockZone::Bottom);
    }

    #[test]
    fn list_all_returns_all_registered_ids() {
        let mut registry = PanelRegistry::new();
        registry
            .register("file_tree", "File Tree", DockZone::Left)
            .unwrap();
        registry
            .register("output", "Output", DockZone::Bottom)
            .unwrap();
        let mut ids = registry.list_all();
        ids.sort();
        assert_eq!(ids, vec!["file_tree", "output"]);
    }
}
