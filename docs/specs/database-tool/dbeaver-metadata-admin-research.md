# DBeaver Metadata and Admin Requirements Research — Task 16.7

> **Scope:** User/role management, session monitoring, lock inspection, storage/tablespace info, database statistics, server configuration viewing
>
> **Source:** DBeaver public documentation (dbeaver.com/docs/dbeaver), GitHub wiki (github.com/dbeaver/dbeaver/wiki), DBeaver Community site (dbeaver.io)
>
> **Format:** Numbered requirements in EARS format, tagged `[DBV-ADMIN]`
>
> **Purpose:** Raw requirements extraction for later synthesis into FileForgeWorkbench integrated database tool specification (Task 16.8)

---

## 1. User and Role Management

### 1.1 User Listing and Inspection

| # | Requirement | Tag |
|---|-------------|-----|
| 1.1.1 | THE system SHALL display all database users/roles in the Database Navigator tree under a "Security" or "Users" node appropriate to the connected database type. | [DBV-ADMIN] |
| 1.1.2 | WHEN the user expands a user/role node in the navigator, THE system SHALL display the object's properties including username, authentication method, account status (locked/unlocked/expired), default tablespace/schema, creation date, and profile (where applicable). | [DBV-ADMIN] |
| 1.1.3 | THE system SHALL support viewing user/role details in the Properties Editor showing metadata tabs: General, Privileges, Role Membership, Object Privileges, and System Privileges (tabs adapted per database platform). | [DBV-ADMIN] |
| 1.1.4 | THE system SHALL support filtering and searching users/roles by name via the Database Navigator filter mechanism. | [DBV-ADMIN] |

### 1.2 User Creation

| # | Requirement | Tag |
|---|-------------|-----|
| 1.2.1 | THE system SHALL provide a "Create User" action (via context menu or toolbar) that opens a form for specifying new user parameters: username, authentication credentials, default schema/tablespace, and account options. | [DBV-ADMIN] |
| 1.2.2 | WHEN the user completes the Create User form, THE system SHALL generate and execute the appropriate DDL statement (CREATE USER/CREATE ROLE) for the connected database dialect. | [DBV-ADMIN] |
| 1.2.3 | THE system SHALL support database-specific user creation options: password expiry, account locking, profile assignment (Oracle), connection limit (PostgreSQL), login/nologin attribute (PostgreSQL), default database (SQL Server). | [DBV-ADMIN] |
| 1.2.4 | THE system SHALL display the generated DDL in a preview panel before execution, allowing the user to review and optionally modify the statement. | [DBV-ADMIN] |

### 1.3 User Modification

| # | Requirement | Tag |
|---|-------------|-----|
| 1.3.1 | THE system SHALL allow modification of user properties (password, default tablespace, profile, account status) via the Properties Editor with a "Save" action that generates and executes ALTER USER statements. | [DBV-ADMIN] |
| 1.3.2 | THE system SHALL support the "Change current user password" action for the currently connected user, prompting for old and new passwords and executing the appropriate ALTER statement. | [DBV-ADMIN] |
| 1.3.3 | THE system SHALL support locking and unlocking user accounts via context menu actions that execute the appropriate ALTER USER ... ACCOUNT LOCK/UNLOCK statement. | [DBV-ADMIN] |

### 1.4 User Deletion

| # | Requirement | Tag |
|---|-------------|-----|
| 1.4.1 | THE system SHALL allow deletion of a user/role via context menu "Delete" action with a confirmation dialog. | [DBV-ADMIN] |
| 1.4.2 | WHEN deleting a user, THE system SHALL offer a CASCADE option (where supported by the database) to drop owned objects along with the user. | [DBV-ADMIN] |
| 1.4.3 | THE system SHALL display the generated DROP USER/DROP ROLE DDL for review before execution. | [DBV-ADMIN] |

### 1.5 Privilege Management (GRANT/REVOKE)

