# ff-connector-extensibility

Connector extensibility framework for FileForgeWorkbench VFS connectors.

## Overview

This crate defines the plugin trait and registration framework that all future VFS connectors (FTP/SFTP, z/OS, cloud) must implement to integrate with the Virtual File System layer. It bridges `VfsProvider` (from `ff-vfs`) with `FileForgePlugin` (from `ff-plugin`), adding lifecycle management, capability advertisement, authentication, and error mapping.

## Quick Start: Implementing a New Connector

```rust
use async_trait::async_trait;
use ff_connector_extensibility::*;

struct MyConnector { /* ... */ }

// 1. Implement VfsProvider for file operations
#[async_trait]
impl VfsProvider for MyConnector { /* ... */ }

// 2. Implement FileForgePlugin for plugin lifecycle
impl FileForgePlugin for MyConnector { /* ... */ }

// 3. Implement ConnectorPlugin for connector-specific methods
#[async_trait]
impl ConnectorPlugin for MyConnector {
    fn descriptor(&self) -> &ConnectorDescriptor { /* ... */ }
    fn connector_capabilities(&self) -> &[ConnectorCapability] { /* ... */ }
    fn api_version(&self) -> ApiVersion { CONNECTOR_API_VERSION }
    fn state(&self) -> ConnectorState { /* ... */ }
    async fn connect(&mut self) -> Result<(), ConnectorError> { /* ... */ }
    async fn disconnect(&mut self) -> Result<(), ConnectorError> { /* ... */ }
    async fn authenticate(&mut self, store: &dyn CredentialStore) -> Result<(), ConnectorError> { /* ... */ }
    fn retry_policy(&self) -> &RetryPolicy { /* ... */ }
    fn map_error(&self, source: Box<dyn std::error::Error + Send + Sync>) -> ConnectorError { /* ... */ }
}
```

## Key Types

| Type | Purpose |
|------|---------|
| `ConnectorPlugin` | Combined trait: VfsProvider + FileForgePlugin + connector lifecycle |
| `ConnectorRegistry` | Validates, stores, and manages connector registrations |
| `ConnectorCapability` | Enum of VFS operations a connector can support |
| `ConnectorState` | Lifecycle state machine (Registered → Connected → Disconnected) |
| `ConnectorError` | Structured error with retryable classification |
| `CredentialStore` | Secure credential storage and retrieval interface |
| `RetryPolicy` | Configurable reconnection behaviour (exponential backoff) |
| `ConnectorDescriptor` | Metadata: scheme, display name, version |
| `ApiVersion` | API version for compatibility checking |

## Registration Flow

1. Plugin loaded → `FileForgePlugin::initialize()` receives `PluginContext`
2. Plugin calls `ConnectorRegistry::register(connector)` during activation
3. Registry validates: scheme uniqueness, required capabilities, API version
4. On success, connector available for VFS URI routing
5. User/platform calls `connect()` → connection established
6. VFS operations routed through `VfsProvider` methods
7. Shutdown calls `disconnect()` / `shutdown_all()`

## Required Capabilities

Every connector must declare at minimum: `Read`, `List`, `Metadata`. Registration fails without these.

## Error Format

All errors follow: `[connector:{scheme}] {operation}: {description}`

Example: `[connector:ftp] read: not connected`

## State Machine

```
Registered → Connecting → Connected → Disconnecting → Disconnected
                    ↘ Error ↗ (retry) → Connecting
```
