//! Persona data types — named layout configurations.

use crate::state::layout_state::LayoutState;

/// A named layout configuration that can be activated to switch the entire
/// workspace appearance with a single action.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Persona {
    /// Unique name for this persona (e.g., "Editor Focus", "Debug").
    pub name: String,
    /// Whether this is a built-in persona (cannot be deleted).
    pub built_in: bool,
    /// The layout state defining this persona's arrangement.
    pub layout: LayoutState,
    /// Optional description for UI display.
    pub description: Option<String>,
}

impl Persona {
    /// Creates a new custom persona from a name and layout state.
    pub fn custom(name: &str, layout: LayoutState) -> Self {
        Self {
            name: name.to_string(),
            built_in: false,
            layout,
            description: None,
        }
    }

    /// Creates a new built-in persona with a description.
    pub fn built_in(name: &str, description: &str, layout: LayoutState) -> Self {
        Self {
            name: name.to_string(),
            built_in: true,
            layout,
            description: Some(description.to_string()),
        }
    }

    /// Returns the kind of this persona (built-in or custom).
    pub fn kind(&self) -> PersonaKind {
        if self.built_in {
            PersonaKind::BuiltIn
        } else {
            PersonaKind::Custom
        }
    }
}

/// Identifies whether a persona is built-in or user-created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonaKind {
    /// A built-in persona (cannot be deleted).
    BuiltIn,
    /// A user-created persona.
    Custom,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::layout_state::LayoutState;

    #[test]
    fn persona_custom_is_not_built_in() {
        let persona = Persona::custom("My Layout", LayoutState::default());
        assert!(!persona.built_in);
        assert_eq!(persona.kind(), PersonaKind::Custom);
        assert_eq!(persona.name, "My Layout");
    }

    #[test]
    fn persona_built_in_has_description() {
        let persona = Persona::built_in("Editor Focus", "Minimal panels", LayoutState::default());
        assert!(persona.built_in);
        assert_eq!(persona.kind(), PersonaKind::BuiltIn);
        assert_eq!(persona.description.as_deref(), Some("Minimal panels"));
    }
}
