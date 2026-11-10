# Config-model change -- READ-ONLY investigation of the current system

Scope: FileForgeWorkbench (repo root `c:/workspace/apps/FileForgeWorkbench`, main @ 2c1c266).
Purpose: ground the owner-confirmed FRAMEWORK CHANGE (see `decision.md`) -- move
from CODE-ONLY-WITH-OVERRIDES to WRITE-THROUGH-CONFIG for the built-in menus (POM,
Settings, SCRM) and the command alias list, with the Rust defaults kept as the
RECOVERY source only. This report documents EXACTLY how the system works TODAY so
the design gate can specify the switch precisely. It recommends no implementation.

All line references are to files under `crates/ff-desktop/src/` unless noted.

---

## Summary answers (one line each)

1. **Menu loading today**: every render the loader tries the user file on disk
   (`menus/<name>.toml`), and if absent/unparseable falls back to the CODE-ONLY
   compiled constant (`recovery_*_menu()` parsing `DEFAULT_*_TOML`). Nothing is
   written on first run. `menus_dir()` resolves the user-data `menus/` dir (test
   override first). `show_calendar` silently defaults to `true` when absent.
2. **Alias loading today**: there is NO shipped X->EXIT seed. The resolver is
   built from an IDENTITY seed (every FFCMD root maps to itself) + the shipped
   read-only locale seed (`i18n/<locale>/aliases/<ENV>.toml`) + a USER per-locale
   override (`<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`).
   The user store is empty until the user adds a row in the Alias Editor. The only
   genuine alias in the base is FFEDIT `X -> EXCLUDE`.
3. **First-run / bootstrap**: startup (`shell/construct.rs`) reads everything and
   writes NOTHING. `config.toml` is created only when a config value is actually
   changed (`set_user_value`). `ensure_menus_dir` / `ensure_keymaps_dir` create
   the (empty) directory but never materialise built-in files. Natural first-run
   write hook: `construct.rs` (per-item, right where each dir/store is loaded).
4. **Reset bare today**: ARCHIVES (moves, does not delete) `menus/ themes/
   keymaps/ workspace-kinds/ session.toml config.toml catalogs.toml` into
   `config-archive/<timestamp>/`, then resets the live in-memory state to the
   compiled baselines. It does NOT touch the alias override store, and it REBUILDS
   nothing on disk -- the barebones state comes from the code-only fallback. This
   is the command that must, post-change, REBUILD the menu + alias TOMLs from the
   Rust defaults.
5. **X / EXIT today**: `EXIT` already exists (registered `file.exit`, FFCMD root
   verb EXIT/QUIT/LOGOFF -> unconditional app-exit). `X` is a DIFFERENT verb: the
   `nav_x` close/collapse verb, NOT app-exit. The POM X option's `command` is the
   literal `"X"` (self-referential, guarded). TARGET: X becomes a SEED ALIAS of
   `EXIT` in the FFCMD override; POM X option command becomes `EXIT`; there is no
   command named X. The EXIT command/verb already exists; nothing new is needed
   there -- but note the owner's "X -> EXIT" is a change of meaning (today X !=
   EXIT), which the design must reconcile (see Q5).
6. **Directories + format**: menus/themes/keymaps/kinds/scrm each resolve through a
   `dir_overrides.<x>` test seam; aliases resolve through `dir_overrides.user_aliases`
   plus the `FFWB_USER_ALIASES_DIR` env seam. Tests isolate via these to a TempDir.
   Write-through must keep these seams authoritative for first-run writes too.
7. **Blast radius**: the reroute list (production readers of the constants /
   `recovery_*` fns) is small and almost entirely in `shell/`. Risks: a partially
   present config dir, a now-authoritative user file shadowing a newer default
   (the deferred UPGRADE problem), the `show_calendar` silent-true default, and
   test isolation. The Menus-editor SCRM bug falls out naturally once scrm.toml is
   on disk, but the hardcoded `[POM, Settings]` list and the POM-fallback in
   `load_menu_for_editor` still need touching.

---

## Q1 -- MENU LOADING TODAY

### How POM / Settings / SCRM are produced at runtime

