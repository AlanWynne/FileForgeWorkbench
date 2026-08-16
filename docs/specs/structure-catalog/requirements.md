# Requirements Document

## Introduction

This feature specifies the **Structure Catalog** for FileForgeWorkbench (`ff-structure-catalog` crate) — a persistent, operator-managed library of named Record_Structure definitions. The catalog provides a central repository of reusable structure definitions that can be applied to any flat-file data file at any time, replacing the need to maintain per-file companion configs.

The structure catalog covers six tightly related capabilities:

1. **Catalog persistent store** — a configurable directory of `.ffs` (FileForge Structure) files in TOML format, persisting named Record_Structure definitions.
2. **Catalog CRUD operations** — create, read, update, and delete structure definitions programmatically and through the UI.
3. **Catalog browsing panel** — a dockable, searchable panel for browsing and selecting structures from the catalog.
4. **Structure editor** — a visual editor for adding, removing, reordering fields and setting types/lengths within a structure definition.
5. **Auto-association** — automatic mapping of file extensions and glob patterns to structure definitions for seamless FileForge_Mode activation.
6. **Structure import/export and versioning** — importing legacy `.fc.json`/`.fc.xlsx` formats, exporting to multiple formats, and tracking structure definition versions.

This spec extends `fileforge-integration` (which owns record parsing, field extraction, and file writing logic) and integrates with `configuration-system` (for catalog path settings), `layout-and-docking` (for panel hosting), `command-framework` (for catalog commands), and `virtual-file-system` (for file access).

**Source references:**
- **FFE-STRUCT** = FileForgeEditor `structure-catalog` spec (15 requirements — catalog management, grid browse/edit, file associations)
- **FFE** = FileForgeEditor `fileforge-integration` spec (field types, record structures, decimal handling)
- **WB** = Workbench Architecture Brief (command-driven, plugin-capable, VFS-aware, dockable panels)

**Cross-references:**
- `fileforge-integration` — record parsing, field extraction, COMP-3 handling, EBCDIC support
- `configuration-system` — catalog path settings, hot-reload of catalog configuration
- `layout-and-docking` — catalog browsing panel as a dockable panel
- `command-framework` — catalog commands (`CATALOG`, `APPLY STRUCTURE`, etc.)
- `virtual-file-system` — file access for structure files and data files

---

## Glossary

- **Structure_Catalog**: The persistent store of Record_Structure definitions, implemented as a directory of `.ffs` files. One or more catalogs may be configured. [FFE-STRUCT]
- **Catalog_Location**: A filesystem path (accessed via VFS) to a Structure_Catalog directory. Multiple locations can be configured; one is designated as the Active_Catalog_Location. [FFE-STRUCT, WB]
- **Active_Catalog_Location**: The currently selected Catalog_Location from which structure definitions are loaded and to which new definitions are saved. [FFE-STRUCT]
- **Structure_Definition**: A single `.ffs` file in a Structure_Catalog representing a named record layout. Contains one or more Record_Structures, metadata, and optional file association patterns. [FFE-STRUCT]
- **Record_Structure**: A named definition describing the field layout for one category of record in a flat file (e.g., "Header", "Detail", "Trailer"). Contains an ordered list of Field_Definitions. [FFE]
- **Field_Definition**: A single field within a Record_Structure, specifying name, offset, length, data type, and optional attributes (decimals, identifiers, filters). [FFE]
- **Field_Type**: The data type of a field. Supported types: `alphanumeric` (character data, default), `numeric` (unsigned integer), `packed-decimal` (IBM COMP-3 packed BCD), `binary` (raw binary bytes), `hex` (hexadecimal display). [FFE, WB]
- **FFS_File**: A `.ffs` (FileForge Structure) file — the TOML-based file format for persisting Structure_Definitions in the catalog. Replaces the legacy `.fc.json` format for catalog use. [WB]
- **File_Pattern_Mask**: A glob pattern (e.g., `*.dat`, `CUST_*.dat`, `INV??????.txt`) or file extension that identifies data files associated with a Structure_Definition. [FFE-STRUCT]
- **Auto_Association**: The automatic mapping of a newly opened file to a Structure_Definition based on its filename matching a File_Pattern_Mask in the catalog. [FFE-STRUCT]
- **Catalog_Browsing_Panel**: A dockable panel in the workbench layout that displays a searchable, filterable list of all Structure_Definitions in the Active_Catalog_Location. [FFE-STRUCT, WB]
- **Structure_Editor**: The visual editor within the catalog browsing panel or a dedicated panel for adding, removing, reordering fields and setting their types and lengths. [FFE-STRUCT]
- **Structure_Version**: A monotonically increasing version number embedded in each `.ffs` file, incremented on each save, enabling change tracking and conflict detection. [WB]
- **Manual_Association_Command**: The `APPLY STRUCTURE` primary command that allows the operator to manually associate a catalog structure with the currently open file. [FFE-STRUCT, WB]
- **Grid_Browse_Mode**: The editor state in which a data file is displayed as a read-only column-per-field grid using the active Record_Structure. [FFE-STRUCT]
- **Grid_Edit_Mode**: The editor state in which individual field cells in the grid are editable, with changes buffered in memory until saved. [FFE-STRUCT]

