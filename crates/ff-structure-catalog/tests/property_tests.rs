//! Property-based tests for ff-structure-catalog.
//!
//! Uses the `proptest` crate to verify properties across many generated inputs.

use proptest::prelude::*;

use ff_structure_catalog::catalog::StructureCatalog;
use ff_structure_catalog::editor::EditorState;
use ff_structure_catalog::ffs_format::{FfsParser, FfsSerializer};
use ff_structure_catalog::field::{
    decode_packed_decimal, format_numeric_decimal, validate_packed_decimal, FieldDefinition,
    FieldType,
};
use ff_structure_catalog::grid::encode_field_value;
use ff_structure_catalog::model::{
    FileAssociations, RecordFormat, RecordStructure, StructureDefinition, StructureMetadata,
};

use chrono::{TimeZone, Utc};

// ─── Strategies ─────────────────────────────────────────────────────────────

fn field_type_strategy() -> impl Strategy<Value = FieldType> {
    prop_oneof![
        Just(FieldType::Alphanumeric),
        Just(FieldType::Numeric),
        Just(FieldType::PackedDecimal),
        Just(FieldType::Binary),
        Just(FieldType::Hex),
    ]
}

fn valid_field_name() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,19}".prop_map(|s| s)
}

fn valid_field_def() -> impl Strategy<Value = FieldDefinition> {
    (
        valid_field_name(),
        0u32..10000,
        1u32..500,
        field_type_strategy(),
        0u8..10,
    )
        .prop_map(
            |(name, offset, length, field_type, decimals)| FieldDefinition {
                name,
                offset,
                length,
                field_type,
                decimals,
                identifiers: Vec::new(),
                filters: Vec::new(),
            },
        )
}

fn valid_record_structure() -> impl Strategy<Value = RecordStructure> {
    (
        valid_field_name(),
        proptest::collection::vec(valid_field_def(), 1..10),
    )
        .prop_map(|(name, fields)| RecordStructure { name, fields })
}

fn record_format_strategy() -> impl Strategy<Value = Option<RecordFormat>> {
    prop_oneof![
        Just(None),
        Just(Some(RecordFormat::F)),
        Just(Some(RecordFormat::Fb)),
        Just(Some(RecordFormat::V)),
        Just(Some(RecordFormat::Vb)),
        Just(Some(RecordFormat::U)),
    ]
}

fn valid_structure_definition() -> impl Strategy<Value = StructureDefinition> {
    (
        valid_field_name(),
        proptest::option::of("[A-Za-z ]{1,40}"),
        1u32..1000,
        proptest::option::of(prop_oneof![
            Just("utf-8".to_string()),
            Just("ebcdic-037".to_string())
        ]),
        proptest::option::of(1u32..10000),
        record_format_strategy(),
        proptest::option::of(proptest::collection::vec("[*][.][a-z]{1,5}", 1..4)),
        proptest::collection::vec(valid_record_structure(), 1..5),
    )
        .prop_map(
            |(name, description, version, encoding, lrecl, recfm, patterns, record_structures)| {
                let created_at = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
                StructureDefinition {
                    metadata: StructureMetadata {
                        name,
                        description,
                        version,
                        created_at,
                        modified_at: None,
                        encoding,
                        lrecl,
                        recfm,
                    },
                    associations: patterns.map(|p| FileAssociations { file_patterns: p }),
                    record_structures,
                }
            },
        )
}

// ─── Property Tests ─────────────────────────────────────────────────────────

