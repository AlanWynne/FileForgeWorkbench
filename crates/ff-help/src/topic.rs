//! `HelpTopic` — a single unit of help content.
//!
//! Each topic represents one command, line command, feature, mode, or
//! configuration key's help documentation.

use std::path::PathBuf;

use crate::topic_key::TopicKey;

/// Identifies where a help topic was registered from.
///
/// Used for priority resolution: runtime-registered topics (from commands or
/// plugins) take precedence over file-based content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopicSource {
    /// Loaded from a `.help.md` file on disk.
    FileBased {
        /// Path to the source file.
        file_path: PathBuf,
    },
    /// Auto-generated from `CommandMetadata.help_text` at command registration.
    CommandRegistry {
        /// The command ID that contributed this topic.
        command_id: String,
    },
    /// Contributed by a plugin during its `initialize` lifecycle phase.
    Plugin {
        /// The plugin ID that contributed this topic.
        plugin_id: String,
    },
}

impl TopicSource {
    /// Returns a priority value for comparison. Higher values win.
    ///
    /// Runtime sources (Command, Plugin) have higher priority than file-based.
    pub fn priority(&self) -> u8 {
        match self {
            Self::FileBased { .. } => 0,
            Self::CommandRegistry { .. } => 1,
            Self::Plugin { .. } => 1,
        }
    }
}

/// A single unit of help content — one topic per command, line command,
/// feature, mode, or configuration key.
///
/// # Fields
///
/// - `key`: The unique `TopicKey` for registry lookup.
/// - `title`: Human-readable title displayed at the top of the Help Panel.
/// - `body`: The Markdown body content of the help topic.
/// - `syntax`: Optional command syntax line (for command topics).
/// - `aliases`: Alternative names that also resolve to this topic.
/// - `see_also`: Cross-reference links to related topics.
/// - `source`: Where this topic was registered from.
#[derive(Debug, Clone)]
pub struct HelpTopic {
    key: TopicKey,
    title: String,
    body: String,
    syntax: Option<String>,
    aliases: Vec<String>,
    see_also: Vec<TopicKey>,
    source: TopicSource,
}

impl HelpTopic {
    /// Creates a new help topic with the given key, title, body, and source.
    pub fn new(key: TopicKey, title: String, body: String, source: TopicSource) -> Self {
        Self {
            key,
            title,
            body,
            syntax: None,
            aliases: Vec::new(),
            see_also: Vec::new(),
            source,
        }
    }

    /// Returns the unique topic key.
    pub fn key(&self) -> &TopicKey {
        &self.key
    }

    /// Returns the human-readable title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the Markdown body content.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the optional syntax line.
    pub fn syntax(&self) -> Option<&str> {
        self.syntax.as_deref()
    }

    /// Returns the list of aliases for this topic.
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// Returns the cross-reference links to related topics.
    pub fn see_also(&self) -> &[TopicKey] {
        &self.see_also
    }

    /// Returns the source of this topic.
    pub fn source(&self) -> &TopicSource {
        &self.source
    }

    /// Sets the syntax line for this topic. Returns self for chaining.
    pub fn with_syntax(mut self, syntax: String) -> Self {
        self.syntax = Some(syntax);
        self
    }

    /// Sets the aliases for this topic. Returns self for chaining.
    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    /// Sets the see-also cross-references. Returns self for chaining.
    pub fn with_see_also(mut self, see_also: Vec<TopicKey>) -> Self {
        self.see_also = see_also;
        self
    }

    /// Extracts cross-reference link TopicKeys from the body content.
    ///
    /// Scans for Markdown link patterns `[text](topic_key)` where
    /// `topic_key` matches a valid TopicKey format.
    pub fn cross_references(&self) -> Vec<TopicKey> {
        let mut refs = Vec::new();
        let body = &self.body;

        // Simple regex-free extraction of [text](topic_key) links
        let mut search_from = 0;
        while let Some(open_paren) = body[search_from..].find("](") {
            let abs_open = search_from + open_paren + 2;
            if let Some(close_paren) = body[abs_open..].find(')') {
                let key_str = &body[abs_open..abs_open + close_paren];
                if let Ok(key) = key_str.parse::<TopicKey>() {
                    refs.push(key);
                }
                search_from = abs_open + close_paren + 1;
            } else {
                break;
            }
        }

        refs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Validates: Requirement 6.1 — HelpTopic construction
    #[test]
    fn help_topic_new_sets_fields_correctly() {
        let key = TopicKey::command("FIND");
        let topic = HelpTopic::new(
            key.clone(),
            "FIND Command".to_string(),
            "Searches for text.".to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("help/commands.help.md"),
            },
        );

        assert_eq!(topic.key(), &key);
        assert_eq!(topic.title(), "FIND Command");
        assert_eq!(topic.body(), "Searches for text.");
        assert_eq!(topic.syntax(), None);
        assert!(topic.aliases().is_empty());
        assert!(topic.see_also().is_empty());
    }

    // Validates: Requirement 7.1 — Topic with syntax and aliases
    #[test]
    fn help_topic_builder_chain_sets_optional_fields() {
        let key = TopicKey::command("FIND");
        let topic = HelpTopic::new(
            key,
            "FIND Command".to_string(),
            "Body".to_string(),
            TopicSource::CommandRegistry {
                command_id: "find".to_string(),
            },
        )
        .with_syntax("FIND 'text' [ALL] [FIRST|LAST|NEXT|PREV]".to_string())
        .with_aliases(vec!["F".to_string(), "SEARCH".to_string()])
        .with_see_also(vec![TopicKey::command("CHANGE")]);

        assert_eq!(
            topic.syntax(),
            Some("FIND 'text' [ALL] [FIRST|LAST|NEXT|PREV]")
        );
        assert_eq!(topic.aliases(), &["F", "SEARCH"]);
        assert_eq!(topic.see_also(), &[TopicKey::command("CHANGE")]);
    }

    // Validates: Requirement 6.4 — TopicSource priority
    #[test]
    fn topic_source_runtime_has_higher_priority_than_file() {
        let file = TopicSource::FileBased {
            file_path: PathBuf::from("help/test.help.md"),
        };
        let cmd = TopicSource::CommandRegistry {
            command_id: "find".to_string(),
        };
        let plugin = TopicSource::Plugin {
            plugin_id: "my_plugin".to_string(),
        };

        assert!(cmd.priority() > file.priority());
        assert!(plugin.priority() > file.priority());
    }

    // Validates: Requirement 5.3 — Cross-reference link extraction
    #[test]
    fn cross_references_extracts_valid_links() {
        let body = "See the [CHANGE command](cmd:CHANGE) for details.\n\
                    Also check [undo feature](feature:undo) and [invalid](not_a_key).";
        let topic = HelpTopic::new(
            TopicKey::command("FIND"),
            "FIND".to_string(),
            body.to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("test.help.md"),
            },
        );

        let refs = topic.cross_references();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], TopicKey::command("CHANGE"));
        assert_eq!(refs[1], TopicKey::feature("undo"));
    }
}
