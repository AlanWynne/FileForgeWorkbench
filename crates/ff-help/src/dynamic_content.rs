//! Dynamic content generation — produces help topics at display time.
//!
//! Generates the function keys help topic from the active key map and
//! the Help Index from the current registry state.

use crate::registry::HelpTopicRegistry;
use crate::topic::HelpTopic;
use crate::topic_key::{TopicCategory, TopicKey};

/// A single function key binding entry for help display.
#[derive(Debug, Clone)]
pub struct FunctionKeyBinding {
    /// Key name, e.g., "F3".
    pub key: String,
    /// Bound command ID, e.g., "file.close".
    pub command_id: String,
    /// Display label, e.g., "Close".
    pub label: String,
}

/// Trait for reading key map state — consumed by `ff-help` for dynamic generation.
///
/// Implemented by the key map system; decouples `ff-help` from `ff-keys`
/// implementation details.
pub trait KeyMapAccess: Send + Sync {
    /// Returns all assigned function key bindings (F1–F24).
    fn function_key_bindings(&self) -> Vec<FunctionKeyBinding>;
    /// Returns the name of the active profile (if any).
    fn active_profile_name(&self) -> Option<String>;
}

/// Generates help topics dynamically at display time (not stored in registry).
pub struct DynamicContentGenerator;

impl DynamicContentGenerator {
    /// Generate the function keys help topic from a key map accessor.
    ///
    /// Produces a Markdown table: Key | Command | Label.
    pub fn generate_function_keys(key_map: &dyn KeyMapAccess) -> HelpTopic {
        let bindings = key_map.function_key_bindings();
        let profile = key_map.active_profile_name();

        let mut body = String::from("# Function Key Assignments\n\n");

        if let Some(ref name) = profile {
            body.push_str(&format!("**Active Profile:** {name}\n\n"));
        }

        if bindings.is_empty() {
            body.push_str(
                "No function keys are currently assigned.\n\n\
                 To configure function keys, add entries to the `[keys]` section\n\
                 of your configuration file.\n",
            );
        } else {
            body.push_str("| Key | Command | Label |\n");
            body.push_str("|-----|---------|-------|\n");
            for binding in &bindings {
                body.push_str(&format!(
                    "| {} | {} | {} |\n",
                    binding.key, binding.command_id, binding.label
                ));
            }
        }

        HelpTopic::new(
            TopicKey::feature("function_keys"),
            "Function Key Assignments".to_string(),
            body,
            crate::topic::TopicSource::FileBased {
                file_path: std::path::PathBuf::from("<dynamic>"),
            },
        )
    }

