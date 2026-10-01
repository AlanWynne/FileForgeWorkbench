//! # Dataset Allocation Dialog (adapter)
//!
//! The dataset allocation dialog was extracted into the standalone
//! `ff-dataset-alloc-dialog` crate (decomposition Wave 7, Task 23). This
//! module is now a thin re-export so every existing
//! `crate::dataset_alloc_dialog::*` reference (files_panel, shell/render,
//! shell/update) resolves unchanged against the extracted crate's public
//! API (Dsorg, Recfm, AllocDatasetForm, AllocOutcome, AllocParams,
//! validate, validate_for_catalog, render).

pub use ff_dataset_alloc_dialog::*;
