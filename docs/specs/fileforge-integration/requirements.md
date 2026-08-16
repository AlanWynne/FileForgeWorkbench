# Requirements Document

## Introduction

This feature specifies the **FileForge Integration** subsystem for FileForgeWorkbench — the `ff-fileforge` crate. This crate integrates the FileForge flat-file processing engine into the workbench platform, enabling structured viewing, editing, and conversion of fixed-width flat files produced by mainframe and enterprise batch systems (COBOL, ABAP, JCL, etc.).

When a flat file is opened alongside a companion structure file (`.ffs`), the workbench activates **FileForge_Mode**: records are identified, classified, and displayed as structured tabular data. The user can browse large files with O(1) seek performance, edit individual fields in Grid_Edit_Mode, run conversions to modern formats, and work natively with EBCDIC-encoded content, packed decimal (COMP-3) fields, variable-length binary (VB) records, and ASA carriage control report files.

The `ff-fileforge` crate is **GUI-independent** — it implements the data model, record parsing, field extraction, encoding conversion, and file I/O logic. A separate GUI layer renders the grid and panels. All file access flows through the Virtual File System abstraction (FFW-ARCH-001).

**Source references:**
- **[FFE-FF]** = FileForgeEditor `fileforge-integration` specification (Requirements 1–25)
- **[WB]** = Workbench Platform Architecture Brief (VFS, command framework, plugin model)

## Cross-References

- **`document-model`** — The document model provides the underlying text buffer. FileForge_Mode overlays structured record interpretation on top of the raw buffer content. [FFE-FF, WB]
- **`encoding-and-characters`** — EBCDIC code page decoding and Unicode conversion for mainframe binary files. The `ff-encoding` crate provides the codec infrastructure; this crate drives the EBCDIC-specific workflows. [FFE-FF]
- **`record-selection-criteria`** — Field-level filter criteria for controlling which records are displayed in the grid. Criteria evaluation operates on the structured records produced by this crate. [FFE-FF]
- **`structure-catalog`** — Persistent library of named structure definitions (`.ffs` files) and file-to-structure association. Catalog management and grid editing are defined there; this crate provides the engine. [FFE-FF]
- **`asa-report-preview`** — Visual rendering of ASA carriage control as formatted report output. Depends on ASA detection defined in this crate. [FFE-FF]
- **`virtual-file-system`** — All file access (source data files, structure files, output files) flows through the VFS abstraction layer. [WB]
- **`command-framework`** — FileForge commands (CONVERT, VALIDATE, FILEFORGE) are registered in the command registry. [WB]

---

## Glossary

- **FileForge_Mode**: The workbench state activated when a flat file is opened with an associated structure definition. In this mode, records are parsed, classified, and displayed as structured tabular data. [FFE-FF]
- **Record_Structure**: A named definition describing the field layout for one category of record in a source file. Contains an ordered list of Field_Definitions. [FFE-FF]
- **Field_Definition**: A single field within a Record_Structure, specifying: field name, byte offset, byte length, data type, decimal places, and optional identifier/filter role. [FFE-FF]
- **Flat_File**: A data file where records are stored as fixed-position byte sequences with no embedded structural metadata. [FFE-FF]
- **Structure_File**: A companion `.ffs` (FileForge Structure) JSON file that describes the Record_Structures of a source file. Replaces the FFE `.fc.json` format with workbench-native naming. [FFE-FF, WB]
- **Record_Type**: The raw value found in an identifier field that determines which Record_Structure applies to a given record. [FFE-FF]
- **Identifier_Field**: A field within a Record_Structure whose value classifies source records. When a record's bytes at the identifier position match a value in the identifiers list, that Record_Structure is applied. [FFE-FF]
- **Filter_List**: An optional inclusion list on an identifier field. When non-empty, only records whose type value appears in the list are displayed or exported. [FFE-FF]
- **Grid_Edit_Mode**: The workbench state in which a structured file is displayed as an editable column-per-field grid with one row per record. Field values can be modified and saved back to the original format. [FFE-FF]
- **Byte_Offset_Index**: An in-memory array of file byte positions (one per record) enabling O(1) seek to any record by index. [FFE-FF]
- **Window**: A contiguous subset of records loaded on demand for display, avoiding full-file memory load. [FFE-FF]
- **LRECL**: Logical Record Length — the fixed byte width of every record in a fixed-width file. Enables O(1) seek without an index scan. [FFE-FF]
- **RECFM**: Record Format — describes the physical structure of records. Values: `F`, `FB`, `V`, `FB_BINARY`, `VB`, `FBA`, `VBA`, `U`. [FFE-FF]
- **RDW**: Record Descriptor Word — 4-byte prefix on VB binary records. Bytes 0–1 are big-endian record length (including RDW); bytes 2–3 are reserved zeros. [FFE-FF]
- **COMP3_Field**: A field stored as IBM packed decimal (COMP-3). Each byte holds two BCD nibbles; the low nibble of the final byte is the sign (C=positive, D=negative, F=unsigned). [FFE-FF]
- **EBCDIC**: Extended Binary Coded Decimal Interchange Code — the character encoding used by IBM mainframe systems. Supported code pages: 037, 285, 500, 1047. [FFE-FF]
- **Code_Page**: A specific EBCDIC variant mapping byte values to characters. [FFE-FF]
- **ASA_Control**: The character in column 1 of FBA/VBA records that defines printer carriage control actions. [FFE-FF]
- **Field_Validation_Error**: A condition where a field value does not conform to its declared data type or constraints. [FFE-FF]
- **FileForge_Mode_Activation**: The process of detecting or manually selecting a structure, building the record index, and transitioning the workbench to structured display. [FFE-FF, WB]

