# DBeaver Schema Browser — Requirements Research

> **Source:** DBeaver Community/Pro public documentation and wiki analysis
> **Task:** 16.4 — Extract DBeaver schema browser requirements
> **Tag:** [DBV-SCHEMA]
> **Format:** EARS (Easy Approach to Requirements Syntax)

---

## 1. Object Tree Navigation (Hierarchical Database Navigator)

DBeaver's Database Navigator is the primary interface for exploring database structure. It presents a hierarchical tree: Connection → Server → Database → Schema → Object Categories (Tables, Views, Procedures, Functions, Triggers, Sequences, etc.). Users expand nodes on demand, with lazy loading for large schemas. The tree supports Simple view (schemas + tables only) and Advanced view (all objects including system objects, indexes, constraints, administrative utilities).

### Requirements

**1.1** [DBV-SCHEMA] THE schema browser panel SHALL display database objects in a hierarchical tree organised as: Connection → Server → Database → Schema → Object Category → Individual Objects.

**1.2** [DBV-SCHEMA] WHEN the user expands a tree node, THE system SHALL load child objects on demand (lazy loading) from the database metadata catalog, displaying a loading indicator until retrieval completes.

**1.3** [DBV-SCHEMA] THE system SHALL support the following object category nodes within each schema: Tables, Views, Materialized Views, Stored Procedures, Functions, Triggers, Sequences, User-Defined Types, Synonyms, and Packages (where the database engine supports them).

**1.4** [DBV-SCHEMA] THE system SHALL display a type-specific icon for each object category and each individual object to visually distinguish tables from views, procedures from functions, etc.

**1.5** [DBV-SCHEMA] WHEN the user selects a tree node, THE system SHALL display the object's properties in a details panel (Properties tab) adjacent to the tree.

**1.6** [DBV-SCHEMA] THE system SHALL support two view modes for the navigator tree:
- **Simple view**: shows only schemas and tables/views (hides system objects and administrative utilities)
- **Advanced view**: shows all database objects including system schemas, indexes, constraints, roles, tablespaces, and other administrative objects

**1.7** [DBV-SCHEMA] WHEN multiple connections are open, THE system SHALL display each connection as a top-level root node in the tree, allowing simultaneous exploration of multiple databases.

**1.8** [DBV-SCHEMA] THE system SHALL support context menus on tree nodes providing actions relevant to the object type (e.g., Open, Edit, Rename, Drop, Refresh, Generate SQL, View Diagram, Filter).

**1.9** [DBV-SCHEMA] WHEN the user applies a filter to a tree node, THE system SHALL restrict the visible child objects to those matching the filter pattern (glob or regex) and persist the filter across sessions.

**1.10** [DBV-SCHEMA] THE system SHALL support drag-and-drop of tree objects into the SQL editor to insert the fully-qualified object name at the cursor position.

**1.11** [DBV-SCHEMA] WHEN the user right-clicks a schema or table folder and selects "Refresh", THE system SHALL re-query the metadata catalog and update the tree to reflect external changes (objects created/dropped outside the tool).

**1.12** [DBV-SCHEMA] THE system SHALL allow the user to expand the tree to show sub-object folders for tables: Columns, Indexes, Constraints (Primary Key, Foreign Keys, Unique, Check), Triggers, and Partitions.

---

## 2. Table Inspection

DBeaver provides comprehensive table inspection via the Database Object Editor with multiple tabs: Properties (name, schema, row count, data size, creation date), Columns (name, type, nullable, default, comment), Indexes (name, type, columns, uniqueness), Constraints (PK, FK, Unique, Check — with referenced table/columns), Triggers (name, event, timing), and Statistics (row count, data length, index length, average row length).

### Requirements

**2.1** [DBV-SCHEMA] WHEN the user opens a table object, THE system SHALL display a tabbed inspector with the following tabs: Properties, Columns, Indexes, Constraints, Triggers, Partitions, Dependencies, DDL, and Data (preview).

