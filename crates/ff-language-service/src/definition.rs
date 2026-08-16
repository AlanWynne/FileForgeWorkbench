//! Language definition types: LanguageId, LanguageDefinition, and related structs.

use std::collections::HashMap;

use serde::Deserialize;

use crate::error::LanguageServiceError;
use crate::keyword_set::KeywordSets;

/// A unique identifier for a registered language.
///
/// Lowercase ASCII string containing only alphanumeric characters, hyphens,
/// and underscores. Examples: `"rust"`, `"cobol"`, `"jcl"`, `"plain_text"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LanguageId(String);

impl LanguageId {
    /// Create a new LanguageId from the given string.
    ///
    /// The input is lowercased and validated to contain only ASCII alphanumeric
    /// characters, hyphens, and underscores.
    ///
    /// # Errors
    ///
    /// Returns `LanguageServiceError::InvalidLanguageId` if the input contains
    /// invalid characters or is empty.
    pub fn new(id: impl Into<String>) -> Result<Self, LanguageServiceError> {
        let id = id.into().to_lowercase();
        if id.is_empty() {
            return Err(LanguageServiceError::InvalidLanguageId { id });
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(LanguageServiceError::InvalidLanguageId { id });
        }
        Ok(Self(id))
    }

    /// The "plain text" sentinel language ID used when no language is detected.
    pub fn plain_text() -> Self {
        Self("plain_text".to_string())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if this is the plain-text sentinel.
    pub fn is_plain_text(&self) -> bool {
        self.0 == "plain_text"
    }
}

impl Default for LanguageId {
    fn default() -> Self {
        Self::plain_text()
    }
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Configuration layer for file-loaded definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigLayer {
    /// Built-in defaults directory.
    BuiltIn,
    /// User configuration directory.
    User,
    /// Project-local directory.
    Project,
}

/// Tracks where a language definition was loaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionSource {
    /// Loaded from a TOML file at the given path.
    File {
        /// Path to the definition file.
        path: String,
        /// Configuration layer this file belongs to.
        layer: ConfigLayer,
    },
    /// Registered at runtime by a plugin.
    Plugin {
        /// Name of the plugin that registered this definition.
        plugin_name: String,
    },
}

impl std::fmt::Display for DefinitionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File { path, layer } => write!(f, "file '{}' ({:?})", path, layer),
            Self::Plugin { plugin_name } => write!(f, "plugin '{}'", plugin_name),
        }
    }
}

/// Describes an embedded language region within a host document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedLanguageDescriptor {
    /// The language_id of the embedded language.
    pub language_id: LanguageId,
    /// Pattern matching the start of the embedded region.
    pub start_pattern: String,
    /// Pattern matching the end of the embedded region.
    pub end_pattern: String,
}

/// Fold keyword definitions for code folding support.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldKeywords {
    /// Keywords that open a fold region.
    pub open: Vec<String>,
    /// Keywords that close a fold region.
    pub close: Vec<String>,
}

/// Comment syntax metadata for a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentSyntax {
    /// Single-line comment markers (one or more styles).
    pub line_comments: Vec<String>,
    /// Block comment start delimiter.
    pub block_comment_start: Option<String>,
    /// Block comment end delimiter.
    pub block_comment_end: Option<String>,
}

/// String syntax metadata for a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringSyntax {
    /// String delimiter characters/sequences.
    pub delimiters: Vec<String>,
    /// Character literal delimiter.
    pub character_delimiter: Option<String>,
    /// Escape character within strings.
    pub escape_character: Option<char>,
    /// Heredoc start patterns.
    pub heredoc_patterns: Vec<String>,
}

