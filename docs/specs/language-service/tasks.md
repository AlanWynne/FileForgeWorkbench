# Implementation Plan: Language Service (`ff-language-service`)

## Overview

This plan covers the complete implementation of the `ff-language-service` crate — the foundational layer responsible for language detection, language definition management, multi-line lexer state persistence, and content-based language identification. The language service loads language definitions from TOML files, matches files to languages via extension or content inspection, manages per-line lexer state for multi-line constructs, and exposes a plugin-extensible registration model for adding new language definitions at runtime.

This is a **Wave 7 (Language and Highlighting)** sub-project. It depends on:
- `ff-logging` (Wave 1) — diagnostics and structured log records
- `ff-configuration-system` (Wave 2) — languages/ directory configuration, layered settings, hot-reload notifications

It is consumed by:
- `ff-syntax-highlighting` (Wave 7 peer) — token production using language definitions and per-line state
- `ff-auto-indentation` (Wave 7 peer) — language-aware indent rules
- `ff-plugin` (Wave 5) — runtime language registration via PluginContext

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-language-service/Cargo.toml` with dependencies (serde, toml, thiserror, regex, proptest dev-dep) and deps on `ff-logging`, `ff-configuration-system`
  - [x] 1.2 Create `crates/ff-language-service/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `definition.rs`, `keyword_set.rs`, `detection.rs`, `content_detection.rs`, `lexer_state.rs`, `properties.rs`, `registry.rs`, `query.rs`, `embedded.rs`, `error.rs`
  - [x] 1.4 Add `ff-language-service` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Language definition types
  - [x] 2.1 Define `LanguageId` newtype wrapping a lowercase ASCII string with validation constructor and `Default` impl returning `"plain_text"`
  - [x] 2.2 Define `LanguageDefinition` struct with fields: `name` (String), `language_id` (LanguageId), `extensions` (Vec<String>), `priority` (i32, default 0), `case_sensitive_keywords` (bool, default true)
  - [x] 2.3 Add optional fields to `LanguageDefinition`: `line_comment`, `block_comment_start`, `block_comment_end`, `string_delimiters`, `character_delimiter`, `escape_character`, `heredoc_patterns`, `shebang_patterns`, `magic_bytes`, `first_line_pattern`
  - [x] 2.4 Add `properties` (HashMap<String, String>) and `embedded_languages` (Vec<EmbeddedLanguageDescriptor>) fields to `LanguageDefinition`
  - [x] 2.5 Define `EmbeddedLanguageDescriptor` struct with `language_id` (LanguageId), `start_pattern` (String), `end_pattern` (String)
  - [x] 2.6 Define `LanguageSummary` struct with `language_id`, `name`, and `extensions` for query API results
  - [x] 2.7 Implement `serde::Deserialize` for `LanguageDefinition` matching TOML schema (name, language_id, extensions, keywords table, optional fields)
  - [x] 2.8 Write unit tests for LanguageId validation, LanguageDefinition default values, serde deserialization from TOML strings
  - Covers: Requirements 1 (AC 1.2, 1.3), 6 (AC 6.1–6.7), 7 (AC 7.1)

- [x] 3. Keyword set management
  - [x] 3.1 Define `KeywordSet` struct containing a sorted `Vec<String>` and a first-character index (`[Option<usize>; 128]`) for O(1) starting position lookup
  - [x] 3.2 Implement `KeywordSet::new(words: Vec<String>) -> Self` that sorts keywords alphabetically and builds the first-character index
  - [x] 3.3 Implement `KeywordSet::contains(&self, word: &str) -> bool` performing case-sensitive binary search using the first-character index
  - [x] 3.4 Implement `KeywordSet::contains_case_insensitive(&self, word: &str) -> bool` performing lowercased comparison against pre-lowercased keywords
  - [x] 3.5 Define `KeywordSets` struct wrapping `[Option<KeywordSet>; 9]` for keyword sets 0–8
  - [x] 3.6 Implement `KeywordSets::from_toml_table(table: &HashMap<String, Vec<String>>, case_sensitive: bool) -> Self` parsing keyword sets from TOML structure
  - [x] 3.7 Implement `in_keyword_set(&self, word: &str, set_number: u8) -> bool` and `in_keyword_set_case_insensitive(&self, word: &str, set_number: u8) -> bool` methods
  - [x] 3.8 Implement keyword set to style identifier mapping: `KeywordSets::style_name_for_set(set_number: u8) -> &str` with configurable name override
  - [x] 3.9 Write unit tests for sorted insertion, first-char index correctness, case-sensitive/insensitive lookup, empty set handling, set number bounds
  - Covers: Requirement 5 (AC 5.1–5.7)

