//! # Deferred-open channel types
//!
//! The payload and shared cell for the deferred file-open channel read by the
//! frame loop. Moved verbatim out of `state.rs` (rust-standards.md 400-line
//! rule); behaviour is unchanged.

use std::sync::{Arc, Mutex};

/// The deferred-open channel payload: the resource `path` and the optional
/// Owning_Environment NAME (`owning_env`; `None` -> host FS default).
///
/// core-001 strip: the mainframe-dataset open fields (`identity` /
/// `recfm_lrecl`, which carried `ff_vfs::RecordFormatKind` for the record-aware
/// SAVE seam) are removed with the catalog/VFS domains. The editor Context is
/// parked, so the pending open no longer sets a record format on a document.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingOpenReq {
    /// The resource path the open reads from.
    pub(super) path: String,
    /// The Owning_Environment NAME; `None` -> host FS default.
    pub(super) owning_env: Option<String>,
}

/// The deferred-open channel: when `Some`, open the [`PendingOpenReq`] at the
/// start of the next frame. A `type` alias to keep the nested
/// `Arc<Mutex<Option<..>>>` readable (and satisfy clippy::type_complexity).
pub(super) type PendingOpen = Arc<Mutex<Option<PendingOpenReq>>>;
