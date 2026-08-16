//! Capability Registry — dynamic index of all active capabilities.
//!
//! Tracks registered capabilities from all active plugins and provides
//! type-based and attribute-based query interfaces.

use std::sync::RwLock;

use crate::capability::{
    Capability, CapabilityDescriptor, CapabilityFilter, CapabilityType, CommandsCapability,
    LanguageSupportCapability, ViewersCapability,
};
use crate::error::PluginError;
use crate::event::PlatformEvent;
use crate::traits::PluginEventBus;
use std::sync::Arc;

/// Dynamic registry of all capabilities currently available.
///
/// Updated as plugins load and unload. Provides runtime query interfaces
/// for discovering what capabilities are registered.
///
/// Thread-safe via `RwLock` — frequent reads (queries), infrequent writes (load/unload).
pub struct CapabilityRegistry {
    /// All registered capability descriptors.
    descriptors: RwLock<Vec<CapabilityDescriptor>>,
    /// Counter for generating registration order values.
    next_order: RwLock<u64>,
    /// Event bus for emitting CapabilityChanged events.
    event_bus: Option<Arc<dyn PluginEventBus>>,
}

impl CapabilityRegistry {
    /// Creates a new empty capability registry without an event bus.
    pub fn new() -> Self {
        Self {
            descriptors: RwLock::new(Vec::new()),
            next_order: RwLock::new(0),
            event_bus: None,
        }
    }

    /// Creates a new capability registry with an event bus for emitting change events.
    pub fn with_event_bus(event_bus: Arc<dyn PluginEventBus>) -> Self {
        Self {
            descriptors: RwLock::new(Vec::new()),
            next_order: RwLock::new(0),
            event_bus: Some(event_bus),
        }
    }

    /// Register a capability for a plugin.
    ///
    /// If the same type + identifier already exists, emits a WARN log
    /// but still registers it. The first-registered provider remains the default.
    ///
    /// Emits a `CapabilityChanged` platform event on success.
    ///
    /// # Errors
    ///
    /// Currently infallible but returns `Result` for future extensibility.
    pub fn register(&self, owner: &str, capability: Capability) -> Result<(), PluginError> {
        let cap_type = capability.cap_type();
        let identifier = capability.identifier();

        let mut descriptors = self.descriptors.write().unwrap();
        let mut order = self.next_order.write().unwrap();

        // Check for duplicates (same type + identifier from different owner)
        let has_duplicate = descriptors.iter().any(|d| {
            d.capability.cap_type() == cap_type && d.capability.identifier() == identifier
        });

        if has_duplicate {
            // Log WARN about duplicate (via ff-logging if available)
            ff_logging::log(
                ff_logging::LogLevel::Warn,
                "capability_registry",
                &format!(
                    "duplicate capability registered: type={cap_type:?}, id={identifier}, owner={owner} (first-registered is default)"
                ),
            );
        }

        let descriptor = CapabilityDescriptor {
            capability,
            owner_plugin: owner.to_string(),
            registration_order: *order,
        };

        *order += 1;
        descriptors.push(descriptor);

        // Emit CapabilityChanged event
        if let Some(bus) = &self.event_bus {
            bus.emit(PlatformEvent::CapabilityChanged {
                capability_type: cap_type,
                owner_plugin: owner.to_string(),
                added: true,
            });
        }

        Ok(())
    }

    /// Remove all capabilities owned by a specific plugin.
    ///
    /// Emits `CapabilityChanged` events for each removal.
    pub fn unregister_all(&self, owner: &str) {
        let mut descriptors = self.descriptors.write().unwrap();
        let removed: Vec<_> = descriptors
            .iter()
            .filter(|d| d.owner_plugin == owner)
            .map(|d| d.capability.cap_type())
            .collect();

        descriptors.retain(|d| d.owner_plugin != owner);

        // Emit events for each removed capability type
        if let Some(bus) = &self.event_bus {
            for cap_type in removed {
                bus.emit(PlatformEvent::CapabilityChanged {
                    capability_type: cap_type,
                    owner_plugin: owner.to_string(),
                    added: false,
                });
            }
        }
    }

