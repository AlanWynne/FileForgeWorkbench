//! Property-based tests for ff-encoding crate.
//!
//! Uses proptest framework to verify universal properties across many inputs.

use proptest::prelude::*;

use ff_encoding::*;

// ============================================================================
// Property 1: Encoding roundtrip preservation
// convert_to_utf8(convert_from_utf8(text, enc)) == text for mappable text
// **Validates: Requirements 3, 4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn encoding_roundtrip_iso_8859_1(
        // Feature: encoding-and-characters, Property 1: Encoding roundtrip preservation
        text in "[\\x00-\\x7F]{1,100}"
    ) {
        // Validates: Requirements 3, 4
        let registry = EncodingRegistry::new();
        let encoding = registry.by_name("iso-8859-1").unwrap();

        let encoded = convert_from_utf8(&text, encoding, UnmappableAction::Abort).unwrap();
        let decoded = convert_to_utf8(&encoded.data, encoding).unwrap();
        let result = String::from_utf8(decoded.data).unwrap();
        prop_assert_eq!(&result, &text);
    }
}

// ============================================================================
// Property 2: BOM detection accuracy
// prepending BOM bytes always yields correct detection
// **Validates: Requirements 2.1, 2.2, 2.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn bom_detection_always_correct(
        // Feature: encoding-and-characters, Property 2: BOM detection accuracy
        content in prop::collection::vec(any::<u8>(), 0..50),
        bom_type in 0u8..5
    ) {
        // Validates: Requirements 2.1, 2.2, 2.3
        let (bom_encoding, expected) = match bom_type {
            0 => (BomEncoding::Utf8, BomEncoding::Utf8),
            1 => (BomEncoding::Utf16Le, BomEncoding::Utf16Le),
            2 => (BomEncoding::Utf16Be, BomEncoding::Utf16Be),
            3 => (BomEncoding::Utf32Le, BomEncoding::Utf32Le),
            _ => (BomEncoding::Utf32Be, BomEncoding::Utf32Be),
        };

        let bom = bom_bytes(bom_encoding);
        let mut data = Vec::with_capacity(bom.len() + content.len());
        data.extend_from_slice(bom);
        data.extend_from_slice(&content);

        let result = detect_bom(&data);
        prop_assert!(result.is_some(), "BOM should be detected");
        let info = result.unwrap();
        prop_assert_eq!(info.encoding, expected);
    }
}

// ============================================================================
// Property 3: Case fold idempotence
// folding an already-folded string yields the same result
// **Validates: Requirements 10.1, 10.4, 10.6**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn case_fold_is_idempotent(
        // Feature: encoding-and-characters, Property 3: Case fold idempotence
        text in "[a-zA-Z\\x{00DF}\\x{FB01}\\x{FB02} ]{1,50}"
    ) {
        // Validates: Requirements 10.1, 10.4, 10.6
        let folder = CaseFolder::new();
        let once = folder.case_convert_string(&text, CaseMode::Fold);
        let twice = folder.case_convert_string(&once, CaseMode::Fold);
        prop_assert_eq!(&once, &twice, "Fold must be idempotent");
    }
}

// ============================================================================
// Property 4: UTF-8 validation consistency
// utf8_validate agrees with std::str::from_utf8
// **Validates: Requirements 5.1, 5.4, 5.5**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn utf8_validation_consistent_with_stdlib(
        // Feature: encoding-and-characters, Property 4: UTF-8 validation consistency
        bytes in prop::collection::vec(any::<u8>(), 0..256)
    ) {
        // Validates: Requirements 5.1, 5.4, 5.5
        let our_result = utf8_validate(&bytes);
        let std_result = std::str::from_utf8(&bytes).is_ok();
        prop_assert_eq!(our_result, std_result,
            "Our validator disagrees with std on {:?}", &bytes[..bytes.len().min(20)]);
    }
}

// ============================================================================
// Property 5: UTF-8 fix produces valid UTF-8
// utf8_fix_invalid output is always valid
// **Validates: Requirements 5.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn utf8_fix_always_produces_valid_utf8(
        // Feature: encoding-and-characters, Property 5: UTF-8 fix produces valid UTF-8
        bytes in prop::collection::vec(any::<u8>(), 0..256)
    ) {
        // Validates: Requirements 5.3
        let fixed = utf8_fix_invalid(&bytes);
        prop_assert!(std::str::from_utf8(fixed.as_bytes()).is_ok(),
            "utf8_fix_invalid produced invalid UTF-8");
    }
}

