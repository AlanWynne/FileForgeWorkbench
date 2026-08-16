//! Property-based tests for ff-language-service.

use proptest::prelude::*;
use std::collections::HashMap;

use ff_language_service::ContentDetector;
use ff_language_service::LanguageRegistry;
use ff_language_service::PropertyStore;
use ff_language_service::{
    ConfigLayer, DefinitionSource, ExtensionMatcher, KeywordSet, KeywordSets, LanguageDefinition,
    LanguageId, LineStateVector, LEXER_STATE_INVALID,
};

fn make_definition_with_ext(id: &str, extensions: &[&str], priority: i32) -> LanguageDefinition {
    LanguageDefinition {
        language_id: LanguageId::new(id).unwrap(),
        name: id.to_string(),
        extensions: extensions.iter().map(|s| s.to_string()).collect(),
        priority,
        case_sensitive_keywords: true,
        keyword_sets: KeywordSets::empty(),
        line_comments: Vec::new(),
        block_comment_start: None,
        block_comment_end: None,
        string_delimiters: Vec::new(),
        character_delimiter: None,
        escape_character: None,
        heredoc_patterns: Vec::new(),
        shebang_patterns: Vec::new(),
        magic_bytes: None,
        first_line_pattern: None,
        embedded_languages: Vec::new(),
        properties: HashMap::new(),
        fold_keywords: None,
        source: DefinitionSource::File {
            path: "test.toml".to_string(),
            layer: ConfigLayer::BuiltIn,
        },
    }
}

// ─── Property 1: Keyword Lookup Correctness ─────────────────────────────────

/// **Validates: Requirements 5.3, 5.4, 5.5**
///
/// For any sorted keyword set and any query word, `contains(word)` SHALL return
/// true if and only if the word is present in the set.
mod keyword_lookup {
    use super::*;

    fn keyword_strategy() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec("[a-zA-Z][a-zA-Z0-9]{0,15}", 1..50)
    }

    proptest! {
        // Feature: language-service, Property 1: keyword lookup correctness (case-sensitive)
        #[test]
        fn contains_matches_set_membership(
            keywords in keyword_strategy(),
            query_idx in 0usize..100,
        ) {
            let ks = KeywordSet::new(keywords.clone(), true);

            // Test hit: pick a known keyword
            if !keywords.is_empty() {
                let idx = query_idx % keywords.len();
                let hit = &keywords[idx];
                prop_assert!(ks.contains(hit),
                    "KeywordSet should contain '{}' which was in the input", hit);
            }
        }

        // Feature: language-service, Property 1: keyword lookup correctness (miss)
        #[test]
        fn contains_rejects_non_members(
            keywords in keyword_strategy(),
            miss in "[a-zA-Z]{17,20}", // longer than max keyword length, guaranteed miss
        ) {
            let ks = KeywordSet::new(keywords.clone(), true);
            // A word longer than all keywords can't be in the set
            if keywords.iter().all(|k| k.len() < 17) {
                prop_assert!(!ks.contains(&miss),
                    "KeywordSet should NOT contain '{}' which was not in the input", miss);
            }
        }

        // Feature: language-service, Property 1: keyword lookup case-insensitive
        #[test]
        fn case_insensitive_matches_regardless_of_casing(
            keywords in prop::collection::vec("[a-z]{1,10}", 1..20),
            query_idx in 0usize..100,
        ) {
            let ks = KeywordSet::new(keywords.clone(), false);

            if !keywords.is_empty() {
                let idx = query_idx % keywords.len();
                let hit = &keywords[idx];
                let upper = hit.to_uppercase();
                prop_assert!(ks.contains_case_insensitive(&upper),
                    "Case-insensitive lookup should match '{}' for keyword '{}'", upper, hit);
            }
        }
    }
}

// ─── Property 2: Extension Matching Case-Insensitivity ──────────────────────

/// **Validates: Requirements 2.1, 2.2**
///
/// For any file extension string and any casing variant, the extension matcher
/// SHALL produce the same detected language.
mod extension_matching {
    use super::*;