    /// Query all capabilities of a given type currently registered.
    ///
    /// Results are ordered by `registration_order` (ascending), so the
    /// first-registered provider appears first.
    pub fn query_by_type(&self, cap_type: CapabilityType) -> Vec<CapabilityDescriptor> {
        let descriptors = self.descriptors.read().unwrap();
        let mut results: Vec<_> = descriptors
            .iter()
            .filter(|d| d.capability.cap_type() == cap_type)
            .cloned()
            .collect();
        results.sort_by_key(|d| d.registration_order);
        results
    }

    /// Query capabilities matching metadata attributes.
    ///
    /// Filters by any combination of: capability type, MIME type (viewers),
    /// category (commands), language ID (language support), or owner plugin.
    pub fn query_by_attribute(&self, filter: &CapabilityFilter) -> Vec<CapabilityDescriptor> {
        let descriptors = self.descriptors.read().unwrap();
        descriptors
            .iter()
            .filter(|d| self.matches_filter(d, filter))
            .cloned()
            .collect()
    }

    /// Check if a specific capability type + identifier is registered.
    pub fn has_capability(&self, cap_type: CapabilityType, id: &str) -> bool {
        let descriptors = self.descriptors.read().unwrap();
        descriptors
            .iter()
            .any(|d| d.capability.cap_type() == cap_type && d.capability.identifier() == id)
    }

    /// Returns the total number of registered capabilities.
    pub fn len(&self) -> usize {
        self.descriptors.read().unwrap().len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.descriptors.read().unwrap().is_empty()
    }

    /// Returns all registered descriptors (used for testing ownership verification).
    pub fn all_descriptors(&self) -> Vec<CapabilityDescriptor> {
        self.descriptors.read().unwrap().clone()
    }

    /// Checks whether a descriptor matches the given filter.
    fn matches_filter(&self, descriptor: &CapabilityDescriptor, filter: &CapabilityFilter) -> bool {
        // Check type filter
        if let Some(cap_type) = filter.cap_type {
            if descriptor.capability.cap_type() != cap_type {
                return false;
            }
        }

        // Check owner filter
        if let Some(ref owner) = filter.owner {
            if &descriptor.owner_plugin != owner {
                return false;
            }
        }

        // Check MIME type filter (applies to Viewers)
        if let Some(ref mime_type) = filter.mime_type {
            match &descriptor.capability {
                Capability::Viewers(ViewersCapability { mime_types, .. }) => {
                    if !mime_types.contains(mime_type) {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        // Check category filter (applies to Commands)
        if let Some(ref category) = filter.category {
            match &descriptor.capability {
                Capability::Commands(CommandsCapability {
                    category: cap_cat, ..
                }) => {
                    if cap_cat != category {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        // Check language_id filter (applies to LanguageSupport)
        if let Some(ref language_id) = filter.language_id {
            match &descriptor.capability {
                Capability::LanguageSupport(LanguageSupportCapability { language_ids, .. }) => {
                    if !language_ids.contains(language_id) {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
    }
}

// Compile-time assertion that CapabilityRegistry is Send + Sync
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CapabilityRegistry>();
};

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::*;
    use crate::version::Version;

    fn make_commands_cap(category: &str) -> Capability {
        Capability::Commands(CommandsCapability {
            command_ids: vec!["test.cmd".to_string()],
            category: category.to_string(),
            version: Version::new(1, 0, 0),
        })
    }

    fn make_viewers_cap(mime: &str, name: &str) -> Capability {
        Capability::Viewers(ViewersCapability {
            mime_types: vec![mime.to_string()],
            display_name: name.to_string(),
            version: Version::new(1, 0, 0),
        })
    }

    fn make_lang_cap(lang_ids: &[&str]) -> Capability {
        Capability::LanguageSupport(LanguageSupportCapability {
            language_ids: lang_ids.iter().map(|s| s.to_string()).collect(),
            features: vec!["highlighting".to_string()],
            version: Version::new(1, 0, 0),
        })
    }

    #[test]
    fn register_and_query_by_type() {
        // Validates: Requirement 4.2, 4.3
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_commands_cap("file"))
            .unwrap();
        registry
            .register("plugin-b", make_viewers_cap("text/plain", "Text Viewer"))
            .unwrap();

        let commands = registry.query_by_type(CapabilityType::Commands);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].owner_plugin, "plugin-a");

        let viewers = registry.query_by_type(CapabilityType::Viewers);
        assert_eq!(viewers.len(), 1);
        assert_eq!(viewers[0].owner_plugin, "plugin-b");
    }

    #[test]
    fn unregister_all_removes_plugin_capabilities() {
        // Validates: Requirement 4.3, 5.6
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_commands_cap("file"))
            .unwrap();
        registry
            .register("plugin-a", make_viewers_cap("text/html", "HTML Viewer"))
            .unwrap();
        registry
            .register("plugin-b", make_commands_cap("edit"))
            .unwrap();

        assert_eq!(registry.len(), 3);

        registry.unregister_all("plugin-a");

        assert_eq!(registry.len(), 1);
        let commands = registry.query_by_type(CapabilityType::Commands);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].owner_plugin, "plugin-b");
    }

    #[test]
    fn query_by_attribute_filters_by_mime_type() {
        // Validates: Requirement 4.5
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_viewers_cap("text/plain", "Plain Viewer"))
            .unwrap();
        registry
            .register("plugin-b", make_viewers_cap("text/html", "HTML Viewer"))
            .unwrap();

        let filter = CapabilityFilter {
            mime_type: Some("text/plain".to_string()),
            ..Default::default()
        };
        let results = registry.query_by_attribute(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].owner_plugin, "plugin-a");
    }

    #[test]
    fn query_by_attribute_filters_by_category() {
        // Validates: Requirement 4.5
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_commands_cap("file"))
            .unwrap();
        registry
            .register("plugin-b", make_commands_cap("edit"))
            .unwrap();

        let filter = CapabilityFilter {
            category: Some("edit".to_string()),
            ..Default::default()
        };
        let results = registry.query_by_attribute(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].owner_plugin, "plugin-b");
    }

