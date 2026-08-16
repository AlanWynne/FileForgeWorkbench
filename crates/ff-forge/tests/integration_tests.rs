//! Integration tests for ff-forge.
//!
//! End-to-end tests that exercise multiple modules working together.

use ff_forge::asa::{detect_asa, strip_asa};
use ff_forge::byte_index::ByteOffsetIndex;
use ff_forge::classifier::{classify_batch, RecordClassification};
use ff_forge::comp3::{decode_comp3, encode_comp3, format_comp3};
use ff_forge::convert::{convert_to_delimited, convert_to_json};
use ff_forge::ebcdic::{decode_ebcdic_field, encode_ebcdic_field, EbcdicCodePage};
use ff_forge::fb_reader::{build_fixed_index, detect_lrecl, read_fb_record, LreclDetection};
use ff_forge::field_def::{DataType, FieldDefinition};
use ff_forge::navigation::RecordNavigator;
use ff_forge::record_format::RecordFormat;
use ff_forge::record_structure::RecordStructure;
use ff_forge::structure_file::{parse_structure_file, WarningKind};
use ff_forge::vb_reader::build_vb_index;
use ff_forge::window::load_fixed_window;
use ff_forge::FileForgeError;

// ─── Structure Parse Integration Tests ──────────────────────────────────────

#[test]
fn parse_complete_ffs_file() {
    // Validates: Requirement 1
    let json = r#"{
        "version": "1.0",
        "lrecl": 80,
        "recfm": "FB",
        "encoding": "ebcdic-037",
        "structures": [
            {
                "name": "Header",
                "fields": [
                    {"field_name": "rec_type", "offset": 0, "length": 2, "data_type": "str", "identifiers": ["HD"]},
                    {"field_name": "date", "offset": 2, "length": 8, "data_type": "str"}
                ]
            },
            {
                "name": "Detail",
                "fields": [
                    {"field_name": "rec_type", "offset": 0, "length": 2, "data_type": "str", "identifiers": ["DT"]},
                    {"field_name": "amount", "offset": 2, "length": 4, "data_type": "comp3", "decimals": 2},
                    {"field_name": "name", "offset": 6, "length": 30, "data_type": "str"}
                ]
            },
            {
                "name": "Trailer",
                "fields": [
                    {"field_name": "rec_type", "offset": 0, "length": 2, "data_type": "str", "identifiers": ["TR"]},
                    {"field_name": "count", "offset": 2, "length": 6, "data_type": "int"}
                ]
            }
        ]
    }"#;

    let (sf, warnings) = parse_structure_file(json.as_bytes()).unwrap();
    assert_eq!(sf.version, "1.0");
    assert_eq!(sf.lrecl, Some(80));
    assert_eq!(sf.recfm, Some(RecordFormat::Fb));
    assert_eq!(sf.encoding, Some("ebcdic-037".to_string()));
    assert_eq!(sf.structures.len(), 3);
    assert_eq!(sf.structures[0].name, "Header");
    assert_eq!(sf.structures[1].name, "Detail");
    assert_eq!(sf.structures[2].name, "Trailer");
    // Should warn about binary format + encoding already specified → no defaulting warning
    assert!(warnings
        .iter()
        .all(|w| w.kind != WarningKind::DefaultingToEbcdic));
}

#[test]
fn parse_legacy_fc_json_format() {
    // Validates: Requirement 1.7, 1.8
    let json = r#"{
        "field_delimeter": "|",
        "structures": [{
            "name": "Legacy",
            "fields": [
                {"field_name": "name", "offset": 0, "length": 20, "data_type": "<class 'str'>"},
                {"field_name": "age", "offset": 20, "length": 3, "data_type": "<class 'int'>"},
                {"field_name": "rate", "offset": 23, "length": 8, "data_type": "<class 'float'>"},
                {"field_name": "active", "offset": 31, "length": 1, "data_type": "<class 'bool'>"}
            ]
        }]
    }"#;

    let (sf, warnings) = parse_structure_file(json.as_bytes()).unwrap();
    assert_eq!(sf.field_delimiter, Some("|".to_string()));
    assert_eq!(sf.structures[0].fields[0].data_type, DataType::Str);
    assert_eq!(sf.structures[0].fields[1].data_type, DataType::Int);
    assert_eq!(sf.structures[0].fields[2].data_type, DataType::Float);
    assert_eq!(sf.structures[0].fields[3].data_type, DataType::Bool);
    assert!(warnings
        .iter()
        .any(|w| w.kind == WarningKind::LegacyKeyNormalised));
    assert!(warnings
        .iter()
        .any(|w| w.kind == WarningKind::LegacyDataTypeNormalised));
}

// ─── FB Session Integration Tests ───────────────────────────────────────────

