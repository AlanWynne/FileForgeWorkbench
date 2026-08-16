# Design Document: Structure Catalog (`ff-structure-catalog`)

## Overview

The `ff-structure-catalog` crate provides a **persistent, operator-managed library of named Record Structure definitions** for the FileForgeWorkbench platform. It is the single source of truth for structure definitions that describe the field layout of flat-file data records — enabling grid-based browse/edit, automatic file-to-structure association, and COBOL copybook import.

### Purpose

- Manage a configurable directory of `.ffs` (FileForge Structure) files in TOML format
- Provide CRUD operations for structure definitions (create, read, update, delete, list, duplicate)
- Parse COBOL copybook source into field definitions for import
- Support structure import from legacy formats (`.fc.json`, `.fc.xlsx`) and export to multiple formats
- Maintain an in-memory catalog index with file-watcher-driven refresh
- Build and query a File Association Map for automatic structure-to-file mapping
- Provide data models consumed by the grid browse/edit rendering layer

### Position in Architecture

```
Wave 12 — FileForge Domain

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│  Catalog Browsing Panel │ Structure Editor │ Grid View        │
├─────────────────────────────────────────────────────────────┤
│  ff-structure-catalog (THIS CRATE) ← Wave 12                 │
│  ff-fileforge-integration │ ff-record-selection-criteria      │
├─────────────────────────────────────────────────────────────┤
│  ff-command │ ff-config │ ff-vfs │ ff-document-model          │
│  ff-undo-redo │ ff-logging                                    │
├─────────────────────────────────────────────────────────────┤
│              Foundation: ff-logging (Wave 0)                  │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **Command-Driven Architecture (Req 4)**: All catalog operations are registered commands (`catalog.*`)
- **GUI Independence (Req 2)**: Zero GUI dependencies — data models and logic are GUI-agnostic; panel rendering is the shell's responsibility
- **Plugin Architecture (Req 3)**: Field type handlers are extensible via traits
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-structure-catalog`
- **Error Message Standards (Req 8)**: All errors follow `[structure-catalog] operation: description` format
- **Async I/O (Req 6)**: Catalog file access goes through VFS (async); in-memory index operations are synchronous
- **VFS Abstraction (FFW-ARCH-001)**: All `.ffs` file I/O uses `ff-vfs`, never `std::fs` directly

### Upstream Dependencies

| Crate | Usage |
|-------|-------|
| `ff-logging` | All diagnostic output (WARN on invalid files, DEBUG on CRUD success) |
| `ff-command` | Command registration for `catalog.*` commands |
| `ff-config` | Reading `[catalog]` configuration keys (locations, active_location, auto_associate, default_field_type) |
| `ff-vfs` | All file I/O for `.ffs` files, file watching for catalog refresh |
| `ff-fileforge-integration` | Legacy `.fc.json` / `.fc.xlsx` parsers for import; field type interpretation for grid rendering |

### Downstream Consumers

| Crate | Usage |
|-------|-------|
| `ff-desktop` (shell) | Renders Catalog Browsing Panel, Structure Editor, Grid Browse/Edit views |
| `ff-record-selection-criteria` | Uses Record_Structure definitions for record matching rules |
| `ff-fileforge-integration` | Receives structure definitions for record parsing and field extraction |
| `ff-document-model` | Associates structure with open document for FileForge_Mode activation |

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Shell [GUI Shell — ff-desktop]
        BROWSE_PANEL[Catalog Browsing Panel<br/>dockable, searchable]
        STRUCT_EDITOR[Structure Editor<br/>field grid, tabs]
        GRID_BROWSE[Grid Browse Mode<br/>read-only field grid]
        GRID_EDIT[Grid Edit Mode<br/>editable field cells]
    end

    subgraph ff-structure-catalog [ff-structure-catalog Crate]
        CATALOG[StructureCatalog<br/>in-memory index + CRUD]
        ENTRY[CatalogEntry<br/>parsed .ffs definition]
        FIELD_DEF[FieldDefinition<br/>name, offset, length, type]
        FFS_PARSER[FfsParser<br/>TOML ↔ StructureDefinition]
        COPYBOOK[CopybookParser<br/>COBOL copybook → fields]
        ASSOC[AssociationMap<br/>glob → structure name]
        IMPORT_EXPORT[ImportExportService<br/>format conversion]
        VERSION[VersionManager<br/>increment + conflict detect]
        COMMANDS[CatalogCommands<br/>command registrations]
    end

    subgraph Upstream [Upstream Crates]
        VFS[ff-vfs<br/>file I/O + watching]
        CONFIG[ff-config<br/>catalog settings]
        CMD[ff-command<br/>command registry]
        FFI[ff-fileforge-integration<br/>legacy parsers, field types]
        LOG[ff-logging<br/>diagnostics]
    end

    BROWSE_PANEL -->|list, search, filter| CATALOG
    STRUCT_EDITOR -->|read, update fields| ENTRY
    GRID_BROWSE -->|get record structure| CATALOG
    GRID_EDIT -->|field validation| FIELD_DEF

    CATALOG -->|read/write .ffs| VFS
    CATALOG -->|watch changes| VFS
    CATALOG -->|read settings| CONFIG
    COMMANDS -->|register catalog.*| CMD
    IMPORT_EXPORT -->|parse .fc.json/.fc.xlsx| FFI
    CATALOG -->|log operations| LOG
    ASSOC -->|match filename| CATALOG
    COPYBOOK -->|produce fields| ENTRY
