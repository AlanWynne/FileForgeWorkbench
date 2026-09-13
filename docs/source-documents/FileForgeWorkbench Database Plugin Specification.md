# FileForgeWorkbench Database Plugin Specification

**Document**ersion*** 1.0  
**Status:** Proposed  
**Author*** Alan Wynne  
**Project:** FileForgeWorkbench  
**Component:** Database Plugin Framework

---

# 1. Introduction

The Database Plugin is intended to provide FileForgeWorkbench users with integrated relational database capabilities while maintaining the product's primary focus as a cross-platform enterprise file editor, mainframe workstation, and modernization platform.

The objective is not to replicate all functionality provided by dedicated database products such as DBeaver, but rather to provide a streamlined, modern, and extensible database environment that integrates naturally with FileForgeWorkbench concepts such as datasets, GDGs, VSAM files, JES output, and project workspaces.

---

# 2. Design Principles

The Database Plugin SHALL:

- Be fully cross-platform.
- Operate on Windows, Linux, and macOS.
- Support embedded databases without requiring a server installation.
- Support local and remote database connections.
- Follow the FileForgeWorkbench plugin architecture.
- Use native Rust database libraries where possible.
- Present a modern explorer-based user interface.
- Integrate database objects with FileForgeWorkbench project resources.

The Database Plugin SHALL NOT:

- Attempt to replace enterprise-grade database administration tools.
- Depend upon Eclipse-based user interface frameworks.
- Require Java to operate.

---

# 3. Design Inspiration

The plugin design SHALL be inspired by:

- DBeaver (multi-database support)
- VS Code Explorer (navigation model)
- SQLiteStudio (simplicity)
- DataGrip (developer productivity)

The plugin SHALL adopt a lightweight explorer-based user experience rather than a complex Eclipse-style workspace model.

---

# 4. Architectural Overview

```text
FileForgeWorkbench
│
├── Editor Framework
├── Dataset Manager
├── JES/SDSF Emulator
├── ISPF Emulator
├── Plugin Framework
│
└── Database Plugin
     │
     ├── Embedded SQLite Engine
     ├── Connection Manager
     ├── SQL Editor
     ├── Data Browser
     ├── Metadata Browser
     ├── Import/Export Engine
     └── Database Driver Layer
```

---

# 5. Supported Database Types

## 5.1 Embedded Database

The initial implementation SHALL provide:

### SQLite

SQLite SHALL be bundled with FileForgeWorkbench.

Users SHALL be able to create a new database directly from the user interface.

Example:

```text
File
 └── New SQLite Database
```

The resulting database SHALL exist as a regular file within the project workspace.

Examples:

```text
customers.db
inventory.db
metadata.db
```

---

## 5.2 External Database Support

The plugin SHOULD support the following databases.

| Database | Priority |
|-----------|-----------|
| SQLite | Mandatory |
| PostgreSQL | High |
| SQL Server | High |
| MySQL | High |
| MariaDB | High |
| DB2 LUW | Medium |
| DB2 z/OS | High |
| Oracle | Medium |
| SAP HANA | Medium |

---

# 6. Connection Manager

The Connection Manager SHALL provide facilities for storing and managing database connections.

## 6.1 Connection Properties

A database connection SHALL support:

- Name
- Description
- Database Type
- Host
- Port
- Database Name
- Username
- Password
- SSL Options
- Connection Parameters

---

## 6.2 Security

Passwords SHALL be encrypted at rest.

Sensitive connection data SHALL NOT be stored in plain text.

The Connection Manager SHOULD integrate with operating system credential stores when available.

---

# 7. Database Explorer

The plugin SHALL provide a hierarchical explorer.

Example:

```text
Databases
│
├── Local SQLite
│    ├── customers.db
│    └── inventory.db
│
├── PostgreSQL DEV
│    ├── Schemas
│    ├── Tables
│    ├── Views
│    └── Functions
│
└── SQL Server TEST
     ├── Tables
     ├── Procedures
     └── Views
```

The explorer SHALL support:

- Expand
- Collapse
- Refresh
- Search
- Filtering

---

# 8. SQL Editor

The plugin SHALL provide an SQL editor.

## 8.1 Editor Features

The editor SHALL support:

- Syntax highlighting
- Auto-completion
- Query execution
- Multiple tabs
- Query history
- Find and replace
- Result navigation

Example:

```sql
SELECT *
FROM CUSTOMER
WHERE CUSTOMER_ID = 100;
```

---

## 8.2 Execution Support

The editor SHOULD support:

- Execute Statement
- Execute Selection
- Execute Script
- Cancel Query

---

# 9. Data Browser

The plugin SHALL provide a spreadsheet-style data browser.

Example:

```text
ID     NAME         CITY
1      Alan         Johannesburg
2      John         Cape Town
```

