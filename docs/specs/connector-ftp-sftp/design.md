# Design Document: FTP/FTPS/SFTP Connector (`ff-connector-ftp-sftp`)

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This is a placeholder design documenting future integration points only.
> No implementation tasks will be created for this connector until it moves
> to active development. The extensibility framework it depends on
> (`ff-connector-extensibility`) ships in the initial release.

---

## 1. Overview

The `ff-connector-ftp-sftp` crate will provide VFS connectors for remote file
access over FTP (plain), FTPS (FTP over TLS), and SFTP (SSH File Transfer
Protocol). It will implement the `ConnectorPlugin` trait from
`ff-connector-extensibility`, which combines `VfsProvider` (from `ff-vfs`) with
connector lifecycle management, authentication, capability advertisement, and
error mapping.

### What This Connector Will Do

| Protocol | Transport | Auth Methods |
|----------|-----------|--------------|
| FTP      | TCP (plain) | Username/password |
| FTPS     | TLS (implicit or explicit STARTTLS) | Username/password, client certificate |
| SFTP     | SSH | Password, RSA/Ed25519/ECDSA key, SSH agent |

All three protocols will present a unified VFS interface — consumers interact
through standard `VfsProvider` operations without protocol awareness.

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
├─────────────────────────────────────────────────────────────┤
│  ff-file-tree-panel / ff-file-operations (consumers)         │
├─────────────────────────────────────────────────────────────┤
│  ff-vfs — routes vfs://ftp/… and vfs://sftp/… to provider    │
├─────────────────────────────────────────────────────────────┤
│  ff-connector-ftp-sftp (THIS CRATE — DEFERRED)               │
│  Implements: ConnectorPlugin trait                            │
├─────────────────────────────────────────────────────────────┤
│  ff-connector-extensibility — ConnectorPlugin, Registry      │
│  ff-vfs — VfsProvider trait                                   │
│  ff-plugin — FileForgePlugin lifecycle                        │
├─────────────────────────────────────────────────────────────┤
│  ff-logging (Wave 0)                                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. VFS URI Schemes

The connector will register three URI schemes with the `ConnectorRegistry`:

| Scheme | Example URI | Description |
|--------|-------------|-------------|
| `ftp`  | `vfs://ftp/myserver:21/path/to/file.txt` | Plain FTP |
| `ftps` | `vfs://ftps/myserver:990/path/to/file.txt` | FTP over TLS |
| `sftp` | `vfs://sftp/myserver:22/path/to/file.txt` | SSH File Transfer |

URI structure: `vfs://{scheme}/{host}:{port}/{path}`

Each scheme is registered as a separate `ConnectorDescriptor` so the
`ConnectorRegistry` can manage them independently (different connection states,
different capabilities, different retry policies).

---

## 3. Integration Points with Upstream Crates

### 3.1 `ff-vfs` (VfsProvider trait)

- Implements `VfsProvider` for all three protocols
- Operations: `read`, `write`, `list`, `stat`, `rename`, `delete`, `create_dir`, `watch` (polling)
- Registered with `ProviderRegistry` via `ConnectorRegistry::register()`
- Maps remote file metadata to `VfsMetadata` (size, mtime, permissions)

### 3.2 `ff-connector-extensibility` (ConnectorPlugin trait)

- Implements `ConnectorPlugin` combining VfsProvider + FileForgePlugin + connector lifecycle
- Declares `ConnectorDescriptor` for each scheme (ftp, ftps, sftp)
- Advertises `ConnectorCapability` set per protocol:
  - **FTP/FTPS**: Read, Write, List, Metadata, Rename, Delete, CreateDirectory
  - **SFTP**: Read, Write, List, Metadata, Rename, Delete, CreateDirectory, Watch (polling)
- Implements `ConnectorState` lifecycle transitions (Registered → Connecting → Connected → …)
- Implements `authenticate()` consuming credentials from `CredentialStore`
- Implements `map_error()` translating protocol errors to `ConnectorError` taxonomy
- Declares `RetryPolicy` for reconnection on transient failures
- Declares `ApiVersion` compatibility with the extensibility framework

### 3.3 `ff-plugin` (FileForgePlugin lifecycle)

- Implements `FileForgePlugin` for plugin initialization and shutdown
- During `initialize()`: obtains `PluginContext`, registers with `ConnectorRegistry`
- During `shutdown()`: gracefully disconnects all active sessions
- Advertises `Capability::Providers` with the `CapabilityRegistry`

