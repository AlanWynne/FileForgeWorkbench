# Implementation Plan: Structure Catalog (`ff-structure-catalog`)

## Overview

This plan covers the complete implementation of the `ff-structure-catalog` crate — the persistent, operator-managed library of named Record_Structure definitions for FileForgeWorkbench. The structure catalog provides a central repository of reusable structure definitions that can be applied to any flat-file data file, replacing per-file companion configs.

This is a **Wave 12 (FileForge Domain)** sub-project. It depends on `ff-logging` (diagnostics), `ff-command` (command registration), `ff-config` (catalog path settings), `ff-vfs` (file access), `ff-layout` (dockable panels), `ff-fileforge` (record parsing, field extraction, COMP-3 handling), and `ff-plugin` (extensible field types).

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-structure-catalog/Cargo.toml` with dependencies (ff-logging, ff-command, ff-config, ff-vfs, ff-fileforge, ff-plugin, thiserror, serde, toml, chrono, glob, proptest dev-dep)
  - [ ] 1.2 Create `crates/ff-structure-catalog/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `model.rs`, `field.rs`, `ffs_format.rs`, `catalog.rs`, `crud.rs`, `persistence.rs`, `browsing.rs`, `editor.rs`, `association.rs`, `import.rs`, `export.rs`, `versioning.rs`, `location.rs`, `config.rs`, `commands.rs`, `grid.rs`, `error.rs`
  - [ ] 1.4 Add `ff-structure-catalog` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. CatalogEntry model (Structure_Definition)
  - [ ] 2.1 Define `StructureMetadata` struct with fields: name (String), description (Option<String>), version (u32), created_at (DateTime), modified_at (Option<DateTime>), encoding (Option<String>), lrecl (Option<u32>), recfm (Option<RecordFormat>)
  - [ ] 2.2 Define `RecordFormat` enum with variants: F, FB, V, FbBinary, VB, U
  - [ ] 2.3 Define `RecordStructure` struct with fields: name (String), fields (Vec<FieldDefinition>)
  - [ ] 2.4 Define `FileAssociations` struct with fields: file_patterns (Vec<String>)
  - [ ] 2.5 Define `StructureDefinition` struct composing: metadata (StructureMetadata), associations (Option<FileAssociations>), record_structures (Vec<RecordStructure>)
  - [ ] 2.6 Implement `Display`, `Debug`, `Clone`, `PartialEq` derives for all model types
  - [ ] 2.7 Write unit tests for model construction, field access, and default values
  - Covers: Requirement 2 (AC 2.1), Requirement 9 (AC 9.1, 9.3, 9.4)

- [ ] 3. FieldDefinition model and field types
  - [ ] 3.1 Define `FieldType` enum with variants: Alphanumeric, Numeric, PackedDecimal, Binary, Hex
  - [ ] 3.2 Define `FieldDefinition` struct with fields: name (String), offset (usize), length (usize), field_type (FieldType), decimals (u8), identifiers (Vec<String>), filters (Vec<String>)
  - [ ] 3.3 Implement field validation: name non-empty, offset >= 0, length >= 1, field_type valid enum, decimals >= 0
  - [ ] 3.4 Implement `FieldDefinition::validate() -> Result<(), ValidationError>` returning all validation failures
  - [ ] 3.5 Implement packed-decimal display logic: unpack COMP-3 bytes to signed decimal string with N decimal places
  - [ ] 3.6 Implement numeric implied-decimal display: insert decimal point N positions from right
  - [ ] 3.7 Implement packed-decimal validation: detect invalid nibble values (not 0-9 for digits, not C/D/F for sign)
  - [ ] 3.8 Write unit tests for field validation, packed-decimal unpacking, numeric decimal display, and invalid nibble detection
  - Covers: Requirement 2 (AC 2.2), Requirement 5 (AC 5.9), Requirement 6 (AC 6.1–6.8)

- [ ] 4. Field type extensibility
  - [ ] 4.1 Define `FieldTypeHandler` trait with methods: decode(bytes, field_def) -> DisplayValue, encode(display_str, field_def) -> Vec<u8>, validate(display_str, field_def) -> Result
  - [ ] 4.2 Implement built-in handlers for Alphanumeric, Numeric, PackedDecimal, Binary, Hex
  - [ ] 4.3 Define `FieldTypeRegistry` struct for registering custom handlers via the plugin trait system
  - [ ] 4.4 Implement handler lookup by FieldType with fallback to built-in for unregistered types
  - [ ] 4.5 Write unit tests for handler registration, lookup, decode/encode round-trips for each built-in type
  - Covers: Requirement 6 (AC 6.9)

- [ ] 5. FFS file format — TOML serialization
  - [ ] 5.1 Implement `FfsSerializer` struct with `serialize(def: &StructureDefinition) -> Result<String, FfsError>` producing valid TOML v1.0
  - [ ] 5.2 Implement `[metadata]` table serialization with all required and optional keys
  - [ ] 5.3 Implement `[associations]` table serialization with file_patterns array
  - [ ] 5.4 Implement `[[record_structures]]` and `[[record_structures.fields]]` array-of-tables serialization
  - [ ] 5.5 Write unit tests verifying output is valid TOML and contains all expected keys/values
  - Covers: Requirement 2 (AC 2.1, 2.7, 2.8, 2.9)

