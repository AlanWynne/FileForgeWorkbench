//! Command-specific execution handlers.
//!
//! Each handler takes a parsed command, the services container, and the
//! execution state. It delegates to the reconciled downstream services
//! (`ff-dscatalog`'s `CatalogService` / `VsamService` and the record-aware
//! `DatasetAccess`) and updates the state accordingly (CR-CH-059,
//! Requirement 28).
//!
//! The handlers are split by command group to keep each file within the
//! 400-non-test-line limit (`rust-standards.md`):
//! - `define` -- DEFINE CLUSTER / AIX / PATH / GDG
//! - `mutate` -- DELETE / ALTER
//! - `query`  -- LISTCAT / VERIFY / EXPORT / IMPORT / BLDINDEX / SET
//! - `io`     -- PRINT / REPRO (record-level copy via the reconciled seam)

mod define;
mod io;
mod mutate;
mod query;

pub use define::{
    execute_define_aix, execute_define_cluster, execute_define_gdg, execute_define_path,
};
pub use io::{execute_print, execute_repro};
pub use mutate::{execute_alter, execute_delete};
pub use query::{
    execute_bldindex, execute_export, execute_import, execute_listcat, execute_set, execute_verify,
};
