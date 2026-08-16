# Design Document: Mainframe Connector (`ff-connector-mainframe`) — DEFERRED

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This is a placeholder design documenting future integration points for the
> mainframe connectivity connector. No implementation tasks will be created.
> The `ff-connector-extensibility` crate (shipping in the initial release)
> defines all traits and infrastructure this connector will consume.

---

## 1. Overview

The `ff-connector-mainframe` crate will provide VFS-integrated access to z/OS
mainframe systems via four protocols:

| Protocol | URI Scheme | Purpose |
|----------|-----------|---------|
| z/OS FTP | `zos-ftp://` | MVS dataset transfer, JES spool retrieval, SITE allocation |
| TN3270 | `tn3270://` | 3270 terminal emulation, ISPF panel navigation |
| z/OSMF REST | `zosmf://` | Modern REST API for datasets, jobs, USS, system info |
| USS SSH | `zos-uss://` | Unix System Services shell access and file operations |

Each protocol registers as a separate VFS provider scheme through the
`ConnectorPlugin` trait from `ff-connector-extensibility`.

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│          ff-desktop (egui shell) — Dataset Explorer UI           │
├─────────────────────────────────────────────────────────────────┤
│  ff-dataset-catalog — local emulation (initial release)          │
│  ff-connector-mainframe — real z/OS access (THIS CRATE, DEFERRED)│
├─────────────────────────────────────────────────────────────────┤
│  ff-connector-extensibility — ConnectorPlugin trait + registry   │
├─────────────────────────────────────────────────────────────────┤
│  ff-vfs — VfsProvider trait, ProviderRegistry, ResourceUri       │
├─────────────────────────────────────────────────────────────────┤
│  ff-plugin — FileForgePlugin lifecycle                           │
├─────────────────────────────────────────────────────────────────┤
│  ff-logging — structured tracing                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. ConnectorPlugin Trait Implementation

The crate will implement `ConnectorPlugin` (which extends `VfsProvider + FileForgePlugin`)
for each protocol adapter. The trait contract from `ff-connector-extensibility`:

```rust
#[async_trait::async_trait]
pub trait ConnectorPlugin: VfsProvider + FileForgePlugin {
    fn descriptor(&self) -> &ConnectorDescriptor;
    fn connector_capabilities(&self) -> &[ConnectorCapability];
    fn api_version(&self) -> ApiVersion;
    fn state(&self) -> ConnectorState;
    async fn connect(&mut self) -> Result<(), ConnectorError>;
    async fn disconnect(&mut self) -> Result<(), ConnectorError>;
    async fn authenticate(&mut self, credential_store: &dyn CredentialStore) -> Result<(), ConnectorError>;
    fn retry_policy(&self) -> &RetryPolicy;
    fn map_error(&self, source: Box<dyn std::error::Error + Send + Sync>) -> ConnectorError;
    async fn custom_operation(&self, name: &str, params: &dyn std::any::Any)
        -> Result<Box<dyn std::any::Any + Send>, ConnectorError>;
}
```

Each protocol adapter will:
1. Return its URI scheme via `descriptor().scheme`
2. Advertise capabilities appropriate to the protocol
3. Manage connection lifecycle through the `ConnectorState` state machine
4. Authenticate via `CredentialStore` (RACF credentials, SSH keys, LTPA/JWT tokens)
5. Map protocol-specific errors to `ConnectorError` variants
6. Expose protocol-specific operations (JCL submission, ISPF navigation) via `custom_operation`

---

## 3. VFS URI Scheme Design

Resources are addressed with protocol-specific URI schemes:

```
zos-ftp://hostname:port/HLQ.QUALIFIER.DATASET
zos-ftp://hostname:port/HLQ.PDS.NAME(MEMBER)
zos-ftp://hostname:port/$JES/JOB12345/SYSPRINT

tn3270://hostname:port/session-id

zosmf://hostname:port/datasets/HLQ.QUALIFIER.DATASET
zosmf://hostname:port/jobs/JOB12345
zosmf://hostname:port/uss/u/userid/file.txt

zos-uss://hostname:port/u/userid/path/to/file
```

The VFS layer routes operations to the appropriate protocol adapter based on the
URI scheme prefix, using the `ProviderRegistry` from `ff-vfs`.