| # | Requirement | Tag |
|---|-------------|-----|
| 1.5.1 | THE system SHALL display system privileges granted to a user/role in a dedicated "System Privileges" tab, showing privilege name and whether it was granted with ADMIN OPTION or GRANT OPTION. | [DBV-ADMIN] |
| 1.5.2 | THE system SHALL display object privileges granted to a user/role in an "Object Privileges" tab, showing grantor, grantee, object, privilege type, and grantable flag. | [DBV-ADMIN] |
| 1.5.3 | THE system SHALL provide a GRANT interface that allows the administrator to grant system privileges or object privileges to a user/role, generating the appropriate GRANT statement. | [DBV-ADMIN] |
| 1.5.4 | THE system SHALL provide a REVOKE interface that allows the administrator to revoke previously granted privileges, generating the appropriate REVOKE statement. | [DBV-ADMIN] |
| 1.5.5 | THE system SHALL support granting privileges WITH GRANT OPTION / WITH ADMIN OPTION where the database supports delegation. | [DBV-ADMIN] |

### 1.6 Role Membership

| # | Requirement | Tag |
|---|-------------|-----|
| 1.6.1 | THE system SHALL display role memberships for a user/role, showing which roles are granted and whether the grant includes ADMIN OPTION. | [DBV-ADMIN] |
| 1.6.2 | THE system SHALL allow granting a role to a user (or role to another role) via a dialog that lists available roles and generates GRANT ROLE statements. | [DBV-ADMIN] |
| 1.6.3 | THE system SHALL allow revoking a role from a user/role, generating the appropriate REVOKE statement. | [DBV-ADMIN] |
| 1.6.4 | THE system SHALL support viewing effective (resolved) privileges for a user by aggregating all granted roles recursively. | [DBV-ADMIN] |

---

## 2. Session Monitoring

### 2.1 Session Manager Display

| # | Requirement | Tag |
|---|-------------|-----|
| 2.1.1 | THE system SHALL provide a Session Manager view accessible from the Database Navigator's Administer section that displays all active database sessions in a tabular list. | [DBV-ADMIN] |
| 2.1.2 | FOR EACH session, THE system SHALL display: session/process ID, username, client hostname/application name, current database/schema, session status (active/idle/waiting), connection time, and the SQL statement currently being executed (if any). | [DBV-ADMIN] |
| 2.1.3 | THE system SHALL support toggling between "Active sessions only" and "All sessions" views via a toolbar button. | [DBV-ADMIN] |
| 2.1.4 | THE system SHALL support a "Show Inactive" option to include sessions that are idle or sleeping in the session list. | [DBV-ADMIN] |
| 2.1.5 | THE system SHALL support a "Show Background" option to display background system processes/workers in the session list. | [DBV-ADMIN] |

### 2.2 Session Details

| # | Requirement | Tag |
|---|-------------|-----|
| 2.2.1 | WHEN the user selects a session in the Session Manager, THE system SHALL display the full SQL text of the query currently being executed by that session in a detail panel. | [DBV-ADMIN] |
| 2.2.2 | THE system SHALL display session-level statistics where available: CPU time, memory usage, I/O reads/writes, wait events, and elapsed time for the current operation. | [DBV-ADMIN] |
| 2.2.3 | THE system SHALL support searching/filtering the session list by username, application name, session ID, or SQL text content. | [DBV-ADMIN] |

### 2.3 Session Actions

| # | Requirement | Tag |
|---|-------------|-----|
| 2.3.1 | THE system SHALL provide a "Kill Session" action that forcefully terminates the selected session, with a confirmation prompt before execution. | [DBV-ADMIN] |
| 2.3.2 | THE system SHALL provide a "Disconnect Session" action (where supported) that gracefully disconnects the selected session without forceful termination. | [DBV-ADMIN] |
| 2.3.3 | WHEN the user invokes Kill or Disconnect, THE system SHALL execute the database-appropriate command (e.g., `ALTER SYSTEM KILL SESSION` for Oracle, `pg_terminate_backend()` for PostgreSQL, `KILL` for MySQL/SQL Server). | [DBV-ADMIN] |
| 2.3.4 | THE system SHALL support selecting and terminating multiple sessions simultaneously via multi-selection. | [DBV-ADMIN] |

### 2.4 Session Auto-Refresh

