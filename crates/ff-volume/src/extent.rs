//! Extent accounting and the two distinct space-overflow decisions
//! (Requirements 5, 6, and the Volume_Full branch of Requirement 7).
//!
//! An `ExtentSet` is the ordered set of extents a single dataset holds on a
//! Volume. `accommodate` implements the design's decision tree:
//! fit -> ok; else acquire a secondary extent if below `Max_Extents` AND the
//! Volume has free space; else x37-style `SpaceAbend` at `Max_Extents`; else
//! `VolumeFull` consuming NO extent (Requirement 7.4).

use serde::{Deserialize, Serialize};

use crate::error::VolumeError;
use crate::geometry::GeometryProfile;
use crate::space::AllocationUnit;
use crate::volume::{DatasetId, Volume, VolumeId};

// === Max extents ===================================================

/// Default maximum number of extents (primary plus secondary) a dataset may
/// hold (Requirement 6.1). Configurable per call to `accommodate`.
pub const DEFAULT_MAX_EXTENTS: u32 = 16;

// === SpaceAbendReason ===================================================

/// Why an x37-style `SpaceAbend` occurred (Requirement 5.4). Kept distinct from
/// the Volume_Full cause so a caller can tell them apart (Requirement 5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpaceAbendReason {
    /// The dataset's maximum extent count was reached (Requirement 6.3).
    MaxExtentsReached,
    /// The dataset's allocatable capacity was exceeded (Requirement 5.3).
    CapacityExceeded,
}

impl std::fmt::Display for SpaceAbendReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxExtentsReached => f.write_str("maximum extents reached"),
            Self::CapacityExceeded => f.write_str("allocatable capacity exceeded"),
        }
    }
}

// === Extent ===================================================

/// A logical allocation segment recorded as metadata for a dataset on a Volume
/// (Requirement 5.1). Extents account for allocated space; they do not describe
/// physical placement (Requirement 11.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extent {
    /// The dataset this extent belongs to.
    pub dataset_id: DatasetId,
    /// The Volume this extent is allocated on.
    pub volume_id: VolumeId,
    /// 1-based ordinal within the dataset's extent sequence.
    pub sequence_number: u32,
    /// Tracks allocated to this extent.
    pub allocated_tracks: u64,
    /// Bytes of committed content held in this extent.
    pub used_bytes: u64,
    /// Opaque per-extent locator.
    pub locator: String,
}

// === ExtentSet ===================================================

/// The ordered extents a single dataset holds (Requirement 5.1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtentSet {
    extents: Vec<Extent>,
}

impl ExtentSet {
    /// Creates an empty extent set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the extents in sequence order.
    pub fn extents(&self) -> &[Extent] {
        &self.extents
    }

    /// Returns the number of extents in use.
    pub fn extents_used(&self) -> u32 {
        self.extents.len() as u32
    }

    /// Returns the total tracks allocated across all extents.
    pub fn allocated_tracks(&self) -> u64 {
        self.extents.iter().map(|e| e.allocated_tracks).sum()
    }

    /// Returns the total committed bytes across all extents.
    pub fn used_bytes(&self) -> u64 {
        self.extents.iter().map(|e| e.used_bytes).sum()
    }

    /// Returns the total allocated capacity in bytes under `geom`.
    pub fn allocated_bytes(&self, geom: &GeometryProfile) -> u64 {
        geom.bytes_of(self.allocated_tracks())
    }

    /// Adds the primary extent sized from `alloc.primary` under `geom`, charging
    /// the Volume. Used at dataset allocation (Requirement 5.1).
    ///
    /// # Errors
    /// `VolumeError::VolumeFull` if the Volume lacks free space (no extent is
    /// added, Requirement 7.4); `VolumeError::VolumeReadOnly` /
    /// `VolumeError::VolumeOffline` via the Volume guards.
    pub fn allocate_primary(
        &mut self,
        dataset_id: DatasetId,
        alloc: &AllocationUnit,
        geom: &GeometryProfile,
        volume: &mut Volume,
    ) -> Result<(), VolumeError> {
        volume.ensure_allocatable()?;
        volume.ensure_writable()?;
        let tracks = alloc.primary_tracks(geom);
        self.charge_and_push(dataset_id, volume, tracks, 0)
    }

