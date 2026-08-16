//! Conversion and export engine.
//!
//! Converts structured flat files to modern formats (CSV, TSV, JSON)
//! and fixed-width reconstruction formats (DAT, TXT).

use crate::comp3;
use crate::error::FileForgeError;
use crate::field_def::{DataType, FieldDefinition};
use crate::record_structure::RecordStructure;

/// Supported output formats for flat-file conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Comma-separated values (UTF-8).
    Csv,
    /// Tab-separated values (UTF-8).
    Tsv,
    /// JSON array of record objects (UTF-8).
    Json,
    /// Fixed-width reconstruction (preserves binary layout).
    Dat,
    /// Fixed-width text (preserves layout, newline-terminated).
    Txt,
}

impl OutputFormat {
    /// Parses an output format from a string (case-insensitive).
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(Self::Csv),
            "tsv" => Some(Self::Tsv),
            "json" => Some(Self::Json),
            "dat" => Some(Self::Dat),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }

    /// Returns the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Json => "json",
            Self::Dat => "dat",
            Self::Txt => "txt",
        }
    }

    /// Returns the field delimiter for delimited formats.
    pub fn delimiter(&self) -> Option<char> {
        match self {
            Self::Csv => Some(','),
            Self::Tsv => Some('\t'),
            _ => None,
        }
    }
}

/// Summary of a completed conversion operation.
#[derive(Debug, Clone, PartialEq)]
pub struct ConversionResult {
    /// Total records read from source.
    pub records_read: usize,
    /// Records written to output.
    pub records_written: usize,
    /// Records skipped (unclassified).
    pub records_skipped: usize,
    /// Records excluded by filters.
    pub records_filtered: usize,
}

/// Converts a set of classified records to CSV or TSV format.
///
/// Returns the output as a UTF-8 string.
pub fn convert_to_delimited(
    records: &[&[u8]],
    structure: &RecordStructure,
    delimiter: char,
) -> String {
    let mut output = String::new();

    // Header row
    let headers: Vec<&str> = structure
        .fields
        .iter()
        .map(|f| f.field_name.as_str())
        .collect();
    output.push_str(&headers.join(&delimiter.to_string()));
    output.push('\n');

    // Data rows
    for record in records {
        let values: Vec<String> = structure
            .fields
            .iter()
            .map(|field| extract_field_as_string(record, field))
            .collect();
        output.push_str(&values.join(&delimiter.to_string()));
        output.push('\n');
    }

    output
}

/// Converts a set of classified records to JSON format.
///
/// Returns a JSON array of objects with field names as keys.
pub fn convert_to_json(records: &[&[u8]], structure: &RecordStructure) -> String {
    let mut entries: Vec<String> = Vec::with_capacity(records.len());

    for record in records {
        let mut fields: Vec<String> = Vec::new();
        for field in &structure.fields {
            let value = extract_field_as_string(record, field);
            let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
            fields.push(format!("\"{}\":\"{}\"", field.field_name, escaped));
        }
        entries.push(format!("{{{}}}", fields.join(",")));
    }

    format!("[{}]", entries.join(",\n"))
}

/// Converts records to fixed-width text format (newline-terminated).
pub fn convert_to_txt(records: &[&[u8]]) -> Vec<u8> {
    let mut output = Vec::new();
    for record in records {
        output.extend_from_slice(record);
        output.push(b'\n');
    }
    output
}

/// Extracts a field value from a record as a display string.
///
/// Decodes COMP-3 to decimal, trims strings, formats numbers.
fn extract_field_as_string(record: &[u8], field: &FieldDefinition) -> String {
    if record.len() < field.offset + field.length {
        return String::new();
    }

    let bytes = &record[field.offset..field.offset + field.length];

    match field.data_type {
        DataType::Str | DataType::Bool => String::from_utf8_lossy(bytes).trim_end().to_string(),
        DataType::Int | DataType::Float => String::from_utf8_lossy(bytes).trim().to_string(),
        DataType::Comp3 => match comp3::decode_comp3(bytes) {
            Ok(mut value) => {
                value.decimals = field.decimals;
                comp3::format_comp3(&value)
            }
            Err(_) => {
                // Output hex for invalid COMP-3
                bytes.iter().map(|b| format!("{b:02X}")).collect::<String>()
            }
        },
    }
}

