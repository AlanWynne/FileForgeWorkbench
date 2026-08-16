# Requirements Document

## Introduction

This feature specifies the connector extensibility framework for FileForgeWorkbench (`ff-connector-extensibility` crate). It defines the plugin trait, registration protocol, capability advertisement, and lifecycle management that future VFS connectors (FTP, SFTP, z/OS, cloud) must implement to integrate with the Virtual File System layer.

The connector extensibility crate extends the `VfsProvider` trait defined by `ff-vfs` with additional lifecycle, authentication, and capability-declaration methods. It provides the hook that ensures future remote connectors can be added without architectural changes — new providers register themselves at plugin initialization time, advertise their capabilities, and manage their own connection lifecycle. The workbench core never depends on a specific connector implementation; it discovers and interacts with connectors exclusively through the traits defined here.

**Initial release scope:** This crate defines the traits and framework only. No remote connector implementations ship in the initial release. The local filesystem provider (`ff-connector-local-fs`) and dataset catalog (`ff-dataset-catalog`) provide all content access for the first release. Remote connectors (FTP/SFTP, z/OS, cloud) are deferred to future phases and will implement the traits defined here.

**Source references:**
- **WB** = Workbench Architecture Brief (VFS extensibility, FFW-ARCH-001)
- **FFW** = FileForgeWorkbench cross-cutting Requirement 1 (VFS Principle), Requirement 3 (Plugin Architecture Principle)
- **DSC** = Dataset Catalog Brief (provider extensibility concept)

## Glossary

- **ConnectorPlugin**: The trait extending `FileForgePlugin` and `VfsProvider` with additional lifecycle, authentication, and capability-advertisement methods that all VFS connectors must implement. [WB]
- **ConnectorDescriptor**: Metadata identifying a connector: URI scheme, display name, description, icon identifier, and version. [WB]
- **ConnectorCapability**: An enumeration of operations a connector supports (read, write, watch, search, rename, delete, create directory, metadata). [WB]
- **ConnectorState**: The lifecycle state of a connector: Registered → Connecting → Connected → Disconnecting → Disconnected → Error. [WB]
- **ConnectorRegistry**: The subsystem within the VFS provider registry responsible for validating, storing, and managing connector registrations. [WB]
- **CredentialStore**: A provider-agnostic interface for securely storing, retrieving, and refreshing authentication credentials. [WB]
- **AuthenticationFlow**: A protocol-specific authentication sequence (password, key-based, OAuth, token) that a connector executes to establish a connection. [WB]
- **RetryPolicy**: A configurable strategy governing automatic reconnection attempts (max retries, backoff interval, jitter). [WB]
- **CapabilityQuery**: A runtime request to determine whether a specific connector supports a given operation before attempting it. [WB]
- **ConnectorError**: A structured error type that maps provider-specific errors to common VFS error categories with additional diagnostic context. [WB]

## Requirements

### Requirement 1: Connector Plugin Trait

**User Story:** As a connector developer, I want a well-defined trait that extends VfsProvider with lifecycle and registration methods, so that I can implement a new remote connector without modifying the VFS core or any consuming code.

**Source:** WB Architecture Brief (VFS extensibility), FFW-ARCH-001 AC 3–4. [WB]

#### Acceptance Criteria

1. THE `ff-connector-extensibility` crate SHALL define a `ConnectorPlugin` trait that requires implementors to also implement `VfsProvider` (from `ff-vfs`) and `FileForgePlugin` (from `ff-plugin`), combining VFS operations with plugin lifecycle and connector-specific methods.
2. THE `ConnectorPlugin` trait SHALL define a `descriptor(&self) -> &ConnectorDescriptor` method returning the connector's metadata, where `ConnectorDescriptor` contains: `scheme` (unique URI scheme identifier, e.g., "ftp", "sftp", "zos"), `display_name` (human-readable name), `description` (one-line summary), `icon` (optional icon identifier), and `version` (semver-compatible version of the connector).
3. THE `ConnectorPlugin` trait SHALL define a `capabilities(&self) -> &[ConnectorCapability]` method returning the complete list of VFS operations this connector supports.
4. THE `ConnectorPlugin` trait SHALL define an `api_version(&self) -> ApiVersion` method returning the VFS core API version the connector was built against, enabling compatibility checks at registration time.
5. THE `ConnectorPlugin` trait SHALL be object-safe, allowing the `ConnectorRegistry` to store connectors as trait objects (`Box<dyn ConnectorPlugin>`).
6. THE `ConnectorPlugin` trait SHALL define `connect(&mut self) -> Result<(), ConnectorError>` and `disconnect(&mut self) -> Result<(), ConnectorError>` methods for connection lifecycle management separate from the VFS read/write operations.

