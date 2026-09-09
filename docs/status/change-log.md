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
- **Status**: DONE -- superseded and implemented by Phase AS (File Explorer Panel)
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

### CR-CH-016 -- END/RETURN from POM close the Workspace, not the application
- **Date/Phase**: Phase DE-fix
- **Prompt**: "wHEN END IS ISSUED FROM THE pom, THAT pom WORKSPACE IS TERMINATED, IF IT IS THE ONLY WORKSSPACE OPEN THEN IT TERMINATE THE WHOLE APPLICATION"
- **Description**: Revised function-keys-and-history Requirement 17.2/17.4. END (and RETURN) issued from a POM tab now close only that POM Workspace and navigate to another open Workspace; the application terminates only when the POM is the last Workspace open (new criterion 17.2a). Requirement 4.3 aligned with 13.2 (blank slot never omitted), and the default Excluded_Command set (Req 8.2 + glossary) updated to include END and RETURN.
- **Affects**: `ff-desktop` `shell/commands.rs` (END/RETURN handlers), `docs/specs/function-keys-and-history/requirements.md` (Req 17.2, 17.2a, 17.4, 4.3, 8.2, glossary)
- **Status**: PENDING GATE

### CR-CH-001 â€” POM option 2 description update
- **Date/Phase**: Phase AS
- **Prompt**: "opetion 2 needs to be a file Exploere..."
- **Description**: POM option 2 label updated from "View Edit Create and Delete of files" to "File Explorer â€” Browse catalogs and files in a tree view".
- **Affects**: `ff-desktop` `primary_option_menu.rs`, `startup-and-session/requirements.md`
- **Status**: DONE -- implemented by Phase AS/AC (POM option 2 relabelled)

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
- **Status**: DONE -- implemented by Phase BB (sorted listing with file attributes)
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

### CR-NR-015 -- Requirements Review and Modernisation
- **Date/Phase**: Phase BQ
- **Prompt**: "You are acting as a Senior Product Architect, Requirements Engineer, UX Architect, and Software Platform Designer... Perform a comprehensive review of the supplied requirements..."
- **Description**: Comprehensive review of all 65 sub-project specifications. Deliverables: inventory, terminology map, domain classification, gap analysis, rewritten requirements catalogue, traceability matrix, consolidation report, and executive assessment. Work is broken into 10 tasks tracked under `docs/reviews/requirements-review/`.
- **Status**: DONE -- Phase BQ complete, all 10 tasks done, 8 artefacts delivered
- **Linked spec**: `docs/reviews/requirements-review/` (all 10 output files complete)

### CR-NR-015 status update -- Tasks 1-4 complete
- Tasks 1 (Inventory), 2 (Terminology), 3 (Domain Classification), 4 (Gap Analysis) are DONE.
- Output files: `docs/reviews/requirements-review/inventory.md`, `terminology-map.md`, `domain-classification.md`, `gap-analysis.md`
- Tasks 5-10 pending.

### CR-NR-015 status update -- Tasks 9-10 complete -- Phase BQ DONE
- Tasks 9 (Consolidation Report) and 10 (Executive Assessment) are DONE.
- Output files: `docs/reviews/requirements-review/consolidation-report.md`, `executive-assessment.md`
- **Status**: DONE -- All 10 tasks complete. 497 tests passing. 8 artefacts delivered.

### CR-NR-017 -- Catalog Location Discriminant (local vs remote catalog transport)
- **Date/Phase**: Phase BV
- **Prompt**: "Proceed with the requirements gate" (following architectural analysis of CatalogMount having no location/transport discriminant)
- **Description**: Add a `CatalogLocation` enum to `CatalogMount` in `ff-dscatalog` so that each mounted catalog declares whether its database and repository are on the local filesystem (`Local { path }`) or accessed via a registered VFS connector (`Remote { scheme, uri }`). Only `Local` is implemented today; `Remote` parses and stores but returns `UnsupportedOperation` until a connector implements it. The TOML `[[catalog.mounted_catalogs]]` schema gains a `location` discriminant field. This keeps the remote-catalog door open without building speculative network code.
- **Status**: DONE -- Phase BV complete, CatalogLocation enum added to ff-dscatalog
- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirement 31)

### CR-NR-016 - Mainframe Dataset Architecture and Virtual File/Dataset Storage Requirements
- **Date/Phase**: Phase BS (next)
- **Prompt**: "i have two new markdown files for this project defining how the mainframe dataset and posix catalogs should work and how mainframe files will be emulated..."
- **Description**: Two new architecture documents define: (1) record-oriented storage for PS/PDS/PDSE/GDG/VSAM/ISAM with no CRLF/LF record boundaries; (2) hybrid storage -- SQLite as catalogue, native files for sequential/library content; (3) StorageProvider abstraction layer separate from VfsProvider; (4) record codecs (F, FB, V, VB, U, binary) as independent components; (5) UUID-based physical object layout; (6) staged transaction protocol for cross-resource consistency; (7) VSAM KSDS/RRDS/ESDS and ISAM support; (8) integrity manifests, workspace backup/restore; (9) security path-traversal guards and audit trail; (10) POSIX files remain native with no SQLite BLOB storage.
- **Status**: DONE -- Phase BS complete (BS.1-BS.15), all 15 deliverables implemented and tested
- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirements 16-30), `docs/specs/virtual-file-system/requirements.md` (new Requirements 9-12)

