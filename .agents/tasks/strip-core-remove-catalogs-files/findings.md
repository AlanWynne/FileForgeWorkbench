# Findings + Requirements-Gate Report -- Strip core: remove Catalogs + Files from the barebones POM

**Change class:** CHANGE REQUEST (modification to working behaviour -- the
default menu option set). Logged as **CR-CH-073** (next free CR-CH id; highest
existing is CR-CH-072).

**Owner's words (authoritative):** The application has just been stripped to its
core (see `docs/status/RESUME-ffdesktop-simplification.md`). The reset/bare core
STILL shows two menu options that should NOT belong in the core bare app:
`Catalogs` (a.k.a. `File Catalogs` / `CATALOGS`) and `Files` (the `FILES`
file-explorer option). These two options MUST be REMOVED from the core/bare
application's default menu. **Removal of the menu options is the FIRM
requirement.**

This step makes DOCUMENTATION changes only. No `.rs` file is edited, no worktree
/ branch / commit is created.

---

## 1. The exact current POM option set in `DEFAULT_POM_TOML`

Source: `crates/ff-desktop/src/menu_workspace/defaults.rs`
(`DEFAULT_POM_TOML`, the code-only Recovery_Baseline POM; CR-CH-021 -- never
written to disk).

Current options, in order:

| Order | key | command   | description                                               | group |
|-------|-----|-----------|-----------------------------------------------------------|-------|
| 1     | `0` | `Settings`| FFWB Settings and Client Parameters                       | Core  |
| 2     | `1` | `Catalogs`| Virtual File Catalogs -- Mainframe, POSIX, Native         | Core  |
| 3     | `2` | `Files`   | File Explorer -- Browse catalogs and files in a tree view | Core  |
| 4     | `3` | `Help`    | Help -- workbench help and topics                         | Core  |
| 5     | `X` | `X`       | Collapse to the visual root / close workspace (exit when last) | Core (`show_in_menu_bar = false`) |

**Count: 5 options.** Single group (`Core`).

**After this change (the firm requirement) the POM option set becomes:**

| Order | key | command   | description                          | group |
|-------|-----|-----------|--------------------------------------|-------|
| 1     | `0` | `Settings`| FFWB Settings and Client Parameters  | Core  |
| 2     | `3` | `Help`    | Help -- workbench help and topics    | Core  |
| 3     | `X` | `X`       | Collapse to the visual root / close workspace (exit when last) | Core (`show_in_menu_bar = false`) |

**Count: 3 options.** The `0`/`3`/`X` keys are retained unchanged (the owner
asked only to remove Catalogs and Files, not to renumber). NOTE a possible
owner-preference follow-up: whether `Help` should be renumbered `1`. The firm
requirement is only the REMOVAL; renumbering is NOT assumed. This is flagged for
the owner at the CONFIRM step.

`DEFAULT_SETTINGS_TOML` is UNAFFECTED (its options are C/A/T/M/K/W + R; no
Catalogs/Files rows).

---

## 2. Every file + test that asserts the POM option set or count

### 2a. SOURCE (ff-desktop) -- compiled constant and its unit tests
All in `crates/ff-desktop/src/menu_workspace/defaults.rs`. These are the
`DEFAULT_POM_TOML` constant and the tests that pin the 5-option set. They WILL
need source edits when the code change is implemented (NOT in this doc-only
step):

- `DEFAULT_POM_TOML` -- the constant itself (the `[[options]]` rows for key `1`
  Catalogs and key `2` Files must be deleted). Its doc-comment prose also names
  "Settings, Catalogs, Files" as always-reachable -- must be updated.
- `recovery_pom_menu_has_barebones_options` -- asserts
  `keys == ["0","1","2","3","X"]` and
  `commands == ["Settings","Catalogs","Files","Help","X"]`.
- `default_pom_toml_has_recovery_baseline_options` -- asserts
  `options.len() == 5` ("must contain exactly 5 options") and
  `commands == ["Settings","Catalogs","Files","Help","X"]`.
- `default_menubar_is_the_barebones_pom` -- equality of menubar to the POM
  (indirectly affected; the menubar IS the POM).
- `default_menubar_bar_visible_options_are_settings_catalogs_files_help` -- name
  and body assert bar-visible `["Settings","Catalogs","Files","Help"]`. Both the
  test NAME and the assertion change.