---

### Requirement 2: Provider Registration

**User Story:** As a platform developer, I want connectors to register themselves during plugin initialization with validation of scheme uniqueness and API compatibility, so that the platform discovers available providers automatically and rejects misconfigured ones early.

**Source:** WB Architecture Brief (VFS extensibility), FFW-ARCH-001 AC 3. [WB]

#### Acceptance Criteria

1. WHEN a connector plugin's `initialize` method is called, THE connector SHALL register itself with the `ConnectorRegistry` via the `PluginContext`, providing its `ConnectorDescriptor` and capability list.
2. THE `ConnectorRegistry` SHALL validate registration by checking: (a) the connector's `scheme` is unique — no other registered connector uses the same scheme, (b) the connector declares at least one valid `ConnectorCapability`, and (c) the connector's `api_version` is compatible with the current VFS core API version (same major version, minor version ≤ current).
3. IF registration validation fails (duplicate scheme, no capabilities, or incompatible API version), THEN THE `ConnectorRegistry` SHALL reject the registration, return a `ConnectorError::RegistrationFailed` with a description of the failure reason, and log an ERROR-level record.
4. WHEN a connector plugin is unloaded or its `shutdown` lifecycle method is called, THE `ConnectorRegistry` SHALL deregister the connector — removing its scheme from the registry and making it unavailable for new operations.
5. THE `ConnectorRegistry` SHALL support hot-swap of a connector implementation: when a new version of a connector registers with the same scheme while the old version is still registered, THE registry SHALL deactivate the old connector (calling `disconnect` if connected), deregister it, register the new version, and preserve any resource URIs that referenced the old provider so they resolve against the new provider without consumer code changes.
6. THE `ConnectorRegistry` SHALL emit a platform event via the event bus whenever a connector is registered or deregistered, including the connector's scheme and display name in the event payload.
7. THE `ConnectorRegistry` SHALL provide a query method `get_connector(scheme: &str) -> Option<&dyn ConnectorPlugin>` for the VFS core to resolve a URI scheme to its backing connector at runtime.

---

### Requirement 3: Capability Advertisement

**User Story:** As a consuming subsystem, I want to query what operations a connector supports before attempting them, so that I can provide appropriate UI affordances and avoid operations that would fail.

**Source:** WB Architecture Brief (VFS extensibility). [WB]

#### Acceptance Criteria

1. THE `ConnectorCapability` enum SHALL define the following variants representing VFS operations: `Read`, `Write`, `Watch`, `Search`, `Rename`, `Delete`, `CreateDirectory`, `Metadata`, `List`, and `Copy`, with `#[non_exhaustive]` to allow future additions without breaking changes.
2. THE `ConnectorPlugin` trait SHALL distinguish between required and optional capabilities — `Read`, `List`, and `Metadata` SHALL be required (a connector that does not support these SHALL fail registration validation), while all other capabilities SHALL be optional.
3. THE `ConnectorRegistry` SHALL provide a method `supports(scheme: &str, capability: ConnectorCapability) -> bool` that allows consumers to check whether a specific connector supports a given operation before attempting it.
4. WHEN a consumer attempts an operation on a resource whose connector does not advertise the required capability, THE VFS layer SHALL return a `ConnectorError::UnsupportedOperation` containing the operation name, the scheme, and a human-readable message explaining the limitation — the operation SHALL NOT panic or produce an untyped error.
5. THE `ConnectorRegistry` SHALL provide a method `capabilities_for(scheme: &str) -> Option<&[ConnectorCapability]>` returning the full capability list for a registered connector, enabling UI code to dynamically show or hide actions based on what the provider supports.
6. WHEN a connector's capabilities change (e.g., after reconnection with different permissions), THE connector MAY call a `refresh_capabilities` method on the registry to update its advertised capabilities, and THE registry SHALL emit a capability-change event.

---

### Requirement 4: Provider Lifecycle

**User Story:** As a platform developer, I want connectors to transition through well-defined connection states with configurable reconnection logic and event notifications, so that the platform handles network-based providers gracefully and keeps consumers informed of connectivity changes.

**Source:** WB Architecture Brief (VFS extensibility). [WB]

