//! Structure editor — field grid model and dirty tracking.
//!
//! Provides [`EditorState`] for managing in-memory editing of a structure
//! definition including field add/remove/reorder, auto-compute offsets,
//! multi-tab record structures, and unsaved-changes tracking.

use crate::error::CatalogError;
use crate::field::{FieldDefinition, FieldType};
use crate::model::{RecordStructure, StructureDefinition};
use crate::versioning::VersionManager;

/// The editor state for a structure definition being edited.
#[derive(Debug, Clone)]
pub struct EditorState {
    /// The definition being edited.
    definition: StructureDefinition,
    /// The original (on-disk) definition for dirty comparison.
    original: StructureDefinition,
    /// Index of the active record structure tab.
    active_tab: usize,
}

impl EditorState {
    /// Open a structure definition for editing.
    pub fn open(definition: StructureDefinition) -> Self {
        let original = definition.clone();
        Self {
            definition,
            original,
            active_tab: 0,
        }
    }

    /// Get a reference to the definition being edited.
    pub fn definition(&self) -> &StructureDefinition {
        &self.definition
    }

    /// Get a mutable reference to the definition being edited.
    pub fn definition_mut(&mut self) -> &mut StructureDefinition {
        &mut self.definition
    }

    /// Get the active record structure tab index.
    pub fn active_tab(&self) -> usize {
        self.active_tab
    }

    /// Switch to a different record structure tab.
    pub fn set_active_tab(&mut self, index: usize) {
        if index < self.definition.record_structures.len() {
            self.active_tab = index;
        }
    }

    /// Get the currently active record structure.
    pub fn active_record_structure(&self) -> Option<&RecordStructure> {
        self.definition.record_structures.get(self.active_tab)
    }

    /// Get the currently active record structure mutably.
    pub fn active_record_structure_mut(&mut self) -> Option<&mut RecordStructure> {
        self.definition.record_structures.get_mut(self.active_tab)
    }

