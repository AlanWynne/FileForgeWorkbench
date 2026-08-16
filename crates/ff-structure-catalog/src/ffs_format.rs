//! FFS file format — TOML serialization and deserialization.
//!
//! Provides [`FfsParser`] for reading `.ffs` files and [`FfsSerializer`] for
//! writing them. The `.ffs` format is a TOML v1.0 document with a defined schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CatalogError;
use crate::field::{FieldDefinition, FieldType};
use crate::model::{
    FileAssociations, RecordFormat, RecordStructure, StructureDefinition, StructureMetadata,
};

// ─── TOML intermediate representation ──────────────────────────────────────

/// Top-level FFS file structure for TOML serialization.
#[derive(Debug, Serialize, Deserialize)]
struct FfsDocument {
    metadata: FfsMetadata,
    #[serde(default)]
    associations: Option<FfsAssociations>,
    #[serde(default, rename = "record_structures")]
    record_structures: Vec<FfsRecordStructure>,
}

/// Metadata table in FFS format.
#[derive(Debug, Serialize, Deserialize)]
struct FfsMetadata {
    name: String,
    #[serde(default)]
    description: Option<String>,
    version: u32,
    created_at: String,
    #[serde(default)]
    modified_at: Option<String>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    lrecl: Option<u32>,
    #[serde(default)]
    recfm: Option<String>,
}

/// Associations table in FFS format.
#[derive(Debug, Serialize, Deserialize)]
struct FfsAssociations {
    #[serde(default)]
    file_patterns: Vec<String>,
}

/// Record structure entry in FFS format.
#[derive(Debug, Serialize, Deserialize)]
struct FfsRecordStructure {
    name: String,
    #[serde(default)]
    fields: Vec<FfsField>,
}

/// Field entry in FFS format.
#[derive(Debug, Serialize, Deserialize)]
struct FfsField {
    name: String,
    offset: u32,
    length: u32,
    field_type: String,
    #[serde(default)]
    decimals: u8,
    #[serde(default)]
    identifiers: Vec<String>,
    #[serde(default)]
    filters: Vec<String>,
}

// ─── Parser ─────────────────────────────────────────────────────────────────

/// Parser for the `.ffs` TOML format.
///
/// Converts between raw TOML text and [`StructureDefinition`] instances.
pub struct FfsParser;

