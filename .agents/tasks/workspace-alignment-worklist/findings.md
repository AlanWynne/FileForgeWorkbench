# Workspace-alignment worklist (read-only investigation, main 8ff3e3b)

## 1. Summary answer

The built-in menu tree is small and fully code-only. There are THREE compiled
menus, all in `crates/ff-desktop/src/menu_workspace/defaults.rs`:
`DEFAULT_POM_TOML` (4 options), `DEFAULT_SETTINGS_TOML` (7 options),
`DEFAULT_SCRM_MENU_TOML` (6 options). Only POM > 0 Settings and POM > S SCRM
are sub-menus. Walking depth-first in on-screen order gives 3 Menu-Kind
screens, 6 distinct workspaces/Contexts reached through Settings
(Config, Aliases, Theme, Menus, Keys, Kinds), 1 modal dialog (Reset bare),
Help, 5 SCRM function options, and the SCRM replay viewer (reached via
CAPTURE REPLAY). 8 further workspaces exist but are typed-command only.

Key facts for planning:
- Menu Kind is ONE kind (POM, Settings, SCRM and user menus all use
  `TabKind::MenuWorkspace` + `menu_workspace_screen_model_full`), so ONE Menu
  Kind gate covers all three menus.
- Only Alias is built on the shared `ispf_table_editor`. Spec says Menus and
  Kinds will consume it next (ispf-table-editor Req 9, "exactly Alias, Menu,
  Kinds"). Keys and Theme are table-ish but NOT in that spec.
- Every Context except Editor/Untitled (parked) and the Menu Kind has a
  `*_screen_model` builder. The Menu Kind has its own provider
  (`menu_workspace_screen_model_full`). Editor is the one documented opt-out.
- `impl WorkspaceContext` exists for: Config, Theme, Menus, Keys, Kinds, Alias,
  Command Configurator, Plugin Manager, Macro Library, Event Log, Search
  Results, SCRM Viewer, Help. NOT for: MenuWorkspace, Files panel, File
  Explorer, Editor (these use `apply_interior_focus`, the documented variance
  in workspace-conformance.md).
- No option fails to resolve (see section 4). The two suspicious rendering
  items are label/description duplication (section 5).

## 2. Evidence: how options resolve

Front door: `shell/dispatch.rs::dispatch_command_string` = `=` strip ->
locale verb rewrite -> `run_command_prelude` (incl. current-menu Option_Key,
`try_current_menu_option` in `shell/commands_menu.rs`) -> `ff_command::resolve_target`
(`ff-command/src/command_target.rs`; order: user command, builtin workspace
target, registered Command_Id, menu name, macro) -> `run_command_ladder`.
Builtin classifier: `command_config/mod.rs::builtin_workspace_target_for`.
Menu names resolvable: `BUILTIN_MENU_NAMES = ["pom","settings","scrm"]` plus any
`<menus_dir>/<name>.toml` (`ShellTargetResolver::is_resolvable_menu`).
`CustomWorkspace` kinds are dispatched in `shell/target_dispatch.rs`.
Menu placement is owned by `commands_menu.rs::open_menu_by_name`: pom = Home,
settings = IN PLACE (navigation stack), anything else (scrm, user menus) = NEW
TAB via `open_menu_workspace_tab`; scrm is seeded from `recovery_scrm_menu()`
when no user file exists.

Verbs that are NOT classified by `resolve_target` fall to the ladder (no
`CommandTarget`): `Help`/`HELP SCRM` (`commands_ladder_a.rs` ~188),
`X` (`commands_ladder_a.rs` ~317), `Reset bare` (`commands_ladder_b.rs` ~95),
`SNAPSHOT` and `CAPTURE ...` (`commands_ladder_b.rs` ~66, ~86). Nothing is
registered as a Command_Id except file.open, file.exit, menu.open, config.open
(`shell/construct_commands.rs`), so none of the menu verbs is a `Function`
target. `X` with key==command is deliberately skipped by the self-reference
guard in `try_current_menu_option`, so the ladder `X` close verb runs.

## 3. THE WORKLIST (depth-first, on-screen order)

Legend: WC = implements WorkspaceContext; SP = snapshot provider / screen model;
TS = table-style (would use shared ispf_table_editor). Target types: Menu,
Custom(kind), Ladder (falls through resolve_target to run_command_ladder).

| # | Menu path | Option | Command | Target type | Kind / TabKind (or function) | Spec folder | WC | SP | TS | Known issues / notes |
|---|-----------|--------|---------|-------------|------------------------------|-------------|----|----|----|----------------------|
| 1 | POM (Home) | - | - | Menu `pom` (open_menu_by_name -> Home in place) | MenuWorkspace (is_home), KindTag::MenuWorkspace | menu-workspace (+ screen-snapshot-scrm Req 22/25/30) | n | y (`menu_workspace_screen_model_full`) | n | MENU KIND GATE (covers #1, #2, #12). Pom-snapshot-review findings: key line above status bar and inside content; command line shows triggering command and underscores short of border (menu path still uses old helper); no menu golden; description repeats command in brackets `(Settings)`; align `:` column; add `Option|Command|Description` md table to clipboard; calendar column position; Zoom segment + snapshot confirmation message PENDING owner decision. Also owner msg 2: `Last: --:--` must never show; set at startup and refresh on every screen transition. |
| 2 | POM > 0 | 0 Settings | `Settings` | Menu `settings` (IN PLACE via open_settings_menu) | MenuWorkspace (Settings), same Kind as #1 | menu-workspace, configuration-system | n | y (same provider) | n | Description "FFWB Settings and Client Parameters". `show_calendar=false`, `group_separator="line"`, title "Settings". Shares gate with #1; extra check: no calendar column, Core/Recovery group separator line. |
| 2.1 | POM > 0 Settings > C | C Config | `Config` | Custom(`config`) | ConfigPanel (WorkspaceKind::Config) | configuration-system | y | y (`config_panel_screen_model`, `screen_providers_config.rs`) | n (tree: `[Group] : expanded`, `key : value`) | Content uses aligned `:` rows (layout.rs says no `. . .` leaders; recorded only, owner says "dot-leader rows": the aligned-colon rows are what exist now). Filter + "Unsaved edits" header rows are body_lines; owner decision pending on whether to stay as-is. |
| 2.2 | POM > 0 Settings > A | A Aliases | `Aliases` | Custom(`aliases`) | AliasEditor (CommandConfigurator descriptor, editor=aliases) | command-configurator (Req 5,7), ispf-table-editor | y | y (`alias_editor_screen_model`) | y (ONLY consumer of ispf_table_editor today) | PARKED gate: branch `alias-kind-gate`, worktree `.worktrees/alias-kind-gate`, docs-only, CR-NR-116, summary `.agents/tasks/alias-kind-gate/gate-summary.md`; created off a6aacae, needs rebase onto 8ff3e3b (merge-base not re-verified by me; git probe was blocked by the guard hook). Waiting on owner decisions (a)-(g) and 5 framework flags. Owner snapshot shows Property/Value table + nested box + dot-leader `SS . . . : SNAPSHOT`: all addressed in that gate. |
| 2.3 | POM > 0 Settings > T | T Theme | `Theme` | Custom(`theme_editor`) (bare only; `THEME <arg>` falls to ladder) | ThemeEditor | theme-and-appearance | y | y (`theme_editor_screen_model`) | partial (token Setting/Value table; NOT in ispf-table-editor spec) | Gate-summary lists Theme among models whose framed tables change appearance. Live preview of working copy; ISPF save convention (SAVE/END/RETURN) applies. |
| 2.4 | POM > 0 Settings > M | M Menus | `Menus` | Custom(`menus`) | MenusEditor | menu-workspace (Req 13), ispf-table-editor | y | y (`menus_editor_screen_model`) | y (planned ispf consumer; today Up/Down/Delete buttons) | Next ispf_table_editor consumer after Alias. Edits `menus/<name>.toml`; this is how user menus are authored. |
| 2.5 | POM > 0 Settings > K | K Keys | `Keys [kind]` | Custom(`keys`) (arg `kind` case-preserved) | KeysEditor | function-keys-and-history (Req 22) | y | y (`keys_editor_screen_model`) | partial (one row per F-key; not in ispf spec) | Decide at gate time whether Keys joins the shared table editor (spec says exactly Alias/Menu/Kinds; a 4th consumer is "additive, out of scope"). |
| 2.6 | POM > 0 Settings > W | W Kinds | `Kinds` | Custom(`kinds`) | KindsEditor | workspace-kinds (Req 6), ispf-table-editor | y | y (`kinds_editor_screen_model`) | y (planned; today a vertical form) | Planned ispf consumer. Drives Kind title/menu-bar/key-list/profile, so edits affect the common frame of every gate. Mnemonic `W` is irregular (K taken by Keys). |
| 2.7 | POM > 0 Settings > R | R Reset bare | `Reset bare` | Ladder (`RESET BARE`) | NO workspace: opens confirm dialog (`reset_bare_confirm`) | configuration-system (Req 19) | n/a | y (dialog capture via `active_dialog_capture`, screen-snapshot-scrm Req 27) | n | Group "Recovery". Dialog, not a Workspace: gate = confirm-dialog snapshot only. Unknown named profile -> error, no dialog (Req 19.12). |
| 3 | POM > 3 | 3 Help | `Help` | Ladder (`HELP`) -> `open_help` -> `enter_help_context` (pushes on active tab) | HelpContext (KindTag::HelpContext) | context-help | y (`HelpContextPanel`) | y (`help_context_screen_model`; explicit title "Help", not Kind display title) | n | Help snapshot title is hard-coded "Help" rather than the Kind display-title path (screen_providers.rs ~259): check against common frame title rule. Description uses " -- " separator. |
| 4 | POM > S | S SCRM | `SCRM` | Menu `scrm` (NEW TAB; seeded from `recovery_scrm_menu()`) | MenuWorkspace (SCRM menu), same Menu Kind | screen-snapshot-scrm (Req 22, 25) | n | y (menu provider) | n | Covered by the Menu Kind gate (#1). Bar-visible (echoes onto menu bar). Description "SCRM : Screen Capture and Recording" repeats the label (see section 5). Opens in a NEW tab, unlike Settings (in place): inconsistent with "Settings in place" but by design (open_menu_by_name). |
| 4.1 | POM > S > S | S Snapshot | `SNAPSHOT` | Ladder | none (function: `handle_snapshot`, clipboard) | screen-snapshot-scrm Req 4 | n/a | n/a | n | Function only; verify via clipboard flavors (picture + text + markdown). Snapshot confirmation message in Message_Line is a PENDING owner decision (pom-snapshot-review). |
| 4.2 | POM > S > CS | CS Capture start | `CAPTURE START` | Ladder | none (function: `handle_capture`) | screen-snapshot-scrm Req 7-9 | n/a | n/a | n | Function only. |
| 4.3 | POM > S > CT | CT Capture stop | `CAPTURE STOP` | Ladder | Save-As dialog (Req 24) | screen-snapshot-scrm Req 24 | n/a | dialog capture | n | Dialog, not a Workspace. |
| 4.4 | POM > S > CR | CR Capture replay | `CAPTURE REPLAY` | Ladder | saved-session picker dialog then ScrmViewer (`open_scrm_viewer_tab`) | screen-snapshot-scrm Req 10, 16 | y (`ScrmViewerState`) | y (`scrm_viewer_screen_model`, title "Screen Replay") | n | Workspace reached only via this option or typed. If no collection is active it sets an error message (`ff-desktop-shell-msg-scrm-no-collection-replay`). Title hard-coded "Screen Replay". |
| 4.5 | POM > S > H | H SCRM Help | `HELP SCRM` | Ladder | HelpContext with SCRM topic (same as #3) | context-help, screen-snapshot-scrm | y | y | n | Same Kind as #3; fold into Help gate. |
| 4.6 | POM > S > X | X Exit | `X` | Ladder (X close verb; self-ref guard skips) | none (closes SCRM menu tab) | menu-workspace Req 14.13 | n/a | n/a | n | SCRM menu X lacks `show_in_menu_bar=false` (POM's X has it) so X shows in the SCRM dropdown; likely intended but record. |
| 5 | POM > X | X (terminate) | `X` | Ladder | none (collapse to visual root / close workspace, exit when last) | menu-workspace Req 2.1g | n/a | n/a | n | `show_in_menu_bar=false`. Description is long: "Collapse to the visual root / close workspace (exit when last)". |

### 3a. Reached but ALSO opened by typed command only (NOT reachable from the built-in POM tree)

Put at the END of the walk. All typed verbs classify in
`builtin_workspace_target_for` -> Custom(...) -> `dispatch_command_target`
except where noted. Removal of Catalogs/Files from the POM is CR-CH-073 (they
remain typed-reachable by owner approval).

| # | Typed command | Target | Kind / TabKind | Spec folder | WC | SP | TS | Notes |
|---|---------------|--------|----------------|-------------|----|----|----|-------|
| 6 | `COMMANDS` | Custom(`command_configurator`) | CommandConfigurator | command-configurator | y | y (`command_configurator_screen_model`) | y-ish (row per command definition) | Candidate ispf consumer later (not in spec). Shares file with Alias editor spec. |
| 7 | `LOG` | Custom(`event_log`) (also marks all read) | EventLog | notification-system | y | y (`event_log_screen_model`) | y-ish (entries + filters) | |
| 8 | `PLUGINS` | Custom(`plugin_manager`) | PluginManager | plugin-manager-ui (rename to extension-manager-ui pending; rename rides along with real work per extension-terminology.md) | y | y (`plugin_manager_screen_model`) | y-ish | Name still "plugin"; rename when touched. |
| 9 | `MACROS` | Custom(`macro_library`) | MacroLibrary | lua-macro-engine | y | y (`macro_library_screen_model`) | y-ish | |
| 10 | `GSEARCH` / `SEARCH` | Custom(`search`) (`open_or_focus_search_panel`) | SearchResults | global-search | y | y (`search_results_screen_model`) | y-ish (rows with hit counts) | Single instance, focused if open. |
| 11 | `FILES` | Custom(`file_explorer`) | FileExplorerPanel | file-tree-panel | n (apply_interior_focus none) | y (`file_explorer_screen_model`) | n | PARKED in core build (core-001 strip): shows "unavailable in core" notice. |
| 11b | `CATALOGS` / `FILE CATALOGS` | Custom(`files`) | FilesPanel | virtual-catalog-manager / dataset-catalog | n | y (`files_panel_screen_model`) | n | PARKED in core build, same notice. |
| 11c | (file open) | - | FileEditor / Untitled | document-model, multi-tab-editor | n | n (opt-out `PARKED_EDITOR_REASON`) | n | Only documented ScreenProvider opt-out. Editor plugin out of core build. |
| 12 | `<name>` or `MENU <name>` for a user `menus/<name>.toml` | Menu `<name>` (stage 4, menu_name_target) | MenuWorkspace, NEW TAB | menu-workspace | n | y | n | Same Menu Kind gate (#1). User `menus/pom.toml` / `settings.toml` / `scrm.toml` OVERRIDE the compiled constants, so the real tree can differ per user. Missing file shows load-error state. User menus authored via Settings > Menus editor (#2.4). |
| 13 | Help / Settings via menu bar | - | same as #3 / #2 | - | - | - | - | The menu bar is a runtime echo of the POM (`render_menu_bar.rs`; SCRM dropdown built from `recovery_scrm_menu()`); not a separate entry. |

## 4. Options that fail to resolve or open nothing

None found by static reading. Every option's command resolves:
- `Settings`, `SCRM`: Menu targets via `BUILTIN_MENU_NAMES`.
- `Config`, `Theme`, `Menus`, `Keys`, `Kinds`, `Aliases`: Custom targets in
  `builtin_workspace_target_for` (verified each `is(...)` arm exists).
- `Help`, `HELP SCRM`, `Reset bare`, `SNAPSHOT`, `CAPTURE START|STOP|REPLAY`, `X`:
  fall through `resolve_target` (Err) and are handled by explicit ladder arms.

Observations (possible bugs / risks, NOT fixed):
1. `Reset bare` has NO classifier entry: it is ladder-only, so it does not get
   the `last_dispatched` SCRM record from `resolve_target`
   (`dispatch.rs` sets it only on a resolved target). Likewise Help, SNAPSHOT,
   CAPTURE*, X. Minor, relates to the CR-NR-098 triggering-command label.
2. `THEME <arg>` returns None from the classifier and relies on the ladder;
   bare `Theme` (the menu command) is fine.
3. SCRM menu opens a NEW tab while Settings navigates IN PLACE. Intentional
   per `open_menu_by_name`, but worth confirming for the "walk the tree" flow
   (END from SCRM returns to a separate tab, not up a stack).
4. `CAPTURE REPLAY` with no active collection sets an error rather than
   opening a viewer; the menu option can therefore appear to do nothing.
5. The terminal probe `git -C ... merge-base alias-kind-gate main` was blocked
   by the guard hook (chained `;`), so the Alias branch base (a6aacae) comes
   from gate-summary.md and is not independently re-verified.

## 5. Duplicate or inconsistent labels / descriptions

Rendering (`screen_snapshot.rs::option_row_text`, test
`option_row_matches_owner_mock_format`) produces `<key> <command> : <description> (<command>)`:
- `S SCRM : SCRM : Screen Capture and Recording (SCRM)` -- the POM S
  description itself starts with "SCRM :" so the label is shown twice, plus the
  trailing `(SCRM)`. Fix in the TOML description ("Screen Capture and Recording")
  and drop the bracket suffix.
- `X X : Collapse to the visual root / close workspace (exit when last) (X)` --
  the owner's "X X :" item: key and command are both "X". Description also
  parenthesised, plus `(X)` suffix gives two bracket groups.
- Every row ends with `(<command>)` e.g. `(Settings)`: owner already asked to
  remove (pom-snapshot-review Menu Kind items).
- Description style is inconsistent across menus: POM uses a plain phrase
  ("FFWB Settings and Client Parameters"), `Help -- ...`, `SCRM : ...`; Settings
  options use `<Name> -- <phrase>` (e.g. `Config` row "All settings -- browse
  every configuration key" does not start with the label; `Aliases` row starts
  "Alias editor -- ..." (label differs from command "Aliases")); SCRM menu uses
  `<Verb> -- <phrase>`. Settings "Reset bare" description starts "Reset to
  barebones -- ..." (label differs).
- Mnemonics: POM has 0,3,S,X (gap 1,2 left by CR-CH-073); Settings uses
  C,A,T,M,K,W,R (W for Kinds is irregular); SCRM uses 1-2 letter keys
  (S,CS,CT,CR,H,X) mixing single and double-letter keys in one column.
- SCRM `X` row is missing `show_in_menu_bar=false` that POM's X has.
- Help title is hard-coded "Help" and SCRM viewer title "Screen Replay"
  (`screen_providers.rs`) rather than Kind display titles (other Contexts use
  `active_display_title`).
- "plugin" naming remains in `PLUGINS` / plugin_manager panels; extension rename
  rides along per extension-terminology.md when touched.

## 6. Recommended gate sequence

Each workspace gets a per-Kind gate on the common frame (CR-NR-115) following
the Alias gate shape (CR-NR-116).

1. MENU KIND gate (one gate): POM (#1), Settings (#2), SCRM menu (#4) and user
   menus (#12). Items: key line last, command line text/underscores via
   `common_frame_command_field_text`, strip `(command)` suffix and the doubled
   `SCRM :`/`X X :` labels, align `:` column, md table for clipboard, calendar
   position, Zoom segment, snapshot confirmation, Last/startup-time rule, add
   Menu golden. Also normalize built-in descriptions (section 5) as content.
2. ALIAS (#2.2): rebase `alias-kind-gate` onto 8ff3e3b, resolve the owner
   decision points (a)-(g), then approve and implement. It delivers the shared
   table pieces (framed renderer, `ColumnSpec`, scratch row, `markdown_sections`),
   so it MUST land before the other table editors.
3. CONFIG (#2.1): first in the on-screen Settings order but not table-style;
   independent of Alias, so it can run before or after it. Order recommended
   per the owner's walk: Config first, then Alias. Alias gate is already parked,
   so doing Config while Alias waits for approvals avoids idle time.
4. THEME (#2.3), MENUS (#2.4), KEYS (#2.5), KINDS (#2.6): in walk order. Menus
   and Kinds are ispf_table_editor adoption steps (Req 9) and inherit the
   Alias shared pieces. Keys and Theme need a decision on whether they join the
   shared table editor.
5. RESET BARE dialog (#2.7): fold into a small "dialog capture" check with the
   other dialogs; no workspace gate.
6. HELP (#3, #4.5): one gate (same Kind); includes the hard-coded title.
7. SCRM functions (#4.1-4.3, #4.6, #5): no workspace; verify only. #4.4 SCRM
   VIEWER gets its own small gate (WC y, SP y).
8. Typed-only workspaces afterwards: COMMANDS, LOG, PLUGINS (rename to
   extensions as you go), MACROS, SEARCH, then parked FILES/CATALOGS and the
   Editor when they re-enter the core build.

Merge only where a Kind is shared: items 1 (all menus), 6 (Help + HELP SCRM).
No other pair shares a Kind.

## 7. Files read

- `crates/ff-desktop/src/menu_workspace/defaults.rs` (all three menus)
- `crates/ff-desktop/src/command_config/mod.rs` (`builtin_workspace_target_for`,
  `BUILTIN_MENU_NAMES`, `ShellTargetResolver`)
- `crates/ff-command/src/command_target.rs` (`resolve_target`)
- `crates/ff-desktop/src/shell/dispatch.rs`, `target_dispatch.rs`,
  `commands_menu.rs`, `commands_ladder_a.rs`, `commands_ladder_b.rs`,
  `screen_provider.rs`, `screen_providers.rs`, `screen_providers_config.rs`
- `crates/ff-desktop/src/screen_snapshot.rs` (`option_row_text` test)
- `docs/specs/ispf-table-editor/requirements.md`
- `.agents/tasks/pom-snapshot-review/findings.md`
- `.agents/tasks/alias-kind-gate/gate-summary.md`
- git: main HEAD = 8ff3e3b (from the proxy result file)

Not verified: spec folder contents beyond names/headings, actual rendered
output of any screen (no run), the alias-kind-gate worktree state.