- [x] 4. TOML definition loading
  - [x] 4.1 Implement `LanguageLoader` struct with `load_from_directory(path: &Path) -> Vec<Result<LanguageDefinition, LoadError>>` scanning all `*.toml` files
  - [x] 4.2 Implement multi-directory search order: built-in defaults → user configuration → project-local, with later directories overriding earlier definitions for the same `language_id`
  - [x] 4.3 Implement TOML parse error handling: skip invalid files, emit WARN log with file path and error description, continue loading remaining files
  - [x] 4.4 Implement duplicate `language_id` detection: use first loaded definition, emit WARN log for duplicates, discard subsequent definitions (within same priority layer)
  - [x] 4.5 Implement post-load summary: emit DEBUG log with count of successfully loaded definitions and count of skipped files
  - [x] 4.6 Write unit tests for successful TOML loading, malformed TOML skipping, duplicate ID handling, multi-directory override semantics, summary logging
  - Covers: Requirement 1 (AC 1.1, 1.4–1.7)

- [x] 5. Language detection by file extension
  - [x] 5.1 Implement `ExtensionMatcher` struct that builds a HashMap from extensions (lowercased) to Vec<(LanguageId, priority, is_compound)> from all loaded definitions
  - [x] 5.2 Implement case-insensitive extension matching: normalize input extension to lowercase before lookup
  - [x] 5.3 Implement compound extension support: match `"test.ts"` against `foo.test.ts` with higher priority than plain `.ts` match
  - [x] 5.4 Implement single-match resolution: return the matching `language_id` when exactly one definition matches
  - [x] 5.5 Implement multi-match resolution: when multiple definitions match the same extension, use highest priority value, falling back to alphabetical `language_id` order
  - [x] 5.6 Implement no-match fallback: return `LanguageId::plain_text()` when no definition matches
  - [x] 5.7 Implement manual override: `set_language_override(doc_id, language_id)` method to force a specific language for a document
  - [x] 5.8 Write unit tests for case-insensitive matching, compound extension priority, single match, multi-match priority resolution, no-match plain text, manual override
  - Covers: Requirement 2 (AC 2.1–2.7)

- [x] 6. Content-based language detection
  - [x] 6.1 Implement `ContentDetector` struct with detection pipeline: magic bytes → shebang → first-line pattern
  - [x] 6.2 Implement magic-byte detection: compare first N bytes against `magic_bytes` patterns from all loaded definitions
  - [x] 6.3 Implement shebang detection: if first line starts with `#!`, extract interpreter name and match against `shebang_patterns` in definitions
  - [x] 6.4 Implement first-line pattern detection: match first line against `first_line_pattern` regex from definitions
  - [x] 6.5 Implement priority ordering: magic bytes (highest) → shebang → first-line pattern (lowest), first successful match wins
  - [x] 6.6 Implement byte limit: content-based detection inspects only the first 8192 bytes of the file
  - [x] 6.7 Implement fallback: return `LanguageId::plain_text()` when content detection also fails
  - [x] 6.8 Write unit tests for shebang parsing (various formats), magic byte matching, first-line regex, priority ordering, 8192-byte limit, fallback to plain text
  - Covers: Requirement 3 (AC 3.1–3.7)