---

## Requirements

### Requirement 1: Record_Structure Definition

**User Story:** As a data engineer, I want to define the field layout of records in a flat file using named Record_Structures, so that the workbench can parse, display, and validate fields by position and type.

**Source:** [FFE-FF] Requirements 3, 15, 16.

#### Acceptance Criteria

1. A Record_Structure SHALL consist of an ordered list of Field_Definitions, where each Field_Definition specifies: `field_name` (non-empty UTF-8 string), `offset` (non-negative byte offset from record start), `length` (positive byte length), `data_type` (one of: `str`, `int`, `float`, `bool`, `comp3`), `decimals` (non-negative integer, default 0), and optional `identifiers` and `filters` lists.
2. WHEN two or more Field_Definitions within the same Record_Structure have overlapping byte ranges (offset to offset+length-1), THE system SHALL report a structure validation warning but SHALL NOT prevent the structure from loading.
3. THE Structure_File schema SHALL support multiple named Record_Structures per file, enabling multi-type flat files where different record categories have different field layouts.
4. THE Structure_File schema SHALL support an optional top-level `lrecl` integer key specifying the logical record length in bytes, and an optional `recfm` string key specifying the record format (valid values: `F`, `FB`, `V`, `FB_BINARY`, `VB`, `FBA`, `VBA`, `U` — case-insensitive on load, normalised to uppercase on save).
5. THE Structure_File schema SHALL support an optional `encoding` key specifying the character encoding of the source file. Valid values include Unicode encodings (`utf-8`, `utf-16le`, `utf-16be`) and EBCDIC code page identifiers (`ebcdic-037`, `ebcdic-285`, `ebcdic-500`, `ebcdic-1047`).
6. THE Structure_File schema SHALL support an optional `version` key for schema migration. Files without a `version` key SHALL be treated as version `"1.0"`.
7. THE system SHALL accept legacy misspelled key `field_delimeter` as equivalent to `field_delimiter` for backward compatibility with existing config files.
8. THE system SHALL normalise legacy Python repr data_type strings on load: `"<class 'str'>"` → `"str"`, `"<class 'int'>"` → `"int"`, `"<class 'float'>"` → `"float"`, `"<class 'bool'>"` → `"bool"`.

---

### Requirement 2: Flat-File Open with Structure Overlay

**User Story:** As an editor user, I want the workbench to open a flat file and automatically overlay the associated structure definition, so that I see structured record data without manual configuration.

**Source:** [FFE-FF] Requirements 1, 2, 17. [WB] VFS.

#### Acceptance Criteria

