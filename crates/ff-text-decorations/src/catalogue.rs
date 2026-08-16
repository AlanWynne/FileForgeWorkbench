//! Indicator catalogue — registry of style configurations for all 44 slots.

use crate::indicator::{IndicatorConfig, IndicatorFlags, StyleAppearance};
use crate::indicator_style::IndicatorStyle;
use crate::theme_integration::ThemeDecorationProvider;
use crate::{ColourRGBA, IndicatorNumber};

/// Registry of indicator style configurations for all 44 slots.
///
/// Addresses: Requirements 1, 2, 15
pub struct IndicatorCatalogue {
    /// Configuration for each indicator number (0–43).
    configs: Vec<IndicatorConfig>,
}

impl IndicatorCatalogue {
    /// Create catalogue with compiled default configurations.
    pub fn new() -> Self {
        let mut configs = Vec::with_capacity(44);
        for _ in 0..44 {
            configs.push(IndicatorConfig::default());
        }

        let mut catalogue = Self { configs };
        catalogue.apply_well_known_defaults();
        catalogue
    }

    /// Get the configuration for an indicator.
    pub fn get(&self, indicator: IndicatorNumber) -> &IndicatorConfig {
        &self.configs[indicator.0 as usize]
    }

    /// Update an indicator's configuration (typically from theme reload).
    pub fn set(&mut self, indicator: IndicatorNumber, config: IndicatorConfig) {
        self.configs[indicator.0 as usize] = config;
    }

    /// Check if an indicator is dynamic (has hover state).
    ///
    /// Addresses: Requirement 2 AC 7
    pub fn is_dynamic(&self, indicator: IndicatorNumber) -> bool {
        self.configs[indicator.0 as usize].is_dynamic()
    }

    /// Reload all configurations from theme palette.
    ///
    /// Validates values (alpha 0–255, stroke_width 0.5–10.0),
    /// falling back to defaults for invalid entries.
    ///
    /// Addresses: Requirement 15 AC 3
    pub fn reload_from_theme(&mut self, theme: &dyn ThemeDecorationProvider) {
        for i in 0..44 {
            let indicator = IndicatorNumber(i as u8);

            if let Some(fore) = theme.indicator_fore(indicator) {
                self.configs[i].normal.fore = fore;
                self.configs[i].hover.fore = fore;
            }

            if let Some(alpha) = theme.indicator_fill_alpha(indicator) {
                self.configs[i].fill_alpha = alpha;
            }

            if let Some(alpha) = theme.indicator_outline_alpha(indicator) {
                self.configs[i].outline_alpha = alpha;
            }

            if let Some(width) = theme.indicator_stroke_width(indicator) {
                if (0.5..=10.0).contains(&width) {
                    self.configs[i].stroke_width = width;
                }
                // Invalid values are silently ignored (fallback to current value)
            }

            if let Some(style) = theme.indicator_style(indicator) {
                self.configs[i].normal.style = style;
                self.configs[i].hover.style = style;
            }
        }
    }

    /// Apply well-known default configurations for search, diagnostic, and other indicators.
    fn apply_well_known_defaults(&mut self) {
        use crate::constants::indicators::*;

        // Search indicators
        let search_current_appearance = StyleAppearance {
            style: IndicatorStyle::StraightBox,
            fore: ColourRGBA::new(255, 165, 0), // bright orange
        };
        self.configs[SEARCH_CURRENT.0 as usize] = IndicatorConfig {
            normal: search_current_appearance,
            hover: search_current_appearance,
            under: false,
            fill_alpha: 100,
            outline_alpha: 255,
            stroke_width: 1.0,
            flags: IndicatorFlags::default(),
        };

        let search_all_appearance = StyleAppearance {
            style: IndicatorStyle::RoundBox,
            fore: ColourRGBA::new(255, 255, 100), // pale yellow
        };
        self.configs[SEARCH_ALL.0 as usize] = IndicatorConfig {
            normal: search_all_appearance,
            hover: search_all_appearance,
            under: false,
            fill_alpha: 60,
            outline_alpha: 100,
            stroke_width: 1.0,
            flags: IndicatorFlags::default(),
        };

        // Diagnostic indicators
        let error_appearance = StyleAppearance {
            style: IndicatorStyle::Squiggle,
            fore: ColourRGBA::new(255, 0, 0), // red
        };
        self.configs[ERROR.0 as usize] = IndicatorConfig {
            normal: error_appearance,
            hover: error_appearance,
            under: true,
            fill_alpha: 30,
            outline_alpha: 50,
            stroke_width: 1.0,
            flags: IndicatorFlags::default(),
        };

        let warning_appearance = StyleAppearance {
            style: IndicatorStyle::Squiggle,
            fore: ColourRGBA::new(255, 191, 0), // amber
        };
        self.configs[WARNING.0 as usize] = IndicatorConfig {
            normal: warning_appearance,
            hover: warning_appearance,
            under: true,
            fill_alpha: 30,
            outline_alpha: 50,
            stroke_width: 1.0,
            flags: IndicatorFlags::default(),
        };

        let info_appearance = StyleAppearance {
            style: IndicatorStyle::Plain,
            fore: ColourRGBA::new(0, 100, 255), // blue
        };
        self.configs[INFO.0 as usize] = IndicatorConfig {
            normal: info_appearance,
            hover: info_appearance,
            under: true,
            fill_alpha: 30,
            outline_alpha: 50,
            stroke_width: 1.0,
            flags: IndicatorFlags::default(),
        };

        let hint_appearance = StyleAppearance {
            style: IndicatorStyle::Dots,
            fore: ColourRGBA::new(128, 128, 128), // grey
        };
        self.configs[HINT.0 as usize] = IndicatorConfig {
            normal: hint_appearance,
            hover: hint_appearance,
            under: true,
            fill_alpha: 30,
            outline_alpha: 50,
            stroke_width: 1.0,
            flags: IndicatorFlags::default(),
        };
    }
}

impl Default for IndicatorCatalogue {
    fn default() -> Self {
        Self::new()
    }
}
