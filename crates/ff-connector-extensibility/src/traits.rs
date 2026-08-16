//! ConnectorPlugin trait definition.
//!
//! Defines the combined trait that all VFS connectors must implement. Extends
//! `VfsProvider` (from ff-vfs) and `FileForgePlugin` (from ff-plugin) with
//! connector-specific lifecycle, authentication, and capability methods.
//!
//! ## Implementation Guidance for Future Connectors
//!
//! ### FTP/SFTP Connector
//! - `list` → FTP LIST/NLST commands for directory listing
//! - `read` → FTP RETR for file download
//! - `write` → FTP STOR for file upload
//! - `rename` → FTP RNFR/RNTO for remote rename
//! - `delete` → FTP DELE/RMD for remote delete
//! - `stat` → FTP MLST/SIZE/MDTM for file metadata
//! - `create_directory` → FTP MKD for remote mkdir
//!
//! ### z/OS Connector
//! - `list` → Dataset catalog listing and PDS member listing
//! - `read` → Dataset/member download
//! - `write` → Dataset/member upload
//! - `stat` → DSCB attributes (RECFM, LRECL, BLKSIZE, DSORG)
//! - `custom_operation("jes_spool", ...)` → JES spool access
//! - `custom_operation("submit_job", ...)` → Job submission
//!
//! ### Cloud Connector (SharePoint, OneDrive)
//! - `list` → File/folder listing via Graph API
//! - `read` → File download via Graph API
//! - `write` → File upload via Graph API
//! - `delete` → Trash or permanent delete
//! - `rename` → Rename via Graph API
//! - `stat` → File properties with sharing metadata
//! - `authenticate` → OAuth flow with automatic token refresh

use std::any::Any;

use async_trait::async_trait;

use ff_plugin::FileForgePlugin;
use ff_vfs::VfsProvider;

use crate::api_version::ApiVersion;
use crate::capability::ConnectorCapability;
use crate::credential::CredentialStore;
use crate::descriptor::ConnectorDescriptor;
use crate::error::ConnectorError;
use crate::reconnection::RetryPolicy;
use crate::state::ConnectorState;

/// The combined trait that all VFS connectors must implement.
///
/// Extends `VfsProvider` (file operations) and `FileForgePlugin` (plugin lifecycle)
/// with connector-specific lifecycle, authentication, and capability methods.
///
/// Object-safe — the `ConnectorRegistry` stores connectors as
/// `Box<dyn ConnectorPlugin>`.
///
/// # Thread Safety
///
/// Requires `Send + Sync` for use across thread boundaries.
///
/// # Lifecycle
///
/// 1. Plugin loaded → `FileForgePlugin::initialize()` called
/// 2. Plugin activated → registers with `ConnectorRegistry`
/// 3. `connect()` called → establishes remote connection
/// 4. VFS operations flow through `VfsProvider` methods
/// 5. `disconnect()` called → graceful teardown
/// 6. Plugin deactivated/shutdown → deregistered
///
/// Addresses: Requirement 1 AC 1, AC 3–6
#[async_trait]
pub trait ConnectorPlugin: VfsProvider + FileForgePlugin {
    /// Returns the connector's metadata descriptor.
    ///
    /// Contains: scheme (unique URI scheme identifier), display_name,
    /// description, icon, and version.
    ///
    /// Addresses: Requirement 1 AC 2
    fn descriptor(&self) -> &ConnectorDescriptor;

    /// Returns the complete list of VFS operations this connector supports.
    ///
    /// Required capabilities (Read, List, Metadata) must always be present.
    /// Optional capabilities indicate additional operations the connector supports.
    ///
    /// Addresses: Requirement 1 AC 3
    fn connector_capabilities(&self) -> &[ConnectorCapability];

    /// Returns the VFS core API version this connector was built against.
    ///
    /// Used for compatibility checking at registration time. A connector is
    /// compatible if: same major version AND minor ≤ current.
    ///
    /// Addresses: Requirement 1 AC 4
    fn api_version(&self) -> ApiVersion;

    /// Returns the current connection lifecycle state.
    ///
    /// Addresses: Requirement 4 AC 2
    fn state(&self) -> ConnectorState;

    /// Establish a connection to the remote service.
    ///
    /// Transitions from Registered/Disconnected → Connecting → Connected.
    /// Returns `ConnectorError` if the connection cannot be established.
    ///
    /// # FTP/SFTP
    /// Opens TCP connection and performs protocol handshake.
    ///
    /// # z/OS
    /// Establishes connection to the mainframe via TN3270 or REST API.
    ///
    /// # Cloud
    /// Validates OAuth tokens and establishes API session.
    ///
    /// Addresses: Requirement 1 AC 6, Requirement 4 AC 1
    async fn connect(&mut self) -> Result<(), ConnectorError>;

    /// Gracefully disconnect from the remote service.
    ///
    /// Transitions from Connected → Disconnecting → Disconnected.
    /// In-flight operations should be allowed to complete within a drain period.
    ///
    /// Addresses: Requirement 1 AC 6, Requirement 4 AC 6
    async fn disconnect(&mut self) -> Result<(), ConnectorError>;

    /// Authenticate using credentials from the credential store.
    ///
    /// Called during the connection phase. The connector retrieves its
    /// credentials using a key scoped to its scheme and connection name.
    ///
    /// Addresses: Requirement 5 AC 3
    async fn authenticate(
        &mut self,
        credential_store: &dyn CredentialStore,
    ) -> Result<(), ConnectorError>;

    /// Returns the retry policy for automatic reconnection.
    ///
    /// The platform uses this to determine how many times and how frequently
    /// to attempt reconnection when a connection failure occurs.
    ///
    /// Addresses: Requirement 4 AC 4
    fn retry_policy(&self) -> &RetryPolicy;

    /// Map a provider-specific error into the common ConnectorError taxonomy.
    ///
    /// Connectors implement this to translate their internal error types
    /// into the common `ConnectorError` enum. The platform calls this
    /// rather than expecting connectors to produce `ConnectorError` directly.
    ///
    /// Addresses: Requirement 7 AC 7
    fn map_error(&self, source: Box<dyn std::error::Error + Send + Sync>) -> ConnectorError;

    /// Execute a provider-specific custom operation.
    ///
    /// For operations that don't map to standard VFS methods (e.g., z/OS JES
    /// spool access, job submission). Default implementation returns
    /// `UnsupportedOperation`.
    ///
    /// # z/OS Examples
    /// - `custom_operation("jes_spool", &JesSpoolParams { ... })`
    /// - `custom_operation("submit_job", &JobSubmitParams { ... })`
    ///
    /// Addresses: Requirement 6 AC 3, AC 6
    async fn custom_operation(
        &self,
        name: &str,
        _params: &(dyn Any + Send + Sync),
    ) -> Result<Box<dyn Any + Send>, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation {
            operation: name.to_string(),
            scheme: self.descriptor().scheme.clone(),
            message: format!("custom operation '{name}' not supported"),
        })
    }
}

/// Compile-time assertion that `ConnectorPlugin` is object-safe.
///
/// Addresses: Requirement 1 AC 5
fn _assert_object_safety() {
    fn _accept(_: &dyn ConnectorPlugin) {}
    fn _accept_box(_: Box<dyn ConnectorPlugin>) {}
}

/// Compile-time assertion that `Box<dyn ConnectorPlugin>` is Send + Sync.
fn _assert_send_sync() {
    fn _assert<T: Send + Sync>() {}
    _assert::<Box<dyn ConnectorPlugin>>();
}