1. WHEN a flat file is opened via VFS and a companion `.ffs` structure file exists (same base name, same directory, or associated via the Structure_Catalog), THE system SHALL activate FileForge_Mode for that file session.
2. WHEN FileForge_Mode is activated, THE system SHALL build a Byte_Offset_Index by scanning the file once through the VFS. For fixed-width files with known LRECL and RECFM `F` or `FB`, record position SHALL be calculated directly without an index scan.
3. WHEN the index build takes more than 2 seconds, THE system SHALL report progress to the workbench progress indicator, keeping the application responsive.
4. WHEN no companion structure file exists and no Structure_Catalog association matches, THE system SHALL open the file in standard text mode and offer to generate a template structure file or open the Structure_Catalog selector.
5. WHEN both a companion `.ffs` and a Structure_Catalog association exist for the same file, THE companion `.ffs` SHALL take precedence.
6. WHEN the structure file cannot be parsed (invalid JSON or missing required fields), THE system SHALL report the parse error and open the file in standard text mode.
7. THE default Window size SHALL be 200 records. THE user SHALL be able to configure the window size in the workbench configuration system.
8. WHEN the user scrolls or navigates, THE system SHALL retrieve the required Window of records by VFS seek, with no full-file memory load.
9. WHEN LRECL is not specified in the structure file, THE system SHALL perform LRECL auto-detection by sampling the first 100 lines and checking for uniform byte length.
10. WHEN LRECL auto-detection finds uniform line length, THE system SHALL use the detected value for the session and offer to persist it to the structure file.
11. WHEN LRECL auto-detection finds variable line lengths, THE system SHALL use variable-length mode with byte-offset indexing.

---

### Requirement 3: Grid_Edit_Mode (Tabular Field-by-Field Editing)

**User Story:** As a data engineer, I want to view and edit flat file records in a tabular grid with one column per field, so that I can inspect and modify individual field values without manually counting byte positions.

**Source:** [FFE-FF] Requirements 5, 20. Structure-catalog spec (Grid_Edit_Mode).

#### Acceptance Criteria

1. WHEN FileForge_Mode is active, THE system SHALL display records in a scrollable grid with one row per record and one column per Field_Definition in the active Record_Structure.
2. THE grid SHALL support three display modes selectable by the user: **Raw** (original byte content), **Structured** (parsed field values), and **Transformed** (values after decimal/COMP-3 conversion).
3. WHEN in Grid_Edit_Mode, THE user SHALL be able to click on any cell and edit its value directly. Field edits SHALL be validated against the field's data_type before acceptance.
4. WHEN a field edit passes validation, THE system SHALL encode the new value back to the correct byte representation (respecting encoding, COMP-3 packing, and field length) and update the document buffer at the correct byte offset.
5. WHEN a field edit would produce a byte sequence longer than the field's declared length, THE system SHALL reject the edit and display a field-length overflow warning.
6. WHEN multiple Record_Structures are defined, THE grid SHALL allow the user to filter the display by Record_Structure type using a record type selector. Cross-ref: `record-selection-criteria` spec for advanced filtering.
7. THE grid SHALL display the 1-based record number for each visible row.
8. WHEN the user selects a record in the grid, THE system SHALL highlight the corresponding raw bytes in any open raw/hex view of the same file.
9. WHEN `COPY` is issued in clipboard-paste or file-insert mode while Grid_Edit_Mode is active, THE system SHALL refuse the operation with a clear error message. In-document record copy (entire records) SHALL be permitted.
10. WHEN records are copied within Grid_Edit_Mode, THE system SHALL preserve the full byte content and re-classify the copied records using the active Record_Structure.

---

### Requirement 4: EBCDIC-to-ASCII Conversion

**User Story:** As a data engineer working with binary-downloaded mainframe files, I want the workbench to decode EBCDIC-encoded text fields using the correct code page, so that I can read and edit mainframe data without manual character conversion.

**Source:** [FFE-FF] Requirements 18, 19, 23. Cross-ref: `encoding-and-characters`.

#### Acceptance Criteria

