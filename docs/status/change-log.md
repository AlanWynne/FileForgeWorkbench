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
- **Description**: Add a steering rule that classifies every user prompt as a bug, new requirement, change request, question, task, or refactor. Bugs are logged to `docs/status/bugs.md`; new requirements and change requests are logged to `docs/status/change-log.md`.
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

### CR-NR-017 -- Catalog Location Discriminant (local vs remote catalog transport)
- **Date/Phase**: Phase BV
- **Prompt**: "Proceed with the requirements gate" (following architectural analysis of CatalogMount having no location/transport discriminant)
- **Description**: Add a `CatalogLocation` enum to `CatalogMount` in `ff-dscatalog` so that each mounted catalog declares whether its database and repository are on the local filesystem (`Local { path }`) or accessed via a registered VFS connector (`Remote { scheme, uri }`). Only `Local` is implemented today; `Remote` parses and stores but returns `UnsupportedOperation` until a connector implements it. The TOML `[[catalog.mounted_catalogs]]` schema gains a `location` discriminant field. This keeps the remote-catalog door open without building speculative network code.
- **Status**: IN PROGRESS
- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirement 31)

### CR-NR-016 - Mainframe Dataset Architecture and Virtual File/Dataset Storage Requirements
- **Date/Phase**: Phase BS (next)
- **Prompt**: "i have two new markdown files for this project defining how the mainframe dataset and posix catalogs should work and how mainframe files will be emulated..."
- **Description**: Two new architecture documents define: (1) record-oriented storage for PS/PDS/PDSE/GDG/VSAM/ISAM with no CRLF/LF record boundaries; (2) hybrid storage — SQLite as catalogue, native files for sequential/library content; (3) StorageProvider abstraction layer separate from VfsProvider; (4) record codecs (F, FB, V, VB, U, binary) as independent components; (5) UUID-based physical object layout; (6) staged transaction protocol for cross-resource consistency; (7) VSAM KSDS/RRDS/ESDS and ISAM support; (8) integrity manifests, workspace backup/restore; (9) security path-traversal guards and audit trail; (10) POSIX files remain native with no SQLite BLOB storage.
- **Status**: IN PROGRESS
- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirements 16-30), `docs/specs/virtual-file-system/requirements.md` (new Requirements 9-12)

