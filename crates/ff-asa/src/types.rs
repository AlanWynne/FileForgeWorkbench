//! Core data types and newtypes for the ASA report preview subsystem.

use std::fmt;

/// A 1-based page number in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageNumber(pub u32);

impl PageNumber {
    /// Create a new page number. Must be >= 1.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0. Use `try_new` for fallible construction.
    pub fn new(n: u32) -> Self {
        assert!(n > 0, "PageNumber must be >= 1");
        Self(n)
    }

    /// Try to create a page number. Returns None if `n` is 0.
    pub fn try_new(n: u32) -> Option<Self> {
        if n > 0 {
            Some(Self(n))
        } else {
            None
        }
    }

    /// Get the underlying value.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for PageNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for PageNumber {
    fn from(n: u32) -> Self {
        Self(n.max(1))
    }
}

/// Page depth (number of print lines per page).
///
/// Valid range: 10–120. Default: 60.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageDepth(pub u16);

impl PageDepth {
    /// Minimum allowed page depth.
    pub const MIN: u16 = 10;
    /// Maximum allowed page depth.
    pub const MAX: u16 = 120;
    /// Default page depth (IBM 1403 standard).
    pub const DEFAULT: u16 = 60;

    /// Create a page depth, clamping to the valid range.
    pub fn new(depth: u16) -> Self {
        Self(depth.clamp(Self::MIN, Self::MAX))
    }

    /// Get the underlying value.
    pub fn get(self) -> u16 {
        self.0
    }
}

impl Default for PageDepth {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// Page width (number of character columns per page).
///
/// Valid range: 60–255. Default: 132.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageWidth(pub u16);

impl PageWidth {
    /// Minimum allowed page width.
    pub const MIN: u16 = 60;
    /// Maximum allowed page width.
    pub const MAX: u16 = 255;
    /// Default page width (IBM 1403 standard).
    pub const DEFAULT: u16 = 132;

    /// Create a page width, clamping to the valid range.
    pub fn new(width: u16) -> Self {
        Self(width.clamp(Self::MIN, Self::MAX))
    }

    /// Get the underlying value.
    pub fn get(self) -> u16 {
        self.0
    }
}

impl Default for PageWidth {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_number_rejects_zero() {
        assert_eq!(PageNumber::try_new(0), None);
        assert_eq!(PageNumber::try_new(1), Some(PageNumber(1)));
    }

    #[test]
    fn page_number_from_u32_clamps_zero_to_one() {
        assert_eq!(PageNumber::from(0), PageNumber(1));
        assert_eq!(PageNumber::from(5), PageNumber(5));
    }

    #[test]
    fn page_depth_clamps_to_valid_range() {
        assert_eq!(PageDepth::new(5), PageDepth(10));
        assert_eq!(PageDepth::new(60), PageDepth(60));
        assert_eq!(PageDepth::new(200), PageDepth(120));
    }

    #[test]
    fn page_width_clamps_to_valid_range() {
        assert_eq!(PageWidth::new(10), PageWidth(60));
        assert_eq!(PageWidth::new(132), PageWidth(132));
        assert_eq!(PageWidth::new(300), PageWidth(255));
    }

    #[test]
    fn page_depth_default_is_60() {
        assert_eq!(PageDepth::default(), PageDepth(60));
    }

    #[test]
    fn page_width_default_is_132() {
        assert_eq!(PageWidth::default(), PageWidth(132));
    }
}
