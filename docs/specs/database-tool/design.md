# Design Document: Database Tool (`ff-database-tool`)

## Overview

The `ff-database-tool` crate is a **workbench plugin** that delivers a full-featured integrated Database IDE within FileForgeWorkbench. It provides connection management, SQL editing with dialect-aware syntax highlighting, query execution, result grid display, schema browser navigation, ER diagram visualisation, data transfer workflows, and database administration panels — all delivered as dockable panels registered through the plugin architecture.

### Purpose

- Provide a complete database IDE experience as a workbench plugin
- Support PostgreSQL, MySQL/MariaDB, SQLite, and Microsoft SQL Server via Rust-native async drivers
- Integrate seamlessly with platform services: command framework, layout system, workflow engine, VFS
- Deliver non-blocking UI via Tokio async I/O for all database operations
- Enable extensibility for additional database platforms through the `DatabaseDriver` trait

### Position in Architecture

```
Wave 6 — Application Tools (depends on Wave 2–4 platform crates)

┌─────────────────────────────────────────────────────────────┐
│                    Application Binary (ff-desktop)            │
├─────────────────────────────────────────────────────────────┤
│     ff-database-tool  │  other tool plugins                  │
│              (Wave 6 — Application Tools)                     │
├─────────────────────────────────────────────────────────────┤
│  ff-workflow │ ff-layout │ ff-command │ ff-plugin │ ff-vfs    │
│  ff-connector-extensibility │ ff-configuration               │
│              (Wave 2–4 — Platform Architecture)               │
├─────────────────────────────────────────────────────────────┤
│                     ff-logging (Wave 0)                       │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints

- **FFW-ARCH-001**: All file access goes through VFS — no direct `std::fs` or `tokio::fs`
- **Plugin Principle**: Implements `FileForgePlugin` trait; no special core coupling
- **Command-Driven**: All user operations are registered commands under `db.*` namespace
- **Async I/O**: All database operations are async on Tokio; never block the egui render thread
- **Multi-Crate Workspace**: Crate at `crates/ff-database-tool`

---

## Architecture

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ff-database-tool                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │ DatabasePlugin│  │  Commands    │  │  Panels (UI) │  │  Workflows │  │
│  │  (lifecycle) │  │  (db.*)      │  │  (egui)      │  │  (transfer)│  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘  │
│         │                  │                  │                 │         │
│  ┌──────┴──────────────────┴──────────────────┴─────────────────┴──────┐ │
│  │                     Core Services Layer                              │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │  DriverRegistry │ ConnectionManager │ ConnectionPool │ QueryExecutor │ │
│  │  MetadataService│ SqlParser │ AutoComplete │ DdlGenerator            │ │
│  └──────────────────────────────┬───────────────────────────────────────┘ │
│                                 │                                         │
│  ┌──────────────────────────────┴───────────────────────────────────────┐ │
│  │                     Driver Abstraction Layer                          │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │  DatabaseDriver trait │ PostgresDriver │ MysqlDriver │ SqliteDriver  │ │
│  │  SqlServerDriver │ (future: custom plugins)                          │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │ uses
                    ┌──────────────┴──────────────┐
                    │    Platform Services         │
                    │  ff-plugin (PluginContext)   │
                    │  ff-command (Registry)       │
                    │  ff-layout (DockablePanel)   │
                    │  ff-workflow (WorkflowDef)   │
                    │  ff-vfs (file I/O)           │
                    │  ff-connector-extensibility  │
                    └─────────────────────────────┘
```

### Module Structure

