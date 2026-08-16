//! Record structure definition.
//!
//! A `RecordStructure` is a named field layout for one category of record
//! in a flat file. Records are classified by matching identifier field values.

use serde::{Deserialize, Serialize};

use crate::field_def::FieldDefinition;

/// A named record structure defining the field layout for one record type.
///
/// Each structure maps to a category of records in the flat file (e.g.,
/// "Header", "Detail", "Trailer"). Records are classified by matching
/// identifier field values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordStructure {
    /// Human-readable name for this record type (e.g., "Detail Record").
    pub name: String,
    /// Ordered list of field definitions within this record layout.
    pub fields: Vec<FieldDefinition>,
}

impl RecordStructure {
    /// Returns the index of the first identifier field, if any.
    pub fn identifier_field_index(&self) -> Option<usize> {
        self.fields.iter().position(|f| f.is_identifier())
    }

    /// Returns the identifier field definition, if any.
    pub fn identifier_field(&self) -> Option<&FieldDefinition> {
        self.fields.iter().find(|f| f.is_identifier())
    }

    /// Returns all identifier values from the identifier field.
    pub fn identifier_values(&self) -> &[String] {
        self.identifier_field()
            .map(|f| f.identifiers.as_slice())
            .unwrap_or(&[])
    }

    /// Returns all filter values from the identifier field.
    pub fn filter_values(&self) -> &[String] {
        self.identifier_field()
            .map(|f| f.filters.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the total byte length required for all fields
    /// (max of offset + length across all fields).
    pub fn required_record_length(&self) -> usize {
        self.fields
            .iter()
            .map(|f| f.offset + f.length)
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_def::DataType;

    // Validates: Requirement 1.1
    #[test]
    fn record_structure_with_ordered_fields() {
        let rs = RecordStructure {
            name: "Detail".to_string(),
            fields: vec![
                FieldDefinition {
                    field_name: "rec_type".to_string(),
                    offset: 0,
                    length: 2,
                    data_type: DataType::Str,
                    decimals: 0,
                    identifiers: vec!["DT".to_string()],
                    filters: vec![],
                },
                FieldDefinition {
                    field_name: "amount".to_string(),
                    offset: 2,
                    length: 10,
                    data_type: DataType::Int,
                    decimals: 0,
                    identifiers: vec![],
                    filters: vec![],
                },
            ],
        };
        assert_eq!(rs.name, "Detail");
        assert_eq!(rs.fields.len(), 2);
        assert_eq!(rs.identifier_field_index(), Some(0));
        assert_eq!(rs.identifier_values(), &["DT"]);
        assert_eq!(rs.required_record_length(), 12);
    }

    #[test]
    fn record_structure_no_identifier_field() {
        let rs = RecordStructure {
            name: "Simple".to_string(),
            fields: vec![FieldDefinition {
                field_name: "data".to_string(),
                offset: 0,
                length: 80,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec![],
                filters: vec![],
            }],
        };
        assert_eq!(rs.identifier_field_index(), None);
        assert_eq!(rs.identifier_values(), &[] as &[String]);
    }

    #[test]
    fn required_record_length_empty_fields() {
        let rs = RecordStructure {
            name: "Empty".to_string(),
            fields: vec![],
        };
        assert_eq!(rs.required_record_length(), 0);
    }
}
