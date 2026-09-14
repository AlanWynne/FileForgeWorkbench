# Core Acceptance Test Plan

The sign-off gate for the CORE product (see `ROADMAP.md` Section 3). CORE is
DONE only when every REQUIRED test below passes. Run against a debug build:

```
C:\tools\powershell7\pwsh.exe -NoProfile -Command "cargo run -p ff-desktop"
```

## This is a living regression document

Treat this file as the routine, repeatable manual test pass -- not an append-only
log. The intent is that you open the app and work through the groups in order,
top to bottom, every test cycle. Because the whole document is walked each pass,
regressions are caught: a row that passed before is re-run after every change, so
a feature that quietly broke after a recent edit is found rather than assumed
good.

Process (how we keep it working as we go):
- You run the app and work down the groups in sequence.
- When a step FAILs or you spot a gap, tell the agent the symptom in a prompt.
- The agent triages that prompt (bug vs new requirement vs change request per
  `.kiro/steering/workflow.md`), logs it to `docs/status/bugs.md` or
  `docs/status/change-log.md`, AND amends THIS document -- inserting or updating
  the test case in its correct group/position so the next pass covers it.
- After any code change, the affected group(s) are reset to `[ ]` (needs retest)
  using the Regression Traceability map at the bottom, so nothing is skipped just
  because it worked previously.

Row status legend:
- `[ ]` not run / needs (re)test
- `[P]` pass (record the build/commit that passed it)
- `[F]` fail (a bug id must exist in `docs/status/bugs.md`)
- `[B]` blocked -- the capability is not built yet; the row traces to a PENDING
  GATE change-request (id shown in the Notes/Backing column). A `[B]` row does
  not gate sign-off until its gate is implemented, but it stays visible in place
  so testing is structured, not deferred to a separate list.

Coverage note: much logic has automated coverage (per-crate `cargo test` +
`verify.ps1`); this plan is the MANUAL / end-to-end pass over the running app.

Priority: rows marked (REQUIRED) gate CORE sign-off; (OPTIONAL) are desirable but
not blocking.

---

## Group 1 -- Shell framework: tabbed, detachable workspaces (Core #1)

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 1.1 | Launch the app. | Workbench opens with a POM workspace as the first tab; `Command ===>` line visible; status bar present. | startup-and-session 14.1 | [ ] |
| 1.2 | Open a second workspace (type `2`/`FILES`, or `1`). | A new tab opens and becomes active; previous tab remains in the tab bar. | multi-tab-editor | [ ] |
| 1.3 | Switch between tabs (click / keyboard). | Active tab changes; content swaps; no crash. | multi-tab-editor | [ ] |
| 1.4 | `SWAP n` on the command line (e.g. `SWAP 2`). | Switches to the n-th tab/workspace (`SWAP 1` -> first, `SWAP 2` -> second). | B043; multi-tab-editor (criterion pending) | [B] |
| 1.5 | `SWAP LIST` on the command line. | A selectable list of open tabs pops up; clicking a row OR typing a number + Enter switches to that tab. | B043; multi-tab-editor (criterion pending) | [B] |
| 1.6 | Close a tab via its close control. | Tab closes; a sensible neighbour becomes active; POM cannot be lost (at least one tab remains). | multi-tab-editor 3.8 | [ ] |
| 1.7 | END (F3) from a POM tab when other tabs are open. | Closes only that POM Workspace and navigates to another open Workspace; the app does NOT exit. | CR-CH-016 (pending) | [B] |
| 1.8 | END (F3) from a POM tab when it is the LAST tab open. | The application terminates. | CR-CH-016 (pending) | [B] |
| 1.9 | Detach a workspace via the tab right-click menu (Detach/Undock action). | A Detach/Undock action exists in the tab context menu; invoking it moves the workspace into a separate OS window (Detached Workspace); content intact. | B045; layout-and-docking 3.1 | [B] |
| 1.10 | Detach by dragging a tab >20px outside the tab bar and releasing. | A new Detached Workspace is created at the release point; content intact. | B045; layout-and-docking 3.9 | [B] |
| 1.11 | Re-dock the detached workspace (close its window or redock gesture). | It returns to the tab bar at its origin; content intact. | B045; layout-and-docking 3.5, 3.11 | [B] |
| 1.12 | Exit and relaunch. | Session restores tabs/active workspace; POM present. | startup-and-session 14.1b | [ ] |

---

