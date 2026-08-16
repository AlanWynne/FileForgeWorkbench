//! Capability types and descriptors.
//!
//! Defines the `Capability` enum and associated metadata structs for each
//! capability type. Capabilities are the typed services that plugins provide
//! to the platform.

use crate::version::Version;

/// A typed service or feature that a plugin provides to the platform.
///
/// Each variant carries metadata specific to that capability type,
/// including a version field for capability-level versioning.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Capability {
    /// Plugin provides one or more commands.
    Commands(CommandsCapability),
    /// Plugin provides one or more viewer implementations.
    Viewers(ViewersCapability),
    /// Plugin provides data or service providers.
    Providers(ProvidersCapability),
    /// Plugin provides language support (highlighting, completion, etc.).
    LanguageSupport(LanguageSupportCapability),
    /// Plugin contributes a theme.
    ThemeContribution(ThemeCapability),
}

impl Capability {
    /// Returns the type classification of this capability.
    pub fn cap_type(&self) -> CapabilityType {
        match self {
            Self::Commands(_) => CapabilityType::Commands,
            Self::Viewers(_) => CapabilityType::Viewers,
            Self::Providers(_) => CapabilityType::Providers,
            Self::LanguageSupport(_) => CapabilityType::LanguageSupport,
            Self::ThemeContribution(_) => CapabilityType::ThemeContribution,
        }
    }

    /// Returns an identifier string for this capability.
    ///
    /// Used for duplicate detection and querying.
    pub fn identifier(&self) -> String {
        match self {
            Self::Commands(c) => c.category.clone(),
            Self::Viewers(v) => v.display_name.clone(),
            Self::Providers(p) => p.provider_type.clone(),
            Self::LanguageSupport(l) => l.language_ids.join(","),
            Self::ThemeContribution(t) => t.theme_name.clone(),
        }
    }

    /// Returns the version of this capability.
    pub fn version(&self) -> &Version {
        match self {
            Self::Commands(c) => &c.version,
            Self::Viewers(v) => &v.version,
            Self::Providers(p) => &p.version,
            Self::LanguageSupport(l) => &l.version,
            Self::ThemeContribution(t) => &t.version,
        }
    }
}

/// Metadata for a Commands capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandsCapability {
    /// Identifiers of commands this plugin provides.
    pub command_ids: Vec<String>,
    /// Category grouping for discovery.
    pub category: String,
    /// Capability version.
    pub version: Version,
}

/// Metadata for a Viewers capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewersCapability {
    /// MIME types this viewer handles.
    pub mime_types: Vec<String>,
    /// Human-readable viewer name.
    pub display_name: String,
    /// Capability version.
    pub version: Version,
}

/// Metadata for a Providers capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidersCapability {
    /// Provider type identifier (e.g., "vfs", "data-source").
    pub provider_type: String,
    /// Capability version.
    pub version: Version,
}

/// Metadata for a LanguageSupport capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSupportCapability {
    /// Language identifiers this plugin supports.
    pub language_ids: Vec<String>,
    /// Features offered (highlighting, completion, diagnostics, etc.).
    pub features: Vec<String>,
    /// Capability version.
    pub version: Version,
}

/// Metadata for a ThemeContribution capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeCapability {
    /// Theme name.
    pub theme_name: String,
    /// Whether it is a dark or light theme.
    pub is_dark: bool,
    /// Capability version.
    pub version: Version,
}

/// Used for type-based capability queries.
///
/// Each variant corresponds to a `Capability` enum variant and enables
/// compile-time type verification of capability usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CapabilityType {
    /// Commands capability type.
    Commands,
    /// Viewers capability type.
    Viewers,
    /// Providers capability type.
    Providers,
    /// LanguageSupport capability type.
    LanguageSupport,
    /// ThemeContribution capability type.
    ThemeContribution,
}

/// A registered capability instance in the Capability Registry.
///
/// Associates a capability with its owning plugin and registration order
/// for first-registered-wins semantics.
#[derive(Debug, Clone)]
pub struct CapabilityDescriptor {
    /// The capability definition.
    pub capability: Capability,
    /// Plugin that owns this capability.
    pub owner_plugin: String,
    /// Registration order (for first-registered-wins semantics).
    pub registration_order: u64,
}

/// Filter criteria for attribute-based capability queries.
///
/// All fields are optional — only non-None fields participate in filtering.
#[derive(Debug, Clone, Default)]
pub struct CapabilityFilter {
    /// Optional: filter by capability type.
    pub cap_type: Option<CapabilityType>,
    /// Optional: filter by MIME type (for viewers).
    pub mime_type: Option<String>,
    /// Optional: filter by category (for commands).
    pub category: Option<String>,
    /// Optional: filter by language ID (for language support).
    pub language_id: Option<String>,
    /// Optional: filter by owning plugin name.
    pub owner: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_commands_type_classification() {
        // Validates: Requirement 4.1
        let cap = Capability::Commands(CommandsCapability {
            command_ids: vec!["cmd.open".to_string()],
            category: "file".to_string(),
            version: Version::new(1, 0, 0),
        });
        assert_eq!(cap.cap_type(), CapabilityType::Commands);
    }

    #[test]
    fn capability_viewers_type_classification() {
        // Validates: Requirement 4.1
        let cap = Capability::Viewers(ViewersCapability {
            mime_types: vec!["text/plain".to_string()],
            display_name: "Text Viewer".to_string(),
            version: Version::new(1, 0, 0),
        });
        assert_eq!(cap.cap_type(), CapabilityType::Viewers);
    }

    #[test]
    fn capability_providers_type_classification() {
        // Validates: Requirement 4.1
        let cap = Capability::Providers(ProvidersCapability {
            provider_type: "data-source".to_string(),
            version: Version::new(1, 0, 0),
        });
        assert_eq!(cap.cap_type(), CapabilityType::Providers);
    }

    #[test]
    fn capability_language_support_type_classification() {
        // Validates: Requirement 4.1
        let cap = Capability::LanguageSupport(LanguageSupportCapability {
            language_ids: vec!["rust".to_string()],
            features: vec!["highlighting".to_string()],
            version: Version::new(1, 0, 0),
        });
        assert_eq!(cap.cap_type(), CapabilityType::LanguageSupport);
    }

    #[test]
    fn capability_theme_type_classification() {
        // Validates: Requirement 4.1
        let cap = Capability::ThemeContribution(ThemeCapability {
            theme_name: "Dark Plus".to_string(),
            is_dark: true,
            version: Version::new(1, 0, 0),
        });
        assert_eq!(cap.cap_type(), CapabilityType::ThemeContribution);
    }

    #[test]
    fn capability_version_accessor() {
        // Validates: Requirement 6.6
        let cap = Capability::Commands(CommandsCapability {
            command_ids: vec![],
            category: "test".to_string(),
            version: Version::new(2, 1, 0),
        });
        assert_eq!(cap.version(), &Version::new(2, 1, 0));
    }

    #[test]
    fn capability_identifier_for_commands() {
        // Validates: Requirement 4.1
        let cap = Capability::Commands(CommandsCapability {
            command_ids: vec!["cmd.a".to_string()],
            category: "editing".to_string(),
            version: Version::new(1, 0, 0),
        });
        assert_eq!(cap.identifier(), "editing");
    }

    #[test]
    fn capability_filter_default_is_empty() {
        // Validates: Requirement 4.5
        let filter = CapabilityFilter::default();
        assert!(filter.cap_type.is_none());
        assert!(filter.mime_type.is_none());
        assert!(filter.category.is_none());
        assert!(filter.language_id.is_none());
        assert!(filter.owner.is_none());
    }
}
