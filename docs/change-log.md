# FileForge Workbench â€” Change Log

Tracks every new requirement and change request raised via user prompts.
Entries are appended automatically by the prompt-triage rule.
Never delete a row â€” update `Status` in-place.

---

## Status Values

| Status | Meaning |
|--------|---------|
| `PENDING GATE` | Logged, requirements gate not yet started |
| `IN PROGRESS` | Gate running or implementation underway |
| `DONE` | Merged and tests passing |
| `DEFERRED` | Accepted but postponed to a later phase |
| `REJECTED` | Decided not to implement |

---

## New Requirements

New capabilities that did not previously exist.

### CR-NR-001 â€” Prompt triage and change tracking
- **Date/Phase**: Phase AS
- **Prompt**: "Can we create a steering rule that every prompt is evaluated as a bug or a new requirement"
- **Description**: Add a steering rule that classifies every user prompt as a bug, new requirement, change request, question, task, or refactor. Bugs are logged to `docs/bugs.md`; new requirements and change requests are logged to `docs/change-log.md`.
- **Status**: DONE
- **Linked spec**: `.amazonq/rules/prompt-triage.md` (new rule file)

### CR-NR-002 â€” File Explorer Panel (POM option 2)
- **Date/Phase**: Phase AS
- **Prompt**: "opetion 2 needs to be a file Exploere. it should have nodes for each open catalog, and list the files that belong in the catalog in a tree view. Option 2 can be invoked by typing =files and pressing enter, Typeing =2 and pressing enter"
- **Description**: POM option 2 becomes a File Explorer panel showing all open catalogs as tree nodes with their files listed beneath. Commands `=2` and `=FILES` close the current context and switch to the Files context in-place; `FILES` (no `=`) opens a new tab in the Files context.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Requirement 19

### CR-NR-004 â€” Default Native catalog pointing to user home directory on first launch
- **Date/Phase**: Phase AX
- **Prompt**: "By Default the when FFWB starts up if there are no native catalogs in existence, it should create a native catalog pointing to the users home directory, and mount it immediately, so that when the files context window is opened we at least can see the users home directory. Once created this default catalog should persist and be there on next start up."
- **Description**: On first launch (or any launch where the catalog registry contains no Native catalogs), FFWB shall automatically create a Native catalog named `Home` pointing to the user's home directory, register it with `auto_mount = true`, and persist it so it survives subsequent restarts.
- **Status**: DONE

### CR-NR-003 â€” HLQ pre-population in Allocate Dataset dialog
- **Date/Phase**: Phase AW
- **Prompt**: "when defining a dataset catalog we are asked for a default high level qualifier, this should be pre-populated in the allocate dataset dataset name text box"
- **Description**: When the Allocate Dataset dialog opens for a Mainframe catalog that has a Default HLQ configured, the Dataset Name field shall be pre-populated with that HLQ followed by a dot, so the user only needs to type the remaining qualifiers.
- **Status**: DONE
- **Linked spec**: `docs/specs/virtual-catalog-manager/requirements.md` Requirement 5.2 (new criterion 5.7)

### CR-NR-006 â€” File Explorer context menu with file operations
- **Date/Phase**: Phase AZ
- **Prompt**: "When right clicking on a file in the file tree a popup menu should appear with normal file operations listed..."
- **Description**: Right-clicking any node in the File Explorer Panel shall display a context menu whose items are determined by the combination of catalog type, node kind, and file extension. Menus cover Native files/directories, POSIX files (read-only), and Mainframe datasets/members/GDG. Copy puts the full file path on the OS clipboard; pasting into an editor tab prompts for file name vs file contents. Rename is inline label edit. Move To / Copy To use ff-bgio with a progress indicator. Git submenu and Submit JCL are present but greyed-out (deferred).
- **Status**: DONE â€” Phase AZ complete, 431 ff-desktop tests passing
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 16)