    proptest! {
        // Feature: language-service, Property 2: extension matching case-insensitivity
        #[test]
        fn case_variants_produce_same_result(
            ext in "[a-z]{1,6}",
        ) {
            let defs = vec![make_definition_with_ext("testlang", &[&ext], 0)];
            let matcher = ExtensionMatcher::from_definitions(&defs);

            let lower_result = matcher.detect(&format!("file.{}", ext));
            let upper_result = matcher.detect(&format!("file.{}", ext.to_uppercase()));

            // Create a mixed case variant
            let mixed: String = ext.chars().enumerate()
                .map(|(i, c)| if i % 2 == 0 { c.to_uppercase().next().unwrap() } else { c })
                .collect();
            let mixed_result = matcher.detect(&format!("file.{}", mixed));

            prop_assert_eq!(&lower_result.language_id, &upper_result.language_id,
                "Lower '{}' and upper '{}' should match same language", ext, ext.to_uppercase());
            prop_assert_eq!(&lower_result.language_id, &mixed_result.language_id,
                "Lower '{}' and mixed '{}' should match same language", ext, mixed);
        }
    }
}

// ─── Property 3: Lexer State Vector Insert/Delete Consistency ───────────────

/// **Validates: Requirements 4.6, 4.7**
///
/// For any sequence of insert and delete operations, the vector length SHALL
/// always equal initial_size + total_inserted - total_deleted.
mod state_vector_consistency {
    use super::*;

    #[derive(Debug, Clone)]
    enum StateOp {
        Insert { at: usize, count: usize },
        Delete { at: usize, count: usize },
    }

    fn state_ops_strategy(initial_size: usize) -> impl Strategy<Value = Vec<StateOp>> {
        prop::collection::vec(
            prop_oneof![
                (0..initial_size.max(1), 1..5usize)
                    .prop_map(|(at, count)| StateOp::Insert { at, count }),
                (0..initial_size.max(1), 1..3usize)
                    .prop_map(|(at, count)| StateOp::Delete { at, count }),
            ],
            1..20,
        )
    }

    proptest! {
        // Feature: language-service, Property 3: lexer state vector insert/delete consistency
        #[test]
        fn length_tracks_insertions_and_deletions(
            initial_size in 10..100usize,
            ops in state_ops_strategy(100),
        ) {
            let mut sv = LineStateVector::new(initial_size);
            let mut expected_len = initial_size;

            for op in ops {
                match op {
                    StateOp::Insert { at, count } => {
                        let at = at % sv.len().max(1);
                        sv.insert_lines(at, count);
                        expected_len += count;
                    }
                    StateOp::Delete { at, count } => {
                        if sv.len() > 1 {
                            let at = at % sv.len();
                            let actual_count = count.min(sv.len() - at);
                            if actual_count > 0 && sv.len() - actual_count > 0 {
                                sv.delete_lines(at, actual_count);
                                expected_len -= actual_count;
                            }
                        }
                    }
                }
                prop_assert_eq!(sv.len(), expected_len,
                    "Vector length mismatch after operation");
            }
        }

        // Feature: language-service, Property 3: inserted entries are INVALID_STATE
        #[test]
        fn inserted_entries_are_invalid(
            initial_size in 5..50usize,
            at in 0..50usize,
            count in 1..5usize,
        ) {
            let mut sv = LineStateVector::new(initial_size);
            // Set all to valid first
            for i in 0..initial_size {
                sv.set_end_state(i, i as i32);
            }

            let insert_at = at % initial_size;
            sv.insert_lines(insert_at, count);

            // Check that inserted entries are INVALID
            for i in insert_at..insert_at + count {
                prop_assert_eq!(sv.get_state(i), Some(LEXER_STATE_INVALID),
                    "Inserted entry at {} should be INVALID_STATE", i);
            }
        }
    }
}

// ─── Property 4: Content Detection Priority Ordering ────────────────────────

/// **Validates: Requirements 3.5**
///
/// Magic bytes > shebang > first-line pattern.
mod content_detection_priority {
    use super::*;
    use ff_language_service::DetectionMethod;