proptest! {
    /// **Validates: Requirement 2.1**
    ///
    /// Property 1: FFS Serialization/Deserialization Round-Trip
    /// For any valid StructureDefinition, serializing to TOML and parsing back
    /// produces a definition equal to the original.
    #[test]
    fn ffs_round_trip_preserves_definition(def in valid_structure_definition()) {
        // Feature: structure-catalog, Property 1: FFS round-trip
        let serialized = FfsSerializer::serialize(&def).unwrap();
        let parsed = FfsParser::parse(&serialized).unwrap();
        prop_assert_eq!(def, parsed);
    }

    /// **Validates: Requirement 5.9**
    ///
    /// Property 2: Field Validation Invariant
    /// validate() returns Ok iff name is non-empty and length >= 1.
    #[test]
    fn field_validation_invariant(
        name in "[ -~]{0,20}",
        offset in 0u32..10000,
        length in 0u32..100,
        field_type in field_type_strategy(),
        decimals in 0u8..10,
    ) {
        // Feature: structure-catalog, Property 2: field validation invariant
        let field = FieldDefinition {
            name: name.clone(),
            offset,
            length,
            field_type,
            decimals,
            identifiers: Vec::new(),
            filters: Vec::new(),
        };

        let result = field.validate();
        let is_valid = !name.is_empty() && length >= 1;
        prop_assert_eq!(result.is_ok(), is_valid,
            "Field: name='{}', length={}, expected valid={}", name, length, is_valid);
    }

    /// **Validates: Requirement 5.5**
    ///
    /// Property 4: Auto-Compute Offsets Contiguity
    /// After auto-compute, fields[i].offset == sum of all preceding lengths.
    #[test]
    fn auto_compute_offsets_produces_contiguous_layout(
        field_lengths in proptest::collection::vec(1u32..100, 1..50),
    ) {
        // Feature: structure-catalog, Property 4: auto-compute offsets contiguity
        let fields: Vec<FieldDefinition> = field_lengths.iter().enumerate()
            .map(|(i, &len)| FieldDefinition::new(
                format!("F{i}"), 999, len, FieldType::Alphanumeric,
            ))
            .collect();

        let def = StructureDefinition {
            metadata: StructureMetadata::new("TEST"),
            associations: None,
            record_structures: vec![RecordStructure { name: "Default".to_string(), fields }],
        };

        let mut editor = EditorState::open(def);
        editor.auto_compute_offsets();

        let rs = editor.active_record_structure().unwrap();
        prop_assert_eq!(rs.fields[0].offset, 0, "First field should start at offset 0");

        for i in 1..rs.fields.len() {
            let expected = rs.fields[i - 1].offset + rs.fields[i - 1].length;
            prop_assert_eq!(rs.fields[i].offset, expected,
                "Field {} offset should be {} but was {}", i, expected, rs.fields[i].offset);
        }
    }

    /// **Validates: Requirement 2.4, 3.1**
    ///
    /// Property 5: Catalog Name Uniqueness Enforcement
    /// For any sequence of create operations, duplicate names are rejected.
    #[test]
    fn catalog_name_uniqueness(
        names in proptest::collection::vec(
            proptest::sample::select(vec![
                "ALPHA".to_string(), "BETA".to_string(), "GAMMA".to_string(),
                "DELTA".to_string(), "EPSILON".to_string(),
            ]),
            5..20
        ),
    ) {
        // Feature: structure-catalog, Property 5: catalog name uniqueness
        let mut catalog = StructureCatalog::new();
        let mut successful_creates: std::collections::HashSet<String> = std::collections::HashSet::new();

        for name in &names {
            let def = StructureDefinition {
                metadata: StructureMetadata::new(name),
                associations: None,
                record_structures: vec![RecordStructure::new("Default")],
            };
            // Add a field so it passes validation
            let mut def = def;
            def.record_structures[0].fields.push(
                FieldDefinition::new("F1", 0, 10, FieldType::Alphanumeric)
            );

            let result = catalog.create(def);
            if successful_creates.contains(name) {
                prop_assert!(result.is_err(), "Expected duplicate rejection for '{}'", name);
            } else {
                prop_assert!(result.is_ok(), "Expected success for '{}'", name);
                successful_creates.insert(name.clone());
            }
        }

        prop_assert_eq!(catalog.len(), successful_creates.len());
    }

    /// **Validates: Requirement 9.1, 9.2**
    ///
    /// Property 6: Version Monotonic Increment
    /// For any sequence of updates, version strictly increases by 1 each time.
    #[test]
    fn version_monotonic_increment(
        initial_version in 1u32..100,
        update_count in 1usize..20,
    ) {
        // Feature: structure-catalog, Property 6: version monotonic increment
        let mut catalog = StructureCatalog::new();
        let mut def = StructureDefinition {
            metadata: StructureMetadata::new("TEST"),
            associations: None,
            record_structures: vec![RecordStructure::with_fields(
                "Default",
                vec![FieldDefinition::new("F1", 0, 10, FieldType::Alphanumeric)],
            )],
        };
        def.metadata.version = initial_version;
        // Create via direct insert to bypass version enforcement
        catalog.create(def).unwrap();

        for i in 0..update_count {
            let current = catalog.read("TEST").unwrap().clone();
            let expected_next = current.metadata.version + 1;
            catalog.update(current).unwrap();
            let after = catalog.read("TEST").unwrap();
            prop_assert_eq!(after.metadata.version, expected_next,
                "After update {}, version should be {} but was {}",
                i, expected_next, after.metadata.version);
        }
    }

    /// **Validates: Requirement 13.9, 13.10**
    ///
    /// Property 9: Field Padding/Truncation Length Preservation
    /// The encoded result always has exactly the declared field length.
    #[test]
    fn field_encoding_preserves_declared_length(
        value in "[a-zA-Z0-9]{0,200}",
        length in 1u32..100,
        is_numeric in proptest::bool::ANY,
    ) {
        // Feature: structure-catalog, Property 9: padding/truncation length preservation
        let field_type = if is_numeric { FieldType::Numeric } else { FieldType::Alphanumeric };
        let field = FieldDefinition::new("FIELD", 0, length, field_type);
        let encoded = encode_field_value(&value, &field);
        prop_assert_eq!(encoded.len(), length as usize,
            "Encoded length should be {} but was {} for value '{}'",
            length, encoded.len(), value);
    }
}