## Group 2 -- Menu workspaces, Settings, menu bar, Create/Edit-Menu (Core #2)

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 2.1 | View the POM. | Rendered as a menu workspace: rows of option / command / description. | menu-workspace | [ ] |
| 2.2 | Open the Settings menu (`SETTINGS` / `0` / `=0`). | Opens as a POM-modelled menu workspace (option/command/description rows), NOT the flat custom panel. | B032; menu-workspace 11.2 | [B] |
| 2.3 | In the Settings menu, select an option by its key. | Routes to the mapped target (same as typing it on the command line). | menu-workspace 3.x | [ ] |
| 2.4 | Type the command for a Settings option on the command line + Enter. | Switches to that option/namespace -- every Settings option has a matching command. | B032; CR-NR-068 (pending) | [B] |
| 2.5 | Inspect the application menu bar. | The bar entries align with what is actually available (no dead entries). | CR-NR-067 (pending) | [B] |
| 2.6 | Open the menu bar AS a Menu Workspace. | The menu bar is itself a named Menu Workspace that can be opened and viewed like the POM. | CR-NR-067 (pending) | [B] |
| 2.7 | Give a workspace its own named menu bar; open another with none. | A workspace uses its named menu bar; a workspace with none falls back to the Primary menu bar. | CR-NR-067 (pending) | [B] |
| 2.8 | Open the Menu Workspace editor (create/edit/delete menus). | A workspace opens for creating/editing/deleting Menu Workspaces (POM, Settings, custom, menu bars): a Name field + an editable list of option/command/description rows. | CR-NR-068 (pending); Core #2 KEY gap | [B] |
| 2.9 | Create a menu "MyTools" with 2-3 option/command/description rows and Save. | The menu is saved as a menu definition file; no error. | CR-NR-068 (pending) | [B] |
| 2.10 | Open the newly-saved menu. | Renders as a menu workspace with the entered rows; selecting an option runs its command. | CR-NR-068 (pending) | [B] |
| 2.11 | Edit an existing menu (add/remove a row) and Save. | Changes persist and re-render. | CR-NR-068 (pending) | [B] |

---

## Group 3 -- File navigator: content, context menu, commands (Core #3)

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 3.1 | Open the File Explorer (option 2 / FILES). | Modern tree: Local Files + Catalogs roots. | file-tree-panel 24.1 | [ ] |
| 3.2 | Expand "Local Files" and compare to the real drive. | The tree matches actual filesystem contents; the Local Files root path is discoverable; Windows known-folder junctions (Documents, My Documents, My Music/Pictures/Videos) resolve to their real target or are omitted -- never shown as dead/empty non-expandable nodes. | B044; file-tree-panel 24.5 | [F] |
| 3.3 | Double-click a text file. | Opens in an editor workspace. | file-tree-panel 24.9 | [ ] |
| 3.4 | Right-click a file -> context menu. | Menu shows View, Edit, Copy, Paste, Copy Full Path, Reveal, Rename, Delete, New File, New Folder. "Open" is REPLACED by "View" + "Edit" (View -> editor in View mode; Edit -> editor in Edit mode). | CR-NR-062 scope-ext (pending) | [B] |
| 3.5 | Use each context-menu action, and its matching command-line command. | Every action (View, Edit, Copy, Paste, Copy Full Path, Reveal, Rename, Delete, New File, New Folder) has a command-line equivalent that accepts the selected file name as a parameter and behaves per its spec. | CR-NR-062 scope-ext (pending) | [B] |
| 3.6 | Rename a file via the context menu. | File renamed on disk; tree refreshes. | file-tree-panel 16 | [ ] |
| 3.7 | New File / New Folder via the context menu. | Created on disk; tree refreshes. | file-tree-panel 16 | [ ] |
| 3.8 | Delete a file (confirm dialog). | Deleted after confirmation; tree refreshes. | file-tree-panel 16 | [ ] |
| 3.9 | Multi-select (Ctrl/Shift click) + Ctrl+C. | Selection highlights; Ctrl+C copies an indented text tree to the clipboard. | file-tree-panel 19 | [ ] |
| 3.10 | Repeat 3.6-3.8 on a file inside a POSIX/Native catalog. | Edit ops operate on the catalog's real file (B042 fix). | file-tree-panel 24.6 | [ ] |
| 3.11 | Tab from the command line into the tree; arrow-navigate; Escape back. | Focus enters the node list, arrows move the cursor, Escape returns to the command line. | file-tree-panel 24.9 | [ ] |
| 3.12 | Single-click a file node. | The node is selected AND the Command ===> field is populated with `VIEW '<file>'`; clicking a different file updates it; Enter opens the file in an Editor Context in View mode. | CR-NR-062 (pending) | [B] |
| 3.13 | Type `cd <path>` on the command line + Enter. | The navigator focus/cursor moves to that node exactly as a mouse click would, scrolling it into view; if expandable, it expands. | CR-NR-066 (pending) | [B] |