    fn make_detection_def(
        id: &str,
        shebang: &[&str],
        magic: Option<Vec<u8>>,
        first_line: Option<&str>,
    ) -> LanguageDefinition {
        LanguageDefinition {
            language_id: LanguageId::new(id).unwrap(),
            name: id.to_string(),
            extensions: Vec::new(),
            priority: 0,
            case_sensitive_keywords: true,
            keyword_sets: KeywordSets::empty(),
            line_comments: Vec::new(),
            block_comment_start: None,
            block_comment_end: None,
            string_delimiters: Vec::new(),
            character_delimiter: None,
            escape_character: None,
            heredoc_patterns: Vec::new(),
            shebang_patterns: shebang.iter().map(|s| s.to_string()).collect(),
            magic_bytes: magic,
            first_line_pattern: first_line.map(|s| s.to_string()),
            embedded_languages: Vec::new(),
            properties: HashMap::new(),
            fold_keywords: None,
            source: DefinitionSource::File {
                path: "test.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        }
    }

    proptest! {
        // Feature: language-service, Property 4: content detection priority ordering
        #[test]
        fn magic_bytes_always_wins_over_shebang(
            magic_byte in 0x01u8..0xFFu8,
        ) {
            let defs = vec![
                make_detection_def("magic-lang", &[], Some(vec![0x7F, magic_byte]), None),
                make_detection_def("shebang-lang", &["interpreter"], None, None),
            ];
            let detector = ContentDetector::from_definitions(&defs);

            // Content that matches BOTH magic bytes and shebang
            let mut content = vec![0x7F, magic_byte];
            content.extend_from_slice(b"#!/usr/bin/env interpreter\n");

            let result = detector.detect(Some(&content), None);
            prop_assert_eq!(result.language_id.as_str(), "magic-lang",
                "Magic bytes should take priority over shebang");
            prop_assert_eq!(result.method, DetectionMethod::MagicBytes);
        }

        // Feature: language-service, Property 4: shebang wins over first-line pattern
        #[test]
        fn shebang_wins_over_first_line_pattern(
            interp in "[a-z]{3,8}",
        ) {
            let defs = vec![
                make_detection_def("shebang-lang", &[&interp], None, None),
                make_detection_def("pattern-lang", &[], None, Some("^#!")),
            ];
            let detector = ContentDetector::from_definitions(&defs);

            let first_line = format!("#!/usr/bin/env {}", interp);
            let result = detector.detect(None, Some(&first_line));
            prop_assert_eq!(result.language_id.as_str(), "shebang-lang",
                "Shebang should take priority over first-line pattern");
        }
    }
}

// ─── Property 5: Property Boolean Parsing Correctness ───────────────────────

/// **Validates: Requirements 8.4**
///
/// get_property_bool SHALL return true for "1", "true", "yes" (case-insensitive),
/// false for "0", "false", "no" (case-insensitive), and default for other strings.
mod property_bool_parsing {
    use super::*;

    proptest! {
        // Feature: language-service, Property 5: property boolean parsing correctness
        #[test]
        fn known_true_values_return_true(
            value in prop_oneof![
                Just("1".to_string()),
                Just("true".to_string()),
                Just("yes".to_string()),
                Just("TRUE".to_string()),
                Just("True".to_string()),
                Just("YES".to_string()),
                Just("Yes".to_string()),
            ],
        ) {
            let mut store = PropertyStore::new();
            let lang = LanguageId::new("test").unwrap();
            let mut props = HashMap::new();
            props.insert("key".to_string(), value.clone());
            store.register_builtins(&lang, props);

            prop_assert!(store.get_property_bool(&lang, "key", false),
                "Value '{}' should parse as true", value);
        }

        // Feature: language-service, Property 5: known false values return false
        #[test]
        fn known_false_values_return_false(
            value in prop_oneof![
                Just("0".to_string()),
                Just("false".to_string()),
                Just("no".to_string()),
                Just("FALSE".to_string()),
                Just("False".to_string()),
                Just("NO".to_string()),
                Just("No".to_string()),
            ],
        ) {
            let mut store = PropertyStore::new();
            let lang = LanguageId::new("test").unwrap();
            let mut props = HashMap::new();
            props.insert("key".to_string(), value.clone());
            store.register_builtins(&lang, props);

            prop_assert!(!store.get_property_bool(&lang, "key", true),
                "Value '{}' should parse as false", value);
        }

        // Feature: language-service, Property 5: unknown values return default
        #[test]
        fn unknown_values_return_default(
            value in "[a-z]{2,10}".prop_filter("not a known value",
                |v| !["true", "false", "yes", "no"].contains(&v.as_str())),
            default in proptest::bool::ANY,
        ) {
            let mut store = PropertyStore::new();
            let lang = LanguageId::new("test").unwrap();
            let mut props = HashMap::new();
            props.insert("key".to_string(), value.clone());
            store.register_builtins(&lang, props);

            let result = store.get_property_bool(&lang, "key", default);
            prop_assert_eq!(result, default,
                "Unknown value '{}' should return default {}", value, default);
        }
    }
}

// ─── Property 6: Language Registration Idempotency ──────────────────────────

/// **Validates: Requirements 9.3**
///
/// Registering a duplicate language_id SHALL always fail, and the existing
/// definition SHALL remain unchanged.
mod registration_idempotency {
    use super::*;

    proptest! {
        // Feature: language-service, Property 6: language registration idempotency
        #[test]
        fn duplicate_registration_fails_and_preserves_original(
            id in "[a-z]{3,10}",
            name_a in "[A-Z][a-z]{2,8}",
            name_b in "[A-Z][a-z]{2,8}",
        ) {
            let lang_id = LanguageId::new(&id).unwrap();
            let def_a = LanguageDefinition {
                language_id: lang_id.clone(),
                name: name_a.clone(),
                extensions: vec!["ext_a".to_string()],
                priority: 0,
                case_sensitive_keywords: true,
                keyword_sets: KeywordSets::empty(),
                line_comments: Vec::new(),
                block_comment_start: None,
                block_comment_end: None,
                string_delimiters: Vec::new(),
                character_delimiter: None,
                escape_character: None,
                heredoc_patterns: Vec::new(),
                shebang_patterns: Vec::new(),
                magic_bytes: None,
                first_line_pattern: None,
                embedded_languages: Vec::new(),
                properties: HashMap::new(),
                fold_keywords: None,
                source: DefinitionSource::File {
                    path: "a.toml".to_string(),
                    layer: ConfigLayer::BuiltIn,
                },
            };
            let def_b = LanguageDefinition {
                language_id: lang_id.clone(),
                name: name_b.clone(),
                extensions: vec!["ext_b".to_string()],
                priority: 5,
                case_sensitive_keywords: true,
                keyword_sets: KeywordSets::empty(),
                line_comments: Vec::new(),
                block_comment_start: None,
                block_comment_end: None,
                string_delimiters: Vec::new(),
                character_delimiter: None,
                escape_character: None,
                heredoc_patterns: Vec::new(),
                shebang_patterns: Vec::new(),
                magic_bytes: None,
                first_line_pattern: None,
                embedded_languages: Vec::new(),
                properties: HashMap::new(),
                fold_keywords: None,
                source: DefinitionSource::Plugin {
                    plugin_name: "test".to_string(),
                },
            };

            let mut registry = LanguageRegistry::new();
            prop_assert!(registry.register(def_a).is_ok(), "First registration should succeed");

            let result = registry.register(def_b);
            prop_assert!(result.is_err(), "Duplicate registration should fail");

            // Original should be unchanged
            let stored = registry.get(&lang_id).unwrap();
            prop_assert_eq!(stored.name(), &name_a,
                "Original definition should be preserved after failed duplicate registration");
        }
    }
}

// ─── Property 7: Compound Extension Priority Over Simple Extension ──────────

/// **Validates: Requirements 2.3, 2.6**
///
/// Compound extensions SHALL have higher priority than simple extensions.
mod compound_extension_priority {
    use super::*;

    proptest! {
        // Feature: language-service, Property 7: compound extension priority over simple extension
        #[test]
        fn compound_extension_wins_over_simple(
            base_ext in "[a-z]{2,4}",
            compound_prefix in "[a-z]{2,6}",
            filename_base in "[a-z]{3,8}",
        ) {
            let compound_ext = format!("{}.{}", compound_prefix, base_ext);

            let defs = vec![
                make_definition_with_ext("simple-lang", &[&base_ext], 0),
                make_definition_with_ext("compound-lang", &[&compound_ext], 0),
            ];
            let matcher = ExtensionMatcher::from_definitions(&defs);

            let filename = format!("{}.{}.{}", filename_base, compound_prefix, base_ext);
            let result = matcher.detect(&filename);

            prop_assert_eq!(result.language_id.as_str(), "compound-lang",
                "Compound extension '{}' should win for file '{}'", compound_ext, filename);
        }
    }
}
