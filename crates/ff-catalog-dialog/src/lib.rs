//! # Catalog Manager Dialogs -- Create / Edit / Delete
//!
//! Modal egui dialogs for creating, editing, and deleting virtual catalogs.
//! Rendered as egui modal windows within the Files Panel frame.
//!
//! Extracted from the `ff-desktop` binary crate into a standalone library crate
//! (decomposition Wave 4, DECOMP.4). The catalog registry model it operates on
//! lives in the `ff-catalog-registry` crate (Wave 3); this crate depends only on
//! `ff-catalog-registry`, `ff-dscatalog` (repository initialisation on create),
//! and `egui`. The `ff-desktop` module `crate::catalog_manager_dialog` is a thin
//! adapter that re-exports this crate's API, so every existing
//! `crate::catalog_manager_dialog::*` reference (files_panel, shell/render,
//! shell/update) resolves unchanged.
//!
//! The three sub-dialogs are split into one module each to keep every file under
//! the 400-line source limit:
//! - [`new_dialog`]: the New Catalog form, validation, build, and render.
//! - [`edit_dialog`]: the Edit Catalog form, validation, and render.
//! - [`delete_dialog`]: the Delete Catalog confirmation, choice, and execution.
//!
//! Validates: Requirement 3.1-3.8, 4.1-4.5, 19.1, 19.2

pub mod delete_dialog;
pub mod edit_dialog;
pub mod new_dialog;

use ff_catalog_registry::CatalogType;

pub use delete_dialog::{execute_delete, render_delete, DeleteCatalogConfirm, DeleteChoice};
pub use edit_dialog::{render_edit, validate_edit, EditCatalogForm};
pub use new_dialog::{build_catalog, render, validate, DialogOutcome, NewCatalogForm};

/// Human-readable label for a catalog type, shared by the New and Edit dialogs.
pub(crate) fn catalog_type_label(ct: CatalogType) -> &'static str {
    match ct {
        CatalogType::Mainframe => "Mainframe",
        CatalogType::Posix => "POSIX",
        CatalogType::Native => "Native",
    }
}