end
```

### Layer Placement

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| **StructureCatalog** | Core | In-memory index of all loaded definitions; CRUD orchestration |
| **CatalogEntry** | Model | Single parsed `.ffs` file — metadata + record structures + associations |
| **FieldDefinition** | Model | Individual field within a record structure |
| **FfsParser** | I/O | TOML serialization/deserialization of `.ffs` format |
| **CopybookParser** | Import | COBOL copybook source → FieldDefinition list conversion |
| **AssociationMap** | Query | Glob pattern index for file-to-structure lookup |
| **ImportExportService** | I/O | Converts between `.ffs`, `.fc.json`, `.fc.xlsx` formats |
| **VersionManager** | Logic | Version increment, conflict detection, timestamp management |
| **CatalogCommands** | Integration | Registers all `catalog.*` commands with `ff-command` |

---

## Components and Interfaces

```
crates/ff-structure-catalog/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Crate root: public re-exports
│   ├── catalog.rs                # StructureCatalog — index + CRUD orchestration
│   ├── entry.rs                  # CatalogEntry — structure definition model
│   ├── field.rs                  # FieldDefinition, FieldType enum
│   ├── record_structure.rs       # RecordStructure — ordered field list
│   ├── metadata.rs               # StructureMetadata — version, timestamps, encoding
│   ├── association.rs            # AssociationMap — glob matching engine
│   ├── parser/
│   │   ├── mod.rs                # Parser module re-exports
│   │   ├── ffs.rs                # FfsParser — TOML read/write for .ffs format
│   │   ├── copybook.rs           # CopybookParser — COBOL copybook import
│   │   └── validation.rs         # Schema validation for parsed definitions
│   ├── import_export/
│   │   ├── mod.rs                # Import/export module re-exports
│   │   ├── import.rs             # Import from .fc.json, .fc.xlsx, .ffs
│   │   └── export.rs             # Export to .fc.json, .fc.xlsx, .ffs
│   ├── versioning.rs             # VersionManager — increment, conflict detection
│   ├── commands.rs               # CatalogCommands — command-framework registrations
│   ├── config.rs                 # CatalogConfig — reads [catalog] keys from ff-config
│   └── error.rs                  # StructureCatalogError enum
└── tests/
    ├── catalog_crud_test.rs      # Integration tests for CRUD operations
    ├── ffs_parser_test.rs        # Round-trip TOML parsing tests
    ├── copybook_parser_test.rs   # COBOL copybook parsing tests
    ├── association_test.rs       # Glob matching and conflict tests
    ├── import_export_test.rs     # Format conversion tests
    └── proptest_properties.rs    # Property-based tests
```

---

## Data Models

### 4.1 StructureCatalog

The top-level service managing the in-memory catalog index and orchestrating all operations.

```rust
/// The central catalog service managing structure definitions.
///
/// Holds an in-memory index of all valid definitions loaded from configured
/// Catalog_Locations. Provides CRUD operations and association lookups.
pub struct StructureCatalog {
    /// All loaded entries indexed by (location_path, structure_name)
    index: HashMap<CatalogKey, CatalogEntry>,
    /// The currently active catalog location
    active_location: ResourceUri,
    /// All configured catalog locations
    locations: Vec<CatalogLocation>,
    /// File-to-structure association map built from all entries
    association_map: AssociationMap,
    /// Configuration handle for [catalog] keys
    config: CatalogConfig,
    /// VFS handle for file operations
    vfs: Arc<dyn VfsAccess>,
}

/// Composite key for index lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalogKey {
    pub location: ResourceUri,
    pub name: String,
}

/// Represents a configured catalog location with metadata
#[derive(Debug, Clone)]
pub struct CatalogLocation {
    pub path: ResourceUri,
    pub label: String,
    pub is_active: bool,
    pub is_available: bool,
}
```

### 4.2 CatalogEntry (StructureDefinition)

A single `.ffs` file parsed into its in-memory representation.

```rust
/// A complete structure definition loaded from a single .ffs file.
///
/// Contains metadata, one or more record structures, and optional
/// file association patterns.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogEntry {
    /// Structure metadata (name, version, timestamps, encoding, etc.)
    pub metadata: StructureMetadata,
    /// Ordered list of record structures (e.g., Header, Detail, Trailer)
    pub record_structures: Vec<RecordStructure>,
    /// Optional file pattern associations for auto-matching
    pub associations: StructureAssociations,
    /// The VFS URI of the source .ffs file (None for unsaved new entries)
    pub source_uri: Option<ResourceUri>,
}

/// Structure metadata from the [metadata] TOML table.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureMetadata {
    /// Unique name within a catalog location
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Monotonically increasing version number (starts at 1)
    pub version: u32,
    /// ISO 8601 datetime of creation
    pub created_at: DateTime<Utc>,
    /// ISO 8601 datetime of last modification
    pub modified_at: DateTime<Utc>,
    /// Expected character encoding of associated data files (optional)
    pub encoding: Option<String>,
    /// Expected logical record length (optional)
    pub lrecl: Option<u32>,
    /// Expected record format (optional)
    pub recfm: Option<RecordFormat>,
}

