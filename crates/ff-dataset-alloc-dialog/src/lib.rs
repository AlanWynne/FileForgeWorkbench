//! # Dataset Allocation Dialog (placeholder)
//!
//! Reserved crate for the dataset allocation dialog, to be extracted from the
//! `ff-desktop` binary crate in a future decomposition wave (CR-NR-098 goal 1).
//!
//! This crate is currently an intentional placeholder: it exists so the
//! workspace `members` entry `crates/ff-dataset-alloc-dialog` has a valid
//! library target (`cargo metadata` fails for a member with a `Cargo.toml` but
//! no target). The dialog implementation still lives in
//! `ff-desktop/src/dataset_alloc_dialog.rs` and moves here when that wave runs.
