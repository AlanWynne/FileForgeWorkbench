//! Trait interfaces for upstream dependencies.
//!
//! These traits allow `ff-tabmask` to interact with the configuration system,
//! language service, and document model without compile-time dependencies on
//! those crates. The session orchestration layer provides concrete implementations.

/// Provides access to configuration values relevant to tab stop management.
///
/// Implemented by the configuration system to supply `editor.default_tab_stops`
/// and `editor.tab_size` values.
pub trait ConfigProvider {
    /// Returns the configured default tab stops as a list of column positions.
    /// Returns an empty vec if the key is absent.
    fn get_tab_stops(&self) -> Vec<u32>;

    /// Returns the configured tab size (spaces per tab).
    /// Returns a sensible default (e.g., 8) if the key is absent.
    fn get_tab_size(&self) -> u32;
}

/// Provides access to language definition values for tab stops and mask.
///
/// Implemented by the language service to supply per-language defaults.
pub struct LanguageDefinitionRef<'a> {
    /// The raw TOML table for the language definition.
    table: &'a toml::Value,
}

impl<'a> LanguageDefinitionRef<'a> {
    /// Creates a new language definition reference from a TOML value.
    pub fn new(table: &'a toml::Value) -> Self {
        Self { table }
    }

    /// Returns the `default_tab_stops` array from the language definition, if present.
    /// Filters out non-positive-integer values.
    pub fn default_tab_stops(&self) -> Option<Vec<u32>> {
        self.table
            .get("default_tab_stops")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_integer())
                    .filter(|&n| n > 0)
                    .map(|n| n as u32)
                    .collect()
            })
    }

    /// Returns the `default_mask` value from the language definition, if present and a string.
    pub fn default_mask(&self) -> Option<&'a toml::Value> {
        self.table.get("default_mask")
    }
}

/// Provides document context information for artifact positioning and rendering.
pub trait DocumentContext {
    /// Returns the width of a line in columns.
    fn line_width(&self) -> usize;

    /// Returns the number of lines in the document.
    fn line_count(&self) -> usize;

    /// Returns the current cursor line (0-indexed), if known.
    fn cursor_line(&self) -> Option<usize>;
}