/// A complete language definition loaded from TOML or registered by a plugin.
#[derive(Debug, Clone)]
pub struct LanguageDefinition {
    /// Unique language identifier (lowercase ASCII).
    pub language_id: LanguageId,
    /// Human-readable display name.
    pub name: String,
    /// File extensions this language matches (without leading dot).
    pub extensions: Vec<String>,
    /// Priority for extension conflict resolution (higher wins, default 0).
    pub priority: i32,
    /// Whether keyword matching is case-sensitive for this language.
    pub case_sensitive_keywords: bool,
    /// Up to 9 keyword sets (indices 0–8).
    pub keyword_sets: KeywordSets,
    /// Single-line comment markers.
    pub line_comments: Vec<String>,
    /// Block comment start delimiter.
    pub block_comment_start: Option<String>,
    /// Block comment end delimiter.
    pub block_comment_end: Option<String>,
    /// String delimiter characters/sequences.
    pub string_delimiters: Vec<String>,
    /// Character literal delimiter.
    pub character_delimiter: Option<String>,
    /// Escape character within strings.
    pub escape_character: Option<char>,
    /// Heredoc start patterns.
    pub heredoc_patterns: Vec<String>,
    /// Shebang interpreter patterns for content-based detection.
    pub shebang_patterns: Vec<String>,
    /// Magic bytes at file start for content-based detection.
    pub magic_bytes: Option<Vec<u8>>,
    /// First-line regex pattern for content-based detection.
    pub first_line_pattern: Option<String>,
    /// Embedded language descriptors.
    pub embedded_languages: Vec<EmbeddedLanguageDescriptor>,
    /// Per-language properties (key-value configuration).
    pub properties: HashMap<String, String>,
    /// Fold keyword definitions.
    pub fold_keywords: Option<FoldKeywords>,
    /// Source of this definition.
    pub source: DefinitionSource,
}

impl LanguageDefinition {
    /// Returns the language identifier.
    pub fn language_id(&self) -> &LanguageId {
        &self.language_id
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the file extensions.
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }

    /// Returns the priority.
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// Returns whether keyword matching is case-sensitive.
    pub fn case_sensitive_keywords(&self) -> bool {
        self.case_sensitive_keywords
    }

    /// Returns a reference to the keyword sets.
    pub fn keyword_sets(&self) -> &KeywordSets {
        &self.keyword_sets
    }

    /// Returns the comment syntax metadata.
    pub fn comment_syntax(&self) -> CommentSyntax {
        CommentSyntax {
            line_comments: self.line_comments.clone(),
            block_comment_start: self.block_comment_start.clone(),
            block_comment_end: self.block_comment_end.clone(),
        }
    }

    /// Returns the string syntax metadata.
    pub fn string_syntax(&self) -> StringSyntax {
        StringSyntax {
            delimiters: self.string_delimiters.clone(),
            character_delimiter: self.character_delimiter.clone(),
            escape_character: self.escape_character,
            heredoc_patterns: self.heredoc_patterns.clone(),
        }
    }

    /// Returns the line comment markers.
    pub fn line_comments(&self) -> &[String] {
        &self.line_comments
    }

    /// Returns the block comment delimiters as (start, end) if both are present.
    pub fn block_comments(&self) -> Option<(&str, &str)> {
        match (&self.block_comment_start, &self.block_comment_end) {
            (Some(start), Some(end)) => Some((start.as_str(), end.as_str())),
            _ => None,
        }
    }

    /// Returns the string delimiters.
    pub fn string_delimiters(&self) -> &[String] {
        &self.string_delimiters
    }

    /// Returns the character delimiter.
    pub fn character_delimiter(&self) -> Option<&str> {
        self.character_delimiter.as_deref()
    }

    /// Returns the escape character.
    pub fn escape_character(&self) -> Option<char> {
        self.escape_character
    }

    /// Returns the heredoc patterns.
    pub fn heredoc_patterns(&self) -> &[String] {
        &self.heredoc_patterns
    }

    /// Returns the shebang patterns.
    pub fn shebang_patterns(&self) -> &[String] {
        &self.shebang_patterns
    }

    /// Returns the magic bytes.
    pub fn magic_bytes(&self) -> Option<&[u8]> {
        self.magic_bytes.as_deref()
    }

