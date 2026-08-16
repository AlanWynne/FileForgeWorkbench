//! Core data models for structure definitions.
//!
//! Provides [`StructureDefinition`], [`StructureMetadata`], [`RecordStructure`],
//! [`RecordFormat`], and [`FileAssociations`] — the in-memory representation of
//! a single `.ffs` catalog entry.

use chrono::{DateTime, Utc};

use crate::field::FieldDefinition;

/// Record format enumeration for mainframe-origin files.
///
/// Indicates the expected record format of data files using this structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum RecordFormat {
    /// Fixed-length records.
    F,
    /// Fixed-length blocked records.
    #[serde(rename = "FB")]
    Fb,
    /// Variable-length records.
    V,
    /// Fixed-length blocked binary records.
    #[serde(rename = "FB_BINARY")]
    FbBinary,
    /// Variable-length blocked records.
    #[serde(rename = "VB")]
    Vb,
    /// Undefined record format.
    U,
}

impl std::fmt::Display for RecordFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F => write!(f, "F"),
            Self::Fb => write!(f, "FB"),
            Self::V => write!(f, "V"),
            Self::FbBinary => write!(f, "FB_BINARY"),
            Self::Vb => write!(f, "VB"),
            Self::U => write!(f, "U"),
        }
    }
}

impl std::str::FromStr for RecordFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "F" => Ok(Self::F),
            "FB" => Ok(Self::Fb),
            "V" => Ok(Self::V),
            "FB_BINARY" => Ok(Self::FbBinary),
            "VB" => Ok(Self::Vb),
            "U" => Ok(Self::U),
            other => Err(format!("invalid record format: {other}")),
        }
    }
}

/// Structure metadata from the `[metadata]` TOML table.
///
/// Contains identification, versioning, and optional encoding information
/// for a structure definition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StructureMetadata {
    /// Unique name within a catalog location.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Monotonically increasing version number (starts at 1).
    pub version: u32,
    /// ISO 8601 datetime of creation.
    pub created_at: DateTime<Utc>,
    /// ISO 8601 datetime of last modification.
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
    /// Expected character encoding of associated data files (optional).
    #[serde(default)]
    pub encoding: Option<String>,
    /// Expected logical record length (optional).
    #[serde(default)]
    pub lrecl: Option<u32>,
    /// Expected record format (optional).
    #[serde(default)]
    pub recfm: Option<RecordFormat>,
}

impl StructureMetadata {
    /// Create minimal metadata for a new structure definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            version: 1,
            created_at: Utc::now(),
            modified_at: None,
            encoding: None,
            lrecl: None,
            recfm: None,
        }
    }
}

/// File association patterns for auto-matching.
///
/// Glob patterns that identify data files associated with this structure.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct FileAssociations {
    /// Glob patterns that match filenames (e.g., `"*.dat"`, `"CUST_*.dat"`).
    #[serde(default)]
    pub file_patterns: Vec<String>,
}

/// A named record layout within a structure definition.
///
/// A structure definition may contain multiple record structures
/// (e.g., Header, Detail, Trailer records in a multi-format file).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecordStructure {
    /// Name of this record structure (e.g., "Header", "Detail").
    pub name: String,
    /// Ordered list of field definitions.
    pub fields: Vec<FieldDefinition>,
}

impl RecordStructure {
    /// Create a new record structure with the given name and no fields.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }

    /// Create a new record structure with the given name and fields.
    pub fn with_fields(name: impl Into<String>, fields: Vec<FieldDefinition>) -> Self {
        Self {
            name: name.into(),
            fields,
        }
    }

    /// Return the total number of fields.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

/// A complete structure definition loaded from a single `.ffs` file.
///
/// Contains metadata, one or more record structures, and optional
/// file association patterns.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StructureDefinition {
    /// Structure metadata (name, version, timestamps, encoding, etc.).
    pub metadata: StructureMetadata,
    /// Optional file pattern associations for auto-matching.
    #[serde(default)]
    pub associations: Option<FileAssociations>,
    /// Ordered list of record structures (e.g., Header, Detail, Trailer).
    pub record_structures: Vec<RecordStructure>,
}

