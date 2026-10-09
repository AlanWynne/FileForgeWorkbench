//! The Volume entity, its identity newtypes, status/access-mode behaviour, the
//! VOLSER-uniqueness registry, and the track-based capacity counters.
//!
//! Capacity is stored in TRACKS (the smallest emulated unit, decision 3);
//! cylinders are a derived view (see `reporting`). `free_tracks = total_tracks
//! - used_tracks`.

use serde::{Deserialize, Serialize};

use crate::error::VolumeError;

// === Identity newtypes ===================================================

/// Stable identifier for a Volume. Supplied by the persistence layer
/// (`ff-dscatalog`); the model treats it as opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VolumeId(pub u64);

/// Stable identifier for a Dataset. Supplied by the persistence layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DatasetId(pub u64);

/// A Volume serial. Stored uppercased and validated non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Volser(String);

impl Volser {
    /// Creates a VOLSER, uppercasing the input. Returns `VolumeNotFound`-free
    /// construction; an empty serial is rejected as a `DuplicateVolser`-free
    /// validation error expressed as `VolumeNotFound` would be wrong, so an
    /// empty serial is reported via `try_new`.
    ///
    /// # Errors
    /// Returns `VolumeError::VolumeNotFound` with an explanatory reference when
    /// the serial is empty after trimming.
    pub fn try_new(serial: impl Into<String>) -> Result<Self, VolumeError> {
        let trimmed = serial.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(VolumeError::VolumeNotFound {
                reference: "<empty VOLSER>".to_string(),
            });
        }
        Ok(Self(trimmed.to_uppercase()))
    }

    /// Returns the VOLSER string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Volser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// === Status and access mode ===================================================

/// Operational state of a Volume. Offline Volumes reject new allocation
/// (Requirement 2.1). Further source states (Mounted/Recovering/Error) are
/// deferred, hence `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VolumeStatus {
    /// The Volume is available for allocation and I/O.
    Online,
    /// The Volume rejects new allocation.
    Offline,
}

/// Whether a Volume permits writes. A ReadOnly Volume rejects write, delete,
/// and extend operations (Requirement 2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AccessMode {
    /// Reads and writes are permitted.
    ReadWrite,
    /// Only reads are permitted.
    ReadOnly,
}

// === Volume ===================================================

/// A first-class emulated DASD-style Volume (Requirement 1).
///
/// Capacity counters are in TRACKS (decision 3). `reserved_tracks` records
/// space held back from allocation (e.g. for VTOC-style overhead) and is
/// excluded from the free pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volume {
    volume_id: VolumeId,
    volser: Volser,
    display_name: Option<String>,
    storage_uri: String,
    status: VolumeStatus,
    access_mode: AccessMode,
    total_tracks: u64,
    used_tracks: u64,
    reserved_tracks: u64,
}

impl Volume {
    /// Creates a Volume with the given identity, location, and total capacity.
    /// Starts Online/ReadWrite with no used or reserved tracks.
    pub fn new(
        volume_id: VolumeId,
        volser: Volser,
        storage_uri: impl Into<String>,
        total_tracks: u64,
    ) -> Self {
        Self {
            volume_id,
            volser,
            display_name: None,
            storage_uri: storage_uri.into(),
            status: VolumeStatus::Online,
            access_mode: AccessMode::ReadWrite,
            total_tracks,
            used_tracks: 0,
            reserved_tracks: 0,
        }
    }

    /// Sets the optional display name (builder style).
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the number of reserved (non-allocatable) tracks (builder style).
    pub fn with_reserved_tracks(mut self, reserved: u64) -> Self {
        self.reserved_tracks = reserved;
        self
    }

    /// Returns the Volume id.
    pub fn volume_id(&self) -> VolumeId {
        self.volume_id
    }

    /// Returns the VOLSER.
    pub fn volser(&self) -> &Volser {
        &self.volser
    }

    /// Returns the optional display name.
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    /// Returns the storage URI (today's Repository root; ADR-001 StorageProvider
    /// seam).
    pub fn storage_uri(&self) -> &str {
        &self.storage_uri
    }