- [ ] 6. FFS file format — TOML deserialization and validation
  - [ ] 6.1 Implement `FfsParser` struct with `parse(toml_str: &str) -> Result<StructureDefinition, FfsError>` parsing TOML v1.0
  - [ ] 6.2 Implement TOML syntax error handling: reject with WARN log including file path and parse error details
  - [ ] 6.3 Implement schema validation: check required keys (metadata.name, metadata.version, record_structures), valid field_type values, non-negative offset/length
  - [ ] 6.4 Implement schema validation error reporting: WARN log with validation details, exclude from catalog listing
  - [ ] 6.5 Implement version key validation: must be positive integer
  - [ ] 6.6 Implement name uniqueness check hook (takes a predicate for collision detection)
  - [ ] 6.7 Write unit tests for valid parsing, TOML syntax errors, schema validation failures, missing keys, invalid field_type values, negative offset/length
  - Covers: Requirement 2 (AC 2.1–2.9), Requirement 2 (AC 2.3, 2.4, 2.5, 2.6)

- [ ] 7. Catalog persistent store — directory management
  - [ ] 7.1 Implement `CatalogStore` struct wrapping a VFS-backed catalog directory path
  - [ ] 7.2 Implement directory creation on first use if Active_Catalog_Location does not exist, with INFO log
  - [ ] 7.3 Implement platform-specific default path resolution (~/.config/ffworkbench/catalogs/ on Linux, %APPDATA%\FFWorkbench\catalogs\ on Windows, ~/Library/Application Support/FFWorkbench/catalogs/ on macOS)
  - [ ] 7.4 Implement inaccessible location handling: WARN log, skip, continue with other locations
  - [ ] 7.5 Implement multi-location support: load definitions from all configured Catalog_Locations
  - [ ] 7.6 Implement VFS integration: all file I/O routed through `virtual-file-system` abstraction
  - [ ] 7.7 Write unit tests for directory creation, default path, inaccessible handling, and multi-location scanning
  - Covers: Requirement 1 (AC 1.1, 1.3, 1.4, 1.5, 1.6, 1.8)

- [ ] 8. Catalog persistence — load and index
  - [ ] 8.1 Implement `CatalogIndex` struct: in-memory HashMap<String, StructureDefinition> keyed by name
  - [ ] 8.2 Implement `load_catalog(location: &Path) -> CatalogIndex` scanning .ffs files, parsing each, skipping invalid with WARN log
  - [ ] 8.3 Implement alphabetical sorting for list operations
  - [ ] 8.4 Implement file-watcher integration: detect .ffs file changes via VFS watcher, reload affected definitions within 2 seconds
  - [ ] 8.5 Implement index refresh on file add/modify/remove detected by watcher
  - [ ] 8.6 Write unit tests for index loading, invalid file skipping, alphabetical ordering, and watcher-triggered reload
  - Covers: Requirement 1 (AC 1.1, 1.2), Requirement 3 (AC 3.10), Requirement 4 (AC 4.10)

- [ ] 9. Catalog CRUD operations — create and read
  - [ ] 9.1 Implement `create(def: StructureDefinition) -> Result<(), CatalogError>` with validation, write to Active_Catalog_Location
  - [ ] 9.2 Implement name uniqueness enforcement on create: reject with error if name already exists
  - [ ] 9.3 Implement `read(name: &str) -> Result<StructureDefinition, CatalogError>` returning parsed definition or error
  - [ ] 9.4 Implement `list() -> Vec<StructureDefinition>` returning all valid definitions sorted alphabetically
  - [ ] 9.5 Implement DEBUG-level log on success, WARN-level log on failure for all operations
  - [ ] 9.6 Write unit tests for create (success, duplicate rejection), read (found, not-found), list (sorted, empty)
  - Covers: Requirement 3 (AC 3.1, 3.2, 3.6, 3.9)

- [ ] 10. Catalog CRUD operations — update, delete, duplicate
  - [ ] 10.1 Implement `update(def: StructureDefinition) -> Result<(), CatalogError>` with version increment, validation, and write
  - [ ] 10.2 Implement `delete(name: &str, confirmed: bool) -> Result<(), CatalogError>` with confirmation requirement
  - [ ] 10.3 Implement unconfirmed delete rejection with descriptive error
  - [ ] 10.4 Implement `duplicate(source_name: &str, new_name: &str) -> Result<(), CatalogError>` with version reset to 1
  - [ ] 10.5 Implement duplicate name collision check for the new name
  - [ ] 10.6 Write unit tests for update (version increment, validation failure), delete (confirmed, unconfirmed), duplicate (success, collision)
  - Covers: Requirement 3 (AC 3.3, 3.4, 3.5, 3.7), Requirement 9 (AC 9.2, 9.7)

- [ ] 11. Catalog browsing panel — data model and state
  - [ ] 11.1 Define `BrowsingPanelState` struct: filtered list, search text, sort mode, selected index, preview content
  - [ ] 11.2 Define `SortMode` enum: ByName, ByModifiedDate, ByFieldCount
  - [ ] 11.3 Implement real-time substring filtering (case-insensitive) against name, field names, and file patterns
  - [ ] 11.4 Implement sort switching logic
  - [ ] 11.5 Implement preview generation: display Record_Structure names with field layouts on selection
  - [ ] 11.6 Implement auto-refresh on catalog index change (watcher notification)
  - [ ] 11.7 Write unit tests for filtering, sorting, preview generation, and refresh behavior
  - Covers: Requirement 4 (AC 4.1–4.5, 4.8, 4.10)

- [ ] 12. Catalog browsing panel — actions and toolbar
  - [ ] 12.1 Implement context menu actions model: OpenInEditor, ApplyToCurrentFile, Duplicate, Export, Delete
  - [ ] 12.2 Implement toolbar actions model: NewStructure, Import, Refresh, LocationSelector
  - [ ] 12.3 Implement Catalog_Location selector: switch active location and trigger reload
  - [ ] 12.4 Implement panel registration with `layout-and-docking` system as dockable panel
  - [ ] 12.5 Implement command `catalog.browse` for opening the panel
  - [ ] 12.6 Write unit tests for action dispatch, location switching, and panel registration
  - Covers: Requirement 4 (AC 4.6, 4.7, 4.8, 4.9)

