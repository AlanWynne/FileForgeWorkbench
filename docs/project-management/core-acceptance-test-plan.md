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
| 1.1 | Launch the app. | Workbench opens with a POM workspace as the first tab; `Command ===>` line visible; status bar present. | startup-and-session 14.1 | [x] |
| 1.2 | Open a second workspace (type `2`/`FILES`, or `1`). | A new tab opens and becomes active; previous tab remains in the tab bar. | multi-tab-editor | [x] |
| 1.3 | Switch between tabs (click / keyboard). | Active tab changes; content swaps; no crash. | multi-tab-editor | [x] |
| 1.3a | At launch, press Tab repeatedly from the command field. | Focus enters the active Workspace's first control (POM: first enabled option), walks the interior in visual order, then the Menu_Bar, then wraps to the command field. Focus NEVER lands on a Status_Bar segment ("RUNNING", session start, "Ln x, Col y", encoding, line count, version label), the `SCROLL ===>` field, or a Key_Label_Bar F-key button. | B055 (FIXED); CR-CH-023 (AUTOMATED: full-shell egui_kittest tests + retest); menu-and-statusbar 16 | [x] |
| 1.3b | On a POM, press Tab from the command field through the whole cycle; then Shift+Tab back. | Command line -> each enabled POM option in order -> calendar `<` -> calendar `>` (calendar shown) -> Menu_Bar items -> wrap to command line. Disabled options are skipped. Shift+Tab reverses exactly. | CR-CH-023 (AUTOMATED: interior order + full-shell wrap/Shift+Tab harness tests; retest); menu-workspace 15; menu-and-statusbar 16 | [x] |
| 1.3c | Open the Menus Editor (`MENUS`); press Tab from the command field. | Command line -> Menu selector -> Title -> Show Calendar -> Group Headers -> Sep Space/Line/None -> per option (key -> Command -> Description -> Group -> on -> up -> down -> delete) -> Add Option -> Save -> New Name -> Save As -> Menu_Bar items -> wrap to command line. No SCROLL field or F-key button in the cycle. | CR-CH-023 (implemented, retest); menu-and-statusbar 16 | [x] |
| 1.3d | Open the Theme Editor (bare `THEME`); press Tab from the command field. | Command line -> Theme selector combo (NO invisible stop first) -> Set Active -> Reset to built-in -> (Confirm reset / Cancel when a reset is pending) -> New name field -> Copy -> Save As -> Save -> colour hex fields in order -> Menu_Bar items -> wrap to command line. No phantom/invisible focus stop between the command line and the Theme selector. | B057 (VERIFIED by owner); CR-CH-023 (AUTOMATED: full-shell egui_kittest test); menu-and-statusbar 16; theme-and-appearance 20.1 | [x] |
| 1.3e | Open the CONFIG panel (`CONFIG`); press Tab from the command field. | Command line -> Filter field (NO invisible stop first) -> the per-key widgets in order -> Menu_Bar -> wrap. No phantom focus stop between the command line and the Filter field. | B058 (FIXED, CR-CH-025/config-tab, AUTOMATED: `full_shell_config_first_tab_focuses_filter_field`); menu-and-statusbar 16; configuration-system 20 | [x] |
| 1.3f | Open the CONFIG panel (`CONFIG`), put focus in the key tree, and use the arrow keys (like the File Navigator tree): Up/Down, Right/Left, Enter, Home/End. | Down/Up move a visible selection highlight between namespace groups and keys (clamped at ends, no wrap); Right expands a collapsed group (its keys appear) or steps into the first child; Left collapses an expanded group or steps to the parent; Enter toggles a group or focuses the highlighted key's value widget; Home/End jump to first/last. Typing in the Filter field is never hijacked by the arrows. Behaviour matches the File Navigator tree. | B069 (FIXED); CR-CH-039 (DONE, AUTOMATED: `config_panel::tree::tests::*` 8 reducer tests + `shell::tests::full_shell_config_tree_arrows_navigate_and_expand`); configuration-system 21 | [ ] |
| 1.4 | `SWAP n` on the command line (e.g. `SWAP 2`); also `SWAP 0`/`SWAP 999`/`SWAP xyz`. | `SWAP 1` -> first tab, `SWAP 2` -> second; out-of-range/invalid shows a clear error and does not switch. | B043 (FIXED, retest); multi-tab-editor 18.1, 18.2 | [x] |
| 1.5 | `SWAP LIST` (and bare `SWAP` with no split). | A selectable list of open tabs pops up (`n: title`); clicking a row OR typing a number + Enter switches; Escape cancels. | B043 (FIXED, retest); multi-tab-editor 18.3-18.7 | [x] |
| 1.6 | Close a tab via its close control. | Tab closes; a sensible neighbour becomes active; POM cannot be lost (at least one tab remains). | multi-tab-editor 3.8 | [x] |
| 1.7 | END (F3) from a POM tab when other tabs are open. | Closes only that POM Workspace and navigates to another open Workspace; the app does NOT exit. | CR-CH-016 (FIXED, retest) | [x] |
| 1.8 | END (F3) from a POM tab when it is the LAST tab open. | The application terminates. | CR-CH-016 (FIXED, retest) | [x] |
| 1.9 | Detach a workspace via the tab right-click menu ("Move to Other View" / SPLIT / drag the tab out). | The action moves the workspace into a separate OS window (Detached Workspace) showing the tab's REAL interactive content (Title_Line + its OWN Command ===> field + content), not a placeholder; the tab's header is removed from the primary bar. | B045 (FIXED, AUTOMATED headless: full_shell_detach_sets_floating_and_records_floating_tab; real OS-window appearance MANUAL); CR-CH-035; menu-and-statusbar 18.1/18.2/18.4/18.8 | [ ] |
| 1.9a | In a detached window, type a command in ITS command line; and separately type in the PRIMARY window's command line. | Each command line is independent: a command typed in the detached window acts on the detached workspace only; a command in the primary window acts on the primary active tab only. No cross-window bleed. | B045 (FIXED, CR-CH-036; buffer isolation + correct-target dispatch AUTOMATED headless: with_workspace_context_saves_and_restores_primary_context, detached_command_acts_on_its_tab_not_the_primary; real OS window MANUAL); menu-and-statusbar 18.10 | [ ] |
| 1.10 | Detach by dragging a tab >20px outside the tab bar and releasing. | A new Detached Workspace is created at the release point; content intact. | B045; CR-CH-035; menu-and-statusbar 18.6 (drag-out MANUAL follow-up -- context-menu/SPLIT detach works today, row 1.9); layout-and-docking 3.9 | [B] |
| 1.11 | Re-dock the detached workspace via the DOCK command (or Shift+F2) in its own command line. | The workspace re-docks into the primary tab bar at its origin index (remove+reinsert, order preserved); content/cursor/modified state intact. DOCK on a non-detached workspace is a no-op with a status message. | B045; CR-NR-088; menu-and-statusbar 18.9/18.13; function-keys 15.2 (Shift+F2=DOCK). AUTOMATED headless (full_shell_dock_command_redocks_tab_at_origin / full_shell_dock_on_non_detached_is_noop_with_message); real OS window MANUAL | [ ] |
| 1.11e | Type `SPLIT` (side-by-side) or `SPLIT DOWN` (stacked) on the command line. | The Workspace area divides into TWO regions separated by a draggable splitter; each region shows a Workspace with its own tab bar; the new region opens a POM. Sizing is relative (proportion), not pixels. `SPLIT` again shows a "one split only" status (no nesting in this slice). | CR-NR-092 / B046 Slice 2b (DONE); layout-and-docking 13.1-13.4; fulfils menu-and-statusbar 19.11. AUTOMATED headless (full_shell_split_creates_two_groups_second_is_pom_and_focused, full_shell_split_down_is_vertical, full_shell_second_split_is_rejected_with_status, full_shell_split_renders_two_regions_and_command_acts_on_focused) | [ ] |
| 1.11f | With the screen split, move focus between the two regions (`FOCUS`/`FOCUS OTHER` or its key), then type a command / open a tab. | The focused region is visibly highlighted; the command / new tab acts on the FOCUSED region only, not the other. | CR-NR-092 / B046 Slice 2b (DONE); layout-and-docking 13.6-13.8; fulfils menu-and-statusbar 19.12. AUTOMATED headless (full_shell_focus_flips_focused_group, full_shell_split_renders_two_regions_and_command_acts_on_focused); the highlight-border pixels are MANUAL | [ ] |
| 1.11g | Drag the splitter between the two regions. | The regions resize by relative proportion; neither shrinks below ~100 px in the split direction. | CR-NR-092 / B046 Slice 2b (DONE); layout-and-docking 13.5. Proportion clamp + min-region math AUTOMATED (set_split_proportion_clamps); the pixel drag GESTURE is MANUAL | [ ] |
| 1.11h | Type `UNSPLIT` (or END on a split, or close the last tab of one region). | The split collapses back to a single region, preserving the surviving region's tabs and active tab. | CR-NR-092 / B046 Slice 2b (DONE); layout-and-docking 13.9; fulfils menu-and-statusbar 19.14. AUTOMATED headless (full_shell_unsplit_collapses_preserving_survivor, full_shell_end_while_split_collapses, closing_last_tab_of_a_group_auto_collapses) | [ ] |
| 1.11k | With the screen split into two regions, look at each region's top area; then compare an UNSPLIT single workspace. | EACH split region shows the FULL chrome of the workspace placed in it -- its own menu bar, Title_Line, and `Command ===>` field (in that order) above its content -- there is NO single shared top menu bar or command line spanning the whole window while split. When UNSPLIT, the single workspace's menu bar + Title_Line + command line occupy the top of the window exactly as before (unchanged). Detaching that workspace shows the SAME chrome in its own window. | CR-CH-041 (DONE); layout-and-docking 16.2/16.3/16.4; workspace-kinds 4 (per-Kind menu bar in-region); menu-and-statusbar 16.15. AUTOMATED headless (full_shell_split_suppresses_app_level_menu_bar_and_title_unsplit_restores; full_shell_split_region_menu_bar_is_live_and_focusable; full_shell_placement_tracks_split_region_then_detached); pixel placement + OS window frame MANUAL | [ ] |
| 1.11a | In a detached window that is drilled into a sub-context, press the window Close button (X) or F4 (RETURN). | RETURN takes the workspace to its POM and the window STAYS open showing the POM; a second X/RETURN (now a POM) closes the workspace and the window closes. END (F3) instead walks back one navigation step. F-keys work in the detached window exactly as in a docked one. | B045, B068; CR-CH-037, CR-CH-038; menu-and-statusbar 18.3/18.11; menu-workspace 14.10. AUTOMATED headless (RETURN semantics + F-key-in-context: return_from_*, detached_function_key_return_acts_on_its_tab); real OS-window close gesture MANUAL | [ ] |
| 1.11b | In a docked workspace: RETURN (F4) from a non-POM; RETURN from a POM; END (F3) from a drilled-in context. | RETURN from a non-POM jumps to the POM; RETURN from a POM closes that workspace (exits the app if it was the last); END pops one navigation level (closes the workspace at its root). | CR-CH-038; menu-workspace 14.10. AUTOMATED headless (return_from_non_pom_navigates_to_pom, return_from_pom_with_other_tabs_closes_pom_not_app, end_at_empty_stack_last_tab_exits) | [ ] |
| 1.12 | Exit and relaunch. | Session restores tabs/active workspace; POM present. | startup-and-session 14.1b | [ ] |
| 1.13 | Run with the log directory unwritable (or force a log-buffer overflow) and view the status bar. | When logging has DEGRADED (fallback mode because the log file could not be created, or dropped-record count > 0), the status bar shows a logging-degradation indicator (e.g. `LOG!`); when logging is healthy nothing is shown. | B038 (FIXED, AUTOMATED: full-shell egui_kittest status-bar indicator present-when-degraded / absent-when-healthy); logging-subsystem 8.7 | [ ] |

