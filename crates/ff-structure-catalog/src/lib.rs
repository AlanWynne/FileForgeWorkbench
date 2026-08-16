//! # ff-structure-catalog — Structure Catalog for FileForgeWorkbench
//!
//! This crate provides a **persistent, operator-managed library of named
//! Record Structure definitions** for the FileForgeWorkbench platform.
//!
//! ## Key Capabilities
//!
//! - **Catalog persistent store** — a configurable directory of `.ffs` (FileForge
//!   Structure) files in TOML format
//! - **Catalog CRUD operations** — create, read, update, delete, list, and duplicate
//!   structure definitions
//! - **Catalog browsing panel** — searchable, filterable list state for the
//!   dockable browsing panel
//! - **Structure editor** — field grid model with add/remove/reorder/auto-compute
//! - **Auto-association** — automatic file-to-structure mapping via glob patterns
//! - **Import/export** — format conversion between `.ffs`, `.fc.json`, `.fc.xlsx`,
//!   and COBOL copybook
//! - **Grid browse/edit** — record-to-grid parsing and edit buffering
//! - **Versioning** — monotonic version increment and conflict detection
//!
//! ## Architecture
//!
//! This is a **Wave 12 (FileForge Domain)** crate. It is GUI-independent —
//! all functionality is testable via the public API without a running editor.
//! Upstream crates are connected via trait interfaces.
//!
//! ## Example
//!
//! ```rust
//! use ff_structure_catalog::model::StructureDefinition;
//! use ff_structure_catalog::field::{FieldDefinition, FieldType};
//! use ff_structure_catalog::catalog::StructureCatalog;
//!
//! let mut catalog = StructureCatalog::new();
//! let def = StructureDefinition::new("CUSTOMER");
//! catalog.create(def).unwrap();
//! let retrieved = catalog.read("CUSTOMER").unwrap();
//! assert_eq!(retrieved.name(), "CUSTOMER");
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// File association mapping — glob pattern → structure name.
pub mod association;

/// Catalog browsing panel state — filtering, sorting, preview.
pub mod browsing;

/// Structure catalog — in-memory index and CRUD operations.
pub mod catalog;

/// Command identifiers and action enums.
pub mod commands;

/// Catalog configuration keys and defaults.
pub mod config;

/// COBOL copybook parser — import from copybook source.
pub mod copybook;

/// Structure editor — field grid model and dirty tracking.
pub mod editor;

/// Error types for the ff-structure-catalog crate.
pub mod error;

/// Field definition and field type models.
pub mod field;

/// FFS file format — TOML serialization and deserialization.
pub mod ffs_format;

/// Grid browse/edit mode — record parsing and cell buffering.
pub mod grid;

/// Structure import and export — format conversion.
pub mod import_export;

/// Catalog location management.
pub mod location;

/// Core data models — StructureDefinition, RecordStructure, metadata.
pub mod model;

/// Structure versioning — version increment and conflict detection.
pub mod versioning;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use association::{AssociationResult, FileAssociationMap, PatternConflict};
pub use browsing::{BrowsingListEntry, BrowsingPanelState, SortMode, StructurePreview};
pub use catalog::StructureCatalog;
pub use commands::{ids as command_ids, ContextMenuAction, ToolbarAction, ALL_COMMAND_IDS};
pub use config::CatalogConfig;
pub use copybook::{CopybookParseResult, CopybookParser, CopybookParserConfig, CopybookWarning};
pub use editor::EditorState;
pub use error::{CatalogError, FieldInterpretError, FieldValidationError, ValidationErrors};
pub use ffs_format::{FfsParser, FfsSerializer};
pub use field::{FieldDefinition, FieldType};
pub use grid::{
    decode_field_value, encode_field_value, FieldValue, GridBrowseState, GridEditState, GridRow,
};
pub use import_export::{
    ConflictResolution, ExportService, ImportResult, ImportService, StructureFormat,
};
pub use location::{CatalogLocation, CatalogLocationManager};
pub use model::{
    FileAssociations, RecordFormat, RecordStructure, StructureDefinition, StructureMetadata,
};
pub use versioning::VersionManager;