Built-in menu content lives as compiled constants in
`menu_workspace/defaults.rs`:
- `DEFAULT_POM_TOML` (lines ~47-74) -- options `0 Settings`, `3 Help`, `S SCRM`,
  `X X` (X hidden from bar).
- `DEFAULT_SETTINGS_TOML` (lines ~90-136) -- `show_calendar = false`,
  `group_separator = "line"`, options C/A/T/M/K/W + R Reset bare.
- `DEFAULT_SCRM_MENU_TOML` (lines ~178-216) -- S/CS/CT/CR/H + X.

Each is parsed on demand into a `MenuFile` by `recovery_pom_menu()` (line 144),
`recovery_settings_menu()` (line 153), `recovery_scrm_menu()` (line 222), each
calling `menu_workspace::loader::parse_menu_str(...)`. **The constant is parsed
fresh every time these are called** -- there is no cached singleton.

### The loader that chooses file-vs-constant

There is no single "loader" function; the file-vs-constant choice is made at each
call site, always in the shape "load user file; `.ok()` or `.unwrap_or_else`
fall back to the recovery constant":

- **POM**: `shell/commands_fastpath.rs::ensure_pom_menu_loaded()` (lines ~21-57).
  Loads `menus/pom.toml` via `MenuWorkspaceState::load_with_limits(&pom_path, ..)`;
  if `state.menu.is_none()` it substitutes `recovery_pom_menu()` and, when a file
  EXISTED but failed to parse, posts a non-blocking notice (`notify_menu_fallback`).
  A merely absent file is silent.
- **Settings**: opened in place via `open_settings_menu()`
  (`shell/commands_menu.rs` ~line 258) which `navigate_to(Menu{"settings"}, true)`;
  the reconstruct path `shell/nav_reconstruct.rs` (~line 128) sets
  `recovery_settings_menu()` when the file did not produce a menu.
- **SCRM**: `shell/commands_menu.rs::open_menu_by_name("scrm")` (lines ~101-105)
  opens `menus/scrm.toml`; when `mw.menu.is_none()` it seeds `recovery_scrm_menu()`.
- **Menu bar echo**: `shell/render_menu_bar.rs::active_pom_menu()` (~line 78) and
  `menu_bar_dropdown_options` (~line 255) prefer a loaded POM tab's menu, else
  `menus/pom.toml` on disk, else `recovery_pom_menu()` (and `recovery_settings_menu`
  / `recovery_scrm_menu` for those bar items). `shell/render_chrome.rs` (~line 106)
  does the same for the active-bar menu.

### menus_dir() and its fallback

`shell/commands_menu.rs::menus_dir()` (lines 29-45):
1. `self.dir_overrides.menus` (test override) if set;
2. else `ff_session::UserDataDir::resolve(None).path().join("menus")`;
3. else `dirs::data_dir()/FileForgeWorkbench/menus`;
4. final fallback the relative `menus`.

### Where show_calendar (and other per-menu flags) default when missing

`menu_workspace/loader.rs`: the `RawMenuFile` struct uses
`#[serde(default = "default_true")] show_calendar: bool` (~line 40). So an ABSENT
`show_calendar` defaults to **TRUE** (`load_show_calendar_defaults_to_true` test,
~line 355). `group_separator` defaults to `space`, `group_headers` to `false`,
option `enabled` to `true` (per `serialiser.rs` doc, lines 9-12). The compiled
Settings constant sets `show_calendar = false` EXPLICITLY precisely to override
this silent-true default.

### Confirmation of current behaviour
CONFIRMED: the constant is parsed each run; a user file overrides when present and
valid; **nothing is written on first run** (`ensure_menus_dir` only creates the
empty directory, defaults.rs lines 240-245; its tests
`ensure_menus_dir_does_not_materialise_built_in_menus` assert pom/settings/scrm
are NOT written).

### CURRENT vs TARGET
- CURRENT: code constant is authoritative unless a user file shadows it; the file
  is optional and never created by the app.
- TARGET: on first run WRITE `pom.toml` / `settings.toml` / `scrm.toml` (COMPLETE,
  every flag explicit incl. `show_calendar`), then RUN FROM the files; the
  constants become the recovery source consumed only by first-run and Reset bare.

---

## Q2 -- ALIAS LOADING TODAY