/// Record format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordFormat {
    F,
    Fb,
    V,
    FbBinary,
    Vb,
    U,
}

/// File association patterns for auto-matching
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructureAssociations {
    /// Glob patterns that match filenames (e.g., "*.dat", "CUST_*.dat")
    pub file_patterns: Vec<String>,
}
```

### 4.3 RecordStructure

A named record layout containing an ordered list of field definitions.

```rust
/// A named record layout within a structure definition.
///
/// A structure definition may contain multiple record structures
/// (e.g., Header, Detail, Trailer records in a multi-format file).
#[derive(Debug, Clone, PartialEq)]
pub struct RecordStructure {
    /// Name of this record structure (e.g., "Header", "Detail")
    pub name: String,
    /// Ordered list of field definitions
    pub fields: Vec<FieldDefinition>,
}
```

### 4.4 FieldDefinition

A single field within a record structure.

```rust
/// A single field within a record structure.
///
/// Specifies the byte-level location, length, and interpretation
/// of one logical field in a flat-file record.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    /// Field name (must be non-empty)
    pub name: String,
    /// Byte offset from start of record (0-based, must be >= 0)
    pub offset: u32,
    /// Field length in bytes (must be >= 1)
    pub length: u32,
    /// Data type determining how bytes are interpreted
    pub field_type: FieldType,
    /// Number of implied decimal positions (0 = integer)
    pub decimals: u8,
    /// Identifier values for record-type matching
    pub identifiers: Vec<String>,
    /// Filter expressions for this field
    pub filters: Vec<String>,
}

/// Supported field data types.
///
/// Determines how raw bytes in a record are decoded and displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FieldType {
    /// Character data (UTF-8 or EBCDIC code page)
    Alphanumeric,
    /// Unsigned integer stored as displayable digit characters (zoned decimal)
    Numeric,
    /// IBM COMP-3 packed BCD encoding
    PackedDecimal,
    /// Raw binary bytes displayed as hex string
    Binary,
    /// Hex dump with optional ASCII sidebar
    Hex,
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Alphanumeric
    }
}
```

### 4.5 CopybookParser

Parses COBOL copybook source text into field definitions.

```rust
/// Parses COBOL copybook source into FieldDefinition lists.
///
/// Supports standard COBOL PIC clauses, COMP-3 USAGE declarations,
/// level numbers (01-49, 66, 77, 88), OCCURS, and REDEFINES.
pub struct CopybookParser {
    /// Configuration for parsing behaviour
    config: CopybookParserConfig,
}

/// Configuration options for COBOL copybook parsing.
#[derive(Debug, Clone)]
pub struct CopybookParserConfig {
    /// Starting column for COBOL source (typically 7 for fixed-format)
    pub start_column: u8,
    /// Ending column for COBOL source (typically 72 for fixed-format)
    pub end_column: u8,
    /// Whether to expand OCCURS clauses into individual fields
    pub expand_occurs: bool,
    /// Default encoding to assign to parsed fields
    pub default_encoding: String,
}

/// Result of parsing a COBOL copybook
#[derive(Debug, Clone)]
pub struct CopybookParseResult {
    /// Successfully parsed field definitions
    pub fields: Vec<FieldDefinition>,
    /// Warnings encountered during parsing (unsupported clauses, etc.)
    pub warnings: Vec<CopybookWarning>,
    /// Computed total record length
    pub record_length: u32,
}

/// A warning from copybook parsing (non-fatal)
#[derive(Debug, Clone)]
pub struct CopybookWarning {
    pub line: u32,
    pub message: String,
}
```

### 4.6 StructureFormat

Represents the supported serialization formats for import/export.

```rust
/// Supported structure file formats for import and export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StructureFormat {
    /// Native FileForge Structure format (TOML-based .ffs)
    Ffs,
    /// Legacy JSON companion config format (.fc.json)
    FcJson,
    /// Legacy Excel companion config format (.fc.xlsx)
    FcXlsx,
    /// COBOL copybook source (import only)
    Copybook,
}

impl StructureFormat {
    /// Returns the file extension associated with this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Ffs => "ffs",
            Self::FcJson => "fc.json",
            Self::FcXlsx => "fc.xlsx",
            Self::Copybook => "cpy",
        }
    }

    /// Returns whether this format supports export.
    pub fn supports_export(&self) -> bool {
        matches!(self, Self::Ffs | Self::FcJson | Self::FcXlsx)
    }
}
```

### 4.7 AssociationMap

The in-memory index mapping glob patterns to structure names.

```rust
/// Maps file glob patterns to structure definition names.
///
/// Built from all loaded CatalogEntry associations in the active location.
/// Used for auto-association when a file is opened.
pub struct AssociationMap {
    /// Entries sorted by pattern specificity (most specific first)
    entries: Vec<AssociationEntry>,
}

/// A single pattern-to-structure mapping
#[derive(Debug, Clone)]
pub struct AssociationEntry {
    pub pattern: GlobPattern,
    pub structure_name: String,
    pub catalog_location: ResourceUri,
}

impl AssociationMap {
    /// Look up matching structures for a given filename.
    /// Returns all matches (may be 0, 1, or multiple).
    pub fn find_matches(&self, filename: &str) -> Vec<&AssociationEntry> { ... }

