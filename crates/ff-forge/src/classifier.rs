//! Record classification engine.
//!
//! Evaluates identifier fields against record bytes to determine which
//! RecordStructure applies to each record. Uses first-match-wins semantics.

use crate::record_structure::RecordStructure;

/// Result of classifying a record against available structures.
#[derive(Debug, Clone, PartialEq)]
pub enum RecordClassification {
    /// Record matched a named structure.
    Matched {
        /// Name of the matching structure.
        structure_name: String,
        /// Index into the structures array.
        structure_index: usize,
    },
    /// Record matched a structure but was excluded by a filter list.
    Filtered {
        /// Name of the structure that matched.
        structure_name: String,
    },
    /// Record matched no structure.
    Unclassified,
}

/// Classification statistics for a file session.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClassificationStats {
    /// Total records processed.
    pub total_records: usize,
    /// Records per structure type (structure_name → count).
    pub records_per_type: std::collections::HashMap<String, usize>,
    /// Records that matched no structure.
    pub records_unclassified: usize,
    /// Records excluded by filter lists.
    pub records_filtered: usize,
}

/// Classifies a single record against a list of record structures.
///
/// Uses first-match-wins semantics: iterates structures in definition order
/// and returns the first whose identifier field matches the record bytes.
///
/// When a structure has no identifier field, it acts as a catch-all (matches
/// any record).
pub fn classify_record(record: &[u8], structures: &[RecordStructure]) -> RecordClassification {
    for (index, structure) in structures.iter().enumerate() {
        if let Some(id_field) = structure.identifier_field() {
            // Extract identifier value from record bytes
            if record.len() < id_field.offset + id_field.length {
                continue; // Record too short for this structure's identifier
            }

            let id_bytes = &record[id_field.offset..id_field.offset + id_field.length];
            // Trim trailing spaces for comparison
            let id_value = String::from_utf8_lossy(id_bytes);
            let id_trimmed = id_value.trim_end();

            // Check if record's identifier value matches any in the identifiers list
            let matches = id_field
                .identifiers
                .iter()
                .any(|expected| expected.trim() == id_trimmed);

            if matches {
                // Check filter list
                if !id_field.filters.is_empty() {
                    let passes_filter = id_field.filters.iter().any(|f| f.trim() == id_trimmed);
                    if !passes_filter {
                        return RecordClassification::Filtered {
                            structure_name: structure.name.clone(),
                        };
                    }
                }

                return RecordClassification::Matched {
                    structure_name: structure.name.clone(),
                    structure_index: index,
                };
            }
        } else {
            // No identifier field — catch-all structure
            return RecordClassification::Matched {
                structure_name: structure.name.clone(),
                structure_index: index,
            };
        }
    }

    RecordClassification::Unclassified
}

