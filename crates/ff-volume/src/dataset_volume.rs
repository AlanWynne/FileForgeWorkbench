//! The DatasetVolume association and multivolume / uncataloged resolution
//! (Requirement 9).
//!
//! `DatasetVolume` replaces a dataset's raw `storage_path` with an ordered
//! association to one or more Volumes, each carrying a sequence number, an
//! is-primary flag, and an opaque per-volume locator. The registry is
//! catalog-agnostic, so datasets registered in different catalogs may share one
//! Volume (Requirement 9.3) and a dataset may span several Volumes
//! (Requirement 9.2).

use serde::{Deserialize, Serialize};

use crate::error::VolumeError;
use crate::volume::{DatasetId, Volser, Volume, VolumeId, VolumeRegistry};

// === DatasetVolume ===================================================

/// The association from a Dataset to one Volume (Requirement 9.1). A
/// single-volume dataset has exactly one of these; a multivolume dataset has an
/// ordered sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetVolume {
    /// The dataset.
    pub dataset_id: DatasetId,
    /// The Volume the dataset resides on for this sequence position.
    pub volume_id: VolumeId,
    /// 1-based position in the dataset's volume sequence.
    pub sequence_number: u32,
    /// Whether this is the primary Volume of the dataset.
    pub is_primary: bool,
    /// Opaque per-volume locator (replaces the raw storage_path).
    pub locator: String,
}

// === ResolvedVolume ===================================================

/// A resolved (Volume, locator) pair in sequence order (Requirement 9.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVolume {
    /// 1-based sequence position.
    pub sequence_number: u32,
    /// The resolved Volume.
    pub volume: Volume,
    /// The opaque per-volume locator for this position.
    pub locator: String,
}

// === DatasetVolumeSet ===================================================

/// The ordered set of DatasetVolume rows for one dataset (Requirement 9.2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetVolumeSet {
    rows: Vec<DatasetVolume>,
}

impl DatasetVolumeSet {
    /// Creates an empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a DatasetVolume row and keeps the set ordered by sequence number.
    pub fn add(&mut self, row: DatasetVolume) {
        self.rows.push(row);
        self.rows.sort_by_key(|r| r.sequence_number);
    }

    /// Returns the rows in sequence order.
    pub fn rows(&self) -> &[DatasetVolume] {
        &self.rows
    }

    /// Returns the number of Volumes the dataset spans.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns `true` when the dataset has no Volume association.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Resolves the dataset's Volumes in sequence order against `registry`,
    /// verifying each is Online (Requirement 9.2, honouring Requirement 2.4).
    ///
    /// # Errors
    /// `VolumeError::VolumeNotFound` if a referenced Volume is missing, or
    /// `VolumeError::VolumeOffline` for the first Offline Volume in sequence.
    pub fn resolve(&self, registry: &VolumeRegistry) -> Result<Vec<ResolvedVolume>, VolumeError> {
        let mut resolved = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            let volume =
                registry
                    .find_by_id(row.volume_id)
                    .ok_or_else(|| VolumeError::VolumeNotFound {
                        reference: format!("volume_id {}", row.volume_id.0),
                    })?;
            volume.ensure_allocatable()?;
            resolved.push(ResolvedVolume {
                sequence_number: row.sequence_number,
                volume: volume.clone(),
                locator: row.locator.clone(),
            });
        }
        Ok(resolved)
    }
}

// === Uncataloged resolution (Requirement 9.4, 9.5) ===================================================