### CR-NR-007 â€” Open With Default Application (file type association launch)
- **Date/Phase**: Phase BA
- **Prompt**: "In Windows file extensions often determine the type of file... When double clicking on these files... The application should launch the appropriate application and open the file."
- **Description**: Double-clicking or selecting "Open" on a Native/POSIX file node shall launch the OS default application when the file is binary or maps to an external file class (.docx, .xlsx, .pdf, .png, etc.). Text/source files continue to open in the FFWB editor. Platform dispatch: Windows=ShellExecuteEx verb "open", macOS=`open`, Linux=`xdg-open`. "Open With..." shows the platform picker. Non-blocking via `Command::spawn()`. Mainframe datasets always open in FFWB.
- **Status**: DONE â€” Phase BA complete, 443 ff-desktop tests passing
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 17)

### CR-NR-005 â€” File Explorer: expandable subdirectories and scrollable panel
- **Date/Phase**: Phase AY
- **Prompt**: "i have created a Native catalog called CDRIVE which now shows all the directories and files from the Root directory, Each directory should be able to be expanded to show the files in it, Also the Screen should be scrollable so that we can page down to see more files"
- **Description**: Native catalog directory nodes in the File Explorer shall be expandable (click to show children recursively) and the panel content area shall be scrollable so the user can page through large directory listings.
- **Status**: DONE
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new criteria 15.1â€“15.3)

### CR-NR-009 â€” File Explorer tree: drag-select and copy tree structure to clipboard
- **Date/Phase**: Phase BD
- **Prompt**: "Would it be possible to make the tree copyable, so that i can Drag the mouse pointer over a selection of files, and then copy the selections to paste the tree structure with all the file names into a text file?"
- **Description**: The user wants to drag-select a range of nodes in the File Explorer tree, then copy the selected set as a formatted plain-text tree structure (indented, with directory/file names) to the OS clipboard, so it can be pasted into a text file or editor tab.
- **Status**: DONE â€” Phase BD complete, 474 ff-desktop tests passing

### CR-NR-010 â€” File Explorer keyboard navigation and focus transfer from command line
- **Date/Phase**: Phase BE
- **Prompt**: "if i am positioned on the command line, if i press the tab key i should tab to the first catalog name in the file list area, as I press tab i should tab through the list of files..."
- **Description**: When the File Explorer is the active tab, Tab from the command line transfers focus to the node list. Tab advances through nodes, expanding containers. Arrow keys move without expanding. Shift+Arrow extends selection. Ctrl+Arrow moves cursor without changing selection. Ctrl+Space toggles selection of the cursor node. Escape clears selection.
- **Status**: DONE â€” Phase BE complete, 474 ff-desktop tests passing
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 20)

### CR-NR-012 â€” Catalog properties: show repository path; VFS dataset path resolution
- **Date/Phase**: Phase BJ
- **Prompt**: "When looking at a catalogs properties we dont see the repository path. We should see the repository path. The VFS, should be able to determine a dataset's filename by looking at the dataset's catalog properties and the catalogs repository path to determine where the dataset resides."
- **Description**: (1) The catalog Properties view (Edit Catalog dialog) must display the repository path for all catalog types. (2) When a Mainframe dataset is opened, the VFS resolver must derive the physical file path by combining the catalog's repository path with the dataset name (DSN mapped to a filename), so the file can actually be read from disk.
- **Status**: DONE
- **Linked spec**: `docs/specs/virtual-catalog-manager/requirements.md` (new Requirements 15, 16)

### CR-NR-011 â€” File Explorer file copy and paste (Ctrl+C / Ctrl+V)
- **Date/Phase**: Phase BE
- **Prompt**: "if i then navigate to a new directory. and press ctrl+v all the selected items should be copied to the new location. if i paste into a file i am editing, the list of files should be pasted."
- **Description**: Ctrl+C in the File Explorer copies selected file paths to an internal File_Copy_Clipboard. Ctrl+V in the file list copies the files to the current directory via ff-bgio with progress indicator, collision prompts, and POSIX read-only guard. Ctrl+V in an editor tab opens a prompt to insert file names or file contents at the caret. Mainframe DSN/member paths are supported with naming-rule transformation.
- **Status**: DONE â€” Phase BE complete, 474 ff-desktop tests passing
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 21)