The browser SHALL support:

- Paging
- Sorting
- Filtering
- Exporting
- Column resizing
- Read-only mode

The browser SHOULD support inline editing.

---

# 10. Schema Browser

The plugin SHALL display database metadata.

Supported objects SHOULD include:

- Tables
- Views
- Indexes
- Constraints
- Functions
- Procedures
- Triggers
- Sequences

---

# 11. Import and Export

The plugin SHALL support data import and export.

## 11.1 Import Formats

The following formats SHALL be supported:

- CSV
- TSV
- JSON
- XML
- Excel

---

## 11.2 Export Formats

The following formats SHALL be supported:

- CSV
- JSON
- XML
- Excel
- Parquet

---

# 12. ERD Viewer

The plugin SHOULD provide a visual Entity Relationship Diagram viewer.

The viewer SHOULD display:

- Tables
- Relationships
- Foreign Keys
- Cardinality

The viewer SHOULD support:

- Zoom
- Pan
- Export as Image
- Export as PDF

---

# 13. FileForgeWorkbench Differentiators

## 13.1 Dataset-to-Database Integration

The plugin SHALL provide unique integration between traditional files and relational databases.

Supported source types MAY include:

- FB datasets
- VB datasets
- VSAM files
- Sequential files
- CSV files
- GDG generations

---

## 13.2 Database Import

Example workflow:

```text
CUSTOMERS.FB
       │
       ▼
Import
       │
       ▼
SQLite Table
```

---

## 13.3 Database Export

Example workflow:

```text
PostgreSQL Query
        │
        ▼
Export
        │
        ▼
CUSTOMER.EXTRACT.G0001V00
```

---

## 13.4 VSAM Mapping

Future implementations SHOULD support:

- VSAM KSDS → Table
- VSAM ESDS → Table
- VSAM RRDS → Table

Mapping metadata SHOULD be preserved.

---

# 14. Plugin Architecture

Database support SHALL be implemented through database driver plugins.

```text
Database Plugin
│
├── SQLite Driver
├── PostgreSQL Driver
├── SQL Server Driver
├── MySQL Driver
├── DB2 Driver
└── Oracle Driver
```

The architecture SHALL allow new database types to be added without modification to the core application.

---

# 15. Recommended Rust Technology Stack

## Database Access

Preferred technologies:

```toml
sqlx
```

Optional:

```toml
sea-orm
```

---

## SQLite

```toml
rusqlite
```

---

## PostgreSQL

```toml
tokio-postgres
```

---

## Connection Pooling

```toml
bb8
```

or

```toml
deadpool
```

---

# 16. Planned Delivery Phases

## Phase 1

Mandatory Deliverables:

- Embedded SQLite
- Connection Manager
- Database Explorer
- SQL Editor
- Data Browser
- CSV Import
- CSV Export

---

## Phase 2

Enhancements:

- PostgreSQL
- SQL Server
- MySQL
- MariaDB
- ERD Viewer

---

## Phase 3

Enterprise Databases:

- DB2 LUW
- DB2 z/OS
- Oracle
- SAP HANA

---

## Phase 4

Mainframe Modernization Features:

- Dataset ↔ Database Conversion
- VSAM Mapping
- GDG Integration
- Metadata Synchronization
- SQL over Dataset Catalogues
- Cross-Platform Data Migration Utilities

---

# 17. EARS Requirements

### DB-001

WHEN FileForgeWorkbench is installed  
THE SYSTEM SHALL provide embedded SQLite database support.

### DB-002

WHEN a user creates a database  
THE SYSTEM SHALL create a valid SQLite database file.

### DB-003

WHEN a database connection is configured  
THE SYSTEM SHALL store connection credentials securely.

### DB-004

WHEN a database is connected  
THE SYSTEM SHALL display database objects in the Database Explorer.

### DB-005

WHEN a user executes SQL  
THE SYSTEM SHALL display query results within the Data Browser.

### DB-006

WHEN a user imports structured data  
THE SYSTEM SHALL support importing CSV, JSON, XML and Excel data.

### DB-007

WHEN a user exports query results  
THE SYSTEM SHALL support exporting CSV, JSON, XML, Excel and Parquet formats.

### DB-008

WHERE a supported database driver exists  
THE SYSTEM SHALL allow remote database connectivity.

### DB-009

WHEN a dataset import operation is requested  
THE SYSTEM SHALL allow conversion of supported datasets into database tables.

### DB-010

WHEN a database export operation is requested  
THE SYSTEM SHALL allow generation of FileForgeWorkbench-compatible datasets from database query results.

### DB-011

WHERE database drivers are installed  
THE SYSTEM SHALL dynamically discover and register the drivers at startup.

### DB-012

WHEN future database types are introduced  
THE SYSTEM SHALL support new database drivers without modification to the core application.