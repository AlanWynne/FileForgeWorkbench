# Implementation Plan: `ff-fileforge` Crate

## Overview

Implement the FileForge Integration subsystem (`ff-fileforge` crate) for FileForgeWorkbench. This crate provides the data model, record parsing, field extraction, encoding conversion orchestration, and file I/O logic for structured flat-file processing. It enables FileForge_Mode — the workbench state where flat files are interpreted as structured tabular data using companion `.ffs` structure definitions.

The implementation follows a bottom-up strategy: foundational types and schema parsing first, then record format engines (FB/VB/VBS), encoding integration (EBCDIC, COMP-3), ASA carriage control, field navigation, mode management, command registration, and finally comprehensive testing.

**Source:** `.kiro/specs/fileforge-integration/requirements.md` (Requirements 1–16)

---

## Tasks

- [ ] 1. Crate scaffolding and core error types
  - [ ] 1.1 Create `crates/ff-fileforge/Cargo.toml` with dependencies (`thiserror`, `serde`, `serde_json`, dev: `proptest`, `pretty_assertions`, `tempfile`)
  - [ ] 1.2 Create `src/lib.rs` with crate-level documentation and public module re-exports (placeholder modules)
  - [ ] 1.3 Implement `src/error.rs` — `FileForgeError` enum with variants: `StructureParse`, `FieldValidation`, `FieldOverflow`, `InvalidRdw`, `EncodingError`, `IoError`, `EmptyFile`, `ResourceNotFound`, `UnsupportedOutputType`, `LreclDetectionFailed`
  - [ ] 1.4 Write unit tests for error Display impls and From conversions
    - Validates: Requirement 16


- [ ] 2. Record format types and structure schema
  - [ ] 2.1 Implement `src/record_format.rs` — `RecordFormat` enum (`F`, `FB`, `V`, `FbBinary`, `VB`, `FBA`, `VBA`, `U`) with case-insensitive parsing and uppercase normalisation on serialize
  - [ ] 2.2 Implement `src/field_def.rs` — `FieldDefinition` struct (`field_name`, `offset`, `length`, `data_type`, `decimals`, `identifiers`, `filters`) with `DataType` enum (`Str`, `Int`, `Float`, `Bool`, `Comp3`)
  - [ ] 2.3 Implement `src/record_structure.rs` — `RecordStructure` struct (name, ordered Vec of FieldDefinition, identifier field index)
  - [ ] 2.4 Implement `src/structure_file.rs` — `StructureFile` struct (version, lrecl, recfm, encoding, field_delimiter, Vec of RecordStructure) with serde deserialization
  - [ ] 2.5 Implement legacy key normalisation: accept `field_delimeter` as `field_delimiter` during deserialization
  - [ ] 2.6 Implement legacy data_type normalisation on load: `"<class 'str'>"` → `Str`, `"<class 'int'>"` → `Int`, `"<class 'float'>"` → `Float`, `"<class 'bool'>"` → `Bool`
  - [ ] 2.7 Implement structure validation: detect overlapping byte ranges within a RecordStructure, negative offsets, zero lengths, blank field names — report warnings without preventing load
  - [ ] 2.8 Implement version defaulting: files without `version` key treated as `"1.0"`
  - [ ] 2.9 Write unit tests for schema parsing, legacy normalisation, validation warnings, overlapping field detection
    - Validates: Requirement 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 9.8

- [ ] 3. EBCDIC codec integration
  - [ ] 3.1 Implement `src/ebcdic.rs` — `EbcdicCodePage` enum (`Cp037`, `Cp285`, `Cp500`, `Cp1047`) with mapping to `ff-encoding` codec identifiers
  - [ ] 3.2 Implement `decode_ebcdic_field(bytes, code_page)` — decode byte slice to Unicode String using specified code page, replacing unmappable bytes with `.`
  - [ ] 3.3 Implement `encode_ebcdic_field(text, code_page, field_length)` — encode Unicode to EBCDIC bytes, return error for unmappable characters
  - [ ] 3.4 Implement encoding routing: string/bool fields through EBCDIC decoder, numeric fields (int/float/comp3) treated as binary — bypass EBCDIC decoder
  - [ ] 3.5 Implement default encoding logic: `FB_BINARY` or `VB` without explicit encoding defaults to EBCDIC-037 with validation warning
  - [ ] 3.6 Write unit tests for EBCDIC encode/decode roundtrip, unmappable byte handling, numeric field bypass, default encoding fallback
    - Validates: Requirement 4.1, 4.2, 4.3, 4.4, 4.5, 4.8, 4.9