1. THE Structure_File `encoding` key SHALL support EBCDIC code page identifiers: `ebcdic-037` (US English), `ebcdic-285` (UK English), `ebcdic-500` (International), `ebcdic-1047` (Open Systems Latin-1), in addition to Unicode encodings.
2. WHEN the encoding is EBCDIC, THE system SHALL decode string fields (`data_type: "str"` or `"bool"`) from EBCDIC to Unicode using the specified code page before display.
3. WHEN the encoding is EBCDIC, numeric fields (`data_type: "int"`, `"float"`, `"comp3"`) SHALL be treated as binary data and SHALL NOT be passed through the EBCDIC decoder.
4. WHEN the user edits a string field in an EBCDIC file, THE system SHALL accept Unicode input and re-encode it to the specified EBCDIC code page on save. IF a Unicode character has no mapping in the target code page, THE system SHALL report an encoding error for that field.
5. WHEN a byte value in an EBCDIC field has no mapping in the specified code page, THE system SHALL display a non-printable indicator (`.`) in structured view and show the raw hex byte in hex view.
6. WHEN exporting an EBCDIC file to CSV, TSV, or JSON format, THE system SHALL output Unicode-decoded text. The output file SHALL be written in UTF-8.
7. WHEN exporting to fixed-width reconstruction format (DAT/TXT), THE system SHALL re-encode string fields back to the specified EBCDIC code page, preserving the original binary format.
8. WHEN `recfm` is `FB_BINARY` or `VB` and no `encoding` is specified in the structure file, THE system SHALL default to EBCDIC-037 and display a validation warning suggesting the operator verify the code page.
9. WHEN encoding cannot be detected reliably and no encoding is specified, THE system SHALL default to UTF-8 and display a warning in the status area. Cross-ref: `encoding-and-characters` Requirement 1.

---

### Requirement 5: Packed Decimal (COMP-3) Display and Edit

**User Story:** As a data engineer working with mainframe financial files, I want the workbench to correctly interpret, display, and edit COMP-3 packed decimal fields, so that I can work with IBM packed decimal data without manual byte-level inspection.

**Source:** [FFE-FF] Requirement 21.

#### Acceptance Criteria

1. THE Structure_File schema SHALL support `"comp3"` as a valid `data_type` value for a Field_Definition, indicating the field contains IBM packed decimal data.
2. WHEN a field has `data_type: "comp3"`, THE system SHALL interpret its bytes as packed decimal: each byte holds two BCD digit nibbles (high nibble first), and the low nibble of the final byte is the sign (`C` = positive, `D` = negative, `F` = unsigned).
3. WHEN displaying a COMP-3 field in Structured or Transformed mode, THE system SHALL convert the raw bytes to a human-readable decimal string with the appropriate sign (e.g., `X'1234567D'` → `-1234567`, `X'1234567C'` → `1234567`, `X'1234567F'` → `1234567`).
4. WHEN a COMP-3 field has `decimals: N > 0`, THE system SHALL apply the implied decimal point after conversion (e.g., `X'0123456C'` with `decimals: 2` → `1234.56`).
5. WHEN the user edits a COMP-3 field in Grid_Edit_Mode, THE system SHALL accept a decimal number as input and re-encode it into packed decimal bytes on save, using `C` for positive, `D` for negative, and `F` for unsigned fields.
6. WHEN encoding a value into COMP-3, IF the value requires more digit pairs than the field's `length` permits, THE system SHALL reject the edit with a field-length overflow warning.
7. WHEN a COMP-3 field's raw bytes contain an invalid nibble (value outside 0x0–0x9 in a digit position, or invalid sign nibble), THE system SHALL treat it as a Field_Validation_Error and display the raw bytes in hex notation in the grid cell.
8. WHEN exporting a file with COMP-3 fields to CSV, TSV, or JSON format, THE system SHALL output the decoded decimal string value.
9. WHEN exporting to fixed-width reconstruction format (DAT/TXT), THE system SHALL re-encode the decimal value back to COMP-3 packed decimal bytes.
10. THE decimal separator SHALL always be `.` (period) regardless of system locale.

---

### Requirement 6: Variable-Length Binary Record (VB) Handling with RDW

**User Story:** As a data engineer working with binary-downloaded mainframe VB files, I want the workbench to correctly read the 4-byte Record Descriptor Word prefix on each record, so that records are identified and displayed at their correct boundaries.

**Source:** [FFE-FF] Requirement 22.

#### Acceptance Criteria

