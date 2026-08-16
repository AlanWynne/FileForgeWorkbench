//! Printer profiles and page dimension configuration.
//!
//! Named printer profiles bundle page dimensions and overflow behaviour
//! to emulate specific line printer hardware (IBM 1403, 3800, 4245).

use crate::types::{PageDepth, PageWidth};

/// Behaviour for lines that exceed the configured page width.
// Validates: Requirement 8.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageOverflow {
    /// Truncate lines at page width boundary.
    #[default]
    Truncate,
    /// Soft-wrap lines that exceed page width.
    Wrap,
}

/// Named printer profile bundling page dimensions and behaviour.
// Validates: Requirement 8.5, 8.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterProfile {
    /// Profile name (e.g., "ibm-1403", "ibm-3800", "custom").
    pub name: String,
    /// Character columns per page.
    pub page_width: PageWidth,
    /// Print lines per page.
    pub page_depth: PageDepth,
    /// How to handle lines exceeding page width.
    pub page_overflow: PageOverflow,
    /// Human-readable description.
    pub description: String,
}

impl PrinterProfile {
    /// IBM 1403 standard: 132×60.
    pub fn ibm_1403() -> Self {
        Self {
            name: "ibm-1403".to_string(),
            page_width: PageWidth(132),
            page_depth: PageDepth(60),
            page_overflow: PageOverflow::Truncate,
            description: "IBM 1403 Impact Printer — 132 columns × 60 lines".to_string(),
        }
    }

    /// IBM 3800 laser: 132×60.
    pub fn ibm_3800() -> Self {
        Self {
            name: "ibm-3800".to_string(),
            page_width: PageWidth(132),
            page_depth: PageDepth(60),
            page_overflow: PageOverflow::Truncate,
            description: "IBM 3800 Laser Printer — 132 columns × 60 lines".to_string(),
        }
    }

    /// IBM 4245 printer: 132×66.
    pub fn ibm_4245() -> Self {
        Self {
            name: "ibm-4245".to_string(),
            page_width: PageWidth(132),
            page_depth: PageDepth(66),
            page_overflow: PageOverflow::Truncate,
            description: "IBM 4245 Line Printer — 132 columns × 66 lines".to_string(),
        }
    }

    /// Custom profile with user-specified dimensions.
    pub fn custom(page_width: u16, page_depth: u16, overflow: PageOverflow) -> Self {
        Self {
            name: "custom".to_string(),
            page_width: PageWidth::new(page_width),
            page_depth: PageDepth::new(page_depth),
            page_overflow: overflow,
            description: format!(
                "Custom — {} columns × {} lines",
                PageWidth::new(page_width).0,
                PageDepth::new(page_depth).0
            ),
        }
    }

    /// Look up a profile by name. Returns None for unknown names.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ibm-1403" => Some(Self::ibm_1403()),
            "ibm-3800" => Some(Self::ibm_3800()),
            "ibm-4245" => Some(Self::ibm_4245()),
            _ => None,
        }
    }

    /// Check whether page dimensions differ from the default (132×60).
    pub fn has_custom_dimensions(&self) -> bool {
        self.page_width.0 != 132 || self.page_depth.0 != 60
    }

    /// Format a dimensions annotation string (e.g., "132×66").
    pub fn dimensions_annotation(&self) -> String {
        format!("{}×{}", self.page_width.0, self.page_depth.0)
    }
}

impl Default for PrinterProfile {
    fn default() -> Self {
        Self::ibm_1403()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 8.6
    fn built_in_profiles_have_correct_dimensions() {
        let ibm1403 = PrinterProfile::ibm_1403();
        assert_eq!(ibm1403.page_width.0, 132);
        assert_eq!(ibm1403.page_depth.0, 60);

        let ibm3800 = PrinterProfile::ibm_3800();
        assert_eq!(ibm3800.page_width.0, 132);
        assert_eq!(ibm3800.page_depth.0, 60);

        let ibm4245 = PrinterProfile::ibm_4245();
        assert_eq!(ibm4245.page_width.0, 132);
        assert_eq!(ibm4245.page_depth.0, 66);
    }

    #[test]
    // Validates: Requirement 8.6
    fn from_name_resolves_known_profiles() {
        assert!(PrinterProfile::from_name("ibm-1403").is_some());
        assert!(PrinterProfile::from_name("ibm-3800").is_some());
        assert!(PrinterProfile::from_name("ibm-4245").is_some());
        assert!(PrinterProfile::from_name("unknown").is_none());
    }

    #[test]
    fn custom_profile_clamps_dimensions() {
        let profile = PrinterProfile::custom(10, 5, PageOverflow::Wrap);
        assert_eq!(profile.page_width.0, 60); // clamped to min
        assert_eq!(profile.page_depth.0, 10); // clamped to min
        assert_eq!(profile.page_overflow, PageOverflow::Wrap);
    }

    #[test]
    fn has_custom_dimensions_detects_non_default() {
        assert!(!PrinterProfile::ibm_1403().has_custom_dimensions());
        assert!(PrinterProfile::ibm_4245().has_custom_dimensions());
    }

    #[test]
    fn dimensions_annotation_format() {
        assert_eq!(PrinterProfile::ibm_1403().dimensions_annotation(), "132×60");
        assert_eq!(PrinterProfile::ibm_4245().dimensions_annotation(), "132×66");
    }
}
