//! # Catalog Registry (adapter)
//!
//! The catalog registry model was extracted into the standalone
//! `ff-catalog-registry` crate (decomposition Wave 3, DECOMP.3). This module is
//! now a thin re-export so every existing `crate::catalog_registry::*` reference
//! (session_manager, shell/mod, shell/commands, shell/render, shell/reset_bare,
//! files_panel, catalog_manager_dialog) resolves unchanged against the extracted
//! crate's public API.
//!
//! Validates: Requirement 19.1, 19.2

pub use ff_catalog_registry::*;