    /// Returns the current status.
    pub fn status(&self) -> VolumeStatus {
        self.status
    }

    /// Returns the current access mode.
    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    // === Status / access transitions (Requirement 2.3) ================

    /// Marks the Volume Online.
    pub fn set_online(&mut self) {
        self.status = VolumeStatus::Online;
    }

    /// Marks the Volume Offline.
    pub fn set_offline(&mut self) {
        self.status = VolumeStatus::Offline;
    }

    /// Mounts the Volume (modelled as bringing it Online).
    pub fn mount(&mut self) {
        self.set_online();
    }

    /// Unmounts the Volume (modelled as taking it Offline).
    pub fn unmount(&mut self) {
        self.set_offline();
    }

    /// Sets the access mode.
    pub fn set_access_mode(&mut self, mode: AccessMode) {
        self.access_mode = mode;
    }

    // === Guards (Requirements 2.1, 2.2) ================

    /// Rejects allocation targeting an Offline Volume (Requirement 2.1).
    ///
    /// # Errors
    /// `VolumeError::VolumeOffline` when the Volume is Offline.
    pub fn ensure_allocatable(&self) -> Result<(), VolumeError> {
        match self.status {
            VolumeStatus::Online => Ok(()),
            VolumeStatus::Offline => Err(VolumeError::VolumeOffline {
                volser: self.volser.as_str().to_string(),
            }),
        }
    }

    /// Rejects write/delete/extend targeting a ReadOnly Volume (Requirement
    /// 2.2).
    ///
    /// # Errors
    /// `VolumeError::VolumeReadOnly` when the Volume is ReadOnly.
    pub fn ensure_writable(&self) -> Result<(), VolumeError> {
        match self.access_mode {
            AccessMode::ReadWrite => Ok(()),
            AccessMode::ReadOnly => Err(VolumeError::VolumeReadOnly {
                volser: self.volser.as_str().to_string(),
            }),
        }
    }

    // === Capacity (Requirement 7) ================

    /// Returns the total capacity in tracks.
    pub fn total_tracks(&self) -> u64 {
        self.total_tracks
    }

    /// Returns the used capacity in tracks.
    pub fn used_tracks(&self) -> u64 {
        self.used_tracks
    }

    /// Returns the free capacity in tracks: `total - used - reserved`, saturating
    /// at zero.
    pub fn free_tracks(&self) -> u64 {
        self.total_tracks
            .saturating_sub(self.used_tracks)
            .saturating_sub(self.reserved_tracks)
    }

    /// Reserves (charges) `tracks` against free capacity (Requirement 7.2).
    ///
    /// # Errors
    /// `VolumeError::VolumeFull` when `tracks` exceeds `free_tracks`; no state
    /// changes on failure (Requirement 7.4).
    pub fn charge(&mut self, tracks: u64) -> Result<(), VolumeError> {
        if tracks > self.free_tracks() {
            return Err(VolumeError::VolumeFull {
                volser: self.volser.as_str().to_string(),
            });
        }
        self.used_tracks += tracks;
        Ok(())
    }

    /// Releases `tracks` back to free capacity (saturating at zero used).
    pub fn release(&mut self, tracks: u64) {
        self.used_tracks = self.used_tracks.saturating_sub(tracks);
    }
}

// === VolumeRegistry ===================================================

/// An in-memory collection of Volumes enforcing VOLSER uniqueness
/// (Requirement 1.2). `ff-dscatalog` maps persisted rows to/from this registry.
#[derive(Debug, Default, Clone)]
pub struct VolumeRegistry {
    volumes: Vec<Volume>,
}