    /// Generate the Help Index topic from the current registry state.
    ///
    /// Organises topics by category with navigable links.
    pub fn generate_index(registry: &HelpTopicRegistry, app_version: &str) -> HelpTopic {
        let mut body = String::from("# Help Index\n\n");

        // Getting Started
        body.push_str("## Getting Started\n\n");
        body.push_str("- [Getting Started Guide](getting_started)\n\n");

        // Primary Commands
        body.push_str("## Primary Commands\n\n");
        let mut commands = registry.topics_by_category(TopicCategory::Command);
        commands.sort_by(|a, b| a.title().cmp(b.title()));
        for topic in &commands {
            body.push_str(&format!("- [{}]({})\n", topic.title(), topic.key()));
        }
        body.push('\n');

        // Line Commands
        body.push_str("## Line Commands\n\n");
        body.push_str("- [Line Command Reference](line:index)\n");
        let mut line_cmds = registry.topics_by_category(TopicCategory::LineCommand);
        line_cmds.sort_by(|a, b| a.title().cmp(b.title()));
        for topic in &line_cmds {
            body.push_str(&format!("- [{}]({})\n", topic.title(), topic.key()));
        }
        body.push('\n');

        // Modes
        body.push_str("## Modes\n\n");
        let modes = registry.topics_by_category(TopicCategory::Mode);
        for topic in &modes {
            body.push_str(&format!("- [{}]({})\n", topic.title(), topic.key()));
        }
        body.push('\n');

        // Features
        body.push_str("## Features\n\n");
        let features = registry.topics_by_category(TopicCategory::Feature);
        for topic in &features {
            body.push_str(&format!("- [{}]({})\n", topic.title(), topic.key()));
        }
        body.push('\n');

        // Configuration
        body.push_str("## Configuration\n\n");
        body.push_str("- [Configuration Overview](feature:configuration)\n\n");

        // Function Keys
        body.push_str("## Function Keys\n\n");
        body.push_str("- [Function Key Assignments](feature:function_keys)\n\n");

        // Macro API
        body.push_str("## Macro API\n\n");
        body.push_str("- [Macro API Reference](feature:macros)\n\n");

        // Footer
        body.push_str("---\n\n");
        body.push_str(&format!("*{app_version}*\n"));

        HelpTopic::new(
            TopicKey::index(),
            "Help Index".to_string(),
            body,
            crate::topic::TopicSource::FileBased {
                file_path: std::path::PathBuf::from("<dynamic>"),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockKeyMap {
        bindings: Vec<FunctionKeyBinding>,
        profile: Option<String>,
    }

    impl KeyMapAccess for MockKeyMap {
        fn function_key_bindings(&self) -> Vec<FunctionKeyBinding> {
            self.bindings.clone()
        }
        fn active_profile_name(&self) -> Option<String> {
            self.profile.clone()
        }
    }

    // Validates: Requirement 15.1 — Function keys topic generated from key map
    #[test]
    fn generate_function_keys_with_bindings() {
        let key_map = MockKeyMap {
            bindings: vec![
                FunctionKeyBinding {
                    key: "F3".to_string(),
                    command_id: "file.close".to_string(),
                    label: "Close".to_string(),
                },
                FunctionKeyBinding {
                    key: "F5".to_string(),
                    command_id: "edit.find".to_string(),
                    label: "Find".to_string(),
                },
            ],
            profile: Some("ISPF".to_string()),
        };

        let topic = DynamicContentGenerator::generate_function_keys(&key_map);
        assert_eq!(topic.key(), &TopicKey::feature("function_keys"));
        assert!(topic.body().contains("F3"));
        assert!(topic.body().contains("file.close"));
        assert!(topic.body().contains("ISPF"));
    }

    // Validates: Requirement 15.4 — Empty key map shows guidance
    #[test]
    fn generate_function_keys_empty_shows_guidance() {
        let key_map = MockKeyMap {
            bindings: vec![],
            profile: None,
        };

        let topic = DynamicContentGenerator::generate_function_keys(&key_map);
        assert!(topic
            .body()
            .contains("No function keys are currently assigned"));
        assert!(topic.body().contains("configure"));
    }

    // Validates: Requirement 12.1 — Help Index generated from registry
    #[test]
    fn generate_index_includes_categories() {
        let registry = HelpTopicRegistry::new();
        registry.register_file_topic(HelpTopic::new(
            TopicKey::command("FIND"),
            "FIND Command".to_string(),
            "body".to_string(),
            crate::topic::TopicSource::FileBased {
                file_path: std::path::PathBuf::from("test.help.md"),
            },
        ));

        let topic = DynamicContentGenerator::generate_index(&registry, "FileForgeWorkbench v0.1.0");
        assert!(topic.body().contains("## Primary Commands"));
        assert!(topic.body().contains("FIND Command"));
        assert!(topic.body().contains("## Getting Started"));
        assert!(topic.body().contains("FileForgeWorkbench v0.1.0"));
    }

    // Validates: Requirement 12.4 — Index includes version at bottom
    #[test]
    fn generate_index_includes_version_footer() {
        let registry = HelpTopicRegistry::new();
        let topic = DynamicContentGenerator::generate_index(&registry, "v1.2.3");
        assert!(topic.body().contains("v1.2.3"));
    }
}