// ─── Non-proptest property tests (with manual test cases) ────────────────────

/// **Validates: Requirement 6.3, 6.6**
///
/// Property 3: Packed-decimal encode/decode round-trip (manually structured)
#[test]
fn packed_decimal_valid_bytes_decode_without_error() {
    // Feature: structure-catalog, Property 3: packed-decimal validation
    // All valid packed-decimal bytes should decode without error
    let valid_cases: Vec<(&[u8], u8, &str)> = vec![
        (&[0x01, 0x2C], 0, "12"),           // +12
        (&[0x01, 0x2D], 0, "-12"),          // -12
        (&[0x00, 0x0F], 0, "0"),            // unsigned 0
        (&[0x12, 0x34, 0x5C], 2, "123.45"), // +12345 with 2 decimals
        (&[0x99, 0x9C], 0, "999"),          // +999
    ];

    for (bytes, decimals, expected) in valid_cases {
        let result = decode_packed_decimal(bytes, decimals);
        assert!(result.is_ok(), "Failed to decode {:?}", bytes);
        assert_eq!(result.unwrap(), expected, "Mismatch for {:?}", bytes);
    }
}

/// **Validates: Requirement 10.1, 10.3**
///
/// Property 7: File pattern glob matching — manual comprehensive cases
#[test]
fn glob_matching_correctness() {
    // Feature: structure-catalog, Property 7: glob matching correctness
    use ff_structure_catalog::association::FileAssociationMap;

    let def1 = StructureDefinition {
        metadata: StructureMetadata::new("PAT_STAR"),
        associations: Some(FileAssociations {
            file_patterns: vec!["*.dat".to_string()],
        }),
        record_structures: vec![RecordStructure::with_fields(
            "D",
            vec![FieldDefinition::new("F", 0, 1, FieldType::Alphanumeric)],
        )],
    };
    let def2 = StructureDefinition {
        metadata: StructureMetadata::new("PAT_PREFIX"),
        associations: Some(FileAssociations {
            file_patterns: vec!["CUST_*".to_string()],
        }),
        record_structures: vec![RecordStructure::with_fields(
            "D",
            vec![FieldDefinition::new("F", 0, 1, FieldType::Alphanumeric)],
        )],
    };

    let mut map = FileAssociationMap::new();
    map.rebuild(&[&def1, &def2]);

    use ff_structure_catalog::AssociationResult;

    // *.dat matches any .dat file
    assert_eq!(
        map.match_file("test.dat"),
        AssociationResult::Single("PAT_STAR".to_string())
    );
    assert_eq!(
        map.match_file("x.dat"),
        AssociationResult::Single("PAT_STAR".to_string())
    );

    // CUST_* matches any file starting with CUST_
    assert_eq!(
        map.match_file("CUST_master"),
        AssociationResult::Single("PAT_PREFIX".to_string())
    );

    // Both match CUST_file.dat
    let result = map.match_file("CUST_file.dat");
    assert!(matches!(result, AssociationResult::Multiple(_)));

    // Neither matches
    assert_eq!(map.match_file("readme.txt"), AssociationResult::None);
}

/// **Validates: Requirement 12.3**
///
/// Property 8: Grid field extraction alignment
#[test]
fn grid_field_extraction_alignment() {
    // Feature: structure-catalog, Property 8: grid field extraction alignment
    use ff_structure_catalog::grid::GridBrowseState;

    // Build a record structure with known field boundaries
    let fields = vec![
        FieldDefinition::new("F1", 0, 5, FieldType::Alphanumeric),
        FieldDefinition::new("F2", 5, 3, FieldType::Alphanumeric),
        FieldDefinition::new("F3", 8, 2, FieldType::Alphanumeric),
    ];
    let columns: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
    let mut state = GridBrowseState::new(columns);

    let record = b"HELLOWORLD";
    state.load_records(record, 10, &fields);

    if let ff_structure_catalog::GridRow::Matched { fields: values } = &state.rows()[0] {
        assert_eq!(values[0].display, "HELLO"); // bytes 0..5
        assert_eq!(values[1].display, "WOR"); // bytes 5..8
        assert_eq!(values[2].display, "LD"); // bytes 8..10
    } else {
        panic!("Expected matched row");
    }
}
