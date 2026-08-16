//! Embedded language support: resolution and nesting management.

use crate::definition::{EmbeddedLanguageDescriptor, LanguageDefinition, LanguageId};
use crate::registry::LanguageRegistry;

/// Maximum nesting depth for embedded languages.
const MAX_EMBEDDING_DEPTH: usize = 3;

/// Resolves embedded language transitions and manages nesting.
#[derive(Debug)]
pub struct EmbeddedLanguageResolver;

impl EmbeddedLanguageResolver {
    /// Create a new embedded language resolver.
    pub fn new() -> Self {
        Self
    }

    /// Resolve an embedded language definition from the registry.
    ///
    /// Returns `None` if the embedded language is not registered.
    pub fn resolve_embedded<'a>(
        &self,
        language_id: &LanguageId,
        registry: &'a LanguageRegistry,
    ) -> Option<&'a LanguageDefinition> {
        registry.get(language_id)
    }

    /// Returns the maximum supported nesting depth for embedded languages.
    pub fn max_embedding_depth(&self) -> usize {
        MAX_EMBEDDING_DEPTH
    }

    /// Encode a state value that includes both host state and embedded language info.
    ///
    /// The encoding packs the host state and an embedded language index into a single i32:
    /// - Bits 0–15: host language state
    /// - Bits 16–23: embedded language index (0 = host, 1–255 = embedded)
    /// - Bits 24–25: nesting depth (0–3)
    pub fn encode_state(host_state: i16, embedded_index: u8, depth: u8) -> i32 {
        let depth = depth.min(MAX_EMBEDDING_DEPTH as u8);
        (host_state as i32 & 0xFFFF) | ((embedded_index as i32) << 16) | ((depth as i32) << 24)
    }

    /// Decode a state value into host state, embedded language index, and depth.
    pub fn decode_state(state: i32) -> (i16, u8, u8) {
        let host_state = (state & 0xFFFF) as i16;
        let embedded_index = ((state >> 16) & 0xFF) as u8;
        let depth = ((state >> 24) & 0x03) as u8;
        (host_state, embedded_index, depth)
    }

    /// Get embedded language descriptors for a language definition.
    pub fn get_embedded_descriptors<'a>(
        &self,
        definition: &'a LanguageDefinition,
    ) -> &'a [EmbeddedLanguageDescriptor] {
        definition.embedded_languages()
    }
}

impl Default for EmbeddedLanguageResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{ConfigLayer, DefinitionSource};
    use crate::keyword_set::KeywordSets;
    use std::collections::HashMap;

    fn make_definition(id: &str) -> LanguageDefinition {
        LanguageDefinition {
            language_id: LanguageId::new(id).unwrap(),
            name: id.to_string(),
            extensions: Vec::new(),
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
    fn resolve_embedded_finds_registered_language() {
        // Validates: Requirement 7.2
        let resolver = EmbeddedLanguageResolver::new();
        let mut registry = LanguageRegistry::new();
        registry.register(make_definition("javascript")).unwrap();

        let js_id = LanguageId::new("javascript").unwrap();
        let result = resolver.resolve_embedded(&js_id, &registry);
        assert!(result.is_some());
        assert_eq!(result.unwrap().language_id().as_str(), "javascript");
    }

    #[test]
    fn resolve_embedded_returns_none_for_unregistered() {
        // Validates: Requirement 7.6
        let resolver = EmbeddedLanguageResolver::new();
        let registry = LanguageRegistry::new();

        let unknown_id = LanguageId::new("unknown").unwrap();
        let result = resolver.resolve_embedded(&unknown_id, &registry);
        assert!(result.is_none());
    }

    #[test]
    fn max_embedding_depth_is_at_least_three() {
        // Validates: Requirement 7.4
        let resolver = EmbeddedLanguageResolver::new();
        assert!(resolver.max_embedding_depth() >= 3);
    }

    #[test]
    fn encode_decode_state_roundtrip() {
        // Validates: Requirement 7.5
        let encoded = EmbeddedLanguageResolver::encode_state(42, 3, 2);
        let (host, embedded, depth) = EmbeddedLanguageResolver::decode_state(encoded);
        assert_eq!(host, 42);
        assert_eq!(embedded, 3);
        assert_eq!(depth, 2);
    }

    #[test]
    fn encode_state_clamps_depth_to_max() {
        // Validates: Requirement 7.4
        let encoded = EmbeddedLanguageResolver::encode_state(0, 1, 10);
        let (_, _, depth) = EmbeddedLanguageResolver::decode_state(encoded);
        assert!(depth <= 3);
    }

    #[test]
    fn encode_decode_zero_state() {
        // Validates: Requirement 7.5
        let encoded = EmbeddedLanguageResolver::encode_state(0, 0, 0);
        let (host, embedded, depth) = EmbeddedLanguageResolver::decode_state(encoded);
        assert_eq!(host, 0);
        assert_eq!(embedded, 0);
        assert_eq!(depth, 0);
    }
}
