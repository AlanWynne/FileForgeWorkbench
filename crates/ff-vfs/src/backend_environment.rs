//! # Backend Command Environment contract (CR-CH-053 Task 21)
//!
//! A [`BackendEnvironment`] is the contract a BACKEND / resource Command
//! Environment implements -- the file-system (and later dataset / database)
//! environments that OWN a resource's store semantics. It is deliberately
//! distinct from the shell-side interactive `CommandEnvironment` (FFEDIT / FFNAV
//! / FFCMD), which carries a Workspace Context the user focuses and mutates
//! shell-entangled state.
//!
//! ## Interactive vs backend environments (the two-category model)
//!
//! The CR-CH-053 model has TWO kinds of Command Environment:
//!
//! - **Interactive / context environments** (FFEDIT, FFNAV, FFCMD): the user
//!   FOCUSES a Workspace Context that makes the environment active; its verbs are
//!   UI-facing and operate on shell state. These implement the shell-side
//!   `CommandEnvironment` trait (which takes `&mut WorkbenchShell`) and can be
//!   reached by the active-env gate AND by addressing.
//!
//! - **Backend / resource environments** (`ff-ce-ntfs`, `ff-ce-posix`, the
//!   mainframe/VSAM CE, a future `ff-ce-sqlite`): NO user ever focuses them --
//!   they are reached ONLY by ADDRESSING (`dispatch_to_environment(name, cmd)`)
//!   from another environment (e.g. FFEDIT addresses SAVE to the owning backend;
//!   a macro `ADDRESS <name>`; the mainframe CE addressing `ff-ce-sqlite`). They
//!   are LIGHT from FFWB's perspective (no Context, no shell dependency) but may
//!   be RICH in backend command support. They implement THIS trait.
//!
//! ## No central hub
//!
//! This trait is a pure CONTRACT. It knows about NO concrete backend. Backend CEs
//! REGISTER THEMSELVES by name into the shell's open `Environment_Registry`
//! (CR-CH-053 Task 13); anyone addresses them BY NAME through that registry. No
//! crate holds a list of backends, so adding a new backend (e.g. `ff-ce-sqlite`)
//! is a new crate that registers itself -- zero edits elsewhere. The trait lives
//! in `ff-vfs` (the storage-abstraction crate every backend already depends on),
//! NOT a dedicated hub crate that would need maintenance per backend.
//!
//! A future configurable "addressable environments" Context (owner idea, logged
//! as a pending CR) would READ/manage that same open registry; this contract does
//! not foreclose it.

use std::path::Path;

/// The outcome of a backend store operation, carrying a return code so the
/// addressing caller (FFEDIT's SAVE forwarding, a macro `ADDRESS`) can map it to
/// a status / macro `RC`. Mirrors the shell-side dispatch outcome's intent at the
/// backend boundary.
#[derive(Debug)]
pub enum BackendOutcome {
    /// The backend handled the operation. `rc` is 0 on success (the macro-RC
    /// convention); a non-zero code is a backend-reported soft failure.
    Handled { rc: i32 },
    /// The backend does not own / recognise the requested verb (the caller may
    /// fall through or report unresolved).
    NotHandled,
}

/// Identifies the dataset a record-aware store targets, by DATASET IDENTITY --
/// NOT a host path. The record-aware store (18.2) addresses a dataset by its DSN
/// within its owning environment, with an optional catalog-identity token; a
/// mainframe Command Environment (BRC.4, Req 18.7) maps this to a
/// `ff_dscatalog::DatasetAccess`. This type is ff-vfs-local on purpose: ff-vfs
/// MUST NOT depend on ff-dscatalog / ff-volume (acyclic DAG), so the identity is
/// carried as plain owned strings here and resolved by the mainframe CE.
///
/// Validates: command-environments Requirement 18.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreTarget {
    /// The dataset name (DSN) the records are stored under.
    pub dsn: String,
    /// The name of the Command Environment that owns the dataset (the addressing
    /// target the records are stored through).
    pub owning_env: String,
    /// An optional catalog-identity token (e.g. a catalog alias / qualifier) when
    /// the dataset identity needs more than the DSN alone; `None` when the DSN
    /// within `owning_env` is sufficient.
    pub catalog_id: Option<String>,
}

/// The record format (RECFM) of a record-aware store, in an ff-vfs-local minimal
/// representation. This is DELIBERATELY NOT `ff-dscatalog::Recfm` (ff-vfs must not
/// depend on ff-dscatalog) and NOT the editor's `RecordFormat` (that lives in
/// ff-document-model and is consumed by the editor SAVE walk, BRC.3). The
/// mainframe CE maps this to the dataset's real RECFM on store.
///
/// `#[non_exhaustive]` so it can gain variants (e.g. spanned formats) without a
/// breaking change to the contract.
///
/// Validates: command-environments Requirement 18.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordFormatKind {
    /// Fixed-length records (RECFM F / FB).
    Fixed,
    /// Variable-length records (RECFM V / VB).
    Variable,
    /// Undefined-length records (RECFM U).
    Undefined,
}