- [ ] 4. COMP-3 packed decimal engine
  - [ ] 4.1 Implement `src/comp3.rs` — `Comp3Value` struct wrapping decoded value with sign and decimal places
  - [ ] 4.2 Implement `decode_comp3(bytes)` — interpret BCD nibbles (high nibble first), extract sign from final low nibble (C=positive, D=negative, F=unsigned)
  - [ ] 4.3 Implement `format_comp3(value, decimals)` — apply implied decimal point (e.g., raw 1234567 with decimals=2 → "12345.67")
  - [ ] 4.4 Implement `encode_comp3(decimal_str, field_length)` — parse decimal string, pack into BCD bytes with sign nibble; reject if digit count exceeds field capacity
  - [ ] 4.5 Implement COMP-3 validation: detect invalid nibbles (>0x9 in digit position, invalid sign nibble) and return `FieldValidationError`
  - [ ] 4.6 Write unit tests for decode (positive, negative, unsigned), decimal placement, encode roundtrip, overflow rejection, invalid nibble detection
    - Validates: Requirement 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.10

- [ ] 5. VB record header parsing and binary reader
  - [ ] 5.1 Implement `src/vb_reader.rs` — `RdwHeader` struct (record_length: u16, reserved: [u8; 2])
  - [ ] 5.2 Implement `parse_rdw(bytes)` — extract big-endian u16 length from bytes 0–1, verify bytes 2–3 are 0x0000, return error if L < 4
  - [ ] 5.3 Implement `VbRecordIterator` — streaming iterator that reads RDW + content from a byte source, building byte-offset index as it reads
  - [ ] 5.4 Implement RDW error handling: stop on L < 4 or read-past-EOF, report structural error with byte offset and records-read count
  - [ ] 5.5 Implement VB index building: record byte-offset index (content start after RDW) for O(1) random access
  - [ ] 5.6 Implement VB record write-back: re-write record with updated RDW prefix when record length changes in Grid_Edit_Mode
  - [ ] 5.7 Implement VB/lrecl conflict detection: emit validation warning when both `recfm: "VB"` and `lrecl` are present; ignore lrecl for VB
  - [ ] 5.8 Write unit tests for RDW parsing, iterator with multi-record VB data, error on invalid RDW, index building, write-back with updated length
    - Validates: Requirement 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8

- [ ] 6. ASA carriage control detection and display
  - [ ] 6.1 Implement `src/asa.rs` — `AsaControl` enum (`SingleSpace`, `DoubleSpace`, `TripleSpace`, `NewPage`, `Overprint`, `Halt`) with display abbreviations (SP, DS, TS, NP, OP, HT)
  - [ ] 6.2 Implement `parse_asa_char(byte)` — map column-1 byte to `AsaControl` variant (space→SingleSpace, '0'→DoubleSpace, '-'→TripleSpace, '1'→NewPage, '+'→Overprint, 'H'→Halt)
  - [ ] 6.3 Implement `detect_asa(records, sample_size)` — sample first 20 non-blank lines, activate if ≥80% have known ASA characters in column 1
  - [ ] 6.4 Implement `strip_asa(records)` — remove column 1 from all records, shift content left by one byte; returns modified record set
  - [ ] 6.5 Implement ASA display mode state: `AsaDisplayMode` struct tracking on/off state and indicator per record
  - [ ] 6.6 Write unit tests for ASA character parsing, detection threshold (80%), strip operation, boundary cases (empty records, unknown chars)
    - Validates: Requirement 7.1, 7.2, 7.3, 7.4, 7.5, 7.8

- [ ] 7. Fixed-length record reader and byte-offset index
  - [ ] 7.1 Implement `src/fb_reader.rs` — `FbRecordReader` for RECFM F/FB files with known LRECL (O(1) direct position calculation: record_n offset = n × lrecl)
  - [ ] 7.2 Implement `src/byte_index.rs` — `ByteOffsetIndex` struct (Vec<u64> of record start offsets) with memory budget enforcement (≤100 MB for 10M records)
  - [ ] 7.3 Implement LRECL auto-detection: sample first 100 lines, check for uniform byte length; return detected value or indicate variable-length
  - [ ] 7.4 Implement variable-length fallback: when LRECL detection finds non-uniform lengths, build full byte-offset index by scanning newlines
  - [ ] 7.5 Implement progress reporting callback for index builds exceeding 2 seconds
  - [ ] 7.6 Write unit tests for FB direct seek, index building, LRECL auto-detect (uniform + non-uniform), memory budget check, progress callback trigger
    - Validates: Requirement 2.2, 2.3, 2.9, 2.10, 2.11, 10.1, 10.6

