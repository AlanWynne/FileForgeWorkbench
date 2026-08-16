# DBeaver Core Requirements Research — Task 16.1

> **Scope:** Connection management, driver registry, multi-database support, credential storage, SSH tunnelling, connection pooling
>
> **Source:** DBeaver public documentation (dbeaver.com/docs/dbeaver), GitHub wiki (github.com/dbeaver/dbeaver/wiki), DBeaver Community site (dbeaver.io)
>
> **Format:** Numbered requirements in EARS format, tagged `[DBV-CORE]`
>
> **Purpose:** Raw requirements extraction for later synthesis into FileForgeWorkbench integrated database tool specification (Task 16.8)

---

## 1. Connection Management

### 1.1 Connection Creation Wizard

| # | Requirement | Tag |
|---|-------------|-----|
| 1.1.1 | THE system SHALL provide a connection creation wizard that guides the user through selecting a database driver, specifying host/port/database parameters, and configuring authentication credentials. | [DBV-CORE] |
| 1.1.2 | WHEN the user initiates connection creation, THE system SHALL display a categorised list of all available database drivers (grouped by database type) and allow selection by searching or browsing. | [DBV-CORE] |
| 1.1.3 | THE system SHALL construct the connection URL automatically from user-supplied parameters (host, port, database) using the driver's URL template in the format `jdbc:vendor://{host}:{port}/{database}`. | [DBV-CORE] |
| 1.1.4 | THE system SHALL allow the user to override the auto-constructed URL by entering a manual JDBC/connection URL directly. | [DBV-CORE] |
| 1.1.5 | THE system SHALL provide a "Test Connection" button that validates connectivity, downloads any missing driver libraries, and reports success or failure with diagnostic information. | [DBV-CORE] |
| 1.1.6 | WHEN the user completes the connection wizard, THE system SHALL persist the connection configuration in a project-scoped JSON configuration file (`data-sources.json`). | [DBV-CORE] |

### 1.2 Connection Editing and Deletion

| # | Requirement | Tag |
|---|-------------|-----|
| 1.2.1 | THE system SHALL allow the user to edit any existing connection's parameters (host, port, credentials, driver, network configuration) via a properties dialog accessible from the Database Navigator context menu. | [DBV-CORE] |
| 1.2.2 | THE system SHALL allow the user to delete a connection, removing it from the project's data-sources file after confirmation. | [DBV-CORE] |
| 1.2.3 | THE system SHALL allow the user to duplicate (copy) an existing connection configuration to create a new connection with the same settings. | [DBV-CORE] |
| 1.2.4 | THE system SHALL support renaming connections in the Database Navigator. | [DBV-CORE] |

### 1.3 Connection Types and Classification

| # | Requirement | Tag |
|---|-------------|-----|
| 1.3.1 | THE system SHALL support connection type classification with at least three built-in types: Development (white/no colour), Test (green), and Production (red). | [DBV-CORE] |
| 1.3.2 | WHEN a connection type is assigned, THE system SHALL apply the type's colour to the Database Navigator entry, associated editor tabs, and status indicators, providing visual safety cues. | [DBV-CORE] |
| 1.3.3 | THE system SHALL allow users to create custom connection types with user-defined name, colour, auto-commit default, and confirmation-on-execute behaviour. | [DBV-CORE] |
| 1.3.4 | IF a connection type has confirmation-on-execute enabled, THEN THE system SHALL prompt the user before executing any DML/DDL statement on that connection. | [DBV-CORE] |

### 1.4 Connection Lifecycle

| # | Requirement | Tag |
|---|-------------|-----|
| 1.4.1 | THE system SHALL support explicit connect and disconnect actions per connection, with visual state indicators (connected, disconnected, connecting) in the Database Navigator. | [DBV-CORE] |
| 1.4.2 | THE system SHALL support an "Invalidate/Reconnect" action that closes a stale connection and re-establishes it, useful when network disruptions occur. | [DBV-CORE] |
| 1.4.3 | WHEN connecting to a database, THE system SHALL download any missing driver libraries automatically (from configured Maven repositories or local paths) before establishing the connection. | [DBV-CORE] |
| 1.4.4 | THE system SHALL support multiple simultaneous connections to different databases within the same workspace session. | [DBV-CORE] |