1. WHEN `recfm: "VB"` is configured in the Structure_File, THE system SHALL use a VB binary reader that interprets the file as a sequence of RDW-prefixed records with no newline characters.
2. THE VB binary reader SHALL process each record as follows: read 4 bytes as the RDW; extract record length `L` from bytes 0–1 as big-endian unsigned 16-bit integer (includes the 4-byte RDW itself); verify bytes 2–3 are `0x0000`; read `L - 4` bytes as record content; advance by `L` bytes total.
3. WHEN the VB reader encounters an RDW where `L < 4` or `L` would read past end-of-file, THE system SHALL stop reading, report a structural error, and display the number of records successfully read.
4. THE VB binary reader SHALL build a Byte_Offset_Index during file open, recording the byte offset of each record's content start (after the RDW) for O(1) random access.
5. WHEN displaying VB records, THE system SHALL show record content only — the RDW bytes SHALL NOT appear as part of the displayed record.
6. WHEN saving modified VB records in Grid_Edit_Mode, THE system SHALL re-write each record with its RDW prefix, updating the `L` value if the record length has changed due to editing.
7. WHEN `recfm: "VB"` is configured, the `lrecl` field SHALL be ignored (VB records have variable lengths). A validation warning SHALL be emitted if both `recfm: "VB"` and `lrecl` are present.
8. WHEN LRECL auto-detection is triggered on a file with `recfm: "VB"`, THE system SHALL skip detection and use the VB binary reader directly.

---

### Requirement 7: ASA Carriage Control Detection

**User Story:** As a data engineer working with mainframe report spool files, I want the workbench to detect ASA carriage control characters in column 1 and display them meaningfully, so that I can read report files without being confused by printer directive characters.

**Source:** [FFE-FF] Requirements 24, 25. Cross-ref: `asa-report-preview`.

#### Acceptance Criteria

1. THE Structure_File schema SHALL support `"FBA"` (fixed blocked, ASA) and `"VBA"` (variable blocked, ASA) as valid `recfm` values, in addition to the base values.
2. WHEN `recfm` is `"FBA"` or `"VBA"`, THE system SHALL activate ASA_Display_Mode, interpreting column 1 of each record as an ASA carriage control character rather than data content.
3. THE system SHALL auto-detect ASA carriage control by sampling the first 20 non-blank lines of a file. IF column 1 of at least 80% of sampled lines contains a known ASA control character (space, `0`, `-`, `1`, `+`, `H`), THE system SHALL activate ASA_Display_Mode and report `ASA CARRIAGE CONTROL DETECTED` in the status area.
4. WHEN ASA_Display_Mode is active, THE system SHALL display a non-editable ASA indicator for each record showing the human-readable meaning: space → `SP` (single space), `0` → `DS` (double space), `-` → `TS` (triple space), `1` → `NP` (new page), `+` → `OP` (overprint), `H` → `HT` (halt).
5. WHEN ASA_Display_Mode is active, data content SHALL begin at column 2; column 1 is shown only as the ASA indicator in the prefix area.
6. THE ASA_Display_Mode SHALL be togglable via `ASA ON` and `ASA OFF` commands registered in the command framework.
7. THE `ASA ON`/`ASA OFF` state SHALL be displayed in the status bar and SHALL NOT be added to the undo stack (it is a display mode change, not a data modification).
8. THE command framework SHALL support an `ASA STRIP` command that removes column 1 ASA characters from all records, shifting content left by one byte. This SHALL be a single undoable transaction. Cross-ref: `asa-report-preview` for visual rendering of ASA content.

---

### Requirement 8: FileForge_Mode Activation (Auto-Detect or Manual)

**User Story:** As an editor user, I want the workbench to automatically detect when a file should be displayed in FileForge_Mode, and also allow me to manually activate the mode, so that structured display works seamlessly for known file types and on-demand for unknown files.

**Source:** [FFE-FF] Requirements 1, 8. [WB] Plugin architecture, command framework.

#### Acceptance Criteria

