//! Colour types for the caret-selection crate.
//!
//! Provides `ColourRGBA` — a simple RGBA colour representation used throughout
//! the crate. When `ff-theme` is available, this will bridge to its colour type.

use serde::{Deserialize, Serialize};

/// An RGBA colour with 8-bit per channel precision.
///
/// Used for all colour settings in the caret-selection system.
/// Alpha of 0xFF means fully opaque; 0x00 means fully transparent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColourRGBA {
    /// Red channel [0, 255].
    pub r: u8,
    /// Green channel [0, 255].
    pub g: u8,
    /// Blue channel [0, 255].
    pub b: u8,
    /// Alpha channel [0, 255]. 0xFF = opaque, 0x00 = transparent.
    pub a: u8,
}

impl ColourRGBA {
    /// Creates a fully opaque colour from RGB values.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    /// Creates a colour with explicit alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the inverse of this colour (complement), preserving alpha.
    ///
    /// Used for block-caret text inversion to maintain legibility.
    pub const fn inverse(&self) -> Self {
        Self {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
            a: self.a,
        }
    }

    /// Returns true if this colour is fully opaque (alpha == 0xFF).
    pub const fn is_opaque(&self) -> bool {
        self.a == 0xFF
    }

    /// Returns true if this colour is fully transparent (alpha == 0x00).
    pub const fn is_transparent(&self) -> bool {
        self.a == 0x00
    }

    /// Black (#000000) fully opaque.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// White (#FFFFFF) fully opaque.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
}

impl Default for ColourRGBA {
    fn default() -> Self {
        Self::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_creates_opaque_colour() {
        let c = ColourRGBA::rgb(128, 64, 32);
        assert_eq!(c.r, 128);
        assert_eq!(c.g, 64);
        assert_eq!(c.b, 32);
        assert_eq!(c.a, 0xFF);
        assert!(c.is_opaque());
    }

    #[test]
    fn rgba_creates_colour_with_alpha() {
        let c = ColourRGBA::rgba(100, 200, 50, 0x3F);
        assert_eq!(c.a, 0x3F);
        assert!(!c.is_opaque());
        assert!(!c.is_transparent());
    }

    #[test]
    fn inverse_produces_colour_complement() {
        let c = ColourRGBA::rgb(0, 0, 0);
        let inv = c.inverse();
        assert_eq!(inv, ColourRGBA::rgb(255, 255, 255));
    }

    #[test]
    fn inverse_preserves_alpha() {
        let c = ColourRGBA::rgba(100, 150, 200, 0x80);
        let inv = c.inverse();
        assert_eq!(inv.a, 0x80);
        assert_eq!(inv.r, 155);
        assert_eq!(inv.g, 105);
        assert_eq!(inv.b, 55);
    }

    #[test]
    fn default_colour_is_black() {
        assert_eq!(ColourRGBA::default(), ColourRGBA::BLACK);
    }
}
