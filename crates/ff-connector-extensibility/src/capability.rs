//! Connector capability advertisement and validation.
//!
//! Defines `ConnectorCapability` — the set of VFS operations a connector can
//! support — and validation logic to ensure connectors declare the minimum
//! required capabilities at registration time.

use crate::error::ConnectorError;

/// Enumerates VFS operations a connector can support.
///
/// Consumers query these before attempting operations to provide appropriate
/// UI affordances and avoid operations that would fail.
///
/// Marked `#[non_exhaustive]` to allow future additions without breaking changes.
///
/// Addresses: Requirement 3 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConnectorCapability {
    /// Read file content (REQUIRED).
    Read,
    /// Write/upload file content.
    Write,
    /// Watch for file changes (real-time notifications).
    Watch,
    /// Search file contents or filenames.
    Search,
    /// Rename/move resources.
    Rename,
    /// Delete resources.
    Delete,
    /// Create directories/containers.
    CreateDirectory,
    /// Retrieve resource metadata (REQUIRED).
    Metadata,
    /// List directory/container contents (REQUIRED).
    List,
    /// Copy resources within the provider.
    Copy,
}

/// Capabilities that MUST be present for a connector to pass registration.
///
/// A connector that does not support Read, List, and Metadata cannot function
/// as a useful VFS provider and will be rejected at registration time.
///
/// Addresses: Requirement 3 AC 2
pub const REQUIRED_CAPABILITIES: &[ConnectorCapability] = &[
    ConnectorCapability::Read,
    ConnectorCapability::List,
    ConnectorCapability::Metadata,
];

/// Validates that a connector's declared capabilities meet registration requirements.
///
/// Returns `Ok(())` if all required capabilities (Read, List, Metadata) are present.
/// Returns `Err(ConnectorError::RegistrationFailed)` if any required capability is missing.
///
/// Addresses: Requirement 3 AC 2
pub fn validate_capabilities(capabilities: &[ConnectorCapability]) -> Result<(), ConnectorError> {
    let mut missing = Vec::new();

    for required in REQUIRED_CAPABILITIES {
        if !capabilities.contains(required) {
            missing.push(format!("{required:?}"));
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(ConnectorError::RegistrationFailed {
            message: format!("missing required capabilities: {}", missing.join(", ")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3 AC 2
    #[test]
    fn validate_capabilities_passes_with_all_required() {
        let caps = vec![
            ConnectorCapability::Read,
            ConnectorCapability::List,
            ConnectorCapability::Metadata,
        ];
        assert!(validate_capabilities(&caps).is_ok());
    }

    // Validates: Requirement 3 AC 2
    #[test]
    fn validate_capabilities_passes_with_required_plus_optional() {
        let caps = vec![
            ConnectorCapability::Read,
            ConnectorCapability::Write,
            ConnectorCapability::List,
            ConnectorCapability::Metadata,
            ConnectorCapability::Search,
        ];
        assert!(validate_capabilities(&caps).is_ok());
    }

    // Validates: Requirement 3 AC 2
    #[test]
    fn validate_capabilities_fails_when_read_missing() {
        let caps = vec![ConnectorCapability::List, ConnectorCapability::Metadata];
        let result = validate_capabilities(&caps);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Read"));
    }

    // Validates: Requirement 3 AC 2
    #[test]
    fn validate_capabilities_fails_when_list_missing() {
        let caps = vec![ConnectorCapability::Read, ConnectorCapability::Metadata];
        let result = validate_capabilities(&caps);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("List"));
    }

    // Validates: Requirement 3 AC 2
    #[test]
    fn validate_capabilities_fails_when_metadata_missing() {
        let caps = vec![ConnectorCapability::Read, ConnectorCapability::List];
        let result = validate_capabilities(&caps);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Metadata"));
    }

    // Validates: Requirement 3 AC 2
    #[test]
    fn validate_capabilities_fails_with_empty_set() {
        let caps: Vec<ConnectorCapability> = vec![];
        let result = validate_capabilities(&caps);
        assert!(result.is_err());
    }

    // Validates: Requirement 3 AC 1
    #[test]
    fn capability_enum_has_all_expected_variants() {
        // Verify all 10 documented variants exist and are distinct
        let all = vec![
            ConnectorCapability::Read,
            ConnectorCapability::Write,
            ConnectorCapability::Watch,
            ConnectorCapability::Search,
            ConnectorCapability::Rename,
            ConnectorCapability::Delete,
            ConnectorCapability::CreateDirectory,
            ConnectorCapability::Metadata,
            ConnectorCapability::List,
            ConnectorCapability::Copy,
        ];
        let set: std::collections::HashSet<_> = all.iter().collect();
        assert_eq!(set.len(), 10);
    }
}