---

## Group 4 -- ISPF editor: primary + line commands, View + Edit (Core #4)

Highest-priority known gap. Do it thoroughly.

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 4.1 | Open a file in View mode (`VIEW <file>`). | Opens read-only; status bar shows View mode; typing does NOT mutate. | edit-operations 17; CR-NR-062 | [ ] |
| 4.2 | In View mode run a display filter (INCLUDE / EXCLUDE / FIND). | Filter applies (display-only); document unchanged. | exclude-show-filter, find-and-replace | [ ] |
| 4.3 | Switch to Edit mode (type `EDIT` + Enter, or open `EDIT <file>`). | Mode switches to Edit; status bar reflects it; mutation now allowed. | edit-operations 17; CR-NR-062 | [ ] |
| 4.4 | Type text and confirm the buffer changes. | Edits apply; undo/redo work. | edit-operations, undo-redo-transactions | [ ] |
| 4.5 | Run PRIMARY commands from the editor command line (FIND, CHANGE, locate/line number). | Each executes against the document (find highlights, change edits, locate scrolls). | edit-operations, navigation-commands | [ ] |
| 4.6 | Enter LINE commands in the gutter (D, I, R, C/M + A/B, X). | Each line command executes on the target line(s) as in ISPF. | line-commands | [ ] |
| 4.7 | Block line commands (DD..DD, CC..CC). | Block operation applies across the marked range. | line-commands | [ ] |
| 4.8 | Save the file (Ctrl+S or SAVE). | File persists to disk; modified indicator clears. | file-operations | [ ] |
| 4.9 | Attempt a mutation while in View mode. | Rejected / no-op; a clear indication mutation is disallowed in View. | edit-operations 17; CR-NR-062 | [ ] |
| 4.10 | F3/END from the editor. | Closes/returns per ISPF convention without data loss (prompt if unsaved). | function-keys-and-history; CR-CH-016 | [ ] |

---

## Group 5 -- Command-line execution of everything (Core #5)

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 5.1 | From the command line, run a navigation command (open a workspace by key). | Same effect as clicking the menu option. | command-semantics | [ ] |
| 5.2 | Run a file command (`VIEW`/`EDIT <file>`). | Opens the file in the right mode. | command-semantics, edit-operations 17 | [ ] |
| 5.3 | Open the Command Palette (Ctrl+Shift+P) and run a command. | Palette lists commands; selection executes. | command-palette | [ ] |
| 5.4 | Run a chained command sequence (STOP `.` vs PUSH `;` semantics). | Chain executes per CR-NR-057 rules (single shared chain executor). | command-framework 10, command-semantics 11 | [ ] |
| 5.5 | Confirm a representative set of activities each have a command equivalent. | Core activities (open workspace, file ops, editor ops, settings) are reachable by command. | command-framework | [ ] |
| 5.6 | Run a short list of commands as a batch/macro. | Commands run in sequence; results as expected. | batch-execution | [ ] |

---

## Group 6 -- Macros as commands (Core #6)

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 6.1 | Author a macro (a list of commands / a Lua macro) and save it to a file. | Macro persists to a file. | lua-macro-engine 12 | [ ] |
| 6.2 | Run the saved macro. | The macro's commands execute in order against the workbench. | lua-macro-engine, batch-execution | [ ] |
| 6.3 | In Settings, configure the saved macro AS an individual command (name it). | The macro is registered as a named command. | command-configurator | [ ] |
| 6.4 | Invoke that macro-command by name from the command line. | The macro runs, same as 6.2. | command-configurator, command-semantics | [ ] |
| 6.5 | Confirm the macro-command survives restart. | It is persisted and available after relaunch. | command-configurator, startup-and-session | [ ] |

---

## Group 7 -- Function keys (Core: whole function-keys spec)