**2.2** [DBV-SCHEMA] THE Properties tab SHALL display: table name, schema, owner, table type (base table, temporary, partitioned), engine/storage type, character set, collation, row count estimate, data size, index size, creation date, last modification date, and comment/description.

**2.3** [DBV-SCHEMA] THE Columns tab SHALL display a grid listing all columns with: ordinal position, column name, data type (with precision/scale/length), nullable flag, default value, auto-increment flag, computed/generated expression, and column comment.

**2.4** [DBV-SCHEMA] THE Indexes tab SHALL display: index name, index type (B-tree, Hash, GiST, GIN, etc.), uniqueness flag, column list (with order ASC/DESC and position), tablespace, and partial index predicate (where supported).

**2.5** [DBV-SCHEMA] THE Constraints tab SHALL display all constraints grouped by type:
- **Primary Key**: name, columns
- **Foreign Keys**: name, columns, referenced table, referenced columns, ON UPDATE rule, ON DELETE rule
- **Unique**: name, columns
- **Check**: name, expression

**2.6** [DBV-SCHEMA] THE Triggers tab for a table SHALL display: trigger name, event (INSERT/UPDATE/DELETE/TRUNCATE), timing (BEFORE/AFTER/INSTEAD OF), for-each mode (ROW/STATEMENT), and enabled/disabled status.

**2.7** [DBV-SCHEMA] THE system SHALL provide a Statistics sub-tab or section showing: exact or estimated row count, total data size, index size, average row length, fragmentation percentage, and last ANALYZE/statistics-refresh timestamp.

**2.8** [DBV-SCHEMA] WHEN the user clicks "Count Rows" or equivalent action, THE system SHALL execute a `SELECT COUNT(*)` against the table and display the exact row count, with a progress indicator for large tables.

**2.9** [DBV-SCHEMA] THE Columns grid SHALL support inline editing of column properties (name, type, nullable, default, comment) with a "Save/Persist" action that generates and executes the corresponding ALTER TABLE statement.

---

## 3. View Inspection

DBeaver displays views with their definition SQL (the SELECT statement), column listing (derived from the view definition), and properties (schema, owner, check option, updatability).

### Requirements

**3.1** [DBV-SCHEMA] WHEN the user opens a view object, THE system SHALL display a tabbed inspector with: Properties, Columns, Definition (SQL source), Dependencies, and DDL tabs.

**3.2** [DBV-SCHEMA] THE Definition tab SHALL display the full SQL SELECT statement that defines the view, rendered with syntax highlighting appropriate to the database dialect.

**3.3** [DBV-SCHEMA] THE Columns tab for a view SHALL display: column name, data type (resolved from the underlying query), nullable flag, and ordinal position.

**3.4** [DBV-SCHEMA] THE Properties tab for a view SHALL display: view name, schema, owner, view type (standard view, materialized view, system view), check option (NONE/LOCAL/CASCADED), is-updatable flag, and comment/description.

**3.5** [DBV-SCHEMA] IF the view is a materialized view, THEN THE system SHALL additionally display: refresh method (ON DEMAND/ON COMMIT), last refresh timestamp, storage size, and a "Refresh Materialized View" action.

**3.6** [DBV-SCHEMA] THE Definition tab SHALL support editing the view SQL source with a "Save/Persist" action that generates and executes `CREATE OR REPLACE VIEW` (or the dialect equivalent).

---

## 4. Procedure and Function Inspection

DBeaver provides source code viewing for stored procedures and functions across supported databases (MySQL, Oracle, PostgreSQL, SQL Server, DB2, Vertica, Firebird). The procedure editor shows parameters (name, type, direction IN/OUT/INOUT, default), source code body with syntax highlighting, and dependencies.

### Requirements

**4.1** [DBV-SCHEMA] WHEN the user opens a stored procedure or function, THE system SHALL display a tabbed inspector with: Properties, Parameters, Source Code, Dependencies, and DDL tabs.

**4.2** [DBV-SCHEMA] THE Parameters tab SHALL display a grid listing all parameters with: ordinal position, parameter name, data type (with precision/scale/length), direction (IN, OUT, INOUT, RETURN), and default value (where supported).