### CR-NR-018 -- MiniX/FTSO Command Environment rationalisation and EARS integration
- **Date/Phase**: Phase BW (pre-gate)
- **Prompt**: "also bring into this new phase of requirements building the discussion from the document FileForgeWorkbench_MiniX_FTSO_Command_Environment_Design.md We need to decide how to integrate this into the new requirements provided"
- **Description**: The MiniX/FTSO Command Environment Design document proposes an ISPF Option 6-style command shell (FTSO) and a portable mainframe service layer (MiniX). Before any requirements.md files are updated, this proposal must be rationalised against: (1) the TSO/SDSF EARS source files which are the authoritative behavioural ground truth; (2) the existing `command-framework`, `shell-command`, `FFW-JES`, `lua-macro-engine`, and `dataset-catalog` specs which already cover significant overlap. Phase EI-0 of the EARS integration workflow governs this rationalisation. No new sub-projects are created and no requirements.md files are modified until EI-0 is complete and approved.
- **Status**: DONE -- EI-0 through EI-6 all complete; all 16 EI-5 batches executed as Phases BW-CI; FTSO resolved as extension to shell-command (no new sub-project); MiniX confirmed as internal architecture label only
- **Linked spec**: `docs/specs/ears-integration/workflow.md` (all phases [x])

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
- **Status**: DONE -- Phase BU complete (BU.1-BU.9), SQLite integration live

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

### CR-NR-032 -- Bootstrap Scripts for new contributors
- **Date/Phase**: Phase CJ
- **Prompt**: "I want to make it easy for somebody who wants to download and build the FileForgeWorkbench project to do so. I want to provide them with a set of scripts to run either in Windows, Linux or Macintosh"
- **Description**: Add a `bootstrap/` folder at the repository root containing three platform-specific scripts (Windows PowerShell, Linux bash, macOS bash) that download and install the Rust stable toolchain into a user-level location (`C:\tools\rust` on Windows, `~/.tools/rust` on Unix) without requiring admin rights, then verify the build. A README guides the user from `git clone` to `cargo build`.
- **Status**: DONE -- Phase CJ complete, bootstrap/ scripts for Windows/Linux/macOS
- **Linked spec**: `docs/specs/bootstrap-scripts/requirements.md` (new sub-project)

### CR-NR-033 -- FFTest Automated Dialog Testing Framework
- **Date/Phase**: Phase CK (pre-gate)
- **Prompt**: "i have created a new requirement for this project the requirement is discussed in: FileForgeWorkbench-Automated-Dialog-Testing-Framework.md Examine this file with a view to incorporate it into the FileForgeWorkbench developement."
- **Description**: Introduce a native FFTest Automated Dialog Testing Framework covering 25 EARS requirements (FFTEST-001 to FFTEST-025). The framework provides: (1) a human-readable FFTest scripting language for dialog automation; (2) stable automation identifiers on every UI control; (3) recording and playback of user interactions; (4) headless execution for CI/CD pipelines; (5) HTML and JSON test reports; (6) visual regression screenshot comparison; (7) plugin dialog testing support; (8) command-layer testing without loading the GUI; (9) cross-platform execution on Windows, Linux, and macOS. The requirement also mandates that not less than 90% of business logic testing can be performed without the GUI loaded. This is a new sub-project `automated-dialog-testing` and a new crate `ff-fftest`. The work is broken into 4 phases: Phase CK-1 (requirements and design gate), Phase CK-2 (automation ID infrastructure), Phase CK-3 (FFTest script engine), Phase CK-4 (headless runner and reporting).
- **Status**: DONE -- Phase CK complete (CK.1-CK.4), ff-fftest crate wired, 429 tests passing
- **Linked spec**: `docs/specs/automated-dialog-testing/requirements.md` (new sub-project)

### CR-CH-007 -- POM guaranteed on startup even when session has no POM tab
- **Date/Phase**: Phase CL
- **Prompt**: "Regarding bug B0001, i think there might be a conflict in the requirements somewhere... if it was shut down without a POM, it should always open with a pom and all other windows that where open"
- **Description**: Amend Requirement 14.1 in startup-and-session/requirements.md to add a third case: when a saved session exists but contains no POM tab, the workbench SHALL restore all saved tabs AND prepend a new POM tab at index 0. The existing two cases (first launch = single POM tab; session with POM = restore exactly) are unchanged. This resolves B001 without discarding the user's other open tabs.
- **Affects**: `ff-desktop` `shell/update.rs` (startup block); `startup-and-session/requirements.md` Req 14.1; `startup-and-session/tasks.md`; `docs/quality/TCR.md`
- **Status**: DONE -- Phase CL complete, 589 tests passing, B001 FIXED

### CR-NR-031 -- Phase CI: command-semantics P2 EARS integration (OUTPUT, CANCEL, SEND, PROFILE, PRINTDS)
- **Date/Phase**: Phase CI
- **Prompt**: "Proceed with phase CI"
- **Description**: Adds 5 new EARS-derived criteria to command-semantics/requirements.md as Requirement 10: OUTPUT command routing to FFW-JES for job output display (TSO-CMD-10), CANCEL command with optional PURGE operand routing to FFW-JES (TSO-CMD-11), SEND command with USER/LOGON/BROADCAST routing to messaging subsystem (TSO-CMD-12), PROFILE command routing to session profile subsystem with MSGID/INTERCOM/NOINTERCOM/PREFIX/SIZE/WTPMSG operands (TSO-CMD-13), and PRINTDS command routing to file-operations pipeline (TSO-CMD-14). Adds Tasks 25-27. Completes EI-5.16 (final EARS integration batch).
- **Status**: DONE -- gate complete, Requirement 10 added, Tasks 25-27 added, TCR rows added, Phase CI added to project-master
- **Linked spec**: `docs/specs/command-semantics/requirements.md` (new Requirement 10)

### CR-NR-034 -- Mouse Text Selection and Clipboard Copy in Editor Canvas and Read-Only Panels
- **Date/Phase**: Phase CM (pre-gate)
- **Prompt**: "In ispf i can at any time select any text in a the window and copy the text to paste into other tools like word or notepad... if it is i would like the same functionality availble in FFWB.. Currently it is not possible in FFWB"
- **Description**: Add mouse-driven text selection (click-drag) and Ctrl+C copy to the OS clipboard in the FFWB editor canvas. The editor currently renders text via custom painter calls with no egui selection machinery, so a custom selection layer must be added: track mouse-down/drag to (line, col) coordinates, render a highlight rect behind selected text, and on Ctrl+C extract the selected text and write it to the OS clipboard via ff-clipboard. As a secondary deliverable, read-only panels (POM, Settings, status bar) shall use egui selectable labels so their text can be selected and copied with zero custom code.
- **Status**: DONE -- Phase CM complete (CM.1 editor drag-select + Ctrl+C, CM.2 selectable labels in POM/Settings/status bar)
- **Linked spec**: `docs/specs/caret-and-selection/requirements.md` (new Requirement 13), `docs/specs/clipboard-operations/requirements.md` (new Requirement 20)