### 3.4 `ff-logging`

- Uses structured logging (`ff-logging`) for connection events, auth attempts, transfer progress
- Log targets: `ff_connector_ftp_sftp::ftp`, `ff_connector_ftp_sftp::sftp`

---

## 4. Placeholder Module Structure

```
crates/ff-connector-ftp-sftp/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API, plugin registration entry point
│   ├── ftp/
│   │   ├── mod.rs              # FTP connector implementation
│   │   ├── connection.rs       # TCP connection management, active/passive mode
│   │   ├── commands.rs         # FTP command encoding/parsing (LIST, RETR, STOR, etc.)
│   │   └── tls.rs             # FTPS TLS negotiation (implicit + explicit STARTTLS)
│   ├── sftp/
│   │   ├── mod.rs              # SFTP connector implementation
│   │   ├── session.rs          # SSH session and channel management
│   │   ├── auth.rs             # Key-based, password, and agent authentication
│   │   └── known_hosts.rs      # Known-hosts verification (strict / TOFU)
│   ├── common/
│   │   ├── mod.rs              # Shared utilities
│   │   ├── provider.rs         # VfsProvider implementation (delegates to ftp/sftp)
│   │   ├── capabilities.rs     # Capability advertisement per protocol
│   │   ├── error_mapping.rs    # Protocol error → ConnectorError mapping
│   │   └── transfer.rs         # Transfer buffering, progress reporting, resume
│   └── config.rs               # Connection configuration types
└── tests/
    ├── ftp_integration.rs      # FTP protocol integration tests
    ├── sftp_integration.rs     # SFTP protocol integration tests
    ├── error_mapping_tests.rs  # Error taxonomy mapping property tests
    └── capability_tests.rs     # Capability advertisement tests
```

---

## 5. Key Types (Planned)

### Connection Configuration

```rust
/// Configuration for establishing an FTP/FTPS connection.
pub struct FtpConnectionConfig {
    pub host: String,
    pub port: u16,
    pub tls_mode: TlsMode,
    pub transfer_mode: TransferMode,
    pub passive_mode: bool,
    pub timeout: std::time::Duration,
    pub keepalive_interval: Option<std::time::Duration>,
    pub buffer_size: usize,
}

/// TLS mode for FTPS connections.
pub enum TlsMode {
    /// No TLS (plain FTP)
    None,
    /// Explicit TLS via STARTTLS on port 21
    Explicit,
    /// Implicit TLS on dedicated port (typically 990)
    Implicit,
}

/// FTP data transfer mode.
pub enum TransferMode {
    Binary,
    Ascii,
}
```

### SFTP Configuration

```rust
/// Configuration for establishing an SFTP connection.
pub struct SftpConnectionConfig {
    pub host: String,
    pub port: u16,
    pub auth_method: SftpAuthMethod,
    pub known_hosts_policy: KnownHostsPolicy,
    pub keepalive_interval: Option<std::time::Duration>,
    pub buffer_size: usize,
}

/// SFTP authentication method selection.
pub enum SftpAuthMethod {
    /// Use SSH agent for key management
    Agent,
    /// Explicit private key file
    KeyFile {
        path: std::path::PathBuf,
        passphrase_key: Option<String>, // key into CredentialStore
    },
    /// Password authentication (fallback)
    Password,
}

/// Known-hosts verification policy.
pub enum KnownHostsPolicy {
    /// Strict — reject unknown or changed host keys
    Strict,
    /// Trust On First Use — accept new keys, reject changed keys
    Tofu,
    /// Accept all keys (insecure, for testing only)
    AcceptAll,
}
```

### Connector Instances

```rust
/// The FTP/FTPS connector plugin instance.
/// Implements ConnectorPlugin (and thus VfsProvider + FileForgePlugin).
pub struct FtpConnector {
    config: FtpConnectionConfig,
    state: ConnectorState,
    descriptor: ConnectorDescriptor,
    retry_policy: RetryPolicy,
    // ... internal connection handle
}

/// The SFTP connector plugin instance.
/// Implements ConnectorPlugin (and thus VfsProvider + FileForgePlugin).
pub struct SftpConnector {
    config: SftpConnectionConfig,
    state: ConnectorState,
    descriptor: ConnectorDescriptor,
    retry_policy: RetryPolicy,
    // ... internal SSH session handle
}
```