**4.3** [DBV-SCHEMA] THE Source Code tab SHALL display the full procedure/function body with syntax highlighting appropriate to the database dialect (PL/SQL, PL/pgSQL, T-SQL, etc.).

**4.4** [DBV-SCHEMA] THE Source Code tab SHALL support editing with a "Compile/Save" action that submits the modified source to the database and reports compilation errors (with line numbers) inline.

**4.5** [DBV-SCHEMA] THE Properties tab for a procedure/function SHALL display: name, schema, owner, language (SQL, PL/pgSQL, Java, C, etc.), deterministic/volatile flag, security definer/invoker, return type (for functions), and comment/description.

**4.6** [DBV-SCHEMA] THE Dependencies tab SHALL show:
- **Objects this procedure/function depends on**: tables, views, other procedures/functions referenced in the body
- **Objects that depend on this procedure/function**: other routines, views, triggers that call it

**4.7** [DBV-SCHEMA] WHEN a procedure/function compilation fails, THE system SHALL display error messages with line number and column position, and highlight the offending line in the source editor.

**4.8** [DBV-SCHEMA] THE system SHALL support viewing overloaded procedures/functions (same name, different parameter signatures) as distinct entries in the tree and inspector.

---

## 5. Trigger Inspection

DBeaver displays triggers with their event, timing, body source code, and associated table. Triggers appear both under the parent table's Triggers sub-folder and in a top-level Triggers folder within the schema.

### Requirements

**5.1** [DBV-SCHEMA] WHEN the user opens a trigger object, THE system SHALL display a tabbed inspector with: Properties, Source Code, and DDL tabs.

**5.2** [DBV-SCHEMA] THE Properties tab for a trigger SHALL display: trigger name, schema, associated table/view, event type (INSERT, UPDATE, DELETE, TRUNCATE, or combination), timing (BEFORE, AFTER, INSTEAD OF), orientation (FOR EACH ROW, FOR EACH STATEMENT), condition (WHEN clause if applicable), enabled/disabled status, and execution order (if multiple triggers on same event).

**5.3** [DBV-SCHEMA] THE Source Code tab SHALL display the full trigger body with syntax highlighting, and support editing with a "Save/Persist" action that recreates the trigger.

**5.4** [DBV-SCHEMA] THE system SHALL display triggers in two locations within the tree:
- Under the associated table's "Triggers" sub-folder
- Under the schema-level "Triggers" category folder

**5.5** [DBV-SCHEMA] THE system SHALL support enabling and disabling triggers via context menu action (generating `ALTER TABLE ... ENABLE/DISABLE TRIGGER` or the dialect equivalent).

**5.6** [DBV-SCHEMA] IF the trigger references transition tables (OLD TABLE / NEW TABLE) or transition variables (OLD / NEW), THEN THE Properties tab SHALL display those references.

---

## 6. DDL Generation

DBeaver can generate SQL DDL scripts from any database object via the context menu "Generate SQL → DDL". This produces CREATE statements for tables (with columns, constraints, indexes), views, procedures, triggers, sequences, and entire schemas. It also supports ALTER statement generation for pending modifications and DROP statements.

### Requirements

**6.1** [DBV-SCHEMA] WHEN the user selects "Generate SQL → DDL" on any database object, THE system SHALL produce the complete CREATE statement for that object, including all dependent sub-objects (columns, constraints, indexes for tables; parameters for procedures).

**6.2** [DBV-SCHEMA] THE DDL generation SHALL support the following statement types:
- **CREATE**: full object definition
- **ALTER**: incremental change statements reflecting pending editor modifications
- **DROP**: DROP statement with optional CASCADE/RESTRICT qualifier

**6.3** [DBV-SCHEMA] THE generated DDL SHALL be syntactically correct for the target database dialect (MySQL, PostgreSQL, Oracle, SQL Server, SQLite, DB2, etc.) using dialect-specific syntax, quoting, and type names.