    /// Check if the definition has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.definition != self.original
    }

    /// Add a new field at the specified position in the active record structure.
    ///
    /// Defaults: empty name placeholder, next available offset, length 1, alphanumeric.
    pub fn add_field(&mut self, position: usize) {
        if let Some(rs) = self.definition.record_structures.get_mut(self.active_tab) {
            let offset = if position > 0 && position <= rs.fields.len() {
                let prev = &rs.fields[position - 1];
                prev.offset + prev.length
            } else if rs.fields.is_empty() {
                0
            } else {
                let last = rs.fields.last().unwrap();
                last.offset + last.length
            };

            let field = FieldDefinition::new("NEW_FIELD", offset, 1, FieldType::Alphanumeric);

            if position >= rs.fields.len() {
                rs.fields.push(field);
            } else {
                rs.fields.insert(position, field);
            }
        }
    }

    /// Remove the field at the specified index.
    ///
    /// Remaining fields retain their original offsets.
    pub fn remove_field(&mut self, index: usize) {
        if let Some(rs) = self.definition.record_structures.get_mut(self.active_tab) {
            if index < rs.fields.len() {
                rs.fields.remove(index);
            }
        }
    }

    /// Move a field up (toward index 0).
    pub fn move_field_up(&mut self, index: usize) {
        if index == 0 {
            return;
        }
        if let Some(rs) = self.definition.record_structures.get_mut(self.active_tab) {
            if index < rs.fields.len() {
                rs.fields.swap(index, index - 1);
            }
        }
    }

    /// Move a field down (toward the end).
    pub fn move_field_down(&mut self, index: usize) {
        if let Some(rs) = self.definition.record_structures.get_mut(self.active_tab) {
            if index + 1 < rs.fields.len() {
                rs.fields.swap(index, index + 1);
            }
        }
    }

    /// Auto-compute offsets sequentially for the active record structure.
    ///
    /// Each field offset = sum of all preceding field lengths.
    /// First field starts at offset 0.
    pub fn auto_compute_offsets(&mut self) {
        if let Some(rs) = self.definition.record_structures.get_mut(self.active_tab) {
            let mut current_offset = 0u32;
            for field in &mut rs.fields {
                field.offset = current_offset;
                current_offset += field.length;
            }
        }
    }

    /// Add a new record structure tab.
    pub fn add_record_structure(&mut self, name: impl Into<String>) {
        self.definition
            .record_structures
            .push(RecordStructure::new(name));
    }

    /// Remove a record structure tab by index.
    ///
    /// Cannot remove the last remaining tab.
    pub fn remove_record_structure(&mut self, index: usize) -> Result<(), CatalogError> {
        if self.definition.record_structures.len() <= 1 {
            return Err(CatalogError::ValidationFailed {
                detail: "cannot remove the last record structure".to_string(),
            });
        }
        if index < self.definition.record_structures.len() {
            self.definition.record_structures.remove(index);
            if self.active_tab >= self.definition.record_structures.len() {
                self.active_tab = self.definition.record_structures.len() - 1;
            }
        }
        Ok(())
    }

    /// Rename a record structure tab.
    pub fn rename_record_structure(&mut self, index: usize, new_name: impl Into<String>) {
        if let Some(rs) = self.definition.record_structures.get_mut(index) {
            rs.name = new_name.into();
        }
    }

    /// Save the edited definition: increment version and update modified_at.
    ///
    /// Returns the serialized definition and updates the original for dirty tracking.
    pub fn save(&mut self) -> Result<StructureDefinition, CatalogError> {
        VersionManager::increment(&mut self.definition.metadata);
        self.original = self.definition.clone();
        Ok(self.definition.clone())
    }

    /// Discard changes and reload from the original.
    pub fn discard(&mut self) {
        self.definition = self.original.clone();
        self.active_tab = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{FieldDefinition, FieldType};
    use crate::model::{RecordStructure, StructureMetadata};

    fn sample_def() -> StructureDefinition {
        StructureDefinition {
            metadata: StructureMetadata::new("TEST"),
            associations: None,
            record_structures: vec![RecordStructure::with_fields(
                "Detail",
                vec![
                    FieldDefinition::new("FIELD_A", 0, 10, FieldType::Alphanumeric),
                    FieldDefinition::new("FIELD_B", 10, 20, FieldType::Numeric),
                    FieldDefinition::new("FIELD_C", 30, 5, FieldType::Binary),
                ],
            )],
        }
    }

    // Validates: Requirement 5.2 — add field with defaults
    #[test]
    fn add_field_inserts_at_position_with_computed_offset() {
        let mut state = EditorState::open(sample_def());
        state.add_field(1); // Insert after first field
        let rs = state.active_record_structure().unwrap();
        assert_eq!(rs.fields.len(), 4);
        assert_eq!(rs.fields[1].name, "NEW_FIELD");
        assert_eq!(rs.fields[1].offset, 10); // prev.offset(0) + prev.length(10)
        assert_eq!(rs.fields[1].length, 1);
        assert_eq!(rs.fields[1].field_type, FieldType::Alphanumeric);
    }

    // Validates: Requirement 5.3 — remove field retains offsets
    #[test]
    fn remove_field_retains_other_offsets() {
        let mut state = EditorState::open(sample_def());
        state.remove_field(1); // Remove FIELD_B
        let rs = state.active_record_structure().unwrap();
        assert_eq!(rs.fields.len(), 2);
        assert_eq!(rs.fields[0].name, "FIELD_A");
        assert_eq!(rs.fields[0].offset, 0);
        assert_eq!(rs.fields[1].name, "FIELD_C");
        assert_eq!(rs.fields[1].offset, 30); // Unchanged
    }

    // Validates: Requirement 5.4 — reorder via move up
    #[test]
    fn move_field_up_swaps_positions_not_offsets() {
        let mut state = EditorState::open(sample_def());
        state.move_field_up(1); // Move FIELD_B up
        let rs = state.active_record_structure().unwrap();
        assert_eq!(rs.fields[0].name, "FIELD_B");
        assert_eq!(rs.fields[0].offset, 10); // Offset unchanged
        assert_eq!(rs.fields[1].name, "FIELD_A");
        assert_eq!(rs.fields[1].offset, 0); // Offset unchanged
    }

    // Validates: Requirement 5.4 — reorder via move down
    #[test]
    fn move_field_down_swaps_positions() {
        let mut state = EditorState::open(sample_def());
        state.move_field_down(0); // Move FIELD_A down
        let rs = state.active_record_structure().unwrap();
        assert_eq!(rs.fields[0].name, "FIELD_B");
        assert_eq!(rs.fields[1].name, "FIELD_A");
    }

    // Validates: Requirement 5.5 — auto-compute offsets
    #[test]
    fn auto_compute_offsets_makes_contiguous() {
        let mut state = EditorState::open(sample_def());
        // Mess up offsets first
        state.active_record_structure_mut().unwrap().fields[0].offset = 100;
        state.active_record_structure_mut().unwrap().fields[1].offset = 200;
        state.active_record_structure_mut().unwrap().fields[2].offset = 300;

        state.auto_compute_offsets();
        let rs = state.active_record_structure().unwrap();
        assert_eq!(rs.fields[0].offset, 0);
        assert_eq!(rs.fields[1].offset, 10); // field[0].length = 10
        assert_eq!(rs.fields[2].offset, 30); // 10 + 20
    }

    // Validates: Requirement 5.11 — dirty tracking
    #[test]
    fn is_dirty_after_modification() {
        let mut state = EditorState::open(sample_def());
        assert!(!state.is_dirty());
        state.add_field(0);
        assert!(state.is_dirty());
    }

    // Validates: Requirement 5.11 — discard resets dirty
    #[test]
    fn discard_resets_to_original() {
        let mut state = EditorState::open(sample_def());
        state.add_field(0);
        assert!(state.is_dirty());
        state.discard();
        assert!(!state.is_dirty());
    }

    // Validates: Requirement 5.10 — multi-tab management
    #[test]
    fn add_and_switch_record_structure_tabs() {
        let mut state = EditorState::open(sample_def());
        assert_eq!(state.definition().record_structures.len(), 1);

        state.add_record_structure("Header");
        assert_eq!(state.definition().record_structures.len(), 2);

        state.set_active_tab(1);
        assert_eq!(state.active_tab(), 1);
        assert_eq!(state.active_record_structure().unwrap().name, "Header");
    }

    // Validates: Requirement 5.10 — rename tab
    #[test]
    fn rename_record_structure_tab() {
        let mut state = EditorState::open(sample_def());
        state.rename_record_structure(0, "NewName");
        assert_eq!(state.active_record_structure().unwrap().name, "NewName");
    }

    // Validates: Requirement 5.10 — cannot remove last tab
    #[test]
    fn cannot_remove_last_record_structure() {
        let mut state = EditorState::open(sample_def());
        let result = state.remove_record_structure(0);
        assert!(result.is_err());
    }

    // Validates: Requirement 9.2 — save increments version
    #[test]
    fn save_increments_version_and_clears_dirty() {
        let mut state = EditorState::open(sample_def());
        state.add_field(0);
        assert!(state.is_dirty());

        let saved = state.save().unwrap();
        assert_eq!(saved.metadata.version, 2);
        assert!(!state.is_dirty());
    }
}