/// Resolves an uncataloged dataset by explicit VOL=SER plus UNIT, with no
/// catalog row (Requirement 9.4, 9.5). `unit` is the device/UNIT name; it is
/// accepted and echoed into the locator so callers can trace the request, but
/// the model does not simulate device types.
///
/// # Errors
/// `VolumeError::VolumeNotFound` when no Volume matches `volser`, or
/// `VolumeError::VolumeOffline` when the matching Volume is Offline.
pub fn resolve_uncataloged(
    volser: &Volser,
    unit: &str,
    registry: &VolumeRegistry,
) -> Result<ResolvedVolume, VolumeError> {
    let volume = registry
        .find_by_volser(volser)
        .ok_or_else(|| VolumeError::VolumeNotFound {
            reference: format!("VOL=SER {volser} UNIT={unit}"),
        })?;
    volume.ensure_allocatable()?;
    Ok(ResolvedVolume {
        sequence_number: 1,
        volume: volume.clone(),
        locator: format!("uncat::{volser}::{unit}"),
    })
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn vol(id: u64, serial: &str) -> Volume {
        Volume::new(
            VolumeId(id),
            Volser::try_new(serial).expect("volser"),
            format!("/vol/{serial}"),
            1000,
        )
    }

    fn dv(dataset: u64, volume: u64, seq: u32, primary: bool, locator: &str) -> DatasetVolume {
        DatasetVolume {
            dataset_id: DatasetId(dataset),
            volume_id: VolumeId(volume),
            sequence_number: seq,
            is_primary: primary,
            locator: locator.to_string(),
        }
    }

    // Validates: Requirement 9.1 -- DatasetVolume carries locator replacing storage_path
    #[test]
    fn datasetvolume_replaces_storage_path() {
        let row = dv(1, 1, 1, true, "obj::uuid-123");
        assert_eq!(row.locator, "obj::uuid-123");
        assert!(row.is_primary);
        assert_eq!(row.sequence_number, 1);
    }

    // Validates: Requirement 9.2 -- multivolume ordered sequence resolves in order
    #[test]
    fn multivolume_ordered_sequence() {
        let mut registry = VolumeRegistry::new();
        registry.define(vol(1, "VOL001")).expect("v1");
        registry.define(vol(2, "VOL002")).expect("v2");
        let mut set = DatasetVolumeSet::new();
        // Add out of order to prove sorting.
        set.add(dv(7, 2, 2, false, "loc-b"));
        set.add(dv(7, 1, 1, true, "loc-a"));
        let resolved = set.resolve(&registry).expect("resolve");
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].sequence_number, 1);
        assert_eq!(resolved[0].locator, "loc-a");
        assert_eq!(resolved[1].sequence_number, 2);
        assert_eq!(resolved[1].locator, "loc-b");
    }

    // Validates: Requirement 9.3 -- two datasets in different catalogs share one volume
    #[test]
    fn shared_volume_across_catalogs() {
        let mut registry = VolumeRegistry::new();
        registry.define(vol(1, "SHARED")).expect("shared");
        // Dataset from "catalog A" and dataset from "catalog B" both on VOL id 1.
        let mut a = DatasetVolumeSet::new();
        a.add(dv(10, 1, 1, true, "a-loc"));
        let mut b = DatasetVolumeSet::new();
        b.add(dv(20, 1, 1, true, "b-loc"));
        let ra = a.resolve(&registry).expect("a");
        let rb = b.resolve(&registry).expect("b");
        assert_eq!(ra[0].volume.volume_id(), VolumeId(1));
        assert_eq!(rb[0].volume.volume_id(), VolumeId(1));
    }

    // Validates: Requirement 2.4 (via 9.2) -- resolve reports first offline volume
    #[test]
    fn resolve_reports_first_offline_volume() {
        let mut registry = VolumeRegistry::new();
        registry.define(vol(1, "VOL001")).expect("v1");
        let mut offline = vol(2, "VOL002");
        offline.set_offline();
        registry.define(offline).expect("v2");
        let mut set = DatasetVolumeSet::new();
        set.add(dv(7, 1, 1, true, "a"));
        set.add(dv(7, 2, 2, false, "b"));
        let err = set.resolve(&registry).expect_err("offline");
        assert_eq!(
            err,
            VolumeError::VolumeOffline {
                volser: "VOL002".to_string()
            }
        );
    }

    // Validates: Requirement 9.4, 9.5 -- uncataloged resolves by VOL=SER + UNIT
    #[test]
    fn uncataloged_resolves_by_volser_unit() {
        let mut registry = VolumeRegistry::new();
        registry.define(vol(1, "WORK01")).expect("v1");
        let volser = Volser::try_new("WORK01").expect("volser");
        let resolved = resolve_uncataloged(&volser, "3390", &registry).expect("resolve");
        assert_eq!(resolved.volume.volume_id(), VolumeId(1));
        assert_eq!(resolved.locator, "uncat::WORK01::3390");
    }

    // Validates: Requirement 9.5 -- unknown VOL=SER reports not found
    #[test]
    fn uncataloged_unknown_volser_reports_not_found() {
        let registry = VolumeRegistry::new();
        let volser = Volser::try_new("NONE01").expect("volser");
        let err = resolve_uncataloged(&volser, "3390", &registry).expect_err("missing");
        assert!(matches!(err, VolumeError::VolumeNotFound { .. }));
    }
}
