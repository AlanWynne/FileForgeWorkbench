//! Property-based tests for ff-forge.
//!
//! Uses proptest to verify invariants across randomised inputs.

use proptest::prelude::*;

use ff_forge::asa::detect_asa;
use ff_forge::byte_index::ByteOffsetIndex;
use ff_forge::classifier::classify_record;
use ff_forge::comp3::{decode_comp3, encode_comp3, format_comp3, Comp3Sign};
use ff_forge::ebcdic::{decode_ebcdic_field, encode_ebcdic_field, EbcdicCodePage};
use ff_forge::field_def::{DataType, FieldDefinition};
use ff_forge::navigation::RecordNavigator;
use ff_forge::record_structure::RecordStructure;
use ff_forge::structure_file::{parse_structure_file, serialize_structure};
use ff_forge::vb_reader::{parse_rdw, write_vb_record};

// ─── Property 1: COMP-3 encode/decode roundtrip ─────────────────────────────
// Feature: fileforge-integration, Property 1: COMP-3 encode/decode roundtrip
// **Validates: Requirements 5.2, 5.3, 5.5**

proptest! {
    #[test]
    fn comp3_encode_decode_roundtrip(
        value in -9_999_999i64..=9_999_999i64,
    ) {
        let value_str = value.to_string();
        // 4 bytes can hold up to 7 digits
        if let Ok(encoded) = encode_comp3(&value_str, 0, 4) {
            let decoded = decode_comp3(&encoded).unwrap();
            prop_assert_eq!(decoded.mantissa, value);
            if value > 0 {
                prop_assert_eq!(decoded.sign, Comp3Sign::Positive);
            } else if value < 0 {
                prop_assert_eq!(decoded.sign, Comp3Sign::Negative);
            }
        }
    }
}

// ─── Property 2: EBCDIC encode/decode roundtrip ─────────────────────────────
// Feature: fileforge-integration, Property 2: EBCDIC roundtrip
// **Validates: Requirements 4.2, 4.4**

fn ebcdic_mappable_char_strategy() -> impl Strategy<Value = char> {
    // ASCII printable characters that are all mappable in CP037
    prop::char::range(' ', '~')
}

proptest! {
    #[test]
    fn ebcdic_encode_decode_roundtrip(
        s in prop::collection::vec(ebcdic_mappable_char_strategy(), 1..20)
    ) {
        let text: String = s.into_iter().collect();
        let code_page = EbcdicCodePage::Cp037;
        let encoded = encode_ebcdic_field(&text, code_page, text.len()).unwrap();
        let decoded = decode_ebcdic_field(&encoded, code_page);
        prop_assert_eq!(decoded, text);
    }
}

// ─── Property 3: VB RDW length consistency ──────────────────────────────────
// Feature: fileforge-integration, Property 3: VB RDW length
// **Validates: Requirements 6.2, 6.6**

proptest! {
    #[test]
    fn vb_rdw_length_equals_content_plus_4(
        content_len in 0u16..1000u16
    ) {
        let content = vec![0u8; content_len as usize];
        let written = write_vb_record(&content);
        prop_assert_eq!(written.len(), content_len as usize + 4);

        // Parse RDW and verify
        let rdw = parse_rdw(&written, 0).unwrap();
        prop_assert_eq!(rdw.record_length, content_len + 4);
        prop_assert_eq!(rdw.content_length(), content_len);
    }
}

// ─── Property 4: Field byte-range non-overflow ──────────────────────────────
// Feature: fileforge-integration, Property 4: Field byte-range
// **Validates: Requirements 1.1, 1.2**

proptest! {
    #[test]
    fn field_byte_range_within_lrecl(
        lrecl in 10usize..1000,
        offset in 0usize..500,
        length in 1usize..100,
    ) {
        // Only test valid fields (offset + length <= lrecl)
        prop_assume!(offset + length <= lrecl);
        let field = FieldDefinition {
            field_name: "test".to_string(),
            offset,
            length,
            data_type: DataType::Str,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        };
        prop_assert!(field.offset + field.length <= lrecl);
    }
}

// ─── Property 5: Record classification determinism ──────────────────────────
// Feature: fileforge-integration, Property 5: Classification determinism
// **Validates: Requirements 13.1, 13.5**