- `default_pom_toml_terminate_is_x_x`, `default_pom_toml_is_valid_toml`,
  `default_pom_toml_title_matches_spec`, `default_pom_toml_ascii_only` -- not
  option-set assertions but exercise the same constant (keep passing; terminate
  option `X` is retained).

### 2b. SOURCE (ff-desktop) -- shell tests coupled to POM option 1 (Catalogs) / option 2 (Files)
These tests depend on POM key `1` resolving to Catalogs and key `2` to Files.
Removing options `1` and `2` from the DEFAULT POM breaks them. They WILL need
source edits/retargeting when the code lands (NOT this step). Catalogued here so
the implementer sees every coupling:

- `crates/ff-desktop/src/shell/tests_nav_ladder.rs`:
  - `equals_1_works_from_a_non_pom_tab` -- asserts `=1` resolves to POM option 1
    (Catalogs -> `FilesPanel`). Breaks: there is no option `1` after removal.
- `crates/ff-desktop/src/shell/tests_nav.rs`:
  - `pom_key_resolves_to_configured_command` -- asserts POM key `2` -> `FILES` ->
    `FileExplorerPanel`. Breaks: no option `2`.
  - `pom_option_key_type_and_click_same_result` -- uses POM option 1 (command
    `Catalogs`). Breaks: no option `1`.
  - `handle_command("1")` / `handle_command("2.1")` / `handle_command("2")`
    usages that assume the POM has options 1 and 2 (see the nav tests around
    Requirement 14.10 and 19.4). Breaks or needs retargeting.
  - NOTE: `equals_files_jump_path_still_opens_file_explorer` and
    `bare_files_still_opens_file_explorer_via_front_door` test the bare `FILES`
    command / `=FILES` jump -- these DO NOT depend on the POM option and stay
    GREEN **iff** the FILES command remains reachable (see the design decision
    point: option B keeps them green; option A would also require editing them).
- `crates/ff-desktop/src/shell/tests_menu_workspace.rs`:
  - `menu_bar_has_file_catalogs_menu` -- asserts the default menu bar contains
    the `Catalogs` entry (POM option 1). Breaks: Catalogs removed from the POM
    (and therefore from the menu bar, which is the POM).
  - `menu_bar_has_help_and_excludes_terminate_option` -- asserts a `Help` entry
    exists and `X` is hidden; `Help` is retained, so only incidental.

### 2c. SOURCE (ff-desktop) -- NOT option-set assertions (kept as-is, context only)
- `crates/ff-desktop/src/workspace_kind/mod.rs` + `workspace_kind/tests.rs` --
  `BuiltinKind::Catalogs` / `BuiltinKind::Files`, titles `[CATALOGS]` / `[FILES]`
  and `resolve_base("catalogs")`. These are the Kind registry and are about the
  Contexts, not the POM menu. Affected ONLY if the owner chooses to ALSO remove
  the Contexts (design decision point, option A).
- `crates/ff-desktop/src/command_config/mod.rs` -- `builtin_workspace_target_for`
  classifies `FILES` / `FILE CATALOGS` / `CATALOGS` and the
  `BUILTIN_WORKSPACE_VERB_ROOTS` list. Affected ONLY under option A (remove the
  commands). Under option B these stay so typed `FILES`/`CATALOGS` still work.
- `crates/ff-desktop/src/menu_workspace/i18n_keys.rs` -- localisation key tests
  referencing a `Catalogs` option. Affected if the option is removed from the
  POM (the i18n key fixtures use `pom`+`Catalogs`); these are fixture strings,
  not the live POM, so they can stand, but noted.
- `crates/ff-desktop/src/shell/tests_session.rs` / `tests_split_detach.rs` /
  `tests_scrm.rs` -- use `FILES` / `FilesPanel` via the COMMAND or descriptor,
  not via a POM option; unaffected by the menu-option removal (affected only
  under option A).

### 2d. DOCS -- spec criteria and tracking
- `docs/specs/menu-workspace/requirements.md`:
  - **Req 12.2** (Recovery_Baseline POM option set) -- lists `0 SETTINGS`,
    `1 CATALOGS`, `2 FILES`, `3 HELP`, `X RETURN`. PRIMARY criterion to edit.
  - **Req 17.2 / 17.2a** prose (lines ~1125-1128) -- "barebones POM SHALL
    contain, in order, `SETTINGS`, `CATALOGS`, `FILES`, `HELP`, `RETURN`" and
    "bar-visible entries are Settings, Catalogs, Files". Must be edited.
  - Req 12 User Story prose ("reach Settings, Catalogs, Files, the event Log...")
    -- update the narrative.
