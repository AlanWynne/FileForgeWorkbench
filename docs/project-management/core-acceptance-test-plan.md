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
- When a step FAILs or you spot something, tell the agent the symptom in a prompt.
- "No test criteria for this" does NOT automatically mean a criterion is missing.
  You often test things ahead of where the sequential walk has reached, so you
  simply have not arrived at that row yet. Before treating anything as a gap, the
  agent MUST first search this plan AND the specs for an existing criterion that
  already covers it:
    - If one exists (a row further down, another group, or a spec criterion), the
      agent points you to it and records your result against that existing row --
      no new row is invented. It was just ahead of your position, not a gap.
    - Only if it genuinely exists nowhere does the agent log it and insert a new
      row in its correct place.
- On a real FAIL, the agent triages the prompt (bug vs new requirement vs change
  request per `.kiro/steering/workflow.md`), logs it to `docs/status/bugs.md` or
  `docs/status/change-log.md`, AND updates the matching test row (existing or new)
  in its correct group/position so the next pass covers it.
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
| 1.3a | At launch, press Tab repeatedly from the command field. | Focus enters the active Workspace's first control (POM: first enabled option), walks the interior in visual order, then the Menu_Bar, then wraps to the command field. Focus NEVER lands on a Status_Bar segment ("RUNNING", session start, "Ln x, Col y", encoding, line count, version label), the `SCROLL ===>` field, or a Key_Label_Bar F-key button. | B055 (FIXED); CR-CH-023 (AUTOMATED: full-shell egui_kittest tests + retest); menu-and-statusbar 16 | [ ] |
| 1.3b | On a POM, press Tab from the command field through the whole cycle; then Shift+Tab back. | Command line -> each enabled POM option in order -> calendar `<` -> calendar `>` (calendar shown) -> Menu_Bar items -> wrap to command line. Disabled options are skipped. Shift+Tab reverses exactly. | CR-CH-023 (AUTOMATED: interior order + full-shell wrap/Shift+Tab harness tests; retest); menu-workspace 15; menu-and-statusbar 16 | [ ] |
| 1.3c | Open the Menus Editor (`MENUS`); press Tab from the command field. | Command line -> Menu selector -> Title -> Show Calendar -> Group Headers -> Sep Space/Line/None -> per option (key -> Command -> Description -> Group -> on -> up -> down -> delete) -> Add Option -> Save -> New Name -> Save As -> Menu_Bar items -> wrap to command line. No SCROLL field or F-key button in the cycle. | CR-CH-023 (implemented, retest); menu-and-statusbar 16 | [ ] |
| 1.3d | Open the Theme Editor (bare `THEME`); press Tab from the command field. | Command line -> Theme selector combo (NO invisible stop first) -> Set Active -> Reset to built-in -> (Confirm reset / Cancel when a reset is pending) -> New name field -> Copy -> Save As -> Save -> colour hex fields in order -> Menu_Bar items -> wrap to command line. No phantom/invisible focus stop between the command line and the Theme selector. | B057 (VERIFIED by owner); CR-CH-023 (AUTOMATED: full-shell egui_kittest test); menu-and-statusbar 16; theme-and-appearance 20.1 | [x] |
| 1.3e | Open the CONFIG panel (`CONFIG`); press Tab from the command field. | Command line -> Filter field (NO invisible stop first) -> the per-key widgets in order -> Menu_Bar -> wrap. No phantom focus stop between the command line and the Filter field. | B058 (FIXED, CR-CH-025/config-tab, AUTOMATED: `full_shell_config_first_tab_focuses_filter_field`); menu-and-statusbar 16; configuration-system 20 | [ ] |
| 1.4 | `SWAP n` on the command line (e.g. `SWAP 2`); also `SWAP 0`/`SWAP 999`/`SWAP xyz`. | `SWAP 1` -> first tab, `SWAP 2` -> second; out-of-range/invalid shows a clear error and does not switch. | B043 (FIXED, retest); multi-tab-editor 18.1, 18.2 | [ ] |
| 1.5 | `SWAP LIST` (and bare `SWAP` with no split). | A selectable list of open tabs pops up (`n: title`); clicking a row OR typing a number + Enter switches; Escape cancels. | B043 (FIXED, retest); multi-tab-editor 18.3-18.7 | [ ] |
| 1.6 | Close a tab via its close control. | Tab closes; a sensible neighbour becomes active; POM cannot be lost (at least one tab remains). | multi-tab-editor 3.8 | [ ] |
| 1.7 | END (F3) from a POM tab when other tabs are open. | Closes only that POM Workspace and navigates to another open Workspace; the app does NOT exit. | CR-CH-016 (FIXED, retest) | [ ] |
| 1.8 | END (F3) from a POM tab when it is the LAST tab open. | The application terminates. | CR-CH-016 (FIXED, retest) | [ ] |
| 1.9 | Detach a workspace via the tab right-click menu (Detach/Undock action). | A Detach/Undock action exists in the tab context menu; invoking it moves the workspace into a separate OS window (Detached Workspace); content intact. | B045; layout-and-docking 3.1 | [B] |
| 1.10 | Detach by dragging a tab >20px outside the tab bar and releasing. | A new Detached Workspace is created at the release point; content intact. | B045; layout-and-docking 3.9 | [B] |
| 1.11 | Re-dock the detached workspace (close its window or redock gesture). | It returns to the tab bar at its origin; content intact. | B045; layout-and-docking 3.5, 3.11 | [B] |
| 1.12 | Exit and relaunch. | Session restores tabs/active workspace; POM present. | startup-and-session 14.1b | [ ] |