/// Parses an output format string, returning an error for unsupported formats.
pub fn parse_output_format(format_str: &str) -> Result<OutputFormat, FileForgeError> {
    OutputFormat::from_str_opt(format_str).ok_or(FileForgeError::UnsupportedOutputFormat {
        format: format_str.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_def::DataType;

    fn make_structure() -> RecordStructure {
        RecordStructure {
            name: "Test".to_string(),
            fields: vec![
                FieldDefinition {
                    field_name: "name".to_string(),
                    offset: 0,
                    length: 10,
                    data_type: DataType::Str,
                    decimals: 0,
                    identifiers: vec![],
                    filters: vec![],
                },
                FieldDefinition {
                    field_name: "amount".to_string(),
                    offset: 10,
                    length: 8,
                    data_type: DataType::Int,
                    decimals: 0,
                    identifiers: vec![],
                    filters: vec![],
                },
            ],
        }
    }

    // Validates: Requirement 15.2
    #[test]
    fn convert_to_csv_produces_header_and_data() {
        let structure = make_structure();
        let records: Vec<&[u8]> = vec![b"Alice     00001234", b"Bob       00005678"];
        let output = convert_to_delimited(&records, &structure, ',');
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[0], "name,amount");
        assert_eq!(lines[1], "Alice,00001234");
        assert_eq!(lines[2], "Bob,00005678");
    }

    #[test]
    fn convert_to_tsv_uses_tab_delimiter() {
        let structure = make_structure();
        let records: Vec<&[u8]> = vec![b"Alice     00001234"];
        let output = convert_to_delimited(&records, &structure, '\t');
        assert!(output.contains("name\tamount"));
        assert!(output.contains("Alice\t00001234"));
    }

    // Validates: Requirement 15.3
    #[test]
    fn convert_to_json_produces_valid_json() {
        let structure = make_structure();
        let records: Vec<&[u8]> = vec![b"Alice     00001234"];
        let output = convert_to_json(&records, &structure);
        assert!(output.starts_with('['));
        assert!(output.ends_with(']'));
        assert!(output.contains("\"name\":\"Alice\""));
        assert!(output.contains("\"amount\":\"00001234\""));
    }

    // Validates: Requirement 15.4
    #[test]
    fn convert_to_txt_preserves_layout_with_newlines() {
        let records: Vec<&[u8]> = vec![b"Line One  ", b"Line Two  "];
        let output = convert_to_txt(&records);
        assert_eq!(output, b"Line One  \nLine Two  \n");
    }

    // Validates: Requirement 5.8
    #[test]
    fn convert_comp3_to_decimal_string() {
        let structure = RecordStructure {
            name: "Test".to_string(),
            fields: vec![FieldDefinition {
                field_name: "balance".to_string(),
                offset: 0,
                length: 4,
                data_type: DataType::Comp3,
                decimals: 2,
                identifiers: vec![],
                filters: vec![],
            }],
        };
        // X'01234567C' → +1234567 with decimals=2 → 12345.67
        let record: &[u8] = &[0x12, 0x34, 0x56, 0x7C];
        let records: Vec<&[u8]> = vec![record];
        let output = convert_to_delimited(&records, &structure, ',');
        assert!(output.contains("12345.67"));
    }

    #[test]
    fn parse_output_format_valid() {
        assert_eq!(parse_output_format("csv").unwrap(), OutputFormat::Csv);
        assert_eq!(parse_output_format("TSV").unwrap(), OutputFormat::Tsv);
        assert_eq!(parse_output_format("json").unwrap(), OutputFormat::Json);
        assert_eq!(parse_output_format("dat").unwrap(), OutputFormat::Dat);
        assert_eq!(parse_output_format("txt").unwrap(), OutputFormat::Txt);
    }

    #[test]
    fn parse_output_format_invalid_returns_error() {
        let result = parse_output_format("xlsx");
        assert!(matches!(
            result,
            Err(FileForgeError::UnsupportedOutputFormat { .. })
        ));
    }

    #[test]
    fn output_format_extensions() {
        assert_eq!(OutputFormat::Csv.extension(), "csv");
        assert_eq!(OutputFormat::Json.extension(), "json");
    }
}