#### Acceptance Criteria

1. THE `ConnectorState` enum SHALL define the following states: `Registered`, `Connecting`, `Connected`, `Disconnecting`, `Disconnected`, and `Error(ConnectorError)`, where the error variant carries the cause of the failure.
2. THE `ConnectorPlugin` trait SHALL define a `state(&self) -> ConnectorState` method allowing the platform and consumers to query the connector's current connection state at any time.
3. WHEN a connector transitions between states, THE connector SHALL notify the platform via the event bus with a `ConnectorStateChanged` event containing the connector scheme, the previous state, and the new state.
4. THE `ConnectorPlugin` trait SHALL define a `retry_policy(&self) -> &RetryPolicy` method, where `RetryPolicy` specifies: `max_retries` (u32, 0 = no retry), `initial_backoff` (Duration), `max_backoff` (Duration), and `use_jitter` (bool).
5. WHEN a connector in the `Connected` state encounters a connection failure, THE platform SHALL transition the connector to the `Error` state, and IF the connector's `RetryPolicy` allows retries, THE platform SHALL automatically attempt reconnection using exponential backoff with optional jitter, transitioning through `Connecting` on each attempt.
6. WHEN the application is shutting down, THE platform SHALL transition all connected connectors to `Disconnecting`, call their `disconnect` method, and allow a configurable drain period (default 5 seconds) for in-flight operations to complete before forcibly dropping the connector.
7. WHEN a connector is in the `Disconnected` or `Error` state, ANY VFS operation directed at that connector's resources SHALL return a `ConnectorError::NotConnected` immediately rather than blocking or queuing.
8. THE platform SHALL support explicit user-initiated connect and disconnect operations, allowing consumers to trigger `connect()` or `disconnect()` on a connector through the `ConnectorRegistry` interface.

---

### Requirement 5: Authentication Framework

**User Story:** As a connector developer, I want a credential storage interface and authentication flow support, so that my connector can authenticate with remote services securely without implementing credential management from scratch.

**Source:** WB Architecture Brief (VFS extensibility). [WB]

#### Acceptance Criteria

1. THE `ff-connector-extensibility` crate SHALL define a `CredentialStore` trait with methods: `store(key: &str, credential: &Credential) -> Result<(), ConnectorError>`, `retrieve(key: &str) -> Result<Option<Credential>, ConnectorError>`, `delete(key: &str) -> Result<(), ConnectorError>`, and `exists(key: &str) -> bool`.
2. THE `Credential` type SHALL be an enum supporting the following authentication methods: `Password { username: String, password: SecureString }`, `KeyBased { username: String, private_key: SecureBytes, passphrase: Option<SecureString> }`, `OAuth { access_token: SecureString, refresh_token: Option<SecureString>, expires_at: Option<SystemTime> }`, and `Token { token: SecureString }`.
3. THE `ConnectorPlugin` trait SHALL define an `authenticate(&mut self, credential_store: &dyn CredentialStore) -> Result<(), ConnectorError>` method that a connector calls during the connection phase to retrieve and apply credentials.
4. WHEN a credential has an expiration time (OAuth tokens), THE platform SHALL provide a `refresh_credential(key: &str) -> Result<Credential, ConnectorError>` method on the `CredentialStore` that connectors can call to trigger renewal, and THE connector SHALL call this method when an operation fails due to an expired token.
5. THE `CredentialStore` implementation SHALL guarantee that credentials are never logged in plaintext — log records SHALL mask credential values, displaying only the credential type and key name.
6. THE `CredentialStore` implementation SHALL use secure memory handling: `SecureString` and `SecureBytes` types SHALL overwrite their backing memory on drop (zeroize) to prevent credentials from lingering in freed memory.
7. THE `CredentialStore` SHALL scope credentials by connector scheme and connection name — credentials for connector "sftp" connection "prod-server" SHALL NOT be accessible to connector "ftp" connection "dev-server".

---

### Requirement 6: Future Connector Hooks

**User Story:** As an architect, I want the connector trait to document what operations future connectors (FTP/SFTP, z/OS, cloud) will implement, so that the extensibility framework is validated against real use cases and future implementors have clear guidance.

**Source:** WB Architecture Brief (VFS extensibility, deferred connectivity). [WB]

#### Acceptance Criteria

