# Implementation Plan: `ff-encoding` Crate

## Overview

Implement the encoding and character-handling subsystem (`ff-encoding` crate) for FileForgeWorkbench. This crate provides encoding detection, BOM handling, encoding conversion, UTF-8 validation, character classification, Unicode category mapping, DBCS support, case folding, grapheme cluster detection, and word-part navigation.

The implementation follows a bottom-up strategy: foundational types and utilities first, then detection/conversion, then classification/navigation, and finally comprehensive testing.

**Source:** `.kiro/specs/encoding-and-characters/requirements.md` (Requirements 1–14)
**Design:** `.kiro/specs/encoding-and-characters/design.md`

---

## Tasks

- [ ] 1. Crate scaffolding, core types, and Encoding Registry
  - [ ] 1.1 Create `crates/ff-encoding/Cargo.toml` with dependencies (`thiserror`, dev: `proptest`, `pretty_assertions`)
  - [ ] 1.2 Create `src/lib.rs` with crate-level documentation and public re-exports (placeholder modules)
  - [ ] 1.3 Implement `src/error.rs` — `EncodingError` enum with all variants per design (thiserror derive)
  - [ ] 1.4 Implement `src/encoding.rs` — `Encoding` struct, `EncodingFamily` enum (`SingleByte`, `Utf8`, `Dbcs`, `Utf16`), `EncodingMetadata` struct
  - [ ] 1.5 Implement `src/registry.rs` — `EncodingRegistry` with lookup by name, code page, and alias; pre-populated with all required encodings (UTF-8, UTF-16LE/BE, UTF-32LE/BE, ISO-8859-1–15, Windows-1250–1258, Shift-JIS, GBK, EUC-KR, Big5, EUC-JP, EBCDIC CP037/CP500/CP1047)
  - [ ] 1.6 Implement `encoding_family(code_page)` function returning correct `EncodingFamily` for any registered code page
  - [ ] 1.7 Write unit tests for encoding types, registry lookup, family classification
    - Validates: Requirements 11.1, 11.2, 14.7, 14.8

- [ ] 2. BOM detection and writing
  - [ ] 2.1 Implement `src/bom.rs` — `BomInfo` struct, `BomEncoding` enum
  - [ ] 2.2 Implement `detect_bom(bytes)` — checks UTF-32 4-byte BOMs before UTF-16 2-byte to disambiguate UTF-32LE from UTF-16LE+NUL
  - [ ] 2.3 Implement `bom_bytes(encoding)` — returns static BOM byte slice for each BomEncoding
  - [ ] 2.4 Implement `write_bom(encoding, writer)` — writes BOM to a `std::io::Write`
  - [ ] 2.5 Write unit tests for BOM detection (all 5 encodings), disambiguation (UTF-32LE vs UTF-16LE), no-BOM case, and BOM writing
    - Validates: Requirements 2.1, 2.2, 2.3, 2.5, 2.6, 2.7, 2.8

- [ ] 3. Encoding detection (heuristic + BOM cascade)
  - [ ] 3.1 Implement `src/detect.rs` — `DetectionResult`, `DetectionConfidence` types
  - [ ] 3.2 Implement BOM-first detection strategy (delegates to `detect_bom`)
  - [ ] 3.3 Implement UTF-8 validity check — scan bytes for RFC 3629 conformance, classify as UTF-8 if valid
  - [ ] 3.4 Implement null-byte pattern analysis for UTF-16/UTF-32 detection (alternating nulls, triple nulls)
  - [ ] 3.5 Implement DBCS lead/trail byte pattern analysis for Shift-JIS, GBK, Big5, EUC-KR heuristics
  - [ ] 3.6 Implement statistical byte-frequency heuristics and configurable fallback
  - [ ] 3.7 Implement `detect_encoding(bytes, max_bytes)` composing all strategies in priority order
  - [ ] 3.8 Implement `detect_encoding_with_fallback(bytes, max_bytes, fallback)` for explicit fallback override
  - [ ] 3.9 Write unit tests for detection with known samples (BOM-marked, valid UTF-8, DBCS, Latin-1, EBCDIC)
    - Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8