The whole function-keys-and-history spec is CORE. Exercise each PF key in each
relevant context (POM, editor View/Edit, File Explorer).

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 7.1 | Press F1 in a context with help. | Context help opens for the current context. | function-keys-and-history; B046 | [F] |
| 7.2 | Press F3/END in the editor, POM, and explorer. | Performs the mapped END/return action per context (POM per CR-CH-016). | function-keys-and-history; B046; CR-CH-016 | [F] |
| 7.3 | Press F7 / F8 in a scrollable context. | Scroll up / down by page. | function-keys-and-history; B046 | [F] |
| 7.4 | Press PF2 while editing. | Splits the screen at the cursor line into two independent editor halves. | menu-and-statusbar 11; B046 | [F] |
| 7.5 | Press PF9 while split. | Swaps keyboard focus between the two split halves. | menu-and-statusbar 12; B046 | [F] |
| 7.6 | Press F3 (END) while split. | Unsplits, restoring the single-panel view. | menu-and-statusbar 14; B046 | [F] |
| 7.7 | Confirm each remaining PF binding in the spec fires its action. | Every PF key mapped in the function-keys spec performs its action in-context. | function-keys-and-history; B046 | [F] |

---

## Group 8 -- Theme (Core)

Theme changing is CORE. Covers the theme round-trip fix (B039) and the Theme
Settings workspace + commands (CR-NR-070).

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 8.1 | Select each theme (Dark / Light / Legacy / High Contrast) from Settings. | The theme applies immediately and persists across frames and restart; High Contrast sticks (does not revert to Dark). | B039; theme-and-appearance 5.x, 16.x | [F] |
| 8.2 | Set a theme by command from the command line. | A command sets the active theme by name; the change applies and persists. | CR-NR-070 (pending) | [B] |
| 8.3 | Invoke the Theme Settings workspace by command. | A command opens the Theme Settings workspace. | CR-NR-070 (pending) | [B] |
| 8.4 | Create / edit / delete a theme setting in the Theme Settings workspace. | Theme settings can be created, edited, and deleted; changes take effect. | CR-NR-070 (pending) | [B] |

---

## Sign-off

CORE is signed off when all REQUIRED, buildable rows are `[P]` and no row is
`[F]`. `[B]` rows are not blocking until their backing gate is implemented, but
each must become `[P]` before final CORE sign-off. Record:

| Field | Value |
|-------|-------|
| Build / commit | (fill in) |
| Date | (fill in) |
| REQUIRED pass count | ( / total) |
| Outstanding FAILs (bug IDs) | (fill in) |
| Outstanding BLOCKED (CR ids) | (fill in) |
| Signed off by | (owner) |

Once signed off, PLUGIN phases (ROADMAP Section 5) may begin, each with its own
test pass before the next.

---

## Regression Traceability (bug/CR -> affected test rows)

After a change lands, reset the affected rows below to `[ ]` (needs retest) and
re-run them next pass. This is how regressions are caught rather than assumed
fixed. Keep this map updated as rows are added.

| Backing item | Status | Affected rows |
|--------------|--------|---------------|
| B032 (Settings not a menu workspace) | OPEN | 2.2, 2.4 |
| B039 (theme changing broken) | OPEN | 8.1 |
| B040 (preview toggle crash) | FIXED | 3.1, 3.2 |
| B041 (catalog repo init / expand) | FIXED | 3.10 |
| B042 (catalog subtree edit ops) | FIXED | 3.10 |
| B043 (SWAP command) | OPEN | 1.4, 1.5 |
| B044 (Local Files content / junctions) | OPEN | 3.2 |
| B045 (detach not wired) | OPEN | 1.9, 1.10, 1.11 |
| B046 (function keys not working) | OPEN | 7.1-7.7 |
| CR-CH-016 (END from POM) | PENDING GATE | 1.7, 1.8, 4.10, 7.2 |
| CR-NR-062 (View/Edit + context-menu commands) | PENDING GATE | 3.4, 3.5, 3.12, 4.1, 4.3, 4.9, 5.2 |
| CR-NR-066 (`cd` focuses navigator) | PENDING GATE | 3.13 |
| CR-NR-067 (configurable menu bar) | PENDING GATE | 2.5, 2.6, 2.7 |
| CR-NR-068 (Menu Workspace editor) | PENDING GATE | 2.4, 2.8, 2.9, 2.10, 2.11 |
| CR-NR-069 (Key Assignment editor) | PENDING GATE | (adds rows to Group 7 at its gate) |
| CR-NR-070 (Theme Settings workspace + commands) | PENDING GATE | 8.2, 8.3, 8.4 |