```
crates/ff-database-tool/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Crate root, public API re-exports
│   ├── plugin.rs                 # DatabasePlugin (FileForgePlugin impl)
│   ├── error.rs                  # DatabaseToolError enum
│   ├── driver/
│   │   ├── mod.rs                # DatabaseDriver trait, DriverCapabilities
│   │   ├── registry.rs           # DriverRegistry
│   │   ├── postgres.rs           # PostgreSQL driver implementation
│   │   ├── mysql.rs              # MySQL/MariaDB driver implementation
│   │   ├── sqlite.rs             # SQLite driver implementation
│   │   ├── sqlserver.rs          # SQL Server (tiberius) driver implementation
│   │   └── dialect.rs            # SqlDialect enum and dialect-specific metadata
│   ├── connection/
│   │   ├── mod.rs                # ConnectionManager
│   │   ├── descriptor.rs         # ConnectionDescriptor, NetworkProfile
│   │   ├── pool.rs               # ConnectionPool
│   │   ├── credential.rs         # CredentialStore (OS keyring + fallback)
│   │   ├── ssh.rs                # SSH tunnel management
│   │   └── state.rs              # ConnectionState enum, lifecycle FSM
│   ├── sql/
│   │   ├── mod.rs                # SQL editor services
│   │   ├── parser.rs             # Statement boundary parser
│   │   ├── highlight.rs          # Dialect-aware syntax highlighting tokens
│   │   ├── complete.rs           # Auto-complete engine
│   │   ├── format.rs             # SQL formatter
│   │   ├── template.rs           # SQL templates/snippets
│   │   └── parameter.rs          # Parameter placeholder detection/binding
│   ├── execution/
│   │   ├── mod.rs                # QueryExecution, ExecutionPlan
│   │   ├── executor.rs           # Async query executor
│   │   ├── plan.rs               # Execution plan tree model
│   │   └── log.rs                # Execution log
│   ├── result/
│   │   ├── mod.rs                # ResultSet data model
│   │   ├── batch.rs              # Batch-fetching logic
│   │   ├── edit.rs               # Pending row edits, DML generation
│   │   └── export.rs             # Clipboard/export formatting
│   ├── schema/
│   │   ├── mod.rs                # Schema browser model
│   │   ├── tree.rs               # Tree node types, lazy loading
│   │   ├── metadata.rs           # Metadata queries per driver
│   │   ├── ddl.rs                # DDL generation
│   │   ├── search.rs             # Global metadata search
│   │   └── cache.rs              # Metadata cache
│   ├── transfer/
│   │   ├── mod.rs                # Data transfer workflow definitions
│   │   ├── import.rs             # Import workflow steps
│   │   ├── export.rs             # Export workflow steps
│   │   ├── cross_db.rs           # Cross-database transfer
│   │   ├── bulk.rs               # Bulk load (COPY, LOAD DATA)
│   │   ├── column_map.rs         # Column mapping engine
│   │   └── error_policy.rs       # Error handling policies
│   ├── diagram/
│   │   ├── mod.rs                # ER diagram model
│   │   ├── layout.rs             # Auto-layout algorithms
│   │   ├── notation.rs           # IDEF1X, Crow's Foot, Bachman rendering
│   │   ├── export.rs             # PNG, SVG, GraphML export
│   │   └── persistence.rs        # Diagram save/restore
│   ├── admin/
│   │   ├── mod.rs                # Administration services
│   │   ├── session.rs            # Session manager queries
│   │   ├── lock.rs               # Lock manager, blocking chains
│   │   ├── storage.rs            # Tablespace/storage info
│   │   ├── dashboard.rs          # Performance dashboard metrics
│   │   └── security.rs           # User/role management
│   └── panel/
│       ├── mod.rs                # Panel registrations
│       ├── schema_browser.rs     # SchemaBrowserPanel (DockablePanel)
│       ├── sql_editor.rs         # SqlEditorPanel (DockablePanel)
│       ├── result_grid.rs        # ResultGridPanel (DockablePanel)
│       ├── er_diagram.rs         # ErDiagramPanel (DockablePanel)
│       ├── connection.rs         # ConnectionPanel (wizard UI)
│       ├── session_manager.rs    # SessionManagerPanel
│       ├── lock_manager.rs       # LockManagerPanel
│       └── dashboard.rs          # DashboardPanel
└── tests/
    ├── driver_registry_props.rs  # Property tests: driver lookups
    ├── connection_pool_props.rs  # Property tests: pool behaviour
    ├── sql_parser_props.rs       # Property tests: statement splitting
    ├── result_batch_props.rs     # Property tests: batch fetching
    ├── column_map_props.rs       # Property tests: column mapping
    └── er_layout_props.rs        # Property tests: ER layout invariants
```

---

## Components and Interfaces

### 3.1 Driver Types

```rust
/// A database driver definition — static metadata about how to connect.
#[derive(Debug, Clone)]
pub struct DriverDefinition {
    pub name: String,                    // Unique driver ID: "postgres", "mysql", etc.
    pub display_name: String,            // Human-readable: "PostgreSQL"
    pub platforms: Vec<String>,          // Supported DB platforms
    pub url_template: String,            // e.g., "postgres://{user}:{password}@{host}:{port}/{database}"
    pub default_port: u16,
    pub crate_name: String,              // Rust crate: "sqlx", "tokio-postgres", "tiberius"
    pub capabilities: DriverCapabilities,
    pub connection_params: Vec<DriverParam>,  // Driver-specific params (SSL, timezone, etc.)
}

/// Capability flags for a database driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverCapabilities {
    pub read_only: bool,
    pub read_write: bool,
    pub transactions: bool,
    pub streaming: bool,
    pub prepared_statements: bool,
    pub bulk_load: bool,
}

/// A driver-specific connection parameter definition.
#[derive(Debug, Clone)]
pub struct DriverParam {
    pub key: String,
    pub display_name: String,
    pub param_type: ParamType,
    pub default_value: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum ParamType {
    Text,
    Integer,
    Boolean,
    Enum(Vec<String>),
    FilePath,
}
```