### CR-NR-035 -- Editor SCROLL field wired to editor Page Up/Down behaviour
- **Date/Phase**: Phase CN (pre-gate)
- **Prompt**: "The editor window in ispf looks something like: [diagram showing SCROLL ===> CSR right-aligned on the command line]. Scroll can be set to CSR or PAGE or a numeric value. This controls how paging in the editor works. FFWB does not have this? We need to add this"
- **Description**: The SCROLL ===> field already exists in the shell command area (Phase BZ, Req 19.1-19.3) and persists a ScrollAmount value. However the editor panel Page Up/Down keys currently always scroll by a fixed visible_count (one full page). The editor must read the active ScrollAmount and apply it: PAGE = full visible_count, HALF = visible_count/2, CSR = scroll to cursor line, a numeric value N = scroll exactly N lines. The SCROLL field must also be visible and editable when an editor tab is active.
- **Status**: DONE -- Phase CN complete (CN.1), scroll_by_amount helper wired, all ScrollAmount variants handled, 5 unit tests, all workspace tests pass
- **Linked spec**: `docs/specs/viewport-and-scrolling/requirements.md` (new criterion), `docs/specs/menu-and-statusbar/requirements.md` Req 19.1-19.3 (field display already covered)

### CR-CH-008 -- Phase BR Requirements Maintenance
- **Date/Phase**: Phase BR
- **Prompt**: "Proceed with BR"
- **Description**: Requirements maintenance sprint completing the five actions recommended by the Phase BQ executive assessment: (1) CA-01 -- fix compiler-toolchain-integration/tasks.md requirement annotations from old Req 15.x/16.x/17.x/18.x numbering to correct Req 1.x/2.x/3.x/4.x; (2) CA-02 -- add Requirement 5 (Generic ToolchainPlugin Extension Point, FR-0971) to compiler-toolchain-integration/requirements.md and tasks.md; (3) rename docs/specs/FFW-JES/ to docs/specs/jes-emulator/ and update all references; (4) mark B009 SUPERSEDED in bugs.md; (5) update CR-NR-035 to DONE in change-log.md.
- **Affects**: `docs/specs/compiler-toolchain-integration/tasks.md`, `docs/specs/compiler-toolchain-integration/requirements.md`, `docs/specs/jes-emulator/` (renamed from FFW-JES), `.amazonq/rules/specs.md`, `docs/status/bugs.md`, `docs/status/change-log.md`, `docs/specs/project-master/tasks.md`
- **Status**: DONE

### CR-NR-036 -- Workspace Model
- **Date/Phase**: Phase BS
- **Prompt**: "proceed with phase BS"
- **Description**: Introduce a Workspace Model: a named, persistable grouping of root directories
  with workspace-scoped settings, a per-workspace MRU list, and a `.ffwb-workspace` TOML file
  format. Implements WORKSPACE OPEN/SAVE/SAVE AS/CLOSE commands, root management, configuration
  layer injection, and session persistence. Foundational prerequisite for Command Palette and
  Global Search scoping.
- **Status**: DONE -- Phase BS-A complete
- **Linked spec**: `docs/specs/workspace-model/requirements.md` (new sub-project, Req 1-6)

### CR-NR-037 -- Command Palette
- **Date/Phase**: Phase BS
- **Prompt**: "proceed with phase BS"
- **Description**: Add a Command Palette (Ctrl+Shift+P) -- a modal fuzzy-search overlay over all
  registered commands. Displays command name, category, description, and bound shortcut. Executes
  commands via the existing Command_Dispatch. Persists recent commands in session state.
- **Status**: DONE -- Phase BS-B complete
- **Linked spec**: `docs/specs/command-palette/requirements.md` (new sub-project, Req 1-5)

### CR-NR-038 -- Global Search (Cross-File Search and Replace)
- **Date/Phase**: Phase BS
- **Prompt**: "proceed with phase BS"
- **Description**: Add Global Search (Ctrl+Shift+F): cross-file search and replace across all
  workspace roots or mounted Native catalogs. New `ff-global-search` crate reuses FindEngine for
  per-file matching. Results streamed to a Search Results panel (TabKind::SearchResults). Cross-file
  replace with preview and per-file undo support.
- **Status**: DONE -- Phase BS-C complete
- **Linked spec**: `docs/specs/global-search/requirements.md` (new sub-project, Req 1-6)

### CR-NR-039 -- Phase BT: Cross-File Search and Replace
- **Date/Phase**: Phase BT
- **Prompt**: "proceed with Phase BT"
- **Description**: Implement the cross-file replace pipeline (GlobalReplaceEngine::replace_all(),
  Replace_Preview confirmation, Replace All with ff-bgio dispatch, per-file undo, unsaved-changes
  guard, regex group substitution) and search history (last 20 queries persisted in session state,
  dropdown on search field, options round-trip). All requirements already exist in
  global-search/requirements.md Req 5.1-5.7 and Req 6.1-6.3.
- **Status**: DONE -- Phase BT complete, 657 tests passing (646 ff-desktop + 11 ff-global-search), 0 failures
- **Linked spec**: `docs/specs/global-search/requirements.md` (Req 5, Req 6)

### CR-NR-041 -- Batch Command Execution (IKJEFT01 analogue)
- **Date/Phase**: Phase CP (pre-gate)
- **Prompt**: "Create a formal Requirement to provide this functionality?" (following discussion of IKJEFT01 batch execution -- feeding a file of TSO commands to FFWB for non-interactive execution)
- **Description**: Add a headless batch execution mode to FFWB analogous to z/OS IKJEFT01 batch. The user supplies a file (or stdin) containing FFWB/FTSO primary commands; FFWB executes them sequentially without opening a GUI window, writes output to stdout or a nominated file, and exits with a meaningful return code. This enables scripted automation, CI/CD pipelines, and JCL-style job submission from outside the workbench. The feature spans ff-desktop (CLI entry point), ff-command-semantics (command pipeline), ff-shell (output capture), and ff-workflow (sequencing). A new sub-project `batch-execution` is created.
- **Status**: DONE
- **Linked spec**: `docs/specs/batch-execution/requirements.md` (new sub-project)

