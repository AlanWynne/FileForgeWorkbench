# Requirements Document

## Introduction

This feature specifies the Database Tool for FileForgeWorkbench — a full-featured integrated Database IDE delivered as a **workbench plugin** (`ff-database-tool` crate). The database tool provides an integrated database IDE within the FileForgeWorkbench ecosystem, drawing on established database tool patterns, providing: a connection management panel, SQL editor panel, result grid panel, schema browser panel, data transfer workflows, ER diagram panel, and database administration views.

The database tool is **not** a standalone application — it integrates with the workbench platform through:
- **Plugin Architecture** (`ff-plugin`): registers as a `FileForgePlugin`, contributes panels, commands, and capabilities via `PluginContext`
- **Command Framework** (`ff-command`): all user-facing database operations are registered commands with metadata, shortcuts, undo support
- **Layout and Docking** (`ff-layout`): all database panels implement `DockablePanel` and participate in the workbench layout system
- **Workflow Engine** (`ff-workflow`): data transfer, import/export, and bulk operations are modelled as state-machine workflows
- **Virtual File System** (`ff-vfs`): SQL scripts and export files are accessed through VFS; database connections may optionally register as VFS providers for future extensibility
- **Connector Extensibility** (`ff-connector-extensibility`): the database tool's connection abstraction follows the connector pattern for lifecycle management

**Rust/egui Adaptation:** Unlike DBeaver (Java/SWT), this tool uses:
- Async I/O via Tokio for all database operations (non-blocking UI)
- Rust database drivers: `sqlx` (PostgreSQL, MySQL, SQLite), `tokio-postgres`, `tiberius` (SQL Server), `rusqlite` (embedded SQLite), with a driver-agnostic trait abstraction
- egui immediate-mode rendering for all panels (grid, tree, diagram canvas, editor)
- No JDBC — a Rust-native driver registry replaces the JDBC driver model

**Source references:**
- **DBV** = DBeaver research files (tasks 16.1–16.7)
- **FFW-ARCH** = FileForgeWorkbench architecture specs (command-framework, plugin-architecture, layout-and-docking, workflow-engine, VFS, connector-extensibility)

## Glossary

- **DatabasePlugin**: The top-level `FileForgePlugin` implementation that bootstraps the database tool, registers all panels, commands, and workflows. [FFW-ARCH]
- **ConnectionManager**: The subsystem managing database connection lifecycle: creation, editing, connect/disconnect, pooling, credential storage. [DBV]
- **ConnectionDescriptor**: A serializable configuration record for a single database connection: driver, host, port, database, credentials reference, network settings. [DBV]
- **DriverRegistry**: The registry of available Rust database drivers, their capabilities, supported databases, and connection URL templates. [DBV]
- **SqlEditorPanel**: The DockablePanel providing multi-statement SQL editing with dialect-aware syntax highlighting, auto-complete, and query execution. [DBV, FFW-ARCH]
- **ResultGridPanel**: The DockablePanel displaying query result sets in a scrollable, sortable, filterable grid with cell editing support. [DBV, FFW-ARCH]
- **SchemaBrowserPanel**: The DockablePanel displaying a hierarchical tree of database objects (connections, schemas, tables, views, procedures). [DBV, FFW-ARCH]
- **ErDiagramPanel**: The DockablePanel rendering entity-relationship diagrams on a zoomable canvas with auto-layout. [DBV, FFW-ARCH]
- **DataTransferWorkflow**: A workflow-engine workflow modelling multi-step import/export/migration operations with progress and cancellation. [DBV, FFW-ARCH]
- **ConnectionPool**: A managed set of reusable database connections to a single target, providing connection reuse, validation, and idle timeout. [DBV]
- **QueryExecution**: An async operation that sends SQL to a database and streams results back, supporting cancellation and timeout. [DBV]
- **ExecutionPlan**: A structured representation of a database query's execution plan (tree of operations with cost/row estimates). [DBV]
- **DatabaseDriver**: A Rust-native trait abstracting over specific database client libraries (sqlx, tokio-postgres, tiberius, rusqlite). [DBV]

---

## Requirements

### Requirement 1: Plugin Registration and Lifecycle

**User Story:** As a workbench user, I want the database tool to load as a plugin that registers its panels, commands, and workflows with the platform, so that database capabilities integrate seamlessly with the rest of the workbench.

**Source:** FFW-ARCH plugin-architecture Reqs 1–5, command-framework Req 1, layout-and-docking Req 1. [FFW-ARCH]

#### Acceptance Criteria

1.1. THE `ff-database-tool` crate SHALL implement the `FileForgePlugin` trait, providing `initialize`, `activate`, `deactivate`, and `shutdown` lifecycle methods.

1.2. WHEN `initialize` is called, THE DatabasePlugin SHALL register all database-tool commands with the command registry via `PluginContext` (connection commands, SQL execution commands, schema browser commands, data transfer commands, ER diagram commands, admin commands).

1.3. WHEN `activate` is called, THE DatabasePlugin SHALL register all database-tool panels with the Panel_Registry (SchemaBrowserPanel, SqlEditorPanel, ResultGridPanel, ErDiagramPanel, ConnectionPanel, SessionManagerPanel, LockManagerPanel, DashboardPanel).

1.4. WHEN `activate` is called, THE DatabasePlugin SHALL register all data transfer workflows with the Workflow_Registry (import workflow, export workflow, cross-database transfer workflow, bulk load workflow).

1.5. WHEN `deactivate` is called, THE DatabasePlugin SHALL disconnect all active database connections gracefully, cancel any running queries or workflows, and deregister all capabilities from the platform.

1.6. WHEN `shutdown` is called, THE DatabasePlugin SHALL persist unsaved connection configurations, close all resources, and release all driver handles.

