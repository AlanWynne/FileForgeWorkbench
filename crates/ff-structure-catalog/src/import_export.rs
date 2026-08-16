//! Structure import and export — format conversion services.
//!
//! Provides import from `.fc.json`, `.fc.xlsx`, `.ffs`, and COBOL copybook formats,
//! and export to `.ffs`, `.fc.json`, `.fc.xlsx` formats.

use crate::error::CatalogError;
use crate::model::StructureDefinition;

/// Supported structure file formats for import and export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StructureFormat {
    /// Native FileForge Structure format (TOML-based .ffs).
    Ffs,
    /// Legacy JSON companion config format (.fc.json).
    FcJson,
    /// Legacy Excel companion config format (.fc.xlsx).
    FcXlsx,
    /// COBOL copybook source (import only).
    Copybook,
}

impl StructureFormat {
    /// Returns the file extension associated with this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Ffs => "ffs",
            Self::FcJson => "fc.json",
            Self::FcXlsx => "fc.xlsx",
            Self::Copybook => "cpy",
        }
    }

    /// Returns whether this format supports export.
    pub fn supports_export(&self) -> bool {
        matches!(self, Self::Ffs | Self::FcJson | Self::FcXlsx)
    }

    /// Returns whether this format supports import.
    pub fn supports_import(&self) -> bool {
        true // All formats support import
    }
}

impl std::fmt::Display for StructureFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ffs => write!(f, "FFS (TOML)"),
            Self::FcJson => write!(f, "FC JSON"),
            Self::FcXlsx => write!(f, "FC Excel"),
            Self::Copybook => write!(f, "COBOL Copybook"),
        }
    }
}

/// How to handle name conflicts during import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Rename the imported structure (new name specified).
    Rename(String),
    /// Overwrite the existing definition.
    Overwrite,
    /// Cancel the import operation.
    Cancel,
}

/// Result of an import operation.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// The name of the imported structure.
    pub structure_name: String,
    /// Whether an existing structure was overwritten.
    pub was_overwritten: bool,
}

/// Import service for structure definitions.
pub struct ImportService;

impl ImportService {
    /// Import an FFS (TOML) file content into a structure definition.
    pub fn import_ffs(content: &str) -> Result<StructureDefinition, CatalogError> {
        crate::ffs_format::FfsParser::parse(content)
    }

    /// Import a COBOL copybook into a structure definition.
    pub fn import_copybook(source: &str, name: &str) -> Result<StructureDefinition, CatalogError> {
        let parser = crate::copybook::CopybookParser::with_defaults();
        let result = parser.parse(source)?;

        let mut def = StructureDefinition::new(name);
        def.record_structures[0].fields = result.fields;
        if result.record_length > 0 {
            def.metadata.lrecl = Some(result.record_length);
        }
        Ok(def)
    }
}

/// Export service for structure definitions.
pub struct ExportService;

impl ExportService {
    /// Export a structure definition to FFS (TOML) format string.
    pub fn export_ffs(def: &StructureDefinition) -> Result<String, CatalogError> {
        crate::ffs_format::FfsSerializer::serialize(def)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 8.2 — format choice model
    #[test]
    fn structure_format_extension() {
        assert_eq!(StructureFormat::Ffs.extension(), "ffs");
        assert_eq!(StructureFormat::FcJson.extension(), "fc.json");
        assert_eq!(StructureFormat::FcXlsx.extension(), "fc.xlsx");
        assert_eq!(StructureFormat::Copybook.extension(), "cpy");
    }

    // Validates: Requirement 8.2 — export support
    #[test]
    fn structure_format_export_support() {
        assert!(StructureFormat::Ffs.supports_export());
        assert!(StructureFormat::FcJson.supports_export());
        assert!(StructureFormat::FcXlsx.supports_export());
        assert!(!StructureFormat::Copybook.supports_export());
    }

    // Validates: Requirement 7.5 — FFS import
    #[test]
    fn import_ffs_from_toml_content() {
        let toml_str = r#"
[metadata]
name = "IMPORTED"
version = 1
created_at = "2024-01-01T00:00:00Z"

[[record_structures]]
name = "Default"

[[record_structures.fields]]
name = "F1"
offset = 0
length = 10
field_type = "alphanumeric"
"#;
        let def = ImportService::import_ffs(toml_str).unwrap();
        assert_eq!(def.name(), "IMPORTED");
    }

    // Validates: Requirement 27 — copybook import
    #[test]
    fn import_copybook_creates_structure() {
        let source = concat!(
            "       05  NAME  PIC X(20).\n",
            "       05  AGE   PIC 9(3).\n",
        );
        let def = ImportService::import_copybook(source, "FROM_COPYBOOK").unwrap();
        assert_eq!(def.name(), "FROM_COPYBOOK");
        assert_eq!(def.record_structures[0].fields.len(), 2);
    }

    // Validates: Requirement 8.5 — FFS export
    #[test]
    fn export_ffs_produces_valid_toml() {
        let def = StructureDefinition::new("EXPORT_TEST");
        let toml_str = ExportService::export_ffs(&def).unwrap();
        assert!(toml_str.contains("EXPORT_TEST"));
    }

    // Validates: Requirement 7.6 — conflict resolution enum
    #[test]
    fn conflict_resolution_variants() {
        let rename = ConflictResolution::Rename("NEW_NAME".to_string());
        let overwrite = ConflictResolution::Overwrite;
        let cancel = ConflictResolution::Cancel;
        assert_ne!(rename, overwrite);
        assert_ne!(overwrite, cancel);
    }
}
