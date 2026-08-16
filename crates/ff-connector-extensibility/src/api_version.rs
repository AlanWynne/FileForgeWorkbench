//! API version type and compatibility checking.
//!
//! Defines `ApiVersion` — the version of the connector API that a connector
//! was built against — and the compatibility checking logic used during
//! registration to ensure connectors are compatible with the current platform.

use std::fmt;

/// Represents the VFS core API version for compatibility checking.
///
/// A connector is compatible with the current platform if:
/// - Same major version (breaking changes increment major)
/// - Connector minor ≤ current minor (new features increment minor)
/// - Patch version is irrelevant for compatibility
///
/// # Examples
///
/// ```
/// use ff_connector_extensibility::ApiVersion;
///
/// let connector_version = ApiVersion::new(1, 0, 0);
/// let current = ApiVersion::new(1, 2, 0);
/// assert!(connector_version.is_compatible_with(&current));
/// ```
///
/// Addresses: Requirement 1 AC 4, Requirement 2 AC 2c
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiVersion {
    /// Major version — incompatible API changes.
    pub major: u32,
    /// Minor version — backwards-compatible additions.
    pub minor: u32,
    /// Patch version — backwards-compatible bug fixes.
    pub patch: u32,
}

/// The current connector API version provided by this crate.
///
/// Connectors declare their built-against version; registration validates
/// compatibility against this constant.
pub const CONNECTOR_API_VERSION: ApiVersion = ApiVersion {
    major: 1,
    minor: 0,
    patch: 0,
};

impl ApiVersion {
    /// Creates a new `ApiVersion` with the given components.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Check if this connector's declared API version is compatible with the
    /// current platform version.
    ///
    /// Compatible means: same major version AND self.minor ≤ current.minor.
    /// Patch version is irrelevant for compatibility.
    ///
    /// Addresses: Requirement 2 AC 2c
    pub fn is_compatible_with(&self, current: &ApiVersion) -> bool {
        self.major == current.major && self.minor <= current.minor
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1 AC 4, Requirement 2 AC 2c
    #[test]
    fn same_version_is_compatible() {
        let v = ApiVersion::new(1, 0, 0);
        assert!(v.is_compatible_with(&v));
    }

    // Validates: Requirement 1 AC 4, Requirement 2 AC 2c
    #[test]
    fn connector_minor_less_than_current_is_compatible() {
        let connector = ApiVersion::new(1, 0, 0);
        let current = ApiVersion::new(1, 2, 0);
        assert!(connector.is_compatible_with(&current));
    }

    // Validates: Requirement 1 AC 4, Requirement 2 AC 2c
    #[test]
    fn connector_minor_greater_than_current_is_incompatible() {
        let connector = ApiVersion::new(1, 3, 0);
        let current = ApiVersion::new(1, 2, 0);
        assert!(!connector.is_compatible_with(&current));
    }

    // Validates: Requirement 1 AC 4, Requirement 2 AC 2c
    #[test]
    fn different_major_is_incompatible() {
        let connector = ApiVersion::new(2, 0, 0);
        let current = ApiVersion::new(1, 5, 0);
        assert!(!connector.is_compatible_with(&current));
    }

    // Validates: Requirement 1 AC 4, Requirement 2 AC 2c
    #[test]
    fn patch_version_is_irrelevant_for_compatibility() {
        let connector = ApiVersion::new(1, 0, 99);
        let current = ApiVersion::new(1, 0, 0);
        assert!(connector.is_compatible_with(&current));
    }

    #[test]
    fn display_format_is_semver() {
        let v = ApiVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn ordering_is_lexicographic() {
        let v1 = ApiVersion::new(1, 0, 0);
        let v2 = ApiVersion::new(1, 1, 0);
        let v3 = ApiVersion::new(2, 0, 0);
        assert!(v1 < v2);
        assert!(v2 < v3);
    }

    #[test]
    fn connector_api_version_constant_is_1_0_0() {
        assert_eq!(CONNECTOR_API_VERSION, ApiVersion::new(1, 0, 0));
    }
}
