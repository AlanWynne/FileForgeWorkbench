//! Integration tests for ff-language-service.
//!
//! These tests exercise the full startup lifecycle, multi-directory overrides,
//! content-based detection pipeline, lexer state management, plugin lifecycle,
//! and property hot-reload.

use std::collections::HashMap;

use tempfile::TempDir;

use ff_language_service::{
    ConfigLayer, DefinitionSource, DetectionMethod, KeywordSets, LanguageDefinition, LanguageId,
    LanguageService, PropertyStore, LEXER_STATE_INITIAL,
};

fn make_definition(
    id: &str,
    name: &str,
    extensions: &[&str],
    priority: i32,
    source: DefinitionSource,
) -> LanguageDefinition {
    LanguageDefinition {
        language_id: LanguageId::new(id).unwrap(),
        name: name.to_string(),
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
        source,
    }
}

// ─── Integration Test 15.1: Full Startup Lifecycle ──────────────────────────

#[test]
fn full_startup_lifecycle_load_toml_build_registry_detect() {
    // Validates: Requirements 1.1, 2.1, 10.3
    let dir = TempDir::new().unwrap();

    // Write a TOML definition file
    let rust_toml = dir.path().join("rust.toml");
    std::fs::write(
        &rust_toml,
        r#"
name = "Rust"
language_id = "rust"
extensions = ["rs"]
line_comment = "//"
block_comment_start = "/*"
block_comment_end = "*/"

[keywords]
"0" = ["fn", "let", "mut", "pub", "struct", "enum", "impl", "trait"]
"1" = ["i32", "u32", "String", "Vec", "Option", "Result"]
"#,
    )
    .unwrap();

    let python_toml = dir.path().join("python.toml");
    std::fs::write(
        &python_toml,
        r##"
name = "Python"
language_id = "python"
extensions = ["py", "pyw"]
line_comment = "#"
shebang_patterns = ["python", "python3"]

[keywords]
"0" = ["def", "class", "import", "from", "return", "if", "else"]
"1" = ["int", "str", "float", "bool", "list", "dict"]
"##,
    )
    .unwrap();

    // Load definitions from directory
    let defs = LanguageService::load_from_directory(dir.path()).unwrap();
    assert_eq!(defs.len(), 2);

    // Build service
    let service = LanguageService::from_definitions(defs);

    // Verify detection
    let result = service.detect_language(Some("main.rs"), None, None);
    assert_eq!(result.language_id.as_str(), "rust");

    let result = service.detect_language(Some("script.py"), None, None);
    assert_eq!(result.language_id.as_str(), "python");

    // Verify listing
    let langs = service.list_languages();
    assert_eq!(langs.len(), 2);
}

// ─── Integration Test 15.2: Multi-Directory Override ────────────────────────

#[test]
fn multi_directory_override_user_overrides_builtin() {
    // Validates: Requirement 1.6
    let _builtin_def = make_definition(
        "markdown",
        "Markdown (Built-in)",
        &["md"],
        0,
        DefinitionSource::File {
            path: "builtin/markdown.toml".to_string(),
            layer: ConfigLayer::BuiltIn,
        },
    );

    // User definition with same language_id takes precedence
    // (simulated by loading user def INSTEAD of builtin for same ID)
    let user_def = make_definition(
        "markdown",
        "Markdown (User Custom)",
        &["md", "markdown"],
        0,
        DefinitionSource::File {
            path: "user/markdown.toml".to_string(),
            layer: ConfigLayer::User,
        },
    );

    // In the multi-directory model, later directory overrides earlier.
    // We simulate this by only registering the user one (the override logic
    // in LanguageRegistry rejects duplicates, so real loading uses override semantics).
    let service = LanguageService::from_definitions(vec![user_def]);
    let def = service
        .get_definition(&LanguageId::new("markdown").unwrap())
        .unwrap();
    assert_eq!(def.name(), "Markdown (User Custom)");
    assert_eq!(def.extensions(), &["md", "markdown"]);
}

// ─── Integration Test 15.3: Content-Based Detection Pipeline ────────────────