---

## Group 2 -- Menu workspaces, Settings, menu bar, Create/Edit-Menu (Core #2)

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 2.1 | View the POM. | Rendered as a menu workspace: rows of option / command / description. | menu-workspace | [ ] |
| 2.2 | Open the Settings menu (`SETTINGS` / `0` / `=0`). | Opens as a POM-modelled menu workspace (option/command/description rows), NOT the flat custom panel. Options: A Config, T Theme, M Menus (Core); R Reset BARE (Recovery). | B032 (FIXED, CR-CH-025, AUTOMATED: `settings_command_opens_menu_workspace_not_flat_panel`); menu-workspace 11.2 | [ ] |
| 2.2a | Open the Settings menu; press Tab from the command line to the end of the option list and beyond. | No calendar is shown (Settings defaults `show_calendar = false`). Tab walks ONLY the option rows, then the Menu_Bar -- there are NO extra "invisible" Tab stops after the last option. | B060 (FIXED, CR-CH-026, AUTOMATED: `default_settings_toml_hides_calendar`, `full_shell_settings_tab_walks_options_only_no_calendar_stops`); menu-workspace 16.1, 16.4 | [ ] |
| 2.2b | Edit a menu (or Settings) to set `show_calendar = true`, open it in a normal and in a NARROW window. | Normal window: the calendar is VISIBLE on the right and its `<`/`>` month buttons are reachable by Tab and on-screen. Narrow window (too small to fit): the calendar is omitted (not drawn off-screen) and there are NO calendar Tab stops. In neither case is there an invisible/off-screen focus stop. | B060 (FIXED, CR-CH-026, AUTOMATED at render level: `menu_calendar_shown_when_wide_next_button_is_on_screen`, `menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops`; live narrow-window resize is MANUAL); menu-workspace 16.2, 16.3, 16.5, 16.6 | [ ] |
| 2.2c | Open any menu (POM/Settings); shrink the window until a long option description wraps to a second line. | The wrapped continuation of the description hangs-indents under the START of the description column (under the first description character), NOT under the key column at the row's left edge. First-line column alignment (key \| command \| description) is unaffected. | B065 (FIXED, AUTOMATED at layout level: `wrap_description_long_hangs_continuation_under_description`, `option_row_job_wraps_with_hanging_indent`; live narrow-window resize is MANUAL); menu-workspace 2.1a | [ ] |
| 2.3 | In the Settings menu, select an option by its key. | Routes to the mapped target (same as typing it on the command line). | menu-workspace 3.x | [ ] |
| 2.4 | Type the command for a Settings option on the command line + Enter (e.g. `CONFIG`, `THEME`, `MENUS`), and the chained form `SETTINGS T`. | Switches to that option/namespace -- every Settings option has a matching command; `SETTINGS T` opens Settings then activates `T` (THEME). | B032 (FIXED, CR-CH-025, AUTOMATED: `settings_t_chains_to_theme_editor`, `config_command_is_registered`); CR-CH-025 | [ ] |
| 2.4a | From ANY workspace, type an `=` fastpath jump on the command line + Enter (e.g. `=0.K`). | The `=` pops to the POM origin, then the path walks option-by-option: `=0.K` = POM option 0 (Settings) then option K (KEYS) -> lands on the Keys Workspace. No "command not yet implemented" error. Bare dotted `2.1` still works; ordinary dotted text (`abc.def`) is not hijacked. | B061 (FIXED, AUTOMATED: `chained_fastpath_equals_zero_dot_k_opens_keys_workspace`, `chained_fastpath_pops_to_pom_origin_from_non_pom`); menu-workspace Req 5 | [ ] |
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
| 3.2 | Expand "Local Files" (rooted at the user home). | The tree matches home-directory contents; a single locked/permission-denied entry does not blank or truncate the listing (B044 per-entry skip). | B044 (per-entry skip FIXED, retest); file-tree-panel 24.5 | [ ] |
| 3.2a | Expand a Windows known-folder junction (Documents / My Documents / My Music). | Directory junctions expand to show their real contents (or are clearly marked), not dead/empty leaves. | CR-NR-072 (pending) | [B] |
| 3.3 | Double-click a real local text file (incl. a path with spaces / OneDrive). | Opens in an editor workspace; no "resource not found" error; the VFS URI resolves to the real file. | file-tree-panel 24.9; B047 (FIXED, retest) | [ ] |
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
| 7.1 | Press F1 in a context. | Context help opens showing the markdown help for the current context. | function-keys-and-history; CR-NR-071 (help content pending) | [B] |
| 7.2 | Press F3/END in the editor, POM, and explorer. | Performs the mapped END/return action per context. F3 confirmed working; POM-not-last-tab case per CR-CH-016. | function-keys-and-history (works); CR-CH-016 (POM case) | [ ] |
| 7.3 | Press F7 / F8 in a scrollable context. | Scroll up / down by one page (bare UP/DOWN). NOT to end-of-file. | navigation-commands 3.1/3.3; B046 (FIXED, retest) | [ ] |
| 7.3a | Type `M` (or `MAX`) on the command line, then press UP / DOWN (or F7/F8). | Scrolls to the top / bottom (MAX modifier); MAX is a command-line action, not an F-key binding. | navigation-commands; B046 | [ ] |
| 7.3b | Type a value in the `Command ===>` field then press an F-key bound to a command: e.g. `1` + F9 (SWAP), `2` + F9, `8` + F8 (DOWN); and press the F-key with an EMPTY field. | The F-key invokes its bound command with the field content as the argument -- observably identical to typing `<command> <field>` + Enter: `1` + F9 -> swap to workspace 1, `2` + F9 -> workspace 2, `8` + F8 -> page down 8. F9 with an empty field swaps to the previously active workspace (bare SWAP). It does NOT run bare `SWAP` (tab picker) when a value was typed. | B066 (FIXED, AUTOMATED: `key_command_merges_command_field_as_argument`, `key_command_with_empty_field_runs_bare_command`, `key_command_does_not_force_clear_command_field`); command-framework Req 9.8/9.1/9.7/9.9/9.10; multi-tab-editor 18.1 | [ ] |
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
| 8.1 | Select each theme (Dark / Light / Legacy / High Contrast) from Settings (menu dispatches the THEME command), and via `THEME <mode>`. | The theme applies immediately AND STICKS (does not revert next frame); persists across restart; High Contrast sticks. Selecting a theme turns OFF follow-OS so it is not clobbered. | B039 (FIXED real cause: follow_os clobber, retest); theme-and-appearance 5.x, 16.4, 17.5 | [ ] |
| 8.2 | `THEME <name>` on the command line: shorthands (`THEME Dark`/`Light`/`High Contrast`/`Legacy`), full names (`THEME Default Dark`), a saved user theme by name; bare `THEME`; unknown `THEME xyz`; and confirm `THEMES` is gone. | Shorthand selects the matching `Default *` built-in (`Legacy` -> `Default Legacy`); full/user names select by exact case-insensitive match; bare `THEME` opens the Theme Editor (END returns); unknown leaves the theme unchanged and shows `THEME: 'xyz' does not exist`; `THEMES` is no longer a recognised command. | theme-and-appearance 17.1-17.5 (CR-CH-024, IMPLEMENTED + AUTOMATED: `full_shell_theme_*`, `resolve_theme_arg_*`; retest) | [ ] |
| 8.5 | Open the theme list (Settings / Theme editor). | Exactly FOUR built-ins: Default Dark, Default Light, Default High Contrast, Default Legacy. NO separate "Legacy (ISPF 3270)" entry. Saving a copy of Default Legacy under a new name adds it as a selectable user theme. | theme-and-appearance 18.1/18.3 (CR-CH-024, IMPLEMENTED + AUTOMATED: `builtin_themes_returns_four_entries`, `full_shell_theme_list_has_four_builtins`) | [ ] |
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
| B032 (Settings not a menu workspace) | FIXED (CR-CH-025, retest 2.2, 2.4) | 2.2, 2.4 |
| B039 (theme changing broken -- follow_os clobber) | FIXED (retest 8.1) | 8.1 |
| B048 (tests wrote real user config) | FIXED | (test-suite) |
| B049 (history dedup THEME LEGACY vs theme legacy) | FIXED (quote-aware dedup) | (Group 1 history rows) |
| B040 (preview toggle crash) | FIXED | 3.1, 3.2 |
| B041 (catalog repo init / expand) | FIXED | 3.10 |
| B042 (catalog subtree edit ops) | FIXED | 3.10 |
| B043 (SWAP command -- tab switching) | FIXED (retest 1.4/1.5) | 1.4, 1.5 |
| B044 (Local Files per-entry skip) | PARTIAL: per-entry skip FIXED (retest 3.2); junctions -> CR-NR-072 | 3.2, 3.2a |
| B045 (detach not wired) | OPEN | 1.9, 1.10, 1.11 |
| B046 (function keys) | PARTIAL: F7/F8 defaults FIXED (retest 7.3/7.3a); rest OPEN | 7.1-7.7, 7.3a |
| B047 (navigator file open resource-not-found) | FIXED (retest 3.3/3.12) | 3.3, 3.12 |
| B055 (Tab lands on status bar segments before menu/POM) | FIXED (retest 1.3a) | 1.3a |
| B057 (Theme Editor phantom Tab stop before selector combo) | VERIFIED (owner-confirmed) | 1.3d |
| B058 (CONFIG/Settings panel phantom Tab stop before Filter field) | FIXED (retest 1.3e) | 1.3e |
| B060 (Settings calendar off-screen but tabbable; calendar-on menu not visible/reachable in narrow workspace) | FIXED (CR-CH-026; retest 2.2a, 2.2b) | 2.2a, 2.2b |
| B065 (menu option description does not hang-indent when it wraps; continuation aligns under key column) | FIXED (retest 2.2c) | 2.2c |
| B066 (F-keys ignore the Command ===> field content; `1`+F9 runs bare SWAP instead of `SWAP 1`) | FIXED (retest 7.3b) | 7.3b |
| CR-CH-023 (unified tab-order: egui-native interior + shell boundary; chrome non-focusable) | IMPLEMENTED (retest 1.3a/1.3b/1.3c) | 1.3a, 1.3b, 1.3c |
| CR-NR-071 (CORE context help content) | PENDING GATE | 7.1 |
| CR-NR-072 (navigator junction/symlink expansion) | PENDING GATE | 3.2a |
| CR-CH-016 (END/RETURN from POM) | FIXED (retest 1.7/1.8/4.10/7.2) | 1.7, 1.8, 4.10, 7.2 |
| CR-NR-062 (View/Edit + context-menu commands) | PENDING GATE | 3.4, 3.5, 3.12, 4.1, 4.3, 4.9, 5.2 |
| CR-NR-066 (`cd` focuses navigator) | PENDING GATE | 3.13 |
| CR-NR-067 (configurable menu bar) | PENDING GATE | 2.5, 2.6, 2.7 |
| CR-NR-068 (Menu Workspace editor) | PENDING GATE | 2.4, 2.8, 2.9, 2.10, 2.11 |
| CR-NR-069 (Key Assignment editor) | PENDING GATE | (adds rows to Group 7 at its gate) |
| theme-and-appearance Req 17 (THEME command parity) | IMPLEMENTED (retest 8.1/8.2) | 8.1, 8.2 |
| CR-CH-024 (legacy theme consolidation + name-based THEME command) | IMPLEMENTED (retest 8.2, 8.5) | 8.2, 8.5 |
| CR-NR-070 (Theme Settings workspace + invoke-settings command) | PENDING GATE | 8.3, 8.4 |
| B048 (tests write real user config -- isolation hazard) | OPEN | (test-suite; no plan row) |
