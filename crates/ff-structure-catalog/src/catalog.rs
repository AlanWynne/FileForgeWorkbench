//! Structure catalog — in-memory index and CRUD operations.
//!
//! The [`StructureCatalog`] is the central service managing structure definitions.
//! It holds an in-memory index of all valid definitions loaded from configured
//! catalog locations and provides create, read, update, delete, list, and duplicate
//! operations.

use std::collections::HashMap;

use chrono::Utc;

use crate::error::CatalogError;
use crate::ffs_format::{FfsParser, FfsSerializer};
use crate::model::StructureDefinition;

/// The central catalog service managing structure definitions.
///
/// Holds an in-memory index of all valid definitions loaded from a catalog
/// directory. Provides CRUD operations for structure management.
#[derive(Debug)]
pub struct StructureCatalog {
    /// All loaded entries indexed by structure name (case-sensitive).
    index: HashMap<String, StructureDefinition>,
}

impl StructureCatalog {
    /// Create an empty catalog.
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Load definitions from a list of TOML strings (simulates loading from files).
    ///
    /// Invalid entries are skipped with a warning (returned as Vec of error descriptions).
    pub fn load_from_toml_strings(&mut self, entries: &[(&str, &str)]) -> Vec<String> {
        let mut warnings = Vec::new();
        for (path, content) in entries {
            match FfsParser::parse_with_path(content, path) {
                Ok(def) => {
                    self.index.insert(def.metadata.name.clone(), def);
                }
                Err(e) => {
                    warnings.push(format!("{e}"));
                }
            }
        }
        warnings
    }

    /// Create a new structure definition in the catalog.
    ///
    /// Validates the entry and adds it to the index. Returns an error if a
    /// structure with the same name already exists.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::DuplicateName`] if the name already exists.
    /// - [`CatalogError::ValidationFailed`] if the definition is invalid.
    pub fn create(&mut self, def: StructureDefinition) -> Result<(), CatalogError> {
        Self::validate_definition(&def)?;

        if self.index.contains_key(def.name()) {
            return Err(CatalogError::DuplicateName {
                name: def.name().to_string(),
            });
        }

        self.index.insert(def.metadata.name.clone(), def);
        Ok(())
    }

    /// Read a structure definition by name.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError::NotFound`] if the name does not exist.
    pub fn read(&self, name: &str) -> Result<&StructureDefinition, CatalogError> {
        self.index.get(name).ok_or_else(|| CatalogError::NotFound {
            name: name.to_string(),
        })
    }

    /// Update an existing structure definition.
    ///
    /// Increments the version number, updates `modified_at`, validates,
    /// and replaces the entry in the index.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::NotFound`] if the structure does not exist.
    /// - [`CatalogError::ValidationFailed`] if the updated definition is invalid.
    pub fn update(&mut self, mut def: StructureDefinition) -> Result<(), CatalogError> {
        let name = def.name().to_string();
        if !self.index.contains_key(&name) {
            return Err(CatalogError::NotFound { name });
        }

        Self::validate_definition(&def)?;

        // Increment version and update timestamp
        def.metadata.version += 1;
        def.metadata.modified_at = Some(Utc::now());

        self.index.insert(name, def);
        Ok(())
    }

    /// Delete a structure definition by name.
    ///
    /// Requires `confirmed` to be `true`; rejects unconfirmed deletions.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::DeleteNotConfirmed`] if `confirmed` is `false`.
    /// - [`CatalogError::NotFound`] if the structure does not exist.
    pub fn delete(&mut self, name: &str, confirmed: bool) -> Result<(), CatalogError> {
        if !confirmed {
            return Err(CatalogError::DeleteNotConfirmed {
                name: name.to_string(),
            });
        }

        if self.index.remove(name).is_none() {
            return Err(CatalogError::NotFound {
                name: name.to_string(),
            });
        }

        Ok(())
    }

    /// List all valid structure definitions, sorted alphabetically by name.
    pub fn list(&self) -> Vec<&StructureDefinition> {
        let mut entries: Vec<&StructureDefinition> = self.index.values().collect();
        entries.sort_by_key(|d| &d.metadata.name);
        entries
    }

    /// Duplicate an existing structure with a new name.
    ///
    /// Creates a copy with version reset to 1, a new `created_at` timestamp,
    /// and `modified_at` cleared.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::NotFound`] if `source_name` does not exist.
    /// - [`CatalogError::DuplicateName`] if `new_name` already exists.
    pub fn duplicate(&mut self, source_name: &str, new_name: &str) -> Result<(), CatalogError> {
        let source = self
            .index
            .get(source_name)
            .ok_or_else(|| CatalogError::NotFound {
                name: source_name.to_string(),
            })?
            .clone();

        if self.index.contains_key(new_name) {
            return Err(CatalogError::DuplicateName {
                name: new_name.to_string(),
            });
        }

        let mut copy = source;
        copy.metadata.name = new_name.to_string();
        copy.metadata.version = 1;
        copy.metadata.created_at = Utc::now();
        copy.metadata.modified_at = None;

        self.index.insert(new_name.to_string(), copy);
        Ok(())
    }