### 1.5 Connection Import/Export

| # | Requirement | Tag |
|---|-------------|-----|
| 1.5.1 | THE system SHALL support importing connection configurations from CSV or XML files with a defined column/attribute schema (host, port, database, user, driver, etc.). | [DBV-CORE] |
| 1.5.2 | THE system SHALL support importing connections from external database tools (e.g., other IDE/tool configurations). | [DBV-CORE] |
| 1.5.3 | THE system SHALL support loading multiple connection definition files matching a naming pattern from the project folder. | [DBV-CORE] |

---

## 2. Driver Registry

### 2.1 Pre-configured Drivers

| # | Requirement | Tag |
|---|-------------|-----|
| 2.1.1 | THE system SHALL ship with pre-configured driver definitions for all major database platforms (PostgreSQL, MySQL/MariaDB, Oracle, SQL Server, SQLite, IBM Db2, and others), including driver class name, URL template, default port, and Maven artifact coordinates. | [DBV-CORE] |
| 2.1.2 | THE system SHALL categorise drivers by database type (SQL, NoSQL, Cloud, Analytical, Embedded) and present them in a browsable, searchable Driver Manager interface. | [DBV-CORE] |
| 2.1.3 | FOR EACH pre-configured driver, THE system SHALL define: driver name, driver type, fully-qualified class name, URL template with placeholders, default port, embedded flag, and authentication requirement flag. | [DBV-CORE] |

### 2.2 Driver Library Management

| # | Requirement | Tag |
|---|-------------|-----|
| 2.2.1 | THE system SHALL support on-demand automatic download of driver libraries from Maven Central when a driver is first used and its JAR files are not present locally. | [DBV-CORE] |
| 2.2.2 | THE system SHALL allow the user to manually add driver library files (JAR) via file browser, folder reference, or Maven artifact coordinates (groupId:artifactId:version). | [DBV-CORE] |
| 2.2.3 | THE system SHALL support automatic checking for new driver versions on application startup (configurable option). | [DBV-CORE] |
| 2.2.4 | THE system SHALL store downloaded driver libraries in a configurable local folder and allow users to specify proxy settings (host, port, user, password) for driver download. | [DBV-CORE] |
| 2.2.5 | THE system SHALL support configuring multiple Maven repository URLs as driver download sources. | [DBV-CORE] |
| 2.2.6 | WHEN multiple versions of a Maven artifact are available, THE system SHALL allow the user to select a specific version or update to a newer version at runtime without reconfiguring driver properties. | [DBV-CORE] |

### 2.3 Custom Driver Creation

| # | Requirement | Tag |
|---|-------------|-----|
| 2.3.1 | THE system SHALL allow the user to create a new custom driver definition by specifying: driver name, driver type, class name, URL template, default port, and library files. | [DBV-CORE] |
| 2.3.2 | THE system SHALL provide a "Find Class" function that scans added JAR files and lists all available driver classes, allowing the user to select the correct one. | [DBV-CORE] |
| 2.3.3 | THE system SHALL provide a "Generic" driver type for connecting to any database with a JDBC-compatible interface that lacks a dedicated pre-configured driver. | [DBV-CORE] |
| 2.3.4 | THE system SHALL allow the user to copy an existing driver definition as a starting point for a new custom driver. | [DBV-CORE] |
| 2.3.5 | THE system SHALL allow deletion of custom driver definitions (permanently) and hiding of built-in drivers (reversible via un-delete/restore). | [DBV-CORE] |

### 2.4 Driver Properties and Configuration