1.7. THE DatabasePlugin's `metadata` SHALL declare the plugin name as `"database-tool"`, declare capabilities `[Commands, Viewers, Providers]`, and specify dependencies on `ff-vfs` and `ff-workflow`.

1.8. ALL database tool panels SHALL implement the `DockablePanel` trait, providing `panel_id`, `default_dock_zone`, `title`, `render`, and `on_dock_state_changed` methods compatible with the workbench layout system.

---

### Requirement 2: Driver Registry

**User Story:** As a database tool user, I want a registry of available Rust database drivers with automatic capability detection, so that I can connect to any supported database without manual driver configuration.

**Source:** DBV-CORE §2 (Driver Registry), adapted to Rust-native drivers. [DBV]

#### Acceptance Criteria

2.1. THE DriverRegistry SHALL maintain a collection of available database drivers, each identified by a unique driver name and associated with: display name, supported database platforms, connection URL template, default port, required Rust crate name, and capabilities (read-only, read-write, transactions, streaming, prepared statements).

2.2. THE DriverRegistry SHALL ship with pre-configured driver definitions for: PostgreSQL (via `sqlx` or `tokio-postgres`), MySQL/MariaDB (via `sqlx`), SQLite (via `rusqlite` or `sqlx`), Microsoft SQL Server (via `tiberius`), and a generic ODBC bridge driver.

2.3. THE DriverRegistry SHALL support runtime discovery: listing all registered drivers, querying drivers by supported database platform, and querying driver capabilities.

2.4. THE DriverRegistry SHALL support user-defined custom driver entries that reference additional Rust database client libraries loaded as dynamic plugins.

2.5. WHEN a driver is selected for a new connection, THE DriverRegistry SHALL provide the URL template with placeholders (e.g., `postgres://{user}:{password}@{host}:{port}/{database}`) for automatic connection string construction.

2.6. THE DriverRegistry SHALL support driver-specific connection parameters (SSL mode, connection timeout, application name, timezone) exposed as typed key-value configuration per driver definition.

2.7. THE DriverRegistry SHALL persist driver configurations in a TOML file within the workbench configuration directory, loadable via the configuration-system.

---

### Requirement 3: Connection Management

**User Story:** As a database tool user, I want to create, edit, test, and manage database connections with credential security and SSH tunnelling, so that I can securely access any database from the workbench.

**Source:** DBV-CORE §1 (Connection Management), §4 (Credential Storage), §5 (SSH Tunnelling). [DBV]

#### Acceptance Criteria

3.1. THE ConnectionManager SHALL provide a connection creation wizard (command `db.connection.create`) that guides the user through: driver selection, host/port/database entry, authentication configuration, and network settings (SSL, SSH tunnel).

3.2. THE ConnectionManager SHALL construct the connection URL automatically from user-supplied parameters using the selected driver's URL template.

3.3. THE ConnectionManager SHALL provide a "Test Connection" command (`db.connection.test`) that validates connectivity by establishing and immediately closing a connection, reporting success or failure with diagnostic information.

3.4. THE ConnectionManager SHALL persist connection configurations in a TOML file (`connections.toml`) within the workbench data directory, with credentials stored separately in an encrypted credential store.

3.5. THE ConnectionManager SHALL support connection type classification with built-in types: Development (neutral), Test (green indicator), and Production (red indicator with confirmation-on-execute behaviour).

3.6. IF a connection type has confirmation-on-execute enabled, THEN THE system SHALL prompt the user with a confirmation dialog before executing any DML or DDL statement on that connection.

3.7. THE ConnectionManager SHALL support explicit connect and disconnect actions per connection, with visual state indicators (connected/disconnected/connecting/error) displayed in the SchemaBrowserPanel tree.

3.8. THE ConnectionManager SHALL support an "Invalidate/Reconnect" action (`db.connection.reconnect`) that closes a stale connection and re-establishes it.

3.9. THE ConnectionManager SHALL support multiple simultaneous connections to different databases within the same workbench session.

3.10. THE ConnectionManager SHALL store all sensitive credentials (passwords, tokens, SSH keys) in an encrypted credential store using OS-native keyring integration (Windows Credential Manager, macOS Keychain, Linux Secret Service) when available, with a fallback to an encrypted local file protected by a master password.

3.11. WHEN a connection is configured with "Save credentials" disabled, THE ConnectionManager SHALL prompt for credentials on each connection attempt without persisting them.

3.12. THE ConnectionManager SHALL support SSH tunnel configuration per connection, specifying: SSH host, port, username, and authentication method (password, private key file, or SSH agent).

3.13. WHEN an SSH tunnel is configured, THE ConnectionManager SHALL establish the SSH tunnel first (using an async SSH library compatible with Tokio) and route database traffic through the encrypted tunnel.

3.14. THE ConnectionManager SHALL support SSH jump hosts (gateway servers) for multi-hop SSH connections when the database is not directly reachable.

3.15. THE ConnectionManager SHALL support configurable idle timeout per connection that automatically disconnects after inactivity, and a keep-alive interval that sends periodic validation queries to maintain the connection.

3.16. THE ConnectionManager SHALL support bootstrap queries (session initialization SQL) that execute automatically after connection establishment.

3.17. THE ConnectionManager SHALL support connection import from CSV/TOML files and export of connection configurations (without credentials) for sharing.

3.18. THE ConnectionManager SHALL support network profiles — reusable bundles of SSH, SSL, and proxy settings applicable to multiple connections.

---

### Requirement 4: Connection Pooling

**User Story:** As a database tool user, I want connection pooling with validation and session isolation, so that multiple editors and operations can share connections efficiently without blocking each other.

**Source:** DBV-CORE §6 (Connection Pooling and Session Management). [DBV]

#### Acceptance Criteria

4.1. THE ConnectionPool SHALL manage a configurable pool of reusable database connections per ConnectionDescriptor, with minimum and maximum pool sizes.

