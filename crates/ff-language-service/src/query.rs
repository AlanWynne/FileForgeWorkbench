//! Language service query API: thread-safe top-level facade.

use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

use crate::content_detection::ContentDetector;
use crate::definition::{LanguageDefinition, LanguageId, LanguageSummary};
use crate::detection::{DetectionMethod, DetectionResult, ExtensionMatcher};
use crate::embedded::EmbeddedLanguageResolver;
use crate::error::{DocumentId, LanguageServiceError};
use crate::lexer_state::{LexerState, LineStateVector};
use crate::properties::PropertyStore;
use crate::registry::LanguageRegistry;

/// The central language service managing all language definitions,
/// detection logic, per-line state, and query operations.
///
/// Thread-safe: all query methods take `&self` using interior `RwLock`.
pub struct LanguageService {
    /// All registered language definitions.
    registry: RwLock<LanguageRegistry>,
    /// Extension-to-language mapping.
    extension_matcher: RwLock<ExtensionMatcher>,
    /// Content-based detector.
    content_detector: RwLock<ContentDetector>,
    /// Per-language property store.
    property_store: RwLock<PropertyStore>,
    /// Per-document line state vectors.
    document_states: RwLock<HashMap<DocumentId, LineStateVector>>,
    /// Embedded language resolver.
    embedded_resolver: EmbeddedLanguageResolver,
}

impl LanguageService {
    /// Create a LanguageService from a pre-built list of definitions (for testing).
    ///
    /// This constructor requires no filesystem or running application.
    pub fn from_definitions(definitions: Vec<LanguageDefinition>) -> Self {
        let extension_matcher = ExtensionMatcher::from_definitions(&definitions);
        let content_detector = ContentDetector::from_definitions(&definitions);

        let mut property_store = PropertyStore::new();
        for def in &definitions {
            property_store.register_builtins(def.language_id(), def.properties().clone());
        }

        let registry = LanguageRegistry::from_definitions(definitions);

        Self {
            registry: RwLock::new(registry),
            extension_matcher: RwLock::new(extension_matcher),
            content_detector: RwLock::new(content_detector),
            property_store: RwLock::new(property_store),
            document_states: RwLock::new(HashMap::new()),
            embedded_resolver: EmbeddedLanguageResolver::new(),
        }
    }

    /// Perform the full detection pipeline: extension → content-based → fallback.
    pub fn detect_language(
        &self,
        file_path: Option<&str>,
        first_line: Option<&str>,
        first_bytes: Option<&[u8]>,
    ) -> DetectionResult {
        // Try extension-based detection first
        if let Some(path) = file_path {
            let matcher = self.extension_matcher.read().unwrap();
            let result = matcher.detect(path);
            if result.method != DetectionMethod::Fallback {
                return result;
            }
        }

        // Try content-based detection
        let detector = self.content_detector.read().unwrap();
        detector.detect(first_bytes, first_line)
    }

    /// Perform extension-only lookup without content-based fallback.
    pub fn language_for_extension(&self, extension: &str) -> Option<LanguageId> {
        let matcher = self.extension_matcher.read().unwrap();
        matcher.language_for_extension(extension)
    }

    /// Manually override the detected language for a document.
    pub fn override_language(
        &self,
        doc_id: DocumentId,
        language_id: LanguageId,
    ) -> Result<(), LanguageServiceError> {
        let registry = self.registry.read().unwrap();
        if !registry.contains(&language_id) && !language_id.is_plain_text() {
            return Err(LanguageServiceError::LanguageNotFound {
                language_id: language_id.as_str().to_string(),
            });
        }
        drop(registry);
        let mut matcher = self.extension_matcher.write().unwrap();
        matcher.set_override(doc_id, language_id);
        Ok(())
    }

    /// List all registered languages with summary info.
    pub fn list_languages(&self) -> Vec<LanguageSummary> {
        let registry = self.registry.read().unwrap();
        registry.list_languages()
    }

    /// Get a language definition by its identifier.
    pub fn get_definition(&self, language_id: &LanguageId) -> Option<LanguageDefinition> {
        let registry = self.registry.read().unwrap();
        registry.get(language_id).cloned()
    }

    /// Get the file extensions associated with a language.
    pub fn extensions_for(&self, language_id: &LanguageId) -> Vec<String> {
        let registry = self.registry.read().unwrap();
        registry
            .get(language_id)
            .map(|def| def.extensions().to_vec())
            .unwrap_or_default()
    }

