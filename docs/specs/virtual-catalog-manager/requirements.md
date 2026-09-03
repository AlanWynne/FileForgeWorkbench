# Requirements Document

## Introduction

This spec defines the **Virtual Catalog Manager** for FileForgeWorkbench — the unified UI subsystem
that owns POM Option 1 ("Files") and provides all dialogs and panels for creating, managing, and
browsing virtual file catalogs of four distinct types:

| Catalog Type | VFS Scheme | Description |
|---|---|---|
| **Mainframe** | `catalog` | z/OS-style datasets (PS, PDS, GDG) backed by `ff-dscatalog` |
| **POSIX** | `posix` | POSIX-style hierarchical filesystem emulation (new provider) |
| **Native** | `local` | The host platform's local filesystem (the host platform (Windows, Linux, or macOS)) surfaced through the VFS |

The Virtual Catalog Manager is rendered as a full-tab panel when the user selects option `1` from
the Primary Option Menu (or types `1` / `FILES` in any command field). It replaces the previous
behaviour of option 1 opening the native Windows file explorer.

### Design Principles

1. **All catalog types are first-class.** Mainframe, POSIX, Windows, and Local catalogs are
   presented with equal prominence in the same unified explorer.
2. **VFS-backed throughout.** Every file operation goes through the VFS abstraction layer
   (FFW-ARCH-001). The UI never calls `std::fs` directly.
3. **Dialog-driven management.** Catalog creation, dataset allocation, and POSIX file management
   are performed through modal dialogs launched from the explorer panel.
4. **ISPF heritage.** Dialog layouts and terminology follow ISPF conventions where applicable
   (e.g., dataset allocation uses RECFM/LRECL/BLKSIZE field names).

### Source References

- **[ISPF-POM]** = ISPF Primary Option Menu heritage
- **[DSC]** = Dataset Catalog Brief (mainframe catalog operations)
- **[WB]** = Workbench Architecture Brief (VFS principle FFW-ARCH-001)
- **[FFE-TREE]** = FileForgeEditor file-tree-panel specification

### Cross-References

| Sub-Project | Relationship |
|---|---|
| `startup-and-session` | POM option 1 routes to this panel (Req 14.6 extension) |
| `virtual-file-system` | All resource access goes through VFS providers |
| `dataset-catalog` | Mainframe catalog CRUD delegated to `ff-dscatalog` |
| `file-tree-panel` | Explorer tree reused/embedded within the Files panel |
| `connector-local-fs` | Native catalog type backed by this provider |

---

## Glossary