impl FfsParser {
    /// Parse a `.ffs` file content string into a [`StructureDefinition`].
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError::ParseError`] if the TOML is syntactically invalid.
    /// Returns [`CatalogError::SchemaError`] if required keys are missing or values are invalid.
    pub fn parse(content: &str) -> Result<StructureDefinition, CatalogError> {
        Self::parse_with_path(content, "<unknown>")
    }

    /// Parse with a known file path for better error messages.
    pub fn parse_with_path(content: &str, path: &str) -> Result<StructureDefinition, CatalogError> {
        let doc: FfsDocument = toml::from_str(content).map_err(|e| CatalogError::ParseError {
            path: path.to_string(),
            detail: e.to_string(),
        })?;

        Self::validate_document(&doc, path)?;
        Self::convert_to_definition(doc, path)
    }

    /// Validate the parsed TOML document against the FFS schema.
    fn validate_document(doc: &FfsDocument, path: &str) -> Result<(), CatalogError> {
        // metadata.name must be non-empty
        if doc.metadata.name.trim().is_empty() {
            return Err(CatalogError::SchemaError {
                path: path.to_string(),
                detail: "metadata.name must be non-empty".to_string(),
            });
        }

        // metadata.version must be positive
        if doc.metadata.version == 0 {
            return Err(CatalogError::SchemaError {
                path: path.to_string(),
                detail: "metadata.version must be a positive integer".to_string(),
            });
        }

        // Must have at least one record structure
        if doc.record_structures.is_empty() {
            return Err(CatalogError::SchemaError {
                path: path.to_string(),
                detail: "at least one [[record_structures]] entry is required".to_string(),
            });
        }

        // Validate each field in each record structure
        for rs in &doc.record_structures {
            for field in &rs.fields {
                // Validate field_type is a known value
                if field.field_type.parse::<FieldType>().is_err() {
                    return Err(CatalogError::SchemaError {
                        path: path.to_string(),
                        detail: format!(
                            "invalid field_type '{}' for field '{}' in record structure '{}'",
                            field.field_type, field.name, rs.name
                        ),
                    });
                }

                // Validate length >= 1
                if field.length == 0 {
                    return Err(CatalogError::SchemaError {
                        path: path.to_string(),
                        detail: format!(
                            "field '{}' in record structure '{}' has length 0 (must be >= 1)",
                            field.name, rs.name
                        ),
                    });
                }
            }
        }

        // Validate recfm if present
        if let Some(ref recfm) = doc.metadata.recfm {
            if recfm.parse::<RecordFormat>().is_err() {
                return Err(CatalogError::SchemaError {
                    path: path.to_string(),
                    detail: format!("invalid recfm value: '{recfm}'"),
                });
            }
        }

        Ok(())
    }

    /// Convert the intermediate TOML document to a domain model.
    fn convert_to_definition(
        doc: FfsDocument,
        path: &str,
    ) -> Result<StructureDefinition, CatalogError> {
        let created_at = parse_datetime(&doc.metadata.created_at, path, "metadata.created_at")?;
        let modified_at = doc
            .metadata
            .modified_at
            .as_deref()
            .map(|s| parse_datetime(s, path, "metadata.modified_at"))
            .transpose()?;

        let recfm = doc
            .metadata
            .recfm
            .as_deref()
            .map(|s| {
                s.parse::<RecordFormat>()
                    .map_err(|_| CatalogError::SchemaError {
                        path: path.to_string(),
                        detail: format!("invalid recfm value: '{s}'"),
                    })
            })
            .transpose()?;

        let metadata = StructureMetadata {
            name: doc.metadata.name,
            description: doc.metadata.description,
            version: doc.metadata.version,
            created_at,
            modified_at,
            encoding: doc.metadata.encoding,
            lrecl: doc.metadata.lrecl,
            recfm,
        };

        let associations = doc.associations.map(|a| FileAssociations {
            file_patterns: a.file_patterns,
        });

        let record_structures: Vec<RecordStructure> = doc
            .record_structures
            .into_iter()
            .map(|rs| RecordStructure {
                name: rs.name,
                fields: rs
                    .fields
                    .into_iter()
                    .map(|f| FieldDefinition {
                        name: f.name,
                        offset: f.offset,
                        length: f.length,
                        field_type: f.field_type.parse().unwrap_or_default(),
                        decimals: f.decimals,
                        identifiers: f.identifiers,
                        filters: f.filters,
                    })
                    .collect(),
            })
            .collect();

        Ok(StructureDefinition {
            metadata,
            associations,
            record_structures,
        })
    }
}

// ─── Serializer ─────────────────────────────────────────────────────────────

/// Serializer for the `.ffs` TOML format.
pub struct FfsSerializer;

impl FfsSerializer {
    /// Serialize a [`StructureDefinition`] to a TOML `.ffs` format string.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] if serialization fails (unlikely for valid definitions).
    pub fn serialize(def: &StructureDefinition) -> Result<String, CatalogError> {
        let doc = Self::convert_to_document(def);
        toml::to_string_pretty(&doc).map_err(|e| CatalogError::ExportError {
            format: "ffs".to_string(),
            path: String::new(),
            detail: e.to_string(),
        })
    }

    /// Convert a domain model to the intermediate TOML document.
    fn convert_to_document(def: &StructureDefinition) -> FfsDocument {
        let metadata = FfsMetadata {
            name: def.metadata.name.clone(),
            description: def.metadata.description.clone(),
            version: def.metadata.version,
            created_at: def.metadata.created_at.to_rfc3339(),
            modified_at: def.metadata.modified_at.map(|dt| dt.to_rfc3339()),
            encoding: def.metadata.encoding.clone(),
            lrecl: def.metadata.lrecl,
            recfm: def.metadata.recfm.map(|r| r.to_string()),
        };

        let associations = def.associations.as_ref().map(|a| FfsAssociations {
            file_patterns: a.file_patterns.clone(),
        });

        let record_structures: Vec<FfsRecordStructure> = def
            .record_structures
            .iter()
            .map(|rs| FfsRecordStructure {
                name: rs.name.clone(),
                fields: rs
                    .fields
                    .iter()
                    .map(|f| FfsField {
                        name: f.name.clone(),
                        offset: f.offset,
                        length: f.length,
                        field_type: f.field_type.to_string(),
                        decimals: f.decimals,
                        identifiers: f.identifiers.clone(),
                        filters: f.filters.clone(),
                    })
                    .collect(),
            })
            .collect();

        FfsDocument {
            metadata,
            associations,
            record_structures,
        }
    }
}

/// Parse an ISO 8601 datetime string into a `DateTime<Utc>`.
fn parse_datetime(s: &str, path: &str, field: &str) -> Result<DateTime<Utc>, CatalogError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| CatalogError::SchemaError {
            path: path.to_string(),
            detail: format!("{field} is not a valid ISO 8601 datetime: {e}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sample_definition() -> StructureDefinition {
        use chrono::TimeZone;

        let created = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        StructureDefinition {
            metadata: StructureMetadata {
                name: "CUSTOMER".to_string(),
                description: Some("Customer master record".to_string()),
                version: 3,
                created_at: created,
                modified_at: Some(Utc.with_ymd_and_hms(2024, 6, 1, 14, 0, 0).unwrap()),
                encoding: Some("utf-8".to_string()),
                lrecl: Some(200),
                recfm: Some(RecordFormat::Fb),
            },
            associations: Some(FileAssociations {
                file_patterns: vec!["CUST_*.dat".to_string(), "*.cust".to_string()],
            }),
            record_structures: vec![RecordStructure {
                name: "Detail".to_string(),
                fields: vec![
                    FieldDefinition::new("CUST_NAME", 0, 30, FieldType::Alphanumeric),
                    FieldDefinition {
                        name: "BALANCE".to_string(),
                        offset: 30,
                        length: 6,
                        field_type: FieldType::PackedDecimal,
                        decimals: 2,
                        identifiers: vec![],
                        filters: vec![],
                    },
                ],
            }],
        }
    }

    // Validates: Requirement 2.1 — FFS serialization produces valid TOML
    #[test]
    fn serialize_produces_valid_toml() {
        let def = sample_definition();
        let toml_str = FfsSerializer::serialize(&def).unwrap();
        // Verify it's parseable TOML
        let _: toml::Value = toml::from_str(&toml_str).unwrap();
    }

    // Validates: Requirement 2.1 — FFS round-trip preserves structure
    #[test]
    fn serialize_deserialize_round_trip() {
        let def = sample_definition();
        let toml_str = FfsSerializer::serialize(&def).unwrap();
        let parsed = FfsParser::parse(&toml_str).unwrap();
        assert_eq!(def, parsed);
    }

    // Validates: Requirement 2.1 — metadata table serialized
    #[test]
    fn serialize_contains_metadata_keys() {
        let def = sample_definition();
        let toml_str = FfsSerializer::serialize(&def).unwrap();
        assert!(toml_str.contains("[metadata]"));
        assert!(toml_str.contains("name = \"CUSTOMER\""));
        assert!(toml_str.contains("version = 3"));
    }

    // Validates: Requirement 2.7 — encoding key serialized
    #[test]
    fn serialize_contains_encoding() {
        let def = sample_definition();
        let toml_str = FfsSerializer::serialize(&def).unwrap();
        assert!(toml_str.contains("encoding = \"utf-8\""));
    }

    // Validates: Requirement 2.8 — lrecl key serialized
    #[test]
    fn serialize_contains_lrecl() {
        let def = sample_definition();
        let toml_str = FfsSerializer::serialize(&def).unwrap();
        assert!(toml_str.contains("lrecl = 200"));
    }

    // Validates: Requirement 2.9 — recfm key serialized
    #[test]
    fn serialize_contains_recfm() {
        let def = sample_definition();
        let toml_str = FfsSerializer::serialize(&def).unwrap();
        assert!(toml_str.contains("recfm = \"FB\""));
    }

    // Validates: Requirement 2.1 — associations table serialized
    #[test]
    fn serialize_contains_associations() {
        let def = sample_definition();
        let toml_str = FfsSerializer::serialize(&def).unwrap();
        assert!(toml_str.contains("[associations]"));
        assert!(toml_str.contains("CUST_*.dat"));
    }

    // Validates: Requirement 2.5 — invalid TOML syntax rejected
    #[test]
    fn parse_rejects_invalid_toml() {
        let result = FfsParser::parse_with_path("invalid [[[toml", "test.ffs");
        assert!(matches!(result, Err(CatalogError::ParseError { .. })));
    }

    // Validates: Requirement 2.6 — missing metadata.name rejected
    #[test]
    fn parse_rejects_empty_name() {
        let toml_str = r#"
[metadata]
name = ""
version = 1
created_at = "2024-01-01T00:00:00Z"

[[record_structures]]
name = "Default"
fields = []
"#;
        let result = FfsParser::parse_with_path(toml_str, "test.ffs");
        assert!(matches!(result, Err(CatalogError::SchemaError { .. })));
    }

    // Validates: Requirement 2.3 — version must be positive
    #[test]
    fn parse_rejects_zero_version() {
        let toml_str = r#"
[metadata]
name = "TEST"
version = 0
created_at = "2024-01-01T00:00:00Z"

[[record_structures]]
name = "Default"
fields = []
"#;
        let result = FfsParser::parse_with_path(toml_str, "test.ffs");
        assert!(matches!(result, Err(CatalogError::SchemaError { .. })));
    }

    // Validates: Requirement 2.6 — no record structures rejected
    #[test]
    fn parse_rejects_no_record_structures() {
        let toml_str = r#"
[metadata]
name = "TEST"
version = 1
created_at = "2024-01-01T00:00:00Z"
"#;
        let result = FfsParser::parse_with_path(toml_str, "test.ffs");
        assert!(matches!(result, Err(CatalogError::SchemaError { .. })));
    }

    // Validates: Requirement 2.6 — invalid field_type rejected
    #[test]
    fn parse_rejects_invalid_field_type() {
        let toml_str = r#"
[metadata]
name = "TEST"
version = 1
created_at = "2024-01-01T00:00:00Z"

[[record_structures]]
name = "Default"

[[record_structures.fields]]
name = "FIELD1"
offset = 0
length = 10
field_type = "invalid_type"
"#;
        let result = FfsParser::parse_with_path(toml_str, "test.ffs");
        assert!(matches!(result, Err(CatalogError::SchemaError { .. })));
    }

    // Validates: Requirement 2.6 — zero length field rejected
    #[test]
    fn parse_rejects_zero_length_field() {
        let toml_str = r#"
[metadata]
name = "TEST"
version = 1
created_at = "2024-01-01T00:00:00Z"

[[record_structures]]
name = "Default"

[[record_structures.fields]]
name = "FIELD1"
offset = 0
length = 0
field_type = "alphanumeric"
"#;
        let result = FfsParser::parse_with_path(toml_str, "test.ffs");
        assert!(matches!(result, Err(CatalogError::SchemaError { .. })));
    }

    // Validates: Requirement 2.1 — valid FFS file parses correctly
    #[test]
    fn parse_valid_ffs_file() {
        let toml_str = r#"
[metadata]
name = "CUSTOMER"
description = "Customer record"
version = 1
created_at = "2024-01-15T10:30:00Z"
encoding = "utf-8"
lrecl = 100
recfm = "FB"

[associations]
file_patterns = ["*.dat"]

[[record_structures]]
name = "Detail"

[[record_structures.fields]]
name = "NAME"
offset = 0
length = 30
field_type = "alphanumeric"

[[record_structures.fields]]
name = "AMOUNT"
offset = 30
length = 6
field_type = "packed-decimal"
decimals = 2
"#;
        let def = FfsParser::parse(toml_str).unwrap();
        assert_eq!(def.metadata.name, "CUSTOMER");
        assert_eq!(def.metadata.version, 1);
        assert_eq!(def.metadata.encoding.as_deref(), Some("utf-8"));
        assert_eq!(def.metadata.lrecl, Some(100));
        assert_eq!(def.metadata.recfm, Some(RecordFormat::Fb));
        assert_eq!(def.record_structures.len(), 1);
        assert_eq!(def.record_structures[0].fields.len(), 2);
        assert_eq!(def.record_structures[0].fields[1].decimals, 2);
    }

    // Validates: Requirement 2.2 — field_type values parsed correctly
    #[test]
    fn parse_all_field_types() {
        let toml_str = r#"
[metadata]
name = "TYPES_TEST"
version = 1
created_at = "2024-01-01T00:00:00Z"

[[record_structures]]
name = "Default"

[[record_structures.fields]]
name = "F1"
offset = 0
length = 10
field_type = "alphanumeric"

[[record_structures.fields]]
name = "F2"
offset = 10
length = 5
field_type = "numeric"

[[record_structures.fields]]
name = "F3"
offset = 15
length = 6
field_type = "packed-decimal"

[[record_structures.fields]]
name = "F4"
offset = 21
length = 4
field_type = "binary"

[[record_structures.fields]]
name = "F5"
offset = 25
length = 8
field_type = "hex"
"#;
        let def = FfsParser::parse(toml_str).unwrap();
        let fields = &def.record_structures[0].fields;
        assert_eq!(fields[0].field_type, FieldType::Alphanumeric);
        assert_eq!(fields[1].field_type, FieldType::Numeric);
        assert_eq!(fields[2].field_type, FieldType::PackedDecimal);
        assert_eq!(fields[3].field_type, FieldType::Binary);
        assert_eq!(fields[4].field_type, FieldType::Hex);
    }
}
