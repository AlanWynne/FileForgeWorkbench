# Requirements Document — DEFERRED

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This specification documents the *future* Network/UNC filesystem connector for
> FileForgeWorkbench. It is NOT scheduled for the initial release. The
> `connector-extensibility` trait (defined in `ff-connector-extensibility`) provides
> the architectural hook that this connector will use when implemented.

## Introduction

The `ff-connector-network-fs` crate will provide a VFS connector for network
filesystem access — Windows UNC paths (`\\server\share`), SMB/CIFS shares,
NFS mounts, and mapped drive resolution. It will implement the `ConnectorPlugin`
trait from `ff-connector-extensibility`, which combines `VfsProvider` (from
`ff-vfs`) with connector lifecycle, authentication, and capability advertisement.

### What This Connector Will Provide

- **Network/UNC path support** — direct access to `\\server\share\path` resources
  as first-class VFS entries, transparent to consuming subsystems.
- **SMB/CIFS protocol integration** — browsing, reading, and writing files on
  Windows file shares and Samba servers.
- **NFS mount support** — access to NFS-exported directories with appropriate
  UID/GID credential mapping.
- **Mapped drive resolution** — resolving Windows drive letters (e.g., `Z:\`)
  back to their underlying UNC paths, and presenting both views coherently
  within the VFS layer.

### Architectural Integration Point

This connector implements:
- `VfsProvider` trait from `ff-vfs` (for read/write/list/stat/watch operations)
- `ConnectorPlugin` trait from `ff-connector-extensibility` (for lifecycle,
  registration, capability advertisement, and authentication)
- `FileForgePlugin` trait from `ff-plugin` (for plugin initialization/shutdown)

Registration occurs at plugin initialization time via the `ConnectorRegistry`.
The connector advertises its URI scheme (e.g., `smb://`, `nfs://`, or `unc://`)
and its supported capabilities.

### Extensibility Hook (Initial Release)

The `connector-extensibility` crate ships in the initial release and defines all
the traits, error types, and registry infrastructure that this connector will
consume. No code changes to VFS core or the workbench platform will be required
to add this connector — it plugs in via the existing extensibility framework.

---

## Placeholder Requirements (Future Scope)

The following outline documents the eventual scope. Full EARS-format acceptance
criteria will be written when this connector moves to active development.

### Requirement 1: UNC Path Resolution

- Resolve `\\server\share\path` addresses to VFS resource URIs
- Detect and normalize forward-slash vs backslash variants
- Handle long UNC paths (`\\?\UNC\server\share\...`)
- Map UNC resources to the VFS `unc://` or `smb://` scheme

### Requirement 2: SMB/CIFS Connectivity

- Connect to SMB/CIFS shares using credentials from the `CredentialStore`
- Support SMBv2 and SMBv3 protocol negotiation
- Support browsing (list shares on a server, list directories)
- Support read, write, rename, delete, and create-directory operations
- Honour `RetryPolicy` for transient network failures
- Emit `ConnectorStateChanged` events on connect/disconnect

### Requirement 3: NFS Mount Support

- Connect to NFS-exported directories (NFSv3, NFSv4)
- Map UID/GID credentials for permission enforcement
- Support file locking semantics where the NFS server supports them
- Detect stale NFS handles and report as `ConnectorError::NetworkError`

### Requirement 4: Mapped Drive Resolution

- Detect Windows mapped drives and resolve to underlying UNC paths
- Present a unified view: operations on `Z:\file.txt` and `\\server\share\file.txt`
  resolve to the same VFS resource (no duplicates)
- Handle drives mapped at login vs. persistent mappings
- Gracefully degrade on non-Windows platforms (no-op or unsupported)

### Requirement 5: File Watching on Network Paths

- Provide VFS watch capability for network paths where the protocol supports it
- SMB: leverage `ReadDirectoryChangesW` or equivalent polling
- NFS: polling-based fallback with configurable interval
- Advertise `Watch` capability only when the underlying share supports it

### Requirement 6: Error Mapping

- Map SMB/CIFS-specific errors to `ConnectorError` taxonomy
- Map NFS-specific errors (ESTALE, EACCES, etc.) to `ConnectorError` variants
- Include remote server name and share in error context
- Classify network timeouts and unreachable hosts as retryable

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
