# Gate Report -- Strip core: trim the default POM (remove Catalogs + Files; reconsidered core set)

**Change class:** CHANGE REQUEST (modifies working behaviour -- the default menu
option set). Logged as **CR-CH-073**.
**Spec area:** `menu-workspace` (with a consistency edit in `startup-and-session`).
**Gate status:** DOCS AUTHORED -- awaiting owner CONFIRM. NO source code edited.
**Coordination:** SUBTRACTIVE half of a two-effort split. The re-introduction of
Catalogs/Files as a first-class EXTENSION is the separate effort
**wf_21b74c6aa15ba293**; this effort only removes them from the core now.

---

## (a) Proposed trimmed default POM option set (the reconsider)

Current compiled Recovery_Baseline POM (`DEFAULT_POM_TOML`, 5 options):
`0 Settings`, `1 Catalogs`, `2 Files`, `3 Help`, `X` terminate.

**Proposed reconsidered trimmed core (3 options, single `Core` group):**

| Order | key | command    | description                                                    | show_in_menu_bar |
|-------|-----|------------|----------------------------------------------------------------|------------------|
| 1     | `0` | `Settings` | FFWB Settings and Client Parameters                            | true (default)   |
| 2     | `3` | `Help`     | Help -- workbench help and topics                              | true (default)   |
| 3     | `X` | `X`        | Collapse to the visual root / close workspace (exit when last) | false (hidden)   |

Reasoning:
- `SETTINGS` -- the recovery gateway (reaches the Menus / Keys / Theme / Kinds
  editors and Reset-bare); keeps a bare/broken config recoverable.
- `HELP` -- orientation; resolves by name (the `HELP` verb); cheap to keep.
- `X` -- the uniform terminate / close verb (Req 2.1g); hidden from the Menu_Bar.
- REMOVED: `Catalogs`, `Files` (reserved for the future extension).
- NOT added: `MENUS`/`LOG` (already under Settings / reachable by command); no
  Extensions/`PLUGINS` option (that belongs to the future extension effort).

Keys `0`/`3`/`X` kept as-is (no renumber). Renumbering `3 Help` -> `1` is an
OPTIONAL owner preference, not part of the firm change. Every remaining option's
command (`SETTINGS`, `HELP`, `X`) resolves by NAME (Req 2.1h). Trimming the POM
auto-trims the default Menu_Bar to bar-visible `Settings`, `Help` (the bar IS the
POM, single source of truth; Req 17.2/17.2a).

---

## (b) Command / Context decision point (recommendation)

Removing the menu OPTIONS is the firm requirement. Open question: should the bare
core ALSO remove/disable the underlying `FILES` / `CATALOGS` commands and their
Contexts (File Explorer / Catalog Explorer) now?

- **Option A** -- remove options AND commands/Contexts. Truly strips the
  capability but has a large blast radius (`builtin_workspace_target_for`,
  `BUILTIN_WORKSPACE_VERB_ROOTS`, `BuiltinKind::Catalogs`/`Files`, `TabKind` /
  `KindTag`, descriptor reconstruction, the `=FILES`/`FILES` tests) and would
  collide with the extension effort, which needs the Context machinery to re-home.
- **Option B (RECOMMENDED)** -- remove ONLY the default menu options; keep the
  commands and Contexts reachable if typed. Minimal and surgical (edit
  `DEFAULT_POM_TOML` + its option-set tests + two menu-bar tests). Typed `FILES` /
  `CATALOGS` and the `=FILES` jump still work; the extension effort
  (wf_21b74c6aa15ba293) owns re-homing the Context machinery without a prior
  teardown/rebuild. Matches the literal firm requirement.

**DECISION: Option B -- owner-APPROVED** (`removeCommands: false`), recorded in
`.agents/tasks/strip-core-pom-trim/gate-approved.json`. Remove ONLY the default
menu options; keep the `FILES`/`CATALOGS` commands and Contexts reachable if
typed. The optional `3 HELP` -> `1` renumber is DECLINED (Help stays on key `3`).

---

## (c) Documents authored (docs-only; ASCII-clean)