4.2. WHEN a database operation requests a connection, THE ConnectionPool SHALL provide an idle connection from the pool or create a new one (up to the maximum); IF no connection is available and the pool is at maximum, THE request SHALL wait with a configurable timeout.

4.3. THE ConnectionPool SHALL validate connections before dispensing them using a driver-appropriate health check (e.g., `SELECT 1`) and SHALL automatically discard and replace invalid connections.

4.4. THE ConnectionPool SHALL support "Separate Connections" mode where each SQL editor tab uses its own independent connection for session isolation (temporary tables, transaction state).

4.5. THE ConnectionPool SHALL support a single shared connection mode where all operations for a given database share one physical connection.

4.6. THE ConnectionPool SHALL support configurable auto-commit mode per connection (on/off) and configurable transaction isolation level (Read Uncommitted, Read Committed, Repeatable Read, Serializable).

4.7. THE ConnectionPool SHALL indicate the current transaction mode (auto-commit ON/OFF) in the SQL editor status bar and provide explicit Commit and Rollback commands when auto-commit is disabled.

---

### Requirement 5: SQL Editor Panel

**User Story:** As a database developer, I want a SQL editor panel with multi-statement support, dialect-aware syntax highlighting, auto-complete, and integrated query execution, so that I can write, test, and refine SQL efficiently within the workbench.

**Source:** DBV-SQL §1–§3, §8. [DBV]

#### Acceptance Criteria

5.1. THE SqlEditorPanel SHALL implement `DockablePanel` with `default_dock_zone` of `Center` and SHALL support multiple concurrent instances, each associated with a database connection.

5.2. THE SqlEditorPanel SHALL provide a script buffer supporting editing of multiple SQL statements separated by a configurable statement delimiter (default: semicolon).

5.3. THE SqlEditorPanel SHALL parse the script buffer to identify individual statement boundaries, respecting string literals, quoted identifiers, comments, and nested blocks (BEGIN...END) so that delimiters inside these constructs are not treated as statement terminators.

5.4. THE SqlEditorPanel SHALL apply syntax highlighting rules specific to the SQL dialect of the active database connection (PostgreSQL, MySQL, T-SQL, PL/SQL, SQLite), highlighting: keywords, built-in functions, string/numeric literals, comments, operators, identifiers, and procedural block keywords.

5.5. THE SqlEditorPanel SHALL allow the user to configure colour and font style (bold, italic) for each syntax token category through the workbench theme system (cross-references `theme-and-appearance`).

5.6. WHEN the user presses the completion shortcut (default: Ctrl+Space) or types a trigger character (`.`), THE SqlEditorPanel SHALL display a completion popup with context-aware suggestions: table names, column names (after table alias + dot), schema-qualified objects, SQL keywords, and function names with parameter hints.

5.7. THE auto-complete SHALL resolve table aliases defined in the current query and provide column completions when the alias is used with a dot separator.

5.8. THE auto-complete popup SHALL filter suggestions as the user types and rank by relevance (exact prefix first, then fuzzy match, recently used prioritised).

5.9. THE SqlEditorPanel SHALL support SQL code formatting (command `db.sql.format`) that reformats selected SQL according to configurable rules (keyword case, indentation, line wrapping).

5.10. THE SqlEditorPanel SHALL support code folding for procedural blocks (BEGIN...END, CREATE PROCEDURE/FUNCTION bodies) and bracket matching with highlight.

5.11. THE SqlEditorPanel SHALL visually indicate the boundaries of the current statement (the statement that would be executed by "Execute Statement") using a background highlight or margin indicator.

5.12. THE SqlEditorPanel SHALL support SQL templates (code snippets) insertable by abbreviation + trigger key, expanding into full SQL with editable placeholders.

5.13. THE SqlEditorPanel SHALL display line numbers in a gutter with support for gutter indicators (problem markers, bookmarks).

5.14. THE SqlEditorPanel SHALL support saving the current script to a file via VFS and maintain a list of recently opened SQL scripts.

5.15. WHEN the user Ctrl+clicks on a database object name, THE SqlEditorPanel SHALL navigate to that object in the SchemaBrowserPanel.

---

### Requirement 6: Query Execution

**User Story:** As a database developer, I want to execute SQL statements and scripts with async non-blocking execution, cancellation, and result display, so that I can run queries without freezing the UI and inspect results immediately.

**Source:** DBV-SQL §4–§5, §6–§7. [DBV]

#### Acceptance Criteria

6.1. WHEN the user invokes "Execute SQL Statement" (command `db.sql.execute_statement`, default: Ctrl+Enter), THE system SHALL identify the single statement at the cursor position and execute it asynchronously against the active connection, displaying results in the ResultGridPanel upon completion.

6.2. WHEN the user selects text and invokes "Execute SQL Statement", THE system SHALL execute only the selected text as the SQL statement, regardless of delimiter boundaries.

6.3. WHEN the user invokes "Execute SQL Script" (command `db.sql.execute_script`, default: Alt+X), THE system SHALL split the entire script into individual statements and execute them sequentially, reporting per-statement success/failure in an execution log panel.

6.4. WHEN the user invokes "Execute in new tab" (command `db.sql.execute_new_tab`), THE system SHALL execute the current statement and display results in a new ResultGridPanel tab rather than replacing existing results.

6.5. THE system SHALL execute all queries asynchronously on the Tokio runtime so that the editor UI remains fully responsive during long-running query execution.

6.6. WHEN a query is executing, THE system SHALL display a progress indicator and provide a Cancel command (`db.sql.cancel`) that sends a cancellation signal to the database driver, aborting the in-progress query.

6.7. IF a configurable query timeout is set, THEN THE system SHALL automatically cancel queries that exceed the timeout duration and report the timeout to the user.

6.8. AFTER each statement execution, THE system SHALL display execution statistics: elapsed time, number of rows affected or returned, and any server-reported warnings.