1. WHEN a file is opened and a companion `.ffs` structure file exists in the same directory with the same base name, THE system SHALL activate FileForge_Mode automatically.
2. WHEN a file is opened and its filename matches a pattern in the Structure_Catalog File_Association_Map, THE system SHALL activate FileForge_Mode using the associated Structure_Definition. Cross-ref: `structure-catalog` spec.
3. WHEN neither auto-detection condition is met, THE system SHALL open the file in standard text mode without FileForge_Mode.
4. THE command framework SHALL support a `FILEFORGE` command that manually activates FileForge_Mode for the current file, opening the Structure_Catalog selector if no structure is currently associated.
5. THE command framework SHALL support a `FILEFORGE OFF` command that deactivates FileForge_Mode and returns to standard text display.
6. WHEN FileForge_Mode is active, THE status bar SHALL display a `FileForge` mode indicator showing the active Structure_Definition name.
7. WHEN the user requests template generation (via command or UI), THE system SHALL generate a skeleton `.ffs` structure file in the source file's directory and open it for editing.
8. WHEN both `.ffs` and legacy `.fc.json` companion files exist, THE system SHALL prefer `.ffs`. IF only `.fc.json` exists, THE system SHALL load it with legacy compatibility rules (Requirement 1, criteria 7–8) and offer to migrate it to `.ffs` format.

---

### Requirement 9: Field Validation per Type

**User Story:** As a data engineer editing structured records, I want field values to be validated against their declared data type in real time, so that I catch data entry errors before saving.

**Source:** [FFE-FF] Requirements 3, 5, 7, 21.

#### Acceptance Criteria

1. WHEN the user enters a value in a `data_type: "int"` field, THE system SHALL accept only integer numeric input (optional leading sign, digits only). Non-numeric input SHALL be rejected with an inline validation error.
2. WHEN the user enters a value in a `data_type: "float"` field, THE system SHALL accept numeric input with optional decimal point and sign. Non-numeric input SHALL be rejected with an inline validation error.
3. WHEN the user enters a value in a `data_type: "bool"` field, THE system SHALL accept only recognised boolean representations: `true`, `false`, `T`, `F`, `Y`, `N`, `1`, `0` (case-insensitive).
4. WHEN the user enters a value in a `data_type: "str"` field, THE system SHALL accept any input that fits within the field's byte length when encoded in the file's encoding.
5. WHEN the user enters a value in a `data_type: "comp3"` field, THE system SHALL accept decimal numeric input and validate that the packed representation fits within the field's byte length.
6. WHEN a field value exceeds the declared byte length after encoding, THE system SHALL reject the edit with a field-length overflow error displayed inline in the grid cell.
7. WHEN a numeric field with `decimals: N > 0` is edited in Transformed mode, THE system SHALL accept decimal input and convert it to the internal packed integer representation (multiply by 10^N, round to nearest integer).
8. WHEN the Structure_File is loaded, THE system SHALL run structural validation (checking for negative offsets, zero lengths, blank field names) and report warnings without preventing FileForge_Mode activation.
9. WHEN a field has a Field_Validation_Error on display (e.g., invalid COMP-3 nibbles, un-decodable EBCDIC bytes), THE system SHALL display the raw bytes in hex notation and mark the cell with a visual error indicator.

---

### Requirement 10: Record Navigation

**User Story:** As a data engineer working with large flat files, I want efficient navigation controls (go-to-record, page up/down, first/last record), so that I can quickly reach any record in a multi-million-record file.

**Source:** [FFE-FF] Requirements 2, 5.

#### Acceptance Criteria

1. WHEN a record number is entered in the navigation field, THE system SHALL seek directly to that record using the Byte_Offset_Index (O(1) access) and display it at the top of the viewport.
2. THE system SHALL support Page Up / Page Down navigation that advances the display by one Window of records.
3. THE system SHALL support First Record (Ctrl+Home) and Last Record (Ctrl+End) navigation that jumps to the beginning or end of the file.
4. THE system SHALL display the current record number, total record count, and percentage position in the status area at all times while FileForge_Mode is active.
5. WHEN the user navigates while a record type filter is active, THE system SHALL skip non-matching records and navigate only among visible (matching) records.
6. THE index memory footprint SHALL NOT exceed 100 MB for files up to 10 million records.
7. THE system SHALL present the first Window of records within 5 seconds of file open for files up to 2 GB.

---

### Requirement 11: Record Insert and Delete in Structured Mode

**User Story:** As a data engineer, I want to insert new records and delete existing records while in Grid_Edit_Mode, so that I can manage the record set without leaving structured editing mode.

**Source:** [FFE-FF] Requirements 5, 7. Structure-catalog spec (Grid_Edit_Mode).

#### Acceptance Criteria