    /// Rebuild the map from a set of catalog entries.
    pub fn rebuild(&mut self, entries: &[CatalogEntry]) { ... }

    /// Detect duplicate patterns across different structures.
    pub fn find_conflicts(&self) -> Vec<PatternConflict> { ... }
}
```

---

## Public API Surface

### 5.1 Catalog Lifecycle

```rust
impl StructureCatalog {
    /// Initialize the catalog from configuration.
    /// Loads all .ffs files from configured locations via VFS.
    pub async fn init(config: &CatalogConfig, vfs: Arc<dyn VfsAccess>) -> Result<Self, StructureCatalogError>;

    /// Reload all entries from the active catalog location.
    pub async fn reload(&mut self) -> Result<(), StructureCatalogError>;

    /// Reload a single entry by name (triggered by file watcher).
    pub async fn reload_entry(&mut self, name: &str) -> Result<(), StructureCatalogError>;

    /// Switch the active catalog location.
    pub async fn set_active_location(&mut self, path: &ResourceUri) -> Result<(), StructureCatalogError>;

    /// Get the current active location.
    pub fn active_location(&self) -> &ResourceUri;

    /// Get all configured catalog locations.
    pub fn locations(&self) -> &[CatalogLocation];
}
```

### 5.2 CRUD Operations

```rust
impl StructureCatalog {
    /// Create a new structure definition in the active catalog location.
    /// Validates the entry and writes the .ffs file.
    pub async fn create(&mut self, entry: CatalogEntry) -> Result<(), StructureCatalogError>;

    /// Read a structure definition by name from the active location.
    pub fn read(&self, name: &str) -> Result<&CatalogEntry, StructureCatalogError>;

    /// Update an existing structure definition.
    /// Increments version, updates modified_at, validates, and writes.
    pub async fn update(&mut self, entry: CatalogEntry) -> Result<(), StructureCatalogError>;

    /// Delete a structure definition by name.
    /// Requires confirmed=true; rejects unconfirmed deletions.
    pub async fn delete(&mut self, name: &str, confirmed: bool) -> Result<(), StructureCatalogError>;

    /// List all valid structure definitions in the active location.
    /// Returns entries sorted alphabetically by name.
    pub fn list(&self) -> Vec<&CatalogEntry>;

    /// Duplicate an existing structure with a new name (version reset to 1).
    pub async fn duplicate(&mut self, source_name: &str, new_name: &str) -> Result<(), StructureCatalogError>;
}
```

### 5.3 Association and Auto-Matching

```rust
impl StructureCatalog {
    /// Find structures matching a given filename via glob patterns.
    /// Returns 0, 1, or multiple matches.
    pub fn find_associations(&self, filename: &str) -> Vec<&CatalogEntry>;

    /// Check if auto-association is enabled in configuration.
    pub fn is_auto_associate_enabled(&self) -> bool;

    /// Get all pattern conflicts (same pattern in multiple structures).
    pub fn pattern_conflicts(&self) -> Vec<PatternConflict>;
}
```

### 5.4 Import and Export

```rust
/// Service for importing and exporting structure definitions.
pub struct ImportExportService {
    vfs: Arc<dyn VfsAccess>,
    ffi_parsers: Arc<dyn LegacyFormatParser>,
}

impl ImportExportService {
    /// Import a structure from a source file into the active catalog location.
    pub async fn import(
        &self,
        source_uri: &ResourceUri,
        format: StructureFormat,
        catalog: &mut StructureCatalog,
        conflict_resolution: ConflictResolution,
    ) -> Result<String, StructureCatalogError>;

    /// Export a structure definition to a specified format and destination.
    pub async fn export(
        &self,
        entry: &CatalogEntry,
        format: StructureFormat,
        destination: &ResourceUri,
    ) -> Result<(), StructureCatalogError>;
}

/// How to handle name conflicts during import
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Rename the imported structure (appends a suffix)
    Rename(String),
    /// Overwrite the existing definition
    Overwrite,
    /// Cancel the import operation
    Cancel,
}
```

### 5.5 Copybook Parsing

```rust
impl CopybookParser {
    /// Create a new parser with the given configuration.
    pub fn new(config: CopybookParserConfig) -> Self;

    /// Parse COBOL copybook source text into field definitions.
    pub fn parse(&self, source: &str) -> Result<CopybookParseResult, StructureCatalogError>;
}
```

### 5.6 FFS Parsing (TOML Serialization)

```rust
/// Parser/serializer for the .ffs TOML format.
pub struct FfsParser;

impl FfsParser {
    /// Parse a .ffs file content string into a CatalogEntry.
    pub fn parse(content: &str) -> Result<CatalogEntry, StructureCatalogError>;

    /// Serialize a CatalogEntry to .ffs TOML format string.
    pub fn serialize(entry: &CatalogEntry) -> Result<String, StructureCatalogError>;

    /// Validate a parsed entry against the .ffs schema.
    pub fn validate(entry: &CatalogEntry) -> Result<(), Vec<ValidationError>>;
}
```

### 5.7 Field Value Interpretation (Grid Support)

```rust
/// Interprets raw record bytes using a field definition.
///
/// Used by the grid browse/edit layer to display decoded field values.
pub struct FieldInterpreter;