### CR-NR-040 -- Phase CO: Accessibility, Plugin Manager UI, and Notification System
- **Date/Phase**: Phase CO
- **Prompt**: "proceed with Phase BU"
- **Description**: Implements the three highest-priority remaining gaps from the Phase BQ executive
  assessment roadmap (originally labelled "Phase BU" in that document; letter CO is the next
  available). Deliverables: (1) `accessibility` sub-project -- cross-cutting WCAG AA compliance,
  keyboard-only operation, screen reader support, and focus indicators across all panels;
  (2) `plugin-manager-ui` sub-project -- Plugin Manager panel (POM option 8) for listing,
  enabling, disabling, and configuring installed plugins; (3) `notification-system` sub-project --
  non-modal notification toasts and a structured event log replacing ad-hoc status bar messages
  for multi-step operations.
- **Status**: DONE
- **Linked spec**: `docs/specs/accessibility/requirements.md` (new sub-project),
  `docs/specs/plugin-manager-ui/requirements.md` (new sub-project),
  `docs/specs/notification-system/requirements.md` (new sub-project)

### CR-NR-042 -- Phase CQ: Enterprise Features (audit logging, settings export/import, locked config keys)
- **Date/Phase**: Phase CQ
- **Prompt**: "Proceed with CQ"
- **Description**: Adds three enterprise-grade capabilities to the configuration system: (1) structured audit logging -- every configuration change is recorded with timestamp, key, old value, new value, actor, and layer, queryable via an AuditLog API and persisted to a rolling log file; (2) settings export/import -- the user can export the current effective configuration (or a specific layer) to a portable TOML file and import a previously exported file to restore settings; (3) locked config keys -- an administrator can mark specific keys as locked in the system layer, preventing user/profile/project layers from overriding them, with a clear error when a locked key is written. New sub-project: none (extends configuration-system). New requirements: Req 16 (audit logging), Req 17 (settings export/import), Req 18 (locked config keys) in configuration-system/requirements.md.
- **Status**: DONE
- **Linked spec**: `docs/specs/configuration-system/requirements.md` (new Requirements 16-18)

### CR-NR-043 -- Phase CR: OS Theme Follow + Macro Library Management
- **Date/Phase**: Phase CR
- **Prompt**: "Proceed with Phase CR"
- **Description**: Adds two medium-priority gap features: (1) OS dark/light mode follow -- the workbench detects the OS dark/light preference via egui and automatically switches the active Visual_Mode when theme.follow_os is enabled; (2) Macro Library Management panel -- POM option 6 opens a panel listing all discovered Lua scripts with Run, Edit, Delete, and filter capabilities.
- **Status**: DONE
- **Linked spec**: `docs/specs/theme-and-appearance/requirements.md` (new Req 16), `docs/specs/lua-macro-engine/requirements.md` (new Req 12)

### CR-CH-009 -- Phase CT: Workbench/Workspace/Context Terminology Standardisation
- **Date/Phase**: Phase CT
- **Prompt**: "Do a review of all the Specifications and apply this terminology consistently across all specifications"
- **Description**: Adopt a consistent three-level UI terminology model across all documentation: (1) Workbench -- the application window as a whole; (2) Workspace -- a single tab in the tab bar (the unit of work the user switches between); (3) Context -- the type of content/function active in a Workspace (e.g. Home Context, Editor Context, Settings Context, Explorer Context). Update the terminology map, the architecture brief, the README, and all 69 sub-project specifications in priority order. No source code changes -- documentation only.
- **Affects**: `docs/reviews/requirements-review/terminology-map.md`, `docs/specs/workbench-requirements-merge/architecture-brief.md`, `README.md`, `.amazonq/rules/`, all `docs/specs/*/requirements.md` and `design.md` files
- **Status**: DONE -- Phase CT complete, all 7 tasks done, ~200 terminology replacements across 69 sub-project specs

### CR-NR-044 -- Phase CS: Test Warning Cleanup
- **Date/Phase**: Phase CS
- **Prompt**: "Create a dedicated cleanup task to be done next"
- **Description**: Eliminate all compiler warnings emitted during `cargo test --workspace`. Warnings are exclusively in test code (unused imports, unused variables, unnecessary mut, unused doc comments, dead code in test helpers). All are auto-fixable via `cargo fix` or trivial manual edits. No behaviour change. REFACTOR -- no requirements gate required.
- **Status**: DONE -- Phase CS complete, zero warnings in cargo test --workspace
- **Linked spec**: N/A (refactor -- no new requirements)

### CR-NR-045 -- Menu Workspace Pattern (Configurable ISPF-style option menus)
- **Date/Phase**: Phase CU (pre-gate)
- **Prompt**: "One of the features of ISPF is the ability to Customize it... we need a Solution where we can create menu workspaces that are a list of options. Each option is a number or set of characters (up to 4) Followed by a Command, and a description."
- **Description**: Introduce a Menu Workspace as a first-class pattern: a Workspace whose Context is a list of options loaded from a TOML config file (`menus/<name>.toml`). Each option has a key (1-4 chars), a command string, and a description. The POM becomes an instance of this pattern. Chained option paths (e.g. `=0.Themes`) are supported. A default `menus/pom.toml` is written on first launch. Hot-reload on file change. New sub-project `menu-workspace`.
- **Status**: DONE -- Phase CU complete (CU.1-CU.6), all 6 spec tasks done
- **Linked spec**: `docs/specs/menu-workspace/requirements.md` (created Phase CU)

