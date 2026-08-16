//! RGBA colour type with hex parsing, serialisation, and display.
//!
//! All colours in the theme system are represented as `ColourRGBA` — a
//! simple value type with 8-bit components for red, green, blue, and alpha.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::ThemeError;

/// An RGBA colour value with 8-bit components.
///
/// This is the foundational colour type for the entire theme system.
/// All palette entries, style slots, element colours, and design tokens
/// use this representation.
///
/// # Examples
///
/// ```
/// use ff_theme::ColourRGBA;
///
/// let opaque = ColourRGBA::rgb(30, 30, 46);
/// assert_eq!(opaque.to_hex(), "#1E1E2E");
///
/// let translucent = ColourRGBA::rgba(30, 30, 46, 128);
/// assert_eq!(translucent.to_hex(), "#1E1E2E80");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColourRGBA {
    /// Red component (0–255).
    pub r: u8,
    /// Green component (0–255).
    pub g: u8,
    /// Blue component (0–255).
    pub b: u8,
    /// Alpha component (0–255, where 255 is fully opaque).
    pub a: u8,
}

impl ColourRGBA {
    /// Create a fully opaque colour from RGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a colour with explicit alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse from a hex string in `#RRGGBB` or `#RRGGBBAA` format.
    ///
    /// # Errors
    ///
    /// Returns `ThemeError::InvalidColourFormat` if the string does not
    /// start with `#`, has an invalid length, or contains non-hex characters.
    pub fn from_hex(s: &str) -> Result<Self, ThemeError> {
        let s = s.trim();
        if !s.starts_with('#') {
            return Err(ThemeError::InvalidColourFormat {
                input: s.to_string(),
            });
        }

        let hex = &s[1..];
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                    ThemeError::InvalidColourFormat {
                        input: s.to_string(),
                    }
                })?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                    ThemeError::InvalidColourFormat {
                        input: s.to_string(),
                    }
                })?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                    ThemeError::InvalidColourFormat {
                        input: s.to_string(),
                    }
                })?;
                Ok(Self { r, g, b, a: 255 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                    ThemeError::InvalidColourFormat {
                        input: s.to_string(),
                    }
                })?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                    ThemeError::InvalidColourFormat {
                        input: s.to_string(),
                    }
                })?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                    ThemeError::InvalidColourFormat {
                        input: s.to_string(),
                    }
                })?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| {
                    ThemeError::InvalidColourFormat {
                        input: s.to_string(),
                    }
                })?;
                Ok(Self { r, g, b, a })
            }
            _ => Err(ThemeError::InvalidColourFormat {
                input: s.to_string(),
            }),
        }
    }

    /// Serialise to a hex string.
    ///
    /// Opaque colours (alpha == 255) produce `#RRGGBB`.
    /// Translucent colours (alpha < 255) produce `#RRGGBBAA`.
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Check if this colour is fully opaque (alpha == 255).
    pub const fn is_opaque(&self) -> bool {
        self.a == 255
    }

    /// Return a copy with alpha forced to 255 (fully opaque).
    pub const fn as_opaque(&self) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 255,
        }
    }

    /// Compute the relative luminance of this colour using the WCAG 2.0 formula.
    ///
    /// Returns a value in the range [0.0, 1.0] where 0.0 is black and 1.0 is white.
    pub fn relative_luminance(&self) -> f64 {
        let r = srgb_to_linear(self.r);
        let g = srgb_to_linear(self.g);
        let b = srgb_to_linear(self.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Compute the WCAG 2.0 contrast ratio between two colours.
    ///
    /// Returns a value >= 1.0 where 1.0 means no contrast and 21.0 is maximum.
    pub fn contrast_ratio(&self, other: &Self) -> f64 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }
}

/// Convert an sRGB 8-bit component to linear space.
fn srgb_to_linear(component: u8) -> f64 {
    let c = f64::from(component) / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

impl fmt::Display for ColourRGBA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for ColourRGBA {
    fn default() -> Self {
        Self::rgb(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_parses_rrggbb_format() {
        // Validates: Requirement 2.9
        let colour = ColourRGBA::from_hex("#1E1E2E").unwrap();
        assert_eq!(colour, ColourRGBA::rgb(0x1E, 0x1E, 0x2E));
    }

    #[test]
    fn from_hex_parses_rrggbbaa_format() {
        // Validates: Requirement 2.9
        let colour = ColourRGBA::from_hex("#1E1E2E80").unwrap();
        assert_eq!(colour, ColourRGBA::rgba(0x1E, 0x1E, 0x2E, 0x80));
    }

    #[test]
    fn from_hex_rejects_missing_hash() {
        // Validates: Requirement 2.9
        assert!(ColourRGBA::from_hex("1E1E2E").is_err());
    }

    #[test]
    fn from_hex_rejects_invalid_length() {
        // Validates: Requirement 2.9
        assert!(ColourRGBA::from_hex("#1E1E").is_err());
        assert!(ColourRGBA::from_hex("#1E1E2E8").is_err());
        assert!(ColourRGBA::from_hex("#1E1E2E80FF").is_err());
    }

    #[test]
    fn from_hex_rejects_non_hex_characters() {
        // Validates: Requirement 2.9
        assert!(ColourRGBA::from_hex("#GGGGGG").is_err());
        assert!(ColourRGBA::from_hex("#1E1EZZ").is_err());
    }

    #[test]
    fn to_hex_opaque_produces_rrggbb() {
        // Validates: Requirement 9.5
        let colour = ColourRGBA::rgb(0xFF, 0x00, 0xAB);
        assert_eq!(colour.to_hex(), "#FF00AB");
    }

    #[test]
    fn to_hex_translucent_produces_rrggbbaa() {
        // Validates: Requirement 9.5
        let colour = ColourRGBA::rgba(0xFF, 0x00, 0xAB, 0x80);
        assert_eq!(colour.to_hex(), "#FF00AB80");
    }

    #[test]
    fn hex_round_trip_opaque() {
        // Validates: Requirement 9.5
        let original = ColourRGBA::rgb(0x12, 0x34, 0x56);
        let round_tripped = ColourRGBA::from_hex(&original.to_hex()).unwrap();
        assert_eq!(original, round_tripped);
    }

    #[test]
    fn hex_round_trip_translucent() {
        // Validates: Requirement 9.5
        let original = ColourRGBA::rgba(0xAB, 0xCD, 0xEF, 0x42);
        let round_tripped = ColourRGBA::from_hex(&original.to_hex()).unwrap();
        assert_eq!(original, round_tripped);
    }

    #[test]
    fn contrast_ratio_black_white_is_21() {
        // Validates: Requirement 5.6
        let black = ColourRGBA::rgb(0, 0, 0);
        let white = ColourRGBA::rgb(255, 255, 255);
        let ratio = black.contrast_ratio(&white);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn contrast_ratio_same_colour_is_1() {
        // Validates: Requirement 5.6
        let colour = ColourRGBA::rgb(128, 128, 128);
        let ratio = colour.contrast_ratio(&colour);
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn as_opaque_forces_alpha_to_255() {
        // Validates: Requirement 10.4
        let translucent = ColourRGBA::rgba(100, 150, 200, 50);
        let opaque = translucent.as_opaque();
        assert_eq!(opaque.a, 255);
        assert_eq!(opaque.r, 100);
        assert_eq!(opaque.g, 150);
        assert_eq!(opaque.b, 200);
    }

    #[test]
    fn display_uses_hex_format() {
        let colour = ColourRGBA::rgb(0xDE, 0xAD, 0xBE);
        assert_eq!(format!("{colour}"), "#DEADBE");
    }
}