### CR-NR-018 -- MiniX/FTSO Command Environment rationalisation and EARS integration
- **Date/Phase**: Phase BW (pre-gate)
- **Prompt**: "also bring into this new phase of requirements building the discussion from the document FileForgeWorkbench_MiniX_FTSO_Command_Environment_Design.md We need to decide how to integrate this into the new requirements provided"
- **Description**: The MiniX/FTSO Command Environment Design document proposes an ISPF Option 6-style command shell (FTSO) and a portable mainframe service layer (MiniX). Before any requirements.md files are updated, this proposal must be rationalised against: (1) the TSO/SDSF EARS source files which are the authoritative behavioural ground truth; (2) the existing `command-framework`, `shell-command`, `FFW-JES`, `lua-macro-engine`, and `dataset-catalog` specs which already cover significant overlap. Phase EI-0 of the EARS integration workflow governs this rationalisation. No new sub-projects are created and no requirements.md files are modified until EI-0 is complete and approved.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/ears-integration/workflow.md` Phase EI-0

### CR-CH-006 -- SQLite catalog integration for Options 1 and 2
- **Date/Phase**: Phase BU
- **Prompt**: "the latest requirements that were incorporated into the project were about the file
  catalog being hosted in SQLite... when creating catalogs, and adding datasets to them, they need
  to be updated to the SQLite catalog. Also when we go to views 1 and 2, we need to read the
  SQLite catalog to get the files"
- **Description**: Options 1 (Files Panel) and 2 (File Explorer) currently maintain an in-memory
  HashMap of AllocatedDataset entries persisted to session TOML. This must be replaced: dataset
  allocation SHALL invoke ff-dscatalog via the dataset.allocate command and write to the SQLite
  catalog.db; dataset listing in both panels SHALL query the SQLite catalog via the CatalogRegistry
  API; path resolution SHALL use the UUID-based physical_locator from the catalog, not a
  DSN-derived path. The in-memory datasets map and its TOML persistence are removed.
- **Affects**: `ff-desktop` `files_panel.rs`, `file_explorer_panel.rs`, `shell/render.rs`,
  `shell/update.rs`, `dataset_alloc_dialog.rs`, `session_manager.rs`;
  `docs/specs/virtual-catalog-manager/requirements.md` Req 13, 16;
  `docs/specs/virtual-catalog-manager/design.md` sections 7, 10
- **Status**: IN PROGRESS

### CR-NR-019 -- Phase BW: edit-operations EARS integration (CAPS, NULLS, PROFILE, SUBMIT, CREATE, REPLACE, BROWSE, VIEW, nested EDIT, COMPARE, LOCK, STATS)
- **Date/Phase**: Phase BW
- **Prompt**: "proceed to EI-5"
- **Description**: Add 11 new EARS-derived criteria to edit-operations/requirements.md covering: CAPS mode (uppercase input), NULLS mode (null character handling), PROFILE command (edit profile display/change), STATS mode (member statistics), LOCK setting (profile lock), SUBMIT primary command (submit buffer as job), CREATE primary command (create dataset from lines), REPLACE primary command (replace dataset content), nested EDIT command (open another dataset from editor), BROWSE command (open dataset for browse), VIEW command (open dataset for view), COMPARE command (compare with another dataset). Also extends 4 existing PARTIAL criteria: edit profile persistence, AUTONUM alias, NUM alias, HILITE setting.
- **Status**: DONE -- gate complete, Requirements 16-17 added, Tasks 28-39 added, TCR rows added, Phase BW added to project-master
- **Linked spec**: `docs/specs/edit-operations/requirements.md` (new Requirements 16-17)

### CR-NR-020 -- Phase BX: line-commands EARS integration (O, W, F, L, ], S)
- **Date/Phase**: Phase BX
- **Prompt**: "next?"
- **Description**: Add 6 new EARS-derived criteria to line-commands/requirements.md covering: Overlay (O/On -- overlay target lines with source content), clipboard copy (W/WW -- copy lines to system clipboard), first-of-excluded (F -- show first line of excluded block), last-of-excluded (L -- show last line of excluded block), single-column shift right (] and ]] -- equivalent to >1), and show-excluded (S -- show first line of excluded block at cursor). Also extends 1 existing PARTIAL criterion: LC-S (show/unexclude excluded line). Adds Requirement 15 with 12 criteria.
- **Status**: DONE -- gate complete, Requirement 15 added, Tasks 22-28 added, TCR rows added, Phase BX added to project-master
- **Linked spec**: `docs/specs/line-commands/requirements.md` (new Requirement 15)

### CR-NR-021 -- Phase BY: sequence-numbers EARS integration (AUTONUM and NUM aliases)
- **Date/Phase**: Phase BY
- **Prompt**: "proceed with EI-5"
- **Description**: Extends 2 existing criteria in sequence-numbers/requirements.md: AUTONUM ON/OFF added as alias for NUMBER ON/OFF (extends Req 6.7), and NUM added as alias for the NUMBER command accepting all sub-commands (extends Req 8). No new requirements section -- both are in-place extensions to existing criteria. Adds Tasks 20-22 to sequence-numbers/tasks.md.
- **Status**: DONE -- gate complete, Req 6.7a and Req 8 alias criterion added, Tasks 20-22 added, TCR rows added, Phase BY added to project-master
- **Linked spec**: `docs/specs/sequence-numbers/requirements.md` (extensions to Req 6.7 and Req 8)

### CR-NR-022 -- Phase BZ: menu-and-statusbar EARS integration (SCROLL field, fastpath, split screen, LOCATE)
- **Date/Phase**: Phase BZ
- **Prompt**: "proceed with EI-5"
- **Description**: Adds 10 new EARS-derived criteria and extends 4 existing partial criteria in menu-and-statusbar/requirements.md as Requirement 19: SCROLL ===> field adjacent to Command ===> (ISPF-1.6, TSO-4.3), fastpath dotted notation (ISPF-2.3), data entry panel layout (ISPF-1.2), list panel layout (ISPF-1.3), list panel LOCATE nearest/partial (ISPF-4.1/4.2/4.3), extended scroll amounts HALF/CSR/MAX/DATA (TSO-4.2), and split screen PF2/PF9/PF3 (ISPF-3.1/3.2/3.3/3.4). Adds Tasks 24-30.
- **Status**: DONE -- gate complete, Requirement 19 added, Tasks 24-30 added, TCR rows added, Phase BZ added to project-master
- **Linked spec**: `docs/specs/menu-and-statusbar/requirements.md` (new Requirement 19)

### CR-NR-023 -- Phase CA: startup-and-session EARS integration (LOGOFF, TIME, STATUS, session timestamps)
- **Date/Phase**: Phase CA
- **Prompt**: "proceed with EI-5"
- **Description**: Adds 5 new EARS-derived criteria and extends 1 existing partial criterion in startup-and-session/requirements.md as Requirement 20: session start timestamp in status bar (TSO-1.2), session end timestamp and logoff message (TSO-1.3), LOGOFF command as exit alias (TSO-1.4), TIME command displaying current date/time/day-of-year (TSO-2.4), STATUS command routing to FFW-JES panel with optional jobname filter (TSO-2.5). Adds Tasks 28-33.
- **Status**: DONE -- gate complete, Requirement 20 added, Tasks 28-33 added, TCR rows added, Phase CA added to project-master
- **Linked spec**: `docs/specs/startup-and-session/requirements.md` (new Requirement 20)

### CR-NR-024 -- Phase CB: command-semantics EARS integration (TSO commands, FTSO operand parsing)
- **Date/Phase**: Phase CB
- **Prompt**: "proceed with EI-5"
- **Description**: Adds 17 new EARS-derived criteria and extends 1 existing partial criterion in command-semantics/requirements.md as Requirement 9: TSO dataset commands (ALLOCATE, FREE, DELETE, RENAME, LISTCAT, LISTDS, LISTALC), TSO job commands (SUBMIT, STATUS), EDIT routing extension (TSO-EDIT-1), FTSO operand parsing (positional + keyword), session prefix (SET PREFIX), command continuation (trailing backslash), ds:// URI scheme, namespace conflict resolution, capability model, secret operand handling, and structured audit events. Adds Tasks 19-24.
- **Status**: DONE -- gate complete, Requirement 9 added, Tasks 19-24 added, TCR rows added, Phase CB added to project-master
- **Linked spec**: `docs/specs/command-semantics/requirements.md` (new Requirement 9)

### CR-NR-025 -- Phase CC: FFW-JES P1 core EARS integration (SDSF panel framework)
- **Date/Phase**: Phase CC
- **Prompt**: "next?"
- **Description**: Adds 20 new EARS-derived criteria and extends 6 existing partial criteria in FFW-JES/requirements.md as Requirement 16: SDSF panel framework core covering action bar (SDSF-1.1), title line with row range (SDSF-1.2), SCROLL field (SDSF-1.5), filter information lines (SDSF-1.6), NP column (SDSF-1.7), fixed first column (SDSF-1.8), action character system S/?/C/H/A/P/D/E/J/W (SDSF-2.1/2.2), = repeat (SDSF-2.3), // block action (SDSF-2.4), command-line action syntax (SDSF-2.5), SET ROWNUM (SDSF-2.6), main panel (SDSF-4.1 through 4.6), PREFIX/OWNER/DEST filter commands (SDSF-FILTER-1/2/3), message area (SDSF-1.3), COMMAND INPUT field (SDSF-1.4), full column set (SDSF-JQ-6), filter input rows (SDSF-JQ-7), SORT command (SDSF-FILTER-5). Adds Tasks 20-25.
- **Status**: DONE -- gate complete, Requirement 16 added, Tasks 20-25 added, TCR rows added, Phase CC added to project-master
- **Linked spec**: `docs/specs/FFW-JES/requirements.md` (new Requirement 16)

### CR-NR-026 -- Phase CD: FFW-JES P1 extended EARS integration (ST panel, FILTER/FIND/LOCATE, SET commands)
- **Date/Phase**: Phase CD
- **Prompt**: "proceed with cd"
- **Description**: Adds 14 new EARS-derived criteria and extends 3 existing partial criteria in FFW-JES/requirements.md as Requirement 17: ST panel showing all jobs (SDSF-JQ-4), FILTER command with field comparisons/AND/OR/wildcard (SDSF-FILTER-4), FIND command with NEXT/PREV/case options (SDSF-FILTER-6), LOCATE command with nearest-alpha fallback (SDSF-FILTER-7), SDSF scroll commands UP/DOWN/LEFT/RIGHT with n/HALF/PAGE/MAX (SDSF-SCROLL-1-5), SET ACTION (SET-1), SET MAIN (SET-8), SET ROWNUM (SET-9), WHO command (SET-12), QUERY AUTH command (SET-13), and SET settings persistence (PERSIST-1). Adds Tasks 26-29.
- **Status**: DONE -- gate complete, Requirement 17 added, Tasks 26-29 added, TCR rows added, Phase CD added to project-master
- **Linked spec**: `docs/specs/FFW-JES/requirements.md` (new Requirement 17)

### CR-NR-027 -- Phase CE: undo-redo-transactions P2 EARS integration (SETUNDO, RECOVERY commands)
- **Date/Phase**: Phase CE
- **Prompt**: "proceed with ce"
- **Description**: Adds 1 new EARS-derived criterion and extends 1 existing partial criterion in undo-redo-transactions/requirements.md as Requirement 19: SETUNDO primary command with ON/OFF/n operands for configuring undo levels at runtime (RU-SETUNDO), and RECOVERY primary command with ON/OFF/n operands for configuring crash recovery interval at runtime (RU-RECOVERY-command, extends Requirement 8.2). Adds Tasks 19-20.
- **Status**: DONE -- gate complete, Requirement 19 added, Tasks 19-20 added, TCR rows added, Phase CE added to project-master
- **Linked spec**: `docs/specs/undo-redo-transactions/requirements.md` (new Requirement 19)

### CR-NR-028 -- Phase CF: syntax-highlighting P2 EARS integration (HILITE command)
- **Date/Phase**: Phase CF
- **Prompt**: "proceed with cf"
- **Description**: Adds 3 new EARS-derived criteria and extends 2 existing partial criteria in syntax-highlighting/requirements.md as Requirement 16: HILITE ON/OFF command toggling syntax highlighting per document (SH-HILITE-toggle / PC-HILITE unified), HILITE LOGIC mode highlighting boolean and comparison operators (SH-HILITE-LOGIC), HILITE PAREN mode highlighting enclosing delimiter pairs with error style for mismatches (SH-HILITE-PAREN), HILITE FIND persisting find-match highlights (SH-HILITE-FIND), and combined operand support. Adds Tasks 21-22.
- **Status**: DONE -- gate complete, Requirement 16 added, Tasks 21-22 added, TCR rows added, Phase CF added to project-master
- **Linked spec**: `docs/specs/syntax-highlighting/requirements.md` (new Requirement 16)

### CR-NR-029 -- Phase CG: lua-macro-engine P2 EARS integration (ISREDIT, ISPEXEC, IMACRO, REXX bridge, FFCMD)
- **Date/Phase**: Phase CG
- **Prompt**: "proceed with cg"
- **Description**: Adds 30 new EARS-derived criteria to lua-macro-engine/requirements.md as Requirement 11: ISREDIT host command environment (AC 11.1), ISPEXEC host command environment (AC 11.2), IMACRO initial macro execution and edit profile setting (AC 11.3-11.4), LINENUM function (AC 11.5), CURSOR get/set extension (AC 11.6), REXX exec invocation via EXEC command/implicit/% prefix/argument passing (AC 11.7-11.10), TSO host environment and ADDRESS switching with ISPEXEC/ISREDIT environments and RC variable (AC 11.11-11.15), REXX built-in functions LISTDSI/MSG/MVSVAR/OUTTRAP/PROMPT/SYSDSN/SYSVAR/USERID (AC 11.16-11.23), EXECIO DISKR/DISKW/FINIS/SKIP with return codes (AC 11.24-11.28), and FFCMD command files with transaction wrapping (AC 11.29-11.30). Adds Tasks 21-24.
- **Status**: DONE -- gate complete, Requirement 11 added, Tasks 21-24 added, TCR rows added, Phase CG added to project-master
- **Linked spec**: `docs/specs/lua-macro-engine/requirements.md` (new Requirement 11)

### CR-NR-030 -- Phase CH: FFW-JES P2 EARS integration (overtype, help, log/system panels, browse/print, SET P2)
- **Date/Phase**: Phase CH
- **Prompt**: "proceed with ch"
- **Description**: Adds 30 new EARS-derived criteria to FFW-JES/requirements.md as Requirement 18: overtype fields with visual distinction, direct overtype, command-line syntax, and extension pop-up (AC 18.1-18.4); context-sensitive help system HELP/ACTH/COLH/CMDH/SEARCH (AC 18.5-18.9); log panels LOG/ULOG with NEXT/PREV/SNAPSHOT (AC 18.10-18.13); system panels SYS/DASH/INIT/JC/SP (AC 18.14-18.18); browse settings, PRINT action, COLS command (AC 18.19-18.21); SET P2 commands BCOLOR/CONFIRM/CURSOR/DATE/DELAY/HEX/SCHARS/SCREEN with persistence (AC 18.22-18.30). Adds Tasks 30-34.
- **Status**: DONE -- gate complete, Requirement 18 added, Tasks 30-34 added, TCR rows added, Phase CH added to project-master
- **Linked spec**: `docs/specs/FFW-JES/requirements.md` (new Requirement 18)

### CR-NR-031 -- Phase CI: command-semantics P2 EARS integration (OUTPUT, CANCEL, SEND, PROFILE, PRINTDS)
- **Date/Phase**: Phase CI
- **Prompt**: "Proceed with phase CI"
- **Description**: Adds 5 new EARS-derived criteria to command-semantics/requirements.md as Requirement 10: OUTPUT command routing to FFW-JES for job output display (TSO-CMD-10), CANCEL command with optional PURGE operand routing to FFW-JES (TSO-CMD-11), SEND command with USER/LOGON/BROADCAST routing to messaging subsystem (TSO-CMD-12), PROFILE command routing to session profile subsystem with MSGID/INTERCOM/NOINTERCOM/PREFIX/SIZE/WTPMSG operands (TSO-CMD-13), and PRINTDS command routing to file-operations pipeline (TSO-CMD-14). Adds Tasks 25-27. Completes EI-5.16 (final EARS integration batch).
- **Status**: DONE -- gate complete, Requirement 10 added, Tasks 25-27 added, TCR rows added, Phase CI added to project-master
- **Linked spec**: `docs/specs/command-semantics/requirements.md` (new Requirement 10)
