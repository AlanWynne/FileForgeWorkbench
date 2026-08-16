# File Forge Workbench — Dataset Catalog Repository & Mainframe Filesystem Emulation

**Development Phase: Dataset Catalog Repository & Mainframe Filesystem Emulation**

| Field | Value |
|-------|-------|
| Version | 1.0 |
| Status | Proposed |
| Priority | High |
| Target Release | File Forge Workbench Phase 4 |

---

## 1. Executive Summary

Introduce a Mainframe Dataset Repository capability into File Forge Workbench.

The capability SHALL provide:

- Mainframe-style dataset naming
- Catalog management
- PDS member navigation
- GDG support
- JCL dataset resolution
- Unified integration with local filesystem views

The feature SHALL coexist with standard desktop file access.

The objective is not to emulate z/OS itself but to provide a development and learning environment familiar to mainframe developers while remaining fully portable across:

- Windows
- Linux
- macOS

---

## 2. Business Drivers

### Problem

Modern desktop operating systems use:
```
C:\Projects\Payroll\data.txt
```

Mainframes use:
```
PAYROLL.INPUT.FILE
```

JCL, COBOL, REXX and utility programs reference:
```
DSN=PAYROLL.INPUT.FILE
```
rather than operating-system paths.

There is currently no abstraction layer inside File Forge Workbench to bridge these concepts.

### Benefits

- **Learning** — Developers can learn Mainframe concepts without requiring a z/OS system.
- **Modernisation** — Legacy applications can be analysed and tested locally.
- **JCL Processing** — JCL parsers can resolve dataset references.
- **Future Integration** — Provides foundation for future TN3270, FTP, SFTP, z/OS Connect, mainframe source migration.

---

## 3. Functional Vision

The File Tree will become a virtual filesystem.

```
FILE FORGE WORKBENCH
├── Local Files
│   ├── C:
│   ├── D:
│   └── Home
│
├── Catalogs
│   ├── TRISDXX
│   │   ├── PRINT.FILE
│   │   ├── JCL
│   │   │   ├── TESTJOB
│   │   │   └── DAILYRUN
│   │   └── COPYLIB
│   │
│   ├── PAYROLL
│   │   ├── INPUT.FILE
│   │   └── MASTER
│   │
│   └── SYS1
│
└── Connections
    ├── FTP
    └── SFTP
```

Users work within a single explorer.

---

## 4. High-Level Architecture

```
+---------------------------------+
| File Forge Workbench            |
+---------------------------------+
         |
         v
+---------------------------------+
| Virtual File System Layer       |
+---------------------------------+
   |       |       |
   v       v       v
Local FS   Dataset   Remote
Provider   Provider  Provider
```

The editor never opens a Windows file or dataset directly. Instead:

```
Editor
  |
  v
Virtual File System
  |
  +--> Local Provider
  |
  +--> Dataset Provider
  |
  +--> Remote Provider
```

---

## 5. New Component: Dataset Provider

### Purpose

Provide a Mainframe-style dataset abstraction.

### Supported Object Types

**Sequential Dataset (PS)**
```
TRISDXX.PRINT.FILE
```
Represents one file.

**PDS**
```
TRISDXX.JCL
```
Contains members:
```
TRISDXX.JCL(TESTJOB)
TRISDXX.JCL(BACKUP)
TRISDXX.JCL(MONTHEND)
```

**PDSE**

Managed similarly to PDS.

**GDG**
```
TRISDXX.REPORT.DAILY
```
Physical versions:
```
G0001V00
G0002V00
G0003V00
```

**VSAM (Future)**

Represented through:
- SQLite
- LMDB
- RocksDB

backend implementations.

---

## 6. Dataset Catalog Design

### Catalog Database

SQLite proposed.

```sql
CREATE TABLE catalog (
    id       INTEGER PRIMARY KEY,
    dsn      TEXT UNIQUE,
    type     TEXT,
    path     TEXT,
    recfm    TEXT,
    lrecl    INTEGER,
    blksize  INTEGER,
    created  TIMESTAMP,
    modified TIMESTAMP
);
```

### Example Entry