1. WHEN the user inserts a new record in Grid_Edit_Mode, THE system SHALL create a new record initialised with spaces (or EBCDIC space equivalent) at the field's byte length, positioned after the currently selected record.
2. WHEN a new record is inserted, THE system SHALL assign it the default Record_Structure (the first structure in the definition or the currently filtered type) and display it as an editable grid row.
3. WHEN the user deletes a record in Grid_Edit_Mode, THE system SHALL remove the record's bytes from the document buffer and update the Byte_Offset_Index accordingly.
4. Record insert and delete operations SHALL each be a single undoable transaction, fully reversible via the undo system. Cross-ref: `undo-redo-transactions`.
5. WHEN a record is inserted or deleted in a VB file (`recfm: "VB"`), THE system SHALL update the RDW of the new record (on insert) or remove the RDW along with the record content (on delete).
6. WHEN a record is inserted in a fixed-width file (`recfm: "F"` or `"FB"`), THE inserted record SHALL be exactly LRECL bytes, padded with spaces if the user has not filled all fields.
7. THE system SHALL update the total record count display and Byte_Offset_Index immediately after insert or delete operations.
8. WHEN multiple records are selected (block selection), THE system SHALL support bulk delete with a single confirmation prompt and a single undoable transaction.

---

### Requirement 12: Structure File Association (.ffs Files)

**User Story:** As a data engineer, I want the workbench to associate flat files with their structure definitions using `.ffs` companion files and catalog patterns, so that the correct field layout is applied automatically on file open.

**Source:** [FFE-FF] Requirements 1, 3, 8, 15. Cross-ref: `structure-catalog`.

#### Acceptance Criteria

1. THE workbench SHALL recognise `.ffs` (FileForge Structure) as the native structure file extension. Legacy `.fc.json` and `.fc.xlsx` files SHALL also be loadable for backward compatibility.
2. WHEN a flat file is opened, THE system SHALL search for a companion structure file using the following precedence: (a) `<basename>.ffs` in the same directory, (b) `<basename>.fc.json` in the same directory, (c) Structure_Catalog File_Association_Map match by filename pattern.
3. WHEN the user saves a structure definition from the GUI, THE system SHALL write a `.ffs` file in JSON format to the source file's directory using the VFS.
4. THE `.ffs` file SHALL contain: version, optional lrecl, optional recfm, optional encoding, optional field_delimiter, and an array of Record_Structures with their Field_Definitions.
5. WHEN a `.ffs` file is modified externally while the file is open in FileForge_Mode, THE system SHALL detect the change (via VFS file-watcher) and offer to reload the structure definition.
6. WHEN the user creates a new structure association via the Structure_Catalog selector, THE system SHALL record the association in the Catalog_Store for future file opens. Cross-ref: `structure-catalog` Requirement 5.
7. WHEN a template structure file is generated, THE system SHALL write a `.ffs` file containing a single empty Record_Structure with placeholder fields, ready for the user to fill in.

---

### Requirement 13: Multiple Record Types per File

**User Story:** As a data engineer working with multi-type flat files (header/detail/trailer, multi-format batch output), I want the workbench to classify each record by its type and display the appropriate field layout for each, so that I can work with complex multi-structure files.

**Source:** [FFE-FF] Requirements 4, 5.

#### Acceptance Criteria

1. WHEN a Structure_File contains multiple named Record_Structures, THE system SHALL classify each record by evaluating identifier fields against the record's bytes at the identifier position.
2. WHEN a record's bytes match an identifier value in a Record_Structure's identifier field, THE system SHALL apply that Record_Structure's field layout to the record for display and editing.
3. WHEN a record matches no Record_Structure (no identifier match), THE system SHALL display it as an unclassified record with a visual indicator and count it in the session's `records_skipped` total.
4. WHEN a record matches a Record_Structure but is excluded by a non-empty Filter_List on the identifier field, THE system SHALL display it as a filtered record with a visual indicator and count it in the session's `records_filtered` total.
5. THE first matching Record_Structure in the definition's order SHALL be applied when a record's identifier matches multiple structures. First-match-wins semantics.
6. THE grid display SHALL support showing all record types interleaved (natural file order), or filtered to a single Record_Structure type using the record type selector.
7. WHEN filtered to a single Record_Structure type, THE grid columns SHALL match that structure's Field_Definitions. WHEN showing all types, THE grid SHALL display a `Record_Type` label column and the raw record content.
8. THE system SHALL report classification statistics in the status area: total records, records per type, skipped records, filtered records.