### Is there a shipped SEED alias set?
There is NO shipped `X -> EXIT` alias and no general user-facing seed table. The
alias resolver is assembled in three layers (`shell/alias_reader.rs`,
`LocaleAliasStore::seed_and_overlay_for_locale`, ~line 230):
1. **Identity seed** (`LocaleAliasStore::seed`, ~line 205): FFEDIT starts from
   `AliasTable::ffedit_english()` (which already carries the two genuine aliases
   `X -> EXCLUDE` and `INCLUDE -> SHOW`, `shell/environment.rs` lines ~126-142);
   FFCMD starts from `AliasTable::identity_for(ffcmd_roots)` -- every built-in root
   maps to itself. The FFCMD roots come from `builtin_ffcmd_roots()`
   (`alias_reader.rs` ~line 150): the option `command` of every option in
   `recovery_pom_menu()` + `recovery_settings_menu()`, UNION
   `BUILTIN_WORKSPACE_VERB_ROOTS` and `BUILTIN_FFCMD_LADDER_VERB_ROOTS`.
2. **Shipped read-only locale seed**: `i18n/<locale>/aliases/<ENV>.toml`
   (`apply_overlay_from_dir`). At the default `en` with no such tree this is
   identity-only. The only shipped example is the Afrikaans POC
   `af/aliases/FFEDIT.toml` (`SOEK -> FIND`).
3. **User per-locale override**: `apply_user_overlay_from_base(base, locale)`.

So the user alias table is EMPTY until the user adds a row (e.g. the owner's
`SS -> SNAPSHOT`). There is no seed path that pre-populates user aliases, and no
`isolate_user_alias_base`-style "base/seed" discipline implying one -- the
`FFWB_USER_ALIASES_DIR` seam exists only for test isolation of the USER override
(see Q6), not as a shipped seed.

### User alias store on disk
CONFIRMED location: `<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`.
- Base resolver: `shell/user_aliases.rs::user_alias_override_base()` (~line 118):
  `FFWB_USER_ALIASES_DIR` env override, else
  `dirs::data_dir()/FileForgeWorkbench/aliases`.
- File path: `user_alias_override_file(base, locale, env_name)` =
  `<base>/<locale>/<ENV>.toml` (~line 130). ENV is encoded in the FILE NAME, not a
  per-row key.
- Row type: `UserAliasRowToml { surface, canonical, environment(skip),
  command_template?, description? }` (`user_aliases.rs` ~line 87).

### How it is written (save)
- `shell/alias_editor.rs::write_override_rows(base, locale, ...)` (~line 262):
  groups rows by ENV, serialises each via
  `user_aliases::serialise_user_aliases(&env_rows)`, creates parent dirs, and
  `std::fs::write(<base>/<locale>/<ENV>.toml, ...)`. (This is the `save_user_aliases`
  / `write_override_rows` referenced in the brief.)
- Save only writes the USER override, never the shipped seed (module doc,
  `alias_editor.rs` lines 9-13). A save triggers a reload
  (`reload_active_alias_store_for_locale`) so edits go live without restart.

### Where the resolver reads it
- `AliasTable::canonical_verb(surface)` (`shell/environment.rs`) -- surface -> root.
- `rewrite_verb_to_root(cmd)` on the shell -- used by `activate_menu_option` and the
  command line to resolve a surface to its ROOT before dispatch
  (`shell/commands_menu.rs` ~line 200).
- `LocaleAliasStore::table_for(env_name)` / `effective_display_rows()`
  (`alias_reader.rs`) -- the resolver and the Alias Editor read from these.
- Per-locale re-seed: `seed_and_overlay_for_locale` (~line 230) and the hot-reload
  `reload_active_alias_store_for_locale` (~line 360) via the process-global
  `ACTIVE_ALIAS_STORE`.

### CURRENT vs TARGET
- CURRENT: user alias file is created only when the user saves a row; no X->EXIT
  anywhere; FFEDIT base has X->EXCLUDE.
- TARGET: on first run WRITE a seed FFCMD override containing `X -> EXIT` (the ONLY
  seeded alias per the owner), load from it thereafter, and have Reset bare rebuild
  it. Note the collision/precedence question with FFEDIT `X -> EXCLUDE` (Q5/Q7).