// ============================================================================
// Property 6: CharClassify completeness
// every byte 0–255 has exactly one class
// **Validates: Requirements 6.1**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn char_classify_completeness(
        // Feature: encoding-and-characters, Property 6: CharClassify completeness
        include_word in any::<bool>(),
        extra_word_chars in prop::collection::vec(any::<u8>(), 0..10)
    ) {
        // Validates: Requirements 6.1
        let mut classify = CharClassify::new(include_word);
        classify.set_word_chars(&extra_word_chars);

        for i in 0..=255u16 {
            let class = classify.classify(i as u8);
            prop_assert!(matches!(class,
                CharacterClass::Space | CharacterClass::NewLine |
                CharacterClass::Word | CharacterClass::Punctuation
            ), "Byte {} has invalid class {:?}", i, class);
        }
    }
}

// ============================================================================
// Property 7: Grapheme boundary monotonicity
// next always advances, prev always retreats
// **Validates: Requirements 9.2, 9.3, 9.4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn grapheme_boundary_monotonicity(
        // Feature: encoding-and-characters, Property 7: Grapheme boundary monotonicity
        text in "[a-z\\x{0300}-\\x{036F}]{1,20}"
    ) {
        // Validates: Requirements 9.2, 9.3, 9.4
        // Test next always advances
        let mut pos = 0;
        let mut iterations = 0;
        while pos < text.len() && iterations < 100 {
            let next = next_grapheme_boundary(&text, pos);
            prop_assert!(next > pos, "next_grapheme_boundary({}) did not advance", pos);
            pos = next;
            iterations += 1;
        }

        // Test prev always retreats
        pos = text.len();
        iterations = 0;
        while pos > 0 && iterations < 100 {
            let prev = prev_grapheme_boundary(&text, pos);
            prop_assert!(prev < pos, "prev_grapheme_boundary({}) did not retreat", pos);
            pos = prev;
            iterations += 1;
        }
    }
}

// ============================================================================
// Property 8: DBCS lead+trail byte disjointness
// no ASCII byte is a lead byte
// **Validates: Requirements 8.2, 8.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn dbcs_ascii_never_lead_byte(
        // Feature: encoding-and-characters, Property 8: DBCS lead+trail byte disjointness
        byte in 0u8..0x80,
        code_page in prop::sample::select(vec![
            DbcsCodePage::ShiftJis,
            DbcsCodePage::Gbk,
            DbcsCodePage::KoreanWansung,
            DbcsCodePage::Big5,
            DbcsCodePage::KoreanJohab,
        ])
    ) {
        // Validates: Requirements 8.2, 8.3
        prop_assert!(!dbcs_is_lead_byte(code_page, byte),
            "ASCII byte 0x{:02X} should not be lead byte for {:?}", byte, code_page);
    }
}

// ============================================================================
// Property 9: Word-part navigation termination
// left always ≤ pos, right always ≥ pos
// **Validates: Requirements 12.2, 12.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn word_part_navigation_bounds(
        // Feature: encoding-and-characters, Property 9: Word-part navigation termination
        text in "[a-zA-Z0-9_]{1,30}",
        pos_frac in 0.0f64..=1.0
    ) {
        // Validates: Requirements 12.2, 12.3
        let classify = CharClassify::new(true);
        let pos = (pos_frac * text.len() as f64) as usize;
        let pos = pos.min(text.len());

        let left = word_part_left(&text, pos, &classify);
        let right = word_part_right(&text, pos, &classify);

        prop_assert!(left <= pos, "word_part_left({}) = {} > pos", pos, left);
        prop_assert!(right >= pos, "word_part_right({}) = {} < pos", pos, right);
    }
}

// ============================================================================
// Property 10: Encoding family consistency
// DBCS code pages map to Dbcs family
// **Validates: Requirements 11.1, 11.2**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn encoding_family_consistency(
        // Feature: encoding-and-characters, Property 10: Encoding family consistency
        _dummy in Just(())
    ) {
        // Validates: Requirements 11.1, 11.2
        let registry = EncodingRegistry::new();
        for enc in registry.all() {
            let computed = ff_encoding::encoding::encoding_family(enc.code_page);
            prop_assert_eq!(enc.family, computed,
                "Family mismatch for '{}' (CP{})", enc.name, enc.code_page);
        }
    }
}
