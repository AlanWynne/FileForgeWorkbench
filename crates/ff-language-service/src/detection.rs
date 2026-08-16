//! Language detection by file extension.

use std::collections::HashMap;

use crate::definition::{LanguageDefinition, LanguageId};

/// How a language was detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMethod {
    /// Matched by file extension.
    Extension,
    /// Matched by magic bytes at file start.
    MagicBytes,
    /// Matched by shebang line.
    Shebang,
    /// Matched by first-line pattern.
    FirstLinePattern,
    /// Manual override by user.
    ManualOverride,
    /// No match — plain text fallback.
    Fallback,
}

/// Result of the full language detection pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResult {
    /// The resolved language identifier.
    pub language_id: LanguageId,
    /// How the detection was made.
    pub method: DetectionMethod,
}

/// Entry in the extension matcher for a single extension mapping.
#[derive(Debug, Clone)]
struct ExtensionEntry {
    language_id: LanguageId,
    priority: i32,
    is_compound: bool,
}

/// Maps file extensions to language IDs with support for case-insensitive
/// matching and compound extensions.
#[derive(Debug, Clone)]
pub struct ExtensionMatcher {
    /// Maps lowercase extension → list of matching language entries.
    extension_map: HashMap<String, Vec<ExtensionEntry>>,
    /// Manual language overrides per document.
    overrides: HashMap<u64, LanguageId>,
}

impl ExtensionMatcher {
    /// Build an extension matcher from a set of language definitions.
    pub fn from_definitions(definitions: &[LanguageDefinition]) -> Self {
        let mut extension_map: HashMap<String, Vec<ExtensionEntry>> = HashMap::new();

        for def in definitions {
            for ext in def.extensions() {
                let ext_lower = ext.to_lowercase();
                let is_compound = ext_lower.contains('.');
                extension_map
                    .entry(ext_lower)
                    .or_default()
                    .push(ExtensionEntry {
                        language_id: def.language_id().clone(),
                        priority: def.priority(),
                        is_compound,
                    });
            }
        }

        Self {
            extension_map,
            overrides: HashMap::new(),
        }
    }

    /// Detect language by file extension.
    ///
    /// Performs case-insensitive matching, supports compound extensions,
    /// and resolves conflicts by priority then alphabetical order.
    pub fn detect(&self, file_path: &str) -> DetectionResult {
        let file_lower = file_path.to_lowercase();

        // Try compound extension first (e.g., "test.ts" from "foo.test.ts")
        if let Some(result) = self.try_compound_match(&file_lower) {
            return result;
        }

        // Try simple extension
        if let Some(ext) = Self::extract_extension(&file_lower) {
            if let Some(entries) = self.extension_map.get(ext) {
                if let Some(lang_id) = Self::resolve_best_match(entries) {
                    return DetectionResult {
                        language_id: lang_id,
                        method: DetectionMethod::Extension,
                    };
                }
            }
        }

        DetectionResult {
            language_id: LanguageId::plain_text(),
            method: DetectionMethod::Fallback,
        }
    }

    /// Try matching compound extensions by checking progressively longer suffixes.
    fn try_compound_match(&self, file_lower: &str) -> Option<DetectionResult> {
        // Get the filename component
        let filename = file_lower.rsplit(['/', '\\']).next().unwrap_or(file_lower);

        // Try progressively removing parts from the beginning
        // e.g., "foo.test.ts" → try "test.ts", then "ts"
        let mut best_match: Option<(LanguageId, i32)> = None;

        let parts: Vec<&str> = filename.split('.').collect();
        if parts.len() > 2 {
            // Try compound extensions (skip the base name)
            for start in 1..parts.len() - 1 {
                let compound = parts[start..].join(".");
                if let Some(entries) = self.extension_map.get(&compound) {
                    let compound_entries: Vec<_> =
                        entries.iter().filter(|e| e.is_compound).collect();
                    if !compound_entries.is_empty() {
                        // Pick highest priority compound match
                        let best = compound_entries
                            .iter()
                            .max_by(|a, b| {
                                a.priority
                                    .cmp(&b.priority)
                                    .then_with(|| b.language_id.cmp(&a.language_id))
                            })
                            .unwrap();
                        match &best_match {
                            None => {
                                best_match = Some((best.language_id.clone(), best.priority));
                            }
                            Some((_, current_priority)) => {
                                if best.priority > *current_priority {
                                    best_match = Some((best.language_id.clone(), best.priority));
                                }
                            }
                        }
                    }
                }
            }
        }

        best_match.map(|(language_id, _)| DetectionResult {
            language_id,
            method: DetectionMethod::Extension,
        })
    }

    /// Extract the simple extension from a filename.
    fn extract_extension(file_lower: &str) -> Option<&str> {
        let filename = file_lower.rsplit(['/', '\\']).next().unwrap_or(file_lower);
        filename
            .rsplit('.')
            .next()
            .filter(|ext| !ext.is_empty() && *ext != filename)
    }