impl FieldInterpreter {
    /// Decode raw bytes into a display string using the field definition.
    pub fn decode(
        field: &FieldDefinition,
        record_bytes: &[u8],
        encoding: Option<&str>,
    ) -> Result<FieldValue, FieldInterpretError>;

    /// Encode a display value back to raw bytes for writing.
    pub fn encode(
        field: &FieldDefinition,
        value: &str,
        encoding: Option<&str>,
    ) -> Result<Vec<u8>, FieldInterpretError>;

    /// Validate a display value against a field's type constraints.
    pub fn validate_value(
        field: &FieldDefinition,
        value: &str,
    ) -> Result<(), FieldValidationError>;
}

/// A decoded field value with display metadata.
#[derive(Debug, Clone)]
pub struct FieldValue {
    /// The display string for the field value
    pub display: String,
    /// Whether the field has a validation warning (e.g., invalid packed-decimal nibbles)
    pub has_warning: bool,
    /// Optional warning message
    pub warning: Option<String>,
}
```

### 5.8 Catalog Location Management

```rust
impl StructureCatalog {
    /// Add a new catalog location.
    pub async fn add_location(&mut self, path: ResourceUri, label: String) -> Result<(), StructureCatalogError>;

    /// Remove a catalog location from configuration (does not delete directory).
    pub fn remove_location(&mut self, path: &ResourceUri) -> Result<(), StructureCatalogError>;

    /// Rename a catalog location's display label.
    pub fn rename_location(&mut self, path: &ResourceUri, new_label: String) -> Result<(), StructureCatalogError>;
}
```

### 5.9 Version and Conflict Detection

```rust
/// Manages structure versioning and edit conflict detection.
pub struct VersionManager;

impl VersionManager {
    /// Increment the version and update modified_at timestamp.
    pub fn increment(metadata: &mut StructureMetadata);

    /// Check if the on-disk version has changed since a given loaded timestamp.
    pub async fn has_external_modification(
        entry: &CatalogEntry,
        vfs: &dyn VfsAccess,
    ) -> Result<bool, StructureCatalogError>;

    /// Reset version to 1 for a duplicated entry.
    pub fn reset_for_duplicate(metadata: &mut StructureMetadata);
}
```

---

## Error Handling

```rust
/// All errors produced by the ff-structure-catalog crate.
///
/// Follows the error message standard: `[structure-catalog] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StructureCatalogError {
    #[error("[structure-catalog] parse: invalid TOML in {path}: {detail}")]
    TomlParseError { path: String, detail: String },

    #[error("[structure-catalog] validate: schema violation in {path}: {detail}")]
    SchemaValidationError { path: String, detail: String },

    #[error("[structure-catalog] create: structure '{name}' already exists in active location")]
    DuplicateName { name: String },

    #[error("[structure-catalog] read: structure '{name}' not found in active location")]
    NotFound { name: String },

    #[error("[structure-catalog] delete: deletion of '{name}' not confirmed")]
    DeleteNotConfirmed { name: String },

    #[error("[structure-catalog] location: path '{path}' is inaccessible: {reason}")]
    LocationInaccessible { path: String, reason: String },

    #[error("[structure-catalog] location: path '{path}' does not exist")]
    LocationNotFound { path: String },

    #[error("[structure-catalog] import: failed to parse {format} file '{path}': {detail}")]
    ImportParseError { format: String, path: String, detail: String },

    #[error("[structure-catalog] export: failed to write {format} to '{path}': {detail}")]
    ExportWriteError { format: String, path: String, detail: String },

    #[error("[structure-catalog] copybook: parse error at line {line}: {detail}")]
    CopybookParseError { line: u32, detail: String },

    #[error("[structure-catalog] conflict: external modification detected for '{name}'")]
    ExternalModification { name: String },

    #[error("[structure-catalog] field: invalid value for {field_name} ({field_type}): {detail}")]
    FieldValidationError { field_name: String, field_type: String, detail: String },

    #[error("[structure-catalog] io: {operation} failed for '{path}': {source}")]
    Io { operation: String, path: String, source: std::io::Error },

    #[error("[structure-catalog] config: {detail}")]
    ConfigError { detail: String },
}

/// Errors specific to field value interpretation.
#[derive(Debug, thiserror::Error)]
pub enum FieldInterpretError {
    #[error("field '{name}' at offset {offset}: insufficient bytes (need {need}, have {have})")]
    InsufficientBytes { name: String, offset: u32, need: u32, have: u32 },

    #[error("field '{name}': invalid packed-decimal nibbles at byte {byte_index}")]
    InvalidPackedDecimal { name: String, byte_index: u32 },

    #[error("field '{name}': encoding error: {detail}")]
    EncodingError { name: String, detail: String },
}

/// Errors specific to field value validation during editing.
#[derive(Debug, thiserror::Error)]
pub enum FieldValidationError {
    #[error("value exceeds field length {max_length}")]
    TooLong { max_length: u32 },

    #[error("non-numeric characters in numeric field")]
    NonNumeric,

    #[error("invalid decimal format")]
    InvalidDecimal,

    #[error("field name must be non-empty")]
    EmptyName,

    #[error("offset must be >= 0")]
    NegativeOffset,