- [ ] 13. Structure editor — field grid model
  - [ ] 13.1 Define `EditorState` struct: active definition, dirty flag, selected record_structure tab, field list, validation errors
  - [ ] 13.2 Implement add-field action: insert at position with defaults (empty name, next offset, length 1, alphanumeric)
  - [ ] 13.3 Implement remove-field action: delete selected row, retain original offsets
  - [ ] 13.4 Implement reorder-field action: move up/down or drag-drop, update display order without changing offsets
  - [ ] 13.5 Implement "Auto-compute offsets" action: recalculate all offsets sequentially (each = prev.offset + prev.length)
  - [ ] 13.6 Implement field_type dropdown model with packed-decimal/numeric enabling decimals column
  - [ ] 13.7 Implement field validation on save with error cell highlighting model
  - [ ] 13.8 Write unit tests for add/remove/reorder/auto-compute, validation, and type-specific behavior
  - Covers: Requirement 5 (AC 5.1–5.9)

- [ ] 14. Structure editor — multi-structure tabs and dirty tracking
  - [ ] 14.1 Implement multi-tab model: one tab per Record_Structure, add/rename/delete tabs
  - [ ] 14.2 Implement unsaved-changes indicator: compare in-memory vs on-disk state
  - [ ] 14.3 Implement save action: serialize to FFS, write via VFS, increment version, update modified_at
  - [ ] 14.4 Implement discard action: reload from disk, reset dirty flag
  - [ ] 14.5 Implement close/switch prompt when dirty (save, discard, cancel)
  - [ ] 14.6 Implement command `catalog.edit_structure` for opening editor with a named structure
  - [ ] 14.7 Write unit tests for tab management, dirty tracking, save/discard, and version increment
  - Covers: Requirement 5 (AC 5.10, 5.11, 5.12), Requirement 9 (AC 9.2, 9.4, 9.5)

- [ ] 15. Auto-association — file pattern matching
  - [ ] 15.1 Implement `FileAssociationMap` struct: HashMap<glob_pattern, structure_name> built from all definitions
  - [ ] 15.2 Implement map building at startup and on catalog reload by scanning all file_patterns
  - [ ] 15.3 Implement conflict detection: same pattern in multiple definitions — WARN log, use first alphabetically
  - [ ] 15.4 Implement `match_file(filename: &str) -> AssociationResult` returning None, Single(name), or Multiple(names)
  - [ ] 15.5 Implement auto-apply on file open: Single match → apply and activate FileForge_Mode with status message
  - [ ] 15.6 Implement multi-match handling: present structure selector to operator
  - [ ] 15.7 Implement no-match handling: open in standard mode without error
  - [ ] 15.8 Implement respect for `catalog.auto_associate` config flag (skip check when false)
  - [ ] 15.9 Write unit tests for glob matching, conflict detection, single/multi/no-match scenarios, and config disable
  - Covers: Requirement 10 (AC 10.1–10.9)

- [ ] 16. Auto-association — pattern management in editor
  - [ ] 16.1 Implement editable file_patterns section in Structure_Editor model
  - [ ] 16.2 Implement add/edit/remove pattern actions with glob syntax validation
  - [ ] 16.3 Write unit tests for pattern CRUD and validation
  - Covers: Requirement 10 (AC 10.10)

- [ ] 17. Manual association command (APPLY STRUCTURE)
  - [ ] 17.1 Register `APPLY STRUCTURE` primary command with `command-framework` (command ID: `catalog.apply_structure`)
  - [ ] 17.2 Implement no-argument mode: open structure selector dialog with search/filter
  - [ ] 17.3 Implement named-argument mode: look up by name, apply directly, error if not found
  - [ ] 17.4 Implement FileForge_Mode activation/switch on successful apply
  - [ ] 17.5 Implement companion config override message: note that catalog structure overrides file-local config for session
  - [ ] 17.6 Implement optional offer to save association as File_Pattern_Mask in the definition's .ffs file
  - [ ] 17.7 Implement no-active-file error when command issued without open file
  - [ ] 17.8 Write unit tests for both modes, apply logic, override messaging, and error cases
  - Covers: Requirement 11 (AC 11.1–11.7)

- [ ] 18. Grid browse mode — data model
  - [ ] 18.1 Define `GridBrowseState` struct: records (Vec<GridRow>), column_defs (from active Record_Structure), scroll position
  - [ ] 18.2 Define `GridRow` enum: Matched { fields: Vec<CellValue> } | Unmatched { raw_text: String }
  - [ ] 18.3 Implement record parsing using active Record_Structure: extract field bytes, decode via FieldTypeHandler
  - [ ] 18.4 Implement decimal display: packed-decimal and numeric fields with decimals > 0 shown with decimal point
  - [ ] 18.5 Implement non-matching record display: full-width raw text with [NO MATCH] indicator
  - [ ] 18.6 Implement record number column (1-based, fixed leftmost)
  - [ ] 18.7 Implement keyboard navigation model: arrow keys, Page Up/Down, Home/End
  - [ ] 18.8 Implement column resize model via drag handles
  - [ ] 18.9 Implement field detail on row click: offset, length, raw bytes, decoded value
  - [ ] 18.10 Write unit tests for record parsing, decimal display, non-matching records, and navigation
  - Covers: Requirement 12 (AC 12.1–12.9)