1. `docs/specs/menu-workspace/requirements.md` -- Req 12 User Story + **Req 12.2**
   restated as the reconsidered trimmed core set; **Req 2.1h** example updated off
   `CATALOGS`; Req 17.2a already consistent (SETTINGS/HELP/terminate).
2. `docs/specs/startup-and-session/requirements.md` -- **Req 14.3** barebones list
   reconciled to the reconsidered set (mirrors menu-workspace Req 12.2).
3. `docs/specs/menu-workspace/design.md` -- NEW **CR-CH-073** section: the trimmed
   set + reasoning, the A/B decision point, retention-of-command/Context note,
   error/edge cases, testability, and the full source-change scope for the
   implement step.
4. `docs/specs/menu-workspace/tasks.md` -- NEW **Phase (strip-core-pom-trim)**,
   tasks 51.1-51.9 (`[ ]`/`[x]` only, titled, Req cross-refs).
5. `docs/quality/TCR.md` -- corrected the stale Recovery_Baseline POM option-set
   shorthand (`POM 0/1/2/L/M/X` -> trimmed `0 Settings / 3 Help / X`) and added
   NOT-COVERED rows for Req 12.2, 2.1h, 17.2a, startup 14.3, and the Option B
   regression guard.
6. `docs/status/change-log.md` -- appended the **CR-CH-073** entry.
7. `.agents/tasks/strip-core-pom-trim/gate.md` -- this report.

Supersedes the unapproved prior draft `.agents/tasks/strip-core-remove-catalogs-files/`
(same CR-CH-073; that effort authored the requirements edits but was never
approved/implemented -- `DEFAULT_POM_TOML` still carries Catalogs/Files and no
`gate-approved.json` exists). This effort reuses CR-CH-073 and adds the
reconsidered-set framing plus the design/tasks/TCR/change-log it left unfinished.

---

## Source assertion sites the IMPLEMENT step must change (Option B)

`crates/ff-desktop/src/menu_workspace/defaults.rs`:
- `DEFAULT_POM_TOML` constant -- delete the key `1` Catalogs and key `2` Files
  `[[options]]` blocks; update the doc-comment prose.
- `recovery_pom_menu_has_barebones_options` -- `keys == ["0","3","X"]`,
  `commands == ["Settings","Help","X"]`.
- `default_pom_toml_has_recovery_baseline_options` -- `len == 3`,
  `commands == ["Settings","Help","X"]`.
- `default_menubar_bar_visible_options_are_settings_catalogs_files_help` -- RENAME
  to `..._settings_help`; body asserts `["Settings","Help"]`.
- `default_menubar_is_the_barebones_pom`, `default_pom_toml_terminate_is_x_x`,
  `default_pom_toml_is_valid_toml`, `default_pom_toml_title_matches_spec`,
  `default_pom_toml_ascii_only` -- stay green; re-verify.

Shell / menu tests coupled to POM options `1`/`2`:
- `crates/ff-desktop/src/shell/tests_nav_ladder.rs::equals_1_works_from_a_non_pom_tab`.
- `crates/ff-desktop/src/shell/tests_nav.rs::pom_key_resolves_to_configured_command`,
  `::pom_option_key_type_and_click_same_result`, and `handle_command("1"/"2"/"2.1")`
  POM-option assertions.
- `crates/ff-desktop/src/shell/tests_menu_workspace.rs::menu_bar_has_file_catalogs_menu`
  -- retire/retarget.
- GREEN guards (Option B): `equals_files_jump_path_still_opens_file_explorer`,
  `bare_files_still_opens_file_explorer_via_front_door` -- confirm still pass.

Additional sites ONLY if the owner picks Option A:
- `crates/ff-desktop/src/command_config/mod.rs` -- `builtin_workspace_target_for`
  (`FILES`/`FILE CATALOGS`/`CATALOGS` arms) + `BUILTIN_WORKSPACE_VERB_ROOTS` +
  its drift-guard test.
- `crates/ff-desktop/src/workspace_kind/mod.rs` + `workspace_kind/tests.rs` --
  `BuiltinKind::Catalogs`/`Files`, `[CATALOGS]`/`[FILES]` titles,
  `resolve_base("catalogs")`.
- `TabKind`/`KindTag` arms + descriptor reconstruction for those Contexts;
  `=FILES`/`FILES` tests become removals rather than green guards.
