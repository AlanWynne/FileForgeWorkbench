//! Visual mode (Dark / Light / High-Contrast) support.
//!
//! The workbench supports three appearance modes. Each mode provides a
//! distinct set of palette values optimised for its context.

use serde::{Deserialize, Serialize};

/// The four supported appearance modes.
///
/// The active mode determines which palette values are served to
/// rendering code. Mode switching takes effect within one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VisualMode {
    /// Dark backgrounds with light text (default).
    #[default]
    Dark,
    /// Light backgrounds with dark text.
    Light,
    /// Maximum contrast colours meeting WCAG AAA (7:1 ratio).
    HighContrast,
    /// IBM 3270 terminal legacy palette — black background with classic ISPF colours.
    Legacy,
}

impl VisualMode {
    /// Returns the TOML section name for this mode.
    pub fn section_name(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::HighContrast => "high_contrast",
            Self::Legacy => "legacy",
        }
    }

    /// Parse a mode from a string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            "high_contrast" | "high-contrast" | "highcontrast" => Some(Self::HighContrast),
            "legacy" => Some(Self::Legacy),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_dark() {
        // Validates: Requirement 5.1
        assert_eq!(VisualMode::default(), VisualMode::Dark);
    }

    #[test]
    fn section_name_matches_toml_convention() {
        // Validates: Requirement 5.2
        assert_eq!(VisualMode::Dark.section_name(), "dark");
        assert_eq!(VisualMode::Light.section_name(), "light");
        assert_eq!(VisualMode::HighContrast.section_name(), "high_contrast");
    }

    #[test]
    fn from_str_loose_parses_variants() {
        // Validates: Requirement 5.3
        assert_eq!(VisualMode::from_str_loose("dark"), Some(VisualMode::Dark));
        assert_eq!(VisualMode::from_str_loose("LIGHT"), Some(VisualMode::Light));
        assert_eq!(
            VisualMode::from_str_loose("high_contrast"),
            Some(VisualMode::HighContrast)
        );
        assert_eq!(
            VisualMode::from_str_loose("high-contrast"),
            Some(VisualMode::HighContrast)
        );
        assert_eq!(
            VisualMode::from_str_loose("legacy"),
            Some(VisualMode::Legacy)
        );
        assert_eq!(VisualMode::from_str_loose("invalid"), None);
    }
}