- [x] 7. Multi-line lexer state persistence
  - [x] 7.1 Define `LexerStateVector` struct wrapping `Vec<i32>` for per-line state storage, with `INVALID_STATE` sentinel constant
  - [x] 7.2 Implement `LexerStateVector::new(line_count: usize) -> Self` initializing all entries to `INVALID_STATE`
  - [x] 7.3 Implement `set_end_state(&mut self, line: usize, state: i32)` storing the end-of-line lexer state
  - [x] 7.4 Implement `get_start_state(&self, line: usize) -> i32` returning the state from line-1, or 0 (initial state) if line is 0
  - [x] 7.5 Implement `invalidate_from(&mut self, line: usize)` marking the specified line's state as invalid
  - [x] 7.6 Implement incremental termination check: `should_continue(&self, line: usize, new_state: i32) -> bool` returning false when new_state equals the previously stored state for that line
  - [x] 7.7 Implement `insert_lines(&mut self, at: usize, count: usize)` inserting `INVALID_STATE` entries and invalidating the following line
  - [x] 7.8 Implement `delete_lines(&mut self, at: usize, count: usize)` removing entries and invalidating the line following the deletion point
  - [x] 7.9 Write unit tests for state get/set, invalidation propagation, incremental termination, insert/delete line operations, large document (millions of lines) allocation
  - Covers: Requirement 4 (AC 4.1–4.8)

- [x] 8. Comment and string syntax definitions
  - [x] 8.1 Define `CommentSyntax` struct with `line_comments: Vec<String>`, `block_comment_start: Option<String>`, `block_comment_end: Option<String>`
  - [x] 8.2 Define `StringSyntax` struct with `delimiters: Vec<String>`, `character_delimiter: Option<String>`, `escape_character: Option<char>`, `heredoc_patterns: Vec<String>`
  - [x] 8.3 Implement `LanguageDefinition::comment_syntax(&self) -> &CommentSyntax` read-only accessor
  - [x] 8.4 Implement `LanguageDefinition::string_syntax(&self) -> &StringSyntax` read-only accessor
  - [x] 8.5 Implement support for multiple line-comment styles per language (single string or array of strings in TOML)
  - [x] 8.6 Write unit tests for single line comment, multiple line comments, block comment pairs, string delimiters, escape character, heredoc patterns, read-only access
  - Covers: Requirement 6 (AC 6.1–6.7)

- [x] 9. Embedded language support
  - [x] 9.1 Implement `EmbeddedLanguageResolver` struct with methods to resolve embedded language transitions
  - [x] 9.2 Implement `resolve_embedded(&self, language_id: &LanguageId) -> Option<&LanguageDefinition>` to look up embedded language definition from registry
  - [x] 9.3 Implement nesting support: track nesting depth up to 3 levels (host → embedded → nested embedded)
  - [x] 9.4 Implement state encoding: per-line state for embedded transitions encodes both host state and embedded language identity
  - [x] 9.5 Implement missing embedded language handling: emit WARN log and treat region as unstyled text when referenced `language_id` is not registered
  - [x] 9.6 Write unit tests for embedded language resolution, 3-level nesting, state encoding/decoding, missing language fallback
  - Covers: Requirement 7 (AC 7.1–7.6)

- [x] 10. Language property configuration
  - [x] 10.1 Implement `PropertyStore` struct with layered lookup: user/project overrides → definition built-in properties
  - [x] 10.2 Implement `get_property(language_id: &LanguageId, key: &str) -> Option<String>` with layered resolution
  - [x] 10.3 Implement `get_property_int(language_id: &LanguageId, key: &str, default: i64) -> i64` parsing value as integer with fallback
  - [x] 10.4 Implement `get_property_bool(language_id: &LanguageId, key: &str, default: bool) -> bool` parsing "1"/"true"/"yes" → true, "0"/"false"/"no" → false, else default
  - [x] 10.5 Implement hot-reload support: when configuration-system reloads a language profile, update properties and signal re-highlight needed
  - [x] 10.6 Implement override key path: `languages.{language_id}.{property_key}` following the configuration-system layered model
  - [x] 10.7 Write unit tests for property retrieval, int parsing, bool parsing, layered override precedence, hot-reload update, invalid value fallback
  - Covers: Requirement 8 (AC 8.1–8.6)