### 3.2 Connection Types

```rust
/// Serializable connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDescriptor {
    pub id: ConnectionId,
    pub name: String,
    pub driver_name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub credential_ref: CredentialRef,    // Reference into credential store
    pub connection_type: ConnectionType,
    pub network_profile: Option<NetworkProfileRef>,
    pub ssh_config: Option<SshConfig>,
    pub ssl_mode: SslMode,
    pub pool_config: PoolConfig,
    pub bootstrap_queries: Vec<String>,
    pub idle_timeout_secs: Option<u64>,
    pub keepalive_interval_secs: Option<u64>,
    pub extra_params: HashMap<String, String>,
}

/// Connection type classification with visual/behavioural properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionType {
    pub name: String,                    // "Development", "Test", "Production"
    pub colour: [u8; 4],                 // RGBA
    pub confirm_on_execute: bool,
}

/// Runtime state of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// Pool configuration per connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout_ms: u64,
    pub idle_timeout_ms: u64,
    pub validation_query: Option<String>,
    pub auto_commit: bool,
    pub isolation_level: IsolationLevel,
    pub separate_connections: bool,       // One connection per editor tab
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}
```

### 3.3 Query Execution Types

```rust
/// Represents a single query execution.
pub struct QueryExecution {
    pub id: ExecutionId,
    pub sql: String,
    pub connection_id: ConnectionId,
    pub started_at: Instant,
    pub state: ExecutionState,
    pub cancel_token: CancellationToken,
}

#[derive(Debug, Clone)]
pub enum ExecutionState {
    Pending,
    Running,
    Completed(ExecutionResult),
    Cancelled,
    Failed(String),
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub elapsed: Duration,
    pub rows_affected: u64,
    pub result_set: Option<ResultSetHandle>,
    pub warnings: Vec<String>,
}

/// Execution plan tree node.
#[derive(Debug, Clone)]
pub struct PlanNode {
    pub operation: String,
    pub estimated_cost: f64,
    pub estimated_rows: u64,
    pub actual_rows: Option<u64>,
    pub children: Vec<PlanNode>,
    pub properties: HashMap<String, String>,
}
```

### 3.4 Result Set Types

```rust
/// Handle to a result set that supports batched row fetching.
pub struct ResultSetHandle {
    pub columns: Vec<ColumnDef>,
    pub total_rows: Option<u64>,         // None if unknown until exhausted
    pub batch_size: usize,
    fetched_rows: Vec<Row>,
    exhausted: bool,
}

/// Column definition from result set metadata.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub ordinal: usize,
    pub name: String,
    pub data_type: SqlType,
    pub nullable: bool,
    pub max_length: Option<usize>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
}

/// Unified SQL type enum for cross-database type representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlType {
    Integer,
    BigInt,
    SmallInt,
    Float,
    Double,
    Decimal { precision: u32, scale: u32 },
    Varchar(Option<u32>),
    Text,
    Boolean,
    Date,
    Time,
    Timestamp,
    Blob,
    Clob,
    Json,
    Uuid,
    Array(Box<SqlType>),
    Custom(String),
}
```

### 3.5 Schema Browser Types

```rust
/// Tree node in the schema browser.
#[derive(Debug, Clone)]
pub enum SchemaNode {
    Connection { id: ConnectionId, name: String, state: ConnectionState },
    Database { name: String },
    Schema { name: String },
    Category { kind: ObjectCategory },
    Table { name: String, schema: String },
    View { name: String, schema: String },
    Procedure { name: String, schema: String },
    Function { name: String, schema: String },
    Column { name: String, data_type: SqlType, nullable: bool },
    Index { name: String },
    Constraint { name: String, kind: ConstraintKind },
    Trigger { name: String },
    Sequence { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectCategory {
    Tables,
    Views,
    MaterializedViews,
    Procedures,
    Functions,
    Triggers,
    Sequences,
    Types,
    Packages,
}
```

