//! The mainframe backend Command Environment (BRC.4, command-environments
//! Requirement 16.1 / 16.4 / 18.7 / 18.8).
//!
//! [`MainframeEnvironment`] is a record-capable [`ff_vfs::BackendEnvironment`]
//! that stores the editor's framed records over a
//! [`ff_dscatalog::DatasetAccess`]. It is reached ONLY by addressing (FFEDIT's
//! SAVE forwarding resolves the owning environment to this backend); no user
//! ever focuses it. It is self-contained in `ff-idcams` and unit-testable
//! against a real `CatalogDatasetAccess` over a `TempDir` -- no `ff-desktop`
//! dependency.
//!
//! ## Byte save is an error
//!
//! A mainframe dataset has NO meaningful host-path byte save: it is a
//! record-oriented resource whose bytes are framed by its RECFM codec. The
//! retained byte [`save`](ff_vfs::BackendEnvironment::save) entry therefore
//! returns an error directing callers to the record-aware store; the
//! record-vs-byte selection in the editor SAVE walk (BRC.3) always takes the
//! record path for this backend because [`record_capable`] is `true`.
//!
//! ## Resolve-only (no allocate-on-NotFound)
//!
//! `save_records` RESOLVES the dataset by `target.dsn` with
//! [`AccessIntent::Write`]. The open flow that produced the editor tab already
//! created/resolved the dataset, so the dataset exists at save time. A
//! `DatasetError::NotFound` is therefore an unexpected condition mapped to a
//! non-zero rc (`rc = 12`), NOT an implicit allocate: allocation needs a target
//! Volume, SPACE, and geometry that `RecordAttrs` (RECFM/LRECL/encoding) does
//! NOT carry, so inventing an allocation here would guess identity the caller
//! never supplied. Allocation stays the open/allocator flow's responsibility.
//!
//! ## Return-code mapping (Requirement 18.8)
//!
//! `save_records` never returns `NotRecordCapable` (it IS record-capable). It
//! maps the dataset-access outcome to [`RecordStoreOutcome::Stored`] with:
//!
//! | DatasetError                | rc | meaning                              |
//! |-----------------------------|----|--------------------------------------|
//! | (Ok)                        |  0 | stored successfully                  |
//! | `SpaceAbend`                | 37 | x37-class space abend                |
//! | `VolumeFull`                | 37 | volume full (space class)            |
//! | `ReadOnlyVolume`            |  8 | target volume is read-only           |
//! | any other error             | 12 | general failure (incl. NotFound)     |

use std::sync::Arc;

use ff_dscatalog::dataset::Recfm;
use ff_dscatalog::{AccessIntent, DatasetAccess, DatasetError, Record};
use ff_vfs::{
    BackendEnvironment, RecordAttrs, RecordFormatKind, RecordSource, RecordStoreOutcome,
    StoreTarget,
};

/// The MAINFRAME backend Command Environment: stores records over an owned,
/// object-safe [`ff_dscatalog::DatasetAccess`] (BRC.4).
///
/// Validates: command-environments Requirement 16.1, 16.4, 18.7, 18.8
pub struct MainframeEnvironment {
    access: Arc<dyn DatasetAccess>,
}

impl MainframeEnvironment {
    /// Construct a mainframe environment over the given dataset access.
    ///
    /// The access is held behind an `Arc<dyn DatasetAccess>` so the shell can
    /// share one `CatalogDatasetAccess` across the backend and other consumers.
    pub fn new(access: Arc<dyn DatasetAccess>) -> Self {
        Self { access }
    }

    /// Map the ff-vfs record format to the dataset catalog's RECFM.
    ///
    /// `Fixed -> FB`, `Variable -> VB`, `Undefined -> U`. The mapping is used
    /// only when a NotFound dataset would be allocated; the current resolve-only
    /// policy does not allocate, but the helper documents the intended mapping
    /// and keeps the RECFM translation in one place.
    fn recfm_for(kind: RecordFormatKind) -> Recfm {
        match kind {
            RecordFormatKind::Fixed => Recfm::FB,
            RecordFormatKind::Variable => Recfm::VB,
            RecordFormatKind::Undefined => Recfm::U,
            // `RecordFormatKind` is #[non_exhaustive]; default any future variant
            // to Undefined (byte pass-through) rather than guess a framing.
            _ => Recfm::U,
        }
    }

    /// Map a dataset-access error to the record-store return code (Req 18.8).
    fn rc_for_error(err: &DatasetError) -> i32 {
        match err {
            DatasetError::SpaceAbend { .. } => 37,
            DatasetError::VolumeFull { .. } => 37,
            DatasetError::ReadOnlyVolume { .. } => 8,
            _ => 12,
        }
    }

    /// Pump every record from `records` into the open dataset, then close it so
    /// the RECFM codec frames the bytes (Req 18.7). Any dataset error short-
    /// circuits and is returned for rc mapping.
    fn store_all(
        &self,
        mut open: ff_dscatalog::OpenDataset,
        records: &mut dyn RecordSource,
    ) -> Result<(), DatasetError> {
        while let Some(rec) = records.next_record() {
            self.access.put(
                &mut open,
                &Record {
                    key: Vec::new(),
                    data: rec.to_vec(),
                },
            )?;
        }
        // The codec frames pending bytes on close; a space failure (x37 /
        // volume-full) surfaces here.
        self.access.close(open)
    }
}

impl BackendEnvironment for MainframeEnvironment {
    fn name(&self) -> &str {
        "MAINFRAME"
    }

    fn is_case_sensitive(&self) -> bool {
        false
    }

    /// A mainframe dataset is a RECORD-ONLY resource with no host-path byte
    /// save; direct the caller to the record-aware store.
    fn save(&self, path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
        let _ = (path, bytes);
        Err(std::io::Error::other(
            "MAINFRAME is record-only; use save_records",
        ))
    }

    fn record_capable(&self) -> bool {
        true
    }

    /// Store the editor's framed records over the dataset access (Req 18.7),
    /// mapping the outcome to a return code (Req 18.8). Resolve-only: a NotFound
    /// dataset maps to a non-zero rc rather than an implicit allocate (see the
    /// module docs). Never returns `NotRecordCapable`.
    fn save_records(
        &self,
        target: &StoreTarget,
        records: &mut dyn RecordSource,
        attrs: &RecordAttrs,
    ) -> RecordStoreOutcome {
        // The RECFM mapping is only needed for allocate; resolve-only does not
        // allocate, but reference it so the documented helper is exercised and
        // the intended mapping is pinned.
        let _ = Self::recfm_for(attrs.recfm);

        let handle = match self.access.resolve(&target.dsn, AccessIntent::Write) {
            Ok(handle) => handle,
            Err(err) => {
                return RecordStoreOutcome::Stored {
                    rc: Self::rc_for_error(&err),
                }
            }
        };
        let open = match self.access.open(&handle, AccessIntent::Write) {
            Ok(open) => open,
            Err(err) => {
                return RecordStoreOutcome::Stored {
                    rc: Self::rc_for_error(&err),
                }
            }
        };
        match self.store_all(open, records) {
            Ok(()) => RecordStoreOutcome::Stored { rc: 0 },
            Err(err) => RecordStoreOutcome::Stored {
                rc: Self::rc_for_error(&err),
            },
        }
    }
}

#[cfg(test)]
#[path = "mainframe_env_tests.rs"]
mod tests;