- [ ] 4. UTF-8 validation and repair
  - [ ] 4.1 Implement `src/utf8.rs` — `utf8_byte_length_from_lead(byte)` returning 1–4 or 1 for invalid leads
  - [ ] 4.2 Implement `utf8_classify(bytes)` — returns (byte_length, is_valid) for first character
  - [ ] 4.3 Implement `utf8_validate(bytes)` — full RFC 3629 validation (rejects overlongs, surrogates, >U+10FFFF)
  - [ ] 4.4 Implement `utf8_fix_invalid(bytes)` — replace invalid sequences with U+FFFD, preserve valid content
  - [ ] 4.5 Write unit tests for validation (valid sequences, overlongs, surrogates, boundary cases), repair, and lead-byte classification
    - Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7

- [ ] 5. Encoding conversion (to/from UTF-8, streaming decoder/encoder)
  - [ ] 5.1 Implement `src/convert.rs` — `ConversionResult`, `ConversionIssue`, `UnmappableAction` types
  - [ ] 5.2 Implement single-byte encoding conversion tables (ISO-8859-x, Windows-125x, EBCDIC → Unicode mapping arrays)
  - [ ] 5.3 Implement `convert_to_utf8(bytes, source_encoding)` — full conversion with U+FFFD replacement and issue logging
  - [ ] 5.4 Implement UTF-16LE/BE and UTF-32LE/BE to UTF-8 conversion (handle surrogate pairs, supplementary planes)
  - [ ] 5.5 Implement DBCS-to-UTF-8 conversion using code-page-specific mapping tables
  - [ ] 5.6 Implement `convert_from_utf8(text, target_encoding, unmappable_action)` — UTF-8 to target encoding
  - [ ] 5.7 Implement `StreamDecoder` for chunk-based decoding (maintains state across chunk boundaries)
  - [ ] 5.8 Implement `StreamEncoder` for chunk-based encoding (maintains state for multi-byte character splits)
  - [ ] 5.9 Write unit tests for load conversion (each encoding family), save conversion, unmappable handling, streaming with split multi-byte chars
    - Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8

- [ ] 6. CharClassify (256-byte table, configurable word chars)
  - [ ] 6.1 Implement `src/classify.rs` — `CharacterClass` enum, `CharClassify` struct with 256-entry array
  - [ ] 6.2 Implement `CharClassify::new(include_word_class)` with default classification rules
  - [ ] 6.3 Implement `classify(byte)`, `is_word(byte)` — O(1) lookup methods
  - [ ] 6.4 Implement `set_char_classes(chars, class)`, `set_word_chars`, `set_whitespace_chars`, `set_punctuation_chars`, `reset_word_chars`
  - [ ] 6.5 Implement `get_chars_of_class(class)` — return all byte values for a given class
  - [ ] 6.6 Write unit tests for default classification, custom word chars, reset, class enumeration
    - Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 13.1, 13.2, 13.3, 13.4, 13.5

