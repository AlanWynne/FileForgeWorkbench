//! Configuration layer definitions.
//!
//! Defines the `ConfigLayer` enum representing the six-layer priority model
//! used to merge configuration values.

use std::fmt;

/// A configuration layer in the layered override model.
///
/// Layers are listed in ascending priority order. When the same key is defined
/// in multiple layers, the highest-priority layer wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigLayer {
    /// Hardcoded defaults defined in code (lowest priority).
    Defaults = 0,
    /// System-wide configuration file.
    System = 1,
    /// Per-user configuration file.
    User = 2,
    /// Active named user profile overlay.
    Profile = 3,
    /// Project-level configuration (`.ffworkbench/config.toml`).
    Project = 4,
    /// Workspace-level configuration (highest priority).
    Workspace = 5,
}

impl PartialOrd for ConfigLayer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConfigLayer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl fmt::Display for ConfigLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Defaults => write!(f, "Defaults"),
            Self::System => write!(f, "System"),
            Self::User => write!(f, "User"),
            Self::Profile => write!(f, "Profile"),
            Self::Project => write!(f, "Project"),
            Self::Workspace => write!(f, "Workspace"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_ordering_defaults_is_lowest() {
        // Validates: Requirement 2.1 — Defaults is lowest priority
        assert!(ConfigLayer::Defaults < ConfigLayer::System);
        assert!(ConfigLayer::Defaults < ConfigLayer::User);
        assert!(ConfigLayer::Defaults < ConfigLayer::Profile);
        assert!(ConfigLayer::Defaults < ConfigLayer::Project);
        assert!(ConfigLayer::Defaults < ConfigLayer::Workspace);
    }

    #[test]
    fn layer_ordering_workspace_is_highest() {
        // Validates: Requirement 2.1 — Workspace is highest priority
        assert!(ConfigLayer::Workspace > ConfigLayer::Defaults);
        assert!(ConfigLayer::Workspace > ConfigLayer::System);
        assert!(ConfigLayer::Workspace > ConfigLayer::User);
        assert!(ConfigLayer::Workspace > ConfigLayer::Profile);
        assert!(ConfigLayer::Workspace > ConfigLayer::Project);
    }

    #[test]
    fn layer_ordering_full_ascending_chain() {
        // Validates: Requirement 2.1 — Fixed ascending priority order
        assert!(ConfigLayer::Defaults < ConfigLayer::System);
        assert!(ConfigLayer::System < ConfigLayer::User);
        assert!(ConfigLayer::User < ConfigLayer::Profile);
        assert!(ConfigLayer::Profile < ConfigLayer::Project);
        assert!(ConfigLayer::Project < ConfigLayer::Workspace);
    }

    #[test]
    fn layer_equality() {
        // Validates: Requirement 2.4 — Layer precedence is fixed
        assert_eq!(ConfigLayer::Defaults, ConfigLayer::Defaults);
        assert_eq!(ConfigLayer::Workspace, ConfigLayer::Workspace);
        assert_ne!(ConfigLayer::System, ConfigLayer::User);
    }

    #[test]
    fn layer_display_names() {
        // Validates: Requirement 2.1 — Each layer has a meaningful name
        assert_eq!(ConfigLayer::Defaults.to_string(), "Defaults");
        assert_eq!(ConfigLayer::System.to_string(), "System");
        assert_eq!(ConfigLayer::User.to_string(), "User");
        assert_eq!(ConfigLayer::Profile.to_string(), "Profile");
        assert_eq!(ConfigLayer::Project.to_string(), "Project");
        assert_eq!(ConfigLayer::Workspace.to_string(), "Workspace");
    }

    #[test]
    fn layer_discriminant_values() {
        // Validates: Requirement 2.4 — Layer ordering is fixed and deterministic
        assert_eq!(ConfigLayer::Defaults as u8, 0);
        assert_eq!(ConfigLayer::System as u8, 1);
        assert_eq!(ConfigLayer::User as u8, 2);
        assert_eq!(ConfigLayer::Profile as u8, 3);
        assert_eq!(ConfigLayer::Project as u8, 4);
        assert_eq!(ConfigLayer::Workspace as u8, 5);
    }

    #[test]
    fn layer_sort_produces_ascending_priority() {
        // Validates: Requirement 2.1 — Sorting layers yields ascending priority order
        let mut layers = vec![
            ConfigLayer::Workspace,
            ConfigLayer::Defaults,
            ConfigLayer::Profile,
            ConfigLayer::System,
            ConfigLayer::Project,
            ConfigLayer::User,
        ];
        layers.sort();
        assert_eq!(
            layers,
            vec![
                ConfigLayer::Defaults,
                ConfigLayer::System,
                ConfigLayer::User,
                ConfigLayer::Profile,
                ConfigLayer::Project,
                ConfigLayer::Workspace,
            ]
        );
    }
}
