//! Catalog browsing panel — data model and state.
//!
//! Provides the [`BrowsingPanelState`] which manages the filtered, sorted,
//! searchable list of structure definitions displayed in the catalog browsing panel.

use std::cmp::Reverse;

use crate::model::StructureDefinition;

/// Sort mode for the browsing panel list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    /// Sort alphabetically by structure name.
    #[default]
    ByName,
    /// Sort by last modification date (most recent first).
    ByModifiedDate,
    /// Sort by total field count (descending).
    ByFieldCount,
}

/// Preview information for a selected structure definition.
#[derive(Debug, Clone, PartialEq)]
pub struct StructurePreview {
    /// Structure name.
    pub name: String,
    /// Record structure summaries (name + field count).
    pub record_structures: Vec<RecordStructureSummary>,
}

/// Summary of a single record structure for the preview.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordStructureSummary {
    /// Record structure name.
    pub name: String,
    /// Number of fields.
    pub field_count: usize,
    /// Field names in order.
    pub field_names: Vec<String>,
}

/// A single row in the browsing panel list.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowsingListEntry {
    /// Structure name.
    pub name: String,
    /// Number of record structures.
    pub record_structure_count: usize,
    /// Total field count.
    pub total_field_count: usize,
    /// Associated file patterns.
    pub file_patterns: Vec<String>,
    /// Version number.
    pub version: u32,
    /// Last modified timestamp as display string (or "never").
    pub modified_at: String,
}

/// Browsing panel state — manages the filtered, sorted view of the catalog.
#[derive(Debug, Default)]
pub struct BrowsingPanelState {
    /// Current search/filter text.
    search_text: String,
    /// Current sort mode.
    sort_mode: SortMode,
    /// Index of the currently selected entry (if any).
    selected_index: Option<usize>,
    /// Cached list entries after filtering and sorting.
    entries: Vec<BrowsingListEntry>,
}