    #[error("length must be >= 1")]
    ZeroLength,

    #[error("invalid field type value: {value}")]
    InvalidFieldType { value: String },
}
```

---

## Integration Points

### 7.1 Integration with `ff-fileforge-integration`

| Direction | Interface | Purpose |
|-----------|-----------|---------|
| Outbound | `LegacyFormatParser` trait | Parse `.fc.json` and `.fc.xlsx` files for import |
| Outbound | `LegacyFormatWriter` trait | Serialize to `.fc.json` and `.fc.xlsx` for export |
| Outbound | `FieldTypeHandler` trait | Extensible field type decoding/encoding (packed-decimal, EBCDIC) |
| Inbound | `StructureProvider` trait | Provides structure definitions to the FileForge_Mode record parser |

```rust
/// Trait that ff-fileforge-integration exposes for legacy format parsing.
pub trait LegacyFormatParser: Send + Sync {
    fn parse_fc_json(&self, content: &str) -> Result<CatalogEntry, Box<dyn std::error::Error>>;
    fn parse_fc_xlsx(&self, bytes: &[u8]) -> Result<CatalogEntry, Box<dyn std::error::Error>>;
}

/// Trait that ff-fileforge-integration exposes for legacy format writing.
pub trait LegacyFormatWriter: Send + Sync {
    fn write_fc_json(&self, entry: &CatalogEntry) -> Result<String, Box<dyn std::error::Error>>;
    fn write_fc_xlsx(&self, entry: &CatalogEntry) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

/// Trait that ff-structure-catalog implements for upstream consumption.
/// Provides structure definitions to FileForge_Mode's record parser.
pub trait StructureProvider: Send + Sync {
    fn get_structure(&self, name: &str) -> Option<&CatalogEntry>;
    fn find_by_filename(&self, filename: &str) -> Vec<&CatalogEntry>;
}
```

### 7.2 Integration with `ff-config` (Configuration System)

| Direction | Interface | Purpose |
|-----------|-----------|---------|
| Inbound | `ConfigProvider::get()` | Read `[catalog]` configuration keys |
| Inbound | Config hot-reload events | React to configuration changes within 2 seconds |

**Configuration keys consumed:**

```toml
[catalog]
locations = ["/path/to/catalog1", "/path/to/catalog2"]
active_location = "/path/to/catalog1"
auto_associate = true
default_field_type = "alphanumeric"
```

```rust
/// Reads and caches [catalog] configuration keys.
pub struct CatalogConfig {
    locations: Vec<String>,
    active_location: String,
    auto_associate: bool,
    default_field_type: FieldType,
}

impl CatalogConfig {
    /// Load from the configuration system.
    pub fn from_config(config: &dyn ConfigAccess) -> Result<Self, StructureCatalogError>;

