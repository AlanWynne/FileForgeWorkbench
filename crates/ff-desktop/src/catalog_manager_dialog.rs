//! # Catalog Manager Dialog (adapter)
//!
//! The New/Edit/Delete catalog modal dialogs were extracted into the standalone
//! `ff-catalog-dialog` crate (decomposition Wave 4, DECOMP.4), split there into
//! one module per sub-dialog. This module is now a thin re-export so every
//! existing `crate::catalog_manager_dialog::*` reference (files_panel,
//! shell/render, shell/update) resolves unchanged against the extracted crate's
//! public API (`render`, `render_edit`, `render_delete`, `execute_delete`,
//! `NewCatalogForm`, `EditCatalogForm`, `DeleteCatalogConfirm`, `DeleteChoice`,
//! `DialogOutcome`, `validate`, `validate_edit`, `build_catalog`).
//!
//! Validates: Requirement 19.1, 19.2

pub use ff_catalog_dialog::*;