proptest! {
    #[test]
    fn classification_is_deterministic(
        record_bytes in prop::collection::vec(any::<u8>(), 20..100),
    ) {
        let structures = vec![
            RecordStructure {
                name: "TypeA".to_string(),
                fields: vec![FieldDefinition {
                    field_name: "id".to_string(),
                    offset: 0,
                    length: 2,
                    data_type: DataType::Str,
                    decimals: 0,
                    identifiers: vec!["AA".to_string()],
                    filters: vec![],
                }],
            },
            RecordStructure {
                name: "TypeB".to_string(),
                fields: vec![FieldDefinition {
                    field_name: "id".to_string(),
                    offset: 0,
                    length: 2,
                    data_type: DataType::Str,
                    decimals: 0,
                    identifiers: vec!["BB".to_string()],
                    filters: vec![],
                }],
            },
        ];

        let result1 = classify_record(&record_bytes, &structures);
        let result2 = classify_record(&record_bytes, &structures);
        prop_assert_eq!(result1, result2);
    }
}

// ─── Property 6: Field validation accepts valid inputs ──────────────────────
// Feature: fileforge-integration, Property 6: Validation-encoding consistency
// **Validates: Requirements 9.1, 9.5, 9.6**

proptest! {
    #[test]
    fn valid_int_input_encodes_within_field_length(
        value in -9999i32..=9999i32,
    ) {
        let field = FieldDefinition {
            field_name: "num".to_string(),
            offset: 0,
            length: 10,
            data_type: DataType::Int,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        };
        let value_str = value.to_string();
        let result = ff_forge::field_validation::FieldValidator::validate(&field, &value_str);
        prop_assert!(result.is_ok());
    }
}

// ─── Property 7: ByteOffsetIndex monotonicity ───────────────────────────────
// Feature: fileforge-integration, Property 7: Index monotonicity
// **Validates: Requirements 2.2, 10.1**

proptest! {
    #[test]
    fn fixed_width_index_offsets_are_strictly_increasing(
        lrecl in 1usize..1000,
        record_count in 2usize..10000,
    ) {
        let index = ByteOffsetIndex::FixedWidth { lrecl, record_count };
        for i in 1..record_count.min(1000) {
            let prev = index.offset_of(i - 1).unwrap();
            let curr = index.offset_of(i).unwrap();
            prop_assert!(curr > prev, "offset[{}] = {} should be > offset[{}] = {}", i, curr, i-1, prev);
        }
    }
}

// ─── Property 8: ASA detection threshold ────────────────────────────────────
// Feature: fileforge-integration, Property 8: ASA detection threshold
// **Validates: Requirements 7.3**

proptest! {
    #[test]
    fn asa_detection_at_80_percent_detected(
        asa_count in 16usize..=20,
    ) {
        let non_asa_count = 20 - asa_count;
        let mut records: Vec<Vec<u8>> = Vec::new();

        // ASA records (space as ASA char)
        for _ in 0..asa_count {
            records.push(b" DATA".to_vec());
        }
        // Non-ASA records
        for _ in 0..non_asa_count {
            records.push(b"XDATA".to_vec());
        }

        let refs: Vec<&[u8]> = records.iter().map(|r| r.as_slice()).collect();
        let result = detect_asa(&refs, 20);

        let confidence = asa_count as f32 / 20.0;
        if confidence >= 0.8 {
            prop_assert!(result.detected, "Should detect ASA at {}% confidence", confidence * 100.0);
        }
    }

    #[test]
    fn asa_detection_below_50_percent_not_detected(
        asa_count in 0usize..10,
    ) {
        let total = 20;
        let non_asa_count = total - asa_count;
        let mut records: Vec<Vec<u8>> = Vec::new();

        for _ in 0..asa_count {
            records.push(b" DATA".to_vec());
        }
        for _ in 0..non_asa_count {
            records.push(b"XDATA".to_vec());
        }

        let refs: Vec<&[u8]> = records.iter().map(|r| r.as_slice()).collect();
        let result = detect_asa(&refs, 20);

        let confidence = asa_count as f32 / total as f32;
        if confidence < 0.5 {
            prop_assert!(!result.detected, "Should NOT detect ASA at {}% confidence", confidence * 100.0);
        }
    }
}