/// Classifies a batch of records and produces statistics.
pub fn classify_batch(
    records: &[&[u8]],
    structures: &[RecordStructure],
) -> (Vec<RecordClassification>, ClassificationStats) {
    let mut stats = ClassificationStats {
        total_records: records.len(),
        ..Default::default()
    };
    let mut classifications = Vec::with_capacity(records.len());

    for record in records {
        let classification = classify_record(record, structures);

        match &classification {
            RecordClassification::Matched { structure_name, .. } => {
                *stats
                    .records_per_type
                    .entry(structure_name.clone())
                    .or_insert(0) += 1;
            }
            RecordClassification::Filtered { .. } => {
                stats.records_filtered += 1;
            }
            RecordClassification::Unclassified => {
                stats.records_unclassified += 1;
            }
        }

        classifications.push(classification);
    }

    (classifications, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_def::{DataType, FieldDefinition};

    fn make_structure(name: &str, identifiers: Vec<&str>) -> RecordStructure {
        RecordStructure {
            name: name.to_string(),
            fields: vec![FieldDefinition {
                field_name: "type".to_string(),
                offset: 0,
                length: 2,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: identifiers.iter().map(|s| s.to_string()).collect(),
                filters: vec![],
            }],
        }
    }

    // Validates: Requirement 13.1
    #[test]
    fn classify_single_type_match() {
        let structures = vec![make_structure("Header", vec!["HD"])];
        let record = b"HD Some header data here";

        let result = classify_record(record, &structures);
        assert_eq!(
            result,
            RecordClassification::Matched {
                structure_name: "Header".to_string(),
                structure_index: 0,
            }
        );
    }

    // Validates: Requirement 13.2
    #[test]
    fn classify_first_match_wins() {
        let structures = vec![
            make_structure("Header", vec!["HD"]),
            make_structure("Detail", vec!["DT"]),
            make_structure("Trailer", vec!["TR"]),
        ];
        let record = b"DT Detail record content";

        let result = classify_record(record, &structures);
        assert_eq!(
            result,
            RecordClassification::Matched {
                structure_name: "Detail".to_string(),
                structure_index: 1,
            }
        );
    }

    // Validates: Requirement 13.3
    #[test]
    fn classify_unclassified_when_no_match() {
        let structures = vec![
            make_structure("Header", vec!["HD"]),
            make_structure("Detail", vec!["DT"]),
        ];
        let record = b"XX Unknown record type";

        let result = classify_record(record, &structures);
        assert_eq!(result, RecordClassification::Unclassified);
    }

    // Validates: Requirement 13.4
    #[test]
    fn classify_filtered_when_excluded_by_filter() {
        let structures = vec![RecordStructure {
            name: "Detail".to_string(),
            fields: vec![FieldDefinition {
                field_name: "type".to_string(),
                offset: 0,
                length: 2,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec!["DT".to_string()],
                filters: vec!["XX".to_string()], // Only show XX, not DT
            }],
        }];
        let record = b"DT Some detail data";

        let result = classify_record(record, &structures);
        assert_eq!(
            result,
            RecordClassification::Filtered {
                structure_name: "Detail".to_string(),
            }
        );
    }

    // Validates: Requirement 13.5
    #[test]
    fn classify_deterministic_same_result_twice() {
        let structures = vec![
            make_structure("Header", vec!["HD"]),
            make_structure("Detail", vec!["DT"]),
        ];
        let record = b"DT Some data";

        let result1 = classify_record(record, &structures);
        let result2 = classify_record(record, &structures);
        assert_eq!(result1, result2);
    }

    #[test]
    fn classify_catch_all_structure_no_identifiers() {
        let structures = vec![RecordStructure {
            name: "CatchAll".to_string(),
            fields: vec![FieldDefinition {
                field_name: "data".to_string(),
                offset: 0,
                length: 80,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec![],
                filters: vec![],
            }],
        }];
        let record = b"Anything goes here";

        let result = classify_record(record, &structures);
        assert_eq!(
            result,
            RecordClassification::Matched {
                structure_name: "CatchAll".to_string(),
                structure_index: 0,
            }
        );
    }

    #[test]
    fn classify_batch_produces_statistics() {
        let structures = vec![
            make_structure("Header", vec!["HD"]),
            make_structure("Detail", vec!["DT"]),
        ];
        let records: Vec<&[u8]> = vec![b"HD Header", b"DT Detail1", b"DT Detail2", b"XX Unknown"];

        let (classifications, stats) = classify_batch(&records, &structures);

        assert_eq!(classifications.len(), 4);
        assert_eq!(stats.total_records, 4);
        assert_eq!(stats.records_per_type["Header"], 1);
        assert_eq!(stats.records_per_type["Detail"], 2);
        assert_eq!(stats.records_unclassified, 1);
    }

    #[test]
    fn classify_record_too_short_for_identifier_skips_structure() {
        let structures = vec![RecordStructure {
            name: "Long".to_string(),
            fields: vec![FieldDefinition {
                field_name: "id".to_string(),
                offset: 10,
                length: 5,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec!["MATCH".to_string()],
                filters: vec![],
            }],
        }];
        // Record is only 5 bytes — too short for offset 10 + length 5
        let record = b"SHORT";

        let result = classify_record(record, &structures);
        assert_eq!(result, RecordClassification::Unclassified);
    }
}
