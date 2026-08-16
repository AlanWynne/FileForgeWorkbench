//! Record format types for flat files.
//!
//! Describes the physical structure of records in a source file,
//! determining how record boundaries are identified.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Physical record format of the source flat file.
///
/// Determines how record boundaries are identified and how the
/// index builder operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecordFormat {
    /// Fixed-length records (one record = LRECL bytes).
    F,
    /// Fixed-blocked — same as F but implies blocking factor.
    Fb,
    /// Variable-length records (newline-delimited).
    V,
    /// Fixed-blocked binary — binary file with fixed LRECL, no newlines.
    FbBinary,
    /// Variable-length binary — RDW-prefixed records.
    Vb,
    /// Fixed-blocked with ASA carriage control in column 1.
    Fba,
    /// Variable-blocked with ASA carriage control in column 1.
    Vba,
    /// Undefined/unstructured.
    U,
}

impl RecordFormat {
    /// Returns the canonical uppercase string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::F => "F",
            Self::Fb => "FB",
            Self::V => "V",
            Self::FbBinary => "FB_BINARY",
            Self::Vb => "VB",
            Self::Fba => "FBA",
            Self::Vba => "VBA",
            Self::U => "U",
        }
    }

    /// Returns true if this format uses ASA carriage control.
    pub fn is_asa(&self) -> bool {
        matches!(self, Self::Fba | Self::Vba)
    }

    /// Returns true if this format uses fixed-width records.
    pub fn is_fixed_width(&self) -> bool {
        matches!(self, Self::F | Self::Fb | Self::FbBinary | Self::Fba)
    }

    /// Returns true if this format uses VB binary (RDW) records.
    pub fn is_vb(&self) -> bool {
        matches!(self, Self::Vb)
    }
}

impl fmt::Display for RecordFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RecordFormat {
    type Err = String;

    /// Parses a record format string (case-insensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "F" => Ok(Self::F),
            "FB" => Ok(Self::Fb),
            "V" => Ok(Self::V),
            "FB_BINARY" => Ok(Self::FbBinary),
            "VB" => Ok(Self::Vb),
            "FBA" => Ok(Self::Fba),
            "VBA" => Ok(Self::Vba),
            "U" => Ok(Self::U),
            _ => Err(format!("unknown record format: '{s}'")),
        }
    }
}

impl Serialize for RecordFormat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RecordFormat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.4
    #[test]
    fn parse_case_insensitive_all_formats() {
        assert_eq!("f".parse::<RecordFormat>().unwrap(), RecordFormat::F);
        assert_eq!("FB".parse::<RecordFormat>().unwrap(), RecordFormat::Fb);
        assert_eq!("fb".parse::<RecordFormat>().unwrap(), RecordFormat::Fb);
        assert_eq!("v".parse::<RecordFormat>().unwrap(), RecordFormat::V);
        assert_eq!(
            "fb_binary".parse::<RecordFormat>().unwrap(),
            RecordFormat::FbBinary
        );
        assert_eq!(
            "FB_BINARY".parse::<RecordFormat>().unwrap(),
            RecordFormat::FbBinary
        );
        assert_eq!("vb".parse::<RecordFormat>().unwrap(), RecordFormat::Vb);
        assert_eq!("VB".parse::<RecordFormat>().unwrap(), RecordFormat::Vb);
        assert_eq!("fba".parse::<RecordFormat>().unwrap(), RecordFormat::Fba);
        assert_eq!("FBA".parse::<RecordFormat>().unwrap(), RecordFormat::Fba);
        assert_eq!("vba".parse::<RecordFormat>().unwrap(), RecordFormat::Vba);
        assert_eq!("u".parse::<RecordFormat>().unwrap(), RecordFormat::U);
    }

    #[test]
    fn parse_invalid_format_returns_error() {
        assert!("XYZ".parse::<RecordFormat>().is_err());
        assert!("".parse::<RecordFormat>().is_err());
    }

    #[test]
    fn display_normalises_to_uppercase() {
        assert_eq!(RecordFormat::F.to_string(), "F");
        assert_eq!(RecordFormat::Fb.to_string(), "FB");
        assert_eq!(RecordFormat::FbBinary.to_string(), "FB_BINARY");
        assert_eq!(RecordFormat::Vb.to_string(), "VB");
        assert_eq!(RecordFormat::Fba.to_string(), "FBA");
        assert_eq!(RecordFormat::Vba.to_string(), "VBA");
        assert_eq!(RecordFormat::U.to_string(), "U");
    }

    #[test]
    fn serialize_to_uppercase_string() {
        let json = serde_json::to_string(&RecordFormat::FbBinary).unwrap();
        assert_eq!(json, "\"FB_BINARY\"");
    }

    #[test]
    fn deserialize_case_insensitive() {
        let fmt: RecordFormat = serde_json::from_str("\"fb_binary\"").unwrap();
        assert_eq!(fmt, RecordFormat::FbBinary);
    }

    #[test]
    fn is_asa_only_for_fba_vba() {
        assert!(RecordFormat::Fba.is_asa());
        assert!(RecordFormat::Vba.is_asa());
        assert!(!RecordFormat::Fb.is_asa());
        assert!(!RecordFormat::Vb.is_asa());
    }

    #[test]
    fn is_fixed_width_for_f_fb_fbbinary_fba() {
        assert!(RecordFormat::F.is_fixed_width());
        assert!(RecordFormat::Fb.is_fixed_width());
        assert!(RecordFormat::FbBinary.is_fixed_width());
        assert!(RecordFormat::Fba.is_fixed_width());
        assert!(!RecordFormat::V.is_fixed_width());
        assert!(!RecordFormat::Vb.is_fixed_width());
    }
}