/// The record attributes a record-aware store carries alongside the records:
/// the RECFM, the logical record length, and an opaque encoding label. These are
/// ff-vfs-local (no ff-dscatalog reuse) so the contract stays dependency-free; the
/// mainframe CE (BRC.4) translates them into the dataset's real attributes.
///
/// Validates: command-environments Requirement 18.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordAttrs {
    /// The record format of the stored records.
    pub recfm: RecordFormatKind,
    /// The logical record length (LRECL) in bytes.
    pub lrecl: u32,
    /// An opaque encoding label (e.g. "cp037", "utf-8"); the backend interprets
    /// it. Carried as a string so ff-vfs does not own an encoding enum.
    pub encoding: String,
}

/// An OBJECT-SAFE pull stream over the editor's framed records, modelled in
/// ff-vfs so the record-aware store contract has NO dependency on
/// ff-document-model. The caller (the editor SAVE walk, BRC.3) adapts its framed
/// records to this trait; the backend pulls one raw record's bytes at a time.
///
/// Object-safe by construction: `&mut self`, no generics, no `Self`-returning
/// methods, no associated types -- so it is usable as `&mut dyn RecordSource` in
/// the (also object-safe) [`BackendEnvironment::save_records`] signature.
///
/// Validates: command-environments Requirement 18.2, 18.4
pub trait RecordSource {
    /// Pull the next record's raw bytes, or `None` at end of stream. The borrow
    /// keeps the per-record bytes owned by the source, avoiding an allocation per
    /// record while remaining object-safe.
    fn next_record(&mut self) -> Option<&[u8]>;
}

/// The outcome of a record-aware store ([`BackendEnvironment::save_records`]).
///
/// This is a SIBLING to [`BackendOutcome`] rather than a reuse: the record store
/// has a third state the byte path does not need -- `NotRecordCapable`, distinct
/// from a handled-with-nonzero-rc soft failure. Reusing `BackendOutcome` would
/// conflate "declined because this backend is not record-capable" (nothing was
/// attempted) with `NotHandled` (the verb was not recognised), which carry
/// different caller semantics. A dedicated outcome keeps the rc-carrying success
/// and the not-capable decline unambiguous while mirroring `BackendOutcome`'s rc
/// convention.
///
/// Validates: command-environments Requirement 18.8
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordStoreOutcome {
    /// The record-aware backend stored the records. `rc == 0` on success; a
    /// non-zero `rc` is a backend-reported SOFT failure (e.g. an x37 space-full
    /// abend the mainframe `DatasetAccess` surfaces -- Req 18.8).
    Stored { rc: i32 },
    /// This backend is NOT record-capable (the provided default, or a backend
    /// that declines). Distinct from `Stored { rc != 0 }`: nothing was attempted.
    NotRecordCapable,
}

/// A backend / resource Command Environment: owns the store semantics for one
/// file system (or dataset / database) kind. Implemented by the `ff-ce-*` crates
/// and the mainframe/VSAM CE; shell-free (no `WorkbenchShell`, no egui), so these
/// crates are pure leaves depending only on `ff-vfs` + std.
///
/// Validates: command-environments Requirement 16.1, 16.4, 16.7, 18.1, 18.2, 18.3, 18.4
pub trait BackendEnvironment: Send + Sync {
    /// The stable NAME this backend registers under and is addressed by (e.g.
    /// "NTFS", "POSIX", "HOSTFS", later "MAINFRAME" / "SQLITE"). Case-insensitive
    /// at the addressing boundary.
    fn name(&self) -> &str;

    /// Whether this file system treats paths case-sensitively (a cheap FS default
    /// the light host CEs expose: NTFS = false, POSIX = true). Advisory metadata;
    /// richer attributes arrive with the heavier backends.
    fn is_case_sensitive(&self) -> bool;

    /// Perform the backend's store WRITE for a resource: write `bytes` to the
    /// resource identified by `path`. For a light host FS CE this is a plain
    /// OS-backed byte write (identical to the former direct local-FS save); a
    /// mainframe backend would instead pack records per RECFM/LRECL. The caller
    /// (FFEDIT via `dispatch_to_environment`) owns the dirty-aware orchestration
    /// (read the buffer, clear the modified flag, set the save point); this method
    /// owns ONLY the physical write, so the ownership boundary (Req 14.7) is clean.
    ///
    /// This BYTE store entry is RETAINED UNCHANGED by the record-aware contract
    /// (Req 18.1): host CEs keep performing a byte-identical native save.
    ///
    /// Validates: command-environments Requirement 14.5, 14.6, 16.5, 18.1
    fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;