6.9. THE system SHALL maintain an execution log panel recording all executed statements with timestamps, duration, row count, and success/error status for the current session.

6.10. WHEN the user invokes "Explain Execution Plan" (command `db.sql.explain`, default: Ctrl+Shift+E), THE system SHALL generate the execution plan for the current statement and display it as a hierarchical tree showing operations, estimated cost, and row counts.

6.11. THE execution plan view SHALL support a visual graph representation with nodes colour-coded by relative cost to highlight performance bottlenecks.

6.12. THE execution plan view SHALL support database-specific EXPLAIN options (e.g., ANALYZE, VERBOSE, BUFFERS for PostgreSQL) configurable before plan generation.

---

### Requirement 7: Parameter Binding

**User Story:** As a database developer, I want the SQL editor to detect parameter placeholders and prompt for values before execution, so that I can run parameterized queries without manually substituting values.

**Source:** DBV-SQL §7 (Parameter Binding). [DBV]

#### Acceptance Criteria

7.1. WHEN the SQL editor detects parameter placeholders in the script (`$1`, `:name`, `@variable`), THE system SHALL identify them as bind parameters requiring values before execution.

7.2. WHEN execution is invoked on a statement containing unresolved parameters, THE system SHALL display a dialog prompting for each parameter value, showing parameter name/position and an input field with optional type specification.

7.3. THE system SHALL support named parameters (`:employee_id`, `@salary`) where the same name used multiple times binds to a single value provided once.

7.4. THE system SHALL support positional parameters (`$1`, `$2`) where each placeholder is bound by ordinal position.

7.5. THE parameter dialog SHALL allow the user to specify the SQL data type for each parameter (VARCHAR, INTEGER, DATE, TIMESTAMP, BOOLEAN) for correct type marshalling.

7.6. THE system SHALL support client-side variable assignment (`@set variable = value`) for reuse across multiple statements without re-prompting.

7.7. THE system SHALL provide a Variables panel displaying all currently assigned variables and their values, allowing interactive modification.

7.8. THE system SHALL support configurable parameter pattern recognition (enable/disable `$N`, `:name`, `@variable` patterns) to avoid false-positive detection in dialect-specific syntax.

---

### Requirement 8: Result Grid Panel

**User Story:** As a database developer, I want query results displayed in a performant scrollable grid with sorting, filtering, cell editing, and export capabilities, so that I can inspect and manipulate data directly.

**Source:** DBV-DATA §1–§7 (Data Viewer). [DBV]

#### Acceptance Criteria

8.1. THE ResultGridPanel SHALL implement `DockablePanel` with `default_dock_zone` of `Bottom` and SHALL display query results in a scrollable grid with column headers matching the query's output columns.

8.2. WHEN multiple queries produce result sets, THE ResultGridPanel SHALL display each in a separate tab, allowing navigation between them.

8.3. THE ResultGridPanel SHALL fetch rows from the database in configurable batch sizes (default: 200 rows) and support incremental scrolling — automatically fetching the next batch when the user scrolls past the last fetched row.

8.4. THE ResultGridPanel SHALL support client-side column sorting (ascending/descending toggle on header click) and multi-column sort with priority indicators.

8.5. THE ResultGridPanel SHALL provide a filter bar where the user can type SQL WHERE-clause expressions applied to the result set, plus column header dropdown filters for quick filtering.

8.6. THE ResultGridPanel SHALL allow column resizing by dragging borders, column reordering by drag-and-drop of headers, and column visibility toggling via a column management dialog.

8.7. THE ResultGridPanel SHALL display NULL values with a configurable visual representation (default: "[NULL]" in distinct greyed italic style) that is visually distinguishable from empty strings.

8.8. THE ResultGridPanel SHALL display CLOB data as truncated text previews with a dedicated text viewer accessible via cell editor activation, and BLOB data as size/type indicators with hex viewer and image rendering for recognized formats (PNG, JPEG, GIF, BMP).

8.9. WHEN the user double-clicks a cell or presses Enter, THE ResultGridPanel SHALL make the cell editable inline, supporting typed input with type validation against the column's data type.

8.10. THE ResultGridPanel SHALL support row addition, row duplication, row deletion (marked for deletion with visual indicator), and cell-level Set to NULL / Set to Default actions.

8.11. WHEN the user activates "Save" (command `db.data.save`), THE ResultGridPanel SHALL generate and execute the appropriate SQL statements (INSERT, UPDATE, DELETE) to persist all pending changes to the database.

8.12. THE ResultGridPanel SHALL support a "Preview SQL" action that displays the generated SQL for pending changes without executing them.

8.13. THE ResultGridPanel SHALL support both auto-commit mode (changes committed immediately) and manual-commit mode (explicit Commit/Rollback required).

8.14. IF a table lacks a unique key, THEN THE ResultGridPanel SHALL allow the user to define a virtual unique key from one or more columns to enable row identification for editing.

8.15. THE ResultGridPanel SHALL support copying selected cells/rows to the clipboard in multiple formats (TAB-delimited, CSV, JSON, SQL INSERT, Markdown) and provide export to file (CSV, JSON, SQL, XML, HTML, Markdown) via the data transfer workflow.

8.16. THE ResultGridPanel SHALL display a row count in the status area and support a "Calculate total row count" action that executes COUNT(*) against the source.

---

### Requirement 9: Schema Browser Panel

**User Story:** As a database developer, I want a hierarchical tree showing all database objects with lazy loading, inspection, DDL generation, and search, so that I can explore and manage database schemas efficiently.

**Source:** DBV-SCHEMA §1–§8. [DBV]

#### Acceptance Criteria

9.1. THE SchemaBrowserPanel SHALL implement `DockablePanel` with `default_dock_zone` of `Left` and SHALL display database objects in a hierarchical tree: Connection → Database → Schema → Object Category → Individual Objects.

