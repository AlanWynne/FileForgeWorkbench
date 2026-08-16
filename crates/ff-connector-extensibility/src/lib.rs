//! # ff-connector-extensibility — Connector Extensibility Framework
//!
//! This crate defines the **plugin trait and registration framework** that all
//! future VFS connectors (FTP/SFTP, z/OS, cloud) must implement to integrate
//! with the Virtual File System layer of FileForgeWorkbench.
//!
//! ## Architecture Position
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Shell Layer: ff-desktop (egui)                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Consuming crates query ConnectorRegistry for capabilities   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ff-connector-extensibility (THIS CRATE) — Wave 3            │
//! │  Depends on: ff-vfs, ff-plugin, ff-core, ff-logging          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ff-vfs │ ff-plugin │ ff-core │ ff-command │ ff-logging      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Components
//!
//! - [`ConnectorPlugin`] — the combined trait extending `VfsProvider` + `FileForgePlugin`
//! - [`ConnectorRegistry`] — validates, stores, and manages connector registrations
//! - [`ConnectorCapability`] — runtime capability advertisement enum
//! - [`ConnectorState`] — lifecycle state machine (Registered → Connected → Disconnected)
//! - [`ConnectorError`] — structured error type with retryable classification
//! - [`CredentialStore`] — secure credential management interface
//! - [`RetryPolicy`] — configurable reconnection behaviour
//!
//! ## Implementing a Connector
//!
//! Future connectors implement the [`ConnectorPlugin`] trait, which combines:
//! 1. `VfsProvider` — file operations (read, write, list, stat, etc.)
//! 2. `FileForgePlugin` — plugin lifecycle (initialize, activate, shutdown)
//! 3. Connector-specific methods (connect, disconnect, authenticate, etc.)
//!
//! ```rust,ignore
//! #[async_trait]
//! impl ConnectorPlugin for MyFtpConnector {
//!     fn descriptor(&self) -> &ConnectorDescriptor { &self.descriptor }
//!     fn connector_capabilities(&self) -> &[ConnectorCapability] { &self.caps }
//!     fn api_version(&self) -> ApiVersion { CONNECTOR_API_VERSION }
//!     fn state(&self) -> ConnectorState { self.state.clone() }
//!     async fn connect(&mut self) -> Result<(), ConnectorError> { /* ... */ }
//!     async fn disconnect(&mut self) -> Result<(), ConnectorError> { /* ... */ }
//!     // ...
//! }
//! ```
//!
//! ## Registration Flow
//!
//! 1. Plugin loaded → `FileForgePlugin::initialize()` called with `PluginContext`
//! 2. Plugin calls `ConnectorRegistry::register(self)` during activation
//! 3. Registry validates: scheme uniqueness, required capabilities, API version
//! 4. On success, connector is available for VFS routing
//! 5. Platform calls `connect()` when the user activates the connector
//! 6. VFS operations flow through `VfsProvider` methods
//! 7. Platform calls `disconnect()` / `shutdown_all()` on shutdown

pub mod api_version;
pub mod capability;
pub mod credential;
pub mod custom_op;
pub mod descriptor;
pub mod error;
pub mod event;
pub mod reconnection;
pub mod registry;
pub mod state;
pub mod traits;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use api_version::{ApiVersion, CONNECTOR_API_VERSION};
pub use capability::{validate_capabilities, ConnectorCapability, REQUIRED_CAPABILITIES};
pub use credential::{Credential, CredentialStore, SecureBytes, SecureString};
pub use custom_op::{CustomOperationRequest, CustomOperationResponse};
pub use descriptor::ConnectorDescriptor;
pub use error::ConnectorError;
pub use event::{
    ConnectorCapabilityChangedEvent, ConnectorRegisteredEvent, ConnectorStateChangedEvent,
};
pub use reconnection::{ReconnectionManager, RetryPolicy};
pub use registry::ConnectorRegistry;
pub use state::{is_valid_transition, ConnectorState};
pub use traits::ConnectorPlugin;