---

## Group 2 -- Menu workspaces, Settings, menu bar, Create/Edit-Menu (Core #2)

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 2.1 | View the POM. | Rendered as a menu workspace: rows of option / command / description. | menu-workspace | [ ] |
| 2.1a | Look at the POM and the Settings menu titles/headings and their tab headers. | Exactly ONE title is shown per menu workspace, sourced from the menu configuration file's `title` and CENTERED consistently (POM and Settings follow the SAME format -- no POM-centered-vs-Settings-left mismatch, no duplicated app-banner + menu-title heading). The POM tab header is a SHORT form (e.g. `POM`), like other workspace tabs; the POM is addressable via a `POM` command (alias `START`). | CR-CH-042 (PENDING GATE -- de-dup title, centered, config-driven; short POM tab; POM command alias START); menu-workspace; menu-and-statusbar 17; workspace-kinds Req 3 | [B] |
| 2.2 | Open the Settings menu (`SETTINGS` / `0` / `=0`). | Opens as a POM-modelled menu workspace (option/command/description rows), NOT the flat custom panel. Options: A Config, T Theme, M Menus (Core); R Reset BARE (Recovery). | B032 (FIXED, CR-CH-025, AUTOMATED: `settings_command_opens_menu_workspace_not_flat_panel`); menu-workspace 11.2 | [ ] |
| 2.2a | Open the Settings menu; press Tab from the command line to the end of the option list and beyond. | No calendar is shown (Settings defaults `show_calendar = false`). Tab walks ONLY the option rows, then the Menu_Bar -- there are NO extra "invisible" Tab stops after the last option. | B060 (FIXED, CR-CH-026, AUTOMATED: `default_settings_toml_hides_calendar`, `full_shell_settings_tab_walks_options_only_no_calendar_stops`); menu-workspace 16.1, 16.4 | [ ] |
| 2.2b | Edit a menu (or Settings) to set `show_calendar = true`, open it in a normal and in a NARROW window. | Normal window: the calendar is VISIBLE on the right and its `<`/`>` month buttons are reachable by Tab and on-screen. Narrow window (too small to fit): the calendar is omitted (not drawn off-screen) and there are NO calendar Tab stops. In neither case is there an invisible/off-screen focus stop. | B060 (FIXED, CR-CH-026, AUTOMATED at render level: `menu_calendar_shown_when_wide_next_button_is_on_screen`, `menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops`; live narrow-window resize is MANUAL); menu-workspace 16.2, 16.3, 16.5, 16.6 | [ ] |
| 2.2c | Open any menu (POM/Settings); shrink the window until a long option description wraps to a second line. | The wrapped continuation of the description hangs-indents under the START of the description column (under the first description character), NOT under the key column at the row's left edge. First-line column alignment (key \| command \| description) is unaffected. | B065 (FIXED, 2nd attempt -- description is a separate wrapping Label to the right of the key+command prefix Button, so it hang-indents under the description column; AUTOMATED at unit level: `option_prefix_job_holds_fixed_columns_and_does_not_wrap`, `option_prefix_jobs_share_width_across_rows`; the pixel-exact wrap alignment is MANUAL/visual); menu-workspace 2.1a | [ ] |
| 2.2d | On a `show_calendar = true` menu, start WIDE then progressively narrow the window. | WIDE: descriptions sit on ONE line at their natural width; the calendar is to the RIGHT of the option column (with room for the option-list scrollbar if needed); any extra width is blank space to the RIGHT of the calendar (calendar NOT pinned to the edge). MEDIUM (one-line descriptions no longer fit alongside the calendar): the CALENDAR HIDES and its width goes to the descriptions, which stay on ONE line. NARROW (one-line descriptions still do not fit even with the calendar hidden): descriptions wrap to a second line (last resort, hang-indented per 2.2c). The transition order is always calendar-hide BEFORE description-wrap. | CR-CH-032 (description-driven layout, DONE; AUTOMATED at render level: natural-width helper unit tests + 3-tier egui_kittest harness tests `menu_wide_shows_calendar_and_one_line_descriptions`/`menu_medium_hides_calendar_keeps_one_line`/`menu_narrow_hides_calendar_and_wraps`; live resize is MANUAL); menu-workspace 16.7-16.11 | [ ] |
| 2.3 | In the Settings menu, select an option by its key. | Routes to the mapped target (same as typing it on the command line). | menu-workspace 3.x | [ ] |
| 2.3a | On the POM, CLICK option 0 "Settings" (mouse). | The Settings menu opens (option/command/description rows: A Config, T Theme, M Menus, K Keys, W Kinds, R Reset), IDENTICAL to typing `SETTINGS` / `=0`. It must NOT open a generic/empty Menu Workspace. | B075 (FIXED -- shared `open_named_menu` router; AUTOMATED headless: `clicking_pom_settings_option_opens_settings_menu_in_place`); menu-workspace 2.1e/2.1i, 11 | [ ] |
| 2.4 | Type the command for a Settings option on the command line + Enter (e.g. `CONFIG`, `THEME`, `MENUS`), and the chained form `SETTINGS T`. | Switches to that option/namespace -- every Settings option has a matching command; `SETTINGS T` opens Settings then activates `T` (THEME). | B032 (FIXED, CR-CH-025, AUTOMATED: `settings_t_chains_to_theme_editor`, `config_command_is_registered`); CR-CH-025 | [ ] |
| 2.4a | From ANY workspace, type an `=` fastpath jump on the command line + Enter (e.g. `=0.K`). | The `=` pops to the POM origin, then the path walks option-by-option: `=0.K` = POM option 0 (Settings) then option K (KEYS) -> lands on the Keys Workspace. No "command not yet implemented" error. Bare dotted `2.1` still works; ordinary dotted text (`abc.def`) is not hijacked. | B061 (FIXED, AUTOMATED: `chained_fastpath_equals_zero_dot_k_opens_keys_workspace`, `chained_fastpath_pops_to_pom_origin_from_non_pom`); menu-workspace Req 5 | [ ] |
| 2.4b | From a Files/Catalogs workspace (or any non-POM Context), type an option-key / `=<key>` jump that switches the workspace to a DIFFERENT Context in place (e.g. `=0` to Settings), then look at the tab header AND the Title_Line. | Both the tab header and the Title_Line show the NEW Context's label (e.g. `[SETTINGS]`), never the stale previous label (e.g. `files`). The header is derived from the live Context, so it can never drift from the content. | B050 (FIXED, AUTOMATED: menu-workspace in-place-switch header-freshness regression test); menu-and-statusbar Req 17.10; CR-CH-034 | [F] |
| 2.4c | Open the Catalog Explorer (POM option 1 / CATALOGS) and, separately, the File Explorer / navigator; look at each tab header + Title_Line. | Each Workspace Kind shows its OWN correct title: the Catalog Explorer reads `[CATALOGS]`, the File Explorer reads `[FILES]` -- they no longer share the `[FILES]` label. Titles derive from each Kind's configured title (Kind registry). | CR-NR-090 B.1 (DONE); workspace-kinds Req 2.4/3.1 (title-from-Kind-config, distinct built-in titles). AUTOMATED headless (kind_title_derives_from_registry_and_user_override_wins, builtin_default_titles_distinguish_catalogs_from_files) | [ ] |
| 2.5 | Inspect the application menu bar. | The bar entries align with what is actually available (no dead entries). | CR-NR-067 (pending) | [B] |
| 2.6 | Open the menu bar AS a Menu Workspace. | The menu bar is itself a named Menu Workspace that can be opened and viewed like the POM. | CR-NR-067 (pending) | [B] |
| 2.7 | Give a workspace its own named menu bar; open another with none. | A workspace uses its named menu bar; a workspace with none falls back to the Primary menu bar. | CR-NR-067 (pending) | [B] |
| 2.7a | Configure a Workspace Kind's menu bar and key list (via a workspace-kinds/<kind>.toml override), open a Workspace of that Kind, and open a Kind with no override. | The configured Kind renders ITS menu bar and uses ITS key list; a Kind with no override falls back to the default menu bar and its base key-map context (behaviour unchanged). | CR-NR-090 B.2 (DONE); workspace-kinds Req 4. AUTOMATED headless (menu_bar_uses_kind_menu_bar_else_default, key_list_context_uses_kind_key_list_else_base); real menu-bar visual + live keystroke MANUAL | [ ] |
| 2.7b | Open the Kinds Editor (KINDS command or Settings entry): select a Kind, edit its title/menu bar/key list/profile and Save; create a "New Kind modelled on" a built-in base; then open a Workspace of the reconfigured/new Kind. | The editor lists the Kinds and edits the selected one; Save writes workspace-kinds/<name>.toml and the change is live (title/menu/keys/profile reflect it without restart); a new Kind is a copy of its base that can differ; RESET BARE restores the compiled default Kinds. | CR-NR-090 B.4 (DONE); workspace-kinds Req 6, 7. AUTOMATED headless (open_kinds_editor_activates_kinds_editor_context, kinds_editor_save_writes_file_and_reloads_registry, kinds_editor_new_kind_seeds_copy_modelled_on_base, full_shell_kinds_first_tab_focuses_first_interior, reset_bare_restores_builtin_kind_registry); dialog pixel layout MANUAL | [ ] |
| 2.8 | Open the Menu Workspace editor (create/edit/delete menus). | A workspace opens for creating/editing/deleting Menu Workspaces (POM, Settings, custom, menu bars): a Name field + an editable list of option/command/description rows. | CR-NR-068 (pending); Core #2 KEY gap | [B] |
| 2.9 | Create a menu "MyTools" with 2-3 option/command/description rows and Save. | The menu is saved as a menu definition file; no error. | CR-NR-068 (pending) | [B] |
| 2.10 | Open the newly-saved menu. | Renders as a menu workspace with the entered rows; selecting an option runs its command. | CR-NR-068 (pending) | [B] |
| 2.11 | Edit an existing menu (add/remove a row) and Save. | Changes persist and re-render. | CR-NR-068 (pending) | [B] |
| 2.12 | Open the Menus editor (`MENUS`), select a menu, and type into an option's Key, Command, Description, and Group fields. | Every field accepts typed input and the edits persist across frames (the field is not frozen); Key uppercases on blur. | B054 (FIXED, AUTOMATED: `menus_editor_command_and_description_fields_accept_typed_input`, `menus_editor_every_stable_control_is_tab_reachable`) | [ ] |

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
| 4.8a | Save when the underlying storage cannot flush/fsync (simulated durability failure). | The save does NOT silently report success on a non-durable write: the atomic save aborts and leaves the original target intact (temp cleaned up), and the failure is logged; a backup-creation failure is logged (WARN) but does not abort the save. | B034 (FIXED, AUTOMATED at unit level via mock-injected fsync failure: atomic-abort / direct-warn / backup-warn tests); file-operations 7.10-7.12 | [ ] |
| 4.8b | Configure an EDITOR-based Workspace Kind's profile (e.g. CAPS On, line endings) via a workspace-kinds/<kind>.toml override, then open a new/loaded editor Workspace of that Kind. | The editor opens with the Kind's edit-profile defaults applied (CAPS On); a NEW buffer takes the Kind's line-ending default; a LOADED file keeps its detected line endings; a later per-tab toggle is not clobbered. A built-in Kind (no override) opens unchanged. | CR-NR-090 B.3 (DONE); workspace-kinds Req 5. AUTOMATED headless (new_editor_takes_kind_edit_profile_on_open, new_buffer_takes_kind_line_end_mode); tab_size deferral documented | [ ] |
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
| 5.7 | Run a command with a mixed-case argument (e.g. `FIND 'MixedCase'`, `theme Default Legacy`, `find 'Error'` case-sensitive). | The VERB matches case-insensitively but the ARGUMENT reaches the handler with its ORIGINAL case (search strings, theme names preserved); a lowercase verb still resolves. | B062 (FIXED, AUTOMATED: `verb_arg_matches_case_insensitively_and_preserves_argument`, `command_arguments_preserve_case_in_history`, `parse_two_args_preserves_argument_case`, `mixed_case_theme_verb_and_argument_resolve`) | [ ] |

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
| 7.3a | Set the `SCROLL ===>` amount to MAX (type `SCROLL MAX`, or `M`/`MAX` where the field accepts it), then press UP / DOWN (or F7/F8). | Scrolls to the top / bottom of the document (the active SCROLL amount governs bare UP/DOWN, ISPF-style); with SCROLL PAGE it is one page, HALF a half page, n = n lines. MAX is a SCROLL-field amount, not an F-key binding. NOTE: currently bare UP/DOWN always scroll one page regardless of the SCROLL amount (nav_manager up_page/down_page ignores scroll_amount), so MAX-then-UP does NOT reach the top yet. | CR-NR-087 (PENDING GATE -- SCROLL-amount-aware UP/DOWN); navigation-commands Req 3.1/3.3; B046 | [F] |
| 7.3b | Type a value in the `Command ===>` field then press an F-key bound to a command: e.g. `1` + F9 (SWAP), `2` + F9, `8` + F8 (DOWN); and press the F-key with an EMPTY field. | The F-key invokes its bound command with the field content as the argument -- observably identical to typing `<command> <field>` + Enter: `1` + F9 -> swap to workspace 1, `2` + F9 -> workspace 2, `8` + F8 -> page down 8. F9 with an empty field swaps to the previously active workspace (bare SWAP). It does NOT run bare `SWAP` (tab picker) when a value was typed. After the F-key command runs, the `Command ===>` field is CLEARED (e.g. `1` does not remain after `1` + F9) -- identical to `<command> <field>` + Enter -- EXCEPT when the command replaces the field (RETRIEVE recalls into it; see 7.8). | B066 (FIXED); CR-CH-033 (field cleared after key-forward unless replaced; AUTOMATED: `key_command_clears_command_field_after_success`, `key_command_retrieve_keeps_recalled_field`); command-framework Req 9.8/9.1/9.7/9.9/9.10; multi-tab-editor 18.1 | [ ] |
| 7.3d | Press F2 (DETACH) on a Workspace; then DOCK (Shift+F2) or type DOCK in the detached window. | F2 detaches the current Workspace into its own OS window (formerly the SPLIT verb); DOCK / Shift+F2 re-attaches it at its origin position. `SPLIT DETACH` typed on the command line still detaches (deprecated alias). Bare `SPLIT` does nothing yet (reserved). | CR-CH-040 / B046 Slice 1 (GATED); menu-and-statusbar 18.14 | [B] |
| 7.4 | Type `SPLIT` (or press its future key) to divide the Workspace area into two in-window regions at a relative proportion. | The Workspace area splits into two side-by-side (or stacked) regions, each rendering a Context, sized by relative proportion (not pixels) with a draggable splitter -- VS-Code-style editor groups. | menu-and-statusbar 19.11 (revised); Slice 2 (future CR, ff-layout TabGroupTree); B046 | [B] |
| 7.5 | With the Workspace split (Slice 2), move focus between the two regions. | Focus moves between the two split regions. | menu-and-statusbar 19.12 (revised); Slice 2 (future CR); B046 | [B] |
| 7.6 | With the Workspace split (Slice 2), issue END to unsplit. | The split collapses, restoring the single-region view. | menu-and-statusbar 19.14 (revised); Slice 2 (future CR); B046 | [B] |
| 7.7 | Confirm each remaining PF binding in the spec fires its action. | Every PF key mapped in the function-keys spec performs its action in-context. | function-keys-and-history; B046 | [F] |
| 7.8a | With focus on the `Command ===>` field, press the Up arrow, then the Down arrow, after entering a few commands. | Up recalls the previous (older) command-history entry into the field one at a time; Down recalls the next (newer) entry; same history as RETRIEVE/F12. Arrows drive history ONLY while the command field has focus (they do NOT scroll the Context body -- that is PF7/PF8 / SCROLL); the RETRIEVE pointer stays consistent. | CR-NR-096 (PENDING GATE -- Up/Down arrow command history); function-keys-and-history Req 6; CR-NR-084 | [B] |
| 7.8 | Type a command and Enter; then press F12 (RETRIEVE) with the field EMPTY, and again with some text already in the field (as often happens in a freshly opened/navigated workspace). | F12 recalls the most recent command into the `Command ===>` field WITHOUT executing it, in BOTH cases (empty and non-empty field); repeated F12 walks older entries; a non-RETRIEVE submission resets the pointer. F12 does not error and does not pollute history with `RETRIEVE ...`. | B067 (FIXED, AUTOMATED: `retrieve_via_key_with_nonempty_field_recalls_previous_command`, `retrieve_list_via_key_opens_history_overlay`); function-keys-and-history Req 19.1-19.7, 22.2 | [ ] |

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