| Term | Definition |
|---|---|
| **Virtual_Catalog** | A named, typed container registered with the VFS that groups related files or datasets. Has one of four types: Mainframe, POSIX, Windows, Local. |
| **Catalog_Type** | The classification of a Virtual_Catalog: `Mainframe` (z/OS dataset emulation), `POSIX` (hierarchical POSIX filesystem emulation), `Native` (the host platform's local filesystem — the host platform (Windows, Linux, or macOS)). |
| **Files_Panel** | The full-tab panel rendered when POM option 1 is selected. Contains the catalog tree, toolbar, and action buttons. |
| **Catalog_Manager_Dialog** | The modal dialog for creating, editing, and deleting Virtual_Catalogs. |
| **Dataset_Allocation_Dialog** | The modal dialog for allocating (creating) a new mainframe-style dataset within a Mainframe catalog. |
| **POSIX_File_Dialog** | The modal dialog for creating, renaming, and deleting files and directories within a POSIX catalog. |
| **Catalog_Registry** | The in-memory and persisted list of all defined Virtual_Catalogs, keyed by catalog name and type. |
| **POSIX_Catalog** | A Virtual_Catalog of type POSIX — a directory on the local filesystem presented as a POSIX-style hierarchical filesystem through the `posix` VFS provider. |
| **POSIX_Provider** | A new VFS provider (scheme `posix`) that maps a root directory to a POSIX-style namespace, enforcing POSIX path conventions and permissions model. |

---

## Requirements

### Requirement 1: POM Option 1 — Files Panel

**User Story:** As an ISPF-familiar operator, I want POM option 1 to open a dedicated Files panel
that gives me access to all my virtual file catalogs, so that I can manage mainframe datasets,
POSIX files, and local files from a single unified interface.

**Source:** [ISPF-POM] option 1 re-definition; [WB] VFS-unified explorer.

#### Acceptance Criteria

1.1 WHEN the user selects option `1` from the Primary Option Menu (or types `1` or `FILES` in any
    `Command ===>` field), THE shell SHALL transform the current POM tab into a Files_Panel tab
    with title `[FILES]`. [ISPF-POM]

1.2 THE Files_Panel SHALL display a split layout: a left-side catalog tree (showing all registered
    Virtual_Catalogs grouped by type) and a right-side content area (showing the contents of the
    selected catalog node). [WB]

1.3 THE Files_Panel SHALL display a toolbar at the top with the following actions: `New Catalog`,
    `Open`, `Refresh`, `Properties`, and a search/filter input. [WB]

1.4 THE catalog tree SHALL group catalogs under three collapsible section headers:
    `Mainframe Catalogs`, `POSIX Catalogs`, `Native Catalogs`. The `Native Catalogs` header
    SHALL include a parenthetical platform label at runtime: e.g., `Native Catalogs (Windows)`,
    `Native Catalogs (Linux)`, `Native Catalogs (macOS)`. [WB]

1.5 WHEN no catalogs of a given type exist, THE section header SHALL display a greyed child node
    reading `No catalogs defined — click New Catalog to create one`. [WB]

1.8 THE Files_Panel SHALL display three catalog type sections (not four). There is no separate
    "Windows" and "Local" distinction — both are unified under `Native`. [WB]

1.6 THE Files_Panel SHALL be navigable via the `Command ===>` field: typing a DSN or path and
    pressing Enter SHALL navigate the tree to that resource. [ISPF-POM]

1.7 WHEN the user presses `PF3` / `F3` or types `END` in the Files_Panel command field, THE shell
    SHALL return the tab to the Primary Option Menu view. [ISPF-POM]

---

### Requirement 2: Catalog Registry

**User Story:** As a user, I want all my virtual catalogs persisted and restored across sessions,
so that I never have to re-register them after restarting the workbench.

**Source:** [WB] session persistence; [DSC] catalog mount persistence.

#### Acceptance Criteria

2.1 THE Catalog_Registry SHALL persist all defined Virtual_Catalogs to the workbench configuration
    under the `[virtual_catalogs]` TOML table, including: name, type, path/repository, description,
    and auto-mount flag. [WB]

2.2 WHEN the workbench starts, THE Catalog_Registry SHALL load all persisted catalogs and
    auto-mount those with `auto_mount = true`. [WB]

2.3 THE Catalog_Registry SHALL support registering a new catalog, updating an existing catalog's
    properties, and removing a catalog (with optional deletion of backing storage). [WB]

2.4 WHEN a catalog is registered, THE Catalog_Registry SHALL validate that the name is unique
    across all catalog types and that the backing path is accessible. [WB]

2.5 THE Catalog_Registry SHALL expose a query API: list all catalogs, list by type, get by name,
    check existence. [WB]

---

### Requirement 3: Catalog Manager Dialog — Create

**User Story:** As a user, I want a dialog to create new virtual catalogs of any type, so that I
can set up my working environment without editing configuration files manually.

**Source:** [DSC] catalog creation; [WB] dialog-driven management.

#### Acceptance Criteria

3.1 WHEN the user clicks `New Catalog` in the Files_Panel toolbar or right-clicks a section header
    and selects `New Catalog`, THE shell SHALL open the Catalog_Manager_Dialog. [WB]

3.2 THE Catalog_Manager_Dialog SHALL present a `Catalog Type` selector with three options:
    `Mainframe`, `POSIX`, `Native`. [WB]

3.3 THE Catalog_Manager_Dialog SHALL present the following common fields for all catalog types:
    - `Catalog Name` (required, 1–32 alphanumeric/hyphen/underscore characters)
    - `Description` (optional, free text up to 120 characters)
    - `Auto-mount on startup` (checkbox, default: checked)
    [WB]

3.4 WHEN `Mainframe` is selected, THE dialog SHALL additionally present:
    - `Repository Path` (required — directory where `catalog.db` and storage subdirs will be created)
    - `Default HLQ` (optional — prepended to bare qualifiers)
    - `Create repository now` (checkbox, default: checked)
    [DSC]

3.5 WHEN `POSIX` is selected, THE dialog SHALL additionally present:
    - `Root Directory` (required — the local directory that becomes the POSIX catalog root)
    - `Mount Point` (optional — the POSIX path prefix, default: `/`)
    - `Read-Only` (checkbox, default: unchecked)
    [WB]

3.6 WHEN `Native` is selected, THE dialog SHALL additionally present:
    - `Root Path` (required — the local directory path to expose, using the host platform's
      path conventions: backslash on Windows, forward-slash on Linux/macOS)
    - `Read-Only` (checkbox, default: unchecked)
    [WB]

3.7 WHEN the user confirms the dialog, THE system SHALL validate all fields, create the catalog
    (initialising the repository for Mainframe type), register it in the Catalog_Registry, and
    mount it immediately if `Auto-mount` is checked. [WB]

3.8 WHEN validation fails (duplicate name, inaccessible path, invalid characters), THE dialog
    SHALL display an inline error message adjacent to the offending field without closing. [WB]

---

### Requirement 4: Catalog Manager Dialog — Edit and Delete

**User Story:** As a user, I want to edit catalog properties and delete catalogs I no longer need,
so that I can keep my catalog registry clean and up to date.

**Source:** [DSC] catalog lifecycle; [WB] dialog-driven management.

#### Acceptance Criteria

4.1 WHEN the user right-clicks a catalog node and selects `Properties`, THE shell SHALL open the
    Catalog_Manager_Dialog pre-populated with the catalog's current properties. [WB]

4.2 THE edit dialog SHALL allow changing: Description, Auto-mount flag, Read-Only flag (POSIX/
    Native), and Default HLQ (Mainframe). The Catalog Name and Type SHALL NOT be editable
    after creation. [WB]

4.3 WHEN the user right-clicks a catalog node and selects `Delete Catalog`, THE shell SHALL
    display a confirmation dialog: `Delete catalog "{name}"? This will unmount it. Optionally
    delete all backing files.` with options `Delete Catalog Only`, `Delete Catalog and Files`,
    and `Cancel`. [WB]

4.4 WHEN `Delete Catalog Only` is confirmed, THE system SHALL unmount the catalog and remove it
    from the Catalog_Registry without touching the backing files. [WB]

4.5 WHEN `Delete Catalog and Files` is confirmed, THE system SHALL unmount the catalog, remove it
    from the Catalog_Registry, and recursively delete the backing repository/directory. [WB]

---

### Requirement 5: Mainframe Dataset Allocation Dialog

**User Story:** As a mainframe developer, I want a dialog to allocate new datasets within a
Mainframe catalog using familiar ISPF-style fields, so that I can create datasets without
memorising command syntax.

**Source:** [DSC] dataset allocation; [ISPF-POM] ISPF dialog heritage.

#### Acceptance Criteria

5.1 WHEN the user right-clicks a Mainframe catalog node (or any node within it) and selects
    `Allocate Dataset`, THE shell SHALL open the Dataset_Allocation_Dialog. [DSC]

5.2 THE Dataset_Allocation_Dialog SHALL present the following fields in ISPF style:
    - `Dataset Name` (required — full DSN or partial; HLQ prepended if configured)
    - `Dataset Organization` (DSORG selector: PS, PO, PDSE, GDG)
    - `Record Format` (RECFM selector: FB, F, VB, V, U)
    - `Logical Record Length` (LRECL — integer, default 80)
    - `Block Size` (BLKSIZE — integer, default 0 — system-determined; 0 means the host OS and Rust I/O layer determine optimal buffering; IBM recommends `BLKSIZE=0` so that z/OS — or in FFWB's case the host OS — selects the optimal block size for the underlying storage device; a non-zero value may be entered as a user override)
    - `Directory Blocks` (integer, shown only when DSORG = PO or PDSE, default 10)
    - `GDG Limit` (integer 1–255, shown only when DSORG = GDG)
    - `Scratch on Roll-off` (checkbox, shown only when DSORG = GDG, default: checked)
    - `Description` (optional free text)
    [DSC]

5.3 WHEN the user confirms the dialog, THE system SHALL validate all fields per the rules in
    `dataset-catalog` Requirement 7 and Requirement 2, then invoke `dataset.allocate` via the
    command framework. [DSC]

5.4 WHEN allocation succeeds, THE dialog SHALL close and the new dataset SHALL appear in the
    catalog tree immediately. [DSC]

5.5 WHEN allocation fails (duplicate DSN, invalid parameters), THE dialog SHALL display the error
    inline without closing. [DSC]

5.6 THE dialog SHALL support an `Allocate Like` mode: WHEN launched from a right-click on an
    existing dataset node with `Allocate Like`, all fields SHALL be pre-populated from the source
    dataset's attributes, requiring only the new DSN to be entered. [DSC]

5.7 WHEN the Allocate Dataset dialog opens for a Mainframe catalog that has a Default HLQ
    configured, THE `Dataset Name` field SHALL be pre-populated with `"{HLQ}."` so the user
    only needs to type the remaining qualifiers. [DSC]

5.8 WHEN the user confirms the Allocate Dataset dialog for a Mainframe catalog, THE system
    SHALL convert the Dataset Name to uppercase before validation and storage. [DSC]

5.9 WHEN the user confirms the Allocate Dataset dialog, THE system SHALL reject the allocation
    if a dataset with the same name (case-insensitive) already exists in the target catalog,
    displaying an inline error without closing the dialog. [DSC]

---

### Requirement 6: Mainframe Dataset Management

**User Story:** As a mainframe developer, I want to rename, delete, and view properties of
datasets and PDS members directly from the Files panel, so that I can manage my catalog without
leaving the workbench.

**Source:** [DSC] dataset CRUD; [ISPF-POM] ISPF heritage.

#### Acceptance Criteria

6.1 WHEN the user right-clicks a sequential dataset (PS) node, THE context menu SHALL include:
    `Open`, `Rename`, `Delete`, `Properties`, `Copy DSN`, `Allocate Like`. [DSC]

6.2 WHEN the user right-clicks a partitioned dataset (PDS/PDSE) node, THE context menu SHALL
    include: `New Member`, `Rename`, `Delete`, `Properties`, `Copy DSN`, `Allocate Like`. [DSC]

6.3 WHEN the user right-clicks a PDS member node, THE context menu SHALL include:
    `Open`, `Rename`, `Delete`, `Copy Member Name`. [DSC]

6.4 WHEN the user right-clicks a GDG base node, THE context menu SHALL include:
    `New Generation`, `List Generations`, `Properties`, `Delete GDG`, `Modify Limit`. [DSC]

6.5 WHEN `Rename` is selected on a dataset, THE shell SHALL display an inline rename field
    pre-filled with the current DSN, validated on Enter. [DSC]

6.6 WHEN `Delete` is selected on a dataset, THE shell SHALL display a confirmation dialog before
    dispatching `dataset.delete`. [DSC]

6.7 WHEN `Properties` is selected, THE shell SHALL display a Properties panel showing all dataset
    attributes as defined in `dataset-catalog` Requirement 11. [DSC]

---

### Requirement 7: POSIX Catalog Provider

**User Story:** As a developer, I want a POSIX virtual filesystem catalog that maps a local
directory to a POSIX-style namespace, so that I can work with files using POSIX path conventions
within the workbench.

**Source:** [WB] VFS extensibility; new requirement.

#### Acceptance Criteria

7.1 THE workbench SHALL provide a `posix` VFS provider (scheme `posix`) that maps a configured
    root directory to a POSIX-style hierarchical namespace. [WB]

7.2 THE POSIX provider SHALL implement the `VfsProvider` trait, supporting: `read`, `write`,
    `create`, `delete`, `rename`, `list`, `stat`, `exists`. [WB]

7.3 THE POSIX provider SHALL enforce POSIX path conventions: forward-slash separator, case-
    sensitive names, no drive letters, paths starting with `/` relative to the catalog root. [WB]

7.4 WHEN a POSIX catalog is mounted, THE provider SHALL register under scheme `posix` with a
    catalog-name sub-path, making resources addressable as `vfs://posix/{catalog-name}/{path}`. [WB]

7.5 THE POSIX provider SHALL support creating directories (`mkdir`) and files, and deleting them
    recursively when requested. [WB]

7.6 WHEN a POSIX catalog is configured as read-only, THE provider SHALL return
    `VfsError::PermissionDenied` for any write, create, delete, or rename operation. [WB]

7.7 THE POSIX provider SHALL advertise capabilities: `Read`, `Write`, `List`, `Metadata`,
    `Create`, `Delete`, `Rename`, `Watch` (delegated to the underlying local filesystem watcher). [WB]

---

### Requirement 8: POSIX File Management Dialog

**User Story:** As a developer, I want dialogs to create, rename, and delete files and directories
within a POSIX catalog, so that I can manage POSIX-style files without leaving the workbench.

**Source:** [WB] dialog-driven management; new requirement.

#### Acceptance Criteria

8.1 WHEN the user right-clicks a POSIX catalog node or directory within it, THE context menu
    SHALL include: `New File`, `New Directory`, `Rename`, `Delete`, `Properties`, `Copy Path`. [WB]

8.2 WHEN `New File` is selected, THE shell SHALL display an inline input for the filename,
    validated on Enter (no path separators, no null bytes, length 1–255). [WB]

8.3 WHEN `New Directory` is selected, THE shell SHALL display an inline input for the directory
    name with the same validation rules. [WB]

8.4 WHEN `Rename` is selected, THE shell SHALL display an inline rename field pre-filled with the
    current name. [WB]

8.5 WHEN `Delete` is selected on a non-empty directory, THE shell SHALL display a confirmation
    dialog: `Delete directory "{name}" and all its contents? This cannot be undone.` [WB]

8.6 WHEN `Properties` is selected, THE shell SHALL display a properties panel showing: full POSIX
    path, size, last modified, permissions (read/write/execute), and catalog name. [WB]

---

### Requirement 9: Native Catalog Browsing

**User Story:** As a user on any platform, I want to register local directories as named Native
catalogs so that I can access them from the Files panel alongside my mainframe and POSIX catalogs,
regardless of whether the host OS is the host platform (Windows, Linux, or macOS).

**Source:** [WB] unified explorer; [FFE-TREE] local filesystem browsing.

#### Acceptance Criteria

9.1 WHEN a Native catalog is mounted, THE Files_Panel SHALL display it under the `Native Catalogs`
    section header (with platform label) with its configured name. [WB]

9.2 THE content area for a Native catalog SHALL render the directory tree via the
    `connector-local-fs` VFS provider, identical to the existing File_Tree_Panel behaviour.
    The provider handles path conventions for the host platform transparently. [WB]

9.3 WHEN the user right-clicks a file node in a Native catalog, THE context menu SHALL include:
    `Open`, `Rename`, `Delete`, `Copy Path`, and platform-appropriate shell actions:
    - On Windows: `Open Containing Folder in Explorer`, `Open in CMD`, `Open in PowerShell`
    - On Linux: `Open Containing Folder in Files`, `Open in Terminal`
    - On macOS: `Reveal in Finder`, `Open in Terminal`
    [WB]

9.4 WHEN the user right-clicks a directory node in a Native catalog, THE context menu SHALL
    include: `New File`, `New Folder`, `Rename`, `Delete`, `Copy Path`,
    `Open in Native File Manager`, `Refresh`. [WB]

9.5 WHEN a Native catalog is configured as read-only, ALL write operations (create, rename,
    delete) SHALL be disabled in the context menu and return an error if attempted via command. [WB]

---

### Requirement 10: Files Panel — Unified Explorer View

**User Story:** As a user, I want the right-side content area of the Files panel to show the
contents of whatever catalog node I have selected, so that I can browse files without expanding
the tree manually.

**Source:** [WB] unified explorer; [FFE-TREE] content area.

#### Acceptance Criteria

10.1 WHEN the user selects a catalog node in the left tree, THE right content area SHALL display
     the immediate children of that node in a list/grid view with columns: Name, Type, Size,
     Modified Date. [WB]

10.2 THE content area SHALL support sorting by any column header (click to sort ascending,
     click again to sort descending). [WB]

10.7 WHEN the content area is sorted by Name (the default), THE sort SHALL group container
     entries (directories, PDS, GDG bases) before non-container entries (files, PS datasets,
     members), with each group sorted case-insensitively in alphabetical order. [WB]

10.3 WHEN the user double-clicks a file/member/dataset node in the content area, THE shell SHALL
     open it in a new editor tab. [WB]

10.4 WHEN the user double-clicks a directory/container node in the content area, THE shell SHALL
     navigate into that node (updating both the tree selection and the content area). [WB]

10.5 THE content area SHALL display a breadcrumb path bar at the top showing the current location
     within the catalog hierarchy, with each segment clickable for navigation. [WB]

10.6 THE content area SHALL support a filter/search input that filters the displayed entries by
     name (case-insensitive substring match). [WB]

---

### Requirement 12: Catalog Storage Default Paths

**User Story:** As a user, I want the workbench to suggest sensible default storage locations
when I create a new Mainframe or POSIX catalog, so that I do not have to type a path from
scratch every time, and so that all catalog data is stored in a predictable, well-organised
location by default.

**Source:** [WB] configuration-system; [DSC] repository root.

#### Acceptance Criteria

12.1 WHEN the Catalog_Manager_Dialog opens for a new Mainframe catalog, THE `Repository Path`
     field SHALL be pre-populated with the value of the configuration key
     `catalogs.default_mainframe_root`, with the new catalog name appended as a subdirectory
     (e.g., `{default_mainframe_root}/{catalog-name}`). [WB]

12.2 WHEN the Catalog_Manager_Dialog opens for a new POSIX catalog, THE `Root Directory`
     field SHALL be pre-populated with the value of the configuration key
     `catalogs.default_posix_root`. [WB]

12.3 THE configuration key `catalogs.default_mainframe_root` SHALL have a built-in default
     value of `{user_data_dir}/catalogs/mainframe`, where `{user_data_dir}` is the
     platform-appropriate user data directory resolved by `ff-core`. [WB]

12.4 THE configuration key `catalogs.default_posix_root` SHALL have a built-in default
     value of `{user_data_dir}/catalogs/posix`, where `{user_data_dir}` is the
     platform-appropriate user data directory resolved by `ff-core`. [WB]

12.5 BOTH configuration keys SHALL be registered in the `ff-config` schema under the
     `[catalogs]` namespace with type `String`, their respective defaults, and a
     human-readable description suitable for display in the Settings panel. [WB]

12.6 WHEN a user changes either key in the Settings panel, THE new value SHALL be persisted
     to the user-layer configuration file and SHALL take effect immediately for any
     subsequently opened Catalog_Manager_Dialog (no restart required). [WB]

12.7 WHEN the pre-populated path does not exist on disk, THE dialog SHALL display it as a
     suggestion only — the path is created only when the user confirms the dialog with
     `Create repository now` checked (Mainframe) or when the POSIX catalog is first mounted. [WB]

---

### Requirement 13: Allocated Dataset Persistence and Display

**User Story:** As a mainframe developer, I want datasets I allocate via the Dataset Allocation
Dialog to appear immediately in the Files Panel content area and persist across sessions, so that
my work is not lost and I can see what I have created.

**Source:** [DSC] dataset CRUD; [WB] session persistence.

#### Acceptance Criteria

13.1 THE `FilesPanelState` SHALL maintain a `datasets` map keyed by catalog name, storing a
     `Vec<AllocatedDataset>` for each catalog. Each `AllocatedDataset` SHALL carry: `name`
     (DSN string), `dsorg` (PS/PO/PDSE/GDG), `recfm`, `lrecl`, `blksize`, and `description`.

13.2 WHEN the Dataset_Allocation_Dialog confirms with `AllocOutcome::Confirmed`, THE shell SHALL
     extract the validated `AllocParams` from the form and insert a new `AllocatedDataset` entry
     into the `datasets` map under the catalog name that was right-clicked to open the dialog.

13.3 WHEN a catalog node is selected in the left tree, THE right content area SHALL populate
     `ContentAreaState::entries` from the `datasets` map for that catalog, converting each
     `AllocatedDataset` to a `ContentEntry` (name, dsorg as type, empty size, empty modified,
     `is_container = false` for PS; `is_container = true` for PO/PDSE/GDG).

13.4 THE `datasets` map SHALL be persisted to the session TOML under
     `[catalog_datasets.<catalog_name>]` and restored on next launch so that allocated datasets
     survive application restarts.

13.5 WHEN a catalog is deleted from the registry, ALL datasets stored under that catalog name
     SHALL also be removed from the `datasets` map.

---

### Requirement 14: Default Home Catalog on First Launch

**User Story:** As a new user, I want the workbench to automatically create a Native catalog
pointing to my home directory when no Native catalogs exist, so that the Files panel shows
useful content immediately without any manual setup.

**Source:** [WB] first-run experience; [FFE-STARTUP] graceful startup.

#### Acceptance Criteria

14.1 WHEN the workbench starts and the loaded `CatalogRegistry` contains no catalogs of type
     `Native`, THE startup sequence SHALL automatically create a `VirtualCatalog` with:
     - `name = "Home"`
     - `catalog_type = CatalogType::Native`
     - `path` = the user's home directory (resolved via `dirs::home_dir()` or equivalent,
       falling back to the process working directory if the home directory cannot be determined)
     - `auto_mount = true`
     - `read_only = false`
     - `description = Some("Default home directory catalog")`
     [WB]

14.2 WHEN the default Home catalog is created, THE startup sequence SHALL register it in the
     `CatalogRegistry` immediately so it is visible in the Files panel on the same launch. [WB]

14.3 WHEN the default Home catalog is created, THE `CatalogRegistry` SHALL be persisted to
     `catalogs.toml` before the first frame is rendered, so that the catalog survives
     application restart without being re-created. [WB]

14.4 WHEN the `CatalogRegistry` already contains one or more `Native` catalogs on startup,
     THE startup sequence SHALL NOT create the default Home catalog — the user's existing
     Native catalogs take precedence. [WB]

14.5 WHEN the home directory cannot be determined at startup, THE startup sequence SHALL
     fall back to the process working directory as the catalog path, log a WARN-level record,
     and still create and persist the catalog. [WB]

14.6 THE default Home catalog named `"Home"` SHALL be protected from deletion: WHEN the
     user attempts to delete a catalog whose name is `"Home"` and whose type is `Native`,
     THE Catalog Manager Dialog SHALL reject the deletion and display an inline error:
     `"The Home catalog cannot be deleted. Rename or edit it instead."` [WB]

14.7 THE Home catalog MAY be renamed or edited (path, description, auto-mount, read-only)
     via the Catalog Manager Dialog. WHEN it is renamed, the name-based deletion guard
     (Req 14.6) no longer applies to the renamed catalog. [WB]

---

### Requirement 15: Catalog Properties — Repository Path Display

**User Story:** As a user, I want to see the repository path when viewing a catalog's properties,
so that I know where the catalog's data is stored on disk.

**Source:** CR-NR-012; [WB] dialog-driven management.

#### Acceptance Criteria

15.1 WHEN the user opens the Properties / Edit dialog for any catalog, THE dialog SHALL display
     the catalog's repository path (the `path` field of `VirtualCatalog`) as a read-only labelled
     field labelled `Repository Path:`. [WB]

15.2 THE repository path field SHALL be visible for all catalog types: Mainframe, POSIX, and
     Native. [WB]

15.3 THE repository path field SHALL be read-only in the Edit dialog — the path cannot be changed
     after catalog creation. [WB]

---

### Requirement 16: VFS Dataset Path Resolution from Catalog Repository

**User Story:** As a mainframe developer, I want to open a Mainframe dataset in the editor and
have FFWB locate the actual file on disk by combining the catalog's repository path with the
dataset name, so that I can read and edit the dataset's content.

**Source:** CR-NR-012; [WB] VFS-backed throughout; [DSC] dataset storage.

#### Acceptance Criteria

16.1 WHEN a Mainframe dataset is opened, THE system SHALL resolve its physical file path by
     delegating to the StorageProvider via the catalogue locator stored for that dataset.
     The catalogue is the sole authority mapping logical DSN to physical location (see
     dataset-catalog Requirement 20.3). The physical path SHALL NOT be derived by replacing
     dots in the DSN with directory separators -- that rule applied only to the legacy
     DSN-derived layout (dataset-catalog Requirement 4, now superseded for new allocations).
     For datasets allocated under the UUID layout, the resolved path will be of the form
     `{workspace}/datasets/objects/<uuid>.dat`. [WB]

16.2 WHEN the resolved physical path exists on disk, THE system SHALL open it in a new editor
     tab using the existing `file.open` VFS path. [WB]

16.3 WHEN the resolved physical path does not exist on disk, THE system SHALL create the
     file and any missing parent directories using the staged create protocol defined in
     dataset-catalog Requirement 25.1 (stage, reserve, publish, activate). For legacy
     DSN-derived repositories only, direct file creation at the DSN-derived path is
     permitted as a compatibility measure. This matches ISPF behaviour where allocating
     a dataset reserves it and opening it for the first time creates the physical file. [WB]

16.6 WHEN creating the physical file or its parent directories fails (e.g. permission
     denied), THE system SHALL display an error message:
     `'<DSN>': cannot create dataset file at <resolved_path>: <os_error>`. [WB]

16.4 WHEN a catalog has no repository path configured (empty string), THE system SHALL display
     the message: `'<DSN>': catalog has no repository path configured`. [WB]

16.5 THE path resolution logic SHALL be a pure function `resolve_dataset_path(repository_path, dsn) -> Option<PathBuf>`, testable without VFS or egui. [WB]


> **Note:** Requirement 11 was added during Phase AA after Requirements 12–16. The numbering is preserved for test annotation compatibility.

**User Story:** As an ISPF-familiar operator, I want POM option 1 to be clearly labelled as the
Files/Catalog manager, so that the menu accurately reflects what the option does.

**Source:** [ISPF-POM] Req 14.3 update.

#### Acceptance Criteria

11.1 THE Primary Option Menu option `1` label SHALL read `Files` with description
     `Virtual File Catalogs — Mainframe, POSIX, Native`. [ISPF-POM]

11.2 WHEN the user selects option `1`, THE tab title SHALL change to `[FILES]` and the tab kind
     SHALL be `FilesPanel` (a new `TabKind` variant). [ISPF-POM]

11.3 THE `[FILES]` tab SHALL persist in the session and be restored on next launch as a
     `FilesPanel` tab kind. [WB]