### CR-NR-046 -- Named Workspaces and Per-Workspace KEYS Command
- **Date/Phase**: Phase CX (pre-gate)
- **Prompt**: "Each Workspace should have its own name to allow menu option mapping as well as Function key mapping. A workspaces Function keys can be mapped at any time by invoking the KEYS command in the workspace."
- **Description**: Each Workspace gains a user-visible name string. The KEYS command is extended to accept an optional name argument (`KEYS <name>`) so the user can open the Key Configuration Dialog pre-loaded with any named key map, not just the current Workspace's map. The Key Configuration Dialog gains a `Map Name` field the user can change mid-session.
- **Status**: DONE
- **Linked spec**: `docs/specs/function-keys-and-history/requirements.md` (extension to Req 20), `docs/specs/function-keys-and-history/cx-requirements.md` (new)

### CR-CH-010 -- SPLIT Command Alias for Workspace Detach
- **Date/Phase**: Phase CX (pre-gate)
- **Prompt**: "A Workspace should be detachable into its own Window... This serves as a replacement to the ISPF split command."
- **Description**: Add a `SPLIT` primary command as an ISPF-heritage alias for the existing Workspace detach operation (Ctrl+Shift+T / Move to Other View). Registers as Command_ID `layout.split`. Adds an explicit note in `layout-and-docking` that this replaces ISPF split-screen with modern OS window management.
- **Affects**: `docs/specs/layout-and-docking/requirements.md` Req 3; `ff-desktop` command handler; `docs/specs/function-keys-and-history/cx-requirements.md` Req 3
- **Status**: DONE

### CR-CH-011 -- Persist Settings_Namespace_View filter in session (unblock Task 14.5)
- **Date/Phase**: Phase CW-impl (follow-up)
- **Prompt**: "Draft a format change proposal etc.." (re: blocked Task 14.5, session persistence of the Settings namespace filter)
- **Description**: Change the `ff-session` persisted session format so a Settings panel opened with a namespace filter is restored with that filter on next launch, unblocking menu-workspace Task 14.5 and satisfying cw-requirements.md Req 10.6 / configuration-system Req 15.9. Recommended approach (Option B): add a data-free `PersistedTabKind::SettingsPanel` variant plus a flat `settings_namespace_filter: Option<String>` field on `TabState` (backward compatible via `serde(default)`), persist the filter on save, and add a Settings-tab restore branch. Full analysis in the proposal doc.
- **Affects**: `crates/ff-session` (`session_state.rs`); `crates/ff-desktop` (`session_manager.rs`, `shell/update.rs`); `docs/specs/menu-workspace/tasks.md` Task 14.5
- **Status**: SUPERSEDED by CR-CH-012 -- the point fix is folded into the descriptor-based persistence model; Settings namespace filter becomes a CustomWorkspace param rather than a new PersistedTabKind variant
- **Linked spec**: `docs/specs/menu-workspace/cw-session-format-proposal.md` (analysis retained), `docs/specs/menu-workspace/cw-requirements.md` Req 10.6

### CR-CH-013 -- Rescope JES `MENU` to a Context-local command; global `MENU` is uniform
- **Date/Phase**: Phase DB (pre-gate, consistency reconciliation for CR-NR-051)
- **Prompt**: "Typing MENU in the command field from any context should take us back to the POM ... The MENU <name> in the Menu Configurator then just makes use of the MENU Command?"
- **Description**: The jes-emulator spec (Req 16.13, 16.17) and its implementation bind the bare word `MENU` to "return to the SDSF main panel", which collides with the new global `MENU` command (menu-workspace Requirement 11: `MENU` returns to the Home Context / POM, `MENU <name>` opens the named menu). Reconcile by making the global `MENU` uniform in every Context and rescoping the JES panel-return action to a Context-local command `SDSF` (alias `=MENU`). This is a change to already-implemented JES behaviour (a Context-local command-name change), not a change to SDSF panel navigation itself.
- **Affects**: `docs/specs/jes-emulator/requirements.md` Req 16.13 / 16.17 (annotated); `crates/ff-jes` (or the JES command handler) command-name binding, to be adjusted during DB.8 wiring
- **Status**: IN PROGRESS -- reconciliation APPROVED (menu-workspace Req 11 + JES annotations written). JES command-name rebinding is done during Phase DB implementation (DB.8 wiring).
- **Linked spec**: `docs/specs/menu-workspace/requirements.md` Requirement 11, `docs/specs/jes-emulator/requirements.md` Req 16.13 / 16.17

### CR-NR-047 -- Settings Context as a Menu Workspace
- **Date/Phase**: Phase CW (pre-gate, depends on CR-NR-045)
- **Prompt**: "The settings workspace should probably be a menu options workspace. A full list of available settings should be extracted from the requirements and a settings menu option created for it."
- **Description**: Restructure the Settings Context as a Menu Workspace whose options are loaded from `menus/settings.toml`. A default settings.toml is written on first launch with one option per major config namespace (Editor, Theme, Catalogs, VFS, Logging, Key Maps, Session, Plugins). Each option opens a sub-context showing only that namespace's keys using the existing flat-list widget. The flat-list view remains accessible as a sub-context.
- **Status**: IN PROGRESS
- **Linked spec**: `docs/specs/configuration-system/requirements.md` Req 15 (revision), `docs/specs/menu-workspace/cw-requirements.md` (new)

### CR-NR-048 -- POM Options Review and Logical Grouping
- **Date/Phase**: Phase CV (pre-gate, depends on CR-NR-045)
- **Prompt**: "We need a full review of the current options in the POM... perhaps we need to create menu options for those that can be grouped."
- **Description**: Review the current 9 POM options (0-8) against all implemented functionality. Revise the option list with logical grouping, adding entries for JES (job monitor), Search (global search), and Batch (batch execution) which currently have no POM entry. Define the default `menus/pom.toml` content. Update `startup-and-session` Req 14.3 accordingly.
- **Status**: IN PROGRESS
- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Req 14.3 (revision), `docs/specs/menu-workspace/cv-requirements.md` (new)