1. THE `ConnectorPlugin` trait SHALL define methods that map directly to `VfsProvider` trait methods, ensuring every VFS operation (open, read, write, list, stat, watch, search, rename, delete, create_directory, copy) has a corresponding method that connectors can implement or return `UnsupportedOperation` for.
2. FOR the FTP/SFTP connector use case, THE trait SHALL support the following operation mapping: `list` → directory listing, `read` → file download, `write` → file upload, `rename` → remote rename, `delete` → remote delete, `stat` → file metadata retrieval, `create_directory` → remote mkdir — these operations SHALL be expressible through the `ConnectorPlugin` + `VfsProvider` trait combination without additional methods.
3. FOR the z/OS connector use case, THE trait SHALL support: `list` → dataset catalog listing and PDS member listing, `read` → dataset/member download, `write` → dataset/member upload, `stat` → dataset attributes (RECFM, LRECL, BLKSIZE, DSORG), and additionally expose a `custom_operation(name: &str, params: &dyn Any) -> Result<Box<dyn Any>, ConnectorError>` escape hatch for z/OS-specific operations (JES spool access, job submission) that do not map to standard VFS methods.
4. FOR the cloud connector use case (SharePoint, OneDrive), THE trait SHALL support: `list` → file/folder listing, `read` → file download, `write` → file upload, `delete` → file/folder delete, `rename` → rename, `stat` → file properties with sharing metadata, and authentication via the OAuth `Credential` variant with automatic token refresh.
5. THE `ConnectorPlugin` trait SHALL include documentation (doc comments) on each method describing the expected mapping for FTP/SFTP, z/OS, and cloud use cases, serving as implementation guidance for future connector authors.
6. THE `custom_operation` method SHALL be optional — connectors that have no provider-specific operations beyond standard VFS methods SHALL return `ConnectorError::UnsupportedOperation` from this method.

---

### Requirement 7: Error Mapping

**User Story:** As a consuming subsystem, I want provider-specific errors mapped to common VFS error types with additional diagnostic context, so that I can display meaningful error messages and decide whether to retry the operation.

**Source:** WB Architecture Brief (VFS extensibility), FFW cross-cutting Req 8 (Error Message Standards). [WB]

#### Acceptance Criteria

1. THE `ConnectorError` enum SHALL define the following common error categories: `NotConnected`, `AuthenticationFailed`, `PermissionDenied`, `ResourceNotFound`, `ResourceAlreadyExists`, `Timeout`, `NetworkError`, `UnsupportedOperation`, `RegistrationFailed`, `ProviderSpecific`, and `Internal`, each carrying a descriptive message string and optional source error.
2. THE `ConnectorError` enum SHALL include a `is_retryable(&self) -> bool` method that classifies each error category as retryable or non-retryable: `Timeout` and `NetworkError` SHALL be retryable; `AuthenticationFailed`, `PermissionDenied`, `ResourceNotFound`, `ResourceAlreadyExists`, `UnsupportedOperation`, and `RegistrationFailed` SHALL NOT be retryable; `NotConnected` SHALL be retryable only if the connector's `RetryPolicy` allows reconnection.
3. THE `ConnectorError` SHALL implement `From<std::io::Error>` to map standard I/O errors to appropriate connector error categories (e.g., `io::ErrorKind::PermissionDenied` → `ConnectorError::PermissionDenied`, `io::ErrorKind::NotFound` → `ConnectorError::ResourceNotFound`, `io::ErrorKind::TimedOut` → `ConnectorError::Timeout`).
4. WHEN a network-based connector encounters a connection failure, THE error SHALL include additional context: the remote host/port (if available), the duration elapsed before timeout, the number of retry attempts already made, and the underlying OS-level error code — all formatted within the 200-character message limit defined by FFW cross-cutting Req 8.
5. THE `ConnectorError` SHALL implement `std::error::Error` with a proper `source()` chain, preserving the original provider-specific error for diagnostic purposes while exposing a common error type to consumers.
6. THE `ConnectorError` SHALL implement `Display` following the workbench error format: `[connector:{scheme}] {operation}: {description}`, where `{scheme}` is the connector's URI scheme, `{operation}` is the VFS operation that failed, and `{description}` is the human-readable error message.
7. THE `ConnectorPlugin` trait SHALL define an `map_error(&self, source: Box<dyn std::error::Error>) -> ConnectorError` method allowing connector implementations to translate their internal error types into the common `ConnectorError` taxonomy, providing a consistent mapping point that the platform calls rather than expecting connectors to produce `ConnectorError` directly from every internal operation.