- [ ] 8. Record structure application and classification
  - [ ] 8.1 Implement `src/classifier.rs` — `RecordClassifier` struct that evaluates identifier fields against record bytes to determine Record_Structure assignment
  - [ ] 8.2 Implement first-match-wins semantics: iterate structures in definition order, apply first matching identifier
  - [ ] 8.3 Implement unclassified record handling: records matching no structure get `Unclassified` status with visual indicator data
  - [ ] 8.4 Implement filter list evaluation: when identifier field has non-empty filters list, exclude records whose type value is not in the list (status: `Filtered`)
  - [ ] 8.5 Implement classification statistics: `ClassificationStats` struct (total, per-type counts, skipped, filtered)
  - [ ] 8.6 Implement record-to-fields extraction: given a classified record and its RecordStructure, extract field byte slices by offset+length
  - [ ] 8.7 Write unit tests for single-type classification, multi-type first-match, unclassified records, filter exclusion, statistics aggregation
    - Validates: Requirement 13.1, 13.2, 13.3, 13.4, 13.5, 13.8, 14.5

- [ ] 9. Field display and value conversion
  - [ ] 9.1 Implement `src/field_display.rs` — `DisplayMode` enum (`Raw`, `Structured`, `Transformed`) and `FieldValue` enum for rendered cell content
  - [ ] 9.2 Implement field rendering pipeline: raw bytes → decode (EBCDIC if needed) → parse by data_type → format for display mode
  - [ ] 9.3 Implement Raw mode: display original byte content (hex for binary, decoded text for string)
  - [ ] 9.4 Implement Structured mode: display parsed field values (strings decoded, ints/floats as numbers, comp3 as decimal, bools as text)
  - [ ] 9.5 Implement Transformed mode: apply decimal/COMP-3 conversion, implied decimal points
  - [ ] 9.6 Implement field validation error display: invalid COMP-3 nibbles, un-decodable EBCDIC → show raw hex with error indicator
  - [ ] 9.7 Write unit tests for each display mode with string, int, float, bool, comp3 fields; error cases
    - Validates: Requirement 3.2, 5.3, 5.7, 9.9

- [ ] 10. Field validation engine
  - [ ] 10.1 Implement `src/field_validation.rs` — `FieldValidator` with validate method dispatching on DataType
  - [ ] 10.2 Implement int validation: optional leading sign + digits only
  - [ ] 10.3 Implement float validation: optional sign + digits + optional decimal point
  - [ ] 10.4 Implement bool validation: accept true/false/T/F/Y/N/1/0 (case-insensitive)
  - [ ] 10.5 Implement str validation: accept any input that fits field byte length when encoded
  - [ ] 10.6 Implement comp3 validation: accept decimal numeric input, verify packed representation fits field length
  - [ ] 10.7 Implement field-length overflow check: reject edits producing byte sequences longer than declared length
  - [ ] 10.8 Implement `decimals` handling in Transformed mode: accept decimal input, convert to packed integer (multiply by 10^N, round)
  - [ ] 10.9 Write unit tests for each data_type validation (valid + invalid inputs), overflow detection, decimal conversion
    - Validates: Requirement 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7

- [ ] 11. Field editing and encode-back
  - [ ] 11.1 Implement `src/field_edit.rs` — `FieldEdit` struct representing a pending edit (record_index, field_index, new_value_string)
  - [ ] 11.2 Implement edit-to-bytes pipeline: validate input → encode to target format (EBCDIC/COMP-3/binary) → verify fits field length → produce byte patch
  - [ ] 11.3 Implement EBCDIC re-encode on edit: accept Unicode input, encode to specified code page, error on unmappable characters
  - [ ] 11.4 Implement COMP-3 re-encode on edit: parse decimal input, pack to BCD bytes with sign nibble
  - [ ] 11.5 Implement byte patch application: update document buffer at correct byte offset with new field bytes
  - [ ] 11.6 Write unit tests for edit pipeline (str, int, float, comp3), overflow rejection, EBCDIC roundtrip, buffer patch correctness
    - Validates: Requirement 3.3, 3.4, 3.5, 4.4, 5.5, 5.6, 5.9

- [ ] 12. Record navigation engine
  - [ ] 12.1 Implement `src/navigation.rs` — `RecordNavigator` struct with current_record, total_records, window_start fields
  - [ ] 12.2 Implement go-to-record: O(1) seek via ByteOffsetIndex, position record at viewport top
  - [ ] 12.3 Implement page up/down: advance by window_size records
  - [ ] 12.4 Implement first/last record navigation
  - [ ] 12.5 Implement filtered navigation: skip non-matching records when type filter is active, navigate only among visible records
  - [ ] 12.6 Implement position reporting: current record number, total count, percentage position
  - [ ] 12.7 Write unit tests for direct seek, page navigation, first/last, filtered skip, position calculation
    - Validates: Requirement 10.1, 10.2, 10.3, 10.4, 10.5