### 3.6 Data Transfer Types

```rust
/// Column mapping between source and target.
#[derive(Debug, Clone)]
pub struct ColumnMapping {
    pub source_column: String,
    pub source_type: SqlType,
    pub target_column: String,
    pub target_type: SqlType,
    pub action: MappingAction,
}

#[derive(Debug, Clone)]
pub enum MappingAction {
    Map,                       // Direct mapping (with conversion if needed)
    Skip,                      // Ignore this source column
    Constant(String),          // Use a constant value
}

/// Error handling policy for data transfer.
#[derive(Debug, Clone, Copy)]
pub enum ErrorPolicy {
    AbortOnFirst,
    SkipAndContinue,
    MaxErrors(u32),
}
```

---

## Data Models

### Trait: `DatabaseDriver`

```rust
/// Abstraction over database-specific driver implementations.
#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    fn name(&self) -> &str;
    fn dialect(&self) -> SqlDialect;

    async fn connect(&self, descriptor: &ConnectionDescriptor) -> Result<Box<dyn DbConnection>>;
    async fn test_connection(&self, descriptor: &ConnectionDescriptor) -> Result<()>;
}

/// An active database connection.
#[async_trait]
pub trait DbConnection: Send + Sync {
    async fn execute(&self, sql: &str, params: &[SqlValue]) -> Result<ExecutionResult>;
    async fn query_stream(&self, sql: &str, params: &[SqlValue]) -> Result<RowStream>;
    async fn begin_transaction(&self) -> Result<()>;
    async fn commit(&self) -> Result<()>;
    async fn rollback(&self) -> Result<()>;
    async fn cancel(&self);
    async fn is_valid(&self) -> bool;
    async fn close(self: Box<Self>) -> Result<()>;

    // Metadata
    async fn schemas(&self) -> Result<Vec<String>>;
    async fn tables(&self, schema: &str) -> Result<Vec<TableInfo>>;
    async fn columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnDef>>;
    async fn indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>>;
    async fn foreign_keys(&self, schema: &str, table: &str) -> Result<Vec<ForeignKeyInfo>>;
}
```

### Public API Functions

```rust
// Plugin entry point
pub fn create_plugin() -> Box<dyn FileForgePlugin>;

// Driver registry
impl DriverRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, definition: DriverDefinition);
    pub fn list_drivers(&self) -> &[DriverDefinition];
    pub fn find_by_name(&self, name: &str) -> Option<&DriverDefinition>;
    pub fn find_by_platform(&self, platform: &str) -> Vec<&DriverDefinition>;
    pub fn load_from_toml(&mut self, path: &Path) -> Result<()>;
    pub fn save_to_toml(&self, path: &Path) -> Result<()>;
}

// Connection manager
impl ConnectionManager {
    pub fn new(registry: Arc<DriverRegistry>, credential_store: Arc<CredentialStore>) -> Self;
    pub async fn connect(&self, descriptor: &ConnectionDescriptor) -> Result<ConnectionId>;
    pub async fn disconnect(&self, id: ConnectionId) -> Result<()>;
    pub async fn test_connection(&self, descriptor: &ConnectionDescriptor) -> Result<()>;
    pub fn state(&self, id: ConnectionId) -> Option<ConnectionState>;
    pub fn list_connections(&self) -> Vec<(ConnectionId, ConnectionState)>;
}

// Connection pool
impl ConnectionPool {
    pub fn new(config: PoolConfig, driver: Arc<dyn DatabaseDriver>, descriptor: ConnectionDescriptor) -> Self;
    pub async fn acquire(&self) -> Result<PooledConnection>;
    pub async fn release(&self, conn: PooledConnection);
    pub fn active_count(&self) -> usize;
    pub fn idle_count(&self) -> usize;
    pub async fn shutdown(&self);
}
```

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum DatabaseToolError {
    #[error("[db] connection failed: {0}")]
    ConnectionFailed(String),

    #[error("[db] query execution error on connection {connection_id}: {message}")]
    QueryExecutionError { connection_id: String, message: String },

    #[error("[db] query timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("[db] authentication failed for user '{username}': {reason}")]
    AuthenticationFailed { username: String, reason: String },

    #[error("[db] driver not found: '{driver_name}'")]
    DriverNotFound { driver_name: String },

    #[error("[db] metadata retrieval failed for {object}: {reason}")]
    MetadataError { object: String, reason: String },

    #[error("[db] data transfer error at row {row}: {reason}")]
    DataTransferError { row: u64, reason: String },

    #[error("[db] operation cancelled")]
    Cancelled,

    #[error("[db] connection pool exhausted (max={max}, timeout={timeout_ms}ms)")]
    PoolExhausted { max: u32, timeout_ms: u64 },

    #[error("[db] SSH tunnel failed: {reason}")]
    SshTunnelFailed { reason: String },

    #[error("[db] credential store error: {reason}")]
    CredentialError { reason: String },

    #[error("[db] invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: ConnectionState, to: ConnectionState },

    #[error("[db] I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("[db] serialization error: {0}")]
    Serialization(String),
}
```

---

## Integration Points

### 6.1 ff-plugin

- `DatabasePlugin` implements `FileForgePlugin` trait
- Uses `PluginContext` to register commands, panels, capabilities
- Declares dependencies: `["ff-vfs", "ff-workflow"]`
- Advertises capabilities: `[Commands, Viewers, Providers]`

### 6.2 ff-command

- Registers all commands under `db.*` namespace via `CommandRegistration` trait
- Each command has: ID, display name, category, default shortcut, enabled predicate
- Data-modifying commands produce `UndoRecord` where feasible

### 6.3 ff-layout

- All panels implement `DockablePanel` trait
- Panels specify `default_dock_zone` per requirement 17
- Panels support float, re-dock, tab group, persona inclusion
- Provides "Database" persona layout configuration

### 6.4 ff-workflow

- Data transfer operations are `WorkflowDefinition` instances
- Each step is a `WorkflowStep` with progress reporting
- Uses `CancellationToken` from workflow engine
- Registered with `WorkflowRegistry` on plugin activation

### 6.5 ff-vfs

- All file I/O (script open/save, import/export files) goes through VFS API
- SQL scripts addressable via `vfs://` URIs
- Database tool does NOT register as a VFS provider

