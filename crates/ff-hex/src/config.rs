//! Hex display configuration.
//!
//! Typed access to hex display configuration settings.
//! Reads from the configuration system under `editor.hex.*`.

use crate::error::HexError;
use crate::types::{AutoActivateBinary, BytesPerRow, HexDigitCase};

/// Typed access to hex display configuration settings.
///
/// Configuration keys:
/// - `editor.hex.bytes_per_row` — 8, 16, 32, or 64 (default: 16)
/// - `editor.hex.digit_case` — "uppercase" or "lowercase" (default: "uppercase")
/// - `editor.hex.auto_activate_binary` — "always", "prompt", or "never" (default: "prompt")
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HexConfig {
    /// Bytes per row (default: 16).
    pub bytes_per_row: BytesPerRow,
    /// Hex digit case (default: uppercase).
    pub digit_case: HexDigitCase,
    /// Auto-activate hex for binary files (default: prompt).
    pub auto_activate_binary: AutoActivateBinary,
}

impl HexConfig {
    /// Validate and apply a bytes_per_row change.
    ///
    /// Returns an error if the value is not 8, 16, 32, or 64.
    pub fn set_bytes_per_row(&mut self, value: u32) -> Result<(), HexError> {
        let bpr = BytesPerRow::from_value(value).ok_or(HexError::InvalidBytesPerRow(value))?;
        self.bytes_per_row = bpr;
        Ok(())
    }

    /// Set the digit case from a string value.
    ///
    /// Accepts "uppercase" or "lowercase".
    pub fn set_digit_case_from_str(&mut self, value: &str) -> Result<(), HexError> {
        match value {
            "uppercase" => {
                self.digit_case = HexDigitCase::Uppercase;
                Ok(())
            }
            "lowercase" => {
                self.digit_case = HexDigitCase::Lowercase;
                Ok(())
            }
            _ => Err(HexError::DumpExportFailed(format!(
                "invalid digit_case value: {value}"
            ))),
        }
    }

    /// Set the auto-activate binary behaviour from a string value.
    ///
    /// Accepts "always", "prompt", or "never".
    pub fn set_auto_activate_from_str(&mut self, value: &str) -> Result<(), HexError> {
        match value {
            "always" => {
                self.auto_activate_binary = AutoActivateBinary::Always;
                Ok(())
            }
            "prompt" => {
                self.auto_activate_binary = AutoActivateBinary::Prompt;
                Ok(())
            }
            "never" => {
                self.auto_activate_binary = AutoActivateBinary::Never;
                Ok(())
            }
            _ => Err(HexError::DumpExportFailed(format!(
                "invalid auto_activate_binary value: {value}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 3 AC 2
    #[test]
    fn default_config_values() {
        let config = HexConfig::default();
        assert_eq!(config.bytes_per_row, BytesPerRow::Sixteen);
        assert_eq!(config.digit_case, HexDigitCase::Uppercase);
        assert_eq!(config.auto_activate_binary, AutoActivateBinary::Prompt);
    }

    // Validates: Requirement 3 AC 1, 3.4
    #[test]
    fn set_bytes_per_row_valid_values() {
        let mut config = HexConfig::default();
        assert!(config.set_bytes_per_row(8).is_ok());
        assert_eq!(config.bytes_per_row, BytesPerRow::Eight);
        assert!(config.set_bytes_per_row(32).is_ok());
        assert_eq!(config.bytes_per_row, BytesPerRow::ThirtyTwo);
    }

    // Validates: Requirement 3 AC 4
    #[test]
    fn set_bytes_per_row_invalid_rejected() {
        let mut config = HexConfig::default();
        let result = config.set_bytes_per_row(12);
        assert!(result.is_err());
        assert_eq!(config.bytes_per_row, BytesPerRow::Sixteen); // unchanged
    }

    // Validates: Requirement 13 AC 1, 5
    #[test]
    fn set_digit_case_from_str() {
        let mut config = HexConfig::default();
        assert!(config.set_digit_case_from_str("lowercase").is_ok());
        assert_eq!(config.digit_case, HexDigitCase::Lowercase);
        assert!(config.set_digit_case_from_str("uppercase").is_ok());
        assert_eq!(config.digit_case, HexDigitCase::Uppercase);
    }

    // Validates: Requirement 13 AC 5
    #[test]
    fn set_digit_case_invalid_rejected() {
        let mut config = HexConfig::default();
        assert!(config.set_digit_case_from_str("mixed").is_err());
    }

    // Validates: Requirement 10 AC 2
    #[test]
    fn set_auto_activate_from_str() {
        let mut config = HexConfig::default();
        assert!(config.set_auto_activate_from_str("always").is_ok());
        assert_eq!(config.auto_activate_binary, AutoActivateBinary::Always);
        assert!(config.set_auto_activate_from_str("never").is_ok());
        assert_eq!(config.auto_activate_binary, AutoActivateBinary::Never);
        assert!(config.set_auto_activate_from_str("prompt").is_ok());
        assert_eq!(config.auto_activate_binary, AutoActivateBinary::Prompt);
    }
}
