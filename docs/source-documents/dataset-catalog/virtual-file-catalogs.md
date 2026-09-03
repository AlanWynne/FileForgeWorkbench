# Virtual File Catalogs — FileForge Workbench

This document explains how virtual file catalogs work in FileForge Workbench. It covers the
three catalog types, how they are stored and addressed, how to create and manage them, and
the underlying architecture that ties them together. It also serves as the reference material
for the context-sensitive help on **POM Option 1 — Files**.

---

## What is a Virtual File Catalog?

A **Virtual File Catalog** is a named, typed container registered with the workbench that
groups related files or datasets under a single name. Once registered, a catalog appears in
the Files panel tree and its contents are accessible to every workbench feature — the editor,
search, compare, JCL resolver, and so on — through a unified addressing scheme.

The key principle is **FFW-ARCH-001**: all content in FileForge Workbench is accessed through
the Virtual File System (VFS) abstraction layer. No feature ever opens a file directly from
the operating system. Instead, every resource is identified by a URI of the form:

```
vfs://<provider>/<path>
```

Catalogs are VFS providers. Registering a catalog registers a new provider with the VFS, making
its contents addressable by URI throughout the workbench.

---

## The Three Catalog Types

### Mainframe Catalogs

A Mainframe catalog emulates a z/OS dataset environment on your local machine. It stores
datasets using mainframe naming conventions (`HLQ.QUALIFIER.NAME`) and supports the three
mainframe dataset organisations:

| Organisation | Abbreviation | Description |
|---|---|---|
| Sequential | PS | A single flat file — equivalent to a regular file |
| Partitioned | PO (PDS/PDSE) | A library of named members — equivalent to a directory of files |
| Generation Data Group | GDG | A versioned collection of datasets with a rolling generation limit |

Mainframe catalog contents are addressed as:

```
vfs://catalog/PAYROLL.INPUT.FILE
vfs://catalog/SYS1.MACLIB(OPEN)
vfs://catalog/PAYROLL.MONTHLY.G0003V00
```

The catalog is backed by a **SQLite database** (`catalog.db`) and a structured **repository
directory** on your local filesystem. The database maps dataset names to physical file paths;
the repository holds the actual content. This is entirely transparent — you work with dataset
names, not file paths.

**Dataset naming rules:**
- 1–8 character qualifiers separated by dots, maximum 44 characters total
- Each qualifier starts with a letter or national character (`@`, `#`, `$`), followed by
  letters, digits, or national characters
- Names are case-insensitive; stored internally in uppercase
- PDS members are referenced as `DSN(MEMBER)` — e.g., `SYS1.MACLIB(OPEN)`

**GDG generations** are accessed using relative references:
- `(0)` — the current (most recently created) generation
- `(-1)`, `(-2)` — previous generations
- `(+1)` — allocate a new generation (write context only)
- `G0001V00`, `G0002V00` — absolute generation references

### POSIX Catalogs

A POSIX catalog maps a directory on your local filesystem to a POSIX-style hierarchical
namespace. It enforces POSIX path conventions regardless of the host operating system:

- Forward-slash path separator (`/`)
- Case-sensitive names
- No drive letters
- Paths start with `/` relative to the catalog root

POSIX catalog contents are addressed as:

```
vfs://posix/<catalog-name>/<path>
```

For example, if you register a POSIX catalog named `dev-posix` rooted at `C:/projects/dev`,
then `C:/projects/dev/src/main.rs` is addressed as:

```
vfs://posix/dev-posix/src/main.rs
```

POSIX catalogs can be configured as read-only, in which case all write, create, delete, and
rename operations are rejected at the provider level.

### Native Catalogs

A Native catalog exposes a directory on the host filesystem using the host platform's own
path conventions. The underlying provider (`connector-local-fs`) handles Windows, Linux, and
macOS path conventions transparently — you do not need a separate catalog type per platform.

Native catalog contents are addressed as:

```
vfs://local/<path>
```

The Files panel labels the section header with the current platform at runtime:
`Native Catalogs (Windows)`, `Native Catalogs (Linux)`, or `Native Catalogs (macOS)`.

---

## The Files Panel (POM Option 1)

Selecting **option 1** from the Primary Option Menu, or typing `1` or `FILES` in any
`Command ===>` field, opens the **Files panel** as a full-tab view.

The panel has two areas:

- **Left — Catalog tree**: All registered catalogs grouped under three collapsible section
  headers: `Mainframe Catalogs`, `POSIX Catalogs`, and `Native Catalogs`.
- **Right — Content area**: The immediate children of the selected tree node, shown in a
  list with columns for Name, Type, Size, and Modified Date.

The toolbar at the top provides: `New Catalog`, `Open`, `Refresh`, `Properties`, and a
search/filter input.

Press `PF3` / `F3` or type `END` to return to the Primary Option Menu.

---

## Creating a Catalog

Click **New Catalog** in the toolbar, or right-click a section header and choose
**New Catalog**. The Catalog Manager dialog opens.

### Common fields (all types)

| Field | Required | Notes |
|---|---|---|
| Catalog Name | Yes | 1–32 characters; letters, digits, hyphens, underscores |
| Description | No | Up to 120 characters |
| Auto-mount on startup | — | Checked by default; restores the catalog on next launch |

### Mainframe-specific fields

| Field | Required | Notes |
|---|---|---|
| Repository Path | Yes | Directory where `catalog.db` and storage subdirs will be created |
| Default HLQ | No | Prepended to bare qualifiers when no HLQ is supplied |
| Create repository now | — | Checked by default; initialises the repository immediately |

### POSIX-specific fields

| Field | Required | Notes |
|---|---|---|
| Root Directory | Yes | The local directory that becomes the POSIX catalog root |
| Mount Point | No | POSIX path prefix; defaults to `/` |
| Read-Only | — | Unchecked by default |

### Native-specific fields

| Field | Required | Notes |
|---|---|---|
| Root Path | Yes | Local directory path using host platform conventions |
| Read-Only | — | Unchecked by default |

Validation runs on confirmation. If a field is invalid (duplicate name, inaccessible path,
illegal characters), an inline error appears next to the offending field without closing the
dialog.

---

## Allocating a Mainframe Dataset

Right-click any node within a Mainframe catalog and choose **Allocate Dataset** to open the
Dataset Allocation dialog. Fields follow ISPF conventions:

| Field | Notes |
|---|---|
| Dataset Name | Full DSN or partial; Default HLQ is prepended if configured |
| Dataset Organization | PS, PO, PDSE, or GDG |
| Record Format (RECFM) | FB, F, VB, V, or U |
| Logical Record Length (LRECL) | Default: 80 |
| Block Size (BLKSIZE) | Default: 27920 |
| Directory Blocks | PDS/PDSE only; default: 10 |
| GDG Limit | GDG only; 1–255 |
| Scratch on Roll-off | GDG only; checked by default |

**Default allocation values** (applied when fields are left blank):

| DSORG | RECFM | LRECL | BLKSIZE |
|---|---|---|---|
| PS | FB | 80 | 27920 |
| PO (PDS/PDSE) | FB | 80 | 27920 |
| GDG generation | Inherited from previous generation, or PS defaults if first |

Use **Allocate Like** (right-click an existing dataset) to pre-populate all fields from an
existing dataset — you only need to supply the new dataset name.

---

## Context Menus by Node Type

### Mainframe catalog node
`Unmount Catalog` · `New Dataset…` · `Properties` · `Export Catalog…` · `Refresh`

### Sequential dataset (PS)
`Open` · `Rename…` · `Delete` · `Properties` · `Copy DSN` · `Allocate Like…`

### Partitioned dataset (PDS/PDSE)
`New Member…` · `Rename…` · `Delete` · `Properties` · `Copy DSN` · `Allocate Like…`

### PDS member
`Open` · `Rename…` · `Delete` · `Copy Member Name` · `Properties`

### GDG base
`New Generation…` · `List Generations` · `Properties` · `Delete GDG` · `Copy DSN` · `Modify Limit…`

### GDG generation
`Open` · `Delete` · `Properties` · `Copy DSN`

### POSIX catalog node or directory
`New File` · `New Directory` · `Rename` · `Delete` · `Properties` · `Copy Path`

### Native catalog file
`Open` · `Rename` · `Delete` · `Copy Path` · platform shell actions

### Native catalog directory
`New File` · `New Folder` · `Rename` · `Delete` · `Copy Path` · `Open in Native File Manager` · `Refresh`

---

## Catalog Persistence

All registered catalogs are saved to `session.toml` under the `[virtual_catalogs]` table.
On startup, every catalog with `auto_mount = true` is mounted automatically, in priority order.

Example configuration entries:

```toml
[[virtual_catalogs]]
name = "PAYROLL"
type = "Mainframe"
path = "C:/ffworkbench/catalogs/payroll"
default_hlq = "PAYROLL"
auto_mount = true

[[virtual_catalogs]]
name = "dev-posix"
type = "POSIX"
path = "C:/projects/dev"
mount_point = "/"
read_only = false
auto_mount = true

[[virtual_catalogs]]
name = "projects"
type = "Native"
path = "C:/projects"
read_only = false
auto_mount = true
```

You never need to edit this file manually — the Catalog Manager dialog maintains it.

---

## Editing and Deleting Catalogs

Right-click a catalog node and choose **Properties** to edit it. You can change the
description, auto-mount flag, read-only flag (POSIX/Native), and default HLQ (Mainframe).
The catalog name and type cannot be changed after creation.

To delete a catalog, right-click and choose **Delete Catalog**. A confirmation dialog offers
two options:

- **Delete Catalog Only** — unmounts the catalog and removes it from the registry; backing
  files are left untouched.
- **Delete Catalog and Files** — unmounts, removes from registry, and recursively deletes
  the backing repository or directory.

---

## How the Repository is Laid Out (Mainframe Catalogs)

The physical storage for a Mainframe catalog is a directory with this structure:

```
{repository_root}/
├── catalog.db          ← SQLite database (WAL mode)
├── storage/            ← Sequential dataset files
│   └── PAYROLL/
│       └── INPUT/
│           └── FILE    ← Content of PAYROLL.INPUT.FILE
├── pds/                ← Partitioned dataset directories
│   └── SYS1/
│       └── MACLIB/     ← PDS directory for SYS1.MACLIB
│           ├── ABEND
│           └── OPEN
├── gdg/                ← GDG base directories
│   └── PAYROLL/
│       └── MONTHLY/
│           ├── G0001V00
│           └── G0002V00
└── temp/               ← Cleaned on mount
```

The `catalog.db` database maps every dataset name to its `storage_path` (a relative path
from the repository root). National characters (`@`, `#`, `$`) in dataset names are
percent-encoded in filesystem paths (`%40`, `%23`, `%24`) for cross-platform compatibility.

---

## Architecture Summary

```
POM Option 1 pressed
  └─ Files panel renders
       ├─ Mainframe section  →  ff-dscatalog VFS provider  (scheme: catalog)
       ├─ POSIX section      →  posix_provider              (scheme: posix)
       └─ Native section     →  connector-local-fs          (scheme: local)

All operations flow through ff-vfs (FFW-ARCH-001):
  vfs://catalog/PAYROLL.INPUT.FILE  →  CatalogVfsProvider  →  SQLite lookup  →  physical file
  vfs://posix/dev-posix/src/main.rs →  PosixProvider       →  local FS (POSIX conventions)
  vfs://local/C:/projects/file.txt  →  LocalFsProvider     →  local FS (host conventions)
```

The VFS provider registry is keyed by scheme. When a catalog is mounted, its provider is
registered; when unmounted, it is deregistered. The editor, search engine, compare tool, and
JCL resolver all use the same `vfs://` URIs — they are unaware of which catalog type is
backing a given resource.

---

## LISTCAT and LISTDS Commands

Type these in any `Command ===>` field to query catalog contents:

**LISTCAT** — list datasets matching a filter pattern:
```
LISTCAT PAYROLL.*
LISTCAT PAY.% TYPE(PS)
LISTCAT *.INPUT.* CATALOG(PAYROLL)
```

Wildcard rules: `*` matches zero or more characters across qualifiers; `%` matches exactly
one qualifier.

**LISTDS** — show detailed attributes for a specific dataset:
```
LISTDS PAYROLL.INPUT.FILE
LISTDS SYS1.MACLIB MEMBERS
LISTDS PAYROLL.MONTHLY HISTORY
```

Output includes DSN, DSORG, RECFM, LRECL, BLKSIZE, creation date, last modified date,
physical size, physical path, and catalog name. With `MEMBERS`, a PDS member list is appended.

---

## Export and Import

Right-click a Mainframe catalog node and choose **Export Catalog…** to package the entire
catalog (database + repository) into a portable ZIP archive. The archive includes a
`manifest.json` with catalog name, description, export timestamp, dataset count, and schema
version.

To restore a catalog from an archive, right-click the Catalogs root node and choose
**Import Catalog…**, then select the archive and a target directory. The system validates
the archive integrity and schema version before extracting.