### CR-NR-049 -- FFTest Context Inspection and Automatic Bug Logging
- **Date/Phase**: Phase CZ (pre-gate)
- **Prompt**: "The Automated Dialog testing capability would have to be able to examine the context of a workspace to verify that what expected to happen happened and also to write the finding to a Log file / Bug report. Bugs should be logged for repair."
- **Description**: Extend the FFTest framework with: (1) Workspace context inspection assertions (`ASSERT CONTEXT IS`, `ASSERT WORKSPACE COUNT IS`, `ASSERT OPTION EXISTS`); (2) automatic bug report generation -- on assertion failure the runner appends a structured entry to `reports/bugs-from-tests.md` in a format compatible with `docs/status/bugs.md`; (3) a full FFTest script suite covering all major functional areas (POM navigation, file ops, editor, catalog management, settings, key config, compiler, plugin manager, notification system, batch, global search, command palette). New requirements Reqs 11-13 in `automated-dialog-testing/requirements.md`.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/automated-dialog-testing/requirements.md` (new Reqs 11-13)

### CR-NR-050 -- Configurable Menu Option Limits (soft warning + hard maximum)
- **Date/Phase**: Phase DA (pre-gate)
- **Prompt**: "the menu workspace has a hard limit of 9 or 12 items? Should this not be configurable? if Configurable should we have an upper limit? an infinite list is impractical, what is Reasonable?"
- **Description**: The Menu Workspace pattern places no explicit cap on option count today (the "9" and "12" figures are content of specific POM menu files, not pattern constraints). Add explicit, configurable option-count limits to the Menu Workspace pattern: a soft limit (default 64) that logs a WARN and shows an in-panel advisory suggesting sub-menus, and a hard limit (default 256) that is treated as a Menu_File load error. Both limits are configuration keys (`menu.soft_option_limit`, `menu.hard_option_limit`) resolved through the layered configuration system, so they can be tuned per user/project. Adds Requirement 9 to `menu-workspace/requirements.md` and cross-references `configuration-system`.
- **Status**: DONE -- Phase DA gate complete (Req 9 added) and DA-impl complete (Tasks 16-20); config keys registered, loader/state/render/hot-reload implemented, 15 new tests passing, TCR rows PASS
- **Linked spec**: `docs/specs/menu-workspace/requirements.md` (new Requirement 9), `docs/specs/configuration-system/requirements.md` (cross-reference note on Req 9 schema)
### CR-NR-051 -- Unified Command Target model (menu / custom-workspace / function / macro / external)
- **Date/Phase**: Phase DB (pre-gate)
- **Prompt**: "Yes a single unified command-target type (menu / custom-workspace / function / macro / external-program) as the centerpiece, with menu options and keybindings both pointing at it"
- **Description**: Introduce a single typed Command_Target abstraction at the command-framework level with five variants -- MenuWorkspace{name}, CustomWorkspace{kind,params}, Function{command_id,params}, Macro{name|path}, and External{program,args,working_dir,mode}. Menu options (menus/*.toml) and keyboard bindings both resolve to a Command_Target. A plain command string remains valid and is parsed into a Function or CustomWorkspace target for backward compatibility. This unifies today's separate, string-only binding mechanisms into one dispatchable target type.
- **Status**: IN PROGRESS -- Phase DB gate APPROVED (spec DB.1-DB.2). DB.8 DONE: CommandTarget type + resolve_target + execute_target implemented in ff-command with TargetResolver/TargetExecutor seams, TOML round-trip, and visible-workspace classification (16 tests, Req 8.1-8.4/8.7-8.9 PASS). Req 8.5 (shortcut) and 8.6 (menu) wiring remain for DB.4.
- **Linked spec**: `docs/specs/command-framework/requirements.md` Requirement 8, `docs/specs/menu-workspace/requirements.md` Requirement 10

### CR-NR-052 -- Command Configurator custom workspace and external command execution (Detached / Captured)
- **Date/Phase**: Phase DB (pre-gate, depends on CR-NR-051)
- **Prompt**: "We need to design a Custom Workspace to configure Commands, Store them somewhere ... if External again they would be Fire and Forget ... we should be prepared to capture a response message from executing the command to the os i.e. capture the stdout and stderr and display in a window"
- **Description**: Add a user-facing Command Configurator (a new Custom Workspace / Context) that lists, adds, edits, and deletes user-defined command definitions stored in a data-driven, hot-reloadable file at `<User_Data_Dir>/commands/commands.toml`. Each definition is a named Command_Target (internal or external). External commands run in one of two modes: Detached (fire-and-forget Started Task -- FFWB spawns and forgets it, no output capture, never persisted or restarted) or Captured (FFWB runs it asynchronously without blocking the UI, collects stdout/stderr and exit code, and displays them in the existing shell Output_Panel). User-defined external commands are gated by the existing `shell.mode` configuration (disabled/prompt/enabled). Introduces a new `command-configurator` sub-project.
- **Status**: IN PROGRESS -- Phase DB gate APPROVED. DB.9 data layer DONE: command_config module (CommandDefinition, CommandStore load/save/validate/duplicate-skip/hot-reload, UserCommandStore resolver view, reserved-id validation) -- Req 1.1-1.7, 4.1-4.3, 4.6 (18 tests). DEFERRED within DB.9: Context UI (Req 2 render/edit, Req 1.8 default content). External execution (Req 3) is DB.10; menu/shortcut binding (Req 4.4/4.5) is DB.4.
- **Linked spec**: `docs/specs/command-configurator/requirements.md` (new sub-project), `docs/specs/shell-command/requirements.md` Requirement 19

### CR-CH-012 -- Descriptor-based session persistence of visible Workspaces (excludes Started Tasks; unblocks Task 14.5)
- **Date/Phase**: Phase DB (pre-gate, depends on CR-NR-051)
- **Prompt**: "resolving the following we may be sorting out the above issue ... only Visible Workspaces will be re-opened as they were last left ... Started Tasks will not be re-started"
- **Description**: Replace the closed `PersistedTabKind` enum plus URI-only restore with descriptor-based persistence: each VISIBLE Workspace is persisted by a Workspace_Descriptor -- either `MenuWorkspace{name}` or `CustomWorkspace{kind, params}` (params carry small values such as a Settings namespace filter or a file URI) -- and restored by re-opening that descriptor on next launch. Started Tasks (Detached external runs) and transient captured-run output tabs are never persisted or restarted. This structurally fixes the current gap where non-file Workspaces are dropped on restore, satisfies startup-and-session Req 14.1a and Req 19.12, and unblocks menu-workspace Task 14.5 / cw-requirements.md Req 10.6 (Settings namespace filter persistence) as a special case of a CustomWorkspace param.
- **Affects**: `crates/ff-session` (session_state.rs), `crates/ff-desktop` (session_manager.rs, shell/update.rs); supersedes CR-CH-011 (Task 14.5 folded in)
- **Status**: IN PROGRESS -- Phase DB gate APPROVED (spec DB.6). DB.11 DONE: descriptor-based persistence implemented in ff-session (WorkspaceKind/WorkspaceDescriptor/DescriptorParams, TabState.descriptor, effective_descriptor legacy mapping) and ff-desktop (descriptor save helpers + restore loop, settings namespace threaded through save). Original Task 14.5 resolved. Req 21.1-21.3/21.5/21.6/21.8/21.9/21.10 PASS (15 new tests); Req 21.4 (menu restore) needs the MENU command (DB.4), Req 21.7 assertion lands with DB.10.
- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Req 21 (also 14.1a / 19.12), `docs/specs/workspace-model/requirements.md` Req 5 note, `docs/specs/menu-workspace/cw-requirements.md` Req 10.6
### CR-NR-053 -- Regina REXX interpreter integration (future planning item)
- **Date/Phase**: Phase DB (planning note; not scheduled)
- **Prompt**: "Should add a note somewhere to plan and incorporate 'Regina Rexx' into the project at some later point.."
- **Description**: Plan, at a later phase, to embed or integrate a real REXX interpreter (candidate: Regina REXX, open source, ANSI/TRL-2 compatible) so that unmodified REXX execs run under a genuine REXX language processor rather than the Lua-engine compatibility bridge specified in lua-macro-engine Requirement 11. Scope, licensing (Regina is LGPL -- to be confirmed for the project's distribution model), and the host-command-environment mapping (ADDRESS ISREDIT/ISPEXEC/TSO) would be defined when scheduled. When this work is gated, also reconcile the stale "no criterion" REXX rows in `docs/specs/ears-integration/coverage-classification.md` against the existing Requirement 11.
- **Status**: DEFERRED -- planning note only; not part of Phase DB, no gate run yet
- **Linked spec**: `docs/specs/lua-macro-engine/requirements.md` Requirement 11 (planning note added)

### CR-CH-014 -- Key Configuration Dialog redesign (F1-F12 range, PCOMM modifier layers, command picker)
- **Date/Phase**: Phase DE (pre-gate)
- **Prompt**: "as a UX designer, redesign the Key Configuration / function-keys dialog... LIMIT the physical function key range to F1-F12 (not F1-F24), while supporting modifier layers... command selection from a picker restricted to defined commands... description sourced from the command definition"
- **Description**: Redesign Requirement 20 (Key Configuration Dialog) in function-keys-and-history. Physical range is limited to F1-F12 (matching a 3270 PCOMM session, where 24 PF keys are reached via F1-F12 plus Shift+F1-F12). Each physical key gains a full set of PCOMM modifier layers: Base, SHIFT, CTRL, ALT, ALTGR, CTRL+SHIFT. The Command field becomes a picker/listbox restricted to commands already defined in the Command_Store (command-configurator Requirement 4.5), and the Description is sourced read-only from the selected command's label/description rather than free-typed. Adds an ALTGR prefix and a CTRL+SHIFT prefix to the extended key-name syntax (Req 20.11) and updates the ModifiedKey slot count (Req 20.12). Notes the effect on Requirement 14 (per-context maps) which inherits the same slot model.
- **Affects**: `docs/specs/function-keys-and-history/requirements.md` (Requirement 20, Req 14 note); later `ff-desktop` key configuration dialog and `ff-function-keys` ModifiedKey type
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/function-keys-and-history/requirements.md` Requirement 20

