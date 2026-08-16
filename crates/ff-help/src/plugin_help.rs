//! Plugin help registration bridge.
//!
//! Provides the interface for plugins to register and deregister help topics
//! during their lifecycle phases.

use std::sync::Arc;

use crate::registry::HelpTopicRegistry;
use crate::topic::{HelpTopic, TopicSource};
use crate::topic_key::TopicKey;

/// Interface for plugins to register and deregister help topics.
///
/// Exposed via `PluginContext` during plugin lifecycle.
pub struct HelpPluginBridge {
    registry: Arc<HelpTopicRegistry>,
}

impl HelpPluginBridge {
    /// Create a new plugin bridge backed by the given registry.
    pub fn new(registry: Arc<HelpTopicRegistry>) -> Self {
        Self { registry }
    }

    /// Register a help topic from a plugin.
    ///
    /// Called during the plugin `initialize` lifecycle phase.
    pub fn register_topic(
        &self,
        plugin_id: &str,
        key: TopicKey,
        title: String,
        markdown_content: String,
    ) {
        let topic = HelpTopic::new(
            key,
            title,
            markdown_content,
            TopicSource::Plugin {
                plugin_id: plugin_id.to_string(),
            },
        );
        self.registry.register_plugin_topic(plugin_id, topic);
    }

    /// Remove all help topics contributed by this plugin.
    ///
    /// Called during the plugin `shutdown` lifecycle phase.
    pub fn deregister_all(&self, plugin_id: &str) {
        self.registry.deregister_plugin(plugin_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 6.1 — Plugin registers topics
    #[test]
    fn plugin_bridge_registers_topic() {
        let registry = Arc::new(HelpTopicRegistry::new());
        let bridge = HelpPluginBridge::new(registry.clone());

        let key = TopicKey::feature("custom_feature");
        bridge.register_topic(
            "my_plugin",
            key.clone(),
            "Custom Feature".to_string(),
            "This is custom help.".to_string(),
        );

        assert!(registry.contains(&key));
        let topic = registry.get(&key).unwrap();
        assert_eq!(topic.title(), "Custom Feature");
    }

    // Validates: Requirement 6.6 — Plugin deregistration removes topics
    #[test]
    fn plugin_bridge_deregisters_all_topics() {
        let registry = Arc::new(HelpTopicRegistry::new());
        let bridge = HelpPluginBridge::new(registry.clone());

        let key1 = TopicKey::feature("feat1");
        let key2 = TopicKey::feature("feat2");
        bridge.register_topic(
            "my_plugin",
            key1.clone(),
            "Feat 1".to_string(),
            "body".to_string(),
        );
        bridge.register_topic(
            "my_plugin",
            key2.clone(),
            "Feat 2".to_string(),
            "body".to_string(),
        );

        assert!(registry.contains(&key1));
        assert!(registry.contains(&key2));

        bridge.deregister_all("my_plugin");
        assert!(!registry.contains(&key1));
        assert!(!registry.contains(&key2));
    }

    // Validates: Requirement 6.4 — Plugin topic overrides file-based
    #[test]
    fn plugin_topic_overrides_file_based() {
        let registry = Arc::new(HelpTopicRegistry::new());
        let key = TopicKey::feature("existing");

        // Register file-based first
        registry.register_file_topic(HelpTopic::new(
            key.clone(),
            "File Version".to_string(),
            "from file".to_string(),
            TopicSource::FileBased {
                file_path: std::path::PathBuf::from("test.help.md"),
            },
        ));

        // Plugin override
        let bridge = HelpPluginBridge::new(registry.clone());
        bridge.register_topic(
            "my_plugin",
            key.clone(),
            "Plugin Version".to_string(),
            "from plugin".to_string(),
        );

        let topic = registry.get(&key).unwrap();
        assert_eq!(topic.title(), "Plugin Version");
    }
}