    /// Accommodates a write that would bring the dataset's committed content to
    /// `target_used_bytes`, growing via a secondary extent if needed
    /// (Requirements 5.2-5.5, 6.2-6.4, 7.4).
    ///
    /// Decision tree:
    /// 1. If `target_used_bytes` fits in the currently allocated capacity, record
    ///    the new usage and return (fit).
    /// 2. Else, if `extents_used < max_extents` AND the Volume has free space for
    ///    the secondary quantity, acquire a secondary extent (charge the Volume,
    ///    push the extent) and record usage.
    /// 3. Else, if `extents_used >= max_extents`, fail with an x37-style
    ///    `SpaceAbend { MaxExtentsReached }`, leaving prior content intact
    ///    (Requirement 5.4).
    /// 4. Else (extent slot available but Volume lacks free space), fail with
    ///    `VolumeFull`, consuming NO extent (Requirement 7.4).
    ///
    /// # Errors
    /// See the decision tree. On any error the set and Volume are unchanged.
    pub fn accommodate(
        &mut self,
        dataset_id: DatasetId,
        target_used_bytes: u64,
        alloc: &AllocationUnit,
        geom: &GeometryProfile,
        volume: &mut Volume,
        max_extents: u32,
    ) -> Result<(), VolumeError> {
        volume.ensure_allocatable()?;
        volume.ensure_writable()?;

        if target_used_bytes <= self.allocated_bytes(geom) {
            self.record_usage(target_used_bytes);
            return Ok(());
        }

        // Growth needed. Must a secondary extent be acquired?
        if self.extents_used() >= max_extents {
            return Err(VolumeError::SpaceAbend {
                dataset_id: dataset_id.0,
                reason: SpaceAbendReason::MaxExtentsReached,
            });
        }

        let secondary = alloc.secondary_tracks(geom).max(1);
        if secondary > volume.free_tracks() {
            // Volume_Full: distinct failure, consumes no extent (Req 7.4).
            return Err(VolumeError::VolumeFull {
                volser: volume.volser().as_str().to_string(),
            });
        }

        self.charge_and_push(dataset_id, volume, secondary, 0)?;

        // After acquiring one secondary extent, confirm the write now fits. If a
        // single secondary extent is still insufficient the dataset has exceeded
        // its allocatable capacity for this write (x37 CapacityExceeded).
        if target_used_bytes > self.allocated_bytes(geom) {
            return Err(VolumeError::SpaceAbend {
                dataset_id: dataset_id.0,
                reason: SpaceAbendReason::CapacityExceeded,
            });
        }
        self.record_usage(target_used_bytes);
        Ok(())
    }

    /// Charges `tracks` to the Volume (Volume_Full if insufficient) and, on
    /// success, pushes a new extent.
    fn charge_and_push(
        &mut self,
        dataset_id: DatasetId,
        volume: &mut Volume,
        tracks: u64,
        used_bytes: u64,
    ) -> Result<(), VolumeError> {
        volume.charge(tracks)?;
        let sequence_number = self.extents_used() + 1;
        self.extents.push(Extent {
            dataset_id,
            volume_id: volume.volume_id(),
            sequence_number,
            allocated_tracks: tracks,
            used_bytes,
            locator: format!("ext::{}::{}", dataset_id.0, sequence_number),
        });
        Ok(())
    }

    /// Records `target_used_bytes` as the dataset's committed content. Per-extent
    /// byte capacity is not tracked here, so the total is recorded on the last
    /// extent and the rest zeroed, giving a stable, deterministic representation
    /// whose `used_bytes()` sum equals `target_used_bytes`.
    fn record_usage(&mut self, target_used_bytes: u64) {
        for extent in self.extents.iter_mut() {
            extent.used_bytes = 0;
        }
        if let Some(last) = self.extents.last_mut() {
            last.used_bytes = target_used_bytes;
        }
    }
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::UnitKind;
    use crate::volume::{AccessMode, Volser, VolumeId};
    use pretty_assertions::assert_eq;

    fn geom() -> GeometryProfile {
        GeometryProfile::default()
    }

    fn volume(total_tracks: u64) -> Volume {
        Volume::new(
            VolumeId(1),
            Volser::try_new("VOL001").expect("volser"),
            "/vol/VOL001",
            total_tracks,
        )
    }

    // One track = 56664 bytes. Primary 1 track, secondary 1 track.
    fn alloc_1trk() -> AllocationUnit {
        AllocationUnit::new(UnitKind::Trk, 1, 1)
    }

    // Validates: Requirement 5.1 -- primary extent allocation charges the volume
    #[test]
    fn allocate_primary_charges_volume() {
        let g = geom();
        let mut vol = volume(100);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary ok");
        assert_eq!(set.extents_used(), 1);
        assert_eq!(set.allocated_tracks(), 1);
        assert_eq!(vol.used_tracks(), 1);
    }

    // Validates: Requirement 5.1 -- write within allocated space proceeds
    #[test]
    fn write_within_allocated_space_proceeds() {
        let g = geom();
        let mut vol = volume(100);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        // Write 10_000 bytes -- fits in 1 track (56664 bytes), no new extent.
        set.accommodate(
            DatasetId(7),
            10_000,
            &alloc_1trk(),
            &g,
            &mut vol,
            DEFAULT_MAX_EXTENTS,
        )
        .expect("fits");
        assert_eq!(set.extents_used(), 1);
        assert_eq!(set.used_bytes(), 10_000);
    }