9.2. WHEN the user expands a tree node, THE system SHALL load child objects on demand (lazy loading) via async metadata queries, displaying a loading indicator until retrieval completes.

9.3. THE system SHALL support object category nodes within each schema: Tables, Views, Materialized Views, Stored Procedures, Functions, Triggers, Sequences, User-Defined Types, and Packages (where supported by the database).

9.4. THE system SHALL display type-specific icons for each object category and individual object to visually distinguish tables from views, procedures from functions, etc.

9.5. WHEN the user selects a tree node, THE system SHALL display the object's properties in a details panel (Properties tab) showing metadata appropriate to the object type.

9.6. THE system SHALL support two view modes: Simple (schemas and tables/views only) and Advanced (all objects including system schemas, indexes, constraints, roles, tablespaces).

9.7. WHEN multiple connections are open, THE system SHALL display each as a top-level root node, allowing simultaneous exploration of multiple databases.

9.8. THE system SHALL support context menus on tree nodes providing type-relevant actions: Open Data, Edit, Rename, Drop, Refresh, Generate SQL, View Diagram, Filter.

9.9. WHEN the user applies a filter to a tree node, THE system SHALL restrict visible child objects to those matching the filter pattern (glob or regex) and persist the filter across sessions.

9.10. THE system SHALL support drag-and-drop of tree objects into the SQL editor to insert the fully-qualified object name at the cursor position.

9.11. WHEN the user opens a table object, THE system SHALL display a tabbed inspector with: Properties, Columns, Indexes, Constraints, Triggers, Partitions, Dependencies, DDL, and Data (preview) tabs.

9.12. THE Columns tab SHALL display all columns with: ordinal position, name, data type (with precision/scale/length), nullable flag, default value, auto-increment flag, and comment.

9.13. THE system SHALL support inline editing of column properties (name, type, nullable, default) with a "Persist" action that generates and executes ALTER TABLE statements.

9.14. WHEN the user opens a stored procedure or function, THE system SHALL display: Properties, Parameters, Source Code (with syntax highlighting and compilation support), and Dependencies tabs.

9.15. WHEN the user selects "Generate SQL → DDL" on any database object, THE system SHALL produce the complete CREATE statement including all sub-objects, syntactically correct for the target database dialect.

9.16. THE DDL generation SHALL support: CREATE, ALTER (incremental changes), and DROP statements, with configurable options for IF EXISTS guards, qualified names, and comment inclusion.

9.17. THE system SHALL provide a Dependencies tab for each object showing bidirectional dependencies: objects this object depends on, and objects that depend on this object (FK references, view definitions, procedure calls).

9.18. THE system SHALL provide a global metadata search (command `db.schema.search`) that finds objects by name pattern across schemas and connections, with type filtering (Tables, Views, Columns, Procedures, Functions, Triggers) and incremental results.

9.19. THE system SHALL provide a quick-filter text field in the SchemaBrowserPanel toolbar for filtering currently visible tree nodes by name without server queries.

9.20. THE system SHALL cache metadata locally to enable instant re-display of recent queries and reduce repeated server round-trips, with a manual Refresh action to update from the server.

---

### Requirement 10: Data Transfer Workflows

**User Story:** As a database user, I want wizard-driven data import, export, and cross-database transfer operations modelled as resumable workflows with progress, cancellation, and error handling, so that I can move data reliably between databases and files.

**Source:** DBV-TRANSFER §1–§7, FFW-ARCH workflow-engine Reqs 1–3. [DBV, FFW-ARCH]

#### Acceptance Criteria

10.1. ALL data transfer operations (import, export, cross-database transfer, bulk load) SHALL be implemented as Workflow_Definitions registered with the Workflow_Registry, using the workflow-engine's state machine, progress, and cancellation infrastructure.

10.2. THE import workflow SHALL guide the user through sequential steps: source file selection, format settings (delimiter, encoding, header), column mapping, preview, and execution — each step modelled as a Workflow_Step.

10.3. THE system SHALL support importing data from CSV, JSON, and XML files into existing database tables, with configurable format options per source type.

10.4. WHEN importing, THE system SHALL display a column mapping interface showing source columns mapped to target table columns, supporting skip, reorder, and constant-value assignment.

10.5. WHEN no target table exists for import, THE system SHALL offer to create a new table with column names and types inferred from the source file content.

10.6. THE export workflow SHALL support exporting data to: CSV, JSON, SQL INSERT statements, XML, HTML, Markdown, and plain text formats, with format-specific configuration options.

10.7. THE export workflow SHALL allow exporting from: a single table, multiple tables, a query result set, or the currently filtered view in the ResultGridPanel.

10.8. THE system SHALL support cross-database transfer (source table in one connection → target table in another connection) with automatic data type mapping between different database vendors.

10.9. WHEN the target table does not exist in cross-database transfer, THE system SHALL offer to create it automatically with type-mapped columns derived from the source table structure.

10.10. THE system SHALL support bulk loading using database-native mechanisms where available (PostgreSQL COPY, MySQL LOAD DATA equivalent via driver API) as an alternative to row-by-row INSERT for high throughput.

10.11. THE system SHALL batch multiple rows into single INSERT statements with configurable batch size (default: 200 rows) and configurable commit interval to control transaction size.

10.12. THE system SHALL provide configurable error handling policies: "Abort on first error", "Skip errors and continue", and "Maximum error count" threshold.

10.13. THE system SHALL maintain an error log during transfer recording: row number, source values, target column, error type, and error message for each failed row.

10.14. WHEN a data transfer completes, THE system SHALL display a summary: total rows processed, rows transferred, rows skipped, errors, and elapsed time.

10.15. ALL data transfer workflows SHALL execute asynchronously in background tasks (via Tokio), allowing the user to continue interacting with the workbench during transfer.

