//! Error types for the language service crate.
//!
//! All errors follow the `[lang-service] operation: description` format.

/// Opaque identifier for a document, used to key per-document state.
pub type DocumentId = u64;

/// Errors originating from the ff-language-service crate.
///
/// Formatted per Error Message Standards: `[lang-service] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LanguageServiceError {
    /// A TOML definition file failed to parse.
    #[error("[lang-service] load: failed to parse '{path}': {reason}")]
    ParseError {
        /// Path to the file that failed parsing.
        path: String,
        /// Description of the parse error.
        reason: String,
    },

    /// A TOML definition file failed schema validation.
    #[error("[lang-service] validate: definition in '{path}' missing required field '{field}'")]
    SchemaValidation {
        /// Path to the file with the validation error.
        path: String,
        /// The missing or invalid field name.
        field: String,
    },

    /// Attempted to register a language_id that already exists.
    #[error(
        "[lang-service] register: language '{language_id}' already registered (owned by {owner})"
    )]
    DuplicateLanguage {
        /// The conflicting language identifier.
        language_id: String,
        /// Description of the current owner.
        owner: String,
    },

    /// Invalid language_id format.
    #[error("[lang-service] validate: invalid language_id '{id}' — must be lowercase ASCII alphanumeric, hyphens, or underscores")]
    InvalidLanguageId {
        /// The invalid identifier string.
        id: String,
    },

    /// Unknown language_id referenced in an operation.
    #[error("[lang-service] lookup: language '{language_id}' not found in registry")]
    LanguageNotFound {
        /// The missing language identifier.
        language_id: String,
    },

    /// Document not tracked for line state operations.
    #[error("[lang-service] state: document {document_id} has no line state vector")]
    DocumentNotTracked {
        /// The untracked document identifier.
        document_id: DocumentId,
    },

    /// Line index out of bounds for line state operations.
    #[error(
        "[lang-service] state: line {line_index} out of bounds (document has {line_count} lines)"
    )]
    LineOutOfBounds {
        /// The invalid line index.
        line_index: usize,
        /// Total number of lines in the document.
        line_count: usize,
    },

    /// Keyword set number out of range (must be 0–8).
    #[error("[lang-service] keyword: set number {set_number} out of range [0, 8]")]
    KeywordSetOutOfRange {
        /// The invalid set number.
        set_number: u8,
    },

    /// Property value could not be parsed as the requested type.
    #[error("[lang-service] property: key '{key}' value '{value}' is not a valid {expected_type}")]
    PropertyParseError {
        /// The property key.
        key: String,
        /// The unparseable value.
        value: String,
        /// The expected type description.
        expected_type: String,
    },

    /// Configuration hot-reload encountered an error.
    #[error("[lang-service] reload: failed to reload language definitions: {reason}")]
    ReloadError {
        /// Description of the reload failure.
        reason: String,
    },

    /// An embedded language reference could not be resolved.
    #[error("[lang-service] embedded: language '{language_id}' referenced as embedded but not registered")]
    UnresolvedEmbedded {
        /// The unresolvable language identifier.
        language_id: String,
    },

    /// I/O error during file operations.
    #[error("[lang-service] io: {0}")]
    Io(#[from] std::io::Error),
}