impl BrowsingPanelState {
    /// Create a new empty browsing panel state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the panel with a new set of definitions.
    ///
    /// Applies the current filter and sort, then rebuilds the entry list.
    pub fn refresh(&mut self, definitions: &[&StructureDefinition]) {
        self.entries = definitions
            .iter()
            .filter(|def| self.matches_filter(def))
            .map(|def| Self::to_list_entry(def))
            .collect();

        self.apply_sort();
        // Clear selection if it's out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= self.entries.len() {
                self.selected_index = None;
            }
        }
    }

    /// Set the search/filter text and re-apply filtering.
    pub fn set_search_text(&mut self, text: &str) {
        self.search_text = text.to_string();
    }

    /// Get the current search text.
    pub fn search_text(&self) -> &str {
        &self.search_text
    }

    /// Set the sort mode and re-apply sorting.
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
        self.apply_sort();
    }

    /// Get the current sort mode.
    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    /// Select an entry by index.
    pub fn select(&mut self, index: usize) {
        if index < self.entries.len() {
            self.selected_index = Some(index);
        }
    }

    /// Get the currently selected index.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Get the filtered and sorted list entries.
    pub fn entries(&self) -> &[BrowsingListEntry] {
        &self.entries
    }

    /// Generate a preview for the selected structure.
    pub fn generate_preview(def: &StructureDefinition) -> StructurePreview {
        StructurePreview {
            name: def.metadata.name.clone(),
            record_structures: def
                .record_structures
                .iter()
                .map(|rs| RecordStructureSummary {
                    name: rs.name.clone(),
                    field_count: rs.fields.len(),
                    field_names: rs.fields.iter().map(|f| f.name.clone()).collect(),
                })
                .collect(),
        }
    }

    /// Check if a definition matches the current filter.
    ///
    /// Case-insensitive substring match against name, field names, and file patterns.
    fn matches_filter(&self, def: &StructureDefinition) -> bool {
        if self.search_text.is_empty() {
            return true;
        }

        let needle = self.search_text.to_lowercase();

        // Match against structure name
        if def.metadata.name.to_lowercase().contains(&needle) {
            return true;
        }

        // Match against field names
        for rs in &def.record_structures {
            for field in &rs.fields {
                if field.name.to_lowercase().contains(&needle) {
                    return true;
                }
            }
        }

        // Match against file patterns
        if let Some(ref assoc) = def.associations {
            for pattern in &assoc.file_patterns {
                if pattern.to_lowercase().contains(&needle) {
                    return true;
                }
            }
        }

        false
    }

    /// Apply the current sort mode to the entries.
    fn apply_sort(&mut self) {
        match self.sort_mode {
            SortMode::ByName => {
                self.entries.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortMode::ByModifiedDate => {
                self.entries
                    .sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
            }
            SortMode::ByFieldCount => {
                self.entries
                    .sort_by_key(|entry| Reverse(entry.total_field_count));
            }
        }
    }

    /// Convert a definition to a list entry.
    fn to_list_entry(def: &StructureDefinition) -> BrowsingListEntry {
        BrowsingListEntry {
            name: def.metadata.name.clone(),
            record_structure_count: def.record_structures.len(),
            total_field_count: def.total_field_count(),
            file_patterns: def
                .associations
                .as_ref()
                .map(|a| a.file_patterns.clone())
                .unwrap_or_default(),
            version: def.metadata.version,
            modified_at: def
                .metadata
                .modified_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "never".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{FieldDefinition, FieldType};
    use crate::model::{FileAssociations, RecordStructure, StructureMetadata};

    fn test_def(
        name: &str,
        field_names: &[&str],
        patterns: Option<Vec<&str>>,
    ) -> StructureDefinition {
        StructureDefinition {
            metadata: StructureMetadata::new(name),
            associations: patterns.map(|p| FileAssociations {
                file_patterns: p.into_iter().map(String::from).collect(),
            }),
            record_structures: vec![RecordStructure::with_fields(
                "Default",
                field_names
                    .iter()
                    .enumerate()
                    .map(|(i, n)| {
                        FieldDefinition::new(*n, (i * 10) as u32, 10, FieldType::Alphanumeric)
                    })
                    .collect(),
            )],
        }
    }

    // Validates: Requirement 4.3 — search filters by name
    #[test]
    fn filter_matches_structure_name_case_insensitive() {
        let def = test_def("CUSTOMER_MASTER", &["NAME"], None);
        let mut state = BrowsingPanelState::new();
        state.set_search_text("customer");
        state.refresh(&[&def]);
        assert_eq!(state.entries().len(), 1);
    }

    // Validates: Requirement 4.3 — search filters by field name
    #[test]
    fn filter_matches_field_name() {
        let def = test_def("INVOICE", &["AMOUNT", "CUSTOMER_ID"], None);
        let mut state = BrowsingPanelState::new();
        state.set_search_text("amount");
        state.refresh(&[&def]);
        assert_eq!(state.entries().len(), 1);
    }

    // Validates: Requirement 4.3 — search filters by file pattern
    #[test]
    fn filter_matches_file_pattern() {
        let def = test_def("DATA", &["F1"], Some(vec!["*.dat"]));
        let mut state = BrowsingPanelState::new();
        state.set_search_text("dat");
        state.refresh(&[&def]);
        assert_eq!(state.entries().len(), 1);
    }

    // Validates: Requirement 4.3 — no match filters out
    #[test]
    fn filter_excludes_non_matching() {
        let def = test_def("CUSTOMER", &["NAME"], None);
        let mut state = BrowsingPanelState::new();
        state.set_search_text("invoice");
        state.refresh(&[&def]);
        assert!(state.entries().is_empty());
    }

    // Validates: Requirement 4.4 — sort by name
    #[test]
    fn sort_by_name_alphabetical() {
        let def1 = test_def("ZEBRA", &["F1"], None);
        let def2 = test_def("ALPHA", &["F1"], None);
        let mut state = BrowsingPanelState::new();
        state.set_sort_mode(SortMode::ByName);
        state.refresh(&[&def1, &def2]);
        assert_eq!(state.entries()[0].name, "ALPHA");
        assert_eq!(state.entries()[1].name, "ZEBRA");
    }

    // Validates: Requirement 4.4 — sort by field count
    #[test]
    fn sort_by_field_count_descending() {
        let def1 = test_def("FEW", &["F1"], None);
        let def2 = test_def("MANY", &["F1", "F2", "F3"], None);
        let mut state = BrowsingPanelState::new();
        state.set_sort_mode(SortMode::ByFieldCount);
        state.refresh(&[&def1, &def2]);
        assert_eq!(state.entries()[0].name, "MANY");
        assert_eq!(state.entries()[1].name, "FEW");
    }

    // Validates: Requirement 4.5 — preview generation
    #[test]
    fn generate_preview_shows_structure_summary() {
        let def = test_def("INVOICE", &["AMOUNT", "DATE", "ID"], None);
        let preview = BrowsingPanelState::generate_preview(&def);
        assert_eq!(preview.name, "INVOICE");
        assert_eq!(preview.record_structures.len(), 1);
        assert_eq!(preview.record_structures[0].field_count, 3);
        assert_eq!(
            preview.record_structures[0].field_names,
            vec!["AMOUNT", "DATE", "ID"]
        );
    }

    // Validates: Requirement 4.2 — list shows all valid definitions
    #[test]
    fn refresh_with_empty_filter_shows_all() {
        let def1 = test_def("A", &["F1"], None);
        let def2 = test_def("B", &["F1"], None);
        let mut state = BrowsingPanelState::new();
        state.refresh(&[&def1, &def2]);
        assert_eq!(state.entries().len(), 2);
    }
}
