//! Persona manager — load, save, activate, and track layout personas.

use crate::error::LayoutError;
use crate::persona::definition::Persona;
use crate::state::layout_state::LayoutState;

/// Manages named layout personas (presets).
///
/// Handles activation, saving, deletion, and modification tracking for
/// both built-in and custom personas.
#[derive(Debug)]
pub struct PersonaManager {
    /// All known personas (built-in and custom).
    personas: Vec<Persona>,
    /// The currently active persona name.
    active_persona: Option<String>,
    /// Whether the layout has been modified from the active persona.
    is_modified: bool,
}

impl PersonaManager {
    /// Creates a new persona manager with built-in personas.
    pub fn new() -> Self {
        Self {
            personas: Self::built_in_personas(),
            active_persona: None,
            is_modified: false,
        }
    }

    /// Creates a persona manager with custom personas added.
    pub fn with_custom_personas(custom: Vec<Persona>) -> Self {
        let mut personas = Self::built_in_personas();
        personas.extend(custom);
        Self {
            personas,
            active_persona: None,
            is_modified: false,
        }
    }

    /// Returns the active persona name.
    pub fn active_persona_name(&self) -> Option<&str> {
        self.active_persona.as_deref()
    }

    /// Returns whether the layout has been modified from the active persona.
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    /// Marks the layout as modified from the active persona.
    pub fn mark_modified(&mut self) {
        if self.active_persona.is_some() {
            self.is_modified = true;
        }
    }

    /// Lists all available personas.
    pub fn list(&self) -> &[Persona] {
        &self.personas
    }

    /// Gets a persona by name.
    pub fn get(&self, name: &str) -> Option<&Persona> {
        self.personas.iter().find(|p| p.name == name)
    }

    /// Activates a persona by name, returning its layout state.
    ///
    /// # Errors
    ///
    /// Returns `PersonaNotFound` if no persona with the given name exists.
    pub fn activate(&mut self, name: &str) -> Result<&LayoutState, LayoutError> {
        if !self.personas.iter().any(|p| p.name == name) {
            return Err(LayoutError::PersonaNotFound {
                name: name.to_string(),
            });
        }
        self.active_persona = Some(name.to_string());
        self.is_modified = false;
        // Return the layout state
        Ok(&self
            .personas
            .iter()
            .find(|p| p.name == name)
            .unwrap()
            .layout)
    }

    /// Saves the current layout as a custom persona.
    pub fn save(&mut self, name: &str, layout: LayoutState) {
        // Check if a custom persona with this name already exists
        if let Some(existing) = self
            .personas
            .iter_mut()
            .find(|p| p.name == name && !p.built_in)
        {
            existing.layout = layout;
        } else {
            self.personas.push(Persona::custom(name, layout));
        }
        self.active_persona = Some(name.to_string());
        self.is_modified = false;
    }

    /// Deletes a custom persona.
    ///
    /// # Errors
    ///
    /// Returns `CannotDeleteBuiltIn` for built-in personas.
    /// Returns `PersonaNotFound` if the persona does not exist.
    pub fn delete(&mut self, name: &str) -> Result<(), LayoutError> {
        let pos = self
            .personas
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| LayoutError::PersonaNotFound {
                name: name.to_string(),
            })?;

        if self.personas[pos].built_in {
            return Err(LayoutError::CannotDeleteBuiltIn {
                name: name.to_string(),
            });
        }

        self.personas.remove(pos);

        // Clear active if we deleted the active persona
        if self.active_persona.as_deref() == Some(name) {
            self.active_persona = None;
            self.is_modified = false;
        }

        Ok(())
    }

    /// Updates the active persona to match the given layout state.
    pub fn update_active(&mut self, layout: LayoutState) -> Result<(), LayoutError> {
        let name = self
            .active_persona
            .as_ref()
            .ok_or_else(|| LayoutError::PersonaNotFound {
                name: "<none>".to_string(),
            })?
            .clone();

        if let Some(persona) = self.personas.iter_mut().find(|p| p.name == name) {
            persona.layout = layout;
            self.is_modified = false;
            Ok(())
        } else {
            Err(LayoutError::PersonaNotFound { name })
        }
    }

    /// Creates the built-in persona definitions.
    fn built_in_personas() -> Vec<Persona> {
        vec![
            Persona::built_in(
                "Editor Focus",
                "Minimal panels, maximized editor area",
                LayoutState::default(),
            ),
            Persona::built_in(
                "Debug",
                "Output and variable panels visible",
                LayoutState::default(),
            ),
            Persona::built_in(
                "FileForge",
                "File tree and structure panels prominent",
                LayoutState::default(),
            ),
            Persona::built_in(
                "Database",
                "Schema browser, SQL editor, result grid visible",
                LayoutState::default(),
            ),
        ]
    }
}

impl Default for PersonaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persona::definition::PersonaKind;

    #[test]
    fn new_manager_has_built_in_personas() {
        // Validates: Requirement 5 criterion 2
        let mgr = PersonaManager::new();
        let names: Vec<&str> = mgr.list().iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Editor Focus"));
        assert!(names.contains(&"Debug"));
        assert!(names.contains(&"FileForge"));
        assert!(names.contains(&"Database"));
    }

    #[test]
    fn activate_sets_active_persona() {
        // Validates: Requirement 5 criterion 4
        let mut mgr = PersonaManager::new();
        mgr.activate("Debug").unwrap();
        assert_eq!(mgr.active_persona_name(), Some("Debug"));
        assert!(!mgr.is_modified());
    }

    #[test]
    fn activate_nonexistent_returns_error() {
        let mut mgr = PersonaManager::new();
        let result = mgr.activate("Nonexistent");
        assert!(matches!(result, Err(LayoutError::PersonaNotFound { .. })));
    }

    #[test]
    fn save_custom_persona() {
        // Validates: Requirement 5 criterion 3
        let mut mgr = PersonaManager::new();
        let layout = LayoutState::default();
        mgr.save("My Custom", layout);
        assert!(mgr.get("My Custom").is_some());
        assert_eq!(mgr.get("My Custom").unwrap().kind(), PersonaKind::Custom);
    }

    #[test]
    fn delete_custom_persona_succeeds() {
        // Validates: Requirement 5 criterion 6
        let mut mgr = PersonaManager::new();
        mgr.save("My Custom", LayoutState::default());
        mgr.delete("My Custom").unwrap();
        assert!(mgr.get("My Custom").is_none());
    }

    #[test]
    fn delete_built_in_persona_fails() {
        // Validates: Requirement 5 criterion 6
        let mut mgr = PersonaManager::new();
        let result = mgr.delete("Editor Focus");
        assert!(matches!(
            result,
            Err(LayoutError::CannotDeleteBuiltIn { .. })
        ));
    }

    #[test]
    fn mark_modified_tracks_changes() {
        // Validates: Requirement 5 criterion 10
        let mut mgr = PersonaManager::new();
        mgr.activate("Debug").unwrap();
        assert!(!mgr.is_modified());
        mgr.mark_modified();
        assert!(mgr.is_modified());
    }

    #[test]
    fn mark_modified_does_nothing_without_active_persona() {
        let mut mgr = PersonaManager::new();
        mgr.mark_modified();
        assert!(!mgr.is_modified());
    }
}