| # | Requirement | Tag |
|---|-------------|-----|
| 2.4.1 | THE system SHALL expose driver-specific JDBC connection properties (loaded from the driver's metadata) and allow the user to configure them per connection. | [DBV-CORE] |
| 2.4.2 | THE system SHALL support advanced driver parameters including: index support, stored procedure support, foreign key support, SELECT count(*) support, view support, script delimiters, escape characters, and metadata model type. | [DBV-CORE] |
| 2.4.3 | THE system SHALL support driver-level query configuration: get/set active database commands, ping query, dual table name, and shutdown commands. | [DBV-CORE] |
| 2.4.4 | THE system SHALL persist all driver configurations in an XML file (`drivers.xml`) that can be customised or distributed across installations. | [DBV-CORE] |
| 2.4.5 | THE system SHALL support ODBC-JDBC bridge drivers for databases that only expose ODBC interfaces. | [DBV-CORE] |

---

## 3. Multi-Database Support

### 3.1 Supported Database Platforms

| # | Requirement | Tag |
|---|-------------|-----|
| 3.1.1 | THE system SHALL provide first-class support (with pre-configured drivers and database-specific metadata handling) for the following relational databases: PostgreSQL, MySQL, MariaDB, Oracle, Microsoft SQL Server, SQLite, IBM Db2, Greenplum, Netezza, Teradata, Yellowbrick, Trino. | [DBV-CORE] |
| 3.1.2 | THE system SHALL provide support for NoSQL databases including: MongoDB, Cassandra, Redis, Couchbase, DynamoDB, DocumentDB, Neo4j. | [DBV-CORE] |
| 3.1.3 | THE system SHALL provide support for cloud-native databases including: Amazon Redshift, Amazon Athena, Amazon Timestream, Google BigQuery, Google Spanner, Google Cloud SQL, Azure Cosmos DB, Databricks, Snowflake, ClickHouse, InfluxDB. | [DBV-CORE] |
| 3.1.4 | THE system SHALL support file-based data sources including: CSV, JSON, Parquet, XLSX, and XML via appropriate drivers. | [DBV-CORE] |
| 3.1.5 | THE system SHALL support any database that provides a JDBC-compatible driver, even if not pre-configured, via the Generic driver mechanism. | [DBV-CORE] |

### 3.2 Database-Specific Behaviour

| # | Requirement | Tag |
|---|-------------|-----|
| 3.2.1 | THE system SHALL adapt its metadata navigation (catalogs, schemas, tables, views, procedures, etc.) to the structure model of each connected database, hiding inapplicable hierarchy levels (e.g., omitting catalog when a database has only one). | [DBV-CORE] |
| 3.2.2 | THE system SHALL support database-specific authentication methods: username/password, Kerberos, SSPI/Windows SSO, PgPass, LDAP, Microsoft Entra ID, OAuth/SSO for cloud databases, two-factor authentication (MySQL). | [DBV-CORE] |
| 3.2.3 | THE system SHALL support database-specific DDL syntax differences (drop column syntax, ALTER TABLE variations, script delimiters) via driver advanced parameters. | [DBV-CORE] |
| 3.2.4 | THE system SHALL support database-specific date/time format patterns configurable per driver. | [DBV-CORE] |
| 3.2.5 | THE system SHALL support authentication profiles that can be shared across multiple connections using the same authentication method and credentials. | [DBV-CORE] |

---

## 4. Credential Storage

### 4.1 Encrypted Local Storage

| # | Requirement | Tag |
|---|-------------|-----|
| 4.1.1 | THE system SHALL store all sensitive connection information (username, password, secret keys, tokens) in an encrypted credentials file separate from the connection definitions. | [DBV-CORE] |
| 4.1.2 | THE system SHALL encrypt credentials using a secure encryption method that prevents plaintext exposure of secrets on disk. | [DBV-CORE] |
| 4.1.3 | WHEN a connection is configured with "Save credentials" enabled, THE system SHALL persist the credentials to the encrypted store; otherwise credentials SHALL be prompted on each connection. | [DBV-CORE] |

### 4.2 Master Password

| # | Requirement | Tag |
|---|-------------|-----|
| 4.2.1 | THE system SHALL support a Master Password that provides an additional encryption layer for all credentials stored in the local workspace. | [DBV-CORE] |
| 4.2.2 | WHEN a Master Password is configured, THE system SHALL prompt the user for the Master Password on workspace startup before allowing access to any stored credentials. | [DBV-CORE] |
| 4.2.3 | THE system SHALL allow the user to set, change, or remove the Master Password via security preferences. | [DBV-CORE] |
| 4.2.4 | IF the user forgets the Master Password, THEN THE system SHALL provide a mechanism to reset it (with the consequence of losing access to previously encrypted credentials). | [DBV-CORE] |

### 4.3 OS Keychain Integration (Integrated Security)

| # | Requirement | Tag |
|---|-------------|-----|
| 4.3.1 | THE system SHALL support an "Integrated Security" mode that uses the operating system's built-in keyring/credential store (Windows Credential Manager, macOS Keychain, Linux Secret Service/libsecret) to store the master encryption key. | [DBV-CORE] |
| 4.3.2 | WHEN Integrated Security is selected, THE system SHALL generate a user-specific master key and save it in the OS keyring, making credential access transparent without requiring an explicit master password entry. | [DBV-CORE] |
| 4.3.3 | THE system SHALL ensure that only the operating system user account that created the secure storage can decrypt the credentials. | [DBV-CORE] |

### 4.4 Secret Providers (External Vault Integration)

| # | Requirement | Tag |
|---|-------------|-----|
| 4.4.1 | THE system SHALL support pluggable "Secret Providers" for retrieving credentials from external secret management systems (e.g., HashiCorp Vault, AWS Secrets Manager, Azure Key Vault). | [DBV-CORE] |
| 4.4.2 | THE system SHALL allow configuration of secret provider endpoints, authentication parameters, and key paths. | [DBV-CORE] |
| 4.4.3 | THE system SHALL support specifying secret requirements per connection — defining which credential fields should be retrieved from which secret provider. | [DBV-CORE] |

### 4.5 Automation Security

| # | Requirement | Tag |
|---|-------------|-----|
| 4.5.1 | THE system SHALL support an "Automation" security mode for headless/console operation where credentials can be supplied via environment variables or password files without interactive prompts. | [DBV-CORE] |
| 4.5.2 | IF Automation security mode is active, THEN THE system SHALL log a warning that local credentials can be decrypted by anyone with machine access, as this mode reduces security for automation convenience. | [DBV-CORE] |

---

## 5. SSH Tunnelling

### 5.1 SSH Tunnel Configuration

| # | Requirement | Tag |
|---|-------------|-----|
| 5.1.1 | THE system SHALL allow configuration of an SSH tunnel per database connection, specifying: SSH host/IP, SSH port (default 22), SSH username, and authentication method. | [DBV-CORE] |
| 5.1.2 | THE system SHALL support three SSH authentication methods: username/password, public key authentication (private key file + optional passphrase), and SSH agent authentication (pageant on Windows, ssh-agent on Linux/macOS). | [DBV-CORE] |
| 5.1.3 | WHEN an SSH tunnel is configured, THE system SHALL route database traffic through the encrypted SSH tunnel, forwarding the database port from the remote server to a local port. | [DBV-CORE] |
| 5.1.4 | THE system SHALL provide a "Test tunnel configuration" button that validates SSH connectivity independently of the database connection. | [DBV-CORE] |
| 5.1.5 | WHEN SSH tunnel is active, THE system SHALL implicitly set the database host to `localhost` and use the forwarded local port for the database connection. | [DBV-CORE] |
| 5.1.6 | THE system SHALL allow the user to optionally save SSH credentials to the secure credential store. | [DBV-CORE] |

### 5.2 Jump Hosts (Gateway Servers)

| # | Requirement | Tag |
|---|-------------|-----|
| 5.2.1 | THE system SHALL support one or more Jump (Gateway) servers in the SSH tunnel chain, enabling multi-hop SSH connections when the database server is not directly reachable from the client. | [DBV-CORE] |
| 5.2.2 | WHEN jump hosts are configured, THE system SHALL establish the SSH connection chain in sequence (local → jump host 1 → jump host 2 → ... → target SSH server → database). | [DBV-CORE] |
| 5.2.3 | FOR EACH jump host, THE system SHALL allow independent configuration of host, port, username, and authentication method. | [DBV-CORE] |

### 5.3 SSH Advanced Settings

| # | Requirement | Tag |
|---|-------------|-----|
| 5.3.1 | THE system SHALL support configurable SSH implementation selection (choice of SSH library). | [DBV-CORE] |
| 5.3.2 | THE system SHALL support a "Bypass host verification" option that disables SSH server fingerprint checking (with security warning). | [DBV-CORE] |
| 5.3.3 | THE system SHALL support SSH keep-alive interval configuration (in milliseconds) to maintain tunnel connectivity during inactivity, with a value of 0 disabling keep-alive packets. | [DBV-CORE] |
| 5.3.4 | THE system SHALL support a tunnel connect timeout setting (in milliseconds) that limits how long the system waits to establish the SSH tunnel before reporting failure. | [DBV-CORE] |
| 5.3.5 | THE system SHALL support explicit local and remote port forwarding configuration for advanced scenarios where the default automatic port assignment is insufficient. | [DBV-CORE] |

### 5.4 SSH Tunnel Sharing

| # | Requirement | Tag |
|---|-------------|-----|
| 5.4.1 | THE system SHALL share SSH tunnels across multiple database connections when the SSH configuration parameters (hostname, port, username, authentication details) are identical. | [DBV-CORE] |
| 5.4.2 | WHEN an SSH tunnel is shared, THE system SHALL keep the tunnel open until all connections using it are closed, even if the initiating connection disconnects. | [DBV-CORE] |
| 5.4.3 | THE system SHALL provide an SSH Tunnel Explorer view showing active tunnels, their destinations, the databases using each tunnel, and port forwarding details. | [DBV-CORE] |

### 5.5 Network Profiles

| # | Requirement | Tag |
|---|-------------|-----|
| 5.5.1 | THE system SHALL support reusable Network Profiles that bundle SSH, SSL, and proxy configurations, allowing the same network settings to be applied to multiple connections without repetition. | [DBV-CORE] |
| 5.5.2 | THE system SHALL allow a connection to reference a named Network Profile instead of specifying network settings inline. | [DBV-CORE] |

---

## 6. Connection Pooling and Session Management

### 6.1 Idle Connection Management

| # | Requirement | Tag |
|---|-------------|-----|
| 6.1.1 | THE system SHALL support a configurable idle timeout (in seconds) per connection that automatically closes connections after a period of inactivity. | [DBV-CORE] |
| 6.1.2 | IF the idle timeout is set to zero, THEN THE system SHALL use the timeout configured in the connection's Connection Type settings as a fallback. | [DBV-CORE] |
| 6.1.3 | THE system SHALL allow disabling idle timeout entirely (connection remains open indefinitely until explicitly disconnected). | [DBV-CORE] |
| 6.1.4 | THE system SHALL support a configurable keep-alive interval (in seconds) that sends periodic signals to maintain the connection during user inactivity, preventing premature disconnection by firewalls or the database server. | [DBV-CORE] |

### 6.2 Connection Validation

| # | Requirement | Tag |
|---|-------------|-----|
| 6.2.1 | THE system SHALL support a configurable "PING query" per driver that validates whether a connection is still alive before use (e.g., `SELECT 1`). | [DBV-CORE] |
| 6.2.2 | WHEN a connection has been idle and the user issues a command, THE system SHALL validate the connection (using the ping query or driver-native mechanism) and automatically reconnect if the connection has gone stale. | [DBV-CORE] |
| 6.2.3 | THE system SHALL support a configurable connection validation timeout to prevent the UI from blocking indefinitely when validation takes too long. | [DBV-CORE] |

### 6.3 Multiple Datasource Connections

| # | Requirement | Tag |
|---|-------------|-----|
| 6.3.1 | THE system SHALL support "Separate Connections" mode where each SQL editor tab or metadata browser uses its own independent database connection to the same server, providing session isolation. | [DBV-CORE] |
| 6.3.2 | THE system SHALL support a single shared connection mode where all editors and metadata operations for a given database connection share one physical connection. | [DBV-CORE] |
| 6.3.3 | THE system SHALL allow the user to choose between shared and separate connection modes per database connection configuration. | [DBV-CORE] |

### 6.4 Transaction and Auto-commit Configuration

| # | Requirement | Tag |
|---|-------------|-----|
| 6.4.1 | THE system SHALL support configurable auto-commit mode per connection (on/off), determining whether each SQL statement is committed immediately. | [DBV-CORE] |
| 6.4.2 | THE system SHALL support configurable transaction isolation level per connection: Read Uncommitted, Read Committed, Repeatable Read, and Serializable. | [DBV-CORE] |
| 6.4.3 | THE system SHALL allow the user to switch between auto-commit and manual commit mode at runtime via toolbar/menu without disconnecting. | [DBV-CORE] |

### 6.5 Session Initialization

| # | Requirement | Tag |
|---|-------------|-----|
| 6.5.1 | THE system SHALL support configurable bootstrap queries that execute automatically when a connection is first established (e.g., `SET timezone = 'UTC'`, `PRAGMA foreign_keys = ON`). | [DBV-CORE] |
| 6.5.2 | THE system SHALL execute bootstrap queries once per session, after connection establishment and before any user-initiated operations or metadata loading. | [DBV-CORE] |
| 6.5.3 | THE system SHALL support an "Ignore errors" option for bootstrap queries that allows the session to continue even if one bootstrap query fails. | [DBV-CORE] |
| 6.5.4 | THE system SHALL support configurable default database and default schema selection that takes effect on connection establishment. | [DBV-CORE] |

### 6.6 Shell Commands on Connection Events

| # | Requirement | Tag |
|---|-------------|-----|
| 6.6.1 | THE system SHALL support configurable shell commands that execute on connection events: before connect, after connect, before disconnect, after disconnect. | [DBV-CORE] |
| 6.6.2 | WHEN a shell command is configured for a connection event, THE system SHALL execute the specified OS command at the appropriate lifecycle point. | [DBV-CORE] |

---

## Sources

Content was rephrased for compliance with licensing restrictions.

- DBeaver Documentation — Connection Management: [Create Connection](https://dbeaver.com/docs/dbeaver/Create-Connection/), [Connection Types](https://dbeaver.com/docs/dbeaver/Connection-Types/), [Admin Manage Connections](https://dbeaver.com/docs/dbeaver/Admin-Manage-Connections/)
- DBeaver Documentation — Driver Manager: [Driver Manager](https://dbeaver.com/docs/dbeaver/Driver-Manager/), [Admin Manage Drivers](https://dbeaver.com/docs/dbeaver/Admin-Manage-Drivers/)
- DBeaver Documentation — SSH Configuration: [SSH Configuration](https://dbeaver.com/docs/dbeaver/SSH-Configuration/)
- DBeaver Documentation — Security: [Master Password](https://dbeaver.com/docs/dbeaver/Managing-Master-Password/), [Integrated Security](https://dbeaver.com/docs/dbeaver/Integrated-Security/), [Secret Providers](https://dbeaver.com/docs/dbeaver/Secret-Providers/), [Automation Security](https://dbeaver.com/docs/dbeaver/Automation-Security/)
- DBeaver Documentation — Initialization Settings: [Configure Connection Initialization Settings](https://dbeaver.com/docs/dbeaver/Configure-Connection-Initialization-Settings/)
- DBeaver Documentation — Network: [Network Profiles](https://dbeaver.com/docs/dbeaver/Network-profiles/), [Separate Connections](https://dbeaver.com/docs/dbeaver/Separate-Connections/)
- DBeaver Supported Databases: [Database List](https://dbeaver.com/databases/)
- GitHub Wiki: [dbeaver/dbeaver](https://github.com/dbeaver/dbeaver/wiki)
