//! The SPACE allocation-unit model (Requirement 4).
//!
//! This is the UNIT MODEL that the JCL `SPACE=` forms map onto. The parsing of
//! the JCL SPACE keyword itself is owned by `ff-dsalloc` and consumes this model
//! (Requirement 4.4) -- there is deliberately NO string parser here.

use serde::{Deserialize, Serialize};

use crate::geometry::GeometryProfile;

// === UnitKind ===================================================

/// The unit a SPACE request is expressed in (Requirement 4.1-4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UnitKind {
    /// Tracks: `SPACE=(TRK,(primary,secondary))` (Requirement 4.1).
    Trk,
    /// Cylinders: `SPACE=(CYL,(primary,secondary))` (Requirement 4.2).
    Cyl,
    /// Average-record / block units with `AVGREC`:
    /// `SPACE=(avgreclen,(primary,secondary))` (Requirement 4.3). The quantity
    /// is a count of records of `avg_record_len` bytes each.
    AvgRec {
        /// The average record length in bytes.
        avg_record_len: u64,
    },
}

// === AllocationUnit ===================================================

/// A SPACE allocation request: a unit kind plus primary and secondary
/// quantities (Requirement 4). The secondary quantity is the per-extent size
/// used when the dataset grows (Requirement 4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationUnit {
    /// The unit the quantities are expressed in.
    pub kind: UnitKind,
    /// The primary allocation quantity.
    pub primary: u64,
    /// The secondary (per-extent growth) quantity.
    pub secondary: u64,
}

impl AllocationUnit {
    /// Creates a new allocation-unit request.
    pub fn new(kind: UnitKind, primary: u64, secondary: u64) -> Self {
        Self {
            kind,
            primary,
            secondary,
        }
    }

    /// Converts the primary quantity to tracks via the geometry profile
    /// (Requirement 4.1-4.3).
    pub fn primary_tracks(&self, geom: &GeometryProfile) -> u64 {
        quantity_to_tracks(self.kind, self.primary, geom)
    }

    /// Converts the secondary quantity to tracks via the geometry profile
    /// (Requirement 4.5).
    pub fn secondary_tracks(&self, geom: &GeometryProfile) -> u64 {
        quantity_to_tracks(self.kind, self.secondary, geom)
    }
}

/// Maps a quantity of `kind` units to tracks:
/// - `Trk` -> as-is;
/// - `Cyl` -> `quantity * tracks_per_cylinder`;
/// - `AvgRec` -> `geom.tracks(avg_record_len * quantity)`.
fn quantity_to_tracks(kind: UnitKind, quantity: u64, geom: &GeometryProfile) -> u64 {
    match kind {
        UnitKind::Trk => quantity,
        UnitKind::Cyl => quantity.saturating_mul(geom.tracks_per_cylinder()),
        UnitKind::AvgRec { avg_record_len } => {
            let bytes = avg_record_len.saturating_mul(quantity);
            geom.tracks(bytes)
        }
    }
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 4.1 -- TRK primary maps straight to tracks
    #[test]
    fn trk_primary_maps_to_tracks() {
        let g = GeometryProfile::default();
        let a = AllocationUnit::new(UnitKind::Trk, 10, 5);
        assert_eq!(a.primary_tracks(&g), 10);
    }

    // Validates: Requirement 4.2 -- CYL primary maps via tracks_per_cylinder
    #[test]
    fn cyl_primary_maps_via_tracks_per_cyl() {
        let g = GeometryProfile::default();
        let a = AllocationUnit::new(UnitKind::Cyl, 3, 1);
        // 3 cylinders * 15 tracks/cyl = 45 tracks.
        assert_eq!(a.primary_tracks(&g), 45);
    }

    // Validates: Requirement 4.3 -- AVGREC maps to units via geometry
    #[test]
    fn avgrec_maps_to_units_via_geometry() {
        let g = GeometryProfile::default();
        // 20000 records * 100 bytes = 2_000_000 bytes.
        // 2_000_000 / 56664 = 35.3..., ceil -> 36 tracks.
        let a = AllocationUnit::new(
            UnitKind::AvgRec {
                avg_record_len: 100,
            },
            20_000,
            1_000,
        );
        assert_eq!(a.primary_tracks(&g), 36);
    }

    // Validates: Requirement 4.5 -- secondary quantity recorded and converted
    #[test]
    fn secondary_quantity_recorded() {
        let g = GeometryProfile::default();
        let a = AllocationUnit::new(UnitKind::Cyl, 10, 2);
        assert_eq!(a.secondary, 2);
        // 2 cylinders * 15 = 30 tracks.
        assert_eq!(a.secondary_tracks(&g), 30);
    }
}
