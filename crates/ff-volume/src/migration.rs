//! The metadata-only migration seam (Requirement 11.3).
//!
//! When a Volume is defined over an existing Repository root, NO dataset bytes
//! move -- the migration constructs a `Volume` plus one `DatasetVolume` row per
//! dataset, deriving each locator from the legacy `storage_path`. These are the
//! pure functions `ff-dscatalog`'s v3->v4 migration calls; they perform no I/O.

use crate::dataset_volume::DatasetVolume;
use crate::volume::{DatasetId, Volser, Volume, VolumeId};

/// Builds a `Volume` over an existing Repository `root` with the given VOLSER
/// and capacity, moving NO bytes (Requirement 11.3). The Volume starts
/// Online/ReadWrite with no used tracks; the caller supplies the id.
///
/// # Errors
/// Propagates `Volser::try_new` validation (empty VOLSER).
pub fn volume_over_repository(
    volume_id: VolumeId,
    root: &str,
    volser: &str,
    capacity_tracks: u64,
) -> Result<Volume, crate::error::VolumeError> {
    let volser = Volser::try_new(volser)?;
    Ok(Volume::new(volume_id, volser, root, capacity_tracks))
}

/// Builds one `DatasetVolume` row per `(dataset_id, storage_path)` pair, using
/// the storage_path as the opaque locator, sequence 1, is_primary true
/// (Requirement 11.3). Pure metadata construction -- no byte movement.
pub fn seed_dataset_volumes(
    datasets: &[(DatasetId, String)],
    volume_id: VolumeId,
) -> Vec<DatasetVolume> {
    datasets
        .iter()
        .map(|(dataset_id, storage_path)| DatasetVolume {
            dataset_id: *dataset_id,
            volume_id,
            sequence_number: 1,
            is_primary: true,
            locator: storage_path.clone(),
        })
        .collect()
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 11.3 -- migration builds volume and datasetvolumes without bytes
    #[test]
    fn migration_builds_volume_and_datasetvolumes_without_bytes() {
        let vol =
            volume_over_repository(VolumeId(1), "/repo/root", "catvol", 10_000).expect("volume");
        assert_eq!(vol.volume_id(), VolumeId(1));
        assert_eq!(vol.volser().as_str(), "CATVOL");
        assert_eq!(vol.storage_uri(), "/repo/root");
        assert_eq!(vol.total_tracks(), 10_000);
        // No bytes moved -- nothing is used at migration time.
        assert_eq!(vol.used_tracks(), 0);

        let datasets = vec![
            (DatasetId(10), "obj/uuid-a".to_string()),
            (DatasetId(20), "obj/uuid-b".to_string()),
        ];
        let rows = seed_dataset_volumes(&datasets, VolumeId(1));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].dataset_id, DatasetId(10));
        assert_eq!(rows[0].locator, "obj/uuid-a");
        assert_eq!(rows[0].sequence_number, 1);
        assert!(rows[0].is_primary);
        assert_eq!(rows[1].locator, "obj/uuid-b");
    }

    // Validates: Requirement 11.3 -- empty dataset list yields no rows
    #[test]
    fn migration_empty_dataset_list_yields_no_rows() {
        let rows = seed_dataset_volumes(&[], VolumeId(1));
        assert!(rows.is_empty());
    }
}