- [ ] 19. Grid edit mode — data model and edit buffer
  - [ ] 19.1 Define `GridEditState` struct extending GridBrowseState with edit_buffer (HashMap<(row, col), EditedValue>)
  - [ ] 19.2 Implement cell activation: display current value in inline edit widget model
  - [ ] 19.3 Implement field value validation against declared field_type on cell deactivation
  - [ ] 19.4 Implement invalid value highlighting with error indicator model
  - [ ] 19.5 Implement modified-record visual distinction tracking
  - [ ] 19.6 Implement non-matching record read-only enforcement
  - [ ] 19.7 Write unit tests for cell editing, validation, buffer tracking, and non-matching exclusion
  - Covers: Requirement 13 (AC 13.1–13.5)

- [ ] 20. Grid edit mode — save, undo, and encoding
  - [ ] 20.1 Implement undo/redo integration: group field edits within same record as single transaction
  - [ ] 20.2 Implement SAVE command: flush edit buffer, merge with original bytes, write via temp-file + atomic rename
  - [ ] 20.3 Implement packed-decimal re-encoding: pack displayed decimal value back to COMP-3 format
  - [ ] 20.4 Implement field padding: right-pad spaces (alphanumeric), left-pad zeros (numeric) when value shorter than length
  - [ ] 20.5 Implement field truncation with warning when value exceeds defined length
  - [ ] 20.6 Implement CANCEL/close prompt with unsaved grid edits (save, discard)
  - [ ] 20.7 Write unit tests for save merge, COMP-3 encoding, padding, truncation, and undo grouping
  - Covers: Requirement 13 (AC 13.6–13.11)

- [ ] 21. Structure import
  - [ ] 21.1 Implement import action accessible via command `catalog.import` and browsing panel toolbar
  - [ ] 21.2 Implement `.fc.json` import: parse via fileforge-integration config parser, convert to StructureDefinition, write as .ffs
  - [ ] 21.3 Implement `.fc.xlsx` import: parse via fileforge-integration Excel parser, convert to StructureDefinition, write as .ffs
  - [ ] 21.4 Implement `.ffs` import from different location: copy to Active_Catalog_Location
  - [ ] 21.5 Implement name collision handling: prompt operator to rename, overwrite, or cancel
  - [ ] 21.6 Implement non-modification guarantee: original source file is never modified or moved
  - [ ] 21.7 Implement success handling: refresh catalog, highlight newly imported definition
  - [ ] 21.8 Implement failure handling: error message, no partial file creation
  - [ ] 21.9 Implement "Promote to Catalog" action for files open with companion .fc.json
  - [ ] 21.10 Write unit tests for each import format, collision handling, error paths, and promote action
  - Covers: Requirement 7 (AC 7.1–7.10)

- [ ] 22. Structure export
  - [ ] 22.1 Implement export action via command `catalog.export`, browsing panel context menu, and editor toolbar
  - [ ] 22.2 Implement format choice model: .ffs (TOML native), .fc.json (legacy JSON), .fc.xlsx (Excel)
  - [ ] 22.3 Implement .fc.json export via fileforge-integration config serializer
  - [ ] 22.4 Implement .fc.xlsx export via fileforge-integration Excel config writer
  - [ ] 22.5 Implement .ffs export: write native TOML to specified destination
  - [ ] 22.6 Implement destination path selection with default to Active_Catalog_Location
  - [ ] 22.7 Implement success status message with output file path and format
  - [ ] 22.8 Implement failure error message on I/O or serialization error
  - [ ] 22.9 Write unit tests for each export format, destination handling, and error paths
  - Covers: Requirement 8 (AC 8.1–8.8)

- [ ] 23. Structure versioning
  - [ ] 23.1 Implement version auto-increment on every save operation
  - [ ] 23.2 Implement created_at timestamp assignment on first creation (ISO 8601)
  - [ ] 23.3 Implement modified_at timestamp update on every save (ISO 8601)
  - [ ] 23.4 Implement external modification conflict detection: compare on-disk modified_at with loaded value
  - [ ] 23.5 Implement conflict resolution prompt: reload from disk or overwrite
  - [ ] 23.6 Implement version/modified_at display in Catalog_Browsing_Panel list view
  - [ ] 23.7 Implement duplicate version reset: new copy gets version 1, new created_at, cleared modified_at
  - [ ] 23.8 Write unit tests for version increment, timestamp assignment, conflict detection, and duplicate reset
  - Covers: Requirement 9 (AC 9.1–9.7)

- [ ] 24. Catalog location management
  - [ ] 24.1 Implement `CatalogLocationManager` accessible via command `catalog.manage_locations` and browsing panel toolbar
  - [ ] 24.2 Implement add-location: verify path exists and is readable, reject with error if not
  - [ ] 24.3 Implement remove-location: remove from list without deleting directory or contents
  - [ ] 24.4 Implement rename-location: update display label
  - [ ] 24.5 Implement set-active-location: designate any configured location as Active, trigger panel reload
  - [ ] 24.6 Implement persistence via configuration-system user-layer file under [catalog] table
  - [ ] 24.7 Implement startup with no config: initialise with default location, empty list, no error
  - [ ] 24.8 Implement startup with missing path: WARN log, mark unavailable, continue with others
  - [ ] 24.9 Write unit tests for add/remove/rename/set-active, persistence, startup scenarios
  - Covers: Requirement 14 (AC 14.1–14.10)

- [ ] 25. Configuration keys
  - [ ] 25.1 Define configuration schema: catalog.locations (array), catalog.active_location (string), catalog.auto_associate (bool, default true), catalog.default_field_type (string, default "alphanumeric")
  - [ ] 25.2 Implement active_location fallback: if path does not exist, emit warning and use default user-level location
  - [ ] 25.3 Implement auto_associate disable: skip auto-association on file open when false
  - [ ] 25.4 Implement hot-reload integration: changes to [catalog] keys take effect within 2 seconds
  - [ ] 25.5 Implement layer precedence: Defaults → System → User → Profile → Project → Workspace
  - [ ] 25.6 Write unit tests for config loading, fallback, hot-reload, and layer override
  - Covers: Requirement 15 (AC 15.1–15.5), Requirement 1 (AC 1.2, 1.7)