### CR-CH-015 -- Two-phase logging init so logging.directory takes effect (fixes B033)
- **Date/Phase**: Phase CX (pre-gate)
- **Prompt**: "the logging should go into the project's directory somewhere that you have access to ... we should not require a code change to redirect the logging we should just be able to change the toml file"
- **Description**: The desktop binary calls `ff_logging::init_default()` in startup step 1, before the configuration system loads in step 2, and never re-points the log sink at the configured directory. As a result the `logging.directory` config setting (logging-subsystem Req 4.1/4.2) is silently ignored and logs always go to the platform default. Fix with a two-phase init: keep the early `init_default()` so no log records are ever lost while config is loading, then add a new `ff_logging::reconfigure(LogConfig)` API that re-points the file sink to the configured directory once config is available, and call it from the desktop startup sequence after config load. This makes config-only log redirection genuinely work (no recompile), satisfying the user's expectation. Adds Requirement 11 (Runtime Reconfiguration) to logging-subsystem/requirements.md.
- **Affects**: `crates/ff-logging` (new `reconfigure` API + supporting sink/writer changes); `crates/ff-desktop` `main.rs` startup sequence (call reconfigure after config load, feed `logging.directory`/`logging.level`/rotation keys); `docs/specs/logging-subsystem/requirements.md` (new Requirement 11); `docs/specs/logging-subsystem/design.md`; `docs/specs/logging-subsystem/tasks.md`; `docs/quality/TCR.md`
- **Status**: DONE -- Phase DC complete. `ff_logging::reconfigure(LogConfig)` added (`ChannelMessage::Reconfigure` + `LogFileWriter::switch_directory` on the writer thread; atomic level update; WARN on failure; INFO on switch). `ff-desktop::apply_logging_config` builds LogConfig from resolved config keys and reconfigures after config load, before the GUI shell. `.ffworkbench/config.toml` redirects dev logs to `.ffworkbench/logs` (gitignored). Req 11 AC 11.1-11.10 covered; TCR rows PASS; 9008 tests pass; verify.ps1 clean. B033 FIXED.
- **Linked spec**: `docs/specs/logging-subsystem/requirements.md` (new Requirement 11), linked to bug B033

