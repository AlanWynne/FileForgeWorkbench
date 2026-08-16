//! Field definition types for record structures.
//!
//! A `FieldDefinition` describes a single field within a record structure,
//! specifying byte position, type, and optional classification role.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// How to interpret a field's raw bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DataType {
    /// Text string — decoded per file encoding.
    Str,
    /// Integer numeric value.
    Int,
    /// Floating-point numeric value.
    Float,
    /// Boolean value (T/F/Y/N/1/0/true/false).
    Bool,
    /// IBM packed decimal (COMP-3).
    Comp3,
}

impl DataType {
    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Str => "str",
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::Comp3 => "comp3",
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DataType {
    type Err = String;

    /// Parses a data type string, handling legacy Python repr formats.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "str" | "string" => Ok(Self::Str),
            "int" | "integer" => Ok(Self::Int),
            "float" | "decimal" => Ok(Self::Float),
            "bool" | "boolean" => Ok(Self::Bool),
            "comp3" | "packed" => Ok(Self::Comp3),
            // Legacy Python repr normalisation (Requirement 1.8)
            "<class 'str'>" => Ok(Self::Str),
            "<class 'int'>" => Ok(Self::Int),
            "<class 'float'>" => Ok(Self::Float),
            "<class 'bool'>" => Ok(Self::Bool),
            _ => Err(format!("unknown data type: '{s}'")),
        }
    }
}

impl Serialize for DataType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DataType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Defines a single field within a record structure.
///
/// Fields are byte-addressed: they specify a start offset and length
/// within the raw record bytes. The data_type determines how those
/// bytes are interpreted for display and editing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Non-empty field name, unique within the parent RecordStructure.
    pub field_name: String,
    /// Byte offset from record start (0-based).
    pub offset: usize,
    /// Byte length of this field (must be > 0).
    pub length: usize,
    /// How to interpret the raw bytes.
    pub data_type: DataType,
    /// Number of implied decimal places (for numeric types).
    #[serde(default)]
    pub decimals: u8,
    /// Optional list of identifier values — when a record's bytes at this
    /// field's position match one of these values, the parent RecordStructure
    /// is applied to the record.
    #[serde(default)]
    pub identifiers: Vec<String>,
    /// Optional filter list — when non-empty, only records whose identifier
    /// value appears in this list are displayed or exported.
    #[serde(default)]
    pub filters: Vec<String>,
}

impl FieldDefinition {
    /// Returns the byte range (end-exclusive) covered by this field.
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        self.offset..self.offset + self.length
    }

    /// Returns true if this field overlaps with another field's byte range.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.offset < other.offset + other.length && other.offset < self.offset + self.length
    }

    /// Returns true if this field has identifier values (is a classifier field).
    pub fn is_identifier(&self) -> bool {
        !self.identifiers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.1
    #[test]
    fn field_definition_stores_all_required_attributes() {
        let field = FieldDefinition {
            field_name: "account_id".to_string(),
            offset: 0,
            length: 10,
            data_type: DataType::Str,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        };
        assert_eq!(field.field_name, "account_id");
        assert_eq!(field.offset, 0);
        assert_eq!(field.length, 10);
        assert_eq!(field.data_type, DataType::Str);
        assert_eq!(field.decimals, 0);
    }

    // Validates: Requirement 1.8
    #[test]
    fn data_type_parses_legacy_python_repr() {
        assert_eq!("<class 'str'>".parse::<DataType>().unwrap(), DataType::Str);
        assert_eq!("<class 'int'>".parse::<DataType>().unwrap(), DataType::Int);
        assert_eq!(
            "<class 'float'>".parse::<DataType>().unwrap(),
            DataType::Float
        );
        assert_eq!(
            "<class 'bool'>".parse::<DataType>().unwrap(),
            DataType::Bool
        );
    }

    #[test]
    fn data_type_parses_standard_names() {
        assert_eq!("str".parse::<DataType>().unwrap(), DataType::Str);
        assert_eq!("int".parse::<DataType>().unwrap(), DataType::Int);
        assert_eq!("float".parse::<DataType>().unwrap(), DataType::Float);
        assert_eq!("bool".parse::<DataType>().unwrap(), DataType::Bool);
        assert_eq!("comp3".parse::<DataType>().unwrap(), DataType::Comp3);
    }

    #[test]
    fn data_type_invalid_returns_error() {
        assert!("unknown".parse::<DataType>().is_err());
    }

    // Validates: Requirement 1.2
    #[test]
    fn overlapping_fields_detected() {
        let f1 = FieldDefinition {
            field_name: "a".to_string(),
            offset: 0,
            length: 10,
            data_type: DataType::Str,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        };
        let f2 = FieldDefinition {
            field_name: "b".to_string(),
            offset: 5,
            length: 10,
            data_type: DataType::Int,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        };
        let f3 = FieldDefinition {
            field_name: "c".to_string(),
            offset: 20,
            length: 5,
            data_type: DataType::Str,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        };
        assert!(f1.overlaps(&f2));
        assert!(f2.overlaps(&f1));
        assert!(!f1.overlaps(&f3));
        assert!(!f3.overlaps(&f1));
    }

    #[test]
    fn byte_range_calculation() {
        let field = FieldDefinition {
            field_name: "x".to_string(),
            offset: 10,
            length: 5,
            data_type: DataType::Str,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        };
        assert_eq!(field.byte_range(), 10..15);
    }

    #[test]
    fn serde_roundtrip_data_type() {
        let json = serde_json::to_string(&DataType::Comp3).unwrap();
        assert_eq!(json, "\"comp3\"");
        let parsed: DataType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, DataType::Comp3);
    }
}