    #[test]
    fn query_by_attribute_filters_by_language_id() {
        // Validates: Requirement 4.5
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_lang_cap(&["rust", "toml"]))
            .unwrap();
        registry
            .register("plugin-b", make_lang_cap(&["python"]))
            .unwrap();

        let filter = CapabilityFilter {
            language_id: Some("rust".to_string()),
            ..Default::default()
        };
        let results = registry.query_by_attribute(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].owner_plugin, "plugin-a");
    }

    #[test]
    fn query_by_attribute_filters_by_owner() {
        // Validates: Requirement 4.5
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_commands_cap("file"))
            .unwrap();
        registry
            .register("plugin-b", make_commands_cap("edit"))
            .unwrap();

        let filter = CapabilityFilter {
            owner: Some("plugin-b".to_string()),
            ..Default::default()
        };
        let results = registry.query_by_attribute(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].owner_plugin, "plugin-b");
    }

    #[test]
    fn has_capability_returns_true_when_present() {
        // Validates: Requirement 4.2
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_commands_cap("file"))
            .unwrap();
        assert!(registry.has_capability(CapabilityType::Commands, "file"));
    }

    #[test]
    fn has_capability_returns_false_when_absent() {
        // Validates: Requirement 4.2
        let registry = CapabilityRegistry::new();
        assert!(!registry.has_capability(CapabilityType::Commands, "file"));
    }

    #[test]
    fn duplicate_capabilities_are_ordered_by_registration() {
        // Validates: Requirement 3.5
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_commands_cap("file"))
            .unwrap();
        registry
            .register("plugin-b", make_commands_cap("file"))
            .unwrap();

        let commands = registry.query_by_type(CapabilityType::Commands);
        assert_eq!(commands.len(), 2);
        // First-registered is first in results
        assert_eq!(commands[0].owner_plugin, "plugin-a");
        assert_eq!(commands[1].owner_plugin, "plugin-b");
        assert!(commands[0].registration_order < commands[1].registration_order);
    }

    #[test]
    fn capabilities_removed_immediately_on_unregister() {
        // Validates: Requirement 4.3
        let registry = CapabilityRegistry::new();
        registry
            .register("plugin-a", make_commands_cap("file"))
            .unwrap();
        assert!(registry.has_capability(CapabilityType::Commands, "file"));

        registry.unregister_all("plugin-a");
        assert!(!registry.has_capability(CapabilityType::Commands, "file"));
    }

    #[test]
    fn empty_registry_queries_return_empty() {
        // Validates: Requirement 4.2
        let registry = CapabilityRegistry::new();
        assert!(registry.query_by_type(CapabilityType::Commands).is_empty());
        assert!(registry.is_empty());
    }
}