- [ ] 13. Window management and record streaming
  - [ ] 13.1 Implement `src/window.rs` — `RecordWindow` struct (start_record, records: Vec<RecordData>, window_size)
  - [ ] 13.2 Implement window loading: VFS seek to byte offset, read window_size records, decode fields per active structures
  - [ ] 13.3 Implement configurable window size: default 200 records, configurable via workbench configuration system
  - [ ] 13.4 Implement on-demand window refresh: load new window when scroll/navigation moves beyond current window bounds
  - [ ] 13.5 Write unit tests for window load, boundary detection, configurable size, no full-file memory load verification
    - Validates: Requirement 2.7, 2.8

- [ ] 14. Record insert and delete operations
  - [ ] 14.1 Implement `src/record_ops.rs` — `RecordInsert` and `RecordDelete` operation structs
  - [ ] 14.2 Implement record insert: create new record initialised with spaces (or EBCDIC space equivalent) at LRECL length, position after current selection
  - [ ] 14.3 Implement record insert for VB: create record with RDW prefix, update VB index
  - [ ] 14.4 Implement record delete: remove record bytes from buffer, update ByteOffsetIndex
  - [ ] 14.5 Implement record delete for VB: remove RDW + content bytes
  - [ ] 14.6 Implement FB insert padding: ensure inserted record is exactly LRECL bytes, pad with spaces
  - [ ] 14.7 Implement bulk delete: support block selection with single operation struct
  - [ ] 14.8 Implement index update: refresh ByteOffsetIndex and total record count after insert/delete
  - [ ] 14.9 Write unit tests for insert (FB, VB), delete (FB, VB), bulk delete, index consistency after operations
    - Validates: Requirement 11.1, 11.2, 11.3, 11.5, 11.6, 11.7, 11.8

- [ ] 15. FileForge mode management
  - [ ] 15.1 Implement `src/mode.rs` — `FileForgeMode` struct tracking active state, structure_def reference, display mode, ASA state
  - [ ] 15.2 Implement auto-activation: detect companion `.ffs` file (same basename, same directory) on file open via VFS
  - [ ] 15.3 Implement Structure_Catalog association lookup: check File_Association_Map patterns when no companion file exists
  - [ ] 15.4 Implement activation precedence: companion `.ffs` > `.fc.json` > Structure_Catalog match
  - [ ] 15.5 Implement legacy `.fc.json` loading with backward compatibility rules (Req 1 criteria 7–8) and migration offer to `.ffs`
  - [ ] 15.6 Implement mode deactivation: return to standard text display, release structure resources
  - [ ] 15.7 Implement template generation: produce skeleton `.ffs` file with single empty RecordStructure and placeholder fields
  - [ ] 15.8 Implement structure file hot-reload: detect external `.ffs` modification via VFS file-watcher, offer to reload
  - [ ] 15.9 Write unit tests for auto-activation (companion found/not found), precedence order, legacy load, deactivation, template content
    - Validates: Requirement 2.1, 2.4, 2.5, 2.6, 8.1, 8.2, 8.3, 8.5, 8.6, 8.7, 8.8, 12.1, 12.2, 12.5, 12.7

- [ ] 16. Record type selection and filtering
  - [ ] 16.1 Implement `src/record_filter.rs` — `RecordTypeFilter` struct with active_type (Option<String>) and filter criteria composition
  - [ ] 16.2 Implement type filtering: when specific type selected, iterate only matching records; "All Types" shows all
  - [ ] 16.3 Implement filter composition: type filter AND Record_Selection_Criteria both must match for display
  - [ ] 16.4 Implement filtered record count and navigation position update on filter change
  - [ ] 16.5 Implement filter-active indicator data for status area display
  - [ ] 16.6 Write unit tests for type filter (single type, all types), composition with criteria, count update, indicator state
    - Validates: Requirement 14.1, 14.2, 14.3, 14.4, 14.5, 14.6, 14.7

- [ ] 17. Conversion and export engine
  - [ ] 17.1 Implement `src/convert.rs` — `OutputFormat` enum (`Csv`, `Tsv`, `Json`, `Dat`, `Txt`) and `ConversionResult` struct (records_read, records_written, records_skipped, records_filtered, output_path)
  - [ ] 17.2 Implement CSV/TSV export: iterate records, decode fields to Unicode (EBCDIC decode, COMP-3 format), write delimited UTF-8 output
  - [ ] 17.3 Implement JSON export: output array of record objects with field names as keys, decoded Unicode values
  - [ ] 17.4 Implement DAT/TXT fixed-width reconstruction: re-encode string fields to EBCDIC, COMP-3 fields to packed bytes, preserve original binary format
  - [ ] 17.5 Implement conversion progress reporting for async non-blocking operation
  - [ ] 17.6 Implement unsupported output type error with clear message
  - [ ] 17.7 Write unit tests for each export format, EBCDIC→Unicode in CSV, COMP-3→decimal in JSON, roundtrip DAT reconstruction
    - Validates: Requirement 4.6, 4.7, 5.8, 5.9, 15.1, 15.2, 15.3, 15.4, 16.5