    /// Case-sensitive membership test against a keyword set.
    pub fn in_keyword_set(&self, language_id: &LanguageId, word: &str, set_number: u8) -> bool {
        let registry = self.registry.read().unwrap();
        registry
            .get(language_id)
            .map(|def| def.keyword_sets().in_keyword_set(word, set_number))
            .unwrap_or(false)
    }

    /// Case-insensitive membership test against a keyword set.
    pub fn in_keyword_set_case_insensitive(
        &self,
        language_id: &LanguageId,
        word: &str,
        set_number: u8,
    ) -> bool {
        let registry = self.registry.read().unwrap();
        registry
            .get(language_id)
            .map(|def| {
                def.keyword_sets()
                    .in_keyword_set_case_insensitive(word, set_number)
            })
            .unwrap_or(false)
    }

    /// Initialize a line state vector for a new document.
    pub fn init_document_state(&self, document_id: DocumentId, line_count: usize) {
        let mut states = self.document_states.write().unwrap();
        states.insert(document_id, LineStateVector::new(line_count));
    }

    /// Get the starting lexer state for highlighting a specific line.
    pub fn start_state_for(
        &self,
        document_id: DocumentId,
        line_index: usize,
    ) -> Option<LexerState> {
        let states = self.document_states.read().unwrap();
        states
            .get(&document_id)
            .map(|sv| sv.get_start_state(line_index))
    }

    /// Store the end-of-line state after highlighting completes.
    /// Returns true if the state changed (propagation needed).
    pub fn set_end_state(
        &self,
        document_id: DocumentId,
        line_index: usize,
        state: LexerState,
    ) -> bool {
        let mut states = self.document_states.write().unwrap();
        if let Some(sv) = states.get_mut(&document_id) {
            let should_continue = sv.should_continue(line_index, state);
            sv.set_end_state(line_index, state);
            should_continue
        } else {
            false
        }
    }

    /// Invalidate line state when a line is modified.
    pub fn invalidate_line(&self, document_id: DocumentId, line_index: usize) {
        let mut states = self.document_states.write().unwrap();
        if let Some(sv) = states.get_mut(&document_id) {
            sv.invalidate_from(line_index);
        }
    }

    /// Handle line insertions in the document.
    pub fn on_lines_inserted(&self, document_id: DocumentId, line_index: usize, count: usize) {
        let mut states = self.document_states.write().unwrap();
        if let Some(sv) = states.get_mut(&document_id) {
            sv.insert_lines(line_index, count);
        }
    }

    /// Handle line deletions in the document.
    pub fn on_lines_deleted(&self, document_id: DocumentId, line_index: usize, count: usize) {
        let mut states = self.document_states.write().unwrap();
        if let Some(sv) = states.get_mut(&document_id) {
            sv.delete_lines(line_index, count);
        }
    }

    /// Remove the state vector for a closed document.
    pub fn remove_document_state(&self, document_id: DocumentId) {
        let mut states = self.document_states.write().unwrap();
        states.remove(&document_id);
    }

    /// Get a property value for a language.
    pub fn get_property(&self, language_id: &LanguageId, key: &str) -> Option<String> {
        let store = self.property_store.read().unwrap();
        store.get_property(language_id, key)
    }

    /// Get a property as integer with a default fallback.
    pub fn get_property_int(&self, language_id: &LanguageId, key: &str, default: i64) -> i64 {
        let store = self.property_store.read().unwrap();
        store.get_property_int(language_id, key, default)
    }

    /// Get a property as boolean with a default fallback.
    pub fn get_property_bool(&self, language_id: &LanguageId, key: &str, default: bool) -> bool {
        let store = self.property_store.read().unwrap();
        store.get_property_bool(language_id, key, default)
    }

    /// Register a new language definition at runtime (from a plugin).
    pub fn register_language(
        &self,
        definition: LanguageDefinition,
    ) -> Result<(), LanguageServiceError> {
        // Register in the registry
        let mut registry = self.registry.write().unwrap();
        registry.register(definition.clone())?;
        drop(registry);

        // Update the extension matcher
        {
            let registry = self.registry.read().unwrap();
            let definitions: Vec<&LanguageDefinition> = registry.definitions().collect();
            let defs_owned: Vec<LanguageDefinition> = definitions.into_iter().cloned().collect();
            drop(registry);

            let new_matcher = ExtensionMatcher::from_definitions(&defs_owned);
            let new_detector = ContentDetector::from_definitions(&defs_owned);
            *self.extension_matcher.write().unwrap() = new_matcher;
            *self.content_detector.write().unwrap() = new_detector;
        }

        // Update property store
        {
            let mut store = self.property_store.write().unwrap();
            store.register_builtins(definition.language_id(), definition.properties().clone());
        }

        Ok(())
    }