#[test]
fn fb_file_navigation_and_field_extraction() {
    // Validates: Requirement 2, 10
    let lrecl = 20;
    let mut data = Vec::new();
    for i in 0..100 {
        data.extend_from_slice(format!("R{i:03}            ").as_bytes());
        // Pad to exactly 20 bytes
        while data.len() % lrecl != 0 {
            data.push(b' ');
        }
    }
    // Truncate to exact 100 records
    data.truncate(100 * lrecl);

    // Build index
    let index = build_fixed_index(data.len() as u64, lrecl).unwrap();
    assert_eq!(index.record_count(), 100);

    // Navigate to record 50
    let mut nav = RecordNavigator::new(&index, 20);
    nav.go_to_record(50).unwrap();
    assert_eq!(nav.current_record(), 49);

    // Read record
    let record = read_fb_record(&data, 49, lrecl).unwrap();
    assert_eq!(&record[0..4], b"R049");
}

// ─── VB Session Integration Tests ───────────────────────────────────────────

#[test]
fn vb_file_read_and_index() {
    // Validates: Requirement 6
    let mut data = Vec::new();
    let records = vec![
        b"Record One".to_vec(),
        b"Record Two Data".to_vec(),
        b"Three".to_vec(),
    ];

    for record in &records {
        let rdw_len = (record.len() as u16) + 4;
        data.extend_from_slice(&rdw_len.to_be_bytes());
        data.extend_from_slice(&[0x00, 0x00]);
        data.extend_from_slice(record);
    }

    let offsets = build_vb_index(&data).unwrap();
    assert_eq!(offsets.len(), 3);

    // Verify content at each offset
    for (i, &offset) in offsets.iter().enumerate() {
        let start = offset as usize;
        let end = start + records[i].len();
        assert_eq!(&data[start..end], records[i].as_slice());
    }
}

// ─── EBCDIC Session Integration Tests ───────────────────────────────────────

#[test]
fn ebcdic_field_decode_and_edit_roundtrip() {
    // Validates: Requirement 4
    let code_page = EbcdicCodePage::Cp037;
    let original_text = "Hello World";

    // Encode to EBCDIC
    let encoded = encode_ebcdic_field(original_text, code_page, 20).unwrap();
    assert_eq!(encoded.len(), 20);

    // Decode back
    let decoded = decode_ebcdic_field(&encoded, code_page);
    assert_eq!(decoded.trim(), original_text);

    // Edit: replace with new value
    let new_text = "New Value";
    let new_encoded = encode_ebcdic_field(new_text, code_page, 20).unwrap();
    let new_decoded = decode_ebcdic_field(&new_encoded, code_page);
    assert_eq!(new_decoded.trim(), new_text);
}

// ─── COMP-3 Session Integration Tests ───────────────────────────────────────

#[test]
fn comp3_full_pipeline_decode_display_edit_reencode() {
    // Validates: Requirement 5
    // Original bytes: +12345.67 (mantissa 1234567, decimals 2)
    let original = encode_comp3("1234567", 0, 4).unwrap();

    // Decode
    let decoded = decode_comp3(&original).unwrap();
    assert_eq!(decoded.mantissa, 1234567);

    // Display with decimals
    let mut display_value = decoded.clone();
    display_value.decimals = 2;
    let formatted = format_comp3(&display_value);
    assert_eq!(formatted, "12345.67");

    // Edit: change to -99.99 (mantissa 9999, decimals 2)
    let new_encoded = encode_comp3("-9999", 0, 4).unwrap();
    let new_decoded = decode_comp3(&new_encoded).unwrap();
    assert_eq!(new_decoded.mantissa, -9999);

    let mut new_display = new_decoded.clone();
    new_display.decimals = 2;
    assert_eq!(format_comp3(&new_display), "-99.99");
}

// ─── ASA Session Integration Tests ──────────────────────────────────────────

#[test]
fn asa_detection_and_strip_pipeline() {
    // Validates: Requirement 7
    let records: Vec<Vec<u8>> = vec![
        b"1NEW PAGE HEADING".to_vec(),
        b" DETAIL LINE ONE".to_vec(),
        b" DETAIL LINE TWO".to_vec(),
        b"0DOUBLE SPACED".to_vec(),
        b"-TRIPLE SPACED".to_vec(),
        b"+OVERPRINT".to_vec(),
    ];

    // Detection
    let refs: Vec<&[u8]> = records.iter().map(|r| r.as_slice()).collect();
    let detection = detect_asa(&refs, 20);
    assert!(detection.detected);
    assert_eq!(detection.confidence, 1.0);

    // Strip ASA
    let stripped = strip_asa(&records);
    assert_eq!(stripped[0], b"NEW PAGE HEADING");
    assert_eq!(stripped[1], b"DETAIL LINE ONE");
    assert_eq!(stripped[3], b"DOUBLE SPACED");
}

// ─── Multi-Type Classification Tests ────────────────────────────────────────

