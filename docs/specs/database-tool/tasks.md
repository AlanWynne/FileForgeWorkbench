# Implementation Plan: Database Tool (`ff-database-tool`)

## Overview

This plan covers the complete implementation of the `ff-database-tool` crate — a full-featured integrated Database IDE delivered as a workbench plugin. The database tool provides: connection management, SQL editor, query execution, result grid, schema browser, ER diagram, data transfer workflows, and database administration panels.

This is a **Wave 6 (Application Tools)** sub-project, depending on: `ff-plugin`, `ff-command`, `ff-layout`, `ff-workflow`, `ff-vfs`, and `ff-connector-extensibility`.

---

## Dependency Graph (Wave-Based)

- **Wave A (Foundation):** Tasks 1–3 — Crate scaffold, error types, driver abstraction + registry. No intra-crate dependencies.
- **Wave B (Connection Layer):** Tasks 4–5 — Connection management, connection pooling, credentials. Depends on Wave A.
- **Wave C (SQL Engine):** Tasks 6–8 — SQL parser, SQL editor services, query execution + parameter binding. Depends on Wave B.
- **Wave D (Data Display):** Tasks 9–10 — Result grid panel, schema browser panel. Depends on Wave C.
- **Wave E (Advanced Features):** Tasks 11–13 — Data transfer workflows, ER diagram panel, administration panels. Depends on Wave D.
- **Wave F (Integration & Panels):** Tasks 14–16 — Panel UI, command integration, VFS integration, layout integration, plugin lifecycle. Depends on all above.

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Foundation Types", "tasks": ["2", "3"], "dependsOn": [0] },
    { "id": 2, "label": "Connection Layer", "tasks": ["4", "5"], "dependsOn": [1] },
    { "id": 3, "label": "SQL Engine", "tasks": ["6", "7", "8"], "dependsOn": [2] },
    { "id": 4, "label": "Data Display", "tasks": ["9", "10"], "dependsOn": [3] },
    { "id": 5, "label": "Advanced Features", "tasks": ["11", "12", "13"], "dependsOn": [4] },
    { "id": 6, "label": "Integration and Panels", "tasks": ["14", "15", "16"], "dependsOn": [5] }
  ]
}
```

---

## Tasks

### Wave A — Foundation

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-database-tool/Cargo.toml` with dependencies (tokio, sqlx, tokio-postgres, tiberius, rusqlite, thiserror, serde, toml, async-trait, proptest dev-dep, egui)
  - [ ] 1.2 Create `crates/ff-database-tool/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module directory structure: `driver/`, `connection/`, `sql/`, `execution/`, `result/`, `schema/`, `transfer/`, `diagram/`, `admin/`, `panel/`
  - [ ] 1.4 Create placeholder `mod.rs` for each module directory with documentation stubs
  - [ ] 1.5 Add `ff-database-tool` to workspace `Cargo.toml` members list
  - [ ] 1.6 Add upstream dependencies: `ff-plugin`, `ff-command`, `ff-layout`, `ff-workflow`, `ff-vfs`, `ff-connector-extensibility`
  - Covers: Structural foundation for all requirements

- [ ] 2. Error types and common types
  - [ ] 2.1 Define `DatabaseToolError` enum in `src/error.rs` with all variants: ConnectionFailed, QueryExecutionError, Timeout, AuthenticationFailed, DriverNotFound, MetadataError, DataTransferError, Cancelled, PoolExhausted, SshTunnelFailed, CredentialError, InvalidStateTransition, Io, Serialization
  - [ ] 2.2 Implement `Display` via `thiserror` with `[db] operation: description` format per coding standards
  - [ ] 2.3 Define `ConnectionId`, `ExecutionId` newtypes with `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`
  - [ ] 2.4 Define `SqlDialect` enum: PostgreSql, MySql, Sqlite, TSql, PlSql
  - [ ] 2.5 Define `SqlType` enum with all variants: Integer, BigInt, SmallInt, Float, Double, Decimal, Varchar, Text, Boolean, Date, Time, Timestamp, Blob, Clob, Json, Uuid, Array, Custom
  - [ ] 2.6 Define `IsolationLevel` enum and `SslMode` enum
  - [ ] 2.7 Write unit tests for error formatting, type Display impls, SqlType compatibility checks
  - Covers: Cross-cutting error handling, Requirements 13, 14

- [ ] 3. Driver abstraction and registry
  - [ ] 3.1 Define `DatabaseDriver` async trait in `src/driver/mod.rs` with: `name`, `dialect`, `connect`, `test_connection` methods
  - [ ] 3.2 Define `DbConnection` async trait with: `execute`, `query_stream`, `begin_transaction`, `commit`, `rollback`, `cancel`, `is_valid`, `close`, and metadata methods (`schemas`, `tables`, `columns`, `indexes`, `foreign_keys`)
  - [ ] 3.3 Define `DriverDefinition` struct with: name, display_name, platforms, url_template, default_port, crate_name, capabilities, connection_params
  - [ ] 3.4 Define `DriverCapabilities` struct with: read_only, read_write, transactions, streaming, prepared_statements, bulk_load flags
  - [ ] 3.5 Define `DriverParam` struct and `ParamType` enum for driver-specific connection parameters
  - [ ] 3.6 Implement `DriverRegistry` in `src/driver/registry.rs` with: `new`, `register`, `list_drivers`, `find_by_name`, `find_by_platform`, `load_from_toml`, `save_to_toml`
  - [ ] 3.7 Implement built-in driver definitions for: PostgreSQL, MySQL/MariaDB, SQLite, SQL Server, generic ODBC
  - [ ] 3.8 Implement TOML persistence for driver configurations
  - [ ] 3.9 Write unit tests for DriverRegistry CRUD operations and TOML round-trip
  - [ ] 3.10 Write property test: Driver Registry Lookup Consistency (Property 1)
  - Covers: Requirement 2 (AC 2.1–2.7), Requirement 14 (AC 14.1, 14.6, 14.7)

### Wave B — Connection Layer

- [ ] 4. Connection management
  - [ ] 4.1 Define `ConnectionDescriptor` struct in `src/connection/descriptor.rs` with all fields: id, name, driver_name, host, port, database, username, credential_ref, connection_type, network_profile, ssh_config, ssl_mode, pool_config, bootstrap_queries, idle_timeout, keepalive_interval, extra_params
  - [ ] 4.2 Define `ConnectionType` struct with: name, colour, confirm_on_execute
  - [ ] 4.3 Define `ConnectionState` enum: Disconnected, Connecting, Connected, Error
  - [ ] 4.4 Define `NetworkProfile` struct with SSH, SSL, and proxy settings bundle
  - [ ] 4.5 Define `SshConfig` struct with: host, port, username, auth_method, jump_hosts
  - [ ] 4.6 Implement `CredentialStore` in `src/connection/credential.rs` with OS keyring integration (Windows Credential Manager) and encrypted fallback
  - [ ] 4.7 Implement `ConnectionManager` in `src/connection/mod.rs` with: `new`, `connect`, `disconnect`, `reconnect`, `test_connection`, `state`, `list_connections`
  - [ ] 4.8 Implement SSH tunnel establishment using async SSH library compatible with Tokio
  - [ ] 4.9 Implement TOML persistence for connection descriptors (`connections.toml`)
  - [ ] 4.10 Implement connection import/export (CSV/TOML) without credentials
  - [ ] 4.11 Implement bootstrap queries execution on connection establishment
  - [ ] 4.12 Write unit tests for ConnectionDescriptor serialization, credential store, SSH config parsing
  - Covers: Requirement 3 (AC 3.1–3.18)

- [ ] 5. Connection pooling
  - [ ] 5.1 Define `PoolConfig` struct with: min_connections, max_connections, acquire_timeout_ms, idle_timeout_ms, validation_query, auto_commit, isolation_level, separate_connections
  - [ ] 5.2 Implement `ConnectionPool` in `src/connection/pool.rs` with: `new`, `acquire`, `release`, `active_count`, `idle_count`, `shutdown`
  - [ ] 5.3 Implement connection validation (health check) before dispensing from pool
  - [ ] 5.4 Implement acquire-with-timeout logic using `tokio::time::timeout`
  - [ ] 5.5 Implement idle connection eviction and minimum pool maintenance
  - [ ] 5.6 Implement separate-connections mode (one connection per SQL editor tab)
  - [ ] 5.7 Write unit tests for pool lifecycle: acquire/release, timeout, validation failure
  - [ ] 5.8 Write property test: Connection Pool Size Invariants (Property 2)
  - Covers: Requirement 4 (AC 4.1–4.7)

### Wave C — SQL Engine

- [ ] 6. SQL statement parser
  - [ ] 6.1 Implement statement boundary parser in `src/sql/parser.rs` that splits scripts on configurable delimiter
  - [ ] 6.2 Implement string literal recognition (single-quoted, dollar-quoted for PG, double-quoted identifiers)
  - [ ] 6.3 Implement comment recognition (line comments `--`, block comments `/* */`, nested for PG)
  - [ ] 6.4 Implement nested block recognition (BEGIN...END, CASE...END, IF...END IF) for procedural SQL
  - [ ] 6.5 Implement cursor-position-to-statement mapping (identify which statement the cursor is within)
  - [ ] 6.6 Write unit tests for: simple splits, quoted delimiters, comment-embedded delimiters, nested blocks, empty statements
  - [ ] 6.7 Write property test: SQL Statement Boundary Parsing (Property 3)
  - Covers: Requirement 5 (AC 5.2, 5.3, 5.11)

- [ ] 7. SQL editor services
  - [ ] 7.1 Implement dialect-aware syntax highlighting token classification in `src/sql/highlight.rs` (keywords, functions, literals, comments, operators, identifiers, procedural blocks per dialect)
  - [ ] 7.2 Implement auto-complete engine in `src/sql/complete.rs` with: table names, column names (with alias resolution), schema-qualified objects, keywords, function names with parameter hints
  - [ ] 7.3 Implement SQL code formatter in `src/sql/format.rs` with configurable rules (keyword case, indentation, line wrapping)
  - [ ] 7.4 Implement SQL template/snippet system in `src/sql/template.rs` with abbreviation expansion and editable placeholders
  - [ ] 7.5 Implement parameter placeholder detection in `src/sql/parameter.rs` supporting `$N`, `:name`, `@variable` patterns with configurable recognition
  - [ ] 7.6 Implement client-side variable assignment (`@set variable = value`) and variable store
  - [ ] 7.7 Write unit tests for: highlighting token sequences per dialect, auto-complete ranking, formatter output, parameter detection
  - Covers: Requirement 5 (AC 5.4–5.12), Requirement 7 (AC 7.1–7.8)

- [ ] 8. Query execution engine
  - [ ] 8.1 Implement `QueryExecution` model in `src/execution/mod.rs` with: id, sql, connection_id, state, cancel_token
  - [ ] 8.2 Implement async query executor in `src/execution/executor.rs` using `tokio::select!` for cancellation support
  - [ ] 8.3 Implement execute-single-statement (identify statement at cursor, execute, return result)
  - [ ] 8.4 Implement execute-script (split and execute sequentially, report per-statement results)
  - [ ] 8.5 Implement execute-selected-text (execute arbitrary selection)
  - [ ] 8.6 Implement query timeout via `tokio::time::timeout` with configurable duration
  - [ ] 8.7 Implement execution plan retrieval (EXPLAIN) with tree parsing into `PlanNode` hierarchy
  - [ ] 8.8 Implement execution log recording: statement text, timestamp, duration, row count, success/error
  - [ ] 8.9 Implement parameter binding dialog logic: detect placeholders, collect values, bind before execution
  - [ ] 8.10 Write unit tests for: executor state transitions, timeout behaviour, plan tree parsing, parameter binding
  - Covers: Requirement 6 (AC 6.1–6.12), Requirement 7 (AC 7.2–7.6), Requirement 13 (AC 13.1, 13.3, 13.5)

### Wave D — Data Display

- [ ] 9. Result grid data model and services
  - [ ] 9.1 Define `ResultSetHandle` in `src/result/mod.rs` with: columns, batch_size, fetched_rows, exhausted flag
  - [ ] 9.2 Implement batch-fetching logic in `src/result/batch.rs`: fetch next batch on demand, track exhaustion
  - [ ] 9.3 Implement client-side sorting (single-column and multi-column with priority)
  - [ ] 9.4 Implement client-side filtering (WHERE-expression parsing and evaluation against in-memory rows)
  - [ ] 9.5 Implement row editing model in `src/result/edit.rs`: pending inserts, updates, deletes with dirty tracking
  - [ ] 9.6 Implement DML generation from pending edits: INSERT, UPDATE, DELETE statements with proper quoting
  - [ ] 9.7 Implement clipboard/export formatting in `src/result/export.rs`: TAB-delimited, CSV, JSON, SQL INSERT, Markdown, HTML, XML
  - [ ] 9.8 Implement NULL display logic and type-appropriate cell rendering hints
  - [ ] 9.9 Write unit tests for: batch fetch sequencing, sort stability, DML generation correctness, export formatting
  - [ ] 9.10 Write property test: Result Grid Batch Fetching (Property 4)
  - Covers: Requirement 8 (AC 8.1–8.16)

- [ ] 10. Schema browser model and services
  - [ ] 10.1 Define `SchemaNode` enum in `src/schema/tree.rs` with all node variants (Connection, Database, Schema, Category, Table, View, Procedure, Function, Column, Index, Constraint, Trigger, Sequence)
  - [ ] 10.2 Implement lazy-loading tree expansion logic: async metadata fetch on node expand
  - [ ] 10.3 Implement per-driver metadata queries in `src/schema/metadata.rs` for: schemas, tables, views, columns, indexes, constraints, triggers, procedures, functions, sequences
  - [ ] 10.4 Implement DDL generation in `src/schema/ddl.rs` for: CREATE, ALTER, DROP with dialect-specific syntax, IF EXISTS guards, qualified names
  - [ ] 10.5 Implement dependency analysis: objects-depended-on and objects-depending-on (FK refs, view defs, procedure calls)
  - [ ] 10.6 Implement global metadata search in `src/schema/search.rs`: find objects by name pattern across schemas/connections with type filtering
  - [ ] 10.7 Implement metadata cache in `src/schema/cache.rs` with manual refresh
  - [ ] 10.8 Write unit tests for: tree construction, DDL generation per dialect, search filtering, cache invalidation
  - Covers: Requirement 9 (AC 9.1–9.20), Requirement 14 (AC 14.2, 14.3, 14.5)

### Wave E — Advanced Features

- [ ] 11. Data transfer workflows
  - [ ] 11.1 Define workflow step types in `src/transfer/mod.rs` as `WorkflowDefinition` implementations for import, export, cross-DB transfer, bulk load
  - [ ] 11.2 Implement import workflow steps in `src/transfer/import.rs`: file selection, format settings, column mapping, preview, execution
  - [ ] 11.3 Implement export workflow steps in `src/transfer/export.rs`: source selection, format configuration, output generation (CSV, JSON, SQL, XML, HTML, Markdown)
  - [ ] 11.4 Implement cross-database transfer in `src/transfer/cross_db.rs`: source query, type mapping, target insert with auto-create option
  - [ ] 11.5 Implement bulk load in `src/transfer/bulk.rs`: PostgreSQL COPY, MySQL LOAD DATA equivalents via driver API
  - [ ] 11.6 Implement column mapping engine in `src/transfer/column_map.rs`: map, skip, constant actions with type compatibility validation
  - [ ] 11.7 Implement error policy handling in `src/transfer/error_policy.rs`: abort-on-first, skip-and-continue, max-error-count with error log
  - [ ] 11.8 Implement batched INSERT with configurable batch size and commit interval
  - [ ] 11.9 Implement progress reporting via workflow-engine Progress_Event: rows processed, percentage, speed, ETA
  - [ ] 11.10 Implement cooperative cancellation via CancellationToken (complete current batch before stopping)
  - [ ] 11.11 Implement transfer configuration persistence as reusable named tasks
  - [ ] 11.12 Write unit tests for: column mapping validation, error policy enforcement, batch INSERT generation, progress calculation
  - [ ] 11.13 Write property test: Column Mapping Validation (Property 5)
  - Covers: Requirement 10 (AC 10.1–10.20)

- [ ] 12. ER diagram model and layout
  - [ ] 12.1 Define ER diagram data model in `src/diagram/mod.rs`: Entity (table box), Relationship (FK line), DiagramLayout (positions, settings)
  - [ ] 12.2 Implement auto-layout algorithm in `src/diagram/layout.rs` that minimizes connection crossings and groups related entities
  - [ ] 12.3 Implement notation rendering logic in `src/diagram/notation.rs` for: IDEF1X, Crow's Foot, Bachman cardinality styles
  - [ ] 12.4 Implement connection routing: shortest-path and orthogonal (rectilinear) line routing
  - [ ] 12.5 Implement entity attribute display modes: All columns, Keys only, Primary key only, None
  - [ ] 12.6 Implement diagram export in `src/diagram/export.rs`: PNG, SVG, GraphML format generation
  - [ ] 12.7 Implement diagram persistence in `src/diagram/persistence.rs`: save/restore entity positions, virtual relationships, display settings
  - [ ] 12.8 Implement virtual (logical) relationships that don't modify physical schema
  - [ ] 12.9 Write unit tests for: layout non-overlap, notation rendering, export format validity, persistence round-trip
  - [ ] 12.10 Write property test: ER Diagram Layout Invariants (Property 6)
  - Covers: Requirement 11 (AC 11.1–11.19)

- [ ] 13. Database administration services
  - [ ] 13.1 Implement session manager queries in `src/admin/session.rs`: list active sessions, filter, kill/disconnect with confirmation
  - [ ] 13.2 Implement lock manager in `src/admin/lock.rs`: list locks, blocking chains, deadlock detection
  - [ ] 13.3 Implement storage info queries in `src/admin/storage.rs`: tablespace list with size/usage/status
  - [ ] 13.4 Implement dashboard metrics in `src/admin/dashboard.rs`: connections, TPS, cache hit ratio, I/O throughput with configurable refresh
  - [ ] 13.5 Implement user/role management in `src/admin/security.rs`: list users, create, modify, delete, GRANT/REVOKE with DDL preview
  - [ ] 13.6 Implement server configuration viewer: list runtime parameters with metadata
  - [ ] 13.7 Implement query manager log: record all executed SQL with filtering
  - [ ] 13.8 Implement per-database platform adaptation: show only relevant admin tools per connected database type
  - [ ] 13.9 Write unit tests for: session query parsing, blocking chain construction, metric aggregation, GRANT/REVOKE DDL generation
  - Covers: Requirement 12 (AC 12.1–12.14), Requirement 14 (AC 14.5)

### Wave F — Integration and Panels

- [ ] 14. Panel implementations (egui DockablePanel)
  - [ ] 14.1 Implement `SchemaBrowserPanel` in `src/panel/schema_browser.rs`: DockablePanel with tree rendering, context menus, drag-to-editor, quick-filter toolbar
  - [ ] 14.2 Implement `SqlEditorPanel` in `src/panel/sql_editor.rs`: DockablePanel with script buffer, syntax highlighting, gutter, statement boundary highlight, code folding
  - [ ] 14.3 Implement `ResultGridPanel` in `src/panel/result_grid.rs`: DockablePanel with scrollable grid, column headers, batch scroll, sorting controls, filter bar, cell editing, row count display
  - [ ] 14.4 Implement `ErDiagramPanel` in `src/panel/er_diagram.rs`: DockablePanel with zoomable canvas, entity boxes, relationship lines, pan/zoom controls, mini-map
  - [ ] 14.5 Implement `ConnectionPanel` in `src/panel/connection.rs`: connection creation/edit wizard UI
  - [ ] 14.6 Implement `SessionManagerPanel` in `src/panel/session_manager.rs`: tabular session list with actions
  - [ ] 14.7 Implement `LockManagerPanel` in `src/panel/lock_manager.rs`: lock list with blocking chain visualisation
  - [ ] 14.8 Implement `DashboardPanel` in `src/panel/dashboard.rs`: real-time charts with configurable refresh
  - [ ] 14.9 Write unit tests for: panel creation, default dock zones, render state management
  - Covers: Requirement 17 (AC 17.1–17.7), Requirement 1 (AC 1.3, 1.8)

- [ ] 15. Command registration and integration
  - [ ] 15.1 Define all database commands with `db.*` namespace IDs in a commands module: connection commands, SQL commands, schema commands, data commands, diagram commands, admin commands
  - [ ] 15.2 Implement enabled predicates per command (context-sensitive activation)
  - [ ] 15.3 Register default keyboard shortcuts: Ctrl+Enter, Alt+X, Ctrl+Shift+E, F5, Ctrl+Space
  - [ ] 15.4 Implement Lua scripting bridge compatibility for all database commands
  - [ ] 15.5 Implement undo records for data-modifying commands (INSERT, UPDATE, DELETE in result grid)
  - [ ] 15.6 Write unit tests for: command registration, enabled predicate evaluation, shortcut mapping
  - Covers: Requirement 15 (AC 15.1–15.6)

- [ ] 16. Plugin lifecycle, VFS integration, and layout persona
  - [ ] 16.1 Implement `DatabasePlugin` in `src/plugin.rs`: `FileForgePlugin` trait with `initialize`, `activate`, `deactivate`, `shutdown`
  - [ ] 16.2 Implement `initialize`: register all `db.*` commands with command registry via PluginContext
  - [ ] 16.3 Implement `activate`: register all panels with Panel_Registry, register workflows with Workflow_Registry
  - [ ] 16.4 Implement `deactivate`: disconnect all connections, cancel running queries/workflows, deregister capabilities
  - [ ] 16.5 Implement `shutdown`: persist unsaved connection configs, close resources, release driver handles
  - [ ] 16.6 Implement plugin metadata: name `"database-tool"`, capabilities `[Commands, Viewers, Providers]`, dependencies on `ff-vfs` and `ff-workflow`
  - [ ] 16.7 Ensure all file I/O uses VFS API (open/save scripts, import/export files) — no direct fs calls
  - [ ] 16.8 Implement "Database" persona layout configuration: SchemaBrowser(Left), SqlEditor(Center), ResultGrid(Bottom), Properties(Right)
  - [ ] 16.9 Write unit tests for: lifecycle state transitions, capability registration/deregistration, VFS-only file access verification
  - Covers: Requirement 1 (AC 1.1–1.8), Requirement 16 (AC 16.1–16.5), Requirement 17 (AC 17.7)

---

## Acceptance Criteria Coverage Map

| Task | Requirements Covered |
|------|---------------------|
| 1 | Foundation (all) |
| 2 | Cross-cutting error handling, Req 13, 14 |
| 3 | Req 2 (AC 2.1–2.7), Req 14 (AC 14.1, 14.6, 14.7) |
| 4 | Req 3 (AC 3.1–3.18) |
| 5 | Req 4 (AC 4.1–4.7) |
| 6 | Req 5 (AC 5.2, 5.3, 5.11) |
| 7 | Req 5 (AC 5.4–5.12), Req 7 (AC 7.1–7.8) |
| 8 | Req 6 (AC 6.1–6.12), Req 7 (AC 7.2–7.6), Req 13 (AC 13.1, 13.3, 13.5) |
| 9 | Req 8 (AC 8.1–8.16) |
| 10 | Req 9 (AC 9.1–9.20), Req 14 (AC 14.2, 14.3, 14.5) |
| 11 | Req 10 (AC 10.1–10.20) |
| 12 | Req 11 (AC 11.1–11.19) |
| 13 | Req 12 (AC 12.1–12.14), Req 14 (AC 14.5) |
| 14 | Req 17 (AC 17.1–17.7), Req 1 (AC 1.3, 1.8) |
| 15 | Req 15 (AC 15.1–15.6) |
| 16 | Req 1 (AC 1.1–1.8), Req 16 (AC 16.1–16.5), Req 17 (AC 17.7) |

---

## Property-Based Test Summary

| Property # | Task | Test File | Statement |
|-----------|------|-----------|-----------|
| 1 | 3.10 | `tests/driver_registry_props.rs` | Registered drivers are findable by name and platform |
| 2 | 5.8 | `tests/connection_pool_props.rs` | Pool size invariants hold under concurrent acquire/release |
| 3 | 6.7 | `tests/sql_parser_props.rs` | Statement splitting preserves content, respects quoted/commented delimiters |
| 4 | 9.10 | `tests/result_batch_props.rs` | Batch fetching yields complete, non-overlapping row coverage |
| 5 | 11.13 | `tests/column_map_props.rs` | Column mapping validation enforces unique targets and type compatibility |
| 6 | 12.10 | `tests/er_layout_props.rs` | Auto-layout produces non-overlapping entities within canvas bounds |


---

## Notes

- This is a Wave 6 (Application Tools) crate depending on: `ff-plugin`, `ff-command`, `ff-layout`, `ff-workflow`, `ff-vfs`, `ff-connector-extensibility`
- All database I/O is async via Tokio; the egui render thread never blocks on database operations
- The `DatabaseDriver` trait abstracts over driver-specific APIs — concrete implementations for PostgreSQL, MySQL, SQLite, SQL Server ship built-in
- SSH tunnel management uses an async SSH library compatible with Tokio (e.g., `russh` or `async-ssh2-tokio`)
- Credential storage uses OS-native keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service) with encrypted local file fallback
- Connection pooling is internal to ff-database-tool (not using an external pool crate) to control validation, session isolation, and cancellation semantics
- The SQL parser is a lightweight boundary-detector, not a full SQL parser — it identifies statement boundaries while respecting string/comment/block nesting
- Data transfer workflows use the `ff-workflow` state machine infrastructure with cancellation tokens and progress events
- Panel rendering uses egui immediate-mode APIs; panels implement `DockablePanel` from `ff-layout`
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The ER diagram auto-layout uses a force-directed or layered graph algorithm (implementation choice deferred to task 12.2)
- Admin features adapt per-database: only relevant tools are shown for each connected database platform
- The database tool does NOT register as a VFS provider — database access flows through the DatabaseDriver trait, not VFS
- All file I/O (scripts, exports, imports) goes through `ff-vfs` exclusively — no direct `std::fs` or `tokio::fs` usage