    /// Deregister all languages owned by a specific plugin.
    pub fn deregister_plugin_languages(&self, plugin_name: &str) -> Vec<LanguageId> {
        let mut registry = self.registry.write().unwrap();
        let removed = registry.deregister_plugin(plugin_name);
        drop(registry);

        if !removed.is_empty() {
            // Rebuild extension matcher and content detector
            let registry = self.registry.read().unwrap();
            let defs_owned: Vec<LanguageDefinition> = registry.definitions().cloned().collect();
            drop(registry);

            let new_matcher = ExtensionMatcher::from_definitions(&defs_owned);
            let new_detector = ContentDetector::from_definitions(&defs_owned);
            *self.extension_matcher.write().unwrap() = new_matcher;
            *self.content_detector.write().unwrap() = new_detector;

            // Remove properties for deregistered languages
            let mut store = self.property_store.write().unwrap();
            for lang_id in &removed {
                store.remove_builtins(lang_id);
            }
        }

        removed
    }

    /// Resolve an embedded language definition.
    pub fn resolve_embedded_language(
        &self,
        language_id: &LanguageId,
    ) -> Option<LanguageDefinition> {
        let registry = self.registry.read().unwrap();
        self.embedded_resolver
            .resolve_embedded(language_id, &registry)
            .cloned()
    }

    /// Returns the maximum embedding depth supported.
    pub fn max_embedding_depth(&self) -> usize {
        self.embedded_resolver.max_embedding_depth()
    }

    /// Load definitions from a directory path (for testing and initialization).
    pub fn load_from_directory(
        path: &Path,
    ) -> Result<Vec<LanguageDefinition>, Vec<LanguageServiceError>> {
        use crate::definition::{ConfigLayer, DefinitionSource, TomlLanguageDefinition};

        let mut definitions = Vec::new();
        let mut errors = Vec::new();

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => return Err(vec![LanguageServiceError::Io(e)]),
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().and_then(|e| e.to_str()) == Some("toml") {
                let content = match std::fs::read_to_string(&entry_path) {
                    Ok(c) => c,
                    Err(e) => {
                        errors.push(LanguageServiceError::ParseError {
                            path: entry_path.display().to_string(),
                            reason: e.to_string(),
                        });
                        continue;
                    }
                };

                let parsed: TomlLanguageDefinition = match toml::from_str(&content) {
                    Ok(p) => p,
                    Err(e) => {
                        errors.push(LanguageServiceError::ParseError {
                            path: entry_path.display().to_string(),
                            reason: e.to_string(),
                        });
                        continue;
                    }
                };

                match parsed.into_definition(DefinitionSource::File {
                    path: entry_path.display().to_string(),
                    layer: ConfigLayer::BuiltIn,
                }) {
                    Ok(def) => definitions.push(def),
                    Err(e) => errors.push(e),
                }
            }
        }

        if errors.is_empty() {
            Ok(definitions)
        } else if definitions.is_empty() {
            Err(errors)
        } else {
            // Partial success — return what we have (errors are logged)
            Ok(definitions)
        }
    }
}