---

## 4. Integration Points with Upstream Crates

### 4.1 `ff-vfs` (VfsProvider)

- Implements `VfsProvider` for each protocol adapter
- Registers with `ProviderRegistry` during plugin initialization
- Maps z/OS dataset attributes (RECFM, LRECL, BLKSIZE, DSORG) to VFS metadata
- Translates MVS dataset names to VFS path segments

### 4.2 `ff-connector-extensibility` (ConnectorPlugin + Registry)

- Implements `ConnectorPlugin` trait for lifecycle, auth, and capability advertisement
- Registers with `ConnectorRegistry` at plugin startup
- Advertises per-protocol capabilities:
  - **z/OS FTP**: Read, Write, List, Metadata, Delete, CreateDirectory
  - **TN3270**: Read (screen scraping only)
  - **z/OSMF**: Read, Write, List, Metadata, Delete, CreateDirectory, Search, Rename
  - **USS SSH**: Read, Write, List, Metadata, Delete, CreateDirectory, Rename
- Uses `RetryPolicy` for automatic reconnection on network failures
- Maps FTP reply codes, HTTP status codes, SSH errors, and TN3270 failures to `ConnectorError`

### 4.3 `ff-plugin` (FileForgePlugin Lifecycle)

- Implements `FileForgePlugin` for initialization/shutdown
- During `initialize()`: obtains `PluginContext`, registers protocol adapters
- During `shutdown()`: disconnects all active sessions gracefully
- Advertises `Capability::Providers` with the plugin `CapabilityRegistry`

### 4.4 `ff-dataset-catalog` (Shared Dataset Model)

- Shares MVS dataset naming conventions (`HLQ.QUALIFIER.NAME`, 44-char limit)
- Shares metadata structures: RECFM, LRECL, BLKSIZE, DSORG
- Shares PDS/PDSE member concepts and GDG generation numbering
- Enables seamless transition: local catalog emulation → remote z/OS access
- Remote dataset listings can populate the local catalog cache for offline browsing
- The dataset catalog's `DatasetEntry` metadata model maps directly to z/OS LISTCAT output

---

## 5. Placeholder Module Structure

```
crates/ff-connector-mainframe/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Crate root, re-exports, plugin registration
│   ├── connection.rs           # MainframeConnection — shared connection config
│   ├── credential.rs           # RACF credential handling, PassTicket, SSH key mgmt
│   ├── codepage.rs             # EBCDIC↔UTF-8 translation, codepage tables
│   ├── ftp/
│   │   ├── mod.rs              # z/OS FTP protocol adapter (ConnectorPlugin impl)
│   │   ├── dataset.rs          # MVS dataset operations via FTP
│   │   ├── jes.rs              # JES spool retrieval and JCL submission
│   │   └── site.rs             # SITE commands for dataset allocation
│   ├── tn3270/
│   │   ├── mod.rs              # TN3270 protocol adapter (ConnectorPlugin impl)
│   │   ├── datastream.rs       # 3270 data stream parser (fields, attributes)
│   │   ├── screen.rs           # Screen model, field extraction, cursor
│   │   └── ispf.rs             # Automated ISPF panel navigation
│   ├── zosmf/
│   │   ├── mod.rs              # z/OSMF REST adapter (ConnectorPlugin impl)
│   │   ├── datasets.rs         # Dataset CRUD via REST
│   │   ├── jobs.rs             # Job submission and monitoring
│   │   └── uss.rs              # USS file operations via REST
│   ├── ssh/
│   │   ├── mod.rs              # USS SSH adapter (ConnectorPlugin impl)
│   │   └── commands.rs         # Remote command execution
│   └── error.rs                # Protocol-specific error types, ConnectorError mapping
└── tests/
    ├── ftp_tests.rs
    ├── tn3270_tests.rs
    ├── zosmf_tests.rs
    ├── ssh_tests.rs
    └── integration.rs
```

---

## 6. Key Types