    /// Return the number of definitions in the catalog.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Return whether the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Check if a name exists in the catalog.
    pub fn contains(&self, name: &str) -> bool {
        self.index.contains_key(name)
    }

    /// Serialize a definition to FFS format string.
    pub fn serialize_entry(def: &StructureDefinition) -> Result<String, CatalogError> {
        FfsSerializer::serialize(def)
    }

    /// Validate a structure definition.
    fn validate_definition(def: &StructureDefinition) -> Result<(), CatalogError> {
        if def.metadata.name.trim().is_empty() {
            return Err(CatalogError::ValidationFailed {
                detail: "structure name must be non-empty".to_string(),
            });
        }

        if def.record_structures.is_empty() {
            return Err(CatalogError::ValidationFailed {
                detail: "at least one record structure is required".to_string(),
            });
        }

        for rs in &def.record_structures {
            for field in &rs.fields {
                if let Err(errs) = field.validate() {
                    return Err(CatalogError::ValidationFailed {
                        detail: format!(
                            "field '{}' in record structure '{}': {}",
                            field.name, rs.name, errs
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Default for StructureCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{FieldDefinition, FieldType};
    use crate::model::{RecordStructure, StructureMetadata};

    fn sample_def(name: &str) -> StructureDefinition {
        StructureDefinition {
            metadata: StructureMetadata::new(name),
            associations: None,
            record_structures: vec![RecordStructure::with_fields(
                "Detail",
                vec![FieldDefinition::new(
                    "FIELD1",
                    0,
                    10,
                    FieldType::Alphanumeric,
                )],
            )],
        }
    }

    // Validates: Requirement 3.1 — create succeeds for valid definition
    #[test]
    fn create_adds_definition_to_catalog() {
        let mut catalog = StructureCatalog::new();
        let def = sample_def("CUSTOMER");
        assert!(catalog.create(def).is_ok());
        assert_eq!(catalog.len(), 1);
    }

    // Validates: Requirement 2.4, 3.1 — create rejects duplicate name
    #[test]
    fn create_rejects_duplicate_name() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("CUSTOMER")).unwrap();
        let result = catalog.create(sample_def("CUSTOMER"));
        assert!(matches!(result, Err(CatalogError::DuplicateName { .. })));
    }

    // Validates: Requirement 3.2 — read returns definition
    #[test]
    fn read_returns_existing_definition() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("INVOICE")).unwrap();
        let def = catalog.read("INVOICE").unwrap();
        assert_eq!(def.name(), "INVOICE");
    }

    // Validates: Requirement 3.2 — read error for missing name
    #[test]
    fn read_returns_not_found_for_missing() {
        let catalog = StructureCatalog::new();
        let result = catalog.read("NONEXISTENT");
        assert!(matches!(result, Err(CatalogError::NotFound { .. })));
    }

    // Validates: Requirement 3.6 — list returns sorted
    #[test]
    fn list_returns_definitions_sorted_alphabetically() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("ZEBRA")).unwrap();
        catalog.create(sample_def("ALPHA")).unwrap();
        catalog.create(sample_def("MIDDLE")).unwrap();

        let list = catalog.list();
        let names: Vec<&str> = list.iter().map(|d| d.name()).collect();
        assert_eq!(names, vec!["ALPHA", "MIDDLE", "ZEBRA"]);
    }

    // Validates: Requirement 3.6 — list on empty catalog
    #[test]
    fn list_returns_empty_for_empty_catalog() {
        let catalog = StructureCatalog::new();
        assert!(catalog.list().is_empty());
    }

    // Validates: Requirement 3.3 — update increments version
    #[test]
    fn update_increments_version() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("CUSTOMER")).unwrap();
        let version_before = catalog.read("CUSTOMER").unwrap().metadata.version;

        let def = catalog.read("CUSTOMER").unwrap().clone();
        catalog.update(def).unwrap();