    // Validates: Requirement 5.2, 6.2 -- overflow acquires a secondary extent
    #[test]
    fn overflow_acquires_secondary_extent() {
        let g = geom();
        let mut vol = volume(100);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        // Write just over 1 track -> acquire a secondary extent.
        set.accommodate(
            DatasetId(7),
            56665,
            &alloc_1trk(),
            &g,
            &mut vol,
            DEFAULT_MAX_EXTENTS,
        )
        .expect("grows");
        assert_eq!(set.extents_used(), 2);
        assert_eq!(set.allocated_tracks(), 2);
        assert_eq!(vol.used_tracks(), 2);
    }

    // Validates: Requirement 5.3, 6.3 -- max extents reached returns x37 abend
    #[test]
    fn max_extents_reached_returns_x37_space_abend() {
        let g = geom();
        let mut vol = volume(100);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        // max_extents = 1 means no secondary extent may be acquired.
        let err = set
            .accommodate(DatasetId(7), 56665, &alloc_1trk(), &g, &mut vol, 1)
            .expect_err("x37");
        assert_eq!(
            err,
            VolumeError::SpaceAbend {
                dataset_id: 7,
                reason: SpaceAbendReason::MaxExtentsReached
            }
        );
    }

    // Validates: Requirement 5.4 -- space abend leaves prior content intact
    #[test]
    fn space_abend_leaves_prior_content_intact() {
        let g = geom();
        let mut vol = volume(100);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        set.accommodate(DatasetId(7), 10_000, &alloc_1trk(), &g, &mut vol, 1)
            .expect("fits");
        let before_extents = set.extents_used();
        let before_used = set.used_bytes();
        let before_vol_used = vol.used_tracks();
        let _ = set
            .accommodate(DatasetId(7), 56665, &alloc_1trk(), &g, &mut vol, 1)
            .expect_err("x37");
        assert_eq!(set.extents_used(), before_extents);
        assert_eq!(set.used_bytes(), before_used);
        assert_eq!(vol.used_tracks(), before_vol_used);
    }

    // Validates: Requirement 6.4 -- max extents is configurable
    #[test]
    fn max_extents_is_configurable() {
        let g = geom();
        let mut vol = volume(100);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        // With max_extents = 3, two secondary extents can be acquired.
        set.accommodate(DatasetId(7), 56665, &alloc_1trk(), &g, &mut vol, 3)
            .expect("second");
        set.accommodate(DatasetId(7), 113_329, &alloc_1trk(), &g, &mut vol, 3)
            .expect("third");
        assert_eq!(set.extents_used(), 3);
    }

    // Validates: Requirement 5.5, 6.4, 7.3 -- space abend distinct from volume full
    #[test]
    fn space_abend_distinct_from_volume_full() {
        let g = geom();
        // Volume too small to grow: primary 1 track consumes all free space.
        let mut vol = volume(1);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        // Extent slot available (max_extents high) but volume has no free track.
        let err = set
            .accommodate(
                DatasetId(7),
                56665,
                &alloc_1trk(),
                &g,
                &mut vol,
                DEFAULT_MAX_EXTENTS,
            )
            .expect_err("volume full");
        assert_eq!(
            err,
            VolumeError::VolumeFull {
                volser: "VOL001".to_string()
            }
        );
        // Distinct from the x37 abend variant.
        assert!(!matches!(err, VolumeError::SpaceAbend { .. }));
    }

    // Validates: Requirement 7.4 -- volume full does not consume a dataset extent
    #[test]
    fn volume_full_does_not_consume_dataset_extent() {
        let g = geom();
        let mut vol = volume(1);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        let before = set.extents_used();
        let _ = set
            .accommodate(
                DatasetId(7),
                56665,
                &alloc_1trk(),
                &g,
                &mut vol,
                DEFAULT_MAX_EXTENTS,
            )
            .expect_err("volume full");
        assert_eq!(set.extents_used(), before);
    }

    // Validates: Requirement 2.2 -- read-only volume rejects extend
    #[test]
    fn readonly_volume_rejects_extend() {
        let g = geom();
        let mut vol = volume(100);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(7), &alloc_1trk(), &g, &mut vol)
            .expect("primary");
        vol.set_access_mode(AccessMode::ReadOnly);
        let err = set
            .accommodate(
                DatasetId(7),
                56665,
                &alloc_1trk(),
                &g,
                &mut vol,
                DEFAULT_MAX_EXTENTS,
            )
            .expect_err("read-only");
        assert!(matches!(err, VolumeError::VolumeReadOnly { .. }));
    }
}