10.16. ALL data transfer workflows SHALL support cooperative cancellation via the workflow-engine's Cancellation_Token, completing the current batch before stopping.

10.17. THE system SHALL report transfer progress via the workflow-engine's Progress_Event system: rows processed, percentage (when total is known), transfer speed (rows/second), and estimated time remaining.

10.18. THE system SHALL allow saving a data transfer configuration as a reusable named task for repeated execution.

10.19. WHEN a transfer is cancelled, THE system SHALL leave already-committed data in place (no automatic rollback of committed batches) unless the entire operation was configured as a single transaction.

10.20. THE column mapping editor SHALL display source and target data types side-by-side, highlight type mismatches that may cause data loss, and perform automatic conversion for compatible types.

---

### Requirement 11: ER Diagram Panel

**User Story:** As a database architect, I want visual entity-relationship diagrams rendered on a zoomable canvas with auto-layout, notation styles, and export capabilities, so that I can visualize and document database schemas.

**Source:** DBV-ER §1–§11 (ER Diagram). [DBV]

#### Acceptance Criteria

11.1. THE ErDiagramPanel SHALL implement `DockablePanel` with `default_dock_zone` of `Center` and SHALL render entities as rectangular boxes with header (table name) and body (column list) on a scrollable, zoomable canvas using egui's painter API.

11.2. WHEN the user opens a diagram for a single table (command `db.diagram.view_table`), THE system SHALL display that table plus all directly related tables via foreign key relationships.

11.3. WHEN the user opens a schema-level diagram (command `db.diagram.view_schema`), THE system SHALL render all tables and views in the schema with their inter-relationships.

11.4. THE system SHALL render each foreign key relationship as a connecting line between entities, with solid lines for mandatory (NOT NULL FK) and dashed lines for optional (nullable FK) relationships.

11.5. THE system SHALL display cardinality indicators at each end of relationship lines, supporting three notation styles: IDEF1X (default), Crow's Foot, and Bachman — switchable via context menu or preferences.

11.6. THE system SHALL provide an "Auto-arrange layout" action (command `db.diagram.auto_arrange`) that repositions entities to minimize connection crossings and group related entities together.

11.7. THE system SHALL support manual entity repositioning by dragging, a configurable grid with snap-to-grid option, and a Pan tool for viewport scrolling.

11.8. THE system SHALL support connection routing types: "Shortest paths" (default) and "Orthogonal paths" (right-angled rectilinear lines).

11.9. THE system SHALL support zoom control (25%–200%) via toolbar dropdown and Zoom In/Out buttons, plus an Outline mini-map panel for navigating large diagrams.

11.10. THE system SHALL visually distinguish primary key columns (key icon) and foreign key columns (FK icon) within entity boxes, with configurable attribute visibility modes: All columns, Keys only, Primary key only, None (header only).

11.11. THE system SHALL support attribute style options: show icons, show data types, show nullability, show comments, show fully qualified names, sort alphabetically.

11.12. THE system SHALL support custom diagrams where the user explicitly selects tables to include by dragging from the SchemaBrowserPanel, including tables from different connections in a single diagram.

11.13. THE system SHALL support creation of virtual (logical) relationships in custom diagrams that do not modify the physical schema.

11.14. THE system SHALL provide export capabilities: save diagram as PNG, SVG, or GraphML format via a file chooser dialog (command `db.diagram.export`).

11.15. THE system SHALL persist custom diagrams as workbench project resources (entity list, positions, virtual relationships, notes, display settings) restorable across sessions.

11.16. THE system SHALL provide an Edit Mode toggle that allows visual schema modification (create tables, add columns, create foreign keys) with generated DDL preview before execution.

11.17. THE system SHALL provide a diagram search function (Ctrl+F) that highlights matching tables and columns on the canvas and scrolls to bring matches into view.

11.18. THE system SHALL support custom background colours per entity box (via context menu "Set color") and distinct colouring for cross-schema references.

11.19. THE system SHALL support keyboard navigation within diagrams for accessibility.

---

### Requirement 12: Database Administration

**User Story:** As a database administrator, I want session monitoring, lock inspection, storage information, performance dashboards, and user/role management within the workbench, so that I can perform administrative tasks without switching to external tools.

**Source:** DBV-ADMIN §1–§6 (Metadata and Admin). [DBV]

#### Acceptance Criteria

12.1. THE system SHALL provide a Session Manager panel (command `db.admin.sessions`) displaying all active database sessions in a tabular list with: session/process ID, username, client application, current database, status (active/idle/waiting), connection time, and currently executing SQL.

12.2. THE Session Manager SHALL support filtering by active-only/all sessions, searching by username or SQL content, and configurable auto-refresh at user-defined intervals.

12.3. THE Session Manager SHALL provide "Kill Session" and "Disconnect Session" actions (with confirmation) that execute the database-appropriate termination command (e.g., `pg_terminate_backend()` for PostgreSQL, `KILL` for MySQL/SQL Server).

12.4. THE system SHALL provide a Lock Manager panel (command `db.admin.locks`) displaying all active database locks with: lock type, lock mode, locked object, holding session ID, waiting session ID, and lock duration.

12.5. THE Lock Manager SHALL display blocking chains (session A blocks B blocks C) and detect potential deadlock situations with visual lock-wait graph representation.

12.6. THE system SHALL display tablespace/storage information in the SchemaBrowserPanel tree under a "Storage" node, showing: tablespace name, status, type, total/used/free size, and percentage utilization with visual indicators for capacity thresholds.

12.7. THE system SHALL provide a Dashboard panel (command `db.admin.dashboard`) displaying real-time performance charts: connections over time, transactions per second, cache hit ratios, and I/O throughput, updated at configurable intervals (default: 1000ms).

12.8. THE Dashboard SHALL support multiple chart types (bar, pie, time series) with configurable SQL query sources, and SHALL allow creation of custom dashboard charts.