- [x] 11. Plugin-extensible language registration
  - [x] 11.1 Implement `LanguageRegistry` struct as the central registry owning all loaded `LanguageDefinition` instances with thread-safe interior mutability
  - [x] 11.2 Implement `register(&self, definition: LanguageDefinition, source: RegistrationSource) -> Result<(), RegistrationError>` validating schema and adding to registry
  - [x] 11.3 Implement validation: required fields present, keyword sets well-formed, reject registration if `language_id` already exists
  - [x] 11.4 Implement `deregister(&self, language_id: &LanguageId, source: RegistrationSource) -> Result<(), RegistrationError>` removing plugin-registered definitions and emitting DEBUG log
  - [x] 11.5 Define `RegistrationSource` enum: BuiltIn, UserConfig, ProjectConfig, Plugin(PluginId) to track definition origin
  - [x] 11.6 Implement deregistration notification: signal syntax-highlighting engine to fall back to plain text for documents using the removed language
  - [x] 11.7 Implement `LanguageSupport` capability advertisement interface for plugin-architecture integration
  - [x] 11.8 Write unit tests for successful registration, duplicate rejection, schema validation failure, deregistration and fallback notification, source tracking
  - Covers: Requirement 9 (AC 9.1–9.7)

- [x] 12. Language service query API
  - [x] 12.1 Implement `LanguageService` struct composing `LanguageRegistry`, `ExtensionMatcher`, `ContentDetector`, `PropertyStore`, and `LexerStateVector` per document
  - [x] 12.2 Implement `list_languages(&self) -> Vec<LanguageSummary>` returning all registered languages with id, name, and extensions
  - [x] 12.3 Implement `get_definition(&self, language_id: &LanguageId) -> Option<&LanguageDefinition>` returning immutable reference to full definition
  - [x] 12.4 Implement `detect_language(&self, file_path: &Path, first_line: Option<&str>, first_bytes: Option<&[u8]>) -> LanguageId` performing full detection pipeline (extension → content-based)
  - [x] 12.5 Implement `extensions_for(&self, language_id: &LanguageId) -> &[String]` returning extension list for a language
  - [x] 12.6 Implement `language_for_extension(&self, extension: &str) -> Option<LanguageId>` performing extension-only lookup without content fallback
  - [x] 12.7 Ensure all query methods use `&self` (interior immutability) with appropriate synchronization for thread safety
  - [x] 12.8 Implement testability: `LanguageService::from_definitions(defs: Vec<LanguageDefinition>) -> Self` constructor for unit testing without filesystem
  - [x] 12.9 Write unit tests for list_languages, get_definition, detect_language pipeline, extensions_for, language_for_extension, thread-safe concurrent access, testability constructor
  - Covers: Requirement 10 (AC 10.1–10.7)

- [x] 13. Error handling
  - [x] 13.1 Define `LanguageServiceError` enum: TomlParseError, SchemaValidationError, DuplicateLanguageId, RegistrationRejected, PropertyParseError, InvalidLanguageId, IoError
  - [x] 13.2 Implement error message formatting per `[language-service] operation: description` standard (≤200 chars)
  - [x] 13.3 Implement graceful degradation: invalid definitions skip with WARN log, invalid properties return defaults, unresolved embedded languages treated as plain text
  - [x] 13.4 Write unit tests for all error variants and message formatting
  - Covers: Cross-cutting error handling for all requirements

- [x] 14. Property-based tests
  - [x] 14.1 Write PBT: keyword lookup correctness
  - [x] 14.2 Write PBT: extension matching case-insensitivity
  - [x] 14.3 Write PBT: lexer state vector insert/delete consistency
  - [x] 14.4 Write PBT: content detection priority ordering
  - [x] 14.5 Write PBT: property boolean parsing correctness
  - [x] 14.6 Write PBT: language registration idempotency
  - [x] 14.7 Write PBT: compound extension priority over simple extension
  - Covers: Requirements 2–5, 8, 9 (see Property-Based Test Definitions below)