---

## Requirements

### Requirement 1: Structure Catalog Persistent Store

**User Story:** As an operator, I want a persistent store of Record_Structure definitions organized as a directory of TOML files, so that I can manage, share, and version-control my structure library independently of individual data files.

**Source:** FFE-STRUCT Req 1, WB §8 (configuration as data). [FFE-STRUCT, WB]

#### Acceptance Criteria

1. THE Structure_Catalog SHALL be implemented as a directory containing `.ffs` files, where each `.ffs` file represents one Structure_Definition in TOML format.
2. THE catalog directory location SHALL be configurable via the `configuration-system` under the key `catalog.locations` (array of paths) and `catalog.active_location` (string path designating the Active_Catalog_Location).
3. THE default Active_Catalog_Location SHALL be the platform user-data directory (`~/.config/ffworkbench/catalogs/` on Linux, `%APPDATA%\FFWorkbench\catalogs\` on Windows, `~/Library/Application Support/FFWorkbench/catalogs/` on macOS), created automatically on first use if it does not exist.
4. THE Structure_Catalog SHALL support multiple configured Catalog_Locations simultaneously, enabling project-local, team-shared, and user-global structure libraries.
5. WHEN the Active_Catalog_Location directory does not exist at startup, THE system SHALL create it with appropriate permissions and log an INFO-level message.
6. WHEN a configured Catalog_Location path is inaccessible (permission denied, missing volume), THE system SHALL emit a WARN-level log record, skip that location, and continue loading from other configured locations.
7. THE catalog configuration SHALL persist across workbench restarts via the `configuration-system` user-layer configuration file.
8. THE Structure_Catalog SHALL be accessed through the `virtual-file-system` abstraction, enabling future support for remote catalog locations via VFS providers.

---

### Requirement 2: Structure File Format (.ffs — TOML-Based)

**User Story:** As a data engineer, I want structure definitions stored in a human-readable, version-control-friendly TOML format, so that I can review changes in diffs, edit definitions in any text editor, and share them across teams via source control.

**Source:** WB §8 (TOML as data format), FFE-STRUCT Req 1. [FFE-STRUCT, WB]

#### Acceptance Criteria

1. EACH `.ffs` file SHALL be a valid TOML v1.0 document containing: a `[metadata]` table (name, description, version, created_at, modified_at), an optional `[associations]` table (file_patterns array), and one or more `[[record_structures]]` array-of-tables entries.
2. EACH `[[record_structures]]` entry SHALL contain: `name` (string), and a `[[record_structures.fields]]` array-of-tables with each field specifying `name` (string), `offset` (integer), `length` (integer), `field_type` (string enum: `"alphanumeric"`, `"numeric"`, `"packed-decimal"`, `"binary"`, `"hex"`), and optional keys `decimals` (integer, default 0), `identifiers` (array of strings), and `filters` (array of strings).
3. THE `[metadata].version` key SHALL be a positive integer, monotonically incremented on each save operation against that Structure_Definition.
4. THE `[metadata].name` key SHALL be unique within a single Catalog_Location; THE system SHALL reject saves that would create a name collision and display an error.
5. WHEN an `.ffs` file contains invalid TOML syntax, THE system SHALL reject the file, emit a WARN-level log record with the file path and parse error, and exclude it from the catalog listing.
6. WHEN an `.ffs` file passes TOML parsing but fails schema validation (missing required keys, invalid field_type value, negative offset/length), THE system SHALL emit a WARN-level log record with validation details and exclude the definition from the catalog listing.
7. THE `.ffs` format SHALL support an optional `[metadata].encoding` key (string, e.g., `"utf-8"`, `"ebcdic-037"`) indicating the expected character encoding of data files using this structure.
8. THE `.ffs` format SHALL support an optional `[metadata].lrecl` key (positive integer) indicating the expected logical record length of data files using this structure.
9. THE `.ffs` format SHALL support an optional `[metadata].recfm` key (string enum: `"F"`, `"FB"`, `"V"`, `"FB_BINARY"`, `"VB"`, `"U"`) indicating the expected record format of data files using this structure.

---

### Requirement 3: Catalog CRUD Operations

**User Story:** As a workbench developer or plugin author, I want a programmatic API for creating, reading, updating, and deleting structure definitions in the catalog, so that automation workflows and plugins can manage structures without GUI interaction.

**Source:** FFE-STRUCT Reqs 2, 3, 14. [FFE-STRUCT, WB]

#### Acceptance Criteria

1. THE Structure_Catalog SHALL provide a `create` operation that accepts a Structure_Definition, validates it, writes it as an `.ffs` file to the Active_Catalog_Location, and returns a success/error result.
2. THE Structure_Catalog SHALL provide a `read` operation that accepts a structure name and returns the parsed Structure_Definition, or an error if the name does not exist in the Active_Catalog_Location.
3. THE Structure_Catalog SHALL provide an `update` operation that accepts a modified Structure_Definition, increments its version number, validates it, and writes the updated `.ffs` file to disk, or returns an error if validation fails.
4. THE Structure_Catalog SHALL provide a `delete` operation that accepts a structure name, removes the corresponding `.ffs` file from the Active_Catalog_Location, and returns a success/error result.
5. WHEN a `delete` operation is requested, THE system SHALL require confirmation before removing the file; the API SHALL accept a `confirmed: bool` parameter and reject unconfirmed deletions with an error.
6. THE Structure_Catalog SHALL provide a `list` operation that returns all valid Structure_Definitions in the Active_Catalog_Location, sorted alphabetically by name.
7. THE Structure_Catalog SHALL provide a `duplicate` operation that creates a copy of an existing Structure_Definition with a new name, resetting the version to 1.
8. ALL CRUD operations SHALL be routed through the `command-framework` as registered commands (`catalog.create`, `catalog.read`, `catalog.update`, `catalog.delete`, `catalog.list`, `catalog.duplicate`).
9. ALL CRUD operations SHALL emit structured log records via the `logging-subsystem` at DEBUG level on success and WARN level on failure.
10. WHEN an `.ffs` file is modified externally (detected via VFS file-watcher), THE Structure_Catalog SHALL reload the affected definition and update the in-memory catalog index within 2 seconds.

---

### Requirement 4: Catalog Browsing Panel

**User Story:** As an operator, I want a dedicated dockable panel that displays a searchable list of all structure definitions in the active catalog, so that I can quickly find, preview, and apply structures without navigating the filesystem.

**Source:** FFE-STRUCT Req 2, WB (layout-and-docking). [FFE-STRUCT, WB]

#### Acceptance Criteria

1. THE Catalog_Browsing_Panel SHALL be a dockable panel registered with the `layout-and-docking` system, dockable to any edge or floating as a standalone window.
2. THE Catalog_Browsing_Panel SHALL display a list of all valid Structure_Definitions in the Active_Catalog_Location, showing at minimum: structure name, number of Record_Structures, number of fields, and associated File_Pattern_Masks.
3. THE Catalog_Browsing_Panel SHALL provide a search/filter text field that filters the structure list in real-time by substring match against structure name, field names, and File_Pattern_Masks (case-insensitive).
4. THE Catalog_Browsing_Panel SHALL support sorting the list by name (alphabetical), by modification date, or by number of fields.
5. WHEN the operator selects a Structure_Definition in the list, THE panel SHALL display a preview showing the Record_Structure names and their field layouts (field name, offset, length, type) in a read-only summary view.
6. THE Catalog_Browsing_Panel SHALL provide context menu actions: Open in Editor, Apply to Current File, Duplicate, Export, Delete.
7. THE Catalog_Browsing_Panel SHALL provide a toolbar with buttons: New Structure, Import, Refresh, and a Catalog_Location selector dropdown.
8. WHEN the operator switches the Catalog_Location via the dropdown, THE panel SHALL reload to display structures from the newly selected location.
9. THE Catalog_Browsing_Panel SHALL be openable via the `command-framework` command `catalog.browse` and from the workbench menu (View → Structure Catalog).
10. THE Catalog_Browsing_Panel SHALL refresh automatically when `.ffs` files are added, modified, or removed in the Active_Catalog_Location (detected via VFS file-watcher).

---

### Requirement 5: Structure Editor

**User Story:** As a data engineer, I want a visual editor for defining and modifying field layouts within a structure, so that I can add, remove, reorder fields and set their types and lengths without editing TOML by hand.

**Source:** FFE-STRUCT Reqs 2–4, FFE Req 7 (Config_Editor). [FFE-STRUCT, FFE]

#### Acceptance Criteria

1. THE Structure_Editor SHALL display a grid with one row per Field_Definition, showing columns: ordinal position, field name, offset, length, field type, decimals, identifiers, and filters.
2. THE Structure_Editor SHALL allow the operator to add a new field row at any position in the field list, with defaults: empty name, next available offset (previous field offset + length), length 1, type `alphanumeric`, decimals 0.
3. THE Structure_Editor SHALL allow the operator to remove a selected field row, with the remaining fields retaining their original offsets (no auto-recompute unless explicitly triggered).
4. THE Structure_Editor SHALL allow the operator to reorder fields via drag-and-drop or move-up/move-down buttons; reordering SHALL update the display order but SHALL NOT automatically change field offsets.
5. THE Structure_Editor SHALL provide an "Auto-compute offsets" action that recalculates all field offsets sequentially (each field offset = previous field offset + previous field length), useful when fields are meant to be contiguous.
6. THE Structure_Editor SHALL provide a `field_type` dropdown for each field with the following options: `alphanumeric`, `numeric`, `packed-decimal`, `binary`, `hex`.
7. WHEN `field_type` is set to `packed-decimal`, THE Structure_Editor SHALL enable the `decimals` column for that field and display a tooltip explaining COMP-3 packed-decimal encoding.
8. WHEN `field_type` is set to `numeric`, THE Structure_Editor SHALL enable the `decimals` column for that field (supporting implied decimal positions).
9. THE Structure_Editor SHALL validate field definitions on save: field name must be non-empty, offset must be ≥ 0, length must be ≥ 1, field_type must be a valid enum value, decimals must be ≥ 0. Invalid cells SHALL be highlighted with an error indicator.
10. THE Structure_Editor SHALL support editing multiple Record_Structures within a single Structure_Definition, displayed as tabs (one tab per Record_Structure) with the ability to add, rename, and delete Record_Structure tabs.
11. WHEN the in-memory structure differs from the on-disk `.ffs` file, THE Structure_Editor SHALL display an unsaved-changes indicator and prompt the user to save or discard when closing or switching structures.
12. THE Structure_Editor SHALL be openable from the Catalog_Browsing_Panel (double-click or context menu "Open in Editor") and via the command `catalog.edit_structure`.

---

### Requirement 6: Field Types

**User Story:** As a data engineer working with mainframe-originated flat files, I want the structure catalog to support all common field encoding types, so that I can correctly define layouts for files containing character data, numeric data, packed-decimal (COMP-3) fields, binary data, and hexadecimal content.

**Source:** FFE fileforge-integration (COMP-3, EBCDIC, binary), WB. [FFE, WB]

#### Acceptance Criteria

1. THE system SHALL support the `alphanumeric` field type: character data interpreted using the structure's declared encoding (UTF-8 default, or EBCDIC code page if specified). Display shows the decoded text value, padded with spaces to the field length.
2. THE system SHALL support the `numeric` field type: unsigned integer data stored as displayable digit characters (zoned decimal). When `decimals > 0`, the value is interpreted as a fixed-point number with the last N digits as the fractional part.
3. THE system SHALL support the `packed-decimal` field type (COMP-3): IBM packed BCD encoding where each byte holds two decimal digits as nibbles, and the low nibble of the final byte is the sign (`C` = positive, `D` = negative, `F` = unsigned). Display shows the signed decimal value.
4. THE system SHALL support the `binary` field type: raw binary bytes displayed as a hexadecimal string. No character decoding is applied. Field length specifies the number of bytes.
5. THE system SHALL support the `hex` field type: similar to binary, but displayed as a formatted hex dump with optional ASCII sidebar. Useful for fields containing mixed or unknown encodings.
6. WHEN a field has `field_type: "packed-decimal"` and `decimals: N > 0`, THE system SHALL display the unpacked decimal value with N decimal places (e.g., bytes `0x12345C` with decimals=2 displays as `"123.45"`).
7. WHEN a field has `field_type: "numeric"` and `decimals: N > 0`, THE system SHALL display the value with an implied decimal point N positions from the right (e.g., `"12345"` with decimals=2 displays as `"123.45"`).
8. WHEN a packed-decimal field contains invalid nibble values (not 0–9 for digit nibbles, not C/D/F for sign nibble), THE system SHALL display the raw hex bytes and flag the field with a validation warning.
9. THE field type enumeration SHALL be extensible via the `plugin-architecture` trait system, allowing plugins to register custom field type handlers for specialized encodings.

---

### Requirement 7: Structure Import

**User Story:** As a data engineer who has existing companion `.fc.json` or `.fc.xlsx` config files, I want to import them into the Structure Catalog as `.ffs` definitions, so that I can consolidate my structure library without recreating definitions from scratch.

**Source:** FFE-STRUCT Req 14 (import from companion config). [FFE-STRUCT]

#### Acceptance Criteria

1. THE system SHALL provide an import action accessible from the Catalog_Browsing_Panel toolbar and via the command `catalog.import`.
2. WHEN the operator triggers import, THE system SHALL present a file picker filtered to `.fc.json`, `.fc.xlsx`, and `.ffs` files.
3. WHEN a `.fc.json` file is selected for import, THE system SHALL parse it using the `fileforge-integration` config parser, convert the structure to the `.ffs` TOML format, and write the converted file to the Active_Catalog_Location.
4. WHEN a `.fc.xlsx` file is selected for import, THE system SHALL parse it using the `fileforge-integration` Excel config parser, convert the structure to the `.ffs` TOML format, and write the converted file to the Active_Catalog_Location.
5. WHEN an `.ffs` file is selected for import from a different location, THE system SHALL copy it to the Active_Catalog_Location.
6. WHEN importing and a Structure_Definition with the same name already exists in the Active_Catalog_Location, THE system SHALL prompt the operator to: rename the import, overwrite the existing definition, or cancel the operation.
7. THE import operation SHALL NOT modify or move the original source file — it SHALL create a new file in the Active_Catalog_Location.
8. WHEN import succeeds, THE Catalog_Browsing_Panel SHALL refresh and highlight the newly imported Structure_Definition.
9. WHEN import fails (parse error, validation failure, I/O error), THE system SHALL display an error message describing the failure and SHALL NOT create a partial file in the catalog.
10. THE system SHALL provide a "Promote to Catalog" action when a file is open in FileForge_Mode with a companion `.fc.json`, allowing one-click import of the file-local config into the catalog.

---

### Requirement 8: Structure Export

**User Story:** As a data engineer, I want to export a structure definition from the catalog to multiple formats, so that I can share it with colleagues using different tools or use it with the standalone `fforge` CLI.

**Source:** FFE-STRUCT Req 3. [FFE-STRUCT]

#### Acceptance Criteria

1. THE system SHALL provide an export action for each Structure_Definition, accessible from the Catalog_Browsing_Panel context menu, the Structure_Editor toolbar, and via the command `catalog.export`.
2. WHEN the operator triggers export, THE system SHALL present a format choice: `.ffs` (TOML, native format), `.fc.json` (legacy JSON, compatible with `fforge` CLI), or `.fc.xlsx` (Excel, compatible with `fforge` CLI).
3. WHEN `.fc.json` is selected, THE system SHALL convert the Structure_Definition to the legacy JSON format using the `fileforge-integration` config serializer, producing a file compatible with the standalone `fforge` CLI tool.
4. WHEN `.fc.xlsx` is selected, THE system SHALL convert the Structure_Definition to the Excel format using the `fileforge-integration` Excel config writer.
5. WHEN `.ffs` is selected, THE system SHALL write the native TOML format to the specified destination path.
6. THE operator SHALL be able to specify the export destination path via a file-save dialog; the default SHALL be the Active_Catalog_Location.
7. WHEN export succeeds, THE system SHALL display a status message identifying the output file path and format.
8. WHEN export fails (I/O error, serialization error), THE system SHALL display an error message describing the failure.

---

### Requirement 9: Structure Versioning

**User Story:** As an operator managing a shared structure library, I want each structure definition to carry a version number that increments on every change, so that I can track modifications, detect conflicts, and understand when definitions were last updated.

**Source:** WB (version-control-friendly data), FFE-STRUCT Req 11. [FFE-STRUCT, WB]

#### Acceptance Criteria

1. EACH `.ffs` file SHALL contain a `[metadata].version` key with a positive integer value, starting at 1 for newly created definitions.
2. WHEN a Structure_Definition is saved (via the Structure_Editor or programmatic update), THE system SHALL increment the `[metadata].version` value by 1.
3. EACH `.ffs` file SHALL contain `[metadata].created_at` (ISO 8601 datetime string) recording when the definition was first created.
4. EACH `.ffs` file SHALL contain `[metadata].modified_at` (ISO 8601 datetime string) updated to the current timestamp on every save.
5. WHEN a Structure_Definition is opened in the Structure_Editor and has been modified externally since it was loaded (detected by comparing on-disk `modified_at` with the loaded value), THE system SHALL warn the operator and offer to reload from disk or overwrite.
6. THE Catalog_Browsing_Panel SHALL display the version number and last-modified date for each Structure_Definition in the list view.
7. WHEN a Structure_Definition is duplicated, THE new copy SHALL have version 1, a new `created_at` timestamp, and a cleared `modified_at`.

---

### Requirement 10: Auto-Association (File Extension → Structure Mapping)

**User Story:** As an operator, I want the workbench to automatically suggest or apply the correct structure definition when I open a data file whose name or extension matches a known pattern, so that FileForge_Mode activates seamlessly without manual selection.

**Source:** FFE-STRUCT Reqs 4–5. [FFE-STRUCT, WB]

#### Acceptance Criteria

1. EACH Structure_Definition MAY include an `[associations].file_patterns` array of glob strings (e.g., `["*.dat", "CUST_*.dat", "INV??????.txt"]`) in the `.ffs` file.
2. THE system SHALL build a File_Association_Map by scanning all Structure_Definitions in the Active_Catalog_Location and collecting their `file_patterns` entries at startup and on catalog reload.
3. WHEN a file is opened that does not have a companion `.fc.json`/`.fc.xlsx` config, THE system SHALL check the File_Association_Map for a File_Pattern_Mask that matches the opened file's name.
4. WHEN exactly one matching Structure_Definition is found, THE system SHALL automatically apply it and activate FileForge_Mode, displaying a status message identifying the applied structure and its source Catalog_Location.
5. WHEN more than one matching Structure_Definition is found, THE system SHALL display a structure selector showing all matches, allowing the operator to choose which to apply.
6. WHEN no matching Structure_Definition is found and no companion config exists, THE system SHALL open the file in standard mode without error.
7. A single File_Pattern_Mask SHALL appear in at most one Structure_Definition within a Catalog_Location. WHEN the same pattern appears in multiple definitions, THE system SHALL emit a WARN-level log record identifying the conflict and use the first match in alphabetical order by structure name.
8. THE `file_patterns` key SHALL be optional. A Structure_Definition with no file patterns is valid and can still be applied manually.
9. THE auto-association check SHALL be performed against the Active_Catalog_Location only. The operator can change the active location to search a different catalog.
10. THE Structure_Editor SHALL include an editable `file_patterns` section where the operator can add, edit, and remove File_Pattern_Masks for a Structure_Definition.

---

### Requirement 11: Manual Association Command

**User Story:** As an operator, I want to manually select a structure definition from the catalog and apply it to the currently open file via a command, so that I can use FileForge_Mode regardless of whether the filename matches a known pattern.

**Source:** FFE-STRUCT Req 6. [FFE-STRUCT, WB]

#### Acceptance Criteria

1. THE system SHALL register an `APPLY STRUCTURE` primary command with the `command-framework`, accessible from the command line, menus (File → Apply Structure), and the Catalog_Browsing_Panel toolbar.
2. WHEN `APPLY STRUCTURE` is issued without arguments, THE system SHALL open a structure selector dialog listing all Structure_Definitions in the Active_Catalog_Location with search/filter capability.
3. WHEN `APPLY STRUCTURE <name>` is issued with a structure name argument, THE system SHALL look up the named Structure_Definition in the Active_Catalog_Location and apply it directly. IF the name is not found, THE system SHALL display an error message listing available structure names.
4. WHEN a Structure_Definition is applied to the current file, THE system SHALL activate FileForge_Mode (or switch the active structure if already in FileForge_Mode) and display the file using the applied structure's Record_Structures.
5. WHEN a Structure_Definition is applied manually to a file that already has a companion `.fc.json`, THE system SHALL display a message noting that the catalog structure overrides the file-local config for this session only; the companion config on disk is not modified.
6. WHEN the operator applies a structure manually, THE system SHALL optionally offer to save the association as a File_Pattern_Mask in the Structure_Definition's `.ffs` file, so future opens of matching files auto-apply the same structure.
7. THE `APPLY STRUCTURE` command SHALL be valid for any open file. WHEN issued with no file open, THE system SHALL display an error indicating no active file.

---

### Requirement 12: Grid Browse Mode

**User Story:** As a data engineer or business analyst, I want to view a structured data file as a read-only column-per-field grid, so that I can inspect field values without manual character counting or raw text parsing.

**Source:** FFE-STRUCT Req 7. [FFE-STRUCT]

#### Acceptance Criteria

1. WHEN FileForge_Mode is active and the editor is in Browse mode, THE system SHALL display records in Grid_Browse_Mode: a scrollable grid with one column per field in the active Record_Structure and one row per record.
2. IN Grid_Browse_Mode, ALL cells SHALL be read-only; no editing SHALL be permitted.
3. THE grid SHALL display field names as column headers and parsed field values as cell content.
4. WHEN a field has `decimals > 0`, THE grid SHALL display the decimal-converted value (packed integers shown with decimal point at the correct position).
5. Non-matching records (records that match no Record_Structure) SHALL be displayed as full-width raw text rows in a visually distinct colour, spanning all columns, with a `[NO MATCH]` indicator.
6. THE grid SHALL display the record number (1-based) in a fixed leftmost column.
7. THE grid SHALL support keyboard navigation (arrow keys, Page Up/Down, Home/End) consistent with the ISPF-style command model defined in `command-semantics`.
8. THE grid SHALL support column resizing via drag handles on column headers.
9. WHEN the operator clicks on a matching record row, THE system SHALL display the field breakdown (offset, length, raw bytes, decoded value) in a detail panel.

---

### Requirement 13: Grid Edit Mode

**User Story:** As a data engineer, I want to edit individual field values in a structured data file using a column-per-field grid, so that I can make targeted corrections without touching the surrounding record layout.

**Source:** FFE-STRUCT Req 8. [FFE-STRUCT]

#### Acceptance Criteria

1. WHEN FileForge_Mode is active and the editor is in Edit mode, THE system SHALL display records in Grid_Edit_Mode: the same grid layout as Browse mode, but with individually editable field cells for matching records.
2. WHEN the operator activates a cell for editing, THE system SHALL display the field's current value in an inline edit widget within the cell.
3. WHEN the operator changes a field value and moves to another cell, THE system SHALL validate the new value against the field's declared `field_type`. IF the value is invalid (e.g., non-numeric in a `numeric` field), THE system SHALL highlight the cell with an error indicator and display the validation error in the status area.
4. WHEN the operator edits a cell, THE system SHALL store the change in the in-memory edit buffer only — the source file on disk SHALL NOT be modified until an explicit save. Modified records SHALL be visually distinguished from unmodified records.
5. Non-matching records SHALL NOT be editable in Grid_Edit_Mode; they SHALL remain displayed as raw text rows and SHALL be visually distinct.
6. THE system SHALL integrate with the `undo-redo-transactions` framework: all edits to fields within the same record during a single editing pass SHALL be grouped as one undoable transaction.
7. WHEN the operator issues the `SAVE` command, THE system SHALL flush the edit buffer: write merged content (original file bytes with buffered field patches applied) via a temporary file followed by an atomic rename. Unmodified records and fields SHALL preserve their exact original byte content.
8. WHEN writing a modified packed-decimal field back, THE system SHALL pack the displayed decimal value back to COMP-3 format.
9. WHEN a field value is shorter than the defined `length` after editing, THE system SHALL right-pad with spaces (for alphanumeric) or left-pad with zeros (for numeric types) to the defined length.
10. WHEN a field value exceeds the defined `length` after editing, THE system SHALL truncate to the defined length and display a warning in the status area.
11. WHEN the operator issues `CANCEL` or attempts to close with unsaved grid edits, THE system SHALL prompt the operator to save or discard.

---

### Requirement 14: Catalog Location Management

**User Story:** As an operator, I want to manage multiple catalog locations (add, remove, rename, switch active) so that I can organize structure libraries by project, team, or purpose.

**Source:** FFE-STRUCT Reqs 1, 11, 12. [FFE-STRUCT, WB]

#### Acceptance Criteria

1. THE system SHALL provide a Catalog Location Manager accessible from the Catalog_Browsing_Panel toolbar and via the command `catalog.manage_locations`.
2. THE Catalog Location Manager SHALL allow the operator to add a new Catalog_Location by specifying a directory path. THE system SHALL verify the path exists and is a readable directory; IF not, THE system SHALL display an error and not add the location.
3. THE Catalog Location Manager SHALL allow the operator to remove a configured Catalog_Location from the list (without deleting the directory or its contents).
4. THE Catalog Location Manager SHALL allow the operator to rename a Catalog_Location's display label.
5. THE Catalog Location Manager SHALL allow the operator to designate any configured location as the Active_Catalog_Location.
6. WHEN the operator switches the Active_Catalog_Location, THE Catalog_Browsing_Panel SHALL reload to display structure definitions from the new location.
7. THE system SHALL persist the list of Catalog_Locations and the Active_Catalog_Location designation in the `configuration-system` user-layer file under the `[catalog]` table.
8. THE operator SHALL be able to designate a project directory (or any subdirectory within it) as a Catalog_Location, enabling project-local structure management.
9. WHEN the workbench starts and no catalog configuration exists, THE system SHALL initialise with the default user-level catalog location and an empty location list, without error.
10. WHEN the workbench starts and a configured Catalog_Location path no longer exists, THE system SHALL emit a WARN-level log record, mark that location as unavailable in the UI, and continue loading from other configured locations.

---

### Requirement 15: Configuration Keys

**User Story:** As an operator, I want all catalog settings accessible in the workbench configuration file, so that I can pre-configure the catalog for a team or deployment without using the GUI.

**Source:** FFE-STRUCT Req 12, WB §8. [FFE-STRUCT, WB]

#### Acceptance Criteria

1. THE `configuration-system` SHALL accept a `[catalog]` table in any configuration layer with the following keys:
   - `locations` (array of strings): list of Catalog_Location directory paths
   - `active_location` (string): path of the Active_Catalog_Location
   - `auto_associate` (boolean, default `true`): enable/disable automatic file-to-structure association on file open
   - `default_field_type` (string, default `"alphanumeric"`): default field type for new fields in the Structure_Editor
2. WHEN the `catalog.active_location` key specifies a path that does not exist, THE system SHALL emit a configuration warning and fall back to the default user-level catalog location.
3. WHEN the `catalog.auto_associate` key is set to `false`, THE system SHALL skip the auto-association check on file open; structures can still be applied manually.
4. THE catalog configuration keys SHALL participate in the `configuration-system` hot-reload mechanism: changes to `[catalog]` keys SHALL take effect within 2 seconds without workbench restart.
5. THE catalog configuration keys SHALL follow the `configuration-system` layer precedence: Defaults → System → User → Profile → Project → Workspace, enabling per-project catalog location overrides.

