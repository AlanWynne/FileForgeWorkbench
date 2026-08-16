//! Wrap visual flag bitfield.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Bitfield controlling which wrap markers are displayed.
///
/// Combinable flags: `NONE`, `END`, `START`, `MARGIN`.
///
/// Addresses: Requirement 6 AC 6.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WrapVisualFlag(u8);

impl WrapVisualFlag {
    /// No wrap markers displayed.
    pub const NONE: Self = Self(0);
    /// Marker at the end of each sub-line that continues.
    pub const END: Self = Self(1);
    /// Marker at the start of each continuation sub-line.
    pub const START: Self = Self(2);
    /// Marker in the margin area for wrapped lines.
    pub const MARGIN: Self = Self(4);

    /// Check if the END flag is set.
    pub fn has_end(self) -> bool {
        self.0 & Self::END.0 != 0
    }

    /// Check if the START flag is set.
    pub fn has_start(self) -> bool {
        self.0 & Self::START.0 != 0
    }

    /// Check if the MARGIN flag is set.
    pub fn has_margin(self) -> bool {
        self.0 & Self::MARGIN.0 != 0
    }

    /// Create from raw bits, masking to valid range (0–7).
    pub fn from_bits(bits: u8) -> Self {
        Self(bits & 0x07)
    }

    /// Get the raw bit value.
    pub fn bits(self) -> u8 {
        self.0
    }

    /// Combine two flags (bitwise OR).
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl Serialize for WrapVisualFlag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.0)
    }
}

impl<'de> Deserialize<'de> for WrapVisualFlag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bits = u8::deserialize(deserializer)?;
        Ok(Self::from_bits(bits))
    }
}
