//! Semantic versioning types for the plugin API.
//!
//! Provides `Version` and `VersionReq` types used throughout the plugin
//! architecture for API compatibility checking and dependency resolution.

use std::fmt;
use std::str::FromStr;

/// Semantic version: major.minor.patch.
///
/// Used for both the plugin API version and individual plugin versions.
/// Implements `PartialOrd` and `Ord` for comparison.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    /// Major version — incompatible API changes.
    pub major: u32,
    /// Minor version — backwards-compatible additions.
    pub minor: u32,
    /// Patch version — backwards-compatible bug fixes.
    pub patch: u32,
}

impl Version {
    /// Creates a new version with the given components.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(format!(
                "invalid version format: '{s}' (expected major.minor.patch)"
            ));
        }
        let major = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("invalid major version: {e}"))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("invalid minor version: {e}"))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|e| format!("invalid patch version: {e}"))?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

/// A version requirement for dependency declarations.
///
/// Specifies a minimum version and optionally constrains the major version
/// to match exactly (semver-compatible range).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionReq {
    /// The minimum version required (inclusive).
    pub minimum: Version,
    /// Whether the major version must match exactly.
    pub same_major: bool,
}

impl VersionReq {
    /// Creates a new version requirement.
    pub fn new(minimum: Version, same_major: bool) -> Self {
        Self {
            minimum,
            same_major,
        }
    }

    /// Checks whether the given version satisfies this requirement.
    ///
    /// If `same_major` is true, the version must have the same major number
    /// as the minimum AND be >= the minimum version.
    /// If `same_major` is false, the version just needs to be >= the minimum.
    pub fn matches(&self, version: &Version) -> bool {
        if self.same_major && version.major != self.minimum.major {
            return false;
        }
        version >= &self.minimum
    }
}

impl fmt::Display for VersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.same_major {
            write!(f, "^{}", self.minimum)
        } else {
            write!(f, ">={}", self.minimum)
        }
    }
}

/// The current version of the plugin API contract.
///
/// Plugins declare their `required_api_version` against this constant.
/// Compatibility rules:
/// - Different major version → reject
/// - Same major, plugin minor > host minor → reject
/// - Same major, plugin minor <= host minor → accept
pub const PLUGIN_API_VERSION: Version = Version::new(1, 0, 0);

/// Checks whether a plugin's required API version is compatible with the host.
///
/// # Rules
///
/// - Different major version → incompatible (reject)
/// - Same major, required minor > available minor → incompatible (needs newer API)
/// - Same major, required minor <= available minor → compatible (accept)
///
/// Patch version is ignored for compatibility decisions.
pub fn check_api_compatibility(
    plugin_name: &str,
    required: &Version,
    available: &Version,
) -> Result<(), crate::error::PluginError> {
    if required.major == available.major && required.minor <= available.minor {
        Ok(())
    } else {
        Err(crate::error::PluginError::IncompatibleApiVersion {
            plugin: plugin_name.to_string(),
            required: required.clone(),
            available: available.clone(),
        })
    }
}

/// Simple boolean check for API compatibility (without error context).
pub fn is_api_compatible(required: &Version, available: &Version) -> bool {
    required.major == available.major && required.minor <= available.minor
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Version Display and FromStr ────────────────────────────────────────

    #[test]
    fn version_display_formats_correctly() {
        // Validates: Requirement 6.1
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn version_from_str_parses_valid_input() {
        // Validates: Requirement 6.1
        let v: Version = "2.5.10".parse().unwrap();
        assert_eq!(v, Version::new(2, 5, 10));
    }

    #[test]
    fn version_from_str_rejects_invalid_format() {
        // Validates: Requirement 6.1
        assert!("1.2".parse::<Version>().is_err());
        assert!("not.a.version".parse::<Version>().is_err());
        assert!("1.2.3.4".parse::<Version>().is_err());
    }

    // ─── Version Comparison ─────────────────────────────────────────────────

    #[test]
    fn version_ordering_compares_major_first() {
        // Validates: Requirement 6.1
        assert!(Version::new(2, 0, 0) > Version::new(1, 9, 9));
    }

    #[test]
    fn version_ordering_compares_minor_second() {
        // Validates: Requirement 6.1
        assert!(Version::new(1, 2, 0) > Version::new(1, 1, 9));
    }

    #[test]
    fn version_ordering_compares_patch_last() {
        // Validates: Requirement 6.1
        assert!(Version::new(1, 0, 2) > Version::new(1, 0, 1));
    }

    #[test]
    fn version_equality() {
        // Validates: Requirement 6.1
        assert_eq!(Version::new(1, 0, 0), Version::new(1, 0, 0));
    }

    // ─── VersionReq Matching ────────────────────────────────────────────────

    #[test]
    fn version_req_matches_exact_minimum() {
        // Validates: Requirement 6.2
        let req = VersionReq::new(Version::new(1, 0, 0), true);
        assert!(req.matches(&Version::new(1, 0, 0)));
    }

    #[test]
    fn version_req_matches_higher_minor() {
        // Validates: Requirement 6.4
        let req = VersionReq::new(Version::new(1, 0, 0), true);
        assert!(req.matches(&Version::new(1, 2, 0)));
    }

    #[test]
    fn version_req_rejects_different_major_when_same_major_required() {
        // Validates: Requirement 6.3
        let req = VersionReq::new(Version::new(1, 0, 0), true);
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn version_req_rejects_lower_version() {
        // Validates: Requirement 6.5
        let req = VersionReq::new(Version::new(1, 2, 0), true);
        assert!(!req.matches(&Version::new(1, 1, 0)));
    }

    #[test]
    fn version_req_without_same_major_accepts_higher_major() {
        // Validates: Requirement 6.2
        let req = VersionReq::new(Version::new(1, 0, 0), false);
        assert!(req.matches(&Version::new(2, 0, 0)));
    }

    // ─── API Compatibility Check ────────────────────────────────────────────

    #[test]
    fn api_compatibility_accepts_same_version() {
        // Validates: Requirement 6.4
        assert!(
            check_api_compatibility("test", &Version::new(1, 0, 0), &Version::new(1, 0, 0)).is_ok()
        );
    }

    #[test]
    fn api_compatibility_accepts_higher_minor() {
        // Validates: Requirement 6.4
        assert!(
            check_api_compatibility("test", &Version::new(1, 0, 0), &Version::new(1, 5, 0)).is_ok()
        );
    }

    #[test]
    fn api_compatibility_rejects_different_major() {
        // Validates: Requirement 6.3
        assert!(
            check_api_compatibility("test", &Version::new(2, 0, 0), &Version::new(1, 0, 0))
                .is_err()
        );
    }

    #[test]
    fn api_compatibility_rejects_higher_minor_required() {
        // Validates: Requirement 6.5
        assert!(
            check_api_compatibility("test", &Version::new(1, 3, 0), &Version::new(1, 2, 0))
                .is_err()
        );
    }

    #[test]
    fn api_compatibility_ignores_patch_version() {
        // Validates: Requirement 6.4
        assert!(
            check_api_compatibility("test", &Version::new(1, 0, 5), &Version::new(1, 0, 0)).is_ok()
        );
    }

    #[test]
    fn plugin_api_version_constant_is_1_0_0() {
        // Validates: Requirement 6.1
        assert_eq!(PLUGIN_API_VERSION, Version::new(1, 0, 0));
    }
}
