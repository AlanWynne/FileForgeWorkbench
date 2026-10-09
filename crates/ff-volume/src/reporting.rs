//! Derived space reporting and the VTOC view (Requirement 8, 11.2).
//!
//! All counters are derived metadata computed from recorded extents and the
//! geometry profile in O(n) time over extents (O(1) for a single-extent
//! dataset). No physical content is scanned (Requirement 8.4, 11.2).

use crate::extent::ExtentSet;
use crate::geometry::GeometryProfile;
use crate::volume::{DatasetId, Volume};

// === DatasetSpaceReport ===================================================

/// Derived space usage for a single dataset (Requirement 8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatasetSpaceReport {
    /// Number of extents currently in use.
    pub extents_used: u32,
    /// Number of extents remaining before `Max_Extents`.
    pub extents_remaining: u32,
    /// Tracks allocated to the dataset.
    pub tracks_in_use: u64,
    /// Cylinders in use (derived from tracks, rounded up).
    pub cylinders_in_use: u64,
}

impl DatasetSpaceReport {
    /// Derives a dataset report from its extent set, the geometry profile, and
    /// the configured maximum extents (Requirement 8.1, 8.4).
    pub fn derive(extents: &ExtentSet, geom: &GeometryProfile, max_extents: u32) -> Self {
        let extents_used = extents.extents_used();
        let tracks_in_use = extents.allocated_tracks();
        Self {
            extents_used,
            extents_remaining: max_extents.saturating_sub(extents_used),
            tracks_in_use,
            cylinders_in_use: geom.cylinders(tracks_in_use),
        }
    }
}

// === VolumeSpaceReport ===================================================

/// Derived space usage for a Volume (Requirement 8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeSpaceReport {
    /// Total capacity in tracks.
    pub total_tracks: u64,
    /// Used capacity in tracks.
    pub used_tracks: u64,
    /// Free capacity in tracks.
    pub free_tracks: u64,
    /// Total capacity in cylinders (derived).
    pub total_cylinders: u64,
    /// Used capacity in cylinders (derived).
    pub used_cylinders: u64,
    /// Free capacity in cylinders (derived).
    pub free_cylinders: u64,
    /// Number of extents allocated across all datasets on the Volume.
    pub extents_allocated: u32,
}

impl VolumeSpaceReport {
    /// Derives a Volume report from the Volume's counters, the geometry profile,
    /// and the count of extents allocated on it (Requirement 8.2, 8.4).
    pub fn derive(volume: &Volume, geom: &GeometryProfile, extents_allocated: u32) -> Self {
        let total = volume.total_tracks();
        let used = volume.used_tracks();
        let free = volume.free_tracks();
        Self {
            total_tracks: total,
            used_tracks: used,
            free_tracks: free,
            total_cylinders: geom.cylinders(total),
            used_cylinders: geom.cylinders(used),
            free_cylinders: geom.cylinders(free),
            extents_allocated,
        }
    }
}

// === VtocView ===================================================

/// A derived, read-only listing of the datasets and extents on a Volume with the
/// Volume's usage counters (Requirement 8.3). Plays the role of a z/OS VTOC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VtocView {
    /// The Volume's derived usage report.
    pub volume: VolumeSpaceReport,
    /// Each dataset on the Volume with its derived report.
    pub datasets: Vec<(DatasetId, DatasetSpaceReport)>,
}

impl VtocView {
    /// Derives a VTOC view from a Volume and the extent sets of the datasets on
    /// it. O(n) in extents, no byte scanning (Requirement 8.3, 8.4, 11.2).
    pub fn derive(
        volume: &Volume,
        geom: &GeometryProfile,
        datasets: &[(DatasetId, ExtentSet)],
        max_extents: u32,
    ) -> Self {
        let extents_allocated: u32 = datasets.iter().map(|(_, set)| set.extents_used()).sum();
        let dataset_reports = datasets
            .iter()
            .map(|(id, set)| (*id, DatasetSpaceReport::derive(set, geom, max_extents)))
            .collect();
        Self {
            volume: VolumeSpaceReport::derive(volume, geom, extents_allocated),
            datasets: dataset_reports,
        }
    }
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extent::DEFAULT_MAX_EXTENTS;
    use crate::space::{AllocationUnit, UnitKind};
    use crate::volume::{Volser, VolumeId};
    use pretty_assertions::assert_eq;