---

## Q3 -- FIRST-RUN / BOOTSTRAP

### Where startup decides what config exists
`shell/construct.rs::new()` -> `new_with_history_store()` is the single startup
wiring point. It LOADS and never seeds on first run:
- `CommandStore::load(<udd>/commands/commands.toml)` -- absent -> empty store.
- `KindRegistry::load(<udd>/workspace-kinds/)` -- absent -> built-in defaults only.
- `build_help_registry_for_locale(locale)`.
- `LocaleAliasStore::seed_and_overlay_for_locale(roots, locale)` -- identity seed +
  on-disk overlay (reads only).
- Keymaps: `ensure_keymaps_dir(&udd)` then `load_context_maps_from_keymaps_dir(...)`.
- The POM menu is loaded lazily on first render by `ensure_pom_menu_loaded`.

### Is anything written to User_Data_Dir on first run today?
Effectively NO config content is written:
- `config.toml` is NOT created at startup. `ff-config/src/init/mod.rs::init()`
  (steps 1-6, ~line 127) registers schema defaults and loads system/user/project/
  workspace files, "gracefully skip if missing". `config.toml` is created only when
  a value is persisted via `set_user_value` / `persist_active_profile`
  (`ff-config/src/config_handle/...`, `profile.rs::persist_active_profile`).
- `ensure_menus_dir` / `ensure_keymaps_dir` create the EMPTY directory only.
- No menu, theme, kind, or alias file is materialised.

### Natural hook point for first-run writing
`shell/construct.rs::new_with_history_store` is the natural place: each config
category is already loaded there, with the resolved dir / store in scope. A
first-run write would, per category, detect "no file present" (respecting the
`dir_overrides` seams, Q6) and write the COMPLETE default (serialise the
`recovery_*_menu()` / the seed alias rows) BEFORE loading, so the subsequent load
reads the just-written file. The menu serialiser already exists
(`menu_workspace::serialiser::serialise`) and the alias serialiser exists
(`user_aliases::serialise_user_aliases`). (The POM is loaded lazily on first
render rather than in `construct`, so its first-run write either happens in
`construct` alongside the others or in `ensure_pom_menu_loaded`; the design should
pick one -- `construct` is the more consistent single hook.)

---

## Q4 -- RESET BARE TODAY

Trigger: `shell/commands_ladder_b.rs` (~line 96) matches `RESET BARE` /
`RESET BARE <args>`, resolves a target profile list via
`resolve_reset_bare_target(args)` and opens a confirmation dialog
(`reset_bare_confirm`). The execution lives in `shell/reset_bare.rs`.

### What it resets now
On confirm, `execute_reset_bare(target)` (`reset_bare.rs` ~line 268):
1. For each targeted profile, `archive_config(udd)` MOVES (not deletes) each
   present item in `ARCHIVED_ITEMS` into `config-archive/<timestamp>/`.
   `ARCHIVED_ITEMS` (lines 15-26) = `menus`, `themes`, `keymaps`,
   `workspace-kinds`, `session.toml`, `config.toml`, `catalogs.toml`.
2. If the running profile is among the targets (`includes_active`),
   `reset_in_memory_to_baseline()` (~line 300): rebuilds `kind_registry` from
   `with_builtin_defaults()`, clears the user theme keys + re-applies
   `Default Legacy`, and clears the Home tab's menu state so
   `ensure_pom_menu_loaded` reloads the code-only POM baseline.

### Does it touch menus and aliases, or only config.toml?
- MENUS: archives the whole `menus/` DIRECTORY (moved aside). It does NOT rebuild
  any menu file; the barebones menus come from the code-only fallback at render.
- ALIASES: **NOT touched at all.** `aliases/` is NOT in `ARCHIVED_ITEMS`, and
  `reset_in_memory_to_baseline` does not reset the alias store. A user alias like
  `SS -> SNAPSHOT` survives a Reset bare today. (This is exactly what the owner
  wants changed: Reset bare must rebuild the alias seed incl. `X -> EXIT`.)
- config.toml: archived (moved aside), so the next `set_user_value` recreates it.