12.9. THE system SHALL display user/role information in the SchemaBrowserPanel under a "Security" node, with inspection tabs for: General properties, Privileges, Role Membership, and Object Privileges.

12.10. THE system SHALL support user creation (command `db.admin.create_user`), modification, and deletion with DDL preview before execution.

12.11. THE system SHALL provide GRANT and REVOKE interfaces for managing system privileges, object privileges, and role membership.

12.12. THE system SHALL provide a server configuration viewer displaying all runtime parameters with: name, current value, description, dynamic/static flag, and scope — with inline editing for dynamic parameters.

12.13. THE system SHALL provide a Query Manager log recording all SQL executed in the session with: SQL text, execution time, duration, rows affected, connection, and error status — with filtering by date, type, and content.

12.14. THE system SHALL adapt administrative features to each connected database platform, showing only relevant tools (e.g., tablespaces for Oracle/PostgreSQL, InnoDB metrics for MySQL).

---

### Requirement 13: Async I/O and Concurrency

**User Story:** As a workbench user, I want all database operations to be non-blocking and cancellable, so that the UI remains responsive during long queries, bulk transfers, and metadata loading.

**Source:** FFW-ARCH VFS async principle, workflow-engine cancellation, Rust/Tokio ecosystem. [FFW-ARCH]

#### Acceptance Criteria

13.1. ALL database I/O operations (query execution, metadata loading, data transfer, connection establishment) SHALL be async, executing on the Tokio runtime without blocking the egui render thread.

13.2. THE system SHALL use Rust database driver libraries that support async operation: `sqlx` (PostgreSQL, MySQL, SQLite), `tokio-postgres` (PostgreSQL), `tiberius` (SQL Server), `rusqlite` with `tokio::task::spawn_blocking` (SQLite when sqlx is not used).

13.3. ALL long-running operations SHALL support cooperative cancellation via `tokio::select!` and cancellation tokens, allowing the user to abort queries, transfers, and metadata loads at any point.

13.4. THE system SHALL stream large result sets incrementally (row-by-row or batch-by-batch) rather than loading entire result sets into memory, using async streaming interfaces provided by the database drivers.

13.5. METADATA loading (schema tree expansion, table property retrieval, dependency analysis) SHALL be performed asynchronously with loading indicators, not blocking the UI thread.

13.6. THE system SHALL enforce a configurable maximum concurrent query limit per connection to prevent resource exhaustion.

13.7. WHEN a database operation fails due to a transient error (connection timeout, network interruption), THE system SHALL attempt automatic reconnection (up to configurable retry count with exponential backoff) before reporting failure to the user.

---

### Requirement 14: Multi-Database Support

**User Story:** As a database developer working with multiple database platforms, I want the database tool to adapt its behaviour (SQL dialect, metadata navigation, admin tools, DDL generation) to each connected database, so that I have a consistent experience regardless of the target database.

**Source:** DBV-CORE §3 (Multi-Database Support), §3.2 (Database-Specific Behaviour). [DBV]

#### Acceptance Criteria

14.1. THE system SHALL provide pre-configured support for the following databases via Rust-native drivers: PostgreSQL, MySQL/MariaDB, SQLite, and Microsoft SQL Server.

14.2. THE system SHALL adapt metadata navigation (catalogs, schemas, tables, views, procedures) to the structure model of each connected database, hiding inapplicable hierarchy levels.

14.3. THE system SHALL adapt SQL syntax highlighting, auto-complete keywords, and DDL generation to the connected database's dialect.

14.4. THE system SHALL support database-specific authentication methods: username/password (all), peer/trust (PostgreSQL local), Windows SSPI (SQL Server), and certificate-based authentication where drivers support it.

14.5. THE system SHALL support database-specific administrative features: per-database session monitoring commands, lock inspection queries, storage model queries, and performance metrics — detected and adapted at connection time.

14.6. THE system SHALL provide an extensibility point (via the plugin architecture) for adding support for additional databases in the future through custom driver plugins that implement the DatabaseDriver trait.

14.7. THE DatabaseDriver trait SHALL abstract over driver-specific APIs, providing a unified interface for: connection establishment, query execution, result streaming, metadata retrieval, transaction control, and cancellation.

---

### Requirement 15: Command Integration

**User Story:** As a workbench user, I want all database operations available as registered commands with keyboard shortcuts, so that I can invoke them from the command palette, menus, keyboard, or macros consistently.

**Source:** FFW-ARCH command-framework Reqs 1–7. [FFW-ARCH]

#### Acceptance Criteria

15.1. ALL user-facing database operations SHALL be registered as commands in the command registry with dot-namespaced IDs under the `db.*` namespace (e.g., `db.connection.create`, `db.sql.execute_statement`, `db.schema.search`, `db.data.save`, `db.diagram.export`).

15.2. EACH database command SHALL have associated metadata: display name, description, category (e.g., `"db.connection"`, `"db.sql"`, `"db.schema"`, `"db.data"`, `"db.diagram"`, `"db.admin"`), and default keyboard shortcut where applicable.

15.3. EACH database command SHALL have an enabled predicate that evaluates to `true` only when the command is contextually applicable (e.g., `db.sql.execute_statement` is enabled only when a SQL editor panel is active and a connection is established).

15.4. DATABASE commands that modify data (INSERT, UPDATE, DELETE, DDL execution) SHALL be undoable where feasible — producing Undo_Records that allow rollback of the last change via the standard `edit.undo` command within the data editor.

15.5. ALL database commands SHALL be invocable from the Lua scripting bridge, enabling macro automation of database workflows (e.g., `workbench.execute("db.sql.execute_statement", {sql = "SELECT 1"})`).