- [ ] 18. Configuration integration
  - [ ] 18.1 Implement `src/config.rs` — `FileForgeConfig` struct (default_window_size, default_encoding, asa_auto_detect, lrecl_auto_detect)
  - [ ] 18.2 Implement configuration loading from workbench configuration system (ff-config crate dependency)
  - [ ] 18.3 Implement sensible defaults: window_size=200, encoding=utf-8, asa_auto_detect=true, lrecl_auto_detect=true
  - [ ] 18.4 Write unit tests for config defaults, config override application
    - Validates: Requirement 2.7, 4.9

- [ ] 19. Command registration
  - [ ] 19.1 Implement `src/commands.rs` — command handler functions for all FileForge commands
  - [ ] 19.2 Register `fileforge.convert` command: accepts optional OutputType argument, validates FileForge_Mode is active, triggers conversion engine
  - [ ] 19.3 Register `fileforge.validate` command: re-run structure validation, display warnings in status area
  - [ ] 19.4 Register `fileforge.export_config` command: export structure definition to CSV format (`<source_stem>_config.csv`)
  - [ ] 19.5 Register `fileforge.on` command: manually activate FileForge_Mode, open Structure_Catalog selector if no structure associated
  - [ ] 19.6 Register `fileforge.off` command: deactivate FileForge_Mode, return to standard text display
  - [ ] 19.7 Register `asa.on` and `asa.off` commands: toggle ASA_Display_Mode (not added to undo stack)
  - [ ] 19.8 Register `asa.strip` command: remove column 1 ASA characters from all records, shift left by one byte; single undoable transaction
  - [ ] 19.9 Implement command guard: `fileforge.convert` and `fileforge.export_config` error when FileForge_Mode is not active
  - [ ] 19.10 Write unit tests for command dispatch, guard errors, ASA toggle state, strip operation
    - Validates: Requirement 7.6, 7.7, 7.8, 8.4, 8.5, 15.1, 15.5, 15.6, 15.7, 15.8

- [ ] 20. Grid edit model (data layer)
  - [ ] 20.1 Implement `src/grid_model.rs` — `GridModel` struct representing the tabular view state (visible rows, columns per structure, display mode)
  - [ ] 20.2 Implement row-per-record mapping with 1-based record number column
  - [ ] 20.3 Implement multi-structure column handling: when "All Types" selected, show Record_Type label + raw content; when single type, show per-field columns
  - [ ] 20.4 Implement clipboard restrictions: refuse COPY in clipboard-paste/file-insert mode with clear error; permit in-document record copy preserving full byte content
  - [ ] 20.5 Implement record copy re-classification: copied records are re-classified using active RecordStructure
  - [ ] 20.6 Write unit tests for grid model state, column mapping, record number assignment, clipboard restriction, copy re-classification
    - Validates: Requirement 3.1, 3.6, 3.7, 3.8, 3.9, 3.10

- [ ] 21. Error handling and resilience
  - [ ] 21.1 Implement error path for resource-not-found (VFS URI in error message)
  - [ ] 21.2 Implement empty file handling: zero-byte files display message and open in standard text mode
  - [ ] 21.3 Implement structure parse error handling: display description, offer to open structure file for editing
  - [ ] 21.4 Implement I/O error propagation with affected resource path
  - [ ] 21.5 Implement VB structural error recovery: display records read before error, report byte offset of failure
  - [ ] 21.6 Implement EBCDIC per-field warning: unmappable bytes warn per field without aborting file display
  - [ ] 21.7 Write unit tests for each error path: not-found, empty, parse error, I/O error, VB error recovery, EBCDIC warning accumulation
    - Validates: Requirement 16.1, 16.2, 16.3, 16.4, 16.6, 16.7, 16.8

- [ ] 22. Structure file association logic
  - [ ] 22.1 Implement `src/association.rs` — `StructureAssociation` struct with resolution precedence logic
  - [ ] 22.2 Implement companion file search: (a) `<basename>.ffs` same dir, (b) `<basename>.fc.json` same dir, (c) Structure_Catalog pattern match
  - [ ] 22.3 Implement `.ffs` file write: serialize StructureFile to JSON with VFS write
  - [ ] 22.4 Implement external modification detection and reload offer via VFS file-watcher integration point
  - [ ] 22.5 Implement catalog association recording on user selection
  - [ ] 22.6 Write unit tests for precedence resolution, file write/read roundtrip, watcher integration point
    - Validates: Requirement 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7