### Archive / backup behaviour
Non-destructive MOVE into `config-archive/<timestamp>/` (`archive_config`,
~line 48). Cross-volume rename falls back to copy-then-remove (`move_path`). Missing
items skipped; prior archives never pruned (Req 19.8 tests). Best-effort: one
item's failure does not abort the rest.

### CURRENT vs TARGET
- CURRENT: Reset bare = "archive config dir + reset live state to code baseline";
  rebuilds NO files; ignores aliases.
- TARGET: Reset bare must, after archiving, REBUILD `pom.toml` / `settings.toml` /
  `scrm.toml` AND the seed alias file(s) (incl. `X -> EXIT`) from the Rust
  recovery defaults, and `aliases/` must join `ARCHIVED_ITEMS` so the old user
  overrides are moved aside before the rebuild. The rebuild is the same
  serialise-defaults step as the first-run write (Q3), so the two should share one
  helper.

---

## Q5 -- X / EXIT TODAY

### How X is handled now
- The EXIT family lives in `shell/commands_ladder_a.rs::try_exit_family()`
  (lines 32-54): matches `EXIT` / `QUIT` / `LOGOFF` only, dispatching
  `file.exit` (registered in `construct_commands.rs`). `X` is explicitly NOT in
  this family (CR-CH-052 comment, lines 33-39).
- `X` is handled separately in `try_commands_a` via the `upper == "X"` arm
  (`commands_ladder_a.rs` ~line 315): calls `self.nav_x()` -- collapse the
  Navigation_Stack to the tab visual root, close the Workspace at the empty root,
  exit only when it is the last Workspace (`shell/nav_stack.rs::nav_x` ~line 215 ->
  `close_workspace_or_exit`). So **X today means "close/collapse", not app-exit.**
- Prelude precedence (`shell/commands.rs::run_command_prelude`, ~lines 62-86): the
  EXIT family runs before the POM Option_Key fastpath; on an editor Context the
  active FFEDIT environment claims bare `X` as EXCLUDE first (active-wins, E8).
- The POM X option: in `DEFAULT_POM_TOML` the X option has `command = "X"`
  (self-referential). The self-reference guard is in
  `shell/commands_menu.rs::try_current_menu_option` (~line 158): when a resolved
  option's command equals its own key and there is no inline target, it returns
  `false` (falls through) to avoid infinite recursion -- which is why `X` then
  reaches the `nav_x` arm.

### Is there already an EXIT command/verb?
YES. `EXIT` is a registered FFCMD root verb (`command_config/mod.rs`
BUILTIN_FFCMD_LADDER_VERB_ROOTS region includes `EXIT`, `QUIT`, `LOGOFF`, ~line
230) and is dispatched through `file.exit`. **No new EXIT command needs to be
introduced** -- the command already exists and already means app-exit.

### What must change for the target
Owner's model: "X should map to the EXIT command; X as a command should be an
alias of EXIT; there should not be a command called X." The pieces:
1. POM X option `command` changes from `"X"` to `EXIT`
   (`DEFAULT_POM_TOML`; same for the SCRM X option in `DEFAULT_SCRM_MENU_TOML`,
   which is also `command = "X"`). Shows as `X EXIT : ...`.
2. `X` becomes a SEED ALIAS of `EXIT` in the FFCMD override
   (`aliases/<locale>/FFCMD.toml`), written on first run and rebuilt by Reset bare.
3. Resolution order the owner specified: the active command environment
   interpreter first (so an editor still gets FFEDIT `X -> EXCLUDE`), then FFCMD
   resolves `X -> EXIT`. The existing active-wins gate (E8, `commands.rs`
   ~line 68) already gives the editor FFEDIT first crack at bare `X`; on non-editor
   contexts `X` must now resolve via the FFCMD alias to `EXIT`.

### IMPORTANT reconciliation for the design gate
This is a **semantic change**, not just a wiring change. Today on non-editor
contexts `X` = `nav_x` (collapse/close, exit only when last). Under the owner's
model `X -> EXIT` = unconditional app-exit. The design gate must decide
explicitly whether:
- (a) the close/collapse behaviour (`nav_x`) is retired in favour of app-exit, or
- (b) `EXIT` is redefined, or
- (c) the POM X option and the bare-`X`-on-POM case specifically map to EXIT while
  `nav_x` remains for intermediate stacks.
