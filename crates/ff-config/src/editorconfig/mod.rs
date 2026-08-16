//! EditorConfig integration.
//!
//! Parses `.editorconfig` files and resolves per-file editor settings
//! (indent style, line endings, whitespace handling) according to the
//! EditorConfig specification (<https://editorconfig.org>).

pub mod parser;
pub mod resolver;

// Re-export commonly used types from the parser module.
pub use parser::{
    load_editorconfig_file, matches_pattern, Charset, EditorConfigFile, EditorConfigProperties,
    EditorConfigSection, EndOfLine, IndentSize, IndentStyle, ParseError,
};

// Re-export the resolver's public API.
pub use resolver::resolve_editorconfig;