- [ ] 23. Property-based tests
  - [ ] 23.1 Create `tests/property_tests.rs` with proptest framework setup
  - [ ] 23.2 Property 1: COMP-3 encode/decode roundtrip — for any valid decimal value that fits field length, `decode_comp3(encode_comp3(value)) == value`
    - **Validates: Requirements 5.2, 5.3, 5.5**
  - [ ] 23.3 Property 2: EBCDIC encode/decode roundtrip — for any string composed of mappable characters, `decode_ebcdic(encode_ebcdic(s, cp), cp) == s`
    - **Validates: Requirements 4.2, 4.4**
  - [ ] 23.4 Property 3: VB RDW length consistency — for any record content bytes, written RDW length == content.len() + 4
    - **Validates: Requirements 6.2, 6.6**
  - [ ] 23.5 Property 4: Field byte-range non-overflow — for any valid FieldDefinition, offset + length ≤ record_length (LRECL)
    - **Validates: Requirements 1.1, 1.2**
  - [ ] 23.6 Property 5: Record classification determinism — classifying the same record bytes twice yields the same RecordStructure assignment
    - **Validates: Requirements 13.1, 13.5**
  - [ ] 23.7 Property 6: Field validation accepts valid inputs — for any value that passes validation, encoding produces bytes ≤ field length
    - **Validates: Requirements 9.1, 9.5, 9.6**
  - [ ] 23.8 Property 7: ByteOffsetIndex monotonicity — index entries are strictly increasing (each offset > previous)
    - **Validates: Requirements 2.2, 10.1**
  - [ ] 23.9 Property 8: ASA detection threshold — files with ≥80% ASA chars in column 1 always trigger detection; files with <50% never trigger
    - **Validates: Requirements 7.3**
  - [ ] 23.10 Property 9: Window navigation bounds — page_up/page_down never produce record indices outside [0, total_records)
    - **Validates: Requirements 10.2, 10.3**
  - [ ] 23.11 Property 10: Structure file serialization roundtrip — serialize(deserialize(json)) preserves all fields and values
    - **Validates: Requirements 1.4, 1.5, 12.4**
  - [ ] 23.12 Property 11: Record insert preserves file integrity — after insert, total byte count == original + inserted record length (+ RDW for VB)
    - **Validates: Requirements 11.1, 11.5, 11.6**
  - [ ] 23.13 Property 12: COMP-3 decimal separator is always period — formatted output never contains locale-specific separators
    - **Validates: Requirements 5.10**

- [ ] 24. Integration tests
  - [ ] 24.1 Create `tests/structure_parse_tests.rs` — end-to-end parsing of `.ffs` files (valid, legacy `.fc.json`, overlapping fields, missing version)
  - [ ] 24.2 Create `tests/fb_session_tests.rs` — open FB flat file with structure, verify record count, field extraction, navigation
  - [ ] 24.3 Create `tests/vb_session_tests.rs` — open VB binary file, verify RDW parsing, record boundaries, random access
  - [ ] 24.4 Create `tests/ebcdic_session_tests.rs` — open EBCDIC file with code page, verify field decoding, edit roundtrip, export to CSV as UTF-8
  - [ ] 24.5 Create `tests/comp3_session_tests.rs` — open file with COMP-3 fields, verify decode/display/edit/re-encode pipeline
  - [ ] 24.6 Create `tests/asa_session_tests.rs` — open FBA report file, verify ASA detection, indicator display, strip operation
  - [ ] 24.7 Create `tests/multi_type_tests.rs` — open file with header/detail/trailer structures, verify classification, type filtering, statistics
  - [ ] 24.8 Create `tests/conversion_tests.rs` — run conversions to CSV, TSV, JSON, DAT; verify output content and format
  - [ ] 24.9 Create `tests/error_resilience_tests.rs` — verify error handling for missing files, empty files, invalid JSON, bad RDW, unmappable EBCDIC
  - [ ] 24.10 Create `tests/record_ops_tests.rs` — insert and delete records in FB and VB files, verify index consistency and byte content
  - [ ] 24.11 Verify crate builds cleanly with `cargo clippy -- -D warnings` and `cargo test` passes

---

## Acceptance Criteria Coverage Map