- [ ] 26. Command registration
  - [ ] 26.1 Register command `catalog.create` routed to CRUD create operation
  - [ ] 26.2 Register command `catalog.read` routed to CRUD read operation
  - [ ] 26.3 Register command `catalog.update` routed to CRUD update operation
  - [ ] 26.4 Register command `catalog.delete` routed to CRUD delete operation
  - [ ] 26.5 Register command `catalog.list` routed to CRUD list operation
  - [ ] 26.6 Register command `catalog.duplicate` routed to CRUD duplicate operation
  - [ ] 26.7 Register command `catalog.browse` routed to browsing panel open
  - [ ] 26.8 Register command `catalog.edit_structure` routed to structure editor open
  - [ ] 26.9 Register command `catalog.import` routed to import action
  - [ ] 26.10 Register command `catalog.export` routed to export action
  - [ ] 26.11 Register command `catalog.apply_structure` routed to manual association
  - [ ] 26.12 Register command `catalog.manage_locations` routed to location manager
  - [ ] 26.13 Write unit tests for command registration and dispatch to correct handlers
  - Covers: Requirement 3 (AC 3.8), Requirement 4 (AC 4.9), Requirement 5 (AC 5.12), Requirement 7 (AC 7.1), Requirement 8 (AC 8.1), Requirement 11 (AC 11.1), Requirement 14 (AC 14.1)

- [ ] 27. COBOL copybook parser (structure import from copybook)
  - [ ] 27.1 Implement basic COBOL copybook level-number and PIC clause parsing for field extraction
  - [ ] 27.2 Implement PIC X(n) → alphanumeric field mapping with correct length
  - [ ] 27.3 Implement PIC 9(n) → numeric field mapping with implied decimals from V position
  - [ ] 27.4 Implement COMP-3 (PACKED-DECIMAL) USAGE clause → packed-decimal field type
  - [ ] 27.5 Implement BINARY/COMP USAGE clause → binary field type
  - [ ] 27.6 Implement REDEFINES handling: create separate Record_Structures for redefined groups
  - [ ] 27.7 Implement offset calculation from level hierarchy and field lengths
  - [ ] 27.8 Implement conversion to StructureDefinition with appropriate metadata
  - [ ] 27.9 Register copybook import as an additional format in the import file picker (.cpy, .cbl extensions)
  - [ ] 27.10 Write unit tests for PIC clause parsing, COMP-3 detection, REDEFINES, and offset calculation
  - Covers: Requirement 7 (extends import capability for mainframe-origin structures)

- [ ] 28. Error types
  - [ ] 28.1 Define `CatalogError` enum with variants: NotFound, DuplicateName, ValidationFailed, IoError, ParseError, SchemaError, PermissionDenied, ConfigError, ImportError, ExportError, ConflictDetected
  - [ ] 28.2 Implement `Display` and `thiserror::Error` derives with descriptive context messages
  - [ ] 28.3 Implement `From` conversions for std::io::Error, toml::de::Error, and upstream crate errors
  - [ ] 28.4 Write unit tests for error display output and conversion paths
  - Covers: All requirements (error paths)

- [ ] 29. Property-based tests
  - [ ] 29.1 Write PBT: FFS serialization/deserialization round-trip property
  - [ ] 29.2 Write PBT: Field validation invariant property
  - [ ] 29.3 Write PBT: Packed-decimal encode/decode round-trip property
  - [ ] 29.4 Write PBT: Auto-compute offsets contiguity property
  - [ ] 29.5 Write PBT: Catalog name uniqueness enforcement property
  - [ ] 29.6 Write PBT: Version monotonic increment property
  - [ ] 29.7 Write PBT: File pattern glob matching correctness property
  - [ ] 29.8 Write PBT: Grid field extraction alignment property
  - [ ] 29.9 Write PBT: Field padding/truncation length preservation property
  - [ ] 29.10 Write PBT: COBOL PIC clause offset calculation property
  - Covers: All requirements (property-based validation)

- [ ] 30. Integration tests
  - [ ] 30.1 Write integration test: full catalog lifecycle (create → read → update → list → duplicate → delete)
  - [ ] 30.2 Write integration test: import .fc.json → browse → apply to file → grid browse
  - [ ] 30.3 Write integration test: structure editor round-trip (open → edit fields → save → reload → verify)
  - [ ] 30.4 Write integration test: auto-association on file open with single match
  - [ ] 30.5 Write integration test: multi-location catalog with conflicting patterns
  - [ ] 30.6 Write integration test: grid edit mode → save → verify file bytes
  - [ ] 30.7 Write integration test: export to .fc.json → re-import → verify equivalence
  - [ ] 30.8 Write integration test: configuration hot-reload of catalog.active_location
  - [ ] 30.9 Write integration test: COBOL copybook import → verify fields and offsets
  - Covers: Cross-requirement integration validation

---

## Property-Based Test Definitions

### Property 1: FFS Serialization/Deserialization Round-Trip

**Validates: Requirement 2.1**

- **Statement:** For any valid `StructureDefinition`, serializing to TOML via `FfsSerializer` and parsing back via `FfsParser` produces a definition equal to the original.
- **Strategy:** Generate:
  - StructureDefinition with 1–5 RecordStructures, each with 1–20 FieldDefinitions
  - Field names: non-empty alphanumeric strings (1–30 chars)
  - Offsets: non-negative integers (0–10000)
  - Lengths: positive integers (1–500)
  - Field types: uniform selection from all FieldType variants
  - Decimals: 0–9 (only for numeric/packed-decimal)
  - Metadata: valid name, version 1–1000, optional encoding/lrecl/recfm
