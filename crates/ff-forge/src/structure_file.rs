//! Structure file parsing and serialization.
//!
//! Handles `.ffs` (FileForge Structure) JSON files with legacy compatibility
//! for `.fc.json` format including misspelled keys and Python repr data types.

use serde::{Deserialize, Serialize};

use crate::error::FileForgeError;
use crate::field_def::FieldDefinition;
use crate::record_format::RecordFormat;
use crate::record_structure::RecordStructure;

/// A complete structure definition loaded from a .ffs file.
///
/// Contains file-level metadata and one or more RecordStructures
/// that describe field layouts for different record types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureFile {
    /// Schema version (default "1.0" if absent).
    #[serde(default = "default_version")]
    pub version: String,
    /// Optional logical record length (bytes per record for fixed-width).
    #[serde(default)]
    pub lrecl: Option<usize>,
    /// Optional record format.
    #[serde(default)]
    pub recfm: Option<RecordFormat>,
    /// Optional encoding specification.
    #[serde(default)]
    pub encoding: Option<String>,
    /// Optional field delimiter character.
    /// Accepts legacy misspelled key `field_delimeter` as well.
    #[serde(default, alias = "field_delimeter")]
    pub field_delimiter: Option<String>,
    /// Named record structures (at least one required).
    pub structures: Vec<RecordStructure>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// A non-fatal validation warning reported during structure loading.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
    /// Warning category.
    pub kind: WarningKind,
    /// Human-readable description.
    pub message: String,
}

/// Categories of non-fatal structure validation warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WarningKind {
    /// Two fields have overlapping byte ranges.
    OverlappingFields,
    /// Both recfm VB and lrecl are specified (lrecl ignored).
    VbWithLrecl,
    /// FB_BINARY/VB without explicit encoding — defaulting to EBCDIC-037.
    DefaultingToEbcdic,
    /// Legacy key name normalised (field_delimeter → field_delimiter).
    LegacyKeyNormalised,
    /// Legacy data_type string normalised (Python repr → short form).
    LegacyDataTypeNormalised,
    /// Unknown encoding — defaulting to UTF-8.
    DefaultingToUtf8,
    /// Field has negative offset or zero length.
    InvalidFieldDimensions,
    /// Field has a blank field_name.
    BlankFieldName,
}

/// Parses a structure file from JSON bytes.
///
/// Handles legacy keys (`field_delimeter`) and Python repr data types.
/// Runs structural validation and returns warnings alongside the parsed definition.
///
/// # Errors
///
/// Returns `FileForgeError::StructureParse` if the JSON is invalid or
/// required fields are missing.
pub fn parse_structure_file(
    json_bytes: &[u8],
) -> Result<(StructureFile, Vec<ValidationWarning>), FileForgeError> {
    let raw_json: serde_json::Value = serde_json::from_slice(json_bytes)?;
    let mut warnings = Vec::new();

    // Check for legacy key before serde parsing
    if let Some(obj) = raw_json.as_object() {
        if obj.contains_key("field_delimeter") && !obj.contains_key("field_delimiter") {
            warnings.push(ValidationWarning {
                kind: WarningKind::LegacyKeyNormalised,
                message: "Legacy key 'field_delimeter' normalised to 'field_delimiter'".to_string(),
            });
        }
    }

    // Check for legacy data_type values in structures
    check_legacy_data_types(&raw_json, &mut warnings);

    let structure: StructureFile = serde_json::from_value(raw_json)?;

    // Validate structures
    validate_structure(&structure, &mut warnings);

    Ok((structure, warnings))
}