- [x] 15. Integration tests
  - [x] 15.1 Write integration test: full startup lifecycle — load TOML definitions from test directory → build registry → detect language for sample files
  - [x] 15.2 Write integration test: multi-directory override — built-in definition overridden by user config directory for same language_id
  - [x] 15.3 Write integration test: content-based detection pipeline — extensionless file with shebang correctly identified
  - [x] 15.4 Write integration test: lexer state management — edit cycle with insert/delete lines and incremental re-highlighting termination
  - [x] 15.5 Write integration test: plugin registration and deregistration lifecycle with fallback to plain text
  - [x] 15.6 Write integration test: property hot-reload — change language profile file and verify updated property values
  - Covers: End-to-end validation across Requirements 1–10

---

## Property-Based Test Definitions

### Property 1: Keyword Lookup Correctness

**Validates: Requirements 5.3, 5.4, 5.5**

- **Statement:** For any sorted keyword set and any query word, `contains(word)` SHALL return true if and only if the word is present in the set. Case-sensitive lookup SHALL only match exact case; case-insensitive lookup SHALL match regardless of input casing when the keyword exists in lowercased form.
- **Strategy:** Generate:
  - keywords: Vec<String> of 1–50 random alphanumeric strings (length 1–20)
  - query: either a random element from the set (hit) or a random string not in the set (miss)
  - case_variant: random upper/lower/mixed casing of the query
- **Invariant:** `contains(query) == keywords.contains(&query)` AND for case-insensitive: `contains_ci(variant) == keywords.iter().any(|k| k.to_lowercase() == variant.to_lowercase())`

### Property 2: Extension Matching Case-Insensitivity

**Validates: Requirements 2.1, 2.2**

- **Statement:** For any file extension string and any casing variant of that string, the extension matcher SHALL produce the same detected language. Extension matching is case-insensitive.
- **Strategy:** Generate:
  - extension: random ASCII string of length 1–10 (lowercase)
  - casing: random mixed-case variant of extension (e.g., "rs" → "Rs", "RS", "rS")
  - definitions: 1–5 random LanguageDefinitions with known extensions
- **Invariant:** `match(extension.to_lowercase()) == match(casing)` for all casing variants

### Property 3: Lexer State Vector Insert/Delete Consistency

**Validates: Requirements 4.6, 4.7**

- **Statement:** For any sequence of insert and delete line operations on a LexerStateVector, the vector length SHALL always equal the initial length plus total insertions minus total deletions. After any insert, newly inserted entries SHALL be INVALID_STATE. After any delete, the line following the deletion point SHALL be invalidated.
- **Strategy:** Generate:
  - initial_size: usize in [1, 1000]
  - operations: Vec of 1–50 random (Insert(at, count) | Delete(at, count)) operations with valid bounds
- **Invariant:** `vec.len() == initial_size + total_inserted - total_deleted` AND inserted entries are INVALID_STATE AND post-delete adjacent line is invalidated

### Property 4: Content Detection Priority Ordering

**Validates: Requirements 3.5**

- **Statement:** When a file matches multiple content-detection rules (magic bytes AND shebang AND first-line pattern), the detection result SHALL always be the magic-bytes match. When only shebang and first-line pattern match, the result SHALL be the shebang match. Magic bytes > shebang > first-line pattern.
- **Strategy:** Generate:
  - file_content: bytes where first 4 bytes match a magic_bytes pattern AND first line has a shebang AND first line matches a first_line_pattern, each pointing to different language_ids
- **Invariant:** `detect(content) == magic_bytes_language` when all three match. `detect(content_without_magic) == shebang_language` when shebang and first-line both match.

### Property 5: Property Boolean Parsing Correctness

**Validates: Requirements 8.4**

- **Statement:** For any property value string, `get_property_bool` SHALL return true for "1", "true", "yes" (case-insensitive), false for "0", "false", "no" (case-insensitive), and the provided default for any other string or absent key.
- **Strategy:** Generate:
  - value: random string from {"1", "true", "yes", "TRUE", "Yes", "0", "false", "no", "FALSE", "No", "maybe", "2", "", random garbage}
  - default: random bool
- **Invariant:** Result matches expected truth table for known values, returns default for unknown values

### Property 6: Language Registration Idempotency

**Validates: Requirements 9.3**

