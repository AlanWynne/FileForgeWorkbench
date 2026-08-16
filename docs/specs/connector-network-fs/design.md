# Design Document: Network Filesystem Connector (`ff-connector-network-fs`)

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This is a placeholder design documenting future integration points only.
> No implementation tasks will be created until this connector moves to active development.
> The `ff-connector-extensibility` crate (shipping in the initial release) provides the
> trait infrastructure this connector will plug into.

---

## 1. Overview

The `ff-connector-network-fs` crate will provide VFS access to network filesystems:
Windows UNC paths (`\\server\share`), SMB/CIFS shares, and NFS mounts. It integrates
with the workbench through the established connector extensibility framework — no
changes to VFS core or platform infrastructure will be required.

### What This Connector Will Do

| Protocol | Description |
|----------|-------------|
| **UNC/SMB** | Access `\\server\share\path` resources, browse shares, CRUD operations over SMBv2/v3 |
| **NFS** | Mount and access NFS-exported directories (NFSv3, NFSv4) with UID/GID mapping |
| **Mapped Drives** | Resolve Windows drive letters (e.g., `Z:\`) to their underlying UNC paths |

### Deferred Scope Rationale

Network filesystem access requires platform-specific system APIs (Win32 `WNetGetConnection`,
`ReadDirectoryChangesW`) and protocol libraries (SMB client, NFS client) that add
significant dependency weight and testing surface. These are deferred until the core
workbench editor, VFS, and local-fs connector are stable.

---

## 2. VFS URI Schemes

The connector will register the following URI schemes with the `ConnectorRegistry`:

| Scheme | Example URI | Usage |
|--------|-------------|-------|
| `smb` | `smb://server/share/path/file.txt` | SMB/CIFS shares (cross-platform) |
| `nfs` | `nfs://server/export/path/file.txt` | NFS mounts |
| `unc` | `unc://server/share/path/file.txt` | Windows UNC path shorthand |

Mapped drive paths (e.g., `Z:\docs\file.txt`) will be resolved to their underlying
`smb://` or `unc://` URI transparently — the VFS will not expose a separate scheme
for mapped drives.

---

## 3. Integration with ConnectorPlugin Trait

This connector implements the `ConnectorPlugin` trait from `ff-connector-extensibility`:

```rust
// Future: crates/ff-connector-network-fs/src/connector.rs

#[async_trait::async_trait]
impl ConnectorPlugin for NetworkFsConnector {
    fn descriptor(&self) -> &ConnectorDescriptor { /* scheme: "smb" or "nfs" */ }
    fn connector_capabilities(&self) -> &[ConnectorCapability] { /* Read, Write, List, Metadata, Watch, ... */ }
    fn api_version(&self) -> ApiVersion { CONNECTOR_API_VERSION }
    fn state(&self) -> ConnectorState { /* ... */ }
    async fn connect(&mut self) -> Result<(), ConnectorError> { /* SMB session / NFS mount */ }
    async fn disconnect(&mut self) -> Result<(), ConnectorError> { /* ... */ }
    async fn authenticate(&mut self, cred_store: &dyn CredentialStore) -> Result<(), ConnectorError> { /* ... */ }
    fn retry_policy(&self) -> &RetryPolicy { /* network-tuned policy */ }
    fn map_error(&self, source: Box<dyn std::error::Error + Send + Sync>) -> ConnectorError { /* ... */ }
}
```

The connector also implements:
- **`VfsProvider`** (from `ff-vfs`) — for `read`, `write`, `list`, `stat`, `watch`, `delete`, `rename`, `create_dir`
- **`FileForgePlugin`** (from `ff-plugin`) — for plugin lifecycle (`initialize`, `activate`, `deactivate`, `shutdown`)

### Registration Flow

```
1. Platform loads plugin via ff-plugin discovery
2. NetworkFsConnector::initialize() obtains PluginContext
3. Connector calls ConnectorRegistry::register(Box::new(self))
4. Registry validates: scheme uniqueness, required capabilities, API version
5. Registry forwards to VFS ProviderRegistry → scheme is routable
6. ConnectorRegisteredEvent emitted on EventBus
```

---

## 4. Integration Points with Upstream Crates

| Upstream Crate | Dependency Direction | API Consumed |
|----------------|---------------------|--------------|
| `ff-vfs` | depends on | `VfsProvider`, `ResourceUri`, `ProviderRegistry`, `VfsCapabilities` |
| `ff-connector-extensibility` | depends on | `ConnectorPlugin`, `ConnectorRegistry`, `ConnectorCapability`, `ConnectorState`, `ConnectorError`, `CredentialStore`, `RetryPolicy` |
| `ff-plugin` | depends on | `FileForgePlugin`, `PluginContext`, `PluginMetadata` |
| `ff-logging` | depends on | Structured logging macros |
| `ff-core` | indirect (via registry) | `EventBus` for state-change events |

---

## 5. Placeholder Module Structure

```
crates/ff-connector-network-fs/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Crate root, public re-exports
│   ├── connector.rs        # NetworkFsConnector: ConnectorPlugin impl
│   ├── smb/
│   │   ├── mod.rs          # SMB sub-module re-exports
│   │   ├── client.rs       # SMB session management (SMBv2/v3)
│   │   ├── provider.rs     # VfsProvider impl for SMB paths
│   │   └── watcher.rs      # ReadDirectoryChangesW / polling
│   ├── nfs/
│   │   ├── mod.rs          # NFS sub-module re-exports
│   │   ├── client.rs       # NFS client (NFSv3/v4)
│   │   ├── provider.rs     # VfsProvider impl for NFS paths
│   │   └── watcher.rs      # Polling-based change detection
│   ├── unc.rs              # UNC path parsing and normalisation
│   ├── mapped_drives.rs    # Windows mapped drive resolution (platform-gated)
│   └── error.rs            # Protocol-specific → ConnectorError mapping
└── tests/
    ├── smb_tests.rs
    ├── nfs_tests.rs
    ├── unc_tests.rs
    └── mapped_drive_tests.rs
```

---

## 6. Key Types (Planned)

```rust
/// The primary connector type. May be instantiated once per protocol (SMB vs NFS)
/// or as a single multi-protocol connector — TBD during active development.
pub struct NetworkFsConnector {
    descriptor: ConnectorDescriptor,
    state: ConnectorState,
    config: NetworkFsConfig,
    smb_client: Option<SmbClient>,
    nfs_client: Option<NfsClient>,
}

/// Configuration for network filesystem connections.
pub struct NetworkFsConfig {
    /// Server hostname or IP
    pub host: String,
    /// Share or export path
    pub share: String,
    /// Protocol preference (SMB, NFS, or Auto-detect)
    pub protocol: NetworkProtocol,
    /// Connection timeout
    pub connect_timeout: std::time::Duration,
    /// Watch polling interval for protocols without push notifications
    pub watch_poll_interval: std::time::Duration,
}

/// Supported network protocols.
#[non_exhaustive]
pub enum NetworkProtocol {
    Smb,
    Nfs,
    /// Auto-detect based on URI scheme or platform heuristics
    Auto,
}

/// Parsed and normalised UNC path components.
pub struct UncPath {
    pub server: String,
    pub share: String,
    pub path: Vec<String>,
}
```

---

## 7. Capability Advertisement

When registered, the connector will advertise these capabilities:

| Capability | SMB | NFS | Notes |
|-----------|-----|-----|-------|
| `Read` | ✅ | ✅ | Required |
| `Write` | ✅ | ✅ | |
| `List` | ✅ | ✅ | Required |
| `Metadata` | ✅ | ✅ | Required |
| `Watch` | conditional | conditional | Advertised only if protocol/server supports it |
| `Rename` | ✅ | ✅ | |
| `Delete` | ✅ | ✅ | |
| `CreateDirectory` | ✅ | ✅ | |
| `Search` | ❌ | ❌ | Not natively supported; deferred to VFS-level search |
| `Copy` | ✅ | ❌ | SMB supports server-side copy; NFS requires read+write |

---

## 8. Open Questions (For Future Active Development)

1. **Single connector or per-protocol?** Should `NetworkFsConnector` register once and handle
   both `smb://` and `nfs://`, or should there be separate `SmbConnector` and `NfsConnector`
   types each registering their own scheme?

2. **SMB library choice** — pure Rust (`pavao`, `smb-rs`) vs. FFI to system `libsmbclient`?

3. **NFS library choice** — pure Rust NFS client vs. relying on OS-level mount and local-fs access?

4. **Credential scoping** — one credential per server, or per share? How does this interact
   with the `CredentialStore` key naming convention?

5. **Windows integration depth** — should the connector hook into Windows Credential Manager
   for SSO/Kerberos ticket reuse, or require explicit credential entry?