    /// Resolve the best match from multiple entries: highest priority, then alphabetical.
    fn resolve_best_match(entries: &[ExtensionEntry]) -> Option<LanguageId> {
        if entries.is_empty() {
            return None;
        }
        let best = entries
            .iter()
            .max_by(|a, b| {
                a.priority
                    .cmp(&b.priority)
                    .then_with(|| b.language_id.cmp(&a.language_id))
            })
            .unwrap();
        Some(best.language_id.clone())
    }

    /// Set a manual language override for a document.
    pub fn set_override(&mut self, doc_id: u64, language_id: LanguageId) {
        self.overrides.insert(doc_id, language_id);
    }

    /// Get the manual override for a document, if set.
    pub fn get_override(&self, doc_id: u64) -> Option<&LanguageId> {
        self.overrides.get(&doc_id)
    }

    /// Remove a manual override.
    pub fn remove_override(&mut self, doc_id: u64) {
        self.overrides.remove(&doc_id);
    }

    /// Perform extension-only lookup without content fallback.
    pub fn language_for_extension(&self, extension: &str) -> Option<LanguageId> {
        let ext_lower = extension.to_lowercase();
        self.extension_map
            .get(&ext_lower)
            .and_then(|entries| Self::resolve_best_match(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{ConfigLayer, DefinitionSource};
    use crate::keyword_set::KeywordSets;

    fn make_definition(
        id: &str,
        name: &str,
        extensions: &[&str],
        priority: i32,
    ) -> LanguageDefinition {
        LanguageDefinition {
            language_id: LanguageId::new(id).unwrap(),
            name: name.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            priority,
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
            source: DefinitionSource::File {
                path: "test.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        }
    }

    #[test]
    fn detect_matches_simple_extension() {
        // Validates: Requirement 2.4
        let defs = vec![make_definition("rust", "Rust", &["rs"], 0)];
        let matcher = ExtensionMatcher::from_definitions(&defs);
        let result = matcher.detect("main.rs");
        assert_eq!(result.language_id.as_str(), "rust");
        assert_eq!(result.method, DetectionMethod::Extension);
    }

    #[test]
    fn detect_is_case_insensitive() {
        // Validates: Requirement 2.2
        let defs = vec![make_definition("rust", "Rust", &["rs"], 0)];
        let matcher = ExtensionMatcher::from_definitions(&defs);

        assert_eq!(matcher.detect("main.RS").language_id.as_str(), "rust");
        assert_eq!(matcher.detect("main.Rs").language_id.as_str(), "rust");
        assert_eq!(matcher.detect("main.rS").language_id.as_str(), "rust");
    }

    #[test]
    fn detect_compound_extension_has_priority() {
        // Validates: Requirement 2.3
        let defs = vec![
            make_definition("typescript", "TypeScript", &["ts"], 0),
            make_definition("typescript-test", "TypeScript Test", &["test.ts"], 0),
        ];
        let matcher = ExtensionMatcher::from_definitions(&defs);
        let result = matcher.detect("foo.test.ts");
        assert_eq!(result.language_id.as_str(), "typescript-test");
    }

    #[test]
    fn detect_no_match_returns_plain_text() {
        // Validates: Requirement 2.5
        let defs = vec![make_definition("rust", "Rust", &["rs"], 0)];
        let matcher = ExtensionMatcher::from_definitions(&defs);
        let result = matcher.detect("readme.xyz");
        assert!(result.language_id.is_plain_text());
        assert_eq!(result.method, DetectionMethod::Fallback);
    }

    #[test]
    fn detect_multi_match_uses_highest_priority() {
        // Validates: Requirement 2.6
        let defs = vec![
            make_definition("lang-a", "Language A", &["x"], 5),
            make_definition("lang-b", "Language B", &["x"], 10),
        ];
        let matcher = ExtensionMatcher::from_definitions(&defs);
        let result = matcher.detect("file.x");
        assert_eq!(result.language_id.as_str(), "lang-b");
    }

    #[test]
    fn detect_multi_match_equal_priority_uses_alphabetical() {
        // Validates: Requirement 2.6
        let defs = vec![
            make_definition("beta", "Beta", &["ext"], 0),
            make_definition("alpha", "Alpha", &["ext"], 0),
        ];
        let matcher = ExtensionMatcher::from_definitions(&defs);
        let result = matcher.detect("file.ext");
        assert_eq!(result.language_id.as_str(), "alpha");
    }

    #[test]
    fn manual_override_works() {
        // Validates: Requirement 2.7
        let mut matcher = ExtensionMatcher::from_definitions(&[]);
        let lang_id = LanguageId::new("custom").unwrap();
        matcher.set_override(1, lang_id.clone());
        assert_eq!(matcher.get_override(1), Some(&lang_id));
    }

    #[test]
    fn language_for_extension_performs_extension_only_lookup() {
        // Validates: Requirement 10.5
        let defs = vec![make_definition("rust", "Rust", &["rs"], 0)];
        let matcher = ExtensionMatcher::from_definitions(&defs);
        assert_eq!(
            matcher.language_for_extension("rs"),
            Some(LanguageId::new("rust").unwrap())
        );
        assert_eq!(matcher.language_for_extension("xyz"), None);
    }
}