    /// Returns the first-line pattern.
    pub fn first_line_pattern(&self) -> Option<&str> {
        self.first_line_pattern.as_deref()
    }

    /// Returns the embedded language descriptors.
    pub fn embedded_languages(&self) -> &[EmbeddedLanguageDescriptor] {
        &self.embedded_languages
    }

    /// Returns the properties map.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// Returns the definition source.
    pub fn source(&self) -> &DefinitionSource {
        &self.source
    }
}

/// Lightweight summary of a registered language for listing/display purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSummary {
    /// Unique language identifier.
    pub language_id: LanguageId,
    /// Human-readable display name.
    pub display_name: String,
    /// Associated file extensions.
    pub extensions: Vec<String>,
}

/// TOML deserialization structure for a language definition file.
#[derive(Debug, Deserialize)]
pub(crate) struct TomlLanguageDefinition {
    pub name: Option<String>,
    pub language_id: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub priority: Option<i32>,
    pub case_sensitive_keywords: Option<bool>,
    pub line_comment: Option<TomlLineComment>,
    pub block_comment_start: Option<String>,
    pub block_comment_end: Option<String>,
    pub string_delimiters: Option<Vec<String>>,
    pub character_delimiter: Option<String>,
    pub escape_character: Option<String>,
    pub heredoc_patterns: Option<Vec<String>>,
    pub shebang_patterns: Option<Vec<String>>,
    pub magic_bytes: Option<Vec<u8>>,
    pub first_line_pattern: Option<String>,
    pub keywords: Option<HashMap<String, Vec<String>>>,
    pub properties: Option<HashMap<String, String>>,
    pub embedded_languages: Option<Vec<TomlEmbeddedLanguage>>,
    pub fold_keywords: Option<TomlFoldKeywords>,
}

/// Support both a single string and an array of strings for `line_comment`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum TomlLineComment {
    Single(String),
    Multiple(Vec<String>),
}

/// TOML representation of an embedded language descriptor.
#[derive(Debug, Deserialize)]
pub(crate) struct TomlEmbeddedLanguage {
    pub language_id: String,
    pub start_pattern: String,
    pub end_pattern: String,
}

/// TOML representation of fold keywords.
#[derive(Debug, Deserialize)]
pub(crate) struct TomlFoldKeywords {
    pub open: Option<Vec<String>>,
    pub close: Option<Vec<String>>,
}

impl TomlLanguageDefinition {
    /// Convert a parsed TOML structure into a LanguageDefinition.
    ///
    /// # Errors
    ///
    /// Returns `LanguageServiceError::SchemaValidation` if required fields are missing.
    pub(crate) fn into_definition(
        self,
        source: DefinitionSource,
    ) -> Result<LanguageDefinition, LanguageServiceError> {
        let path_for_error = match &source {
            DefinitionSource::File { path, .. } => path.clone(),
            DefinitionSource::Plugin { plugin_name } => plugin_name.clone(),
        };

        let name = self
            .name
            .ok_or_else(|| LanguageServiceError::SchemaValidation {
                path: path_for_error.clone(),
                field: "name".to_string(),
            })?;

        let language_id_str =
            self.language_id
                .ok_or_else(|| LanguageServiceError::SchemaValidation {
                    path: path_for_error.clone(),
                    field: "language_id".to_string(),
                })?;

        let language_id = LanguageId::new(&language_id_str)?;

        let extensions = self
            .extensions
            .ok_or_else(|| LanguageServiceError::SchemaValidation {
                path: path_for_error.clone(),
                field: "extensions".to_string(),
            })?;

        let case_sensitive = self.case_sensitive_keywords.unwrap_or(true);

        let keyword_sets = match self.keywords {
            Some(kw_table) => KeywordSets::from_toml_table(&kw_table, case_sensitive),
            None => KeywordSets::empty(),
        };

        let line_comments = match self.line_comment {
            Some(TomlLineComment::Single(s)) => vec![s],
            Some(TomlLineComment::Multiple(v)) => v,
            None => Vec::new(),
        };

        let escape_character = self.escape_character.and_then(|s| s.chars().next());

        let embedded_languages = self
            .embedded_languages
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| {
                LanguageId::new(&e.language_id)
                    .ok()
                    .map(|lid| EmbeddedLanguageDescriptor {
                        language_id: lid,
                        start_pattern: e.start_pattern,
                        end_pattern: e.end_pattern,
                    })
            })
            .collect();