The user messages (POM X -> EXIT; "x issued from the POM should resolve to EXIT")
point at the POM/last-workspace case. There is also an existing FFEDIT
`X -> EXCLUDE` that must NOT be shadowed. Flag this as the central open question of
the gate; do not assume the resolution here.

---

## Q6 -- DIRECTORIES + FORMAT + TEST ISOLATION

### Directories
- menus: `menus_dir()` = `dir_overrides.menus` -> `<udd>/menus` (Q1).
- themes: `commands_theme.rs::themes_dir()` = `dir_overrides.themes` -> `<udd>/themes`.
- keymaps: `keys_editor.rs::keymaps_dir()` = `dir_overrides.keymaps` -> `<udd>/keymaps`.
- workspace-kinds: `kinds_editor.rs::workspace_kinds_dir()` = `dir_overrides.workspace_kinds`.
- scrm: `commands_session.rs::scrm_dir()` = `dir_overrides.scrm`.
- aliases: `alias_editor.rs::user_alias_override_base()` = `dir_overrides.user_aliases`
  -> `user_aliases::user_alias_override_base()` (env `FFWB_USER_ALIASES_DIR` ->
  `<udd>/aliases`).

`DirOverrides` is defined in `shell/state_groups.rs` (~line 28): one `Option<PathBuf>`
per category (`themes`, `menus`, `keymaps`, `workspace_kinds`, `scrm`, `user_aliases`),
default all `None` in production (set in `construct.rs` ~line 287).

### How dir_overrides work for tests
`shell/tests_common.rs`:
- `make_shell()` sets `shell.dir_overrides.menus = Some(isolated_menus_dir())` (~line
  219) -- a process-global guaranteed-EMPTY temp dir, so Menu Kinds fall back to the
  code-only baseline regardless of ambient `menus/`.
- `isolate_user_alias_base_once()` (~line 117) sets `FFWB_USER_ALIASES_DIR` to an
  empty temp dir BEFORE any shell is built (the construction-time alias overlay runs
  before a test can set `dir_overrides.user_aliases`), so no ambient
  `aliases/<locale>/<ENV>.toml` leaks rows.
- Tests needing specific content set their own `dir_overrides.<x>` / env to a TempDir
  AFTER `make_shell()` (e.g. `alias_editor_line_commands.rs` ~line 362;
  `tests_alias_editor.rs::point_user_aliases_at_temp` ~line 24;
  `nav_reconstruct.rs` ~line 328).

### Format
Menu files: TOML parsed by `menu_workspace/loader.rs` (`RawMenuFile`) and written by
`menu_workspace/serialiser.rs::serialise` (omits fields that equal their loader
default -- this is incompatible with the owner's "every flag explicit" requirement,
see Q7). Alias files: `[[alias]]` array, `surface`/`canonical`/`command_template?`/
`description?`, ENV in the file name (`user_aliases.rs`).

### Implication for the write-through design
First-run writing and Reset-bare rebuild MUST route through the SAME `*_dir()` /
`user_alias_override_base()` seams so tests can keep isolating to a TempDir. A
write-through model that bypasses `dir_overrides` would break the entire shell test
suite's isolation.

---

## Q7 -- BLAST RADIUS + RISKS

### Reroute list -- production readers of the code-only constants / recovery fns
(These are the sites that today read defaults directly and, under write-through,
should instead read the on-disk file -- or read the file via a single shared loader
that falls back to the constant only for recovery.)

1. `shell/commands_fastpath.rs::ensure_pom_menu_loaded` -- `recovery_pom_menu()`
   fallback (~line 43) AND `resolve_pom_option_key` parses `DEFAULT_POM_TOML` as a
   last resort (~line 182).
2. `shell/commands_menu.rs::open_menu_by_name` -- `recovery_scrm_menu()` fallback
   for SCRM (~line 102).
3. `shell/nav_reconstruct.rs` -- `recovery_settings_menu()` fallback (~line 128).
4. `shell/render_menu_bar.rs::active_pom_menu` / `menu_bar_dropdown_options` --
   `recovery_pom_menu` / `recovery_settings_menu` / `recovery_scrm_menu` fallbacks
   (~lines 78, 259-264).
