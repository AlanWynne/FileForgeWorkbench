//! Language registry: central store of all active LanguageDefinition instances.

use std::collections::HashMap;

use crate::definition::{DefinitionSource, LanguageDefinition, LanguageId, LanguageSummary};
use crate::error::LanguageServiceError;

/// Source of a language registration for tracking and deregistration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationSource {
    /// Built-in definition loaded from disk.
    BuiltIn,
    /// User configuration.
    UserConfig,
    /// Project configuration.
    ProjectConfig,
    /// Plugin-registered.
    Plugin(String),
}

/// The central language registry owning all loaded LanguageDefinition instances.
#[derive(Debug)]
pub struct LanguageRegistry {
    /// All registered definitions, keyed by language_id.
    definitions: HashMap<String, LanguageDefinition>,
    /// Ordered list of language IDs for consistent enumeration.
    order: Vec<String>,
}

impl LanguageRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Create a registry pre-populated with definitions.
    pub fn from_definitions(definitions: Vec<LanguageDefinition>) -> Self {
        let mut registry = Self::new();
        for def in definitions {
            let id = def.language_id().as_str().to_string();
            if !registry.definitions.contains_key(&id) {
                registry.order.push(id.clone());
                registry.definitions.insert(id, def);
            }
        }
        registry
    }

    /// Register a new language definition.
    ///
    /// # Errors
    ///
    /// Returns `DuplicateLanguage` if the language_id already exists.
    pub fn register(&mut self, definition: LanguageDefinition) -> Result<(), LanguageServiceError> {
        let id = definition.language_id().as_str().to_string();
        if self.definitions.contains_key(&id) {
            let existing = &self.definitions[&id];
            let owner = existing.source().to_string();
            return Err(LanguageServiceError::DuplicateLanguage {
                language_id: id,
                owner,
            });
        }
        self.order.push(id.clone());
        self.definitions.insert(id, definition);
        Ok(())
    }

    /// Deregister a language by its identifier.
    ///
    /// Only removes if the definition was registered from a plugin source.
    /// Returns the removed definition if successful.
    pub fn deregister(&mut self, language_id: &LanguageId) -> Option<LanguageDefinition> {
        let id = language_id.as_str();
        if let Some(def) = self.definitions.get(id) {
            if matches!(def.source(), DefinitionSource::Plugin { .. }) {
                let removed = self.definitions.remove(id);
                self.order.retain(|o| o != id);
                return removed;
            }
        }
        None
    }

    /// Deregister all languages owned by a specific plugin.
    pub fn deregister_plugin(&mut self, plugin_name: &str) -> Vec<LanguageId> {
        let ids_to_remove: Vec<String> = self
            .definitions
            .iter()
            .filter(|(_, def)| {
                matches!(def.source(), DefinitionSource::Plugin { plugin_name: name } if name == plugin_name)
            })
            .map(|(id, _)| id.clone())
            .collect();

        let mut removed = Vec::new();
        for id in &ids_to_remove {
            if let Some(def) = self.definitions.remove(id) {
                removed.push(def.language_id().clone());
            }
        }
        self.order.retain(|o| !ids_to_remove.contains(o));
        removed
    }

    /// Get a reference to a language definition by its identifier.
    pub fn get(&self, language_id: &LanguageId) -> Option<&LanguageDefinition> {
        self.definitions.get(language_id.as_str())
    }

    /// Check if a language is registered.
    pub fn contains(&self, language_id: &LanguageId) -> bool {
        self.definitions.contains_key(language_id.as_str())
    }

    /// List all registered languages as summaries.
    pub fn list_languages(&self) -> Vec<LanguageSummary> {
        self.order
            .iter()
            .filter_map(|id| self.definitions.get(id))
            .map(|def| LanguageSummary {
                language_id: def.language_id().clone(),
                display_name: def.name().to_string(),
                extensions: def.extensions().to_vec(),
            })
            .collect()
    }

    /// Returns the number of registered languages.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns true if no languages are registered.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Returns all definitions as a slice-like iterator.
    pub fn definitions(&self) -> impl Iterator<Item = &LanguageDefinition> {
        self.order
            .iter()
            .filter_map(move |id| self.definitions.get(id))
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{ConfigLayer, DefinitionSource};
    use crate::keyword_set::KeywordSets;

    fn make_definition(id: &str, source: DefinitionSource) -> LanguageDefinition {
        LanguageDefinition {
            language_id: LanguageId::new(id).unwrap(),
            name: id.to_string(),
            extensions: vec!["ext".to_string()],
            priority: 0,
            case_sensitive_keywords: true,
            keyword_sets: KeywordSets::empty(),
            line_comments: Vec::new(),
            block_comment_start: None,
            block_comment_end: None,
            string_delimiters: Vec::new(),
            character_delimiter: None,
            escape_character: None,
            heredoc_patterns: Vec::new(),
            shebang_patterns: Vec::new(),
            magic_bytes: None,
            first_line_pattern: None,
            embedded_languages: Vec::new(),
            properties: std::collections::HashMap::new(),
            fold_keywords: None,
            source,
        }
    }

    #[test]
    fn register_adds_definition_successfully() {
        // Validates: Requirement 9.1
        let mut registry = LanguageRegistry::new();
        let def = make_definition(
            "rust",
            DefinitionSource::File {
                path: "rust.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        );
        assert!(registry.register(def).is_ok());
        assert!(registry.contains(&LanguageId::new("rust").unwrap()));
    }

    #[test]
    fn register_rejects_duplicate_language_id() {
        // Validates: Requirement 9.3
        let mut registry = LanguageRegistry::new();
        let def1 = make_definition(
            "rust",
            DefinitionSource::File {
                path: "rust.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        );
        let def2 = make_definition(
            "rust",
            DefinitionSource::Plugin {
                plugin_name: "test-plugin".to_string(),
            },
        );
        assert!(registry.register(def1).is_ok());
        let result = registry.register(def2);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LanguageServiceError::DuplicateLanguage { .. }
        ));
    }

    #[test]
    fn deregister_removes_plugin_definitions() {
        // Validates: Requirement 9.4
        let mut registry = LanguageRegistry::new();
        let def = make_definition(
            "custom",
            DefinitionSource::Plugin {
                plugin_name: "my-plugin".to_string(),
            },
        );
        registry.register(def).unwrap();
        assert!(registry.contains(&LanguageId::new("custom").unwrap()));

        let removed = registry.deregister(&LanguageId::new("custom").unwrap());
        assert!(removed.is_some());
        assert!(!registry.contains(&LanguageId::new("custom").unwrap()));
    }

    #[test]
    fn deregister_does_not_remove_file_definitions() {
        // Validates: Requirement 9.4
        let mut registry = LanguageRegistry::new();
        let def = make_definition(
            "rust",
            DefinitionSource::File {
                path: "rust.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        );
        registry.register(def).unwrap();

        let removed = registry.deregister(&LanguageId::new("rust").unwrap());
        assert!(removed.is_none());
        assert!(registry.contains(&LanguageId::new("rust").unwrap()));
    }

    #[test]
    fn deregister_plugin_removes_all_plugin_definitions() {
        // Validates: Requirement 9.4
        let mut registry = LanguageRegistry::new();
        let def1 = make_definition(
            "lang-a",
            DefinitionSource::Plugin {
                plugin_name: "my-plugin".to_string(),
            },
        );
        let def2 = make_definition(
            "lang-b",
            DefinitionSource::Plugin {
                plugin_name: "my-plugin".to_string(),
            },
        );
        let def3 = make_definition(
            "lang-c",
            DefinitionSource::Plugin {
                plugin_name: "other-plugin".to_string(),
            },
        );
        registry.register(def1).unwrap();
        registry.register(def2).unwrap();
        registry.register(def3).unwrap();

        let removed = registry.deregister_plugin("my-plugin");
        assert_eq!(removed.len(), 2);
        assert!(!registry.contains(&LanguageId::new("lang-a").unwrap()));
        assert!(!registry.contains(&LanguageId::new("lang-b").unwrap()));
        assert!(registry.contains(&LanguageId::new("lang-c").unwrap()));
    }

    #[test]
    fn list_languages_returns_all_registered() {
        // Validates: Requirement 10.1
        let mut registry = LanguageRegistry::new();
        let def1 = make_definition(
            "rust",
            DefinitionSource::File {
                path: "rust.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        );
        let def2 = make_definition(
            "python",
            DefinitionSource::File {
                path: "python.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        );
        registry.register(def1).unwrap();
        registry.register(def2).unwrap();

        let list = registry.list_languages();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].language_id.as_str(), "rust");
        assert_eq!(list[1].language_id.as_str(), "python");
    }

    #[test]
    fn get_returns_definition_reference() {
        // Validates: Requirement 10.2
        let mut registry = LanguageRegistry::new();
        let def = make_definition(
            "rust",
            DefinitionSource::File {
                path: "rust.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        );
        registry.register(def).unwrap();

        let lang_id = LanguageId::new("rust").unwrap();
        let retrieved = registry.get(&lang_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "rust");
    }

    #[test]
    fn from_definitions_constructor() {
        // Validates: Requirement 10.7
        let defs = vec![
            make_definition(
                "rust",
                DefinitionSource::File {
                    path: "rust.toml".to_string(),
                    layer: ConfigLayer::BuiltIn,
                },
            ),
            make_definition(
                "python",
                DefinitionSource::File {
                    path: "python.toml".to_string(),
                    layer: ConfigLayer::BuiltIn,
                },
            ),
        ];
        let registry = LanguageRegistry::from_definitions(defs);
        assert_eq!(registry.len(), 2);
        assert!(registry.contains(&LanguageId::new("rust").unwrap()));
        assert!(registry.contains(&LanguageId::new("python").unwrap()));
    }
}