| Requirement | Tasks |
|------------|-------|
| Req 1 (Record_Structure Definition) | 2.1–2.9, 23.5, 23.11, 24.1 |
| Req 2 (Flat-File Open with Structure Overlay) | 7.1–7.6, 13.1–13.5, 15.2–15.4, 23.8, 24.2 |
| Req 3 (Grid_Edit_Mode) | 9.1–9.7, 20.1–20.6, 24.2 |
| Req 4 (EBCDIC-to-ASCII Conversion) | 3.1–3.6, 23.3, 24.4 |
| Req 5 (Packed Decimal COMP-3) | 4.1–4.6, 23.2, 23.13, 24.5 |
| Req 6 (VB Record Handling with RDW) | 5.1–5.8, 23.4, 24.3 |
| Req 7 (ASA Carriage Control Detection) | 6.1–6.6, 19.7–19.8, 23.9, 24.6 |
| Req 8 (FileForge_Mode Activation) | 15.1–15.9, 19.5–19.6 |
| Req 9 (Field Validation per Type) | 10.1–10.9, 23.7, 24.5 |
| Req 10 (Record Navigation) | 7.2, 12.1–12.7, 23.8, 23.10, 24.2 |
| Req 11 (Record Insert and Delete) | 14.1–14.9, 23.12, 24.10 |
| Req 12 (Structure File Association) | 15.4–15.5, 22.1–22.6, 23.11, 24.1 |
| Req 13 (Multiple Record Types per File) | 8.1–8.7, 23.6, 24.7 |
| Req 14 (Record Type Selection and Filtering) | 16.1–16.6, 24.7 |
| Req 15 (FileForge Command Integration) | 17.1–17.7, 19.1–19.10, 24.8 |
| Req 16 (Error Handling and Resilience) | 1.3–1.4, 21.1–21.7, 24.9 |

---

## Property-Based Test Definitions

| # | Property | Strategy | Requirement |
|---|----------|----------|-------------|
| 1 | COMP-3 roundtrip: `decode(encode(v)) == v` for values fitting field length | i64 values × field lengths 1–16 bytes | Req 5.2, 5.3, 5.5 |
| 2 | EBCDIC roundtrip: `decode(encode(s, cp), cp) == s` for mappable strings | ASCII printable strings × 4 code pages | Req 4.2, 4.4 |
| 3 | VB RDW length: written RDW L == content.len() + 4 | Random byte content 0–32760 bytes | Req 6.2, 6.6 |
| 4 | Field byte-range non-overflow: offset + length ≤ LRECL | Random FieldDef × LRECL 10–1000 | Req 1.1, 1.2 |
| 5 | Classification determinism: classify(record) is idempotent | Multi-structure definitions × random record bytes | Req 13.1, 13.5 |
| 6 | Validation-encoding consistency: valid input → bytes fit field | String/int/float/comp3 values × field lengths | Req 9.1, 9.5, 9.6 |
| 7 | ByteOffsetIndex monotonicity: offsets strictly increasing | Random file sizes with known LRECL | Req 2.2, 10.1 |
| 8 | ASA detection threshold: ≥80% ASA → detected; <50% → not detected | Records with varying ASA-char percentages | Req 7.3 |
| 9 | Navigation bounds: page ops keep index in [0, total) | Window sizes × total records × random page ops | Req 10.2, 10.3 |
| 10 | Structure file roundtrip: `serialize(deserialize(json)) == json` (semantically) | Generated StructureFile instances | Req 1.4, 1.5, 12.4 |
| 11 | Record insert integrity: new byte count == old + inserted length (+4 for VB) | FB/VB files × random record content | Req 11.1, 11.5, 11.6 |
| 12 | COMP-3 decimal separator: formatted output matches `^-?[0-9]+(\.[0-9]+)?$` | i64 values × decimals 0–9 | Req 5.10 |

---

## Notes

- Phase 1 (scaffolding) must complete before any other phase begins.
- Phases 2 (schema), 3 (EBCDIC), 4 (COMP-3), 5 (VB reader), and 6 (ASA) are independent of each other and can proceed in parallel after Phase 1.
- Phase 7 (FB reader / byte index) depends on Phase 2 for LRECL/RECFM types.
- Phase 8 (classifier) depends on Phase 2 for RecordStructure and FieldDefinition types.
- Phase 9 (field display) depends on Phases 3, 4, and 8 for encoding decode, COMP-3 format, and classification.
- Phase 10 (field validation) depends on Phase 2 for DataType definitions.
- Phase 11 (field edit) depends on Phases 3, 4, and 10 for encoding, COMP-3, and validation.
- Phase 12 (navigation) depends on Phase 7 for ByteOffsetIndex.
- Phase 13 (window) depends on Phases 7 and 8 for index and record reading.
- Phase 14 (record ops) depends on Phases 5 and 7 for VB/FB reader integration.
- Phase 15 (mode management) depends on Phases 7, 8, and 22 for activation logic.
- Phase 16 (filtering) depends on Phase 8 for classifier output.
- Phase 17 (conversion) depends on Phases 3, 4, 8, and 9 for decode/format pipeline.
- Phase 18 (configuration) is independent after Phase 1; provides defaults consumed by other phases.
- Phase 19 (commands) depends on Phases 15, 17, and 6 for mode, conversion, and ASA operations.
- Phase 20 (grid model) depends on Phases 8, 9, and 12 for classification, display, and navigation.
- Phase 21 (error handling) depends on Phase 1 error types; tests span all phases.
- Phase 22 (association) depends on Phase 2 for StructureFile serialization and Phase 15 for mode activation.
- Phases 23 and 24 (property-based and integration tests) depend on all implementation phases being complete.
- The `ff-encoding` crate provides EBCDIC codec infrastructure; this crate drives EBCDIC-specific workflows through that dependency.
- All file access flows through the VFS abstraction layer (trait-based, injected at construction).
- The `ff-config` crate dependency provides workbench configuration; if unavailable during early development, use hardcoded defaults.
- Command registration uses the command framework's `CommandRegistry` trait from `ff-command-framework`.