        let fold_keywords = self.fold_keywords.map(|fk| FoldKeywords {
            open: fk.open.unwrap_or_default(),
            close: fk.close.unwrap_or_default(),
        });

        Ok(LanguageDefinition {
            language_id,
            name,
            extensions,
            priority: self.priority.unwrap_or(0),
            case_sensitive_keywords: case_sensitive,
            keyword_sets,
            line_comments,
            block_comment_start: self.block_comment_start,
            block_comment_end: self.block_comment_end,
            string_delimiters: self.string_delimiters.unwrap_or_default(),
            character_delimiter: self.character_delimiter,
            escape_character,
            heredoc_patterns: self.heredoc_patterns.unwrap_or_default(),
            shebang_patterns: self.shebang_patterns.unwrap_or_default(),
            magic_bytes: self.magic_bytes,
            first_line_pattern: self.first_line_pattern,
            embedded_languages,
            properties: self.properties.unwrap_or_default(),
            fold_keywords,
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_id_new_accepts_valid_lowercase() {
        // Validates: Requirement 1.2
        let id = LanguageId::new("rust").unwrap();
        assert_eq!(id.as_str(), "rust");
    }

    #[test]
    fn language_id_new_lowercases_input() {
        // Validates: Requirement 1.2
        let id = LanguageId::new("RUST").unwrap();
        assert_eq!(id.as_str(), "rust");
    }

    #[test]
    fn language_id_new_accepts_hyphens_and_underscores() {
        // Validates: Requirement 1.2
        let id = LanguageId::new("c-sharp").unwrap();
        assert_eq!(id.as_str(), "c-sharp");

        let id = LanguageId::new("plain_text").unwrap();
        assert_eq!(id.as_str(), "plain_text");
    }

    #[test]
    fn language_id_new_rejects_empty_string() {
        // Validates: Requirement 1.2
        let result = LanguageId::new("");
        assert!(result.is_err());
    }

    #[test]
    fn language_id_new_rejects_special_characters() {
        // Validates: Requirement 1.2
        let result = LanguageId::new("c++");
        assert!(result.is_err());

        let result = LanguageId::new("lang.ext");
        assert!(result.is_err());
    }

    #[test]
    fn language_id_plain_text_returns_sentinel() {
        // Validates: Requirement 2.5
        let id = LanguageId::plain_text();
        assert_eq!(id.as_str(), "plain_text");
        assert!(id.is_plain_text());
    }

    #[test]
    fn language_id_default_returns_plain_text() {
        // Validates: Requirement 2.5
        let id = LanguageId::default();
        assert!(id.is_plain_text());
    }

    #[test]
    fn toml_deserialization_minimal_definition() {
        // Validates: Requirement 1.2
        let toml_str = r#"
name = "Rust"
language_id = "rust"
extensions = ["rs"]
"#;
        let parsed: TomlLanguageDefinition = toml::from_str(toml_str).unwrap();
        let def = parsed
            .into_definition(DefinitionSource::File {
                path: "test.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            })
            .unwrap();

        assert_eq!(def.language_id().as_str(), "rust");
        assert_eq!(def.name(), "Rust");
        assert_eq!(def.extensions(), &["rs"]);
        assert_eq!(def.priority(), 0);
        assert!(def.case_sensitive_keywords());
    }

    #[test]
    fn toml_deserialization_full_definition() {
        // Validates: Requirements 1.2, 1.3
        let toml_str = r##"
name = "Python"
language_id = "python"
extensions = ["py", "pyw"]
priority = 10
case_sensitive_keywords = true
line_comment = "#"
string_delimiters = ["\"", "'", "\"\"\"", "'''"]
escape_character = "\\"
shebang_patterns = ["python", "python3"]
first_line_pattern = "^#.*coding"

[keywords]
"0" = ["def", "class", "import", "from", "return"]
"1" = ["int", "str", "float", "bool", "list"]

[properties]
"fold.comment" = "1"
"tab.size" = "4"
"##;
        let parsed: TomlLanguageDefinition = toml::from_str(toml_str).unwrap();
        let def = parsed
            .into_definition(DefinitionSource::File {
                path: "python.toml".to_string(),
                layer: ConfigLayer::User,
            })
            .unwrap();

        assert_eq!(def.language_id().as_str(), "python");
        assert_eq!(def.name(), "Python");
        assert_eq!(def.extensions(), &["py", "pyw"]);
        assert_eq!(def.priority(), 10);
        assert_eq!(def.line_comments(), &["#"]);
        assert_eq!(def.escape_character(), Some('\\'));
        assert_eq!(def.shebang_patterns(), &["python", "python3"]);
        assert_eq!(def.first_line_pattern(), Some("^#.*coding"));
        assert_eq!(def.properties().get("fold.comment"), Some(&"1".to_string()));
    }

    #[test]
    fn toml_deserialization_multiple_line_comments() {
        // Validates: Requirement 6.3
        let toml_str = r#"
name = "Rust"
language_id = "rust"
extensions = ["rs"]
line_comment = ["//", "///", "//!"]
"#;
        let parsed: TomlLanguageDefinition = toml::from_str(toml_str).unwrap();
        let def = parsed
            .into_definition(DefinitionSource::File {
                path: "rust.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            })
            .unwrap();

        assert_eq!(def.line_comments(), &["//", "///", "//!"]);
    }

    #[test]
    fn toml_deserialization_missing_name_returns_error() {
        // Validates: Requirement 1.2
        let toml_str = r#"
language_id = "test"
extensions = ["txt"]
"#;
        let parsed: TomlLanguageDefinition = toml::from_str(toml_str).unwrap();
        let result = parsed.into_definition(DefinitionSource::File {
            path: "bad.toml".to_string(),
            layer: ConfigLayer::BuiltIn,
        });
        assert!(result.is_err());
    }

    #[test]
    fn toml_deserialization_missing_language_id_returns_error() {
        // Validates: Requirement 1.2
        let toml_str = r#"
name = "Test"
extensions = ["txt"]
"#;
        let parsed: TomlLanguageDefinition = toml::from_str(toml_str).unwrap();
        let result = parsed.into_definition(DefinitionSource::File {
            path: "bad.toml".to_string(),
            layer: ConfigLayer::BuiltIn,
        });
        assert!(result.is_err());
    }

    #[test]
    fn toml_deserialization_with_embedded_languages() {
        // Validates: Requirement 7.1
        let toml_str = r#"
name = "HTML"
language_id = "html"
extensions = ["html", "htm"]

[[embedded_languages]]
language_id = "javascript"
start_pattern = "<script>"
end_pattern = "</script>"

[[embedded_languages]]
language_id = "css"
start_pattern = "<style>"
end_pattern = "</style>"
"#;
        let parsed: TomlLanguageDefinition = toml::from_str(toml_str).unwrap();
        let def = parsed
            .into_definition(DefinitionSource::File {
                path: "html.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            })
            .unwrap();

        assert_eq!(def.embedded_languages().len(), 2);
        assert_eq!(
            def.embedded_languages()[0].language_id.as_str(),
            "javascript"
        );
        assert_eq!(def.embedded_languages()[0].start_pattern, "<script>");
        assert_eq!(def.embedded_languages()[1].language_id.as_str(), "css");
    }
}
