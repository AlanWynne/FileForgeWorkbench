//! Connector descriptor metadata type.
//!
//! Defines `ConnectorDescriptor` — metadata identifying a connector including
//! its URI scheme, display name, description, icon, and version.

use ff_plugin::Version;

/// Metadata identifying a connector: scheme, display name, version, etc.
///
/// Every connector provides a descriptor during registration. The scheme
/// serves as the unique identifier for URI routing (e.g., "ftp", "sftp", "zos").
///
/// Addresses: Requirement 1 AC 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorDescriptor {
    /// Unique URI scheme identifier (e.g., "ftp", "sftp", "zos", "onedrive").
    pub scheme: String,
    /// Human-readable display name (e.g., "FTP/FTPS Connector").
    pub display_name: String,
    /// One-line description of the connector.
    pub description: String,
    /// Optional icon identifier for UI rendering.
    pub icon: Option<String>,
    /// Semantic version of the connector implementation.
    pub version: Version,
}