**6.4** [DBV-SCHEMA] WHEN generating DDL for a table, THE system SHALL include: CREATE TABLE with columns and inline constraints, followed by separate statements for indexes, foreign keys, triggers, comments, grants, and partitioning (as applicable).

**6.5** [DBV-SCHEMA] THE system SHALL support generating DDL for multiple selected objects simultaneously, producing a combined script in dependency order (referenced objects before referencing objects).

**6.6** [DBV-SCHEMA] THE generated DDL SHALL be displayed in a new editor window with syntax highlighting, and provide options to: copy to clipboard, save to file, or execute directly.

**6.7** [DBV-SCHEMA] THE DDL generation SHALL support configuration options including: include/exclude DROP statement prefix, include/exclude IF EXISTS/IF NOT EXISTS guards, include/exclude comments/descriptions, include/exclude grants/permissions, qualified vs. unqualified names.

**6.8** [DBV-SCHEMA] WHEN the DDL tab is displayed for an object in the inspector, THE system SHALL show the current DDL representation of the object as stored in the database (reverse-engineered from metadata).

**6.9** [DBV-SCHEMA] THE system SHALL support generating DDL for an entire schema, producing CREATE statements for all objects in dependency order with appropriate DROP-IF-EXISTS guards.

---

## 7. Dependency Graph

DBeaver provides dependency viewing for database objects showing which objects reference which. This includes foreign key relationships, view dependencies on tables, procedure/function dependencies on tables and other routines, trigger dependencies, and synonym resolution. Dependencies are shown both as a list (tab in the object editor) and visually in ER diagrams.

### Requirements

**7.1** [DBV-SCHEMA] THE system SHALL provide a Dependencies tab for each database object showing two directions:
- **Uses (depends on)**: objects that this object references or depends upon
- **Used by (dependents)**: objects that reference or depend on this object

**7.2** [DBV-SCHEMA] THE dependency analysis SHALL detect the following relationship types:
- Foreign key references between tables
- View definitions referencing tables, views, and functions
- Procedure/function bodies referencing tables, views, and other routines
- Trigger bodies referencing tables and procedures
- Synonym targets
- Materialized view base tables
- Partition parent/child relationships

**7.3** [DBV-SCHEMA] THE Dependencies tab SHALL display each dependency as: referenced object name, object type, relationship type (FK, view reference, procedure call, etc.), and schema.

**7.4** [DBV-SCHEMA] WHEN the user clicks a dependency entry, THE system SHALL navigate to the referenced/dependent object in the tree and open its inspector.

**7.5** [DBV-SCHEMA] THE system SHALL support a visual dependency graph view showing objects as nodes and dependencies as directed edges, with layout options (hierarchical top-down, left-right, radial).

**7.6** [DBV-SCHEMA] THE visual dependency graph SHALL support:
- Expanding/collapsing dependency levels (1-hop, 2-hop, full transitive closure)
- Filtering by relationship type
- Highlighting the path between two selected objects
- Exporting the graph as an image (PNG/SVG) or PDF

**7.7** [DBV-SCHEMA] WHEN the user attempts to DROP an object that has dependents, THE system SHALL warn about dependent objects and display the full dependency chain that would be affected.

**7.8** [DBV-SCHEMA] THE system SHALL detect circular dependencies and display them without entering an infinite loop, marking cycles visually in the dependency graph.

---

## 8. Search Across Objects

DBeaver provides two search mechanisms: Metadata Search (Ctrl+H) for finding objects by name across the schema, and Full-Text Data Search for finding data within table contents. Metadata Search allows filtering by object type (tables, views, procedures, columns, constraints, indexes) and searching across multiple connections simultaneously.

### Requirements

**8.1** [DBV-SCHEMA] THE system SHALL provide a global metadata search function (keyboard shortcut accessible) that finds database objects by name pattern across the entire schema or multiple connections.

