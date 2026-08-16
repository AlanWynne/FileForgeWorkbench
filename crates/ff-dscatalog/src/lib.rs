//! # ff-dscatalog — Mainframe Dataset Catalog Emulation
//!
//! This crate provides **mainframe dataset filesystem emulation on the local desktop**.
//! It implements a SQLite-backed catalog database that maps mainframe-style dataset
//! names (HLQ.qualifier format) to physical files stored in a structured repository
//! layout on the local filesystem.
//!
//! ## Supported Dataset Types
//!
//! - **PS (Sequential)** — Single flat files stored in `storage/`
//! - **PO (Partitioned — PDS/PDSE)** — Libraries of members stored in `pds/`
//! - **GDG (Generation Data Group)** — Versioned dataset collections in `gdg/`
//!
//! ## Architecture
//!
//! The catalog integrates with the VFS layer as a dedicated provider (scheme `catalog`),
//! making datasets addressable as `vfs://catalog/HLQ.QUALIFIER.NAME` throughout the
//! workbench.
//!
//! ## Example
//!
//! ```rust,no_run
//! use ff_dscatalog::dsn::Dsn;
//!
//! let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
//! assert_eq!(dsn.hlq(), "PAYROLL");
//! assert_eq!(dsn.as_str(), "PAYROLL.INPUT.FILE");
//! ```

pub mod catalog;
pub mod catalog_registry;
pub mod commands;
pub mod config;
pub mod context_menu;
pub mod dataset;
pub mod dsn;
pub mod encoding;
pub mod error;
pub mod gdg;
pub mod listcat;
pub mod pds;
pub mod properties;
pub mod repository;
pub mod schema;
pub mod vfs_provider;

// Re-exports for public API
pub use catalog::Catalog;
pub use catalog_registry::CatalogRegistry;
pub use dataset::{AllocParams, DatasetRecord, Dsorg, PartitionedSubtype, Recfm};
pub use dsn::{Dsn, MemberName};
pub use error::CatalogError;
pub use gdg::{GdgBase, GdgGeneration, GdgStatus};
pub use pds::PdsMemberInfo;
pub use properties::DatasetProperties;
pub use repository::Repository;
pub use vfs_provider::CatalogVfsProvider;