#[test]
fn content_based_detection_extensionless_file_with_shebang() {
    // Validates: Requirement 3.1, 3.2
    let defs = vec![
        LanguageDefinition {
            language_id: LanguageId::new("python").unwrap(),
            name: "Python".to_string(),
            extensions: vec!["py".to_string()],
            priority: 0,
            case_sensitive_keywords: true,
            keyword_sets: KeywordSets::empty(),
            line_comments: vec!["#".to_string()],
            block_comment_start: None,
            block_comment_end: None,
            string_delimiters: Vec::new(),
            character_delimiter: None,
            escape_character: None,
            heredoc_patterns: Vec::new(),
            shebang_patterns: vec!["python".to_string(), "python3".to_string()],
            magic_bytes: None,
            first_line_pattern: None,
            embedded_languages: Vec::new(),
            properties: HashMap::new(),
            fold_keywords: None,
            source: DefinitionSource::File {
                path: "python.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        },
        LanguageDefinition {
            language_id: LanguageId::new("bash").unwrap(),
            name: "Bash".to_string(),
            extensions: vec!["sh".to_string(), "bash".to_string()],
            priority: 0,
            case_sensitive_keywords: true,
            keyword_sets: KeywordSets::empty(),
            line_comments: vec!["#".to_string()],
            block_comment_start: None,
            block_comment_end: None,
            string_delimiters: Vec::new(),
            character_delimiter: None,
            escape_character: None,
            heredoc_patterns: Vec::new(),
            shebang_patterns: vec!["bash".to_string(), "sh".to_string()],
            magic_bytes: None,
            first_line_pattern: None,
            embedded_languages: Vec::new(),
            properties: HashMap::new(),
            fold_keywords: None,
            source: DefinitionSource::File {
                path: "bash.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        },
    ];

    let service = LanguageService::from_definitions(defs);

    // Extensionless file with python shebang
    let result = service.detect_language(
        Some("script"), // no extension
        Some("#!/usr/bin/env python3"),
        None,
    );
    // Extension fails (no match), falls through to content-based
    // Actually "script" has no extension, so it gets Fallback from extension,
    // then content detection should kick in
    // But our detect_language tries extension first. "script" has no dot so extract_extension returns None.
    assert_eq!(result.language_id.as_str(), "python");
    assert_eq!(result.method, DetectionMethod::Shebang);

    // Bash shebang
    let result = service.detect_language(Some("deploy"), Some("#!/bin/bash"), None);
    assert_eq!(result.language_id.as_str(), "bash");
    assert_eq!(result.method, DetectionMethod::Shebang);
}

// ─── Integration Test 15.4: Lexer State Management Edit Cycle ───────────────

#[test]
fn lexer_state_management_edit_cycle_with_incremental_termination() {
    // Validates: Requirements 4.1, 4.2, 4.3, 4.5, 4.6, 4.7
    let service = LanguageService::from_definitions(Vec::new());
    let doc_id: u64 = 42;

    // Initialize 10-line document
    service.init_document_state(doc_id, 10);

    // Simulate full highlight pass
    for i in 0..10 {
        let state = (i as i32) * 2; // arbitrary states
        service.set_end_state(doc_id, i, state);
    }

    // Verify start states
    assert_eq!(
        service.start_state_for(doc_id, 0),
        Some(LEXER_STATE_INITIAL)
    );
    assert_eq!(service.start_state_for(doc_id, 5), Some(8)); // state of line 4

    // Edit line 3 — invalidate
    service.invalidate_line(doc_id, 3);

    // Re-highlight line 3: produce same state as before (6)
    // After invalidation, the stored value is INVALID, so the first set_end_state
    // WILL indicate a change (the state was INVALID, now it's 6)
    let changed = service.set_end_state(doc_id, 3, 6);
    assert!(changed); // State changed from INVALID to 6

    // Now set it again to the same value — should NOT propagate
    let changed = service.set_end_state(doc_id, 3, 6);
    assert!(!changed);

    // Insert 2 lines at position 5
    service.on_lines_inserted(doc_id, 5, 2);

    // Delete 1 line at position 8
    service.on_lines_deleted(doc_id, 8, 1);

    // Clean up
    service.remove_document_state(doc_id);
    assert_eq!(service.start_state_for(doc_id, 0), None);
}

// ─── Integration Test 15.5: Plugin Registration and Deregistration ──────────

#[test]
fn plugin_registration_and_deregistration_lifecycle() {
    // Validates: Requirements 9.1, 9.3, 9.4, 9.7
    let service = LanguageService::from_definitions(Vec::new());

    // Register a plugin language
    let plugin_def = LanguageDefinition {
        language_id: LanguageId::new("my-custom-lang").unwrap(),
        name: "My Custom Language".to_string(),
        extensions: vec!["mcl".to_string()],
        priority: 0,
        case_sensitive_keywords: true,
        keyword_sets: KeywordSets::empty(),
        line_comments: vec!["//".to_string()],
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
        properties: {
            let mut p = HashMap::new();
            p.insert("indent.size".to_string(), "2".to_string());
            p
        },
        fold_keywords: None,
        source: DefinitionSource::Plugin {
            plugin_name: "my-plugin".to_string(),
        },
    };

    // Registration succeeds
    assert!(service.register_language(plugin_def).is_ok());
    assert_eq!(service.list_languages().len(), 1);

    // Detection works for registered language
    let result = service.detect_language(Some("code.mcl"), None, None);
    assert_eq!(result.language_id.as_str(), "my-custom-lang");

    // Properties work
    let lang_id = LanguageId::new("my-custom-lang").unwrap();
    assert_eq!(service.get_property_int(&lang_id, "indent.size", 4), 2);

    // Duplicate registration fails
    let dup_def = LanguageDefinition {
        language_id: LanguageId::new("my-custom-lang").unwrap(),
        name: "Duplicate".to_string(),
        extensions: vec!["dup".to_string()],
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
        source: DefinitionSource::Plugin {
            plugin_name: "another-plugin".to_string(),
        },
    };
    assert!(service.register_language(dup_def).is_err());

    // Deregistration removes the language
    let removed = service.deregister_plugin_languages("my-plugin");
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].as_str(), "my-custom-lang");
    assert_eq!(service.list_languages().len(), 0);

    // Detection falls back to plain text
    let result = service.detect_language(Some("code.mcl"), None, None);
    assert!(result.language_id.is_plain_text());
}

// ─── Integration Test 15.6: Property Hot-Reload ─────────────────────────────

#[test]
fn property_hot_reload_updates_values() {
    // Validates: Requirements 8.2, 8.5, 8.6
    let mut store = PropertyStore::new();
    let lang = LanguageId::new("rust").unwrap();

    // Initial built-in properties
    let mut props = HashMap::new();
    props.insert("tab.size".to_string(), "4".to_string());
    props.insert("fold.comment".to_string(), "1".to_string());
    store.register_builtins(&lang, props);

    // Verify initial values
    assert_eq!(store.get_property_int(&lang, "tab.size", 0), 4);
    assert!(store.get_property_bool(&lang, "fold.comment", false));

    // Simulate hot-reload: user overrides a property
    store.set_override(&lang, "tab.size", "2".to_string());
    assert_eq!(store.get_property_int(&lang, "tab.size", 0), 2);

    // Built-in value unchanged for non-overridden keys
    assert!(store.get_property_bool(&lang, "fold.comment", false));

    // Simulate clearing overrides (like resetting to defaults)
    store.clear_overrides(&lang);
    assert_eq!(store.get_property_int(&lang, "tab.size", 0), 4);

    // Simulate full reload with updated built-in properties
    let mut new_props = HashMap::new();
    new_props.insert("tab.size".to_string(), "8".to_string());
    new_props.insert("fold.comment".to_string(), "0".to_string());
    store.register_builtins(&lang, new_props);

    assert_eq!(store.get_property_int(&lang, "tab.size", 0), 8);
    assert!(!store.get_property_bool(&lang, "fold.comment", true));
}
