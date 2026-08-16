//! Configuration for the ASA preview subsystem.
//!
//! Defines the `AsaPreviewConfig` struct with all configurable fields
//! from the `[asa_preview]` section of the workbench configuration.

use crate::printer::PageOverflow;

/// Style for page separators in text export.
// Validates: Requirement 11.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportPageSeparator {
    /// `--- PAGE N ---` separator line.
    #[default]
    Dashes,
    /// ASCII form-feed character (0x0C).
    FormFeed,
}

/// Complete configuration for the ASA preview subsystem.
///
/// Parsed from the `[asa_preview]` section of the workbench configuration.
// Validates: Requirement 8.5, 12.1
#[derive(Debug, Clone, PartialEq)]
pub struct AsaPreviewConfig {
    /// Character columns per page (default 132).
    pub page_width: u16,
    /// Print lines per page (default 60).
    pub page_depth: u16,
    /// How to handle lines exceeding page width (default Truncate).
    pub page_overflow: PageOverflow,
    /// Number of lines per shading band (default 5).
    pub band_size: u8,
    /// Whether to show alternating line shading (default true).
    pub show_line_bands: bool,
    /// Whether to run ASA auto-detection on file open (default true).
    pub auto_detect: bool,
    /// Whether to automatically strip ASA column on file open (default false).
    pub auto_strip: bool,
    /// Detection confidence threshold (default 0.8).
    pub detection_threshold: f64,
    /// Number of lines to sample for detection (default 50).
    pub detection_sample_size: usize,
    /// Named printer profile (default "ibm-1403").
    pub printer_profile: String,
    /// Text export page break style (default Dashes).
    pub export_page_separator: ExportPageSeparator,
    /// Whether to insert implicit page breaks at page_depth intervals (default true).
    pub implicit_page_breaks: bool,
}

impl Default for AsaPreviewConfig {
    fn default() -> Self {
        Self {
            page_width: 132,
            page_depth: 60,
            page_overflow: PageOverflow::Truncate,
            band_size: 5,
            show_line_bands: true,
            auto_detect: true,
            auto_strip: false,
            detection_threshold: 0.8,
            detection_sample_size: 50,
            printer_profile: "ibm-1403".to_string(),
            export_page_separator: ExportPageSeparator::Dashes,
            implicit_page_breaks: true,
        }
    }
}

impl AsaPreviewConfig {
    /// Validate and clamp configuration values to acceptable ranges.
    ///
    /// Returns a list of warnings for values that were clamped or reset to defaults.
    pub fn validate(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        // page_width: 60–255
        if self.page_width < 60 || self.page_width > 255 {
            warnings.push(format!(
                "page_width {} out of range [60, 255], clamped",
                self.page_width
            ));
            self.page_width = self.page_width.clamp(60, 255);
        }

        // page_depth: 10–120
        if self.page_depth < 10 || self.page_depth > 120 {
            warnings.push(format!(
                "page_depth {} out of range [10, 120], clamped",
                self.page_depth
            ));
            self.page_depth = self.page_depth.clamp(10, 120);
        }

        // band_size: 1–20
        if self.band_size < 1 || self.band_size > 20 {
            warnings.push(format!(
                "band_size {} out of range [1, 20], clamped",
                self.band_size
            ));
            self.band_size = self.band_size.clamp(1, 20);
        }

        // detection_threshold: 0.5–1.0
        if self.detection_threshold < 0.5 || self.detection_threshold > 1.0 {
            warnings.push(format!(
                "detection_threshold {} out of range [0.5, 1.0], clamped",
                self.detection_threshold
            ));
            self.detection_threshold = self.detection_threshold.clamp(0.5, 1.0);
        }

        // detection_sample_size: 10–500
        if self.detection_sample_size < 10 || self.detection_sample_size > 500 {
            warnings.push(format!(
                "detection_sample_size {} out of range [10, 500], clamped",
                self.detection_sample_size
            ));
            self.detection_sample_size = self.detection_sample_size.clamp(10, 500);
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_standard_values() {
        let config = AsaPreviewConfig::default();
        assert_eq!(config.page_width, 132);
        assert_eq!(config.page_depth, 60);
        assert_eq!(config.page_overflow, PageOverflow::Truncate);
        assert_eq!(config.band_size, 5);
        assert!(config.show_line_bands);
        assert!(config.auto_detect);
        assert!(!config.auto_strip);
        assert_eq!(config.detection_threshold, 0.8);
        assert_eq!(config.detection_sample_size, 50);
        assert_eq!(config.printer_profile, "ibm-1403");
        assert_eq!(config.export_page_separator, ExportPageSeparator::Dashes);
        assert!(config.implicit_page_breaks);
    }

    #[test]
    fn validate_clamps_out_of_range_values() {
        let mut config = AsaPreviewConfig {
            page_width: 10,
            page_depth: 5,
            band_size: 0,
            detection_threshold: 0.2,
            detection_sample_size: 5,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.page_width, 60);
        assert_eq!(config.page_depth, 10);
        assert_eq!(config.band_size, 1);
        assert_eq!(config.detection_threshold, 0.5);
        assert_eq!(config.detection_sample_size, 10);
        assert_eq!(warnings.len(), 5);
    }

    #[test]
    fn validate_does_not_warn_for_valid_config() {
        let mut config = AsaPreviewConfig::default();
        let warnings = config.validate();
        assert!(warnings.is_empty());
    }
}