### 6.6 ff-connector-extensibility

- Connection abstraction follows the connector lifecycle pattern
- SSH tunnel follows connector network session model

---

## Correctness Properties

### Property 1: Driver Registry Lookup Consistency

**Statement:** For any registered driver `d`, querying by `d.name` returns `Some(d)`, and querying by each platform in `d.platforms` includes `d` in the result set.

**Validates: Requirements 2.1, 2.3**

### Property 2: Connection Pool Size Invariants

**Statement:** At any point in time, `pool.active_count() + pool.idle_count() <= pool.config.max_connections`, and `pool.idle_count() >= pool.config.min_connections` (when steady state is reached after warmup).

**Validates: Requirements 4.1, 4.2**

### Property 3: SQL Statement Boundary Parsing

**Statement:** For any valid SQL script (with statements separated by `;`), the parser identifies boundaries such that: (a) no statement text is lost (concatenating all parsed statements reproduces the original minus whitespace/delimiters), (b) delimiters within string literals, quoted identifiers, or comments are not treated as boundaries, (c) the number of parsed statements equals the number of top-level unquoted semicolons plus one for the trailing statement.

**Validates: Requirements 5.3**

### Property 4: Result Grid Batch Fetching

**Statement:** For a result set of `N` rows with batch size `B`, fetching batches sequentially yields exactly `ceil(N/B)` batches, the union of all batched rows equals the full result set, and no row appears in more than one batch.

**Validates: Requirements 8.3**

### Property 5: Column Mapping Validation

**Statement:** For any column mapping configuration, the mapping is valid if and only if: every target column maps to at most one source column (or constant), and mapped type pairs are in the set of compatible conversions. No two source columns map to the same target column via `MappingAction::Map`.

**Validates: Requirements 10.4, 10.20**

### Property 6: ER Diagram Layout Invariants

**Statement:** After auto-arrange, (a) no two entity boxes overlap (bounding rectangles do not intersect), (b) every relationship line connects exactly two entities present in the diagram, (c) all entities remain within the diagram canvas bounds.

**Validates: Requirements 11.1, 11.6**

---

## Testing Strategy

| Test Category | Framework | Location |
|---------------|-----------|----------|
| Unit tests | `#[cfg(test)]` | Each source module |
| Property tests | `proptest` | `tests/` directory |
| Integration tests | `#[tokio::test]` | `tests/integration/` |

Property-based tests use `proptest` with minimum 100 cases per property. Strategies generate:
- Random driver definitions for registry tests
- SQL scripts with embedded quotes/comments for parser tests
- Pool operations sequences for pool invariant tests
- Column mapping configurations for validation tests
- Entity/relationship sets for layout invariant tests