---

## 6. Authentication Flows

### FTP/FTPS Authentication

```
1. Connector enters Connecting state
2. TCP connection established (with TLS handshake if FTPS)
3. Connector calls authenticate(credential_store)
4. Retrieves Credential::Password { username, password } from store
   using key "ftp:{host}:{port}"
5. Sends USER + PASS commands
6. On 230 response → Connected state
7. On 530 response → ConnectorError::AuthenticationFailed
```

### SFTP Key-Based Authentication

```
1. Connector enters Connecting state
2. SSH TCP connection established
3. Known-hosts check (per KnownHostsPolicy)
4. Connector calls authenticate(credential_store)
5. Depending on SftpAuthMethod:
   a. Agent → queries SSH agent for matching key
   b. KeyFile → retrieves Credential::KeyBased from store
      using key "sftp:{host}:{port}"
   c. Password → retrieves Credential::Password from store
6. SSH userauth attempt
7. On success → opens SFTP subsystem channel → Connected state
8. On failure → ConnectorError::AuthenticationFailed
```

### SFTP SSH Agent Flow

```
1. Connector queries platform SSH agent (ssh-agent on Unix, Pageant on Windows)
2. Agent provides list of available identities
3. Connector attempts authentication with each identity until one succeeds
4. If no identity works → falls back to password if configured
5. If no fallback → ConnectorError::AuthenticationFailed
```

---

## 7. Error Mapping Strategy

| Protocol Error | ConnectorError Variant |
|---------------|----------------------|
| FTP 421 (service unavailable) | `NetworkError` (retryable) |
| FTP 425/426 (data connection failure) | `NetworkError` (retryable) |
| FTP 530 (not logged in) | `AuthenticationFailed` |
| FTP 550 (file not found) | `ResourceNotFound` |
| FTP 553 (permission denied) | `PermissionDenied` |
| SSH connection refused | `NetworkError` (retryable) |
| SSH auth failure | `AuthenticationFailed` |
| SSH host key mismatch | `AuthenticationFailed` |
| SFTP no such file | `ResourceNotFound` |
| SFTP permission denied | `PermissionDenied` |
| Connection timeout | `Timeout` (retryable) |
| DNS resolution failure | `NetworkError` (retryable) |

---

## 8. Dependencies (Planned)

| Crate | Role |
|-------|------|
| `ff-vfs` | `VfsProvider` trait implementation |
| `ff-connector-extensibility` | `ConnectorPlugin` trait, `ConnectorRegistry`, `ConnectorError` |
| `ff-plugin` | `FileForgePlugin` lifecycle |
| `ff-logging` | Structured logging |
| `tokio` | Async runtime for network I/O |
| `rustls` / `native-tls` | TLS for FTPS |
| `ssh2` or `russh` | SSH/SFTP protocol |
| `suppaftp` or similar | FTP protocol client |

---

## 9. Capability Advertisement

| Capability | FTP | FTPS | SFTP |
|-----------|-----|------|------|
| Read | ✅ | ✅ | ✅ |
| Write | ✅ | ✅ | ✅ |
| List | ✅ | ✅ | ✅ |
| Metadata | ✅ | ✅ | ✅ |
| Rename | ✅ | ✅ | ✅ |
| Delete | ✅ | ✅ | ✅ |
| CreateDirectory | ✅ | ✅ | ✅ |
| Watch | ❌ | ❌ | ✅ (polling) |
| Search | ❌ | ❌ | ❌ |
| Copy | ❌ | ❌ | ❌ |

Watch for FTP/FTPS is not supported because the protocol lacks change
notifications and server-side metadata queries are expensive. SFTP supports
polling-based watch using stat() comparisons at configurable intervals.

---

## 10. References

- `ff-connector-extensibility` design — defines `ConnectorPlugin`, `ConnectorRegistry`, all trait contracts
- `ff-vfs` design — defines `VfsProvider`, `ProviderRegistry`, `ResourceUri`
- `ff-plugin` design — defines `FileForgePlugin`, `PluginContext`
- Project-master requirements — FFW-ARCH-001 (VFS Principle), Req 3 (Plugin Architecture)
- Connector-extensibility requirements — Requirement 6 (Future Connector Hooks)