---

## Change Requests

Modifications to existing behaviour that already works.

### CR-CH-001 â€” POM option 2 description update
- **Date/Phase**: Phase AS
- **Prompt**: "opetion 2 needs to be a file Exploere..."
- **Description**: POM option 2 label updated from "View Edit Create and Delete of files" to "File Explorer â€” Browse catalogs and files in a tree view".
- **Affects**: `ff-desktop` `primary_option_menu.rs`, `startup-and-session/requirements.md`
- **Status**: PENDING GATE

### CR-CH-003 â€” Help fallback message uses human-readable context label
- **Date/Phase**: Phase AV
- **Prompt**: "When a help is not available for a specific context the message should display something like 'help not yet available for Context' in a way that we can determine which help needs to be built"
- **Description**: The `resolve_with_fallback` message shall replace the raw `TopicKey` string (e.g. `cmd:FIND`) with a human-readable label (e.g. `command "FIND"`) so developers can identify exactly which help topic needs authoring.
- **Affects**: `ff-help` `context_detector.rs`
- **Status**: DONE

### CR-CH-002 â€” Home catalog deletion blocked
- **Date/Phase**: Phase AX
- **Prompt**: "Deleting the Home catalog should not be allowed!"
- **Description**: Req 14.6 revised: the `"Home"` Native catalog is protected from deletion. The Catalog Manager Dialog shall reject any delete attempt on a catalog named `"Home"` of type `Native` with an inline error. Renaming and editing remain permitted (Req 14.7).
- **Affects**: `ff-desktop` `catalog_manager_dialog.rs`, `virtual-catalog-manager/requirements.md` Req 14.6
- **Status**: DONE

### CR-CH-004 â€” File Explorer default sort: directories alphabetically first, then files alphabetically
- **Date/Phase**: Phase BC
- **Prompt**: "default sort order of files in the file explorer is Directories first and then file in alphabetic order. This is correct but the Directories must also be in alphabetic order first then the files in alphabetic order"
- **Description**: The content area default sort (Name ascending) currently places all directories before files but does not sort the directories themselves alphabetically. Both the directory group and the file group must each be sorted case-insensitively by name.
- **Affects**: `ff-desktop` `files_panel.rs` â€” `visible_entries()` sort comparator; `virtual-catalog-manager/requirements.md` Req 10.2
- **Status**: DONE

### CR-CH-005 â€” Default BLKSIZE to 0 in Dataset Allocation Dialog
- **Date/Phase**: Phase BI
- **Prompt**: "For the moment default blocksize to 0"
- **Description**: Change the default value of the BLKSIZE field in `AllocDatasetForm` from `27920` to `0`. BLKSIZE=0 is the modern z/OS convention meaning system-determined block size. In FFWB, BLKSIZE is metadata only; the host OS handles actual I/O buffering. Validation must also be updated to accept 0 as a valid BLKSIZE (skip the `blksize >= lrecl` guard when blksize is 0).
- **Affects**: `ff-desktop` `dataset_alloc_dialog.rs`
- **Status**: DONE

| Phase | Change |
|-------|--------|
| Phase AS | File created. CR-NR-001 logged â€” prompt triage steering rule. |

### CR-NR-008 â€” File Explorer: sorted file listing with file attributes
- **Date/Phase**: Phase BB
- **Prompt**: "The files list displays in no particular order, we should sort the file by filename order. We should also see more than just the file name but some of the file attributes, like file size, Timestamp created, timestamp Modified, Timestamp accessed. Also perhaps some of the other attributes like the permission attributes in a user friendly way?"
- **Description**: Native catalog file and directory nodes in the File Explorer shall be sorted alphabetically (directories first, then files, both case-insensitive). Each file node shall display file size, created timestamp, modified timestamp, accessed timestamp, and permission attributes (read/write/execute, hidden, system) in a user-friendly format alongside the file name.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 18)