- **Invariant:** `parse(serialize(def)) == def` (structural equality)

### Property 2: Field Validation Invariant

**Validates: Requirement 5.9**

- **Statement:** `FieldDefinition::validate()` returns Ok if and only if: name is non-empty, offset >= 0, length >= 1, field_type is a valid enum variant, and decimals >= 0. Any violation returns Err with specific failure details.
- **Strategy:** Generate:
  - Valid fields: all constraints satisfied
  - Invalid fields: one or more constraints violated (empty name, zero length, etc.)
- **Invariant:** `validate().is_ok() ⟺ all constraints hold`

### Property 3: Packed-Decimal Encode/Decode Round-Trip

**Validates: Requirement 6.3, 6.6**

- **Statement:** For any valid signed decimal value within the range representable by a given packed-decimal field length, encoding to COMP-3 and decoding back produces the original value.
- **Strategy:** Generate:
  - Field length: 1–8 bytes (representing 1–15 digits)
  - Decimal value: signed integer within ±(10^(digits) - 1)
  - Decimals: 0–4
- **Invariant:** `decode(encode(value, length, decimals), length, decimals) == value`

### Property 4: Auto-Compute Offsets Contiguity

**Validates: Requirement 5.5**

- **Statement:** After applying "Auto-compute offsets" to any ordered list of FieldDefinitions, every field's offset equals the sum of all preceding field lengths, and the first field has offset 0.
- **Strategy:** Generate:
  - Field lists of length 1–50 with arbitrary initial offsets and lengths 1–100
- **Invariant:** `∀i: fields[i].offset == Σ(fields[0..i].length)` and `fields[0].offset == 0`

### Property 5: Catalog Name Uniqueness Enforcement

**Validates: Requirement 2.4, Requirement 3.1**

- **Statement:** For any sequence of create operations, the catalog shall contain at most one definition per unique name. Any create with a duplicate name shall fail and leave the catalog unchanged.
- **Strategy:** Generate:
  - Sequences of 5–30 create attempts with names drawn from a pool of 3–10 unique names
  - Track expected catalog contents after each operation
- **Invariant:** `catalog.list().len() == unique_successful_creates` and duplicate attempts return Err(DuplicateName)

### Property 6: Version Monotonic Increment

**Validates: Requirement 9.1, 9.2**

- **Statement:** For any sequence of update operations on a StructureDefinition, the version number is strictly monotonically increasing: each save produces version = previous_version + 1.
- **Strategy:** Generate:
  - Initial version: positive integer 1–100
  - Number of updates: 1–50
  - Each update modifies at least one field (name change, field add/remove)
- **Invariant:** After N updates from initial version V, `definition.version == V + N`

### Property 7: File Pattern Glob Matching Correctness

**Validates: Requirement 10.1, 10.3**

- **Statement:** For any filename and set of glob patterns, `match_file` returns the correct set of matching patterns according to standard glob semantics (wildcard `*`, single-char `?`, character classes).
- **Strategy:** Generate:
  - Filenames: strings of 3–30 chars from [a-z0-9._-]
  - Patterns: valid glob strings using *, ?, and character literals
  - Use a reference glob implementation to determine expected matches
- **Invariant:** `match_file(filename) == reference_glob_match(filename, patterns)` for all generated inputs

### Property 8: Grid Field Extraction Alignment

**Validates: Requirement 12.3**

- **Statement:** For any record bytes and Record_Structure, extracting field values from the record produces byte slices at the exact (offset, offset+length) positions defined in the FieldDefinitions, with no overlap or gap violations.
- **Strategy:** Generate:
  - Record bytes: Vec<u8> of length 10–500
  - Record_Structure with 1–20 fields at valid non-overlapping offsets within record bounds
- **Invariant:** `∀field: extracted_bytes(field) == record[field.offset..field.offset+field.length]`

### Property 9: Field Padding/Truncation Length Preservation

**Validates: Requirement 13.9, 13.10**

- **Statement:** For any field value written via the grid edit mode, the resulting byte representation has exactly the declared field length — shorter values are padded, longer values are truncated.
- **Strategy:** Generate:
  - Field length: 1–100
  - Input values: strings of length 0–200
  - Field types: alphanumeric (right-pad spaces) or numeric (left-pad zeros)
- **Invariant:** `encode(value, field_def).len() == field_def.length` for all inputs

### Property 10: COBOL PIC Clause Offset Calculation

**Validates: Requirement 27 (COBOL copybook import)**

- **Statement:** For any valid COBOL copybook with contiguous fields at a given group level, the computed offsets are sequential: each field starts immediately after the previous field ends.
- **Strategy:** Generate:
  - PIC clauses: X(1–50), 9(1–18), 9(n)V9(m), S9(n) COMP-3
  - Compute expected byte lengths per PIC type
  - Generate copybook text with 3–20 fields