    /// The Record_Store_Contract: the record-aware store entry, ADDITIVE
    /// alongside the retained byte [`save`](BackendEnvironment::save) (Req 18.2).
    /// It carries the dataset identity (`target`, a [`StoreTarget`] -- NOT a host
    /// path), the editor's framed records as an object-safe stream (`records`,
    /// `&mut dyn RecordSource`), and the record attributes (`attrs`, a
    /// [`RecordAttrs`] carrying RECFM / LRECL / encoding). A record-capable
    /// backend (the mainframe CE, BRC.4) stores each record through its dataset
    /// access and returns [`RecordStoreOutcome::Stored`] with a return code.
    ///
    /// PROVIDED DEFAULT (Req 18.3): declines with
    /// [`RecordStoreOutcome::NotRecordCapable`], so an existing host-FS Command
    /// Environment compiles and behaves UNCHANGED without implementing it -- only
    /// a record-aware CE overrides it. The outcome is a sibling to
    /// [`BackendOutcome`] so "not record-capable" is distinct from a handled soft
    /// failure (see [`RecordStoreOutcome`]).
    ///
    /// Object-safe (Req 18.4): `&self`, a `&dyn`/`&mut dyn` stream, concrete
    /// struct/enum parameters and return -- no generics, so the trait stays
    /// `dyn`-compatible for the `Environment_Registry`.
    ///
    /// Validates: command-environments Requirement 18.2, 18.3
    fn save_records(
        &self,
        target: &StoreTarget,
        records: &mut dyn RecordSource,
        attrs: &RecordAttrs,
    ) -> RecordStoreOutcome {
        let _ = (target, records, attrs);
        RecordStoreOutcome::NotRecordCapable
    }

    /// Whether this backend is RECORD-CAPABLE (overrides
    /// [`save_records`](BackendEnvironment::save_records) to store records). The
    /// provided default is `false`, so host CEs advertise as not record-capable
    /// without any edit; the mainframe CE (BRC.4) overrides to `true`.
    ///
    /// Validates: command-environments Requirement 18.3
    fn record_capable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// A host-style backend implementing ONLY the byte `save` (mirrors a host CE
    /// such as `ff-ce-ntfs` / `ff-ce-posix`); it inherits the record defaults.
    struct ByteOnlyBackend;

    impl BackendEnvironment for ByteOnlyBackend {
        fn name(&self) -> &str {
            "BYTE_ONLY"
        }

        fn is_case_sensitive(&self) -> bool {
            true
        }

        fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            std::fs::write(path, bytes)
        }
    }

    /// A record-capable backend that stores records (and the threaded-through
    /// target + attrs) into in-memory sinks, and reports a configurable rc.
    struct RecordCapableBackend {
        records: Mutex<Vec<Vec<u8>>>,
        last_target: Mutex<Option<StoreTarget>>,
        last_attrs: Mutex<Option<RecordAttrs>>,
        rc: i32,
    }

    impl RecordCapableBackend {
        fn with_rc(rc: i32) -> Self {
            Self {
                records: Mutex::new(Vec::new()),
                last_target: Mutex::new(None),
                last_attrs: Mutex::new(None),
                rc,
            }
        }
    }

    impl BackendEnvironment for RecordCapableBackend {
        fn name(&self) -> &str {
            "RECORD_CAPABLE"
        }

        fn is_case_sensitive(&self) -> bool {
            false
        }

        fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            std::fs::write(path, bytes)
        }

        fn save_records(
            &self,
            target: &StoreTarget,
            records: &mut dyn RecordSource,
            attrs: &RecordAttrs,
        ) -> RecordStoreOutcome {
            *self.last_target.lock().expect("target lock") = Some(target.clone());
            *self.last_attrs.lock().expect("attrs lock") = Some(attrs.clone());
            let mut sink = self.records.lock().expect("records lock");
            while let Some(rec) = records.next_record() {
                sink.push(rec.to_vec());
            }
            RecordStoreOutcome::Stored { rc: self.rc }
        }

        fn record_capable(&self) -> bool {
            true
        }
    }

    /// A `RecordSource` over a borrowed slice of record byte-slices.
    struct SliceRecordSource<'a> {
        records: &'a [&'a [u8]],
        index: usize,
    }

    impl<'a> SliceRecordSource<'a> {
        fn new(records: &'a [&'a [u8]]) -> Self {
            Self { records, index: 0 }
        }
    }

    impl RecordSource for SliceRecordSource<'_> {
        fn next_record(&mut self) -> Option<&[u8]> {
            let rec = self.records.get(self.index)?;
            self.index += 1;
            Some(rec)
        }
    }

    fn sample_target() -> StoreTarget {
        StoreTarget {
            dsn: "USER.TEST.DATA".to_string(),
            owning_env: "MAINFRAME".to_string(),
            catalog_id: Some("CATALOG.MASTER".to_string()),
        }
    }

    fn sample_attrs() -> RecordAttrs {
        RecordAttrs {
            recfm: RecordFormatKind::Fixed,
            lrecl: 80,
            encoding: "cp037".to_string(),
        }
    }

    #[test]
    fn byte_save_entry_is_retained_and_writes_bytes() {
        // Validates: command-environments Requirement 18.1
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("out.bin");
        let backend = ByteOnlyBackend;
        let bytes = b"hello record contract";
        backend.save(&path, bytes).expect("byte save");
        let read_back = std::fs::read(&path).expect("read back");
        assert_eq!(read_back, bytes);
    }

    #[test]
    fn default_backend_is_not_record_capable() {
        // Validates: command-environments Requirement 18.3
        let backend = ByteOnlyBackend;
        assert!(!backend.record_capable());
    }

    #[test]
    fn default_save_records_declines() {
        // Validates: command-environments Requirement 18.3
        let backend = ByteOnlyBackend;
        let target = sample_target();
        let attrs = sample_attrs();
        let recs: [&[u8]; 1] = [b"ABC"];
        let mut source = SliceRecordSource::new(&recs);
        let outcome = backend.save_records(&target, &mut source, &attrs);
        assert_eq!(outcome, RecordStoreOutcome::NotRecordCapable);
    }

    #[test]
    fn record_capable_backend_stores_records_with_target_and_attrs() {
        // Validates: command-environments Requirement 18.2
        let backend = RecordCapableBackend::with_rc(0);
        assert!(backend.record_capable());
        let target = sample_target();
        let attrs = sample_attrs();
        let recs: [&[u8]; 3] = [b"REC1", b"REC2", b"REC3"];
        let mut source = SliceRecordSource::new(&recs);
        let outcome = backend.save_records(&target, &mut source, &attrs);
        assert_eq!(outcome, RecordStoreOutcome::Stored { rc: 0 });

        let stored = backend.records.lock().expect("records lock");
        assert_eq!(
            *stored,
            vec![b"REC1".to_vec(), b"REC2".to_vec(), b"REC3".to_vec()]
        );
        let stored_target = backend.last_target.lock().expect("target lock");
        assert_eq!(stored_target.as_ref(), Some(&target));
        let stored_attrs = backend.last_attrs.lock().expect("attrs lock");
        assert_eq!(stored_attrs.as_ref(), Some(&attrs));
    }

    #[test]
    fn backend_environment_is_object_safe_for_both_entries() {
        // Validates: command-environments Requirement 18.4
        let backend: Box<dyn BackendEnvironment> = Box::new(RecordCapableBackend::with_rc(0));
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("dyn.bin");
        backend.save(&path, b"via dyn").expect("byte save via dyn");
        assert_eq!(std::fs::read(&path).expect("read back"), b"via dyn");

        let target = sample_target();
        let attrs = sample_attrs();
        let recs: [&[u8]; 2] = [b"R1", b"R2"];
        let mut source = SliceRecordSource::new(&recs);
        let outcome = backend.save_records(&target, &mut source, &attrs);
        assert_eq!(outcome, RecordStoreOutcome::Stored { rc: 0 });
    }

    #[test]
    fn record_store_outcome_carries_rc_and_distinguishes_decline() {
        // Validates: command-environments Requirement 18.8
        let success = RecordCapableBackend::with_rc(0);
        let soft_fail = RecordCapableBackend::with_rc(8);
        let declining = ByteOnlyBackend;
        let target = sample_target();
        let attrs = sample_attrs();
        let recs: [&[u8]; 1] = [b"X"];

        let mut s1 = SliceRecordSource::new(&recs);
        let success_outcome = success.save_records(&target, &mut s1, &attrs);
        assert!(matches!(
            success_outcome,
            RecordStoreOutcome::Stored { rc: 0 }
        ));

        let mut s2 = SliceRecordSource::new(&recs);
        let soft_outcome = soft_fail.save_records(&target, &mut s2, &attrs);
        match soft_outcome {
            RecordStoreOutcome::Stored { rc } => assert_eq!(rc, 8),
            other => panic!("expected soft-failure Stored, got {other:?}"),
        }

        let mut s3 = SliceRecordSource::new(&recs);
        let declined = declining.save_records(&target, &mut s3, &attrs);
        assert_eq!(declined, RecordStoreOutcome::NotRecordCapable);
        assert_ne!(declined, RecordStoreOutcome::Stored { rc: 0 });
    }
}