15.6. THE system SHALL register the following default keyboard shortcuts (non-reserved, user-configurable): Ctrl+Enter (Execute Statement), Alt+X (Execute Script), Ctrl+Shift+E (Explain Plan), F5 (Refresh), Ctrl+Space (Auto-complete in SQL editor).

---

### Requirement 16: VFS Integration

**User Story:** As a workbench user, I want SQL scripts and export files managed through the VFS, so that the database tool benefits from the unified file abstraction (recent files, session restore, file tree visibility).

**Source:** FFW-ARCH VFS Reqs 1–3 (VFS Abstraction, URI Scheme, Provider Registry). [FFW-ARCH]

#### Acceptance Criteria

16.1. ALL file operations in the database tool (open script, save script, export data, import file) SHALL use the VFS API (`ff-vfs`) — no direct `std::fs` or `tokio::fs` calls.

16.2. SQL script files SHALL be addressable via VFS Resource_URIs (e.g., `vfs://local/path/to/script.sql`), enabling them to appear in the workbench file tree and recent files list.

16.3. Data export operations SHALL write output through the VFS, allowing export to any registered VFS provider (local filesystem, dataset catalog).

16.4. Data import operations SHALL read source files through the VFS, allowing import from any registered VFS provider.

16.5. THE database tool SHALL NOT register its own VFS provider for database content access (database connections are not filesystem-like resources); database access flows through the DatabaseDriver trait, not VFS.

---

### Requirement 17: Layout Integration

**User Story:** As a workbench user, I want database panels to participate fully in the workbench layout system — dockable, floatable, saveable in personas — so that I can arrange my database workspace alongside other workbench tools.

**Source:** FFW-ARCH layout-and-docking Reqs 1–3 (Panel System, Tab Groups, Floating Windows). [FFW-ARCH]

#### Acceptance Criteria

17.1. THE SchemaBrowserPanel SHALL register with `default_dock_zone` of `Left`, allowing it to dock alongside the file tree panel.

17.2. THE SqlEditorPanel SHALL register with `default_dock_zone` of `Center`, opening as tabs within the editor tab group area.

17.3. THE ResultGridPanel SHALL register with `default_dock_zone` of `Bottom`, appearing below the SQL editor in a split layout.

17.4. THE ErDiagramPanel SHALL register with `default_dock_zone` of `Center`, opening as editor tabs.

17.5. ALL database panels SHALL support undocking to floating windows, re-docking, tab group splits, and inclusion in workbench personas (named layout configurations).

17.6. WHEN multiple SQL editors are open, EACH SHALL appear as a separate tab in the center tab group with a title showing the script name or connection name.

17.7. THE database tool SHALL provide a "Database" persona that pre-configures the layout with SchemaBrowserPanel (left), SqlEditorPanel (center), ResultGridPanel (bottom), and Properties panel (right).

---

## Cross-Cutting Concerns

### Error Handling

All database tool errors SHALL use `thiserror` for structured error types with variants covering: connection failure, query execution error, timeout, authentication failure, driver not found, metadata retrieval failure, data transfer error, and cancellation. Application-level code SHALL use `anyhow` for context-enriched error chains per project coding standards.

### Logging

All significant database operations (connection events, query execution start/end, errors, transfer progress milestones) SHALL emit structured log records via the workbench logging subsystem (`ff-logging`) with appropriate levels: ERROR for failures, WARN for degraded conditions, INFO for lifecycle events, DEBUG for detailed operation traces.

### Configuration

Database tool settings (default fetch size, query timeout, auto-commit default, colour scheme for connection types, driver paths) SHALL be stored under the `[plugins.database-tool]` namespace in the workbench configuration system (TOML-based), accessible via `PluginContext` configuration API.

### Security

Credential handling SHALL follow the principle of least exposure: passwords never appear in log output, connection strings in logs mask the password component, and the credential store uses OS-native encryption where available.

---

## Source Reference Key

| Tag | Source |
|-----|--------|
| DBV | DBeaver Community Edition research (tasks 16.1–16.7) |
| DBV-CORE | DBeaver core research: connections, drivers, credentials, SSH, pooling |
| DBV-SQL | DBeaver SQL editor research: editing, highlighting, auto-complete, execution, explain plan, parameters |
| DBV-DATA | DBeaver data viewer research: grid, editing, filtering, sorting, export, LOB, NULL |
| DBV-SCHEMA | DBeaver schema browser research: tree navigation, inspection, DDL, dependencies, search |
| DBV-TRANSFER | DBeaver data transfer research: import, export, bulk load, cross-DB, column mapping, errors, progress |
| DBV-ER | DBeaver ER diagram research: canvas, relationships, layout, notation, export, persistence |
| DBV-ADMIN | DBeaver metadata/admin research: users, sessions, locks, storage, statistics, config |
| FFW-ARCH | FileForgeWorkbench architecture specs: command-framework, plugin-architecture, layout-and-docking, workflow-engine, VFS, connector-extensibility |

---

## Non-Functional Requirements

### Performance

- Query execution SHALL be fully async — the UI thread SHALL NOT block during any database operation.
- Schema tree expansion (lazy loading) SHALL complete within 3 seconds for schemas with up to 1,000 objects on a local database.
- Result grid SHALL render up to 200 rows without perceptible lag on a modern desktop.

### Reliability

- WHEN a database connection is lost, THE system SHALL detect the disconnection within 30 seconds and display a reconnection prompt.
- WHEN a query is cancelled, THE system SHALL release the database connection back to the pool within 5 seconds.

### Security

- Credentials SHALL never appear in log output or connection strings displayed in the UI.
- THE credential store SHALL use OS-native encryption (Windows Credential Manager, macOS Keychain, Linux Secret Service) where available.

### Scalability

- THE result grid SHALL support result sets of up to 1,000,000 rows via incremental batch fetching without loading the entire result set into memory.
- THE schema browser SHALL support databases with up to 10,000 tables without performance degradation in tree rendering.
