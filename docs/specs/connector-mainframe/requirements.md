# Requirements Document — DEFERRED

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This specification documents the *future* Mainframe connectivity connector for
> FileForgeWorkbench. It is NOT scheduled for the initial release. The
> `dataset-catalog` sub-project provides local mainframe filesystem emulation
> (MVS datasets, PDS/PDSE, GDG, sequential files) in the initial release; this
> connector adds REAL z/OS mainframe connectivity in a future phase.

## Introduction

The `ff-connector-mainframe` crate will provide VFS connectors for z/OS mainframe
access via multiple protocols: z/OS FTP (dataset transfer), TN3270 terminal
emulation, z/OSMF REST API, and USS SSH. It will implement the `ConnectorPlugin`
trait from `ff-connector-extensibility`, which combines `VfsProvider` (from
`ff-vfs`) with connector lifecycle, authentication, and capability advertisement.

### What This Connector Will Provide

- **z/OS FTP** — dataset transfer via the mainframe FTP server, supporting MVS
  dataset naming conventions, JES spool retrieval, SITE commands for dataset
  allocation, and automatic EBCDIC↔UTF-8 translation.
- **TN3270 terminal emulation** — 3270 screen interaction for ISPF/TSO sessions,
  including screen scraping, field extraction, cursor navigation, and automated
  ISPF panel navigation.
- **z/OSMF REST API** — modern REST interface for z/OS resource management
  including dataset operations, job submission/monitoring, USS file access, and
  system variable queries.
- **USS SSH** — Unix System Services shell access on z/OS for file operations
  in the UNIX filesystem, command execution, and pipe-based data transfer.

### Architectural Integration Point

This connector implements:
- `VfsProvider` trait from `ff-vfs` (for read/write/list/stat operations)
- `ConnectorPlugin` trait from `ff-connector-extensibility` (for lifecycle,
  registration, capability advertisement, and authentication)
- `FileForgePlugin` trait from `ff-plugin` (for plugin initialization/shutdown)

Registration occurs at plugin initialization time via the `ConnectorRegistry`.
The connector advertises its URI schemes (`zos-ftp://`, `tn3270://`,
`zosmf://`, `zos-uss://`) and its supported capabilities per protocol.

### Relationship to Dataset Catalog

The `dataset-catalog` sub-project (included in the initial release) provides
**local emulation** of mainframe dataset structures on the desktop — SQLite-based
catalog, MVS naming, PDS member navigation, and repository layout. This connector
extends that model by providing **real mainframe connectivity** to an actual z/OS
system. Both share dataset naming conventions and metadata structures, enabling
seamless transition from local emulation to remote access.

### Extensibility Hook (Initial Release)

The `connector-extensibility` crate ships in the initial release and defines all
the traits, error types, and registry infrastructure that this connector will
consume. No code changes to VFS core or the workbench platform will be required
to add this connector — it plugs in via the existing extensibility framework.

---

## Placeholder Requirements (Future Scope)

The following outline documents the eventual scope. Full EARS-format acceptance
criteria will be written when this connector moves to active development.

### Requirement 1: z/OS FTP — Dataset Transfer

- Connect to z/OS FTP server using credentials from `CredentialStore`
- Support MVS dataset naming: `'HLQ.QUALIFIER.NAME'` (quoted) for PDS, sequential
- Navigate PDS members via directory listing on `'HLQ.PDS.NAME'`
- JES spool retrieval: submit JCL, retrieve job output via JES interface
- SITE commands for dataset allocation (LRECL, BLKSIZE, RECFM, SPACE, UNIT)
- Automatic EBCDIC↔UTF-8 code page translation (configurable codepage)
- Binary transfer mode for COMP-3 and packed-decimal data
- Support GDG (Generation Data Group) relative generation syntax
- Map MVS dataset attributes to VFS metadata (RECFM, LRECL, DSORG)

### Requirement 2: TN3270 Terminal Emulation

- Establish TN3270/TN3270E connections to z/OS (port 23 or custom)
- Parse 3270 data stream: fields, attributes, extended attributes, colours
- Screen scraping: extract text from named screen regions and field positions
- Field input: place cursor, type data, send AID keys (Enter, PF1-PF24)
- ISPF navigation: automated panel traversal for common operations
- Session management: multiple concurrent 3270 sessions per connection
- Extended data stream support (SFE, SA, MF) for DBCS and graphic fields
- TLS/SSL encryption for secure 3270 connections (TN3270 + TLS)

### Requirement 3: z/OSMF REST API Integration

- Authenticate via z/OSMF (LTPA token, JWT, or basic auth)
- Dataset operations: list, read, write, create, delete, rename via REST
- Job operations: submit JCL, monitor status, retrieve output (spool files)
- USS file operations: read, write, list, chmod, chown via z/OSMF files API
- System variable queries: LPAR name, sysplex info, system symbols
- Paging support for large dataset/member lists
- Honour z/OSMF CSRF protection headers

### Requirement 4: USS SSH Access

- Connect to z/OS USS via SSH (key-based or password authentication)
- File operations in USS: read, write, list, stat, mkdir, rm
- Command execution: run shell commands, capture stdout/stderr
- Support non-standard SSH ports and jump-host / proxy configurations
- Automatic tagging awareness (USS file tags for codepage identification)
- Known-hosts verification with configurable TOFU policy

### Requirement 5: Credential Management

- RACF user/password authentication for FTP, TN3270, z/OSMF
- PassTicket generation/validation for single-sign-on scenarios
- Client certificate authentication (RACF certificate mapping)
- SSH key management for USS access (RSA, Ed25519, ECDSA)
- Secure credential storage via workbench `CredentialStore`
- Session token caching (LTPA, JWT) with automatic refresh

### Requirement 6: Error Mapping and Reconnection

- Map z/OS FTP reply codes to `ConnectorError` taxonomy
- Map TN3270 connection failures to `ConnectorError` variants
- Map z/OSMF HTTP status codes (401, 403, 404, 500) to error kinds
- Map SSH errors to `ConnectorError` variants
- Classify RACF auth failures distinctly from network errors
- Include LPAR name, job ID, and dataset name in error context
- Automatic reconnection with configurable `RetryPolicy`
- Session keep-alive for long-running TN3270 and SSH sessions

---

## Dependencies

| Crate | Relationship |
|-------|-------------|
| `ff-vfs` | Implements `VfsProvider` trait |
| `ff-connector-extensibility` | Implements `ConnectorPlugin` trait |
| `ff-plugin` | Implements `FileForgePlugin` lifecycle |
| `ff-logging` | Structured logging |
| `ff-dataset-catalog` | Shares dataset naming conventions and metadata model |

## References

- **WB**: Workbench Architecture Brief — VFS extensibility, FFW-ARCH-001
- **FFW**: FileForgeWorkbench cross-cutting requirements (VFS Principle, Plugin Architecture)
- **DSC**: Dataset Catalog Brief — local mainframe emulation (initial release)
- Connector-extensibility requirements (Requirement 6: Future Connector Hooks)
- IBM z/OSMF REST API documentation
- IBM FTP for z/OS documentation (JES, MVS dataset access)
- RFC 2355 (TN3270E) and RFC 1576 (TN3270)