| Field | Value |
|-------|-------|
| DSN | TRISDXX.PRINT.FILE |
| PATH | repositories/default/storage/a83b4.dat |
| TYPE | PS |

---

## 7. Repository Layout

Physical structures hidden from users.

```
RepositoryRoot/
├── catalog.db
├── storage/
├── pds/
├── gdg/
└── temp/
```

Example: `TRISDXX.PRINT.FILE` may physically reside as:
```
storage/8b/a83b4.dat
```

Catalog performs lookup.

---

## 8. Catalog Management

Users can create Catalog nodes.

**Add Catalog Wizard asks for:**
- Catalog Name
- Repository Location

**Result:**
```
Catalogs
└── TRISDXX
```
appears in tree view.

**Users may:**
- Add Catalog
- Remove Catalog
- Mount Catalog
- Unmount Catalog
- Export Catalog
- Import Catalog

---

## 9. JCL Integration

Given:
```jcl
//INFILE DD DSN=TRISDXX.PRINT.FILE
```

Parser flow:
```
DSN Found
  |
  Catalog Lookup
  |
  Physical File
  |
  Open
```

Dataset resolution service:
```rust
resolve_dsn("TRISDXX.PRINT.FILE")
// returns: RepositoryRoot/storage/8b/a83b4.dat
```

---

## 10. File Tree Requirements

| ID | Requirement |
|----|-------------|
| FFW-DS-001 | System SHALL provide a Catalogs root node |
| FFW-DS-002 | System SHALL allow multiple catalogs to be mounted |
| FFW-DS-003 | Mounted catalogs SHALL appear in the explorer tree |
| FFW-DS-004 | Datasets SHALL be navigable through tree nodes |
| FFW-DS-005 | PDS members SHALL appear as children of PDS datasets |
| FFW-DS-006 | Double-clicking a dataset SHALL open the editor |
| FFW-DS-007 | Editor SHALL support Browse, Edit, Save As, and Compare operations |

### PDS Example:
```
TRISDXX.JCL
├── TESTJOB
├── BACKUP
└── DAILYRUN
```

---

## 11. Search Integration

Global search must search:
- Local Files
- Dataset Repositories
- PDS Members

...from a single search box.

Example: `Find: CUSTOMER-NUMBER` searches everywhere.

---

## 12. File Forge APIs

### DatasetService

```rust
trait DatasetService {
    fn create_dataset(&mut self, ...);
    fn delete_dataset(&mut self, ...);
    fn rename_dataset(&mut self, ...);
    fn resolve_dataset(&self, ...) -> Result<PathBuf>;
    fn allocate_dataset(&mut self, ...);
}
```

### CatalogService

```rust
trait CatalogService {
    fn mount_catalog(&mut self, ...);
    fn unmount_catalog(&mut self, ...);
    fn add_catalog(&mut self, ...);
    fn remove_catalog(&mut self, ...);
}
```

### JclResolver

```rust
trait JclResolver {
    fn resolve_dd(&self, ...) -> Result<ResolvedResource>;
}
```

---

## 13. UI Requirements

### Context Menu:
- New Dataset
- New PDS
- New Member
- Delete
- Rename
- Properties

### Properties Panel:
- DSN
- Type
- RECFM
- LRECL
- Created
- Modified
- Physical Location

---

## 14. Future Enhancements

**Phase 5:** GDG Support, Catalog Export

**Phase 6:** VSAM Emulation, IDCAMS Commands

**Phase 7:** TSO Dataset Browser, ISPF Dataset Lists, Dataset Allocation Screens

**Phase 8:** Remote z/OS Connection, TN3270 Integration

---

## 15. Key Architectural Principle

**FFW-ARCH-001**: All content in File Forge Workbench SHALL be accessed through a Virtual File System (VFS) abstraction layer.

This ensures that:
```
C:\temp\file.txt
```
and
```
TRISDXX.PRINT.FILE
```
are treated as equivalent resources by the editor, compare tool, search engine, syntax highlighter, COBOL tooling, JCL tooling, and future plugins.

This single requirement keeps the architecture clean and makes future mainframe integration dramatically easier.