/// Checks for legacy Python repr data types in the raw JSON.
fn check_legacy_data_types(value: &serde_json::Value, warnings: &mut Vec<ValidationWarning>) {
    if let Some(structures) = value.get("structures").and_then(|s| s.as_array()) {
        for structure in structures {
            if let Some(fields) = structure.get("fields").and_then(|f| f.as_array()) {
                for field in fields {
                    if let Some(dt) = field.get("data_type").and_then(|d| d.as_str()) {
                        if dt.starts_with("<class '") {
                            warnings.push(ValidationWarning {
                                kind: WarningKind::LegacyDataTypeNormalised,
                                message: format!(
                                    "Legacy data_type '{dt}' normalised to standard form"
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Runs structural validation on a parsed structure file.
fn validate_structure(structure: &StructureFile, warnings: &mut Vec<ValidationWarning>) {
    // Check VB + lrecl conflict
    if let Some(RecordFormat::Vb) = structure.recfm {
        if structure.lrecl.is_some() {
            warnings.push(ValidationWarning {
                kind: WarningKind::VbWithLrecl,
                message: "Both recfm 'VB' and lrecl specified — lrecl will be ignored".to_string(),
            });
        }
    }

    // Check encoding defaults for binary formats
    if matches!(
        structure.recfm,
        Some(RecordFormat::FbBinary) | Some(RecordFormat::Vb)
    ) && structure.encoding.is_none()
    {
        warnings.push(ValidationWarning {
            kind: WarningKind::DefaultingToEbcdic,
            message: "No encoding specified for binary format — defaulting to EBCDIC-037"
                .to_string(),
        });
    }

    // Validate each structure's fields
    for rs in &structure.structures {
        // Check for blank field names
        for field in &rs.fields {
            if field.field_name.trim().is_empty() {
                warnings.push(ValidationWarning {
                    kind: WarningKind::BlankFieldName,
                    message: format!(
                        "Structure '{}': field at offset {} has a blank name",
                        rs.name, field.offset
                    ),
                });
            }
            if field.length == 0 {
                warnings.push(ValidationWarning {
                    kind: WarningKind::InvalidFieldDimensions,
                    message: format!(
                        "Structure '{}': field '{}' has zero length",
                        rs.name, field.field_name
                    ),
                });
            }
        }

        // Check for overlapping byte ranges
        for i in 0..rs.fields.len() {
            for j in (i + 1)..rs.fields.len() {
                if rs.fields[i].overlaps(&rs.fields[j]) {
                    warnings.push(ValidationWarning {
                        kind: WarningKind::OverlappingFields,
                        message: format!(
                            "Structure '{}': fields '{}' and '{}' have overlapping byte ranges",
                            rs.name, rs.fields[i].field_name, rs.fields[j].field_name
                        ),
                    });
                }
            }
        }
    }
}

/// Generates a template .ffs structure file with placeholder fields.
pub fn generate_template(base_name: &str) -> String {
    serde_json::to_string_pretty(&StructureFile {
        version: "1.0".to_string(),
        lrecl: Some(80),
        recfm: Some(RecordFormat::Fb),
        encoding: Some("utf-8".to_string()),
        field_delimiter: None,
        structures: vec![RecordStructure {
            name: format!("{base_name}_record"),
            fields: vec![FieldDefinition {
                field_name: "field_1".to_string(),
                offset: 0,
                length: 80,
                data_type: crate::field_def::DataType::Str,
                decimals: 0,
                identifiers: vec![],
                filters: vec![],
            }],
        }],
    })
    .unwrap_or_default()
}

/// Serializes a StructureFile to JSON format.
pub fn serialize_structure(definition: &StructureFile) -> String {
    serde_json::to_string_pretty(definition).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 1.6
    #[test]
    fn parse_defaults_version_to_1_0_when_absent() {
        let json = r#"{"structures": [{"name": "Test", "fields": []}]}"#;
        let (sf, _) = parse_structure_file(json.as_bytes()).unwrap();
        assert_eq!(sf.version, "1.0");
    }

    // Validates: Requirement 1.4
    #[test]
    fn parse_lrecl_and_recfm() {
        let json = r#"{
            "version": "2.0",
            "lrecl": 120,
            "recfm": "FB",
            "structures": [{"name": "Record", "fields": []}]
        }"#;
        let (sf, _) = parse_structure_file(json.as_bytes()).unwrap();
        assert_eq!(sf.version, "2.0");
        assert_eq!(sf.lrecl, Some(120));
        assert_eq!(sf.recfm, Some(RecordFormat::Fb));
    }

    // Validates: Requirement 1.5
    #[test]
    fn parse_encoding_key() {
        let json = r#"{
            "encoding": "ebcdic-037",
            "structures": [{"name": "R", "fields": []}]
        }"#;
        let (sf, _) = parse_structure_file(json.as_bytes()).unwrap();
        assert_eq!(sf.encoding, Some("ebcdic-037".to_string()));
    }

    // Validates: Requirement 1.7
    #[test]
    fn parse_legacy_field_delimeter_spelling() {
        let json = r#"{
            "field_delimeter": "|",
            "structures": [{"name": "R", "fields": []}]
        }"#;
        let (sf, warnings) = parse_structure_file(json.as_bytes()).unwrap();
        assert_eq!(sf.field_delimiter, Some("|".to_string()));
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::LegacyKeyNormalised));
    }

    // Validates: Requirement 1.8
    #[test]
    fn parse_legacy_python_repr_data_types() {
        let json = r#"{
            "structures": [{
                "name": "Legacy",
                "fields": [
                    {"field_name": "name", "offset": 0, "length": 10, "data_type": "<class 'str'>"},
                    {"field_name": "age", "offset": 10, "length": 4, "data_type": "<class 'int'>"}
                ]
            }]
        }"#;
        let (sf, warnings) = parse_structure_file(json.as_bytes()).unwrap();
        assert_eq!(
            sf.structures[0].fields[0].data_type,
            crate::field_def::DataType::Str
        );
        assert_eq!(
            sf.structures[0].fields[1].data_type,
            crate::field_def::DataType::Int
        );
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::LegacyDataTypeNormalised));
    }

    // Validates: Requirement 1.2
    #[test]
    fn validate_overlapping_fields_warns_without_preventing_load() {
        let json = r#"{
            "structures": [{
                "name": "Overlap",
                "fields": [
                    {"field_name": "a", "offset": 0, "length": 10, "data_type": "str"},
                    {"field_name": "b", "offset": 5, "length": 10, "data_type": "str"}
                ]
            }]
        }"#;
        let (sf, warnings) = parse_structure_file(json.as_bytes()).unwrap();
        assert_eq!(sf.structures.len(), 1);
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::OverlappingFields));
    }

    // Validates: Requirement 1.3
    #[test]
    fn parse_multiple_record_structures() {
        let json = r#"{
            "structures": [
                {"name": "Header", "fields": [{"field_name": "type", "offset": 0, "length": 2, "data_type": "str", "identifiers": ["HD"]}]},
                {"name": "Detail", "fields": [{"field_name": "type", "offset": 0, "length": 2, "data_type": "str", "identifiers": ["DT"]}]},
                {"name": "Trailer", "fields": [{"field_name": "type", "offset": 0, "length": 2, "data_type": "str", "identifiers": ["TR"]}]}
            ]
        }"#;
        let (sf, _) = parse_structure_file(json.as_bytes()).unwrap();
        assert_eq!(sf.structures.len(), 3);
        assert_eq!(sf.structures[0].name, "Header");
        assert_eq!(sf.structures[1].name, "Detail");
        assert_eq!(sf.structures[2].name, "Trailer");
    }

    // Validates: Requirement 9.8
    #[test]
    fn validate_zero_length_field_warns() {
        let json = r#"{
            "structures": [{
                "name": "Bad",
                "fields": [{"field_name": "zero", "offset": 0, "length": 0, "data_type": "str"}]
            }]
        }"#;
        let (_, warnings) = parse_structure_file(json.as_bytes()).unwrap();
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::InvalidFieldDimensions));
    }

    #[test]
    fn validate_blank_field_name_warns() {
        let json = r#"{
            "structures": [{
                "name": "Bad",
                "fields": [{"field_name": "", "offset": 0, "length": 10, "data_type": "str"}]
            }]
        }"#;
        let (_, warnings) = parse_structure_file(json.as_bytes()).unwrap();
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::BlankFieldName));
    }

    // Validates: Requirement 6.7
    #[test]
    fn validate_vb_with_lrecl_warns() {
        let json = r#"{
            "recfm": "VB",
            "lrecl": 100,
            "structures": [{"name": "R", "fields": []}]
        }"#;
        let (_, warnings) = parse_structure_file(json.as_bytes()).unwrap();
        assert!(warnings.iter().any(|w| w.kind == WarningKind::VbWithLrecl));
    }

    // Validates: Requirement 4.8
    #[test]
    fn validate_binary_without_encoding_defaults_to_ebcdic() {
        let json = r#"{
            "recfm": "FB_BINARY",
            "lrecl": 100,
            "structures": [{"name": "R", "fields": []}]
        }"#;
        let (_, warnings) = parse_structure_file(json.as_bytes()).unwrap();
        assert!(warnings
            .iter()
            .any(|w| w.kind == WarningKind::DefaultingToEbcdic));
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = parse_structure_file(b"not json");
        assert!(result.is_err());
    }

    #[test]
    fn generate_template_produces_valid_json() {
        let template = generate_template("test_file");
        let parsed: StructureFile = serde_json::from_str(&template).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.structures[0].name, "test_file_record");
    }

    #[test]
    fn serialize_roundtrip_preserves_fields() {
        let json = r#"{
            "version": "1.0",
            "lrecl": 80,
            "recfm": "FB",
            "encoding": "utf-8",
            "structures": [{
                "name": "Main",
                "fields": [{"field_name": "data", "offset": 0, "length": 80, "data_type": "str"}]
            }]
        }"#;
        let (sf, _) = parse_structure_file(json.as_bytes()).unwrap();
        let serialized = serialize_structure(&sf);
        let (sf2, _) = parse_structure_file(serialized.as_bytes()).unwrap();
        assert_eq!(sf, sf2);
    }
}