#[test]
fn multi_type_file_classification_and_filtering() {
    // Validates: Requirement 13, 14
    let structures = vec![
        RecordStructure {
            name: "Header".to_string(),
            fields: vec![FieldDefinition {
                field_name: "type".to_string(),
                offset: 0,
                length: 2,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec!["HD".to_string()],
                filters: vec![],
            }],
        },
        RecordStructure {
            name: "Detail".to_string(),
            fields: vec![FieldDefinition {
                field_name: "type".to_string(),
                offset: 0,
                length: 2,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec!["DT".to_string()],
                filters: vec![],
            }],
        },
        RecordStructure {
            name: "Trailer".to_string(),
            fields: vec![FieldDefinition {
                field_name: "type".to_string(),
                offset: 0,
                length: 2,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec!["TR".to_string()],
                filters: vec![],
            }],
        },
    ];

    let records: Vec<&[u8]> = vec![
        b"HD20240101        ",
        b"DT001 Alice       ",
        b"DT002 Bob         ",
        b"DT003 Carol       ",
        b"TR003             ",
    ];

    let (classifications, stats) = classify_batch(&records, &structures);

    assert_eq!(stats.total_records, 5);
    assert_eq!(stats.records_per_type["Header"], 1);
    assert_eq!(stats.records_per_type["Detail"], 3);
    assert_eq!(stats.records_per_type["Trailer"], 1);
    assert_eq!(stats.records_unclassified, 0);

    assert_eq!(
        classifications[0],
        RecordClassification::Matched {
            structure_name: "Header".to_string(),
            structure_index: 0,
        }
    );
    assert_eq!(
        classifications[3],
        RecordClassification::Matched {
            structure_name: "Detail".to_string(),
            structure_index: 1,
        }
    );
}

// ─── Conversion Integration Tests ───────────────────────────────────────────

#[test]
fn convert_fb_file_to_csv() {
    // Validates: Requirement 15
    let structure = RecordStructure {
        name: "Record".to_string(),
        fields: vec![
            FieldDefinition {
                field_name: "id".to_string(),
                offset: 0,
                length: 5,
                data_type: DataType::Int,
                decimals: 0,
                identifiers: vec![],
                filters: vec![],
            },
            FieldDefinition {
                field_name: "name".to_string(),
                offset: 5,
                length: 15,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec![],
                filters: vec![],
            },
        ],
    };

    let records: Vec<&[u8]> = vec![b"00001Alice          ", b"00002Bob            "];

    let csv_output = convert_to_delimited(&records, &structure, ',');
    let lines: Vec<&str> = csv_output.lines().collect();
    assert_eq!(lines[0], "id,name");
    assert_eq!(lines[1], "00001,Alice");
    assert_eq!(lines[2], "00002,Bob");
}

#[test]
fn convert_comp3_fields_to_json() {
    // Validates: Requirement 5.8
    let structure = RecordStructure {
        name: "Financial".to_string(),
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

    let encoded = encode_comp3("123456", 0, 4).unwrap();
    let records: Vec<&[u8]> = vec![&encoded];

    let json_output = convert_to_json(&records, &structure);
    assert!(json_output.contains("\"balance\":\"1234.56\""));
}

// ─── Error Resilience Tests ─────────────────────────────────────────────────

#[test]
fn error_on_invalid_structure_json() {
    // Validates: Requirement 16
    let result = parse_structure_file(b"not valid json at all");
    assert!(result.is_err());
}

#[test]
fn error_on_empty_structures_array_still_parses() {
    let json = r#"{"structures": []}"#;
    let (sf, _) = parse_structure_file(json.as_bytes()).unwrap();
    assert!(sf.structures.is_empty());
}

#[test]
fn vb_error_recovery_reports_records_read() {
    // Validates: Requirement 16, 6.3
    let mut data = Vec::new();
    // Valid record
    data.extend_from_slice(&[0x00, 0x08, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]);
    // Another valid record
    data.extend_from_slice(&[0x00, 0x06, 0x00, 0x00, 0x05, 0x06]);
    // Invalid RDW (length < 4)
    data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]);

    let result = build_vb_index(&data);
    assert!(result.is_err());
    let (error, records_read) = result.unwrap_err();
    assert_eq!(records_read, 2); // Two good records before error
    assert!(matches!(error, FileForgeError::InvalidRdw { .. }));
}

#[test]
fn lrecl_detection_with_mixed_line_lengths() {
    // Validates: Requirement 2.11
    let data = b"SHORT\nMUCH LONGER LINE HERE\nMED\n";
    let result = detect_lrecl(data, 100);
    assert_eq!(result, LreclDetection::Variable);
}

// ─── Window and Navigation Integration ──────────────────────────────────────

#[test]
fn window_load_and_navigate_through_file() {
    // Validates: Requirement 2.7, 2.8, 10
    let lrecl = 10;
    let record_count = 500;
    let data: Vec<u8> = (0..record_count)
        .flat_map(|i| format!("REC{i:05}  ").into_bytes()[..lrecl].to_vec())
        .collect();

    let index = ByteOffsetIndex::for_fixed_width(data.len() as u64, lrecl);
    let mut nav = RecordNavigator::new(&index, 50);

    // Load first window
    let window = load_fixed_window(&data, lrecl, 0, 50);
    assert_eq!(window.len(), 50);
    assert_eq!(window.start_index, 0);

    // Navigate to page 2
    nav.page_down();
    let window2 = load_fixed_window(&data, lrecl, nav.current_record(), 50);
    assert_eq!(window2.start_index, 50);
    assert_eq!(window2.len(), 50);

    // Records don't overlap between windows
    assert_eq!(window.end_index(), window2.start_index);
}