```rust
/// Shared connection configuration for all mainframe protocols.
pub struct MainframeConnection {
    pub hostname: String,
    pub port: u16,
    pub lpar_name: Option<String>,
    pub sysplex: Option<String>,
    pub credential_key: String,       // Key into CredentialStore
    pub codepage: CodePage,           // EBCDIC codepage for translation
    pub retry_policy: RetryPolicy,
}

/// EBCDIC codepage identifier for character translation.
pub enum CodePage {
    Ibm037,    // US/Canada EBCDIC
    Ibm1047,   // Latin-1/Open Systems
    Ibm500,    // International
    Ibm875,    // Greek
    Custom(u16),
}

/// Access handle for MVS dataset operations.
pub struct DatasetAccess {
    pub dsname: String,               // e.g., "HLQ.QUALIFIER.NAME"
    pub member: Option<String>,       // PDS member name
    pub recfm: RecordFormat,
    pub lrecl: u32,
    pub blksize: u32,
    pub dsorg: DatasetOrganization,
}

/// Record format (maps to z/OS RECFM).
pub enum RecordFormat {
    Fixed,          // F
    FixedBlocked,   // FB
    Variable,       // V
    VariableBlocked,// VB
    Undefined,      // U
}

/// Dataset organization (maps to z/OS DSORG).
pub enum DatasetOrganization {
    Sequential,     // PS
    Partitioned,    // PO (PDS)
    PartitionedE,   // PO-E (PDSE)
    Vsam,           // VSAM (various)
    Direct,         // DA
}

/// JCL submission and job tracking.
pub struct JclSubmission {
    pub jcl_content: String,
    pub job_id: Option<String>,       // Assigned after submission
    pub job_name: String,
    pub job_class: char,
    pub notify_on_completion: bool,
}

/// Job status from JES.
pub enum JobStatus {
    Submitted,
    Queued,
    Active,
    Completed { return_code: i32 },
    Abended { system_code: String },
    Purged,
}

/// TN3270 screen state.
pub struct ScreenBuffer {
    pub rows: u16,
    pub cols: u16,
    pub fields: Vec<ScreenField>,
    pub cursor_row: u16,
    pub cursor_col: u16,
}

/// A field on a 3270 screen.
pub struct ScreenField {
    pub row: u16,
    pub col: u16,
    pub length: u16,
    pub text: String,
    pub protected: bool,
    pub numeric: bool,
    pub highlighted: bool,
}
```

---

## 7. Dataset Catalog Integration

The connector bridges remote z/OS systems with the local `ff-dataset-catalog`:

1. **Catalog Sync** — Remote `LISTCAT` output populates local `DatasetEntry` rows,
   enabling offline browsing of dataset inventories.
2. **Transparent Open** — When a user opens a dataset URI (`zos-ftp://...`), the
   connector fetches content from z/OS and presents it through the VFS layer;
   the dataset catalog provides naming validation and metadata context.
3. **GDG Resolution** — Generation Data Group relative references (e.g., `(+1)`, `(0)`,
   `(-1)`) are resolved against the catalog's GDG base entry, with remote verification.
4. **Allocation** — Dataset allocation parameters from the catalog's allocation templates
   map to FTP `SITE` commands or z/OSMF REST calls for remote dataset creation.

---

## 8. Dependencies (Future)

| Crate | Relationship |
|-------|-------------|
| `ff-vfs` | Implements `VfsProvider` trait |
| `ff-connector-extensibility` | Implements `ConnectorPlugin` trait |
| `ff-plugin` | Implements `FileForgePlugin` lifecycle |
| `ff-logging` | Structured tracing |
| `ff-dataset-catalog` | Shares naming conventions, metadata model, catalog sync |
| `tokio` | Async runtime for network I/O |
| `rustls` | TLS for secure FTP, TN3270+TLS, HTTPS (z/OSMF) |
| `russh` | SSH2 client for USS access |
| `reqwest` | HTTP client for z/OSMF REST API |

---

## 9. Deferred Scope Notes

- No implementation code will be written for this crate in the initial release.
- The `ff-connector-extensibility` crate ships with all required traits and
  infrastructure; adding this connector requires zero changes to upstream crates.
- Full EARS-format acceptance criteria and implementation tasks will be produced
  when this connector moves to active development.
- The connector will be distributed as an optional plugin crate, loaded at runtime
  via the `ff-plugin` dynamic loading mechanism.
