//! ViewerKey — validated, unique identifier for a viewer.
//!
//! A ViewerKey is a non-empty string containing only lowercase ASCII letters,
//! digits, and hyphens, with a maximum length of 64 characters.

use crate::error::ViewerError;

/// A validated viewer key identifier.
///
/// ViewerKeys are used to uniquely identify viewers in the registry, commands,
/// configuration, and language profile declarations. They follow a strict format:
/// - Non-empty
/// - Maximum 64 characters
/// - Contains only lowercase ASCII letters (`a-z`), digits (`0-9`), and hyphens (`-`)
///
/// # Examples
///
/// ```
/// use ff_viewers::ViewerKey;
///
/// let key = ViewerKey::new("asa-report").unwrap();
/// assert_eq!(key.as_str(), "asa-report");
///
/// // Invalid: contains uppercase
/// assert!(ViewerKey::new("ASA-Report").is_err());
///
/// // Invalid: empty string
/// assert!(ViewerKey::new("").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ViewerKey(String);

impl ViewerKey {
    /// Maximum length for a viewer key.
    pub const MAX_LENGTH: usize = 64;

    /// Parse and validate a viewer key string.
    ///
    /// # Errors
    ///
    /// Returns `ViewerError::InvalidKeyFormat` if the string is empty, exceeds
    /// 64 characters, or contains characters other than lowercase ASCII letters,
    /// digits, and hyphens.
    pub fn new(key: &str) -> Result<Self, ViewerError> {
        if key.is_empty() {
            return Err(ViewerError::InvalidKeyFormat {
                key: key.to_string(),
                reason: "viewer key must not be empty".to_string(),
            });
        }

        if key.len() > Self::MAX_LENGTH {
            return Err(ViewerError::InvalidKeyFormat {
                key: key.to_string(),
                reason: format!(
                    "viewer key must be at most {} characters, got {}",
                    Self::MAX_LENGTH,
                    key.len()
                ),
            });
        }

        if !key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(ViewerError::InvalidKeyFormat {
                key: key.to_string(),
                reason: "viewer key must contain only lowercase ASCII letters, digits, and hyphens"
                    .to_string(),
            });
        }

        Ok(Self(key.to_string()))
    }

    /// Returns the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ViewerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ViewerKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_simple_key() {
        // Validates: Requirement 1 AC 1
        let key = ViewerKey::new("hex").unwrap();
        assert_eq!(key.as_str(), "hex");
    }

    #[test]
    fn valid_key_with_hyphens() {
        // Validates: Requirement 1 AC 1
        let key = ViewerKey::new("asa-report").unwrap();
        assert_eq!(key.as_str(), "asa-report");
    }

    #[test]
    fn valid_key_with_digits() {
        // Validates: Requirement 1 AC 1
        let key = ViewerKey::new("viewer-2").unwrap();
        assert_eq!(key.as_str(), "viewer-2");
    }

    #[test]
    fn empty_key_rejected() {
        // Validates: Requirement 1 AC 1
        let result = ViewerKey::new("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn uppercase_key_rejected() {
        // Validates: Requirement 1 AC 1
        let result = ViewerKey::new("ASA-Report");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("lowercase ASCII"));
    }

    #[test]
    fn key_with_spaces_rejected() {
        // Validates: Requirement 1 AC 1
        let result = ViewerKey::new("my viewer");
        assert!(result.is_err());
    }

    #[test]
    fn key_with_underscore_rejected() {
        // Validates: Requirement 1 AC 1
        let result = ViewerKey::new("my_viewer");
        assert!(result.is_err());
    }

    #[test]
    fn key_exceeding_max_length_rejected() {
        // Validates: Requirement 1 AC 1
        let long_key = "a".repeat(65);
        let result = ViewerKey::new(&long_key);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("at most 64"));
    }

    #[test]
    fn key_at_max_length_accepted() {
        // Validates: Requirement 1 AC 1
        let max_key = "a".repeat(64);
        let key = ViewerKey::new(&max_key).unwrap();
        assert_eq!(key.as_str().len(), 64);
    }

    #[test]
    fn display_impl_shows_key_string() {
        let key = ViewerKey::new("csv-table").unwrap();
        assert_eq!(format!("{key}"), "csv-table");
    }
}