impl StructureDefinition {
    /// Create a new structure definition with the given name and a single empty record structure.
    pub fn new(name: impl Into<String>) -> Self {
        let name_str: String = name.into();
        Self {
            metadata: StructureMetadata::new(&name_str),
            associations: None,
            record_structures: vec![RecordStructure::new("Default")],
        }
    }

    /// Return the structure name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Return the total number of fields across all record structures.
    pub fn total_field_count(&self) -> usize {
        self.record_structures
            .iter()
            .map(|rs| rs.field_count())
            .sum()
    }

    /// Return the total number of record structures.
    pub fn record_structure_count(&self) -> usize {
        self.record_structures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::FieldType;

    // Validates: Requirement 2.1 — StructureDefinition construction
    #[test]
    fn structure_definition_new_creates_with_defaults() {
        let def = StructureDefinition::new("CUSTOMER");
        assert_eq!(def.name(), "CUSTOMER");
        assert_eq!(def.metadata.version, 1);
        assert!(def.associations.is_none());
        assert_eq!(def.record_structures.len(), 1);
        assert_eq!(def.record_structures[0].name, "Default");
    }

    // Validates: Requirement 9.1, 9.3 — version and created_at
    #[test]
    fn metadata_new_sets_version_1_and_timestamp() {
        let meta = StructureMetadata::new("TEST");
        assert_eq!(meta.version, 1);
        assert!(meta.created_at <= Utc::now());
        assert!(meta.modified_at.is_none());
    }

    // Validates: Requirement 2.1 — RecordStructure with fields
    #[test]
    fn record_structure_with_fields() {
        let fields = vec![
            FieldDefinition::new("NAME", 0, 30, FieldType::Alphanumeric),
            FieldDefinition::new("AMOUNT", 30, 8, FieldType::PackedDecimal),
        ];
        let rs = RecordStructure::with_fields("Detail", fields);
        assert_eq!(rs.name, "Detail");
        assert_eq!(rs.field_count(), 2);
    }

    // Validates: Requirement 2.1 — total field count
    #[test]
    fn structure_definition_total_field_count() {
        let mut def = StructureDefinition::new("INVOICE");
        def.record_structures[0].fields.push(FieldDefinition::new(
            "F1",
            0,
            10,
            FieldType::Alphanumeric,
        ));
        def.record_structures.push(RecordStructure::with_fields(
            "Trailer",
            vec![FieldDefinition::new("F2", 0, 5, FieldType::Numeric)],
        ));
        assert_eq!(def.total_field_count(), 2);
        assert_eq!(def.record_structure_count(), 2);
    }

    // Validates: Requirement 2.9 — RecordFormat parsing
    #[test]
    fn record_format_from_str() {
        assert_eq!("F".parse::<RecordFormat>().unwrap(), RecordFormat::F);
        assert_eq!("FB".parse::<RecordFormat>().unwrap(), RecordFormat::Fb);
        assert_eq!("V".parse::<RecordFormat>().unwrap(), RecordFormat::V);
        assert_eq!(
            "FB_BINARY".parse::<RecordFormat>().unwrap(),
            RecordFormat::FbBinary
        );
        assert_eq!("VB".parse::<RecordFormat>().unwrap(), RecordFormat::Vb);
        assert_eq!("U".parse::<RecordFormat>().unwrap(), RecordFormat::U);
        assert!("INVALID".parse::<RecordFormat>().is_err());
    }

    // Validates: Requirement 2.9 — RecordFormat display
    #[test]
    fn record_format_display() {
        assert_eq!(RecordFormat::F.to_string(), "F");
        assert_eq!(RecordFormat::Fb.to_string(), "FB");
        assert_eq!(RecordFormat::FbBinary.to_string(), "FB_BINARY");
    }

    // Validates: Requirement 2.1 — FileAssociations default
    #[test]
    fn file_associations_default_is_empty() {
        let assoc = FileAssociations::default();
        assert!(assoc.file_patterns.is_empty());
    }

    // Validates: Requirement 2 — Clone and PartialEq
    #[test]
    fn structure_definition_clone_eq() {
        let def = StructureDefinition::new("CLONE_TEST");
        let cloned = def.clone();
        assert_eq!(def, cloned);
    }
}