    fn geom() -> GeometryProfile {
        GeometryProfile::default()
    }

    fn volume(total: u64) -> Volume {
        Volume::new(
            VolumeId(1),
            Volser::try_new("VOL001").expect("volser"),
            "/vol/VOL001",
            total,
        )
    }

    fn dataset_with_extents(vol: &mut Volume, primary: u64, grows: &[u64]) -> ExtentSet {
        let g = geom();
        let alloc = AllocationUnit::new(UnitKind::Trk, primary, 1);
        let mut set = ExtentSet::new();
        set.allocate_primary(DatasetId(5), &alloc, &g, vol)
            .expect("primary");
        for target in grows {
            set.accommodate(DatasetId(5), *target, &alloc, &g, vol, DEFAULT_MAX_EXTENTS)
                .expect("grow");
        }
        set
    }

    // Validates: Requirement 8.1 -- dataset report extents and tracks
    #[test]
    fn dataset_report_extents_and_tracks() {
        let g = geom();
        let mut vol = volume(1000);
        // Primary 2 tracks, grow past 2 tracks to add a 1-track secondary.
        let set = dataset_with_extents(&mut vol, 2, &[2 * 56664 + 1]);
        let report = DatasetSpaceReport::derive(&set, &g, DEFAULT_MAX_EXTENTS);
        assert_eq!(report.extents_used, 2);
        assert_eq!(report.extents_remaining, DEFAULT_MAX_EXTENTS - 2);
        assert_eq!(report.tracks_in_use, 3);
        assert_eq!(report.cylinders_in_use, g.cylinders(3));
    }

    // Validates: Requirement 8.2 -- volume report total/used/free
    #[test]
    fn volume_report_total_used_free() {
        let g = geom();
        let mut vol = volume(1000);
        let _set = dataset_with_extents(&mut vol, 10, &[]);
        let report = VolumeSpaceReport::derive(&vol, &g, 1);
        assert_eq!(report.total_tracks, 1000);
        assert_eq!(report.used_tracks, 10);
        assert_eq!(report.free_tracks, 990);
        assert_eq!(report.extents_allocated, 1);
    }

    // Validates: Requirement 8.3 -- VTOC view lists datasets and counters
    #[test]
    fn vtoc_view_lists_datasets_and_counters() {
        let g = geom();
        let mut vol = volume(1000);
        let set = dataset_with_extents(&mut vol, 5, &[]);
        let view = VtocView::derive(&vol, &g, &[(DatasetId(5), set)], DEFAULT_MAX_EXTENTS);
        assert_eq!(view.datasets.len(), 1);
        assert_eq!(view.datasets[0].0, DatasetId(5));
        assert_eq!(view.datasets[0].1.tracks_in_use, 5);
        assert_eq!(view.volume.used_tracks, 5);
        assert_eq!(view.volume.extents_allocated, 1);
    }

    // Validates: Requirement 8.4, 11.2 -- reporting is derived, not scanned
    #[test]
    fn reporting_is_derived_not_scanned() {
        let g = geom();
        let mut vol = volume(1000);
        // Single extent -> O(1) derivation path; counters come from metadata only.
        let set = dataset_with_extents(&mut vol, 3, &[]);
        let report = DatasetSpaceReport::derive(&set, &g, DEFAULT_MAX_EXTENTS);
        // tracks_in_use equals recorded allocated_tracks, not any byte scan.
        assert_eq!(report.tracks_in_use, set.allocated_tracks());
    }
}