- **Statement:** Attempting to register a LanguageDefinition with a `language_id` that already exists SHALL always fail with a RegistrationRejected error. The existing definition SHALL remain unchanged after the failed registration attempt.
- **Strategy:** Generate:
  - definition_a: random valid LanguageDefinition
  - definition_b: different LanguageDefinition with same language_id as definition_a
- **Invariant:** `register(a)` succeeds, `register(b)` returns Err(RegistrationRejected), `get_definition(id) == Some(&a)` unchanged

### Property 7: Compound Extension Priority Over Simple Extension

**Validates: Requirements 2.3, 2.6**

- **Statement:** When both a compound extension (e.g., "test.ts") and a simple extension (e.g., "ts") match a filename, the compound extension match SHALL have higher priority. A file named `foo.test.ts` SHALL resolve to the language with `extensions = ["test.ts"]` over one with `extensions = ["ts"]`.
- **Strategy:** Generate:
  - base_ext: random 2–4 char extension
  - compound_prefix: random 2–8 char prefix
  - compound_ext: `{compound_prefix}.{base_ext}`
  - filename: `{random_name}.{compound_ext}`
  - lang_simple: definition with extensions = [base_ext]
  - lang_compound: definition with extensions = [compound_ext]
- **Invariant:** `detect(filename) == lang_compound.language_id` regardless of registration order or priority values

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "3", "13"], "dependsOn": [0] },
    { "id": 2, "label": "Loading and Detection", "tasks": ["4", "5", "6"], "dependsOn": [1] },
    { "id": 3, "label": "State and Syntax", "tasks": ["7", "8", "9"], "dependsOn": [1] },
    { "id": 4, "label": "Configuration and Registration", "tasks": ["10", "11"], "dependsOn": [2, 3] },
    { "id": 5, "label": "Public API", "tasks": ["12"], "dependsOn": [4] },
    { "id": 6, "label": "Validation", "tasks": ["14", "15"], "dependsOn": [5] }
  ]
}
```

---

## Notes

- This is a Wave 7 (Language and Highlighting) crate that is **GUI-independent** — no rendering framework dependency.
- The language service owns language definitions and detection logic but does NOT perform tokenization — that responsibility belongs to `ff-syntax-highlighting`.
- Thread safety is a strict requirement: all public query methods use `&self` with interior immutability (e.g., `RwLock<HashMap<...>>`) so multiple subsystems can query concurrently.
- TOML is the definition format for language files. The `toml` crate handles deserialization; `serde` derives handle struct mapping.
- The `LexerStateVector` uses `Vec<i32>` for compact per-line state storage. The `INVALID_STATE` sentinel (e.g., `i32::MIN`) marks lines needing re-highlighting.
- Plugin-registered languages have full feature parity with file-loaded definitions — same schema, same keyword sets, same detection rules.
- Content-based detection is bounded to 8192 bytes to avoid scanning large binary files.
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property.
- Hot-reload of language properties leverages the configuration-system file watcher. When a language profile changes, affected documents are flagged for re-highlighting.
- The `LanguageService` is constructable from a `Vec<LanguageDefinition>` for unit testing without any filesystem or running application.

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Language Definition Loading from TOML | AC 1.1–1.7 | Tasks 2, 4 |
| Req 2: Language Detection by File Extension | AC 2.1–2.7 | Task 5 |
| Req 3: Content-Based Language Detection | AC 3.1–3.7 | Task 6 |
| Req 4: Multi-Line Lexer State Persistence | AC 4.1–4.8 | Task 7 |
| Req 5: Keyword Lists | AC 5.1–5.7 | Task 3 |
| Req 6: Comment and String Syntax Definitions | AC 6.1–6.7 | Tasks 2, 8 |
| Req 7: Embedded Languages (Sub-Languages) | AC 7.1–7.6 | Tasks 2, 9 |
| Req 8: Language Property Configuration | AC 8.1–8.6 | Task 10 |
| Req 9: Plugin-Extensible Language Registration | AC 9.1–9.7 | Task 11 |
| Req 10: Language Service Query API | AC 10.1–10.7 | Task 12 |
| Cross-cutting: Error Handling | All | Task 13 |