5. `shell/render_chrome.rs` -- `active_pom_menu()` fallback for the active bar
   (~line 106).
6. `shell/menus_editor.rs::load_menu_for_editor` -- `recovery_settings_menu()` /
   `recovery_pom_menu()` fallback (~line 175); `refresh_menus_editor_list`
   hardcodes `["POM","Settings"]` (~line 185).
7. `shell/alias_reader.rs::builtin_ffcmd_roots` -- derives FFCMD identity-seed
   roots from `recovery_pom_menu()` + `recovery_settings_menu()` (~line 163).
8. `command_config/mod.rs` -- `BUILTIN_MENU_NAMES = ["pom","settings","scrm"]`
   (~line 136) classifies these names as Menu targets (mentions
   `DEFAULT_SCRM_MENU_TOML` in docs only).

Non-production (test-only) readers -- safe to ignore for the reroute but update as
behaviour changes: `screen_snapshot.rs` test, `screen_snapshot_menu/tests.rs`,
`shell/tests_*`, and the `defaults.rs` test module.

NOTE: outside `shell/` and `menu_workspace/`, the ONLY production reader is the
`BUILTIN_MENU_NAMES` list and the `builtin_ffcmd_roots` seed derivation -- the blast
radius is contained in `ff-desktop`.

### Places that assume "no file on first run"
- `menu_workspace/defaults.rs::ensure_menus_dir` + its tests
  (`ensure_menus_dir_does_not_materialise_built_in_menus`) assert pom/settings/scrm
  are NEVER written. These tests MUST be inverted/retired under the new model.
- `ensure_pom_menu_loaded` treats a merely-absent file as the silent code-only
  fallback path -- under write-through an absent pom.toml on first run should be
  written, not silently substituted.
- The reset_bare test `archive_config_moves_present_items_and_removes_originals`
  and `archived_items_includes_workspace_kinds` encode the current ARCHIVED_ITEMS;
  adding `aliases` changes these.

### The Menus-editor SCRM bug (and how write-through fixes it)
- `refresh_menus_editor_list` (`menus_editor.rs` ~line 184) hardcodes
  `vec!["POM","Settings"]` + an on-disk `*.toml` scan. SCRM is code-only, so no
  `scrm.toml` exists on disk today, so SCRM never appears in the editor dropdown.
- `load_menu_for_editor` (~line 166) falls back to `recovery_settings_menu()` ONLY
  for "SETTINGS"; everything else (including "SCRM") falls back to
  `recovery_pom_menu()` -- so even if SCRM were selected, the editor would load the
  POM, not the SCRM menu.
- Under write-through, `scrm.toml` exists on disk after first run, so the on-disk
  scan lists it naturally AND `load_menu_for_editor` loads the real file. BUT the
  hardcoded `["POM","Settings"]` seed list would still omit SCRM if the scan were
  ever empty, and the `_ => recovery_pom_menu()` fallback is still wrong for SCRM.
  Recommend: the design list the editor's built-in set as `[POM, Settings, SCRM]`
  (or derive it from the on-disk files + the known built-in names) and fix the
  fallback match to map SCRM -> `recovery_scrm_menu()`.

### The show_calendar silent-default-true behaviour
`loader.rs` defaults absent `show_calendar` to TRUE (Q1). The owner wants this
silent default REMOVED: first-run must write COMPLETE config with every flag
explicit. Two concrete design consequences:
- `menu_workspace/serialiser.rs::serialise` currently OMITS fields equal to their
  loader default (e.g. `show_calendar = true`, lines 9-27). The first-run writer
  must emit a COMPLETE file (every flag explicit), so either the serialiser gains a
  "complete" mode or the writer post-fills defaults. A plain `serialise` of
  `recovery_pom_menu()` would NOT currently emit `show_calendar`.
- The POM default (`DEFAULT_POM_TOML`) does not set `show_calendar` at all (relies
  on true); Settings sets it false. A complete first-run pom.toml must state it.