### CR-NR-013 â€” Native File Browser: egui-file-dialog Integration
- **Date/Phase**: Phase BK
- **Prompt**: "Confirm Plan lets proceed.."
- **Description**: Replace the custom `render_native_children()` recursive tree renderer for Native catalog browsing with the `egui-file-dialog` third-party widget. Mainframe and POSIX dataset browsing is unchanged. The widget provides breadcrumbs, bookmarks, search, and keyboard navigation out of the box.
- **Status**: DONE â€” Phase BK complete, 486 ff-desktop tests passing, release build clean
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 22)

### CR-NR-014 â€” File Explorer Panel: egui-file-dialog look-and-feel with catalog mount points
- **Date/Phase**: Phase BM
- **Prompt**: "option 2 from the POM should now be using the egui-file-dialog? is this correct? The idea of the file explorer window context is that it should look and work like the egui-file-dialog works, with each catalog kind of like a mount point"
- **Description**: Redesign the File Explorer Panel (POM option 2) so its overall look and feel matches egui-file-dialog. Catalogs appear as "mounted nodes" in a left sidebar (like drive letters/bookmarks). The right pane shows file contents. Native catalogs use the egui-file-dialog widget. Mainframe catalogs list datasets with dot-separated qualifiers. POSIX catalogs render files and folders with forward-slash paths. Right-click context menu uses egui-file-dialog's native menu for now.
- **Status**: DONE -- Phase BM complete, 497 tests passing (sidebar_width persistence added)
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 23)

### CR-NR-015 — Requirements Review and Modernisation
- **Date/Phase**: Phase BQ
- **Prompt**: "You are acting as a Senior Product Architect, Requirements Engineer, UX Architect, and Software Platform Designer... Perform a comprehensive review of the supplied requirements..."
- **Description**: Comprehensive review of all 65 sub-project specifications. Deliverables: inventory, terminology map, domain classification, gap analysis, rewritten requirements catalogue, traceability matrix, consolidation report, and executive assessment. Work is broken into 10 tasks tracked under `docs/reviews/requirements-review/`.
- **Status**: IN PROGRESS — Task 1 (Inventory) complete
- **Linked spec**: `docs/reviews/requirements-review/inventory.md` (Task 1 complete)

### CR-NR-015 status update — Tasks 1-4 complete
- Tasks 1 (Inventory), 2 (Terminology), 3 (Domain Classification), 4 (Gap Analysis) are DONE.
- Output files: `docs/reviews/requirements-review/inventory.md`, `terminology-map.md`, `domain-classification.md`, `gap-analysis.md`
- Tasks 5-10 pending.

### CR-NR-015 status update — Tasks 9-10 complete — Phase BQ DONE
- Tasks 9 (Consolidation Report) and 10 (Executive Assessment) are DONE.
- Output files: `docs/reviews/requirements-review/consolidation-report.md`, `executive-assessment.md`
- **Status**: DONE — All 10 tasks complete. 497 tests passing. 8 artefacts delivered.

### CR-NR-016 - Mainframe Dataset Architecture and Virtual File/Dataset Storage Requirements
- **Date/Phase**: Phase BS (next)
- **Prompt**: "i have two new markdown files for this project defining how the mainframe dataset and posix catalogs should work and how mainframe files will be emulated..."
- **Description**: Two new architecture documents define: (1) record-oriented storage for PS/PDS/PDSE/GDG/VSAM/ISAM with no CRLF/LF record boundaries; (2) hybrid storage — SQLite as catalogue, native files for sequential/library content; (3) StorageProvider abstraction layer separate from VfsProvider; (4) record codecs (F, FB, V, VB, U, binary) as independent components; (5) UUID-based physical object layout; (6) staged transaction protocol for cross-resource consistency; (7) VSAM KSDS/RRDS/ESDS and ISAM support; (8) integrity manifests, workspace backup/restore; (9) security path-traversal guards and audit trail; (10) POSIX files remain native with no SQLite BLOB storage.
- **Status**: IN PROGRESS
- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirements 16-30), `docs/specs/virtual-file-system/requirements.md` (new Requirements 9-12)