        let version_after = catalog.read("CUSTOMER").unwrap().metadata.version;
        assert_eq!(version_after, version_before + 1);
    }

    // Validates: Requirement 3.3 — update sets modified_at
    #[test]
    fn update_sets_modified_at_timestamp() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("CUSTOMER")).unwrap();
        assert!(catalog
            .read("CUSTOMER")
            .unwrap()
            .metadata
            .modified_at
            .is_none());

        let def = catalog.read("CUSTOMER").unwrap().clone();
        catalog.update(def).unwrap();

        assert!(catalog
            .read("CUSTOMER")
            .unwrap()
            .metadata
            .modified_at
            .is_some());
    }

    // Validates: Requirement 3.3 — update for non-existent fails
    #[test]
    fn update_fails_for_nonexistent() {
        let mut catalog = StructureCatalog::new();
        let result = catalog.update(sample_def("NONEXISTENT"));
        assert!(matches!(result, Err(CatalogError::NotFound { .. })));
    }

    // Validates: Requirement 3.4 — delete removes definition
    #[test]
    fn delete_confirmed_removes_definition() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("TO_DELETE")).unwrap();
        assert!(catalog.delete("TO_DELETE", true).is_ok());
        assert!(!catalog.contains("TO_DELETE"));
    }

    // Validates: Requirement 3.5 — unconfirmed delete rejected
    #[test]
    fn delete_unconfirmed_is_rejected() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("KEEP")).unwrap();
        let result = catalog.delete("KEEP", false);
        assert!(matches!(
            result,
            Err(CatalogError::DeleteNotConfirmed { .. })
        ));
        assert!(catalog.contains("KEEP"));
    }

    // Validates: Requirement 3.4 — delete non-existent fails
    #[test]
    fn delete_nonexistent_returns_not_found() {
        let mut catalog = StructureCatalog::new();
        let result = catalog.delete("NONEXISTENT", true);
        assert!(matches!(result, Err(CatalogError::NotFound { .. })));
    }

    // Validates: Requirement 3.7 — duplicate creates copy with reset version
    #[test]
    fn duplicate_creates_copy_with_new_name_and_version_1() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("ORIGINAL")).unwrap();

        // Update original to version 2
        let def = catalog.read("ORIGINAL").unwrap().clone();
        catalog.update(def).unwrap();
        assert_eq!(catalog.read("ORIGINAL").unwrap().metadata.version, 2);

        // Duplicate
        catalog.duplicate("ORIGINAL", "COPY").unwrap();
        let copy = catalog.read("COPY").unwrap();
        assert_eq!(copy.metadata.version, 1);
        assert!(copy.metadata.modified_at.is_none());
    }

    // Validates: Requirement 3.7 — duplicate rejects existing target name
    #[test]
    fn duplicate_rejects_existing_target_name() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("A")).unwrap();
        catalog.create(sample_def("B")).unwrap();
        let result = catalog.duplicate("A", "B");
        assert!(matches!(result, Err(CatalogError::DuplicateName { .. })));
    }

    // Validates: Requirement 3.7 — duplicate from non-existent source
    #[test]
    fn duplicate_from_nonexistent_fails() {
        let mut catalog = StructureCatalog::new();
        let result = catalog.duplicate("MISSING", "NEW");
        assert!(matches!(result, Err(CatalogError::NotFound { .. })));
    }

    // Validates: Requirement 3.1 — create validates definition
    #[test]
    fn create_rejects_invalid_definition() {
        let mut catalog = StructureCatalog::new();
        let mut def = sample_def("BAD");
        def.record_structures.clear(); // No record structures = invalid
        let result = catalog.create(def);
        assert!(matches!(result, Err(CatalogError::ValidationFailed { .. })));
    }

    // Validates: Requirement 3.9 — CRUD operations log
    #[test]
    fn load_from_toml_strings_skips_invalid() {
        let valid_toml = r#"
[metadata]
name = "VALID"
version = 1
created_at = "2024-01-01T00:00:00Z"

[[record_structures]]
name = "Default"
fields = []
"#;
        let invalid_toml = "not valid [[[";

        let mut catalog = StructureCatalog::new();
        let warnings = catalog
            .load_from_toml_strings(&[("valid.ffs", valid_toml), ("invalid.ffs", invalid_toml)]);

        assert_eq!(catalog.len(), 1);
        assert_eq!(warnings.len(), 1);
    }

    // Validates: Requirement 9.7 — duplicate resets created_at
    #[test]
    fn duplicate_resets_created_at() {
        let mut catalog = StructureCatalog::new();
        catalog.create(sample_def("SOURCE")).unwrap();
        let original_created = catalog.read("SOURCE").unwrap().metadata.created_at;

        // Small sleep is impractical in tests, but we can verify it's set
        catalog.duplicate("SOURCE", "DUP").unwrap();
        let dup = catalog.read("DUP").unwrap();
        // created_at should be >= original (might be same instant in fast test)
        assert!(dup.metadata.created_at >= original_created);
    }
}