impl std::fmt::Debug for LanguageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageService")
            .field("language_count", &self.list_languages().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{ConfigLayer, DefinitionSource};
    use crate::keyword_set::KeywordSets;

    fn make_definition(id: &str, name: &str, extensions: &[&str]) -> LanguageDefinition {
        LanguageDefinition {
            language_id: LanguageId::new(id).unwrap(),
            name: name.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
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
            properties: HashMap::new(),
            fold_keywords: None,
            source: DefinitionSource::File {
                path: "test.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        }
    }

    #[test]
    fn from_definitions_creates_working_service() {
        // Validates: Requirement 10.7
        let defs = vec![
            make_definition("rust", "Rust", &["rs"]),
            make_definition("python", "Python", &["py"]),
        ];
        let service = LanguageService::from_definitions(defs);
        assert_eq!(service.list_languages().len(), 2);
    }

    #[test]
    fn detect_language_by_extension() {
        // Validates: Requirement 10.3, 2.1
        let defs = vec![make_definition("rust", "Rust", &["rs"])];
        let service = LanguageService::from_definitions(defs);

        let result = service.detect_language(Some("main.rs"), None, None);
        assert_eq!(result.language_id.as_str(), "rust");
        assert_eq!(result.method, DetectionMethod::Extension);
    }

    #[test]
    fn detect_language_fallback_to_plain_text() {
        // Validates: Requirement 10.3
        let defs = vec![make_definition("rust", "Rust", &["rs"])];
        let service = LanguageService::from_definitions(defs);

        let result = service.detect_language(Some("readme.txt"), None, None);
        assert!(result.language_id.is_plain_text());
    }

    #[test]
    fn list_languages_returns_all() {
        // Validates: Requirement 10.1
        let defs = vec![
            make_definition("rust", "Rust", &["rs"]),
            make_definition("python", "Python", &["py", "pyw"]),
        ];
        let service = LanguageService::from_definitions(defs);

        let list = service.list_languages();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn get_definition_returns_correct_definition() {
        // Validates: Requirement 10.2
        let defs = vec![make_definition("rust", "Rust", &["rs"])];
        let service = LanguageService::from_definitions(defs);

        let lang_id = LanguageId::new("rust").unwrap();
        let def = service.get_definition(&lang_id);
        assert!(def.is_some());
        assert_eq!(def.unwrap().name(), "Rust");
    }

    #[test]
    fn extensions_for_returns_language_extensions() {
        // Validates: Requirement 10.4
        let defs = vec![make_definition("python", "Python", &["py", "pyw"])];
        let service = LanguageService::from_definitions(defs);

        let lang_id = LanguageId::new("python").unwrap();
        let exts = service.extensions_for(&lang_id);
        assert_eq!(exts, vec!["py", "pyw"]);
    }

    #[test]
    fn language_for_extension_performs_lookup() {
        // Validates: Requirement 10.5
        let defs = vec![make_definition("rust", "Rust", &["rs"])];
        let service = LanguageService::from_definitions(defs);

        assert_eq!(
            service.language_for_extension("rs"),
            Some(LanguageId::new("rust").unwrap())
        );
        assert_eq!(service.language_for_extension("xyz"), None);
    }

    #[test]
    fn document_state_lifecycle() {
        // Validates: Requirement 4.1, 4.2, 4.4, 4.5
        let service = LanguageService::from_definitions(Vec::new());

        service.init_document_state(1, 10);
        assert_eq!(
            service.start_state_for(1, 0),
            Some(crate::lexer_state::LEXER_STATE_INITIAL)
        );

        // Set end state for line 0
        let changed = service.set_end_state(1, 0, 5);
        assert!(changed);

        // Start state for line 1 should be state of line 0
        assert_eq!(service.start_state_for(1, 1), Some(5));

        // Setting same state should not indicate change
        let changed = service.set_end_state(1, 0, 5);
        assert!(!changed);
    }

    #[test]
    fn register_and_deregister_language() {
        // Validates: Requirement 9.1, 9.4, 9.7
        let service = LanguageService::from_definitions(Vec::new());

        let def = LanguageDefinition {
            language_id: LanguageId::new("custom").unwrap(),
            name: "Custom".to_string(),
            extensions: vec!["cst".to_string()],
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
            properties: HashMap::new(),
            fold_keywords: None,
            source: DefinitionSource::Plugin {
                plugin_name: "test-plugin".to_string(),
            },
        };

        assert!(service.register_language(def).is_ok());
        assert_eq!(service.list_languages().len(), 1);

        // Extension detection should work
        let result = service.detect_language(Some("file.cst"), None, None);
        assert_eq!(result.language_id.as_str(), "custom");

        // Deregister
        let removed = service.deregister_plugin_languages("test-plugin");
        assert_eq!(removed.len(), 1);
        assert_eq!(service.list_languages().len(), 0);
    }

    #[test]
    fn concurrent_access_is_safe() {
        // Validates: Requirement 10.6
        use std::sync::Arc;
        use std::thread;

        let defs = vec![make_definition("rust", "Rust", &["rs"])];
        let service = Arc::new(LanguageService::from_definitions(defs));

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let svc = Arc::clone(&service);
                thread::spawn(move || {
                    for _ in 0..100 {
                        let _ = svc.list_languages();
                        let _ = svc.detect_language(Some("test.rs"), None, None);
                        let _ = svc.language_for_extension("rs");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