impl VolumeRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a Volume, rejecting a duplicate VOLSER (Requirement 1.2).
    ///
    /// # Errors
    /// `VolumeError::DuplicateVolser` when a Volume with the same VOLSER already
    /// exists.
    pub fn define(&mut self, volume: Volume) -> Result<VolumeId, VolumeError> {
        if self.find_by_volser(volume.volser()).is_some() {
            return Err(VolumeError::DuplicateVolser {
                volser: volume.volser().as_str().to_string(),
            });
        }
        let id = volume.volume_id();
        self.volumes.push(volume);
        Ok(id)
    }

    /// Returns the number of registered Volumes.
    pub fn len(&self) -> usize {
        self.volumes.len()
    }

    /// Returns `true` when no Volumes are registered.
    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }

    /// Finds a Volume by VOLSER.
    pub fn find_by_volser(&self, volser: &Volser) -> Option<&Volume> {
        self.volumes.iter().find(|v| v.volser() == volser)
    }

    /// Finds a Volume by id.
    pub fn find_by_id(&self, id: VolumeId) -> Option<&Volume> {
        self.volumes.iter().find(|v| v.volume_id() == id)
    }

    /// Finds a mutable Volume by id.
    pub fn find_by_id_mut(&mut self, id: VolumeId) -> Option<&mut Volume> {
        self.volumes.iter_mut().find(|v| v.volume_id() == id)
    }

    /// Iterates all registered Volumes.
    pub fn iter(&self) -> impl Iterator<Item = &Volume> {
        self.volumes.iter()
    }

    /// Verifies every Volume id in `required` is Online, returning the FIRST
    /// unavailable Volume's VOLSER otherwise (Requirement 2.4).
    ///
    /// # Errors
    /// `VolumeError::VolumeNotFound` if a required id is unknown, or
    /// `VolumeError::VolumeOffline` for the first Offline required Volume.
    pub fn ensure_all_online(&self, required: &[VolumeId]) -> Result<(), VolumeError> {
        for id in required {
            let volume = self
                .find_by_id(*id)
                .ok_or_else(|| VolumeError::VolumeNotFound {
                    reference: format!("volume_id {}", id.0),
                })?;
            volume.ensure_allocatable()?;
        }
        Ok(())
    }
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn vol(id: u64, serial: &str, total: u64) -> Volume {
        Volume::new(
            VolumeId(id),
            Volser::try_new(serial).expect("valid volser"),
            format!("/vol/{serial}"),
            total,
        )
    }

    // Validates: Requirement 1.1 -- Volume carries required fields
    #[test]
    fn volume_carries_required_fields() {
        let v = vol(1, "VOL001", 1000)
            .with_display_name("Primary")
            .with_reserved_tracks(10);
        assert_eq!(v.volume_id(), VolumeId(1));
        assert_eq!(v.volser().as_str(), "VOL001");
        assert_eq!(v.display_name(), Some("Primary"));
        assert_eq!(v.storage_uri(), "/vol/VOL001");
        assert_eq!(v.status(), VolumeStatus::Online);
        assert_eq!(v.access_mode(), AccessMode::ReadWrite);
        assert_eq!(v.total_tracks(), 1000);
        assert_eq!(v.used_tracks(), 0);
    }

    // Validates: Requirement 1.2 -- duplicate VOLSER is rejected
    #[test]
    fn duplicate_volser_is_rejected() {
        let mut reg = VolumeRegistry::new();
        reg.define(vol(1, "VOL001", 100)).expect("first accepted");
        let err = reg
            .define(vol(2, "VOL001", 100))
            .expect_err("duplicate rejected");
        assert_eq!(
            err,
            VolumeError::DuplicateVolser {
                volser: "VOL001".to_string()
            }
        );
        assert_eq!(reg.len(), 1);
    }

    // Validates: Requirement 1.2 -- distinct VOLSERs are accepted
    #[test]
    fn distinct_volsers_are_accepted() {
        let mut reg = VolumeRegistry::new();
        reg.define(vol(1, "VOL001", 100)).expect("first accepted");
        reg.define(vol(2, "VOL002", 100)).expect("second accepted");
        assert_eq!(reg.len(), 2);
    }

    // Validates: Requirement 1.1 -- VOLSER is stored uppercased
    #[test]
    fn volser_is_stored_uppercase() {
        let v = Volser::try_new("vol001").expect("valid");
        assert_eq!(v.as_str(), "VOL001");
    }

    // Validates: Requirement 1.1 -- empty VOLSER is rejected
    #[test]
    fn empty_volser_is_rejected() {
        assert!(Volser::try_new("   ").is_err());
    }

    // Validates: Requirement 2.1 -- offline volume rejects new allocation
    #[test]
    fn offline_volume_rejects_new_allocation() {
        let mut v = vol(1, "VOL001", 100);
        v.set_offline();
        let err = v.ensure_allocatable().expect_err("offline rejects");
        assert_eq!(
            err,
            VolumeError::VolumeOffline {
                volser: "VOL001".to_string()
            }
        );
    }

    // Validates: Requirement 2.2 -- read-only volume rejects write/delete/extend
    #[test]
    fn readonly_volume_rejects_write_delete_extend() {
        let mut v = vol(1, "VOL001", 100);
        v.set_access_mode(AccessMode::ReadOnly);
        let err = v.ensure_writable().expect_err("read-only rejects");
        assert_eq!(
            err,
            VolumeError::VolumeReadOnly {
                volser: "VOL001".to_string()
            }
        );
    }

    // Validates: Requirement 2.3 -- set-online/offline/mount/unmount transitions
    #[test]
    fn set_online_offline_mount_unmount_transition() {
        let mut v = vol(1, "VOL001", 100);
        v.set_offline();
        assert_eq!(v.status(), VolumeStatus::Offline);
        v.set_online();
        assert_eq!(v.status(), VolumeStatus::Online);
        v.unmount();
        assert_eq!(v.status(), VolumeStatus::Offline);
        v.mount();
        assert_eq!(v.status(), VolumeStatus::Online);
    }

    // Validates: Requirement 2.4 -- resolution reports first offline volume
    #[test]
    fn resolution_reports_first_offline_volume() {
        let mut reg = VolumeRegistry::new();
        reg.define(vol(1, "VOL001", 100)).expect("v1");
        let mut v2 = vol(2, "VOL002", 100);
        v2.set_offline();
        reg.define(v2).expect("v2");
        reg.define(vol(3, "VOL003", 100)).expect("v3");

        reg.ensure_all_online(&[VolumeId(1)]).expect("v1 online");
        let err = reg
            .ensure_all_online(&[VolumeId(1), VolumeId(2), VolumeId(3)])
            .expect_err("v2 offline");
        assert_eq!(
            err,
            VolumeError::VolumeOffline {
                volser: "VOL002".to_string()
            }
        );
    }

    // Validates: Requirement 2.4 -- unknown required volume reports not found
    #[test]
    fn resolution_reports_unknown_volume() {
        let reg = VolumeRegistry::new();
        let err = reg.ensure_all_online(&[VolumeId(99)]).expect_err("unknown");
        assert!(matches!(err, VolumeError::VolumeNotFound { .. }));
    }

    // Validates: Requirement 7.1 -- volume tracks used/free accounting
    #[test]
    fn volume_tracks_used_free_accounting() {
        let mut v = vol(1, "VOL001", 1000).with_reserved_tracks(100);
        assert_eq!(v.free_tracks(), 900);
        v.charge(300).expect("charge ok");
        assert_eq!(v.used_tracks(), 300);
        assert_eq!(v.free_tracks(), 600);
        v.release(100);
        assert_eq!(v.used_tracks(), 200);
        assert_eq!(v.free_tracks(), 700);
    }

    // Validates: Requirement 7.2 -- allocation beyond free space reports volume full
    #[test]
    fn allocation_beyond_free_space_reports_volume_full() {
        let mut v = vol(1, "VOL001", 500);
        let err = v.charge(600).expect_err("over capacity");
        assert_eq!(
            err,
            VolumeError::VolumeFull {
                volser: "VOL001".to_string()
            }
        );
        // Requirement 7.4: no state change on failure.
        assert_eq!(v.used_tracks(), 0);
    }
}