---

### Requirement 14: Record Type Selection and Filtering

**User Story:** As a data engineer, I want to filter the record display by type or by field criteria, so that I can focus on specific record categories in a large multi-type file.

**Source:** [FFE-FF] Requirements 4, 5. Cross-ref: `record-selection-criteria`.

#### Acceptance Criteria

1. THE system SHALL provide a Record_Type selector (dropdown or tab control) that lists all Record_Structure names defined in the active Structure_File, plus an "All Types" option.
2. WHEN a specific Record_Type is selected, THE grid SHALL display only records matching that type, hiding all other records without modifying the source file.
3. WHEN "All Types" is selected, THE grid SHALL display all records in file order, with a Record_Type label column identifying each record's classification.
4. THE record type filter SHALL compose with the Record_Selection_Criteria system: when both a type filter and field-level criteria are active, only records that satisfy BOTH conditions SHALL be displayed. Cross-ref: `record-selection-criteria`.
5. WHEN the record type filter changes, THE system SHALL update the record count display and navigation position to reflect the filtered view.
6. WHEN navigating with a type filter active, THE system SHALL skip non-matching records (go-to-record numbers refer to the filtered sequence, not the raw file position).
7. THE system SHALL display a filter-active indicator in the status area when any type filter or selection criteria restrict the visible record set.

---

### Requirement 15: FileForge Command Integration

**User Story:** As an editor user, I want to invoke FileForge operations from the command framework, so that I can use keyboard-driven workflows for flat file processing.

**Source:** [FFE-FF] Requirements 9, 10, 13. [WB] Command framework.

#### Acceptance Criteria

1. THE command framework SHALL register a `fileforge.convert` command that triggers a flat-file conversion using the current session's structure and settings. The command SHALL accept an optional OutputType argument (`csv`, `tsv`, `json`, `dat`, `txt`).
2. WHEN `fileforge.convert` is executed, THE system SHALL run the conversion asynchronously (non-blocking) and report progress to the workbench progress indicator.
3. WHEN conversion completes, THE system SHALL display a summary: records read, records written, records skipped, records filtered, and the output file path.
4. WHEN conversion fails, THE system SHALL display a clear error message without crashing. The workbench SHALL remain in a usable state.
5. THE command framework SHALL register a `fileforge.validate` command that re-runs structure validation and displays any warnings in the status area.
6. THE command framework SHALL register a `fileforge.export_config` command that exports the current structure definition to CSV format (`<source_stem>_config.csv`).
7. THE `fileforge.convert` and `fileforge.export_config` commands SHALL be valid only when FileForge_Mode is active. WHEN issued outside FileForge_Mode, THE system SHALL display an error indicating no structure is loaded.
8. THE command framework SHALL register `fileforge.on` and `fileforge.off` commands for manual mode activation/deactivation (equivalent to Requirement 8 criteria 4–5).

---

### Requirement 16: Error Handling and Resilience

**User Story:** As a data engineer, I want clear, specific error messages when something goes wrong with a flat file operation, so that I understand what failed and can correct it.

**Source:** [FFE-FF] Requirement 11.

#### Acceptance Criteria

1. WHEN a source file cannot be found via VFS, THE system SHALL display an error including the resource URI that was not found.
2. WHEN a source file is empty (zero bytes), THE system SHALL display a message indicating the file contains no records and open it in standard text mode.
3. WHEN a structure file contains invalid JSON syntax or missing required fields, THE system SHALL display a parse error description and offer to open the structure file for editing.
4. WHEN an I/O error occurs during file reading, conversion, or saving, THE system SHALL display the I/O error description with the affected resource path.
5. WHEN an unsupported output type is requested for conversion, THE system SHALL display an error identifying the unsupported type.
6. IN ALL error cases, THE system SHALL NOT crash, SHALL NOT corrupt the source file, and SHALL remain in a usable state after displaying the error.
7. WHEN a VB binary file contains a structural error (invalid RDW), THE system SHALL display records read before the error and report the byte offset of the failure.
8. WHEN EBCDIC decoding encounters unmappable bytes, THE system SHALL display per-field warnings without aborting the entire file display.
