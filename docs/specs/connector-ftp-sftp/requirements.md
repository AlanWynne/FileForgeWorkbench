# Requirements Document — DEFERRED

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This specification documents the *future* FTP/FTPS/SFTP connector for
> FileForgeWorkbench. It is NOT scheduled for the initial release. The
> `connector-extensibility` trait (defined in `ff-connector-extensibility`) provides
> the architectural hook that this connector will use when implemented.

## Introduction

The `ff-connector-ftp-sftp` crate will provide VFS connectors for FTP, FTPS, and
SFTP remote file access. It will implement the `ConnectorPlugin` trait from
`ff-connector-extensibility`, which combines `VfsProvider` (from `ff-vfs`) with
connector lifecycle, authentication, and capability advertisement.

### What This Connector Will Provide

- **FTP protocol support** — plain-text FTP connectivity with both active and
  passive transfer modes, directory browsing, upload, download, rename, and
  delete operations.
- **FTPS support (FTP over TLS/SSL)** — secure FTP using both implicit TLS
  (dedicated port) and explicit TLS (STARTTLS upgrade on standard port).
- **SFTP support (SSH File Transfer Protocol)** — file transfer over SSH with
  key-based authentication, agent forwarding, and known-host verification.
- **Full file operations** — browsing, upload, download, rename, delete, and
  create-directory across all three protocols, presented uniformly through the
  VFS abstraction.

### Architectural Integration Point

This connector implements:
- `VfsProvider` trait from `ff-vfs` (for read/write/list/stat/watch operations)
- `ConnectorPlugin` trait from `ff-connector-extensibility` (for lifecycle,
  registration, capability advertisement, and authentication)
- `FileForgePlugin` trait from `ff-plugin` (for plugin initialization/shutdown)

Registration occurs at plugin initialization time via the `ConnectorRegistry`.
The connector advertises its URI schemes (`ftp://`, `ftps://`, `sftp://`) and
its supported capabilities per protocol.

### Extensibility Hook (Initial Release)

The `connector-extensibility` crate ships in the initial release and defines all
the traits, error types, and registry infrastructure that this connector will
consume. No code changes to VFS core or the workbench platform will be required
to add this connector — it plugs in via the existing extensibility framework.

---

## Placeholder Requirements (Future Scope)

The following outline documents the eventual scope. Full EARS-format acceptance
criteria will be written when this connector moves to active development.

### Requirement 1: FTP Connectivity

- Connect to FTP servers using username/password from `CredentialStore`
- Support active mode (PORT) and passive mode (PASV/EPSV) data connections
- Directory listing with parsed file metadata (size, date, permissions)
- Upload, download, rename, delete, and mkdir operations
- Configurable connection timeout and keep-alive interval
- Emit `ConnectorStateChanged` events on connect/disconnect

### Requirement 2: FTPS TLS Negotiation

- Explicit FTPS via STARTTLS command on standard port 21
- Implicit FTPS via dedicated TLS port (typically 990)
- Certificate validation with configurable trust store
- Support for client certificate authentication
- TLS session reuse for data connections (required by many servers)
- Graceful fallback notification if server does not support TLS

### Requirement 3: SFTP via SSH

- Connect via SSH using key-based authentication (RSA, Ed25519, ECDSA)
- Support password authentication as fallback
- SSH agent forwarding for key management
- Known-hosts verification with configurable strict/TOFU policy
- Support non-standard SSH ports
- SSH keepalive for long-lived connections

### Requirement 4: Transfer Modes

- Binary transfer mode (default for all file types)
- ASCII transfer mode with line-ending conversion (FTP/FTPS only)
- Resume support for interrupted transfers (REST command / SFTP seek)
- Configurable transfer buffer size for throughput tuning
- Progress reporting via workbench progress infrastructure

### Requirement 5: Error Mapping to ConnectorError Taxonomy

- Map FTP reply codes (4xx, 5xx) to `ConnectorError` variants
- Map SSH/SFTP error codes to `ConnectorError` variants
- Include remote server hostname and path in error context
- Classify connection-refused, timeout, and auth-failure as distinct error kinds
- Honour `RetryPolicy` for transient failures (connection drops, timeouts)

### Requirement 6: File Watching (Polling-Based)

- Polling-based change detection with configurable interval
- Compare file metadata (size, mtime) across poll cycles
- Advertise `Watch` capability with polling semantics (no push notification)
- Rate-limit polling to avoid excessive server load
- Emit VFS change events compatible with `file-tree-panel` refresh

---

## Dependencies

| Crate | Relationship |
|-------|-------------|
| `ff-vfs` | Implements `VfsProvider` trait |
| `ff-connector-extensibility` | Implements `ConnectorPlugin` trait |
| `ff-plugin` | Implements `FileForgePlugin` lifecycle |
| `ff-logging` | Structured logging |

## References

- **WB**: Workbench Architecture Brief — VFS extensibility, FFW-ARCH-001
- **FFW**: FileForgeWorkbench cross-cutting requirements (VFS Principle, Plugin Architecture)
- Connector-extensibility requirements (Requirement 6: Future Connector Hooks)

---

## Formal Acceptance Criteria (DEFERRED — Future Release)

> The following criteria are written in EARS format for traceability. All criteria
> carry status **DEFERRED** — they are not scheduled for the initial release.
> Full implementation details will be added to `design.md` when this connector
> moves to active development.

### Requirement 1: FTP Connectivity *(DEFERRED)*

#### Acceptance Criteria

1. WHEN the connector is configured with FTP credentials and a server address, THE connector SHALL establish a connection supporting active and passive transfer modes, and SHALL support directory listing, upload, download, rename, delete, and mkdir operations.

---

### Requirement 2: FTPS TLS Negotiation *(DEFERRED)*

#### Acceptance Criteria

1. WHEN the connector is configured for FTPS, THE connector SHALL negotiate TLS using either explicit STARTTLS on port 21 or implicit TLS on the configured port, validating the server certificate against the configured trust store.

---

### Requirement 3: SFTP via SSH *(DEFERRED)*

#### Acceptance Criteria

1. WHEN the connector is configured for SFTP, THE connector SHALL establish an SSH connection using key-based or password authentication, verify the server against the known-hosts store, and support all VFS file operations over the encrypted channel.

---

### Requirement 4: Transfer Modes *(DEFERRED)*

#### Acceptance Criteria

1. WHEN a file transfer is initiated, THE connector SHALL use binary transfer mode by default and SHALL support ASCII mode with line-ending conversion for FTP/FTPS. THE connector SHALL support resuming interrupted transfers where the protocol supports it.

---

### Requirement 5: Error Mapping to ConnectorError Taxonomy *(DEFERRED)*

#### Acceptance Criteria

1. WHEN a protocol operation fails, THE connector SHALL map FTP reply codes (4xx, 5xx) and SSH/SFTP error codes to the appropriate ConnectorError variant, including the remote server hostname and path in the error context.

---

### Requirement 6: File Watching (Polling-Based) *(DEFERRED)*

#### Acceptance Criteria

1. WHEN a directory node is expanded, THE connector SHALL register a polling-based watch at a configurable interval, comparing file metadata (size, mtime) across poll cycles and emitting VFS change events for detected differences. THE connector SHALL advertise the Watch capability with polling semantics.

---

