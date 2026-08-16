//! Configuration aggregate for caret and selection.
//!
//! Composes all individual configs into a single `CaretSelectionConfig`
//! and provides theme integration methods.

use crate::blink::BlinkState;
use crate::caret_colour::CaretColours;
use crate::caret_line::CaretLineConfig;
use crate::caret_style::CaretShape;
use crate::modified_marker::ModifiedMarkerConfig;
use crate::selection_colours::SelectionColourSet;
use crate::selection_display::SelectionDisplayConfig;

/// Aggregate configuration for all caret and selection visual settings.
///
/// GUI-independent: stores pure data with no rendering framework types.
///
/// Addresses: Requirement 11, criteria 11.1–11.5
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CaretSelectionConfig {
    /// Caret shape configuration (style, width, overstrike override).
    pub shape: CaretShape,
    /// Caret colour configuration (primary, additional).
    pub colours: CaretColours,
    /// Blink state (period, last reset).
    pub blink: BlinkState,
    /// Caret-line highlight configuration.
    pub caret_line: CaretLineConfig,
    /// Selection display configuration (visibility, layer, EOL fill).
    pub selection_display: SelectionDisplayConfig,
    /// Selection colour set (all context colours).
    pub selection_colours: SelectionColourSet,
    /// Modified line marker configuration.
    pub modified_marker: ModifiedMarkerConfig,
}

impl CaretSelectionConfig {
    /// Creates a new config with all defaults.
    ///
    /// Addresses: Requirement 11, criterion 11.3
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies a theme update, re-reading all settings.
    ///
    /// In this implementation without `ff-theme`, this resets to defaults.
    /// When `ff-theme` is available, this will read from the theme handle.
    ///
    /// Addresses: Requirement 11, criterion 11.2
    pub fn apply_defaults(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_constructs_without_panic() {
        // Validates: Requirement 11.3
        let config = CaretSelectionConfig::new();
        assert_eq!(config.shape, CaretShape::default());
        assert_eq!(config.colours, CaretColours::default());
        assert_eq!(config.blink, BlinkState::default());
        assert_eq!(config.caret_line, CaretLineConfig::default());
        assert_eq!(config.selection_display, SelectionDisplayConfig::default());
        assert_eq!(config.selection_colours, SelectionColourSet::default());
        assert_eq!(config.modified_marker, ModifiedMarkerConfig::default());
    }

    #[test]
    fn apply_defaults_resets_to_defaults() {
        // Validates: Requirement 11.2
        let mut config = CaretSelectionConfig::new();
        config.blink.set_period(1000);
        config.apply_defaults();
        assert_eq!(config.blink.period_ms(), 530);
    }

    #[test]
    fn config_fields_are_modifiable() {
        // Validates: Requirement 11.5
        let mut config = CaretSelectionConfig::new();
        config.blink.set_period(800);
        assert_eq!(config.blink.period_ms(), 800);
    }
}
