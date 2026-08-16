//! Lexer registry mapping language identifiers to Lexer implementations.

use std::collections::HashMap;

use crate::lexer::traits::Lexer;

/// Registry mapping language identifiers to Lexer implementations.
/// Supports dynamic registration for plugin-provided lexers.
/// Addresses: Requirement 1, criterion 1.8
pub struct LexerRegistry {
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn Lexer> + Send + Sync>>,
}

impl LexerRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a lexer factory for a language identifier.
    /// Returns the previous factory if one was registered for this language_id.
    pub fn register(
        &mut self,
        language_id: &str,
        factory: Box<dyn Fn() -> Box<dyn Lexer> + Send + Sync>,
    ) -> Option<Box<dyn Fn() -> Box<dyn Lexer> + Send + Sync>> {
        self.factories.insert(language_id.to_string(), factory)
    }

    /// Unregister a lexer for a language identifier.
    /// Returns true if a factory was removed.
    pub fn unregister(&mut self, language_id: &str) -> bool {
        self.factories.remove(language_id).is_some()
    }

    /// Create a new lexer instance for the given language.
    /// Returns None if no lexer is registered for this language_id.
    pub fn create_lexer(&self, language_id: &str) -> Option<Box<dyn Lexer>> {
        self.factories.get(language_id).map(|factory| factory())
    }

    /// Check if a lexer is registered for the given language.
    pub fn has_lexer(&self, language_id: &str) -> bool {
        self.factories.contains_key(language_id)
    }

    /// List all registered language identifiers.
    pub fn registered_languages(&self) -> Vec<&str> {
        self.factories.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for LexerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fold::context::FoldContext;
    use crate::style::context::StyleContext;
    use crate::types::{KeywordSetDescriptor, PropertyDescriptor, StyleSlotIndex};
    use std::collections::HashMap as StdHashMap;

    struct DummyLexer {
        lang: &'static str,
        properties: StdHashMap<String, String>,
    }

    impl DummyLexer {
        fn new(lang: &'static str) -> Self {
            Self {
                lang,
                properties: StdHashMap::new(),
            }
        }
    }

    impl Lexer for DummyLexer {
        fn name(&self) -> &str {
            self.lang
        }
        fn style_text(&self, _context: &mut StyleContext) {}
        fn fold_text(&self, _context: &mut FoldContext) {}
        fn default_style(&self) -> StyleSlotIndex {
            StyleSlotIndex::DEFAULT
        }
        fn keyword_sets(&self) -> &[KeywordSetDescriptor] {
            &[]
        }
        fn sub_style_bases(&self) -> &[StyleSlotIndex] {
            &[]
        }
        fn get_property(&self, key: &str) -> Option<&str> {
            self.properties.get(key).map(|v| v.as_str())
        }
        fn set_property(&mut self, key: &str, value: &str) {
            self.properties.insert(key.to_string(), value.to_string());
        }
        fn property_names(&self) -> &[PropertyDescriptor] {
            &[]
        }
        fn style_slot_count(&self) -> u8 {
            1
        }
    }

    #[test]
    fn register_and_create_lexer() {
        // Validates: Requirement 1, criterion 1.8
        let mut registry = LexerRegistry::new();
        registry.register("rust", Box::new(|| Box::new(DummyLexer::new("rust"))));

        let lexer = registry.create_lexer("rust");
        assert!(lexer.is_some());
        assert_eq!(lexer.unwrap().name(), "rust");
    }

    #[test]
    fn create_lexer_unknown_language_returns_none() {
        // Validates: Requirement 1, criterion 1.8
        let registry = LexerRegistry::new();
        assert!(registry.create_lexer("unknown").is_none());
    }

    #[test]
    fn duplicate_registration_returns_previous() {
        // Validates: Requirement 1, criterion 1.8
        let mut registry = LexerRegistry::new();
        let prev = registry.register("rust", Box::new(|| Box::new(DummyLexer::new("rust"))));
        assert!(prev.is_none());

        let prev = registry.register("rust", Box::new(|| Box::new(DummyLexer::new("rust_v2"))));
        assert!(prev.is_some());

        // New factory is now active
        let lexer = registry.create_lexer("rust").unwrap();
        assert_eq!(lexer.name(), "rust_v2");
    }

    #[test]
    fn unregister_removes_factory() {
        let mut registry = LexerRegistry::new();
        registry.register("rust", Box::new(|| Box::new(DummyLexer::new("rust"))));
        assert!(registry.has_lexer("rust"));
        assert!(registry.unregister("rust"));
        assert!(!registry.has_lexer("rust"));
        assert!(registry.create_lexer("rust").is_none());
    }

    #[test]
    fn unregister_nonexistent_returns_false() {
        let mut registry = LexerRegistry::new();
        assert!(!registry.unregister("nonexistent"));
    }

    #[test]
    fn registered_languages_lists_all() {
        let mut registry = LexerRegistry::new();
        registry.register("rust", Box::new(|| Box::new(DummyLexer::new("rust"))));
        registry.register("cpp", Box::new(|| Box::new(DummyLexer::new("cpp"))));
        registry.register("python", Box::new(|| Box::new(DummyLexer::new("python"))));

        let mut langs = registry.registered_languages();
        langs.sort();
        assert_eq!(langs, vec!["cpp", "python", "rust"]);
    }

    #[test]
    fn has_lexer_checks_existence() {
        let mut registry = LexerRegistry::new();
        assert!(!registry.has_lexer("rust"));
        registry.register("rust", Box::new(|| Box::new(DummyLexer::new("rust"))));
        assert!(registry.has_lexer("rust"));
    }
}