- `docs/specs/startup-and-session/requirements.md`:
  - **Req 14.3** (the POM built-in option list) -- lists `0 Settings`,
    `1 File Catalogs`, `2 Files`, `3 Utilities`, ... plus `14.3a` (option 1 =
    File Catalogs) and menu-bar mirror criterion 14.7. This spec's Req 14.3 is a
    LEGACY list (0-8 + 9/S/B, "unchanged from Phase AC") that is already
    INCONSISTENT with the live 5-option barebones POM. It must be reconciled to
    the barebones set and the Catalogs/Files entries removed.
  - **Req 11.8 / 11.9** reference the Recovery_Baseline fallback (menu-workspace
    Req 12) -- consistent by reference; no option list inline, so no edit needed
    beyond confirming the cross-reference still holds.
- `docs/quality/TCR.md`:
  - The row recording "compiled Recovery_Baseline (**POM 0/1/2/L/M/X**; Settings
    T/M/A)" -- the POM option-set shorthand is STALE (it says 0/1/2/L/M/X; the
    live set is 0/1/2/3/X). Must be updated to the new `0/3/X` set.
  - The menu-bar rows referencing
    `default_menubar_bar_visible_options_are_settings_catalogs_files_help` and
    `menu_bar_has_file_catalogs_menu` -- note the test-name/assertion change.
  - Add a NOT-COVERED row per new/edited criterion.
- `docs/status/change-log.md` -- append the CR-CH-073 entry.
- `docs/project-management/ROADMAP.md` -- check for a recorded POM option set
  (control-tower doc); update if it records the barebones set.

---

## 3. Affected EARS criteria ids

| Spec | Criterion | Nature of edit |
|------|-----------|----------------|
| menu-workspace | Req 12.2 | Remove `1 CATALOGS` and `2 FILES` from the Recovery_Baseline POM option set; restate the set as `0 SETTINGS`, `3 HELP`, `X X`; drop count-of-5 language. |
| menu-workspace | Req 17.2 / 17.2a (prose) | Update the barebones-POM order and the bar-visible list to drop Catalogs/Files. |
| menu-workspace | Req 12 User Story | Update narrative ("reach Settings, Help..."). |
| startup-and-session | Req 14.3 (+ 14.3a, 14.7) | Reconcile the stale legacy option list to the barebones set; remove File Catalogs/Files option rows and the 14.3a option-1 criterion; update the menu-bar mirror list. |
| (tracking) TCR.md | POM option-set shorthand row | `0/1/2/L/M/X` -> `0/3/X`; add NOT-COVERED rows. |

## 4. Design decision point to raise with the owner (do NOT decide here)

Removing the menu OPTIONS is firm. OPEN QUESTION: should the underlying
`FILES` / `CATALOGS` commands and their Workspace Contexts (File Explorer /
Catalog Explorer) ALSO be removed/disabled, or should ONLY the default menu
options be removed while the commands remain reachable if typed?

- **Option A -- remove menu options AND the commands/Contexts.** Truly strips the
  capability from the bare core. Larger blast radius: touches
  `builtin_workspace_target_for`, `BUILTIN_WORKSPACE_VERB_ROOTS`, the
  `BuiltinKind::Catalogs`/`Files` registry, `TabKind`/`KindTag` arms, descriptor
  reconstruction, and the `=FILES`/bare-FILES tests. Risk of reintroducing
  "unknown command" errors elsewhere.
- **Option B -- remove ONLY the menu options; keep the commands reachable.**
  Minimal, surgical: edit only `DEFAULT_POM_TOML` + its option-set tests + the
  two menu-bar tests. Typed `FILES`/`CATALOGS` and `=FILES` still work (those
  tests stay green). Matches the literal firm requirement ("remove the menu
  options").

The requirements/design docs are written to the FIRM requirement (remove the
options) and raise this as an explicit owner decision; the DESIGN records both
options with the tradeoffs and leaves the choice to the owner. The gate CONFIRM
step asks the owner to pick A or B and records it in
`.agents/tasks/strip-core-remove-catalogs-files/gate-approved.json`
(`removeCommands: true` = A, `false` = B).
