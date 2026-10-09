//! Physical storage backends for mainframe record/VSAM datasets.
//!
//! CR-CH-059 RC.A.3: the duplicate `ff-dscatalog::storage::StorageProvider`
//! trait (UUID `ObjectId` / `&[ProviderCapability]` / `workspace_root`-threaded)
//! has been RETIRED. The five backends (NativeFile, ESDS, KSDS/SqliteRecord,
//! RRDS, ISAM) now implement the SINGLE physical seam `ff_vfs::StorageProvider`
//! (opaque `StorageLocator`, `HashSet<StorageCapability>`, `VfsError`). The old
//! `ProviderCapability` variants map 1:1 to `ff_vfs::StorageCapability`.
//!
//! Validates: virtual-file-system Requirement 13.1, 13.2, 13.5.

mod esds;
mod isam;
mod native;
mod rrds;
mod sqlite_record;

pub use esds::{EsdsRecord, EsdsRecordAddress, NativeEsdsProvider};
pub use isam::IsamProvider;
pub use native::NativeFileProvider;
pub use rrds::{RrdsRecord, RrdsSlot, SqliteRrdsProvider};
pub use sqlite_record::{
    AlternateIndex, KeyCollation, KeyDefinition, KeyType, KsdsKeyDefinition, KsdsRecord,
    PrimaryKeyDefinition, SqliteRecordProvider,
};
