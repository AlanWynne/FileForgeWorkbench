//! # ff-vsam-services — VSAM Services for FileForgeWorkbench
//!
//! This crate is the **single authority** for VSAM record-level operations:
//! KSDS, ESDS, RRDS, LDS behaviour, alternate indexes, record insertion,
//! record retrieval, and key management.
//!
//! ## Ownership (ADR-001)
//!
//! - KSDS behaviour (key-sequenced insertion, key lookup, key-ordered traversal)
//! - ESDS behaviour (entry-sequenced insertion, sequential access)
//! - RRDS behaviour (relative record addressing)
//! - LDS behaviour (byte-oriented linear access)
//! - Alternate index management (definition, maintenance, path access)
//! - Record insertion logic
//! - Record retrieval logic
//! - Key management (uniqueness enforcement, key comparison, index maintenance)
//!
//! ## Does NOT Own
//!
//! - Catalog metadata persistence (owned by ff-dataset-catalog)
//! - IDCAMS command parsing (owned by ff-idcams)
//! - JCL parsing (owned by ff-dsalloc)
//! - Storage provider registration (owned by ff-vfs)

pub mod traits;

pub use traits::*;