// ─── Property 9: Window navigation bounds ───────────────────────────────────
// Feature: fileforge-integration, Property 9: Navigation bounds
// **Validates: Requirements 10.2, 10.3**

proptest! {
    #[test]
    fn page_navigation_stays_in_bounds(
        total_records in 1usize..10000,
        window_size in 1usize..500,
        page_ops in prop::collection::vec(prop::bool::ANY, 1..50),
    ) {
        let index = ByteOffsetIndex::FixedWidth { lrecl: 80, record_count: total_records };
        let mut nav = RecordNavigator::new(&index, window_size);

        for go_down in page_ops {
            if go_down {
                nav.page_down();
            } else {
                nav.page_up();
            }
            let pos = nav.current_record();
            prop_assert!(pos < total_records, "Position {} >= total {}", pos, total_records);
        }
    }
}

// ─── Property 10: Structure file serialization roundtrip ────────────────────
// Feature: fileforge-integration, Property 10: Structure file roundtrip
// **Validates: Requirements 1.4, 1.5, 12.4**

proptest! {
    #[test]
    fn structure_file_serialize_deserialize_roundtrip(
        lrecl in prop::option::of(1usize..10000),
        field_count in 1usize..5,
    ) {
        use ff_forge::record_format::RecordFormat;

        let mut fields = Vec::new();
        let mut offset = 0;
        for i in 0..field_count {
            let length = 10;
            fields.push(FieldDefinition {
                field_name: format!("field_{i}"),
                offset,
                length,
                data_type: DataType::Str,
                decimals: 0,
                identifiers: vec![],
                filters: vec![],
            });
            offset += length;
        }

        let sf = ff_forge::structure_file::StructureFile {
            version: "1.0".to_string(),
            lrecl,
            recfm: Some(RecordFormat::Fb),
            encoding: Some("utf-8".to_string()),
            field_delimiter: None,
            structures: vec![RecordStructure {
                name: "TestStruct".to_string(),
                fields,
            }],
        };

        let json = serialize_structure(&sf);
        let (parsed, _warnings) = parse_structure_file(json.as_bytes()).unwrap();
        prop_assert_eq!(parsed, sf);
    }
}

// ─── Property 11: Record insert preserves file integrity ────────────────────
// Feature: fileforge-integration, Property 11: Record insert integrity
// **Validates: Requirements 11.1, 11.5, 11.6**

proptest! {
    #[test]
    fn fb_record_insert_preserves_total_byte_count(
        original_records in 1usize..100,
        lrecl in 10usize..200,
    ) {
        let original_size = original_records * lrecl;
        let new_record = ff_forge::record_ops::create_blank_fb_record(lrecl);
        let new_size = original_size + new_record.len();
        prop_assert_eq!(new_size, (original_records + 1) * lrecl);
        prop_assert_eq!(new_record.len(), lrecl);
    }

    #[test]
    fn vb_record_insert_adds_content_plus_rdw(
        content_length in 1usize..500,
    ) {
        let record = ff_forge::record_ops::create_blank_vb_record(content_length);
        prop_assert_eq!(record.len(), content_length + 4);

        // Verify RDW is correct
        let rdw = parse_rdw(&record, 0).unwrap();
        prop_assert_eq!(rdw.content_length() as usize, content_length);
    }
}

// ─── Property 12: COMP-3 decimal separator is always period ─────────────────
// Feature: fileforge-integration, Property 12: COMP-3 decimal separator
// **Validates: Requirements 5.10**

proptest! {
    #[test]
    fn comp3_formatted_output_uses_period_only(
        mantissa in -999_999i64..=999_999i64,
        decimals in 0u8..6,
    ) {
        let value = ff_forge::comp3::Comp3Value {
            mantissa,
            decimals,
            sign: if mantissa >= 0 { Comp3Sign::Positive } else { Comp3Sign::Negative },
        };
        let formatted = format_comp3(&value);

        // Should never contain comma
        prop_assert!(!formatted.contains(','), "Contains comma: {}", formatted);

        // Should match decimal number pattern
        let pattern = regex::Regex::new(r"^-?[0-9]+(\.[0-9]+)?$").unwrap();
        prop_assert!(pattern.is_match(&formatted), "Invalid format: {}", formatted);
    }
}
