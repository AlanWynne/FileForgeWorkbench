//! Help Topic Registry — thread-safe indexed store of all help topics.
//!
//! Aggregates topics from file-based `.help.md` content, command metadata,
//! and plugin contributions. Supports O(1) lookup by `TopicKey`.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::topic::{HelpTopic, TopicSource};
use crate::topic_key::{TopicCategory, TopicKey};

/// Thread-safe store of all help topics, indexed by `TopicKey`.
///
/// Supports O(1) lookup, priority-based registration (runtime > file-based),
/// and plugin topic lifecycle management.
///
/// # Thread Safety
///
/// All read and write operations are safe from any thread. The registry uses
/// an internal `RwLock` to allow concurrent reads with exclusive writes.
#[derive(Debug, Clone)]
pub struct HelpTopicRegistry {
    /// TopicKey → HelpTopic mapping.
    topics: Arc<RwLock<HashMap<TopicKey, HelpTopic>>>,
    /// Plugin ID → set of TopicKeys contributed by that plugin (for cleanup).
    plugin_topics: Arc<RwLock<HashMap<String, Vec<TopicKey>>>>,
}

impl HelpTopicRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            plugin_topics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Look up a topic by key. Returns `None` if not found.
    pub fn get(&self, key: &TopicKey) -> Option<HelpTopic> {
        let topics = self.topics.read().ok()?;
        topics.get(key).cloned()
    }

    /// Check if a topic exists for the given key.
    pub fn contains(&self, key: &TopicKey) -> bool {
        self.topics
            .read()
            .map(|t| t.contains_key(key))
            .unwrap_or(false)
    }

    /// Register a topic from file-based content.
    ///
    /// Does NOT overwrite existing runtime-registered topics (lower priority).
    pub fn register_file_topic(&self, topic: HelpTopic) {
        let mut topics = match self.topics.write() {
            Ok(t) => t,
            Err(_) => return,
        };

        let key = topic.key().clone();
        if let Some(existing) = topics.get(&key) {
            // Only overwrite if existing is also file-based (same or lower priority)
            if existing.source().priority() <= topic.source().priority() {
                topics.insert(key, topic);
            }
        } else {
            topics.insert(key, topic);
        }
    }

    /// Register a topic from `CommandMetadata` (auto-created at command registration).
    ///
    /// Overwrites file-based topics for the same key.
    pub fn register_command_topic(&self, topic: HelpTopic) {
        let mut topics = match self.topics.write() {
            Ok(t) => t,
            Err(_) => return,
        };
        let key = topic.key().clone();
        topics.insert(key, topic);
    }

    /// Register a topic from a plugin. Associates with `plugin_id` for cleanup.
    ///
    /// Overwrites file-based topics for the same key.
    pub fn register_plugin_topic(&self, plugin_id: &str, topic: HelpTopic) {
        let key = topic.key().clone();

        // Insert or overwrite the topic
        if let Ok(mut topics) = self.topics.write() {
            topics.insert(key.clone(), topic);
        }

        // Track this key under the plugin for later deregistration
        if let Ok(mut plugin_map) = self.plugin_topics.write() {
            plugin_map
                .entry(plugin_id.to_string())
                .or_default()
                .push(key);
        }
    }

    /// Remove all topics contributed by the given plugin.
    pub fn deregister_plugin(&self, plugin_id: &str) {
        let keys_to_remove = {
            let mut plugin_map = match self.plugin_topics.write() {
                Ok(m) => m,
                Err(_) => return,
            };
            plugin_map.remove(plugin_id).unwrap_or_default()
        };

        if let Ok(mut topics) = self.topics.write() {
            for key in keys_to_remove {
                topics.remove(&key);
            }
        }
    }

    /// Bulk-register topics from a content load (startup or hot-reload).
    pub fn load_file_topics(&self, topics_to_load: Vec<HelpTopic>) {
        for topic in topics_to_load {
            self.register_file_topic(topic);
        }
    }

    /// Register a topic from command metadata.
    ///
    /// Creates a `HelpTopic` from the command's help_text and help_syntax fields.
    /// If `help_text` is empty, does not register (falls back to file-based).
    pub fn register_from_command_metadata(
        &self,
        command_id: &str,
        help_text: &str,
        help_syntax: &str,
    ) {
        if help_text.is_empty() {
            return; // Fall back to file-based content
        }

        let key = TopicKey::command(&command_id.to_uppercase());
        let title = format!("{} Command", command_id.to_uppercase());
        let mut body = String::new();
        if !help_syntax.is_empty() {
            body.push_str("## Syntax\n\n```\n");
            body.push_str(help_syntax);
            body.push_str("\n```\n\n");
        }
        body.push_str(help_text);

        let topic = HelpTopic::new(
            key,
            title,
            body,
            TopicSource::CommandRegistry {
                command_id: command_id.to_string(),
            },
        );

        self.register_command_topic(topic);
    }

    /// Remove a topic by key.
    pub fn unregister(&self, key: &TopicKey) {
        if let Ok(mut topics) = self.topics.write() {
            topics.remove(key);
        }
    }

    /// Return all registered `TopicKey`s.
    pub fn all_keys(&self) -> Vec<TopicKey> {
        self.topics
            .read()
            .map(|t| t.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Return all registered topics (cloned).
    pub fn all_topics(&self) -> Vec<HelpTopic> {
        self.topics
            .read()
            .map(|t| t.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Return all topics matching a given category.
    pub fn topics_by_category(&self, category: TopicCategory) -> Vec<HelpTopic> {
        self.topics
            .read()
            .map(|t| {
                t.values()
                    .filter(|topic| topic.key().category() == category)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Total number of registered topics.
    pub fn len(&self) -> usize {
        self.topics.read().map(|t| t.len()).unwrap_or(0)
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for HelpTopicRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn file_topic(key: TopicKey, title: &str, body: &str) -> HelpTopic {
        HelpTopic::new(
            key,
            title.to_string(),
            body.to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("test.help.md"),
            },
        )
    }

    fn cmd_topic(key: TopicKey, title: &str, body: &str, cmd_id: &str) -> HelpTopic {
        HelpTopic::new(
            key,
            title.to_string(),
            body.to_string(),
            TopicSource::CommandRegistry {
                command_id: cmd_id.to_string(),
            },
        )
    }

    fn plugin_topic(key: TopicKey, title: &str, body: &str, plugin_id: &str) -> HelpTopic {
        HelpTopic::new(
            key,
            title.to_string(),
            body.to_string(),
            TopicSource::Plugin {
                plugin_id: plugin_id.to_string(),
            },
        )
    }

    // Validates: Requirement 6.1 — Registry stores and retrieves topics
    #[test]
    fn register_and_get_topic() {
        let registry = HelpTopicRegistry::new();
        let key = TopicKey::command("FIND");
        let topic = file_topic(key.clone(), "FIND", "Find command help");

        registry.register_file_topic(topic);
        let retrieved = registry.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title(), "FIND");
    }

    // Validates: Requirement 6.1 — Contains check
    #[test]
    fn contains_returns_true_for_registered_key() {
        let registry = HelpTopicRegistry::new();
        let key = TopicKey::command("FIND");
        assert!(!registry.contains(&key));

        registry.register_file_topic(file_topic(key.clone(), "FIND", "body"));
        assert!(registry.contains(&key));
    }

    // Validates: Requirement 6.4 — Runtime topic overwrites file-based
    #[test]
    fn command_topic_overwrites_file_based() {
        let registry = HelpTopicRegistry::new();
        let key = TopicKey::command("FIND");

        registry.register_file_topic(file_topic(key.clone(), "File FIND", "from file"));
        registry.register_command_topic(cmd_topic(key.clone(), "Cmd FIND", "from command", "find"));

        let retrieved = registry.get(&key).unwrap();
        assert_eq!(retrieved.title(), "Cmd FIND");
    }

    // Validates: Requirement 6.4 — File-based does not overwrite runtime
    #[test]
    fn file_topic_does_not_overwrite_runtime() {
        let registry = HelpTopicRegistry::new();
        let key = TopicKey::command("FIND");

        registry.register_command_topic(cmd_topic(key.clone(), "Cmd FIND", "from command", "find"));
        registry.register_file_topic(file_topic(key.clone(), "File FIND", "from file"));

        let retrieved = registry.get(&key).unwrap();
        assert_eq!(retrieved.title(), "Cmd FIND");
    }

    // Validates: Requirement 6.5 — Empty help_text does not register
    #[test]
    fn empty_help_text_does_not_register() {
        let registry = HelpTopicRegistry::new();
        registry.register_from_command_metadata("FIND", "", "");
        assert!(!registry.contains(&TopicKey::command("FIND")));
    }

    // Validates: Requirement 6.5 — Non-empty help_text registers command topic
    #[test]
    fn non_empty_help_text_registers_topic() {
        let registry = HelpTopicRegistry::new();
        registry.register_from_command_metadata("FIND", "Searches for text", "FIND 'text'");
        assert!(registry.contains(&TopicKey::command("FIND")));

        let topic = registry.get(&TopicKey::command("FIND")).unwrap();
        assert!(topic.body().contains("Searches for text"));
        assert!(topic.body().contains("FIND 'text'"));
    }

    // Validates: Requirement 6.6 — Plugin deregistration removes topics
    #[test]
    fn deregister_plugin_removes_all_its_topics() {
        let registry = HelpTopicRegistry::new();
        let key1 = TopicKey::feature("custom1");
        let key2 = TopicKey::feature("custom2");

        registry.register_plugin_topic(
            "my_plugin",
            plugin_topic(key1.clone(), "Custom 1", "body", "my_plugin"),
        );
        registry.register_plugin_topic(
            "my_plugin",
            plugin_topic(key2.clone(), "Custom 2", "body", "my_plugin"),
        );

        assert!(registry.contains(&key1));
        assert!(registry.contains(&key2));

        registry.deregister_plugin("my_plugin");
        assert!(!registry.contains(&key1));
        assert!(!registry.contains(&key2));
    }

    // Validates: Requirement 6.1 — Category filtering
    #[test]
    fn topics_by_category_filters_correctly() {
        let registry = HelpTopicRegistry::new();
        registry.register_file_topic(file_topic(TopicKey::command("FIND"), "FIND", "body"));
        registry.register_file_topic(file_topic(TopicKey::command("CHANGE"), "CHANGE", "body"));
        registry.register_file_topic(file_topic(TopicKey::line_command("CC"), "CC", "body"));

        let commands = registry.topics_by_category(TopicCategory::Command);
        assert_eq!(commands.len(), 2);

        let line_cmds = registry.topics_by_category(TopicCategory::LineCommand);
        assert_eq!(line_cmds.len(), 1);
    }

    // Validates: Requirement 6.7 — Thread safety (basic smoke test)
    #[test]
    fn registry_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HelpTopicRegistry>();
    }

    // Validates: Requirement 6.1 — len and is_empty
    #[test]
    fn len_and_is_empty_reflect_state() {
        let registry = HelpTopicRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register_file_topic(file_topic(TopicKey::command("FIND"), "FIND", "body"));
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }
}
