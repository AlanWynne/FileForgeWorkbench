//! Font configuration: font stacks, sizes, zoom, and resolution.
//!
//! The theme system manages separate monospace and proportional font stacks.
//! Each stack defines an ordered list of font family names with automatic
//! fallback, a base size in points, and a zoom level offset for the editor.

use serde::{Deserialize, Serialize};

/// Default base size for the monospace (editor) font stack.
pub const DEFAULT_MONOSPACE_SIZE_PT: f32 = 14.0;
/// Default base size for the proportional (UI) font stack.
pub const DEFAULT_PROPORTIONAL_SIZE_PT: f32 = 13.0;

/// Minimum valid font size in points (configuration validation).
pub const MIN_FONT_SIZE_PT: f32 = 6.0;
/// Maximum valid font size in points (configuration validation).
pub const MAX_FONT_SIZE_PT: f32 = 72.0;

/// Minimum effective font size in points (after zoom applied).
pub const MIN_EFFECTIVE_SIZE_PT: f32 = 2.0;
/// Maximum effective font size in points (after zoom applied).
pub const MAX_EFFECTIVE_SIZE_PT: f32 = 128.0;

/// An ordered font family list with base size.
///
/// The first available family in the list is used for rendering. If no
/// family in the list is available, the platform default is used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontStack {
    /// Ordered list of font family names (first available is used).
    pub families: Vec<String>,
    /// Base font size in points.
    pub base_size_pt: f32,
}

impl FontStack {
    /// Create a new font stack with the given families and base size.
    ///
    /// The base size is clamped to the valid range [6.0, 72.0].
    pub fn new(families: Vec<String>, base_size_pt: f32) -> Self {
        Self {
            families,
            base_size_pt: clamp_font_size(base_size_pt),
        }
    }

    /// Validate and clamp the base size to the allowed range.
    ///
    /// Returns `true` if the size was already within range, `false` if it was clamped.
    pub fn validate_size(&mut self) -> bool {
        let clamped = clamp_font_size(self.base_size_pt);
        let was_valid = (clamped - self.base_size_pt).abs() < f32::EPSILON;
        self.base_size_pt = clamped;
        was_valid
    }
}

/// Font configuration for both editor and UI contexts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontConfig {
    /// Monospace font stack for editor content.
    pub monospace: FontStack,
    /// Proportional font stack for UI elements.
    pub proportional: FontStack,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            monospace: FontStack {
                families: Vec::new(),
                base_size_pt: DEFAULT_MONOSPACE_SIZE_PT,
            },
            proportional: FontStack {
                families: Vec::new(),
                base_size_pt: DEFAULT_PROPORTIONAL_SIZE_PT,
            },
        }
    }
}

/// Zoom level for the editor monospace font.
///
/// The zoom level is an integer offset added to the base font size.
/// The effective size is clamped to [2.0, 128.0] without modifying
/// the stored zoom level value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZoomLevel {
    level: i32,
}

impl ZoomLevel {
    /// Create a new zoom level.
    pub const fn new(level: i32) -> Self {
        Self { level }
    }

    /// Get the current zoom level value.
    pub const fn level(&self) -> i32 {
        self.level
    }

    /// Set the zoom level value.
    pub fn set_level(&mut self, level: i32) {
        self.level = level;
    }

    /// Calculate the effective font size given a base size and this zoom level.
    ///
    /// The result is clamped to [2.0, 128.0] without modifying the zoom level.
    pub fn effective_size(&self, base_size_pt: f32) -> f32 {
        let effective = base_size_pt + self.level as f32;
        effective.clamp(MIN_EFFECTIVE_SIZE_PT, MAX_EFFECTIVE_SIZE_PT)
    }
}

/// Clamp a font size to the valid configuration range [6.0, 72.0].
pub fn clamp_font_size(size: f32) -> f32 {
    size.clamp(MIN_FONT_SIZE_PT, MAX_FONT_SIZE_PT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_font_config_has_correct_sizes() {
        // Validates: Requirement 4.5
        let config = FontConfig::default();
        assert_eq!(config.monospace.base_size_pt, 14.0);
        assert_eq!(config.proportional.base_size_pt, 13.0);
    }

    #[test]
    fn font_stack_new_clamps_size() {
        // Validates: Requirement 4.6
        let stack = FontStack::new(vec![], 3.0);
        assert_eq!(stack.base_size_pt, 6.0);

        let stack = FontStack::new(vec![], 100.0);
        assert_eq!(stack.base_size_pt, 72.0);

        let stack = FontStack::new(vec![], 14.0);
        assert_eq!(stack.base_size_pt, 14.0);
    }

    #[test]
    fn validate_size_reports_clamping() {
        // Validates: Requirement 4.6
        let mut stack = FontStack {
            families: vec![],
            base_size_pt: 3.0,
        };
        let was_valid = stack.validate_size();
        assert!(!was_valid);
        assert_eq!(stack.base_size_pt, 6.0);
    }

    #[test]
    fn zoom_level_effective_size_calculation() {
        // Validates: Requirement 4.7
        let zoom = ZoomLevel::new(5);
        assert_eq!(zoom.effective_size(14.0), 19.0);
    }

    #[test]
    fn zoom_level_effective_size_clamped_low() {
        // Validates: Requirement 4.8
        let zoom = ZoomLevel::new(-100);
        assert_eq!(zoom.effective_size(14.0), 2.0);
    }

    #[test]
    fn zoom_level_effective_size_clamped_high() {
        // Validates: Requirement 4.8
        let zoom = ZoomLevel::new(200);
        assert_eq!(zoom.effective_size(14.0), 128.0);
    }

    #[test]
    fn zoom_level_not_modified_by_clamping() {
        // Validates: Requirement 4.8
        let zoom = ZoomLevel::new(-100);
        let _ = zoom.effective_size(14.0);
        assert_eq!(zoom.level(), -100);
    }

    #[test]
    fn empty_font_stack_is_valid() {
        // Validates: Requirement 4.3
        let config = FontConfig::default();
        assert!(config.monospace.families.is_empty());
        assert!(config.proportional.families.is_empty());
    }
}
