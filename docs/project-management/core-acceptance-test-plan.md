# Core Acceptance Test Plan

The sign-off gate for the CORE product (see `ROADMAP.md` Section 3). CORE is
DONE only when every REQUIRED test below passes. Run against a debug build:

```
C:\tools\powershell7\pwsh.exe -NoProfile -Command "cargo run -p ff-desktop"
```

How to use:
- Work through each group in order. Record the result in the Result column
  (PASS / FAIL / BLOCKED) and note the build/date.
- A FAIL becomes a bug in `docs/status/bugs.md` (B### row) and is fixed before
  sign-off.
- "REQUIRED" tests gate CORE sign-off; "OPTIONAL" are desirable but not blocking.
- Automated coverage exists for much of the logic (per-crate `cargo test` +
  `verify.ps1`); this plan is the MANUAL / end-to-end pass over the running app.

Legend: [ ] not run, [P] pass, [F] fail, [B] blocked.

---

## Group 1 -- Shell framework: tabbed, detachable workspaces (Core #1)

| # | Step | Expected result | Req | Result |
|---|------|-----------------|-----|--------|
| 1.1 | Launch the app. | Workbench opens with a POM workspace as the first tab; `Command ===>` line visible; status bar present. | startup-and-session 14.1 | [ ] |
| 1.2 | Open a second workspace (e.g. type `2`/`FILES`, or `1`). | A new tab opens and becomes active; previous tab remains in the tab bar. | multi-tab-editor | [ ] |
| 1.3 | Switch between tabs (click / keyboard). | Active tab changes; content swaps; no crash. | multi-tab-editor | [ ] |
| 1.4 | Close a tab via its close control. | Tab closes; a sensible neighbour becomes active; POM cannot be lost (at least one tab remains). | multi-tab-editor 3.8 | [ ] |
| 1.5 | Detach a workspace to its own OS window. | Workspace becomes a Detached Workspace in a separate window; content intact. | layout-and-docking | [ ] |
| 1.6 | Re-dock the detached workspace. | It returns to the tab bar; content intact. | layout-and-docking | [ ] |
| 1.7 | Exit and relaunch. | Session restores tabs/active workspace; POM present. | startup-and-session 14.1b | [ ] |

---

## Group 2 -- Menu workspaces + Create-Menu dialog (Core #2)

| # | Step | Expected result | Req | Result |
|---|------|-----------------|-----|--------|
| 2.1 | View the POM. | Rendered as a menu workspace: rows of option / command / description. | menu-workspace | [ ] |
| 2.2 | Open the Settings menu. | Opens as a menu workspace modelled on the POM (option/command/description rows). | menu-workspace 11.2 | [ ] |
| 2.3 | Select an option by its key from the menu. | Routes to the mapped command/target (same as typing it on the command line). | menu-workspace 3.x | [ ] |
| 2.4 | Open the **Create Menu** dialog (from Settings). | A dialog appears to create a new menu: a Name field + an editable list of option/command/description rows. | menu-workspace (create-menu) | [ ] |
| 2.5 | Create a menu named e.g. "MyTools" with 2-3 option/command/description rows and Save. | The menu is saved (as a menu definition file); no error. | menu-workspace | [ ] |
| 2.6 | Open the newly-saved menu. | It renders as a menu workspace with the rows entered; selecting an option runs its command. | menu-workspace | [ ] |
| 2.7 | Edit an existing menu (add/remove a row) and Save. | Changes persist and re-render. | menu-workspace | [ ] |

NOTE: 2.4-2.7 (the Create/Edit-Menu dialog) is the KEY known gap for Core #2.

---

## Group 3 -- File directory navigator (Core #3)  [largely DONE, confirm]

| # | Step | Expected result | Req | Result |
|---|------|-----------------|-----|--------|
| 3.1 | Open the File Explorer (option 2 / FILES). | Modern tree: Local Files + Catalogs roots. | file-tree-panel 24.1 | [ ] |
| 3.2 | Expand a Local Files directory. | Children load via the VFS provider. | file-tree-panel 24.5 | [ ] |
| 3.3 | Double-click a text file. | Opens in an editor workspace. | file-tree-panel 24.9 | [ ] |
| 3.4 | Right-click a file -> context menu. | Menu with Open / Copy / Paste / Copy Full Path / Reveal / Rename / Delete / New File / New Folder. | file-tree-panel 16 | [ ] |
| 3.5 | Rename a file via the context menu. | File renamed on disk; tree refreshes. | file-tree-panel 16 | [ ] |
| 3.6 | New File / New Folder via the context menu. | Created on disk; tree refreshes. | file-tree-panel 16 | [ ] |
| 3.7 | Delete a file (confirm dialog). | Deleted after confirmation; tree refreshes. | file-tree-panel 16 | [ ] |
| 3.8 | Multi-select (Ctrl/Shift click) + Ctrl+C. | Selection highlights; Ctrl+C copies an indented text tree to the clipboard. | file-tree-panel 19 | [ ] |
| 3.9 | Repeat 3.4-3.7 on a file inside a POSIX/Native catalog. | Edit ops operate on the catalog's real file (B042 fix). | file-tree-panel 24.6 | [ ] |
| 3.10 | Tab from the command line into the tree; arrow-navigate; Escape back. | Focus enters the node list, arrows move the cursor, Escape returns to the command line. | file-tree-panel 24.9 | [ ] |

---

## Group 4 -- ISPF editor: primary + line commands, Browse/View + Edit (Core #4)

This is the highest-priority known gap. Do it thoroughly.

| # | Step | Expected result | Req | Result |
|---|------|-----------------|-----|--------|
| 4.1 | Open a file in View mode (e.g. `VIEW <file>`). | Opens read-only; status bar shows View/Browse mode; typing does NOT mutate. | edit-operations 17 | [ ] |
| 4.2 | In View mode run a display filter (INCLUDE / EXCLUDE / FIND). | Filter applies (display-only); document unchanged. | exclude-show-filter, find-and-replace | [ ] |
| 4.3 | Switch to Edit mode (type `EDIT` + Enter, or open with `EDIT <file>`). | Mode switches to Edit; status bar reflects it; mutation now allowed. | edit-operations 17 | [ ] |
| 4.4 | Type text and confirm the buffer changes. | Edits apply; undo/redo work. | edit-operations, undo-redo-transactions | [ ] |
| 4.5 | Run PRIMARY commands from the editor command line: e.g. FIND, CHANGE, locate/line number. | Each executes against the document (find highlights, change edits, locate scrolls). | edit-operations, navigation-commands | [ ] |
| 4.6 | Enter LINE commands in the gutter/prefix area: e.g. D (delete), I (insert), R (repeat), C/M + A/B (copy/move + after/before), X (exclude). | Each line command executes on the target line(s) as in ISPF. | line-commands | [ ] |
| 4.7 | Block line commands (e.g. DD..DD, CC..CC). | Block operation applies across the marked range. | line-commands | [ ] |
| 4.8 | Save the file (Ctrl+S or SAVE). | File persists to disk; modified indicator clears. | file-operations | [ ] |
| 4.9 | Attempt a mutation while in View mode. | Rejected / no-op; a clear indication mutation is disallowed in View. | edit-operations 17 | [ ] |
| 4.10 | F3/END from the editor. | Closes/returns per ISPF convention without data loss (prompt if unsaved). | function-keys-and-history | [ ] |

---

## Group 5 -- Command-line execution of everything (Core #5)

| # | Step | Expected result | Req | Result |
|---|------|-----------------|-----|--------|
| 5.1 | From the command line, run a navigation command (e.g. open a workspace by key). | Same effect as clicking the menu option. | command-semantics | [ ] |
| 5.2 | Run a file command (e.g. VIEW/EDIT `<file>`). | Opens the file in the right mode. | command-semantics, edit-operations 17 | [ ] |
| 5.3 | Open the Command Palette (Ctrl+Shift+P) and run a command. | Palette lists commands; selection executes. | command-palette | [ ] |
| 5.4 | Run a chained command sequence (STOP `.` vs PUSH `;` semantics). | Chain executes per CR-NR-057 rules (single shared chain executor). | command-framework 10, command-semantics 11 | [ ] |
| 5.5 | Confirm a representative set of activities each have a command equivalent. | Core activities (open workspace, file ops, editor ops, settings) are reachable by command. | command-framework | [ ] |
| 5.6 | Run a short list of commands as a batch/macro. | Commands run in sequence; results as expected. | batch-execution | [ ] |

---

## Group 6 -- Macros as commands (Core #6)

| # | Step | Expected result | Req | Result |
|---|------|-----------------|-----|--------|
| 6.1 | Author a macro (a list of commands / a Lua macro) and save it to a file. | Macro persists to a file. | lua-macro-engine 12 | [ ] |
| 6.2 | Run the saved macro. | The macro's commands execute in order against the workbench. | lua-macro-engine, batch-execution | [ ] |
| 6.3 | In Settings, configure the saved macro AS an individual command (name it). | The macro is registered as a named command. | command-configurator | [ ] |
| 6.4 | Invoke that macro-command by name from the command line. | The macro runs, same as 6.2. | command-configurator, command-semantics | [ ] |
| 6.5 | Confirm the macro-command survives restart. | It is persisted and available after relaunch. | command-configurator, startup-and-session | [ ] |

---

## Sign-off

CORE is signed off when all REQUIRED rows are [P]. Record:

| Field | Value |
|-------|-------|
| Build / commit | (fill in) |
| Date | (fill in) |
| REQUIRED pass count | ( / total) |
| Outstanding FAILs (bug IDs) | (fill in) |
| Signed off by | (owner) |

Once signed off, PLUGIN phases (ROADMAP Section 5) may begin, each with its own
test pass before the next.
