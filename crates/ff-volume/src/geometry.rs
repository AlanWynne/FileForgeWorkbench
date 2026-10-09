//! The emulated geometry profile (ADR-005, Requirement 3).
//!
//! Tracks and cylinders are ACCOUNTING units only -- no physical disk layout is
//! simulated. Conversions are deterministic integer `ceil` operations: identical
//! inputs under an identical profile always produce identical results
//! (Requirement 3.3, 11.1).

use serde::{Deserialize, Serialize};

// === Defaults (3390-style) ===================================================

/// Default bytes per emulated track (a 3390-style value, Requirement 3.1).
pub const DEFAULT_BYTES_PER_TRACK: u64 = 56664;

/// Default emulated tracks per cylinder (Requirement 3.1).
pub const DEFAULT_TRACKS_PER_CYLINDER: u64 = 15;

// === GeometryProfile ===================================================

/// A configurable byte<->track<->cylinder conversion profile (Requirement 3.1).
///
/// All conversions are deterministic and use integer ceil arithmetic; no
/// floating point is involved (Requirement 3.3). The profile constrains
/// accounting only, never physical on-disk layout (Requirement 3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryProfile {
    bytes_per_track: u64,
    tracks_per_cylinder: u64,
}

impl Default for GeometryProfile {
    fn default() -> Self {
        Self {
            bytes_per_track: DEFAULT_BYTES_PER_TRACK,
            tracks_per_cylinder: DEFAULT_TRACKS_PER_CYLINDER,
        }
    }
}

impl GeometryProfile {
    /// Creates a profile with explicit values. Both are clamped to a minimum of
    /// 1 so conversions never divide by zero.
    pub fn new(bytes_per_track: u64, tracks_per_cylinder: u64) -> Self {
        Self {
            bytes_per_track: bytes_per_track.max(1),
            tracks_per_cylinder: tracks_per_cylinder.max(1),
        }
    }

    /// Returns the bytes-per-track value.
    pub fn bytes_per_track(&self) -> u64 {
        self.bytes_per_track
    }

    /// Returns the tracks-per-cylinder value.
    pub fn tracks_per_cylinder(&self) -> u64 {
        self.tracks_per_cylinder
    }

    /// Converts bytes to whole tracks, rounding up (Requirement 3.2).
    pub fn tracks(&self, bytes: u64) -> u64 {
        ceil_div(bytes, self.bytes_per_track)
    }

    /// Converts tracks to whole cylinders, rounding up (Requirement 3.2).
    pub fn cylinders(&self, tracks: u64) -> u64 {
        ceil_div(tracks, self.tracks_per_cylinder)
    }

    /// Converts tracks to bytes (Requirement 3.2).
    pub fn bytes_of(&self, tracks: u64) -> u64 {
        tracks.saturating_mul(self.bytes_per_track)
    }
}

/// Integer ceiling division. `ceil_div(x, 0)` is defined as 0 (never panics);
/// callers hold `divisor >= 1` via `GeometryProfile::new`.
fn ceil_div(numerator: u64, divisor: u64) -> u64 {
    if divisor == 0 {
        return 0;
    }
    numerator.div_ceil(divisor)
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 3.1 -- defaults are the documented 3390-style values
    #[test]
    fn defaults_are_3390_style() {
        let g = GeometryProfile::default();
        assert_eq!(g.bytes_per_track(), 56664);
        assert_eq!(g.tracks_per_cylinder(), 15);
    }

    // Validates: Requirement 3.2 -- bytes to tracks ceils (worked example)
    #[test]
    fn bytes_to_tracks_ceils() {
        let g = GeometryProfile::default();
        // 1_000_000 / 56664 = 17.6..., ceil -> 18 tracks.
        assert_eq!(g.tracks(1_000_000), 18);
        // Exact multiple does not round up.
        assert_eq!(g.tracks(56664), 1);
        // One byte over a track boundary rounds up.
        assert_eq!(g.tracks(56665), 2);
        // Zero bytes is zero tracks.
        assert_eq!(g.tracks(0), 0);
    }

    // Validates: Requirement 3.2 -- tracks to cylinders ceils (worked example)
    #[test]
    fn tracks_to_cylinders_ceils() {
        let g = GeometryProfile::default();
        // 18 tracks / 15 = 1.2, ceil -> 2 cylinders.
        assert_eq!(g.cylinders(18), 2);
        assert_eq!(g.cylinders(15), 1);
        assert_eq!(g.cylinders(16), 2);
        assert_eq!(g.cylinders(0), 0);
    }

    // Validates: Requirement 3.3, 11.1 -- conversions are deterministic
    #[test]
    fn conversions_are_deterministic() {
        let g = GeometryProfile::default();
        for _ in 0..5 {
            assert_eq!(g.tracks(1_000_000), 18);
            assert_eq!(g.cylinders(18), 2);
            assert_eq!(g.bytes_of(18), 18 * 56664);
        }
    }

    // Validates: Requirement 3.2 -- bytes_of multiplies tracks by bytes_per_track
    #[test]
    fn bytes_of_multiplies_tracks() {
        let g = GeometryProfile::new(1000, 10);
        assert_eq!(g.bytes_of(3), 3000);
    }
}
