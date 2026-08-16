//! Splitter data types — draggable borders between dock zones and tab groups.

/// Opaque identifier for a splitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SplitterId(pub(crate) u32);

impl SplitterId {
    /// Creates a new splitter ID from a raw value.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw numeric value.
    pub fn value(self) -> u32 {
        self.0
    }
}

/// Orientation of a splitter handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitterOrientation {
    /// Horizontal splitter (divides top/bottom areas).
    Horizontal,
    /// Vertical splitter (divides left/right areas).
    Vertical,
}

/// A draggable border between adjacent dock zones or tab groups.
///
/// Stores the current proportional position and enforces minimum size
/// constraints on both adjacent areas.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Splitter {
    /// Unique identifier for this splitter.
    pub id: SplitterId,
    /// Current proportional position [0.0, 1.0].
    pub proportion: f32,
    /// Default proportional position (for double-click reset).
    pub default_proportion: f32,
    /// Orientation of the splitter.
    pub orientation: SplitterOrientation,
    /// Minimum size constraint for the first (left/top) area in logical pixels.
    pub min_first: f32,
    /// Minimum size constraint for the second (right/bottom) area in logical pixels.
    pub min_second: f32,
}

impl Splitter {
    /// Creates a new splitter with the given parameters.
    pub fn new(
        id: SplitterId,
        default_proportion: f32,
        orientation: SplitterOrientation,
        min_first: f32,
        min_second: f32,
    ) -> Self {
        Self {
            id,
            proportion: default_proportion,
            default_proportion,
            orientation,
            min_first,
            min_second,
        }
    }

    /// Clamps a proportion value to respect minimum size constraints.
    ///
    /// Given a total available size, ensures neither side goes below its minimum.
    /// If the total size is insufficient for both minimums, returns the current
    /// proportion unchanged.
    pub fn clamp_proportion(&self, target: f32, total_size: f32) -> f32 {
        if total_size <= 0.0 {
            return self.proportion;
        }
        let min_first_prop = self.min_first / total_size;
        let max_prop = 1.0 - (self.min_second / total_size);

        // If minimums exceed total size, we cannot satisfy both constraints
        if min_first_prop > max_prop {
            return self.proportion;
        }

        target.clamp(min_first_prop, max_prop)
    }

    /// Resets this splitter to its default proportion.
    pub fn reset_to_default(&mut self) {
        self.proportion = self.default_proportion;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_MIN_PANEL_SIZE;

    #[test]
    fn splitter_new_sets_proportion_to_default() {
        let splitter = Splitter::new(
            SplitterId(1),
            0.3,
            SplitterOrientation::Vertical,
            DEFAULT_MIN_PANEL_SIZE,
            DEFAULT_MIN_PANEL_SIZE,
        );
        assert_eq!(splitter.proportion, 0.3);
        assert_eq!(splitter.default_proportion, 0.3);
    }

    #[test]
    fn splitter_clamp_proportion_respects_minimums() {
        let splitter = Splitter::new(
            SplitterId(1),
            0.5,
            SplitterOrientation::Vertical,
            100.0, // min_first
            100.0, // min_second
        );
        // Total size 1000: min_first_prop = 0.1, max_prop = 0.9
        assert_eq!(splitter.clamp_proportion(0.5, 1000.0), 0.5);
        assert_eq!(splitter.clamp_proportion(0.05, 1000.0), 0.1);
        assert_eq!(splitter.clamp_proportion(0.95, 1000.0), 0.9);
    }

    #[test]
    fn splitter_clamp_proportion_handles_zero_total() {
        let splitter = Splitter::new(
            SplitterId(1),
            0.5,
            SplitterOrientation::Vertical,
            48.0,
            48.0,
        );
        // With zero total, returns current proportion
        assert_eq!(splitter.clamp_proportion(0.8, 0.0), 0.5);
    }

    #[test]
    fn splitter_reset_to_default() {
        let mut splitter = Splitter::new(
            SplitterId(1),
            0.3,
            SplitterOrientation::Horizontal,
            48.0,
            48.0,
        );
        splitter.proportion = 0.7;
        splitter.reset_to_default();
        assert_eq!(splitter.proportion, 0.3);
    }
}