- [ ] 7. CharacterCategoryMap (Unicode General Category, UAX #31)
  - [ ] 7.1 Create `data/UnicodeData.txt` placeholder and `build.rs` script to generate category tables at build time
  - [ ] 7.2 Implement `src/category_map.rs` — `CharacterCategory` enum (30 categories), `CharacterCategoryMap` struct
  - [ ] 7.3 Implement dense BMP array (U+0000–U+FFFF) and sorted range binary search for supplementary planes
  - [ ] 7.4 Implement `category_for(code_point)` — O(1) BMP lookup, O(log n) supplementary lookup
  - [ ] 7.5 Implement `optimize(count_characters)` — pre-allocate dense array up to specified range
  - [ ] 7.6 Implement `is_id_start`, `is_id_continue`, `is_xid_start`, `is_xid_continue` predicates (UAX #31)
  - [ ] 7.7 Implement `is_word_char(code_point)` — true for categories L*, Nd, Nl, Pc
  - [ ] 7.8 Write unit tests for known code points (ASCII letters, CJK, Cyrillic, emoji), identifier predicates, word-char classification
    - Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7

- [ ] 8. DBCS code pages (lead/trail byte, safe segmentation)
  - [ ] 8.1 Implement `src/dbcs.rs` — `DbcsCodePage` enum, `DbcsCodePageDef` struct with lead/trail byte ranges for all 5 code pages
  - [ ] 8.2 Implement `is_dbcs_code_page(code_page)` — returns true for CP932, CP936, CP949, CP950, CP1361
  - [ ] 8.3 Implement `dbcs_is_lead_byte(code_page, byte)` — range-based lookup per code page
  - [ ] 8.4 Implement `dbcs_is_trail_byte(code_page, byte)` — range-based lookup per code page
  - [ ] 8.5 Implement `is_dbcs_valid_single_byte(code_page, byte)` — half-width katakana etc.
  - [ ] 8.6 Implement `safe_segment(data, code_page)` — return longest prefix ending on character boundary
  - [ ] 8.7 Implement `DBCSFoldMap` per code page for case-insensitive search in DBCS content
  - [ ] 8.8 Write unit tests for each code page's lead/trail byte ranges, safe segmentation, fold maps
    - Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8

- [ ] 9. Case folding (fold/upper/lower, ICaseConverter trait)
  - [ ] 9.1 Create `data/CaseFolding.txt` placeholder and extend `build.rs` to generate case-folding tables at build time
  - [ ] 9.2 Implement `src/case_fold.rs` — `CaseMode` enum, `CaseFoldResult` struct, `ICaseConverter` trait
  - [ ] 9.3 Implement `CaseFolder::new()` with compiled static data tables
  - [ ] 9.4 Implement `case_convert(code_point, mode)` — single code point conversion with multi-byte expansion support
  - [ ] 9.5 Implement `case_convert_string(text, mode)` — full string conversion handling expansions (ß→ss, ﬁ→fi)
  - [ ] 9.6 Implement `ICaseConverter` for `CaseFolder` (trait impl for find-and-replace integration)
  - [ ] 9.7 Write unit tests for fold (ß→ss, ﬁ→fi, ΐ), upper/lower ASCII+Unicode, multi-char expansion, trait usage
    - Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8

- [ ] 10. Grapheme cluster boundaries (UAX #29)
  - [ ] 10.1 Create `data/GraphemeBreakProperty.txt` placeholder and extend `build.rs` for grapheme break property tables
  - [ ] 10.2 Implement `src/grapheme.rs` — `GraphemeMode` enum, `GraphemeIterator` struct
  - [ ] 10.3 Implement UAX #29 grapheme break rules state machine (GB1–GB999 rules)
  - [ ] 10.4 Implement `is_grapheme_boundary(text, byte_offset)` — boundary test at position
  - [ ] 10.5 Implement `next_grapheme_boundary(text, byte_offset)` and `prev_grapheme_boundary(text, byte_offset)`
  - [ ] 10.6 Handle combining marks, emoji modifiers, ZWJ sequences, regional indicators, Hangul syllables
  - [ ] 10.7 Implement simplified mode (code-point-level only, for large files)
  - [ ] 10.8 Implement `GraphemeIterator` yielding grapheme cluster string slices
  - [ ] 10.9 Write unit tests for combining marks (é), emoji ZWJ (👨‍👩‍👧), flags (🇺🇸), Hangul, simplified mode
    - Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8

- [ ] 11. Word-part navigation (camelCase/snake_case sub-word)
  - [ ] 11.1 Implement `src/word_part.rs` — `is_word_part_separator(code_point)` predicate
  - [ ] 11.2 Implement `word_part_left(text, position, classify)` — find previous word-part boundary
  - [ ] 11.3 Implement `word_part_right(text, position, classify)` — find next word-part boundary
  - [ ] 11.4 Handle camelCase transitions (lower→upper), PascalCase runs (uppercase sequence→lowercase), underscore boundaries, letter↔digit transitions
  - [ ] 11.5 Respect CharClassify — non-Word characters act as hard boundaries
  - [ ] 11.6 Support Unicode Lu/Ll transitions (not just ASCII A-Z/a-z)
  - [ ] 11.7 Write unit tests for camelCase (`getDocumentModel`), snake_case (`get_document_model`), PascalCase (`XMLParser`), digit transitions (`line42count`), Unicode identifiers
    - Validates: Requirements 12.1, 12.2, 12.3, 12.4, 12.5, 12.6

- [ ] 12. Property-based tests
  - [ ] 12.1 Create `tests/property_tests.rs` with proptest framework setup
  - [ ] 12.2 Property 1: Encoding roundtrip preservation — convert_to_utf8(convert_from_utf8(text, enc)) == text for mappable text
    - **Validates: Requirements 3, 4**
  - [ ] 12.3 Property 2: BOM detection accuracy — prepending BOM bytes always yields correct detection
    - **Validates: Requirements 2.1, 2.2, 2.3**
  - [ ] 12.4 Property 3: Case fold idempotence — folding an already-folded string yields the same result
    - **Validates: Requirements 10.1, 10.4, 10.6**
  - [ ] 12.5 Property 4: UTF-8 validation consistency — utf8_validate agrees with std::str::from_utf8
    - **Validates: Requirements 5.1, 5.4, 5.5**
  - [ ] 12.6 Property 5: UTF-8 fix produces valid UTF-8 — utf8_fix_invalid output is always valid
    - **Validates: Requirements 5.3**
  - [ ] 12.7 Property 6: CharClassify completeness — every byte 0–255 has exactly one class
    - **Validates: Requirements 6.1**
  - [ ] 12.8 Property 7: Grapheme boundary monotonicity — next always advances, prev always retreats
    - **Validates: Requirements 9.2, 9.3, 9.4**
  - [ ] 12.9 Property 8: DBCS lead+trail byte disjointness — no ASCII byte is a lead byte
    - **Validates: Requirements 8.2, 8.3**
  - [ ] 12.10 Property 9: Word-part navigation termination — left always ≤ pos, right always ≥ pos
    - **Validates: Requirements 12.2, 12.3**
  - [ ] 12.11 Property 10: Encoding family consistency — DBCS code pages map to Dbcs family
    - **Validates: Requirements 11.1, 11.2**

- [ ] 13. Integration tests
  - [ ] 13.1 Create `tests/detect_tests.rs` — end-to-end encoding detection with reference files (UTF-8, UTF-16LE with BOM, Shift-JIS, Latin-1, EBCDIC)
  - [ ] 13.2 Create `tests/convert_tests.rs` — roundtrip conversion of known reference files through load+save pipeline
  - [ ] 13.3 Create `tests/classify_tests.rs` — CharClassify + CharacterCategoryMap combined word-boundary scenarios
  - [ ] 13.4 Create `tests/case_fold_tests.rs` — case folding integration with ICaseConverter trait consumers
  - [ ] 13.5 Create `tests/grapheme_tests.rs` — grapheme iteration over complex Unicode text (mixed scripts, emoji, combining marks)
  - [ ] 13.6 Create `tests/streaming_tests.rs` — StreamDecoder/StreamEncoder with multi-chunk input splitting at various byte boundaries
  - [ ] 13.7 Verify crate builds cleanly with `cargo clippy -- -D warnings` and `cargo test` passes

---

## Acceptance Criteria Coverage Map

| Requirement | Tasks |
|------------|-------|
| Req 1 (Encoding Detection) | 3.1–3.9, 12.2, 13.1 |
| Req 2 (BOM Detection) | 2.1–2.5, 12.3, 13.1 |
| Req 3 (Conversion on Load) | 5.1–5.9, 12.2, 13.2, 13.6 |
| Req 4 (Conversion on Save) | 5.6–5.9, 12.2, 13.2, 13.6 |
| Req 5 (UTF-8 Validation) | 4.1–4.5, 12.5, 12.6 |
| Req 6 (CharClassify) | 6.1–6.6, 12.7, 13.3 |
| Req 7 (CharacterCategoryMap) | 7.1–7.8, 13.3 |
| Req 8 (DBCS Support) | 8.1–8.8, 12.9, 13.2 |
| Req 9 (Grapheme Clusters) | 10.1–10.9, 12.8, 13.5 |
| Req 10 (Case Folding) | 9.1–9.7, 12.4, 13.4 |
| Req 11 (Encoding Family) | 1.4–1.7, 12.11 |
| Req 12 (Word-Part Navigation) | 11.1–11.7, 12.10 |
| Req 13 (Configurable Word Chars) | 6.4–6.6, 13.3 |
| Req 14 (Encoding Metadata/Registry) | 1.4–1.7 |

---

## Property-Based Test Definitions

| # | Property | Strategy | Requirement |
|---|----------|----------|-------------|
| 1 | Encoding roundtrip: `convert_to_utf8(convert_from_utf8(text, enc)) == text` for mappable text | ASCII + Latin-1 text × single-byte encodings | Req 3, 4 |
| 2 | BOM detection: prepend BOM bytes → detect correctly | 5 BOM types × random content | Req 2.1, 2.2, 2.3 |
| 3 | Case fold idempotence: `fold(fold(text)) == fold(text)` | Unicode strings with ß, ﬁ, ΐ | Req 10.1, 10.4, 10.6 |
| 4 | UTF-8 validation consistency: agrees with `std::str::from_utf8` | Random byte vectors 0–256 bytes | Req 5.1, 5.4, 5.5 |
| 5 | UTF-8 fix produces valid output: result is always valid UTF-8 | Arbitrary byte vectors | Req 5.3 |
| 6 | CharClassify completeness: all 256 bytes classified | Various configurations | Req 6.1 |
| 7 | Grapheme boundary monotonicity: next advances, prev retreats | Unicode with combining chars, emoji, Hangul | Req 9.2, 9.3, 9.4 |
| 8 | DBCS lead byte disjointness: ASCII bytes are never lead bytes | 5 code pages × bytes 0x00–0x7F | Req 8.2, 8.3 |
| 9 | Word-part termination: left ≤ pos, right ≥ pos | camelCase/snake_case/PascalCase identifiers | Req 12.2, 12.3 |
| 10 | Encoding family consistency: DBCS pages → Dbcs family | All registered encodings | Req 11.1, 11.2 |

---

## Notes

- Phases 1, 2, 4, 6, and 7 can proceed in parallel after Phase 1 completes — they have no inter-dependencies.
- Phase 3 (Detection) depends on Phase 2 (BOM) because BOM detection is the first strategy in the detection cascade.
- Phase 5 (Conversion) depends on Phase 4 (UTF-8) because conversion uses UTF-8 validation internally.
- Phase 8 (DBCS) depends on Phase 5 (Conversion) because DBCS conversion tables are integrated into the converter.
- Phases 9 and 10 depend on Phase 7 (CategoryMap) for Unicode property lookups.
- Phase 11 (Word-Part) depends on Phases 6 and 7 for CharClassify and Unicode category predicates.
- Phases 12 and 13 (testing) depend on all implementation phases being complete.
- Unicode data files (`UnicodeData.txt`, `CaseFolding.txt`, `GraphemeBreakProperty.txt`) are sourced from Unicode 15.1 and processed by `build.rs` at compile time.
- No external encoding crate dependencies — all tables are self-contained, derived from Scintilla's implementation.
- The `ff-config` dependency provides default encoding and BOM policy; if not yet available, use hardcoded defaults (UTF-8, no BOM) during early development.

---

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 1,
      "label": "Crate Scaffolding & Core Types",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7"],
      "dependsOn": []
    },
    {
      "id": 2,
      "label": "BOM Detection and Writing",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Encoding Detection",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9"],
      "dependsOn": [1, 2]
    },
    {
      "id": 4,
      "label": "UTF-8 Validation and Repair",
      "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5"],
      "dependsOn": [1]
    },
    {
      "id": 5,
      "label": "Encoding Conversion",
      "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9"],
      "dependsOn": [1, 4]
    },
    {
      "id": 6,
      "label": "CharClassify",
      "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6"],
      "dependsOn": [1]
    },
    {
      "id": 7,
      "label": "CharacterCategoryMap",
      "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8"],
      "dependsOn": [1]
    },
    {
      "id": 8,
      "label": "DBCS Code Pages",
      "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8"],
      "dependsOn": [1, 5]
    },
    {
      "id": 9,
      "label": "Case Folding",
      "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7"],
      "dependsOn": [1, 7]
    },
    {
      "id": 10,
      "label": "Grapheme Cluster Boundaries",
      "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9"],
      "dependsOn": [1, 7]
    },
    {
      "id": 11,
      "label": "Word-Part Navigation",
      "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7"],
      "dependsOn": [6, 7]
    },
    {
      "id": 12,
      "label": "Property-Based Tests",
      "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "12.9", "12.10", "12.11"],
      "dependsOn": [2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    },
    {
      "id": 13,
      "label": "Integration Tests",
      "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7"],
      "dependsOn": [2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    }
  ]
}
```
