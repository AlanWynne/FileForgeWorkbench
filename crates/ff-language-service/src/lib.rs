//! # ff-language-service — Language Detection and Definition Management
//!
//! This crate is the foundational layer responsible for language detection,
//! language definition management, multi-line lexer state persistence, and
//! content-based language identification for the FileForgeWorkbench.
//!
//! ## Responsibilities
//!
//! - Load and validate language definitions from TOML files
//! - Detect file language by extension (case-insensitive, compound-aware)
//! - Detect file language by content inspection (shebang, magic bytes, first-line patterns)
//! - Maintain per-line lexer state vectors for incremental re-highlighting
//! - Manage keyword sets (up to 9) with efficient sorted lookup
//! - Expose comment, string, and embedded language metadata
//! - Provide per-language property configuration with layered overrides
//! - Support runtime language registration/deregistration via plugins
//! - Thread-safe query API for all read operations
//!
//! ## Architecture
//!
//! This is a Wave 7 (Language and Highlighting) crate that is GUI-independent.
//! It depends on `ff-logging` (Wave 0) and `ff-config` (Wave 2), and is consumed
//! by `ff-syntax-highlighting` and `ff-auto-indentation` (Wave 7 peers).

pub mod content_detection;
pub mod definition;
pub mod detection;
pub mod embedded;
pub mod error;
pub mod keyword_set;
pub mod lexer_state;
pub mod properties;
pub mod query;
pub mod registry;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use content_detection::ContentDetector;
pub use definition::{
    ConfigLayer, DefinitionSource, EmbeddedLanguageDescriptor, FoldKeywords, LanguageDefinition,
    LanguageId, LanguageSummary,
};
pub use detection::{DetectionMethod, DetectionResult, ExtensionMatcher};
pub use embedded::EmbeddedLanguageResolver;
pub use error::LanguageServiceError;
pub use keyword_set::{KeywordSet, KeywordSets};
pub use lexer_state::{LexerState, LineStateVector, LEXER_STATE_INITIAL, LEXER_STATE_INVALID};
pub use properties::PropertyStore;
pub use query::LanguageService;
pub use registry::LanguageRegistry;