| # | Requirement | Tag |
|---|-------------|-----|
| 2.4.1 | THE system SHALL support configurable auto-refresh of the session list at a user-defined interval (e.g., every 5, 10, 30 seconds). | [DBV-ADMIN] |
| 2.4.2 | THE system SHALL provide a manual "Refresh" button to immediately update the session list on demand. | [DBV-ADMIN] |
| 2.4.3 | THE system SHALL provide a "SQL Script" action that runs the underlying session query and displays results as a standard result set for custom analysis. | [DBV-ADMIN] |

### 2.5 Supported Databases for Session Monitoring

| # | Requirement | Tag |
|---|-------------|-----|
| 2.5.1 | THE system SHALL support session monitoring for the following database platforms (at minimum): PostgreSQL, MySQL, MariaDB, Oracle, Microsoft SQL Server, IBM Db2, Greenplum, Exasol, AlloyDB, and MongoDB. | [DBV-ADMIN] |
| 2.5.2 | THE system SHALL adapt the session manager columns and available actions to the capabilities of each connected database (e.g., Oracle shows SID/Serial#, PostgreSQL shows PID, MySQL shows thread ID). | [DBV-ADMIN] |

---

## 3. Lock Inspection

### 3.1 Lock Manager Display

| # | Requirement | Tag |
|---|-------------|-----|
| 3.1.1 | THE system SHALL provide a Lock Manager view accessible from the Database Navigator's Administer section that displays all active database locks in a tabular list. | [DBV-ADMIN] |
| 3.1.2 | FOR EACH lock, THE system SHALL display: lock type (row, table, page, advisory), lock mode (shared, exclusive, update), locked object name, holding session ID, waiting session ID (if blocked), and lock duration. | [DBV-ADMIN] |
| 3.1.3 | THE system SHALL clearly distinguish between holding sessions (blocker) and waiting sessions (blocked) using visual indicators or column separation. | [DBV-ADMIN] |

### 3.2 Blocking Query Analysis

| # | Requirement | Tag |
|---|-------------|-----|
| 3.2.1 | THE system SHALL display the "Hold Statement" — the SQL text of the session currently holding the lock (the blocker). | [DBV-ADMIN] |
| 3.2.2 | THE system SHALL display the "Wait Statement" — the SQL text of the session waiting to acquire the lock (the blocked session). | [DBV-ADMIN] |
| 3.2.3 | THE system SHALL identify and highlight blocking chains where session A blocks session B, which in turn blocks session C, presenting the chain hierarchy. | [DBV-ADMIN] |

### 3.3 Lock Wait Graph

| # | Requirement | Tag |
|---|-------------|-----|
| 3.3.1 | THE system SHALL provide a visual or hierarchical representation of lock dependencies showing which sessions are blocking other sessions (lock wait graph). | [DBV-ADMIN] |
| 3.3.2 | THE system SHALL detect and flag potential deadlock situations where circular lock dependencies exist. | [DBV-ADMIN] |
| 3.3.3 | THE system SHALL display lock wait time for each blocked session, indicating how long the session has been waiting to acquire the lock. | [DBV-ADMIN] |

### 3.4 Lock Management Actions

| # | Requirement | Tag |
|---|-------------|-----|
| 3.4.1 | THE system SHALL provide a "Kill waiting session" action that terminates the session waiting for a lock, resolving the immediate contention by stopping the blocked process. | [DBV-ADMIN] |
| 3.4.2 | THE system SHALL support auto-refresh of lock data at configurable intervals to keep the display current without manual intervention. | [DBV-ADMIN] |
| 3.4.3 | THE system SHALL provide a manual "Refresh locks" button to immediately update lock information on demand. | [DBV-ADMIN] |

### 3.5 Supported Databases for Lock Inspection

| # | Requirement | Tag |
|---|-------------|-----|
| 3.5.1 | THE system SHALL support lock inspection for the following database platforms (at minimum): PostgreSQL, Oracle, IBM Db2, Greenplum, Exasol, AlloyDB, and Altibase. | [DBV-ADMIN] |
| 3.5.2 | THE system SHALL adapt lock manager columns and terminology to the connected database's lock model (e.g., Oracle shows enqueue locks, PostgreSQL shows advisory/relation locks, Db2 shows lock escalation details). | [DBV-ADMIN] |

---

## 4. Storage and Tablespace Information

### 4.1 Tablespace Listing

| # | Requirement | Tag |
|---|-------------|-----|
| 4.1.1 | THE system SHALL display tablespaces/storage containers in the Database Navigator tree under a "Storage" or "Tablespaces" node (for databases that support the concept: Oracle, PostgreSQL, IBM Db2, SQL Server filegroups). | [DBV-ADMIN] |
| 4.1.2 | FOR EACH tablespace, THE system SHALL display: tablespace name, status (online/offline/read-only), type (permanent/temporary/undo), total allocated size, used size, free size, and percentage utilization. | [DBV-ADMIN] |
| 4.1.3 | THE system SHALL support viewing tablespace properties in the Properties Editor with tabs for General, Datafiles, and Objects (segments residing in the tablespace). | [DBV-ADMIN] |

### 4.2 Datafile Information

| # | Requirement | Tag |
|---|-------------|-----|
| 4.2.1 | FOR EACH tablespace, THE system SHALL list associated datafiles/data files showing: file name/path, file size, auto-extend status, maximum size, next increment size, and current usage. | [DBV-ADMIN] |
| 4.2.2 | THE system SHALL display the physical storage path of each datafile and its online/offline status. | [DBV-ADMIN] |
| 4.2.3 | THE system SHALL support viewing fragmentation information for datafiles where the database provides such metadata. | [DBV-ADMIN] |

### 4.3 Storage Usage Visualization

| # | Requirement | Tag |
|---|-------------|-----|
| 4.3.1 | THE system SHALL provide a summary view showing total database storage: sum of all tablespace sizes, total used, total free, and overall percentage utilization. | [DBV-ADMIN] |
| 4.3.2 | THE system SHALL support dashboard charts for storage monitoring showing disk space usage trends over time (via the Dashboards facility). | [DBV-ADMIN] |
| 4.3.3 | THE system SHALL provide visual indicators (colour coding or progress bars) for tablespaces approaching capacity thresholds (e.g., >80% used, >90% used). | [DBV-ADMIN] |

### 4.4 Storage Administration Actions

| # | Requirement | Tag |
|---|-------------|-----|
| 4.4.1 | THE system SHALL support creating new tablespaces via a creation dialog that generates and executes the appropriate CREATE TABLESPACE DDL. | [DBV-ADMIN] |
| 4.4.2 | THE system SHALL support modifying tablespace properties (adding datafiles, resizing, changing auto-extend settings) via ALTER TABLESPACE DDL generation. | [DBV-ADMIN] |
| 4.4.3 | THE system SHALL support taking tablespaces offline/online and changing read-write mode via context menu actions. | [DBV-ADMIN] |
| 4.4.4 | THE system SHALL support dropping empty tablespaces with appropriate DDL generation and confirmation. | [DBV-ADMIN] |

---

## 5. Database Statistics and Performance Monitoring

### 5.1 Dashboard Infrastructure

| # | Requirement | Tag |
|---|-------------|-----|
| 5.1.1 | THE system SHALL provide a Dashboards panel that displays real-time performance charts for the connected database, accessible via toolbar button or keyboard shortcut. | [DBV-ADMIN] |
| 5.1.2 | THE system SHALL ship with predefined dashboard chart sets for major databases: MySQL (InnoDB data, InnoDB memory, Key Efficiency, Queries, Server sessions, Traffic), Oracle (CPU usage, Global Query Stats, IO Stats, Memory usage, Memory usage by components), PostgreSQL (Block IO, Server sessions, Transactions per second), Exasol (Connections, User activity), BigQuery (Bytes Processed). | [DBV-ADMIN] |
| 5.1.3 | THE system SHALL update dashboard charts continuously at a configurable refresh interval (default: 1000ms) to provide real-time monitoring. | [DBV-ADMIN] |

### 5.2 Dashboard Chart Configuration

| # | Requirement | Tag |
|---|-------------|-----|
| 5.2.1 | FOR EACH dashboard chart, THE system SHALL support configuration of: chart name, description, update period (ms), maximum items to display, SQL query source, data type (timeseries/statistics/provided), calculation type (value/delta), and value type (decimal/integer/percent/bytes). | [DBV-ADMIN] |
| 5.2.2 | THE system SHALL support three chart visualisation types: Bar, Pie, and Time Series, selectable per chart via context menu or settings. | [DBV-ADMIN] |
| 5.2.3 | THE system SHALL support showing/hiding legend, grid, domain axis, and range axis per chart. | [DBV-ADMIN] |
| 5.2.4 | THE system SHALL support zooming (in, out, reset) on Time Series and Bar chart representations. | [DBV-ADMIN] |

### 5.3 Custom Dashboard Charts

| # | Requirement | Tag |
|---|-------------|-----|
| 5.3.1 | THE system SHALL allow creation of custom dashboard charts by specifying a SQL query, chart type, update interval, and display parameters. | [DBV-ADMIN] |
| 5.3.2 | THE system SHALL allow copying a predefined chart as a template to create a new custom chart with modified query or display settings. | [DBV-ADMIN] |
| 5.3.3 | THE system SHALL allow editing and deletion of custom dashboard charts (predefined charts are read-only). | [DBV-ADMIN] |
| 5.3.4 | THE system SHALL support exporting chart screenshots to clipboard (Copy to Clipboard), to file (Save as PNG), and to printer (Print). | [DBV-ADMIN] |

### 5.4 Table and Index Statistics

| # | Requirement | Tag |
|---|-------------|-----|
| 5.4.1 | THE system SHALL display table-level statistics in the table's Properties Editor: row count (estimated and actual), total data size, index size, average row length, last analyzed date, and modification count since last analysis. | [DBV-ADMIN] |
| 5.4.2 | THE system SHALL display index-level statistics: index size, number of distinct keys, clustering factor, B-tree depth/levels, leaf blocks, and last analyzed date (adapted per database). | [DBV-ADMIN] |
| 5.4.3 | THE system SHALL support a "Gather Statistics" / "Analyze" action that generates and executes the appropriate ANALYZE TABLE or DBMS_STATS command to refresh table/index statistics. | [DBV-ADMIN] |
| 5.4.4 | THE system SHALL display column-level statistics where available: number of distinct values, null count, low/high values, histogram information. | [DBV-ADMIN] |

### 5.5 Query Execution Statistics

| # | Requirement | Tag |
|---|-------------|-----|
| 5.5.1 | THE system SHALL provide a Query Manager view that logs all SQL queries executed in the current session, showing: SQL text, execution time (start/end), duration, number of affected rows, connection name, and error status. | [DBV-ADMIN] |
| 5.5.2 | THE system SHALL support filtering the Query Manager log by: date range, query type (SELECT/DML/DDL), connection, execution status (success/error), and text content search. | [DBV-ADMIN] |
| 5.5.3 | THE system SHALL persist query log history and allow configuration of maximum log entries and log file output in preferences. | [DBV-ADMIN] |
| 5.5.4 | THE system SHALL provide a Transaction Log view that records all data-changing queries (INSERT/UPDATE/DELETE) with their commit/rollback status during the current session. | [DBV-ADMIN] |

### 5.6 Connection and Session Statistics

| # | Requirement | Tag |
|---|-------------|-----|
| 5.6.1 | THE system SHALL support dashboard charts monitoring the number of active connections over time for the connected database server. | [DBV-ADMIN] |
| 5.6.2 | THE system SHALL support dashboard charts monitoring transactions per second (TPS) for the connected database. | [DBV-ADMIN] |
| 5.6.3 | THE system SHALL support dashboard charts monitoring cache hit ratios (buffer cache, query cache, key cache) where the database exposes such metrics. | [DBV-ADMIN] |
| 5.6.4 | THE system SHALL support dashboard charts monitoring I/O throughput (reads/writes per second, block I/O) for performance analysis. | [DBV-ADMIN] |

---

## 6. Server Configuration Viewing

### 6.1 Server Variables / Parameters

| # | Requirement | Tag |
|---|-------------|-----|
| 6.1.1 | THE system SHALL provide a view of server configuration variables/parameters accessible from the Database Navigator or Administer section, displaying all runtime parameters with their current values. | [DBV-ADMIN] |
| 6.1.2 | FOR EACH server variable, THE system SHALL display: parameter name, current value, default value (where available), description/documentation, whether it is dynamic (changeable at runtime) or static (requires restart), and scope (global/session). | [DBV-ADMIN] |
| 6.1.3 | THE system SHALL support filtering/searching server variables by name or value to quickly locate specific configuration parameters. | [DBV-ADMIN] |
| 6.1.4 | THE system SHALL support categorisation or grouping of server variables by functional area (e.g., memory, networking, logging, replication, security) where the database provides such classification. | [DBV-ADMIN] |

### 6.2 Variable Modification

| # | Requirement | Tag |
|---|-------------|-----|
| 6.2.1 | IF a server variable is dynamic (modifiable at runtime), THEN THE system SHALL allow the user to change its value via an edit action that generates and executes the appropriate SET command (e.g., `SET GLOBAL variable = value` for MySQL, `ALTER SYSTEM SET` for PostgreSQL/Oracle). | [DBV-ADMIN] |
| 6.2.2 | THE system SHALL indicate which variables are read-only (static) and which are modifiable (dynamic) through visual differentiation (e.g., greyed-out vs. editable). | [DBV-ADMIN] |
| 6.2.3 | WHEN a variable requires a server restart to take effect, THE system SHALL display a warning or annotation indicating that the change will not apply until the server is restarted. | [DBV-ADMIN] |

### 6.3 Server Information

| # | Requirement | Tag |
|---|-------------|-----|
| 6.3.1 | THE system SHALL display general server information: database version, server uptime, host operating system, character set configuration, maximum connections, and server process/thread count. | [DBV-ADMIN] |
| 6.3.2 | THE system SHALL display server memory configuration: shared buffers, sort buffers, work memory, InnoDB buffer pool size (MySQL), shared_buffers/work_mem (PostgreSQL), SGA/PGA (Oracle) — adapted per database platform. | [DBV-ADMIN] |
| 6.3.3 | THE system SHALL display replication status information where applicable: master/slave status, replication lag, binary log position, connected replicas. | [DBV-ADMIN] |

### 6.4 Database-Specific Admin Nodes

| # | Requirement | Tag |
|---|-------------|-----|
| 6.4.1 | THE system SHALL adapt the Administer node's content to the connected database platform, showing only administration tools relevant to that database (e.g., Oracle: tablespaces, redo logs, archived logs; PostgreSQL: extensions, event triggers; MySQL: engines, status variables). | [DBV-ADMIN] |
| 6.4.2 | THE system SHALL support viewing database-level properties: database name, owner, character encoding, collation, compatibility level, and creation options. | [DBV-ADMIN] |
| 6.4.3 | THE system SHALL support viewing scheduled jobs/maintenance tasks where the database provides a job scheduler (Oracle DBMS_SCHEDULER, PostgreSQL pg_cron, SQL Server Agent jobs). | [DBV-ADMIN] |

---

## Sources

Content was rephrased for compliance with licensing restrictions.

- DBeaver Documentation — Session Manager: [Session Manager Guide](https://dbeaver.com/docs/dbeaver/Session-Manager-Guide/)
- DBeaver Documentation — Lock Manager: [Lock Manager](https://dbeaver.com/docs/dbeaver/Lock-Manager/)
- DBeaver Documentation — Dashboards: [Dashboards](https://dbeaver.com/docs/dbeaver/Dashboards/)
- DBeaver Documentation — Query Manager: [Query Manager](https://dbeaver.com/docs/dbeaver/Query-Manager/)
- DBeaver Documentation — Transaction Log: [Transaction Log](https://dbeaver.com/docs/dbeaver/Transaction-Log/)
- DBeaver Documentation — Change Password: [Change current user password](https://dbeaver.com/docs/dbeaver/Change-current-user-password/)
- DBeaver Documentation — Database Navigator: [Database Navigator](https://dbeaver.com/docs/dbeaver/Database-Navigator/)
- DBeaver Documentation — Properties Editor: [Properties Editor](https://dbeaver.com/docs/dbeaver/Properties-Editor/)
- DBeaver GitHub Wiki: [Session Manager Guide](https://github.com/dbeaver/dbeaver/wiki/Session-Manager-Guide), [Lock Manager](https://github.com/dbeaver/dbeaver/wiki/Lock-Manager), [Dashboards](https://github.com/dbeaver/dbeaver/wiki/Dashboards), [Query Manager](https://github.com/dbeaver/dbeaver/wiki/Query-Manager)