    /// Handle a configuration change event (hot-reload).
    pub fn on_config_changed(&mut self, config: &dyn ConfigAccess) -> Result<bool, StructureCatalogError>;
}
```

### 7.3 Integration with `ff-command` (Command Framework)

| Direction | Interface | Purpose |
|-----------|-----------|---------|
| Outbound | `CommandRegistry::register()` | Register all `catalog.*` commands at startup |
| Outbound | `CommandHandler` trait impl | Handle command execution dispatched from command framework |

**Registered commands:**

| Command ID | Description | Arguments |
|------------|-------------|-----------|
| `catalog.create` | Create a new structure definition | `entry: CatalogEntry` |
| `catalog.read` | Read a structure by name | `name: String` |
| `catalog.update` | Update an existing structure | `entry: CatalogEntry` |
| `catalog.delete` | Delete a structure | `name: String, confirmed: bool` |
| `catalog.list` | List all structures in active location | (none) |
| `catalog.duplicate` | Duplicate a structure with new name | `source: String, new_name: String` |
| `catalog.import` | Import from external file | `path: String, format: StructureFormat` |
| `catalog.export` | Export to external format | `name: String, format: StructureFormat, dest: String` |
| `catalog.browse` | Open the Catalog Browsing Panel | (none) |
| `catalog.edit_structure` | Open a structure in the Structure Editor | `name: String` |
| `catalog.apply_structure` | Apply a structure to current file (APPLY STRUCTURE) | `name: Option<String>` |
| `catalog.manage_locations` | Open Catalog Location Manager | (none) |
| `catalog.promote` | Promote file-local .fc.json to catalog | (none) |

```rust
/// Registers all catalog commands with the command framework.
pub fn register_commands(registry: &mut dyn CommandRegistry, catalog: Arc<Mutex<StructureCatalog>>) {
    // Registers handlers for all catalog.* commands listed above
}
```

### 7.4 Integration with `ff-vfs` (Virtual File System)

| Direction | Interface | Purpose |
|-----------|-----------|---------|
| Outbound | `VfsAccess` trait | Read/write `.ffs` files, list catalog directories |
| Outbound | `VfsWatcher` trait | Watch catalog directories for external changes |

```rust
/// VFS operations used by the catalog:
/// - vfs.read_to_string(uri)     — load .ffs file content
/// - vfs.write_string(uri, content) — save .ffs file
/// - vfs.delete(uri)             — remove .ffs file on delete
/// - vfs.list(uri)               — enumerate .ffs files in a catalog location
/// - vfs.stat(uri)               — check file existence and timestamps
/// - vfs.watch(uri, callback)    — watch for external modifications
```

### 7.5 Integration with `ff-undo-redo` (Undo/Redo Transactions)

| Direction | Interface | Purpose |
|-----------|-----------|---------|
| Outbound | `TransactionManager` | Group grid edits into undoable transactions |

Grid Edit Mode integrates with `ff-undo-redo` to ensure all field edits within a single record editing pass are grouped as one undoable transaction. The structure catalog itself does not directly own undo state — that responsibility belongs to the document model and edit operations layer.

### 7.6 Integration with `ff-layout-and-docking`

| Direction | Interface | Purpose |
|-----------|-----------|---------|
| Outbound | Panel registration API | Register Catalog Browsing Panel as a dockable panel |

The Catalog Browsing Panel and Structure Editor are registered as dockable panels via the `layout-and-docking` system. The `ff-structure-catalog` crate provides the data and logic; the shell layer (ff-desktop) provides the rendering.

---

## Configuration and Persistence

### 8.1 .ffs File Format (TOML Schema)

```toml
# Example .ffs file: CUSTOMER_MASTER.ffs

[metadata]
name = "CUSTOMER_MASTER"
description = "Customer master file layout — header and detail records"
version = 3
created_at = "2024-01-15T10:30:00Z"
modified_at = "2024-03-22T14:15:00Z"
encoding = "ebcdic-037"
lrecl = 200
recfm = "FB"

[associations]
file_patterns = ["CUST_*.dat", "customer_master.*"]

[[record_structures]]
name = "Header"

[[record_structures.fields]]
name = "RECORD_TYPE"
offset = 0
length = 2
field_type = "alphanumeric"

[[record_structures.fields]]
name = "CUSTOMER_ID"
offset = 2
length = 10
field_type = "numeric"
identifiers = ["HD"]

[[record_structures]]
name = "Detail"

[[record_structures.fields]]
name = "RECORD_TYPE"
offset = 0
length = 2
field_type = "alphanumeric"
identifiers = ["DT"]

[[record_structures.fields]]
name = "BALANCE"
offset = 50
length = 5
field_type = "packed-decimal"
decimals = 2

[[record_structures.fields]]
name = "ACCOUNT_FLAGS"
offset = 55
length = 4
field_type = "binary"
```

### 8.2 Default Catalog Location

| Platform | Default Path |
|----------|-------------|
| Linux | `~/.config/ffworkbench/catalogs/` |
| Windows | `%APPDATA%\FFWorkbench\catalogs\` |
| macOS | `~/Library/Application Support/FFWorkbench/catalogs/` |

The default location is created automatically on first use if it does not exist.

### 8.3 Catalog Index Lifecycle

```
Startup:
  1. Read [catalog] config → get locations + active_location
  2. For each location: VFS list → find all .ffs files
  3. For each .ffs file: parse TOML → validate schema → add to index
     - Invalid files: log WARN, skip, do not add to index
  4. Build AssociationMap from all valid entries in active location
  5. Register VFS watcher on all catalog location directories

Hot-Reload (file watcher event):
  1. Detect .ffs file added/modified/removed
  2. Re-parse affected file(s)
  3. Update in-memory index
  4. Rebuild AssociationMap
  5. Notify UI layer (if Catalog Browsing Panel is visible)

Config Hot-Reload:
  1. Detect [catalog] keys changed
  2. Update CatalogConfig
  3. If active_location changed: reload entries from new location
  4. If locations changed: add/remove watched directories
```

---

## Correctness Properties

These properties are suitable for property-based testing with `proptest`.

### Property 1: FFS Round-Trip Fidelity

**Statement:** For any valid `CatalogEntry`, serializing to `.ffs` TOML format and parsing back produces a structurally identical entry.

```
∀ entry ∈ valid CatalogEntry:
    FfsParser::parse(FfsParser::serialize(entry)) == entry
```

**Validates: Requirements 2.1, 2.2, 2.3**

### Property 2: CRUD Consistency

**Statement:** After a successful `create` operation, a subsequent `read` with the same name returns the created entry. After a successful `delete`, a `read` returns `NotFound`.

```
∀ entry ∈ valid CatalogEntry:
    catalog.create(entry) == Ok(())
    → catalog.read(entry.metadata.name) == Ok(&entry)

∀ name ∈ existing entries:
    catalog.delete(name, confirmed=true) == Ok(())
    → catalog.read(name) == Err(NotFound)
```

**Validates: Requirements 3.1, 3.2, 3.4**

### Property 3: Version Monotonicity

**Statement:** Every successful `update` operation increments the version number by exactly 1 and sets `modified_at` ≥ the previous `modified_at`.

```
∀ entry ∈ catalog:
    let v_before = entry.metadata.version
    let t_before = entry.metadata.modified_at
    catalog.update(entry) == Ok(())
    → entry.metadata.version == v_before + 1
    → entry.metadata.modified_at >= t_before
```

**Validates: Requirements 9.1, 9.2, 9.4**

### Property 4: Association Map Completeness

**Statement:** Every file_pattern in every loaded CatalogEntry in the active location appears in the AssociationMap. A filename matching a pattern returns the corresponding structure.

```
∀ entry ∈ active_location_entries:
    ∀ pattern ∈ entry.associations.file_patterns:
        ∀ filename matching pattern:
            catalog.find_associations(filename).contains(entry)
```

**Validates: Requirements 10.1, 10.2, 10.3, 10.4**

### Property 5: Field Offset Contiguity After Auto-Compute

**Statement:** After invoking "auto-compute offsets" on a RecordStructure, each field's offset equals the sum of all preceding field lengths. The first field starts at offset 0.

```
∀ record_structure after auto_compute_offsets():
    fields[0].offset == 0
    ∀ i > 0:
        fields[i].offset == fields[i-1].offset + fields[i-1].length
```

**Validates: Requirements 5.5**

### Property 6: Packed-Decimal Codec Round-Trip

**Statement:** For any valid packed-decimal byte sequence, decoding to a display string and re-encoding produces the original bytes.

```
∀ bytes ∈ valid_packed_decimal(length, decimals):
    let display = FieldInterpreter::decode(packed_field, bytes)
    FieldInterpreter::encode(packed_field, display) == bytes
```

**Validates: Requirements 6.3, 6.6**

### Property 7: Name Uniqueness Enforcement

**Statement:** The catalog never contains two entries with the same name in the same catalog location. A `create` with a duplicate name returns `DuplicateName` error.

```
∀ name ∈ existing_names(active_location):
    catalog.create(entry_with_name(name)) == Err(DuplicateName { name })
```

**Validates: Requirements 2.4**

### Property 8: Field Validation Completeness

**Statement:** A FieldDefinition passes validation if and only if: name is non-empty, offset ≥ 0, length ≥ 1, field_type is a valid enum variant, and decimals ≥ 0. All other combinations are rejected.

```
∀ field ∈ FieldDefinition:
    validate(field) == Ok(())
    ⟺ field.name.len() > 0
       ∧ field.length >= 1
       ∧ field.field_type ∈ {Alphanumeric, Numeric, PackedDecimal, Binary, Hex}
       ∧ field.decimals >= 0
```

**Validates: Requirements 5.9**

### Property 9: Import Idempotency

**Statement:** Importing the same source file twice with `Overwrite` conflict resolution produces the same catalog state as importing it once.

```
∀ source_file, format:
    catalog_after_import_once(source_file, Overwrite)
    == catalog_after_import_twice(source_file, Overwrite)
```

**Validates: Requirements 7.6, 7.7**

### Property 10: Duplicate Resets Version

**Statement:** Duplicating a structure always produces a new entry with version=1 and a fresh created_at timestamp, regardless of the source entry's version.

```
∀ source ∈ catalog:
    let dup = catalog.duplicate(source.name, new_name)
    → dup.metadata.version == 1
    → dup.metadata.created_at >= now - epsilon
    → dup.metadata.name == new_name
```

**Validates: Requirements 3.7, 9.7**

---

## Design Decisions and Rationale

| Decision | Rationale |
|----------|-----------|
| TOML for .ffs format | Human-readable, diff-friendly, version-control compatible. Aligns with WB §8 directive for TOML as data format. |
| In-memory index with file-watcher refresh | Provides fast lookups without repeated disk I/O. File watcher ensures external edits (e.g., git pull) are reflected within 2 seconds. |
| Separate `AssociationMap` structure | Glob matching is a hot-path operation (runs on every file open). A pre-built index avoids scanning all entries on each open. |
| `StructureProvider` trait for downstream | Decouples `ff-fileforge-integration` from `ff-structure-catalog` internals. Integration crate depends on the trait, not the concrete catalog. |
| CopybookParser as a dedicated component | COBOL copybook parsing is complex and stateful (level numbers, OCCURS, REDEFINES). Isolating it simplifies testing and allows independent evolution. |
| GUI-agnostic data models | The crate provides data and logic only. Panel rendering (Catalog Browsing Panel, Structure Editor, Grid views) is the shell layer's responsibility. This preserves GUI independence. |
| `confirmed: bool` on delete | Prevents accidental deletion from automation scripts. The UI layer handles confirmation dialogs; the API requires explicit confirmation. |
| All operations via commands | Ensures auditability (command history), scriptability (Lua macros), and undo integration. No direct state mutation from UI. |
| VFS for all file access | Enables future remote catalog locations (shared network catalogs, cloud-hosted catalogs) without changing catalog code. |
| Non-exhaustive enums for FieldType and StructureFormat | Allows plugins to extend the type system without breaking existing code. Future field types can be added via the plugin trait system. |

---

## Testing Strategy

| Test Category | Framework | Coverage |
|---------------|-----------|----------|
| Unit tests | `#[cfg(test)]` in each module | Field validation, TOML parsing, version logic, glob matching |
| Integration tests | `tests/` directory | Full CRUD lifecycle, import/export round-trips, association resolution |
| Property-based tests | `proptest` | Properties 1–10 above (round-trip fidelity, CRUD consistency, version monotonicity, etc.) |
| Fixture-based tests | `tests/fixtures/*.ffs` | Real-world .ffs files for parser edge cases |

### Test Dependencies

- `tempfile` — temporary directories for catalog location tests
- `proptest` — property-based test framework
- `pretty_assertions` — diff-friendly assertion output
- `chrono` — timestamp generation and comparison in version tests