**8.2** [DBV-SCHEMA] THE metadata search SHALL support:
- Substring matching (contains)
- Prefix matching (starts with)
- Wildcard/glob patterns (e.g., `user*`, `*_log`)
- Case-insensitive matching by default with an option for case-sensitive

**8.3** [DBV-SCHEMA] THE metadata search SHALL allow the user to filter by object type, with individually selectable checkboxes for: Tables, Views, Columns, Indexes, Constraints, Procedures, Functions, Triggers, Sequences, Schemas, and Synonyms.

**8.4** [DBV-SCHEMA] WHEN the user specifies "Columns" as a search target, THE system SHALL search column names within all tables and views, returning results as `table.column` qualified references.

**8.5** [DBV-SCHEMA] THE search results SHALL display: object name, object type (with icon), parent schema, parent table (for sub-objects like columns/indexes), and a relevance indicator or match highlight.

**8.6** [DBV-SCHEMA] WHEN the user double-clicks a search result, THE system SHALL navigate to that object in the tree navigator and open its inspector/editor.

**8.7** [DBV-SCHEMA] THE metadata search SHALL support searching across multiple database connections simultaneously, with results grouped by connection.

**8.8** [DBV-SCHEMA] THE system SHALL provide incremental/as-you-type search results with debounced queries to avoid excessive metadata catalog queries.

**8.9** [DBV-SCHEMA] THE system SHALL provide a quick-filter text field in the navigator tree toolbar that filters currently visible tree nodes by name pattern without querying the server.

**8.10** [DBV-SCHEMA] THE system SHALL cache metadata search results locally to enable instant re-display of recent searches and reduce repeated server round-trips.

---

## Summary

| Area | Requirements | Key Capabilities |
|------|-------------|-----------------|
| Object Tree Navigation | 1.1–1.12 | Hierarchical tree, lazy loading, simple/advanced view, filtering, drag-drop, context menus |
| Table Inspection | 2.1–2.9 | Properties, columns, indexes, constraints, triggers, statistics, row count, inline editing |
| View Inspection | 3.1–3.6 | Definition SQL, columns, materialized view support, editable source |
| Procedure/Function Inspection | 4.1–4.8 | Parameters, source code, compilation errors, overloads, dependencies |
| Trigger Inspection | 5.1–5.6 | Event/timing/body, enable/disable, dual tree location |
| DDL Generation | 6.1–6.9 | CREATE/ALTER/DROP, dialect-aware, multi-object, configurable options, schema-wide |
| Dependency Graph | 7.1–7.8 | Bidirectional deps, FK/view/proc relationships, visual graph, cycle detection |
| Search Across Objects | 8.1–8.10 | Global search, type filtering, column search, multi-connection, incremental |

**Total requirements extracted:** 55

---

## References

- [DBeaver Database Navigator documentation](https://dbeaver.com/docs/cloudbeaver/Database-Navigator/) — hierarchical tree, object browsing
- [DBeaver Database Object Editor](https://dbeaver.com/docs/dbeaver/Database-Object-Editor/) — tabbed metadata inspector
- [DBeaver SQL Generation](https://dbeaver.com/docs/dbeaver/SQL-Generation/) — DDL generation from navigator and data editor
- [DBeaver Filter Database Objects](https://dbeaver.com/docs/dbeaver/Filter-Database-Objects/) — tree filtering
- [DBeaver Metadata Search](https://dbeaver.com/2023/10/16/how-you-can-search-in-dbeaver-metadata-search/) — global object search
- [DBeaver Incorporating Triggers](https://dbeaver.com/docs/dbeaver/Incorporating-Triggers/) — trigger management
- [DBeaver Properties Editor](https://dbeaver.com/docs/dbeaver/Properties-Editor/) — object properties display
- [DBeaver Simple and Advanced View](https://github.com/dbeaver/dbeaver/wiki/Simple-and-Advanced-View) — navigator view modes
- [DBeaver Foreign Keys](https://github.com/dbeaver/dbeaver/wiki/Utilizing-Foreign-Keys) — constraint relationships

Content was rephrased for compliance with licensing restrictions.
