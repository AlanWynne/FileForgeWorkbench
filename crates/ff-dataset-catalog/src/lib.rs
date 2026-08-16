//! # ff-dataset-catalog — Dataset Catalog for FileForgeWorkbench
//!
//! This crate is the **single authority** for dataset metadata, catalog entries,
//! naming validation, and resolution APIs. All subsystems obtain dataset information
//! from this crate's public API (`CatalogService` trait).
//!
//! ## Ownership (ADR-001)
//!
//! - Dataset definitions (create, read, update, delete metadata)
//! - Catalog entries (the SQLite catalog database)
//! - Dataset attributes (RECFM, LRECL, BLKSIZE, DSORG)
//! - Dataset aliases
//! - GDG catalog metadata (base definitions, generation tracking, roll-off policy)
//! - Dataset resolution APIs (DSN to physical path)
//! - Dataset naming validation (DSN syntax, qualifier rules, HLQ management)
//!
//! ## Does NOT Own
//!
//! - Physical dataset allocation workflows driven by JCL (owned by ff-dsalloc)
//! - Dataset content I/O beyond path resolution (content flows through ff-vfs)
//! - VSAM record storage or retrieval logic (owned by ff-vsam-services)
//! - JCL parsing (owned by ff-dsalloc)
//! - IDCAMS command processing (owned by ff-idcams)

pub mod traits;

pub use traits::*;