---

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 1,
      "label": "Crate Scaffolding & Error Types",
      "tasks": ["1.1", "1.2", "1.3", "1.4"],
      "dependsOn": []
    },
    {
      "id": 2,
      "label": "Record Format Types & Structure Schema",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "2.9"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "EBCDIC Codec Integration",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6"],
      "dependsOn": [1]
    },
    {
      "id": 4,
      "label": "COMP-3 Packed Decimal Engine",
      "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6"],
      "dependsOn": [1]
    },
    {
      "id": 5,
      "label": "VB Record Header Parsing",
      "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8"],
      "dependsOn": [1]
    },
    {
      "id": 6,
      "label": "ASA Carriage Control Detection",
      "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6"],
      "dependsOn": [1]
    },
    {
      "id": 7,
      "label": "Fixed-Length Record Reader & Byte Index",
      "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5", "7.6"],
      "dependsOn": [2]
    },
    {
      "id": 8,
      "label": "Record Classification",
      "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7"],
      "dependsOn": [2]
    },
    {
      "id": 9,
      "label": "Field Display & Value Conversion",
      "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7"],
      "dependsOn": [3, 4, 8]
    },
    {
      "id": 10,
      "label": "Field Validation Engine",
      "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9"],
      "dependsOn": [2]
    },
    {
      "id": 11,
      "label": "Field Editing & Encode-Back",
      "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6"],
      "dependsOn": [3, 4, 10]
    },
    {
      "id": 12,
      "label": "Record Navigation Engine",
      "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7"],
      "dependsOn": [7]
    },
    {
      "id": 13,
      "label": "Window Management & Record Streaming",
      "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5"],
      "dependsOn": [7, 8]
    },
    {
      "id": 14,
      "label": "Record Insert & Delete Operations",
      "tasks": ["14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7", "14.8", "14.9"],
      "dependsOn": [5, 7]
    },
    {
      "id": 15,
      "label": "FileForge Mode Management",
      "tasks": ["15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "15.7", "15.8", "15.9"],
      "dependsOn": [7, 8]
    },
    {
      "id": 16,
      "label": "Record Type Selection & Filtering",
      "tasks": ["16.1", "16.2", "16.3", "16.4", "16.5", "16.6"],
      "dependsOn": [8]
    },
    {
      "id": 17,
      "label": "Conversion & Export Engine",
      "tasks": ["17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7"],
      "dependsOn": [3, 4, 8, 9]
    },
    {
      "id": 18,
      "label": "Configuration Integration",
      "tasks": ["18.1", "18.2", "18.3", "18.4"],
      "dependsOn": [1]
    },
    {
      "id": 19,
      "label": "Command Registration",
      "tasks": ["19.1", "19.2", "19.3", "19.4", "19.5", "19.6", "19.7", "19.8", "19.9", "19.10"],
      "dependsOn": [6, 15, 17]
    },
    {
      "id": 20,
      "label": "Grid Edit Model (Data Layer)",
      "tasks": ["20.1", "20.2", "20.3", "20.4", "20.5", "20.6"],
      "dependsOn": [8, 9, 12]
    },
    {
      "id": 21,
      "label": "Error Handling & Resilience",
      "tasks": ["21.1", "21.2", "21.3", "21.4", "21.5", "21.6", "21.7"],
      "dependsOn": [1, 3, 5]
    },
    {
      "id": 22,
      "label": "Structure File Association",
      "tasks": ["22.1", "22.2", "22.3", "22.4", "22.5", "22.6"],
      "dependsOn": [2, 15]
    },
    {
      "id": 23,
      "label": "Property-Based Tests",
      "tasks": ["23.1", "23.2", "23.3", "23.4", "23.5", "23.6", "23.7", "23.8", "23.9", "23.10", "23.11", "23.12", "23.13"],
      "dependsOn": [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22]
    },
    {
      "id": 24,
      "label": "Integration Tests",
      "tasks": ["24.1", "24.2", "24.3", "24.4", "24.5", "24.6", "24.7", "24.8", "24.9", "24.10", "24.11"],
      "dependsOn": [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22]
    }
  ]
}
```