- **Invariant:** `∀i > 0: field[i].offset == field[i-1].offset + field[i-1].length` and `field[0].offset == 0`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Models and Types", "tasks": ["2", "3", "4", "28"], "dependsOn": [0] },
    { "id": 2, "label": "FFS Format (Serialization)", "tasks": ["5", "6"], "dependsOn": [1] },
    { "id": 3, "label": "Catalog Store and Persistence", "tasks": ["7", "8"], "dependsOn": [2] },
    { "id": 4, "label": "CRUD Operations", "tasks": ["9", "10"], "dependsOn": [3] },
    { "id": 5, "label": "Browsing Panel and Editor", "tasks": ["11", "12", "13", "14"], "dependsOn": [4] },
    { "id": 6, "label": "Association and Commands", "tasks": ["15", "16", "17", "25", "26"], "dependsOn": [5] },
    { "id": 7, "label": "Grid Modes", "tasks": ["18", "19", "20"], "dependsOn": [5, 6] },
    { "id": 8, "label": "Import, Export, and Copybook", "tasks": ["21", "22", "27"], "dependsOn": [4, 6] },
    { "id": 9, "label": "Versioning and Location Management", "tasks": ["23", "24"], "dependsOn": [4] },
    { "id": 10, "label": "Property-Based and Integration Tests", "tasks": ["29", "30"], "dependsOn": [7, 8, 9] }
  ]
}
```

---

## Notes

- This is a Wave 12 (FileForge Domain) crate depending on multiple upstream crates: ff-logging (Wave 0), ff-command (Wave 2), ff-config (Wave 2), ff-vfs (Wave 3), ff-layout (Wave 2), ff-plugin (Wave 2), and ff-fileforge (Wave 12, sibling)
- The `ff-fileforge` crate provides the record parsing engine, field extraction, COMP-3 encoding/decoding, and EBCDIC transcoding that `ff-structure-catalog` consumes for grid display and editing
- The COBOL copybook parser (Task 27) is a value-add feature for mainframe migration scenarios; it extends the import capability beyond the core .fc.json/.fc.xlsx formats
- The Grid Browse/Edit modes (Tasks 18–20) define the data model and logic only; actual rendering is handled by the GUI shell layer consuming this crate's API
- The Catalog_Browsing_Panel (Tasks 11–12) and Structure_Editor (Tasks 13–14) likewise define state/logic; UI rendering is provided by the egui shell
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The file-watcher for catalog reload (Task 8.4) integrates with the VFS watcher abstraction; initial implementation uses polling with configurable interval
- Configuration hot-reload (Task 25.4) depends on the `configuration-system` change notification mechanism
- All CRUD operations go through the command framework (Task 26) to support scripting, history, and undo integration
- The `FieldTypeHandler` trait (Task 4) enables future plugins to register custom field types without modifying the catalog crate
- Thread safety for the `CatalogIndex` uses `Arc<RwLock<...>>` for concurrent read access during browsing and editing
- The "Promote to Catalog" action (Task 21.9) bridges the per-file companion config model from `ff-fileforge` to the centralized catalog model

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Catalog Persistent Store | AC 1.1 | Tasks 7, 8 |
| Req 1: Catalog Persistent Store | AC 1.2 | Tasks 25, 8 |
| Req 1: Catalog Persistent Store | AC 1.3 | Task 7 |
| Req 1: Catalog Persistent Store | AC 1.4 | Task 7 |
| Req 1: Catalog Persistent Store | AC 1.5 | Task 7 |
| Req 1: Catalog Persistent Store | AC 1.6 | Task 7 |
| Req 1: Catalog Persistent Store | AC 1.7 | Task 25 |
| Req 1: Catalog Persistent Store | AC 1.8 | Task 7 |
| Req 2: Structure File Format | AC 2.1 | Tasks 5, 6 |
| Req 2: Structure File Format | AC 2.2 | Tasks 5, 6 |
| Req 2: Structure File Format | AC 2.3 | Task 6 |
| Req 2: Structure File Format | AC 2.4 | Tasks 6, 9 |
| Req 2: Structure File Format | AC 2.5 | Task 6 |
| Req 2: Structure File Format | AC 2.6 | Task 6 |
| Req 2: Structure File Format | AC 2.7 | Task 5 |
| Req 2: Structure File Format | AC 2.8 | Task 5 |
| Req 2: Structure File Format | AC 2.9 | Task 5 |
| Req 3: Catalog CRUD | AC 3.1 | Task 9 |
| Req 3: Catalog CRUD | AC 3.2 | Task 9 |
| Req 3: Catalog CRUD | AC 3.3 | Task 10 |
| Req 3: Catalog CRUD | AC 3.4 | Task 10 |
| Req 3: Catalog CRUD | AC 3.5 | Task 10 |
| Req 3: Catalog CRUD | AC 3.6 | Task 9 |
| Req 3: Catalog CRUD | AC 3.7 | Task 10 |
| Req 3: Catalog CRUD | AC 3.8 | Task 26 |
| Req 3: Catalog CRUD | AC 3.9 | Task 9 |
| Req 3: Catalog CRUD | AC 3.10 | Task 8 |
| Req 4: Catalog Browsing Panel | AC 4.1 | Tasks 11, 12 |
| Req 4: Catalog Browsing Panel | AC 4.2 | Task 11 |
| Req 4: Catalog Browsing Panel | AC 4.3 | Task 11 |
| Req 4: Catalog Browsing Panel | AC 4.4 | Task 11 |
| Req 4: Catalog Browsing Panel | AC 4.5 | Task 11 |
| Req 4: Catalog Browsing Panel | AC 4.6 | Task 12 |
| Req 4: Catalog Browsing Panel | AC 4.7 | Task 12 |
| Req 4: Catalog Browsing Panel | AC 4.8 | Tasks 11, 12 |
| Req 4: Catalog Browsing Panel | AC 4.9 | Tasks 12, 26 |
| Req 4: Catalog Browsing Panel | AC 4.10 | Tasks 8, 11 |
| Req 5: Structure Editor | AC 5.1 | Task 13 |
| Req 5: Structure Editor | AC 5.2 | Task 13 |
| Req 5: Structure Editor | AC 5.3 | Task 13 |
| Req 5: Structure Editor | AC 5.4 | Task 13 |
| Req 5: Structure Editor | AC 5.5 | Task 13 |
| Req 5: Structure Editor | AC 5.6 | Task 13 |
| Req 5: Structure Editor | AC 5.7 | Task 13 |
| Req 5: Structure Editor | AC 5.8 | Task 13 |
| Req 5: Structure Editor | AC 5.9 | Tasks 3, 13 |
| Req 5: Structure Editor | AC 5.10 | Task 14 |
| Req 5: Structure Editor | AC 5.11 | Task 14 |
| Req 5: Structure Editor | AC 5.12 | Tasks 14, 26 |
| Req 6: Field Types | AC 6.1 | Tasks 3, 4 |
| Req 6: Field Types | AC 6.2 | Tasks 3, 4 |
| Req 6: Field Types | AC 6.3 | Tasks 3, 4 |
| Req 6: Field Types | AC 6.4 | Tasks 3, 4 |
| Req 6: Field Types | AC 6.5 | Tasks 3, 4 |
| Req 6: Field Types | AC 6.6 | Tasks 3, 4 |
| Req 6: Field Types | AC 6.7 | Tasks 3, 4 |
| Req 6: Field Types | AC 6.8 | Task 3 |
| Req 6: Field Types | AC 6.9 | Task 4 |
| Req 7: Structure Import | AC 7.1 | Tasks 21, 26 |
| Req 7: Structure Import | AC 7.2 | Task 21 |
| Req 7: Structure Import | AC 7.3 | Task 21 |
| Req 7: Structure Import | AC 7.4 | Task 21 |
| Req 7: Structure Import | AC 7.5 | Task 21 |
| Req 7: Structure Import | AC 7.6 | Task 21 |
| Req 7: Structure Import | AC 7.7 | Task 21 |
| Req 7: Structure Import | AC 7.8 | Task 21 |
| Req 7: Structure Import | AC 7.9 | Task 21 |
| Req 7: Structure Import | AC 7.10 | Task 21 |
| Req 8: Structure Export | AC 8.1 | Tasks 22, 26 |
| Req 8: Structure Export | AC 8.2 | Task 22 |
| Req 8: Structure Export | AC 8.3 | Task 22 |
| Req 8: Structure Export | AC 8.4 | Task 22 |
| Req 8: Structure Export | AC 8.5 | Task 22 |
| Req 8: Structure Export | AC 8.6 | Task 22 |
| Req 8: Structure Export | AC 8.7 | Task 22 |
| Req 8: Structure Export | AC 8.8 | Task 22 |
| Req 9: Structure Versioning | AC 9.1 | Tasks 2, 23 |
| Req 9: Structure Versioning | AC 9.2 | Tasks 10, 14, 23 |
| Req 9: Structure Versioning | AC 9.3 | Tasks 2, 23 |
| Req 9: Structure Versioning | AC 9.4 | Tasks 14, 23 |
| Req 9: Structure Versioning | AC 9.5 | Task 23 |
| Req 9: Structure Versioning | AC 9.6 | Task 23 |
| Req 9: Structure Versioning | AC 9.7 | Tasks 10, 23 |
| Req 10: Auto-Association | AC 10.1 | Task 15 |
| Req 10: Auto-Association | AC 10.2 | Task 15 |
| Req 10: Auto-Association | AC 10.3 | Task 15 |
| Req 10: Auto-Association | AC 10.4 | Task 15 |
| Req 10: Auto-Association | AC 10.5 | Task 15 |
| Req 10: Auto-Association | AC 10.6 | Task 15 |
| Req 10: Auto-Association | AC 10.7 | Task 15 |
| Req 10: Auto-Association | AC 10.8 | Task 15 |
| Req 10: Auto-Association | AC 10.9 | Task 15 |
| Req 10: Auto-Association | AC 10.10 | Task 16 |
| Req 11: Manual Association | AC 11.1 | Tasks 17, 26 |
| Req 11: Manual Association | AC 11.2 | Task 17 |
| Req 11: Manual Association | AC 11.3 | Task 17 |
| Req 11: Manual Association | AC 11.4 | Task 17 |
| Req 11: Manual Association | AC 11.5 | Task 17 |
| Req 11: Manual Association | AC 11.6 | Task 17 |
| Req 11: Manual Association | AC 11.7 | Task 17 |
| Req 12: Grid Browse Mode | AC 12.1 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.2 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.3 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.4 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.5 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.6 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.7 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.8 | Task 18 |
| Req 12: Grid Browse Mode | AC 12.9 | Task 18 |
| Req 13: Grid Edit Mode | AC 13.1 | Task 19 |
| Req 13: Grid Edit Mode | AC 13.2 | Task 19 |
| Req 13: Grid Edit Mode | AC 13.3 | Task 19 |
| Req 13: Grid Edit Mode | AC 13.4 | Task 19 |
| Req 13: Grid Edit Mode | AC 13.5 | Task 19 |
| Req 13: Grid Edit Mode | AC 13.6 | Task 20 |
| Req 13: Grid Edit Mode | AC 13.7 | Task 20 |
| Req 13: Grid Edit Mode | AC 13.8 | Task 20 |
| Req 13: Grid Edit Mode | AC 13.9 | Task 20 |
| Req 13: Grid Edit Mode | AC 13.10 | Task 20 |
| Req 13: Grid Edit Mode | AC 13.11 | Task 20 |
| Req 14: Catalog Location Mgmt | AC 14.1 | Tasks 24, 26 |
| Req 14: Catalog Location Mgmt | AC 14.2 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.3 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.4 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.5 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.6 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.7 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.8 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.9 | Task 24 |
| Req 14: Catalog Location Mgmt | AC 14.10 | Task 24 |
| Req 15: Configuration Keys | AC 15.1 | Task 25 |
| Req 15: Configuration Keys | AC 15.2 | Task 25 |
| Req 15: Configuration Keys | AC 15.3 | Task 25 |
| Req 15: Configuration Keys | AC 15.4 | Task 25 |
| Req 15: Configuration Keys | AC 15.5 | Task 25 |