### CR-NR-054 -- Command arguments and command chaining (parameters passed to commands)
- **Date/Phase**: Phase DF (pre-gate)
- **Prompt**: "New Requirement: Passing parameters to commands. I should be able to pass parameters to commands... DOWN 8 ... DOWN M ... 8 on the command line then F8 ... MENU SETTINGS EDITOR ... =0.e ... EDITOR on the command line and press a function assigned to MENU Settings"
- **Description**: A general capability for commands to receive arguments from the command line, plus argument chaining for menu navigation. Three connected parts: (1) command-framework -- a Command_Invocation from a `Command ===>` field is parsed into a verb plus an argument string, and the argument is forwarded to the command as Command_Params (extending the existing Requirement 2.1/2.8 dispatch that already accepts Command_Params); a Command_Definition/binding may also carry a fixed argument. (2) navigation-commands -- `UP`/`DOWN` accept `<n>` (positive integer, already Req 3.2/3.4) or `M`/`MAX` (scroll to top/bottom); plus a Command_Line_Prefix_Argument: a bare integer or `M` typed in the command field is consumed as the argument to the next function-key-invoked scroll command (e.g. type `8`, press F8=DOWN -> page down 8). (3) menu-workspace -- `MENU <name> <arg...>` opens the named menu and forwards the remaining argument, which the menu resolves to an option key and immediately activates (so `MENU SETTINGS EDITOR` == `=0.E` == typing `EDITOR` and pressing a key bound to `MENU SETTINGS`), extending the existing Requirement 5 chained-navigation and Requirement 11 MENU command.
- **Status**: PENDING GATE
- **Affects**: `docs/specs/command-framework/requirements.md` (new Requirement 9), `docs/specs/navigation-commands/requirements.md` (new Requirement 20 + note on Req 3), `docs/specs/menu-workspace/requirements.md` (extends Req 5, Req 11); later `ff-command`, `ff-navigation-commands`, `ff-desktop` shell dispatch
- **Linked spec**: `docs/specs/command-framework/requirements.md`, `docs/specs/navigation-commands/requirements.md`, `docs/specs/menu-workspace/requirements.md`

### CR-NR-055 -- Periodic Logging Inventory and Gap Report tool
- **Date/Phase**: Phase (log-audit) (pre-gate)
- **Prompt**: "Examine the project code and create a list of potential 'Log items' that could appear in the logs. Also make a list of possible gaps in logging... write a script to re-create this list periodically? if there are gaps in the logging we should probably create a bug report to close the gaps?"
- **Description**: Add a reusable maintenance tool under `tools/` (per tooling.md) that statically scans the Rust workspace and regenerates two artefacts on demand: (1) a Logging Inventory -- every `ff_logging::log_*!` macro call, `ff_logging::log()`/`log_lazy()` call, and `PluginLogHandle` method call, grouped by crate, with file:line, level (where statically determinable), and the enclosing module; and (2) a Logging Gap report -- crates with zero log calls, plus silent-error-swallow sites (`let _ =` on a Result, `.ok()` discarding a Result, `unwrap()`/`expect()` in non-test code, empty error match arms). Output is a generated Markdown report written outside the committed source tree (or to a designated `docs/quality/` artefact if the team wants it tracked) with stdout mirrored to `tools/logs/` per the tooling rules. The tool is read-only (no source mutation) and safe to rerun. It does NOT auto-file bugs; gap review remains a human decision (the initial gaps found this phase are logged as B034-B038).
- **Status**: IN PROGRESS -- gate complete (logging-subsystem Req 12, design Section 12 + Property 12, Tasks 23-24, master Phase DD, TCR Req 12 rows). Approved: Python, output to `docs/quality`. Implementing the tool next.
- **Affects**: `tools/python/logging_inventory.py` (new Python tool); `docs/quality/logging-inventory.md` (generated tracked artefact); `tools/logs/logging-inventory.txt` (mirrored log); no crate source changes
- **Linked spec**: `docs/specs/logging-subsystem/requirements.md` Requirement 12; `docs/specs/logging-subsystem/design.md` Section 12; Tasks 23-24; master Phase DD

### CR-NR-056 -- Systematic whole-project analysis and re-structuring
- **Date/Phase**: Phase DG (pre-gate)
- **Prompt**: "This project is large and broken into many smaller subprojects... Create a plan and task list to systematically analyse this whole project in a logical sequence. Each sub project must be analysed to see if it could be split into smaller logical units... consistency and conflict resolution... ensure development work is completed tested... proper logging is enabled... Existing incompleted Tasks will have to be revised and re-ordered... a task at the end of each logical unit completion to commit and push to Git."
- **Description**: Introduce a meta/analysis sub-project (`project-analysis`) defining a systematic, dependency-ordered pass over all 79 sub-projects. For each: (1) evaluate whether it should be split into smaller Logical_Units (proposal only, no execution); (2) check cross-unit consistency/conflicts via a Consistency_Matrix; (3) verify completeness (tasks vs TCR vs code+tests) and log incomplete work into a consolidated Incomplete_Work_Register that extends the prior EI-3 audit; (4) audit logging enablement per crate (findings only, no source edits); (5) revise and re-order incomplete tasks. Analysis proceeds in 6 dependency waves; a commit+push checkpoint follows each analysed sub-project. Non-destructive: no crate source edited, no criteria deleted, no split executed -- all material changes surfaced as owner-reviewable proposals.
- **Status**: PENDING GATE -- spec drafted (requirements.md, design.md, tasks.md); awaiting owner approval to begin the analysis pass
- **Affects**: `docs/specs/project-analysis/` (new); later `project-master/tasks.md`, `docs/quality/TCR.md`, per-sub-project `tasks.md` ordering (doc-only revisions)
- **Linked spec**: `docs/specs/project-analysis/requirements.md`, builds on `docs/specs/ears-integration/*` and `docs/reviews/requirements-review/*`