### Other risks / edge cases for the gate
- **Partially present config dir**: a dir with some built-in files but not others
  (e.g. user deleted `scrm.toml` only). The design must decide per-file first-run
  detection (write the missing ones) vs all-or-nothing. Current fallback silently
  substitutes the constant; write-through must not leave a hole.
- **Authoritative user file shadowing a newer default**: because files become
  authoritative, a user who already has `pom.toml` will NOT get a newly added
  default option on upgrade. This is the DEFERRED UPGRADE problem (decision.md);
  the design must "leave room" (e.g. a version stamp) but not build it.
- **X -> EXIT vs FFEDIT X -> EXCLUDE collision**: the FFCMD seed `X -> EXIT` must
  coexist with the FFEDIT base `X -> EXCLUDE` (different environments -- the overlay
  allows the same surface in different ENVs, `alias_overlay.rs` test ~line 305). The
  resolution order (active env first, then FFCMD) must be specified so an editor
  still excludes and the POM exits.
- **Seed alias write location**: `X -> EXIT` belongs in the FFCMD file
  (`aliases/<locale>/FFCMD.toml`) per the ENV-in-filename model; the locale is the
  active `ui.locale` (default `en`). First-run must choose the locale deliberately
  (en seed) and Reset bare must rebuild the same file.
- **Test isolation**: all first-run writes must honour `dir_overrides` /
  `FFWB_USER_ALIASES_DIR`, or the shell test suite's "empty dir -> code baseline"
  isolation breaks (and the existing `ensure_menus_dir_does_not_materialise_*`
  tests will need rewriting, which itself is a signal to audit every test that
  relies on "no file written").
- **Menu-bar echo**: the bar reads the live POM / on-disk pom.toml / recovery, in
  that order (`render_menu_bar.rs`). Once pom.toml is always on disk, the recovery
  arm becomes recovery-only; confirm the bar still echoes a user-edited pom.toml
  (it should, since the on-disk arm is already preferred).

---

## Conclusions and recommendations for the design gate

1. Introduce ONE first-run/rebuild helper that serialises the recovery defaults
   (menus via a COMPLETE-mode serialiser; aliases via `serialise_user_aliases`)
   into the `*_dir()` / alias-base seams, and call it from BOTH `construct.rs`
   first-run detection and `reset_bare.rs` after archiving. This keeps "write the
   defaults" in one place and guarantees first-run and Reset-bare agree.
2. Reroute the eight production read sites (Q7) to read the on-disk file as the
   authoritative source; keep the `recovery_*` constants reachable ONLY from the
   first-run/rebuild helper and as a last-ditch fallback if a file is missing at
   runtime (graceful degradation), not as the normal path.
3. Add `aliases` to `ARCHIVED_ITEMS` and have Reset bare rebuild the seed alias
   file (incl. `X -> EXIT`); today aliases are untouched by Reset bare.
4. Make the menu serialiser emit a COMPLETE file so `show_calendar` (and the other
   loader-defaulted flags) are explicit; retire the silent-true default's role in
   first-run output.
5. Resolve the X/EXIT SEMANTIC question explicitly (Q5): `EXIT` already exists; the
   work is (a) POM/SCRM X option command -> `EXIT`, (b) seed `X -> EXIT` in FFCMD,
   (c) decide the fate of `nav_x` close/collapse vs app-exit, (d) preserve FFEDIT
   `X -> EXCLUDE` via active-env-first resolution.
6. Fix the Menus-editor SCRM listing + fallback as a natural companion once
   `scrm.toml` is on disk (list `[POM, Settings, SCRM]`; map SCRM ->
   `recovery_scrm_menu()`).
7. Keep every first-run write behind the `dir_overrides` / `FFWB_USER_ALIASES_DIR`
   seams, and plan to rewrite the "never materialised" tests
   (`ensure_menus_dir_does_not_materialise_built_in_menus` and the alias isolation
   assumptions) as part of the change.
8. Update steering `framework-conformance.md` mechanism 4 / CR-CH-021 to record the
   reversal (code-only -> write-through), per the decision doc.

This change is contained within `ff-desktop` (plus the `ff-config` first-run
semantics are untouched -- config.toml already materialises on first write). No
public cross-crate type changes are implied by the investigation; the main surface
affected is the shell's menu/alias load+save seams and Reset bare.
