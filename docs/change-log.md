# FileForge Workbench — Change Log

Tracks every new requirement and change request raised via user prompts.
Entries are appended automatically by the prompt-triage rule.
Never delete a row — update `Status` in-place.

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

### CR-NR-001 — Prompt triage and change tracking
- **Date/Phase**: Phase AS
- **Prompt**: "Can we create a steering rule that every prompt is evaluated as a bug or a new requirement"
- **Description**: Add a steering rule that classifies every user prompt as a bug, new requirement, change request, question, task, or refactor. Bugs are logged to `docs/bugs.md`; new requirements and change requests are logged to `docs/change-log.md`.
- **Status**: DONE
- **Linked spec**: `.amazonq/rules/prompt-triage.md` (new rule file)

### CR-NR-002 — File Explorer Panel (POM option 2)
- **Date/Phase**: Phase AS
- **Prompt**: "opetion 2 needs to be a file Exploere. it should have nodes for each open catalog, and list the files that belong in the catalog in a tree view. Option 2 can be invoked by typing =files and pressing enter, Typeing =2 and pressing enter"
- **Description**: POM option 2 becomes a File Explorer panel showing all open catalogs as tree nodes with their files listed beneath. Commands `=2` and `=FILES` close the current context and switch to the Files context in-place; `FILES` (no `=`) opens a new tab in the Files context.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Requirement 19

### CR-NR-004 — Default Native catalog pointing to user home directory on first launch
- **Date/Phase**: Phase AX
- **Prompt**: "By Default the when FFWB starts up if there are no native catalogs in existence, it should create a native catalog pointing to the users home directory, and mount it immediately, so that when the files context window is opened we at least can see the users home directory. Once created this default catalog should persist and be there on next start up."
- **Description**: On first launch (or any launch where the catalog registry contains no Native catalogs), FFWB shall automatically create a Native catalog named `Home` pointing to the user's home directory, register it with `auto_mount = true`, and persist it so it survives subsequent restarts.
- **Status**: DONE

### CR-NR-003 — HLQ pre-population in Allocate Dataset dialog
- **Date/Phase**: Phase AW
- **Prompt**: "when defining a dataset catalog we are asked for a default high level qualifier, this should be pre-populated in the allocate dataset dataset name text box"
- **Description**: When the Allocate Dataset dialog opens for a Mainframe catalog that has a Default HLQ configured, the Dataset Name field shall be pre-populated with that HLQ followed by a dot, so the user only needs to type the remaining qualifiers.
- **Status**: DONE
- **Linked spec**: `docs/specs/virtual-catalog-manager/requirements.md` Requirement 5.2 (new criterion 5.7)

### CR-NR-006 — File Explorer context menu with file operations
- **Date/Phase**: Phase AZ
- **Prompt**: "When right clicking on a file in the file tree a popup menu should appear with normal file operations listed..."
- **Description**: Right-clicking any node in the File Explorer Panel shall display a context menu whose items are determined by the combination of catalog type, node kind, and file extension. Menus cover Native files/directories, POSIX files (read-only), and Mainframe datasets/members/GDG. Copy puts the full file path on the OS clipboard; pasting into an editor tab prompts for file name vs file contents. Rename is inline label edit. Move To / Copy To use ff-bgio with a progress indicator. Git submenu and Submit JCL are present but greyed-out (deferred).
- **Status**: DONE — Phase AZ complete, 431 ff-desktop tests passing
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 16)

### CR-NR-007 — Open With Default Application (file type association launch)
- **Date/Phase**: Phase BA
- **Prompt**: "In Windows file extensions often determine the type of file... When double clicking on these files... The application should launch the appropriate application and open the file."
- **Description**: Double-clicking or selecting "Open" on a Native/POSIX file node shall launch the OS default application when the file is binary or maps to an external file class (.docx, .xlsx, .pdf, .png, etc.). Text/source files continue to open in the FFWB editor. Platform dispatch: Windows=ShellExecuteEx verb "open", macOS=`open`, Linux=`xdg-open`. "Open With..." shows the platform picker. Non-blocking via `Command::spawn()`. Mainframe datasets always open in FFWB.
- **Status**: DONE — Phase BA complete, 443 ff-desktop tests passing
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 17)

### CR-NR-005 — File Explorer: expandable subdirectories and scrollable panel
- **Date/Phase**: Phase AY
- **Prompt**: "i have created a Native catalog called CDRIVE which now shows all the directories and files from the Root directory, Each directory should be able to be expanded to show the files in it, Also the Screen should be scrollable so that we can page down to see more files"
- **Description**: Native catalog directory nodes in the File Explorer shall be expandable (click to show children recursively) and the panel content area shall be scrollable so the user can page through large directory listings.
- **Status**: DONE
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new criteria 15.1–15.3)

---

## Change Requests

Modifications to existing behaviour that already works.

### CR-CH-001 — POM option 2 description update
- **Date/Phase**: Phase AS
- **Prompt**: "opetion 2 needs to be a file Exploere..."
- **Description**: POM option 2 label updated from "View Edit Create and Delete of files" to "File Explorer — Browse catalogs and files in a tree view".
- **Affects**: `ff-desktop` `primary_option_menu.rs`, `startup-and-session/requirements.md`
- **Status**: PENDING GATE

### CR-CH-003 — Help fallback message uses human-readable context label
- **Date/Phase**: Phase AV
- **Prompt**: "When a help is not available for a specific context the message should display something like 'help not yet available for Context' in a way that we can determine which help needs to be built"
- **Description**: The `resolve_with_fallback` message shall replace the raw `TopicKey` string (e.g. `cmd:FIND`) with a human-readable label (e.g. `command "FIND"`) so developers can identify exactly which help topic needs authoring.
- **Affects**: `ff-help` `context_detector.rs`
- **Status**: DONE

### CR-CH-002 — Home catalog deletion blocked
- **Date/Phase**: Phase AX
- **Prompt**: "Deleting the Home catalog should not be allowed!"
- **Description**: Req 14.6 revised: the `"Home"` Native catalog is protected from deletion. The Catalog Manager Dialog shall reject any delete attempt on a catalog named `"Home"` of type `Native` with an inline error. Renaming and editing remain permitted (Req 14.7).
- **Affects**: `ff-desktop` `catalog_manager_dialog.rs`, `virtual-catalog-manager/requirements.md` Req 14.6
- **Status**: DONE

---

## Changelog

| Phase | Change |
|-------|--------|
| Phase AS | File created. CR-NR-001 logged — prompt triage steering rule. |

### CR-NR-008 — File Explorer: sorted file listing with file attributes
- **Date/Phase**: Phase BB
- **Prompt**: "The files list displays in no particular order, we should sort the file by filename order. We should also see more than just the file name but some of the file attributes, like file size, Timestamp created, timestamp Modified, Timestamp accessed. Also perhaps some of the other attributes like the permission attributes in a user friendly way?"
- **Description**: Native catalog file and directory nodes in the File Explorer shall be sorted alphabetically (directories first, then files, both case-insensitive). Each file node shall display file size, created timestamp, modified timestamp, accessed timestamp, and permission attributes (read/write/execute, hidden, system) in a user-friendly format alongside the file name.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 18)