## Group 9 -- Configuration recovery: RESET BARE and profiles (Core)

RESET BARE and Application_Profiles are CORE recovery behaviour. Covers B064
(profile-aware config + active-profile reset) and CR-NR-083 (targeted / ALL).

| # | Step | Expected result | Req / Backing | Result |
|---|------|-----------------|---------------|--------|
| 9.1 | Launch under a profile (`ffwb -p ispf`), change the theme, run `RESET BARE`, confirm the dialog. | The dialog names the active profile; on confirm the ACTIVE profile's config is archived and its theme resets to the Default Legacy baseline; other profiles are untouched; config is isolated per profile (`profiles/<name>/`). | B064 (FIXED, AUTOMATED: `execute_reset_bare_resets_theme_to_default_legacy`, `paths::tests::user_config_path_is_profile_aware`) | [ ] |
| 9.2 | `RESET BARE <profile1> [<profile2> ...]` (a subset, including and excluding the active profile) and an unknown name. | Each named profile is resolved at `profiles/<slug>/` (case-insensitive, de-duplicated) and archived; the in-memory shell resets ONLY if the active profile is in the list; an unknown name errors with no dialog (all-or-nothing). | CR-NR-083 (AUTOMATED: `reset_bare::tests::resolve_named_*`, `reset_bare_unknown_named_profile_errors_without_dialog`) | [ ] |
| 9.3 | `RESET BARE ALL`. | The confirmation dialog enumerates every profile (default base + each `profiles/<slug>/`); on confirm all are archived+reset. | CR-NR-083 (AUTOMATED: `reset_bare::tests::resolve_all_*`, `enumerate_profiles_under`) | [ ] |

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
| B045 (detach) | CR-CH-035: real immediate-viewport content + faithful redock + 80-char title + unified 16-limit + drag-out gesture. CR-CH-036 (in progress): independent per-window command context (own command line, dispatch targets its tab, bidirectional isolation). AUTOMATED headless; real OS window MANUAL | 1.9, 1.11 (retest); 1.9a [B] independent cmd line; 1.10 drag-out |
| B046 (function keys) | PARTIAL: dispatch works + F7/F8 defaults FIXED (UP/DOWN); remnants parcelled to CRs -- configurable keylist = CR-NR-069, F1 help content = CR-NR-071, SCROLL-amount-aware UP/DOWN (MAX modifier) = CR-NR-087, PF2/PF9/PF3 split = menu-and-statusbar 11-14 | 7.1-7.8, 7.3a |
| B047 (navigator file open resource-not-found) | FIXED (retest 3.3/3.12) | 3.3, 3.12 |
| B055 (Tab lands on status bar segments before menu/POM) | FIXED (retest 1.3a) | 1.3a |
| B057 (Theme Editor phantom Tab stop before selector combo) | VERIFIED (owner-confirmed) | 1.3d |
| B058 (CONFIG/Settings panel phantom Tab stop before Filter field) | FIXED (retest 1.3e) | 1.3e |
| B060 (Settings calendar off-screen but tabbable; calendar-on menu not visible/reachable in narrow workspace) | FIXED (CR-CH-026; retest 2.2a, 2.2b) | 2.2a, 2.2b |
| B065 (menu option description does not hang-indent when it wraps; continuation aligns under key column) | FIXED 2nd attempt (retest 2.2c -- 1st attempt did not work at runtime) | 2.2c |
| CR-CH-032 (description-driven menu layout: calendar hides before descriptions wrap; extra width trails right of the calendar) | DONE (retest 2.2b, 2.2d) | 2.2b, 2.2d |
| B066 (F-keys ignore the Command ===> field content; `1`+F9 runs bare SWAP instead of `SWAP 1`) | FIXED (retest 7.3b) | 7.3b |
| CR-CH-033 (Command ===> field not cleared after a key-forwarded command; `1` remains after `1`+F9) | DONE Slices 1-2 (retest 7.3b) | 7.3b |
| B067 (F12 RETRIEVE broken when field non-empty; regression from B066's merged `RETRIEVE <field>`) | FIXED (retest 7.8) | 7.8 |
| CR-CH-023 (unified tab-order: egui-native interior + shell boundary; chrome non-focusable) | IMPLEMENTED (retest 1.3a/1.3b/1.3c) | 1.3a, 1.3b, 1.3c |
| CR-NR-071 (CORE context help content) | PENDING GATE | 7.1 |
| CR-NR-072 (navigator junction/symlink expansion) | PENDING GATE | 3.2a |
| CR-NR-087 (SCROLL-amount-aware bare UP/DOWN, B046 MAX modifier) | PENDING GATE | 7.3a |
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
| CR-CH-041 (instance owns chrome; region is placement; per-instance chrome incl. Kind menu bar renders in-region; placement derived from layout snapshot; core has one tab system) | PENDING GATE | 1.11k |
| B075 (clicking POM "Settings" opens a generic Menu Workspace instead of the Settings menu) | FIXED (shared open_named_menu router; retest 2.3a) | 2.3a |
| CR-CH-042 (de-dup title -> single config-driven centered title; short POM tab; POM command alias START) | PENDING GATE | 2.1a |
| CR-NR-096 (Up/Down arrows step command history on the focused command line) | PENDING GATE | 7.8a |
