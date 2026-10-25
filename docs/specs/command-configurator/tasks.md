# Implementation Plan -- Command Configurator

Spec status: APPROVED (Phase DB gate). Implementation in progress under DB.9.
Tasks 1 and 2 (the data + resolution layer) are complete and tested. The
ff-shell half of external execution (Task 3.1/3.2, DB.10) is complete and tested.
The Context UI (Task 4) and the ff-desktop adapter share of external execution
(Task 3.3-3.6: placeholder expansion, prompt-confirm UI, adapter tests) are
still `[ ]`. Each task references the acceptance criteria it satisfies.

---

## Task 1: Command Store file format and loader (DB.9)

- [x] 1.1 Define `CommandDefinition` and `CommandStore` types in `command_config/mod.rs`
  - Satisfies: Req 1.2, 1.3
- [x] 1.2 Implement `commands.toml` load in `command_config/store.rs` (skip invalid entries, keep-first on duplicate id, populate load_error)
  - Satisfies: Req 1.1, 1.4, 1.6
- [x] 1.3 Implement store save (write full definition list, create commands/ dir on first save)
  - Satisfies: Req 1.5, 2.4, 2.5
- [x] 1.4 Implement hot-reload poll (mtime check) reusing the Menu Workspace pattern
  - Satisfies: Req 1.7
- [x] 1.5 Write unit tests: valid round-trip, invalid entry skipped, duplicate id kept-first, absent file = empty store
  - Validates: Requirement 1.1, 1.4, 1.5, 1.6
  - NOTE: Req 1.8 (ASCII default content) not covered -- there is no default
    `commands.toml` generator yet; it lands with the Context UI (Task 4).

## Task 2: Target resolution integration (DB.9)

- [x] 2.1 Expose loaded definitions to `ff_command::resolve_target` as a `UserCommandStore` view
  - Satisfies: command-framework Req 8.3, command-configurator Req 4.3
- [x] 2.2 Validate definitions and reject ids that shadow a reserved built-in Command_ID
  - Satisfies: Req 4.1, 4.2, 4.6
- [x] 2.3 Write unit tests: id resolves to stored target; reserved-id conflict rejected; unknown id does not resolve
  - Validates: Requirement 4.1, 4.2, 4.6, command-framework Requirement 8.3
  - NOTE: Req 4.5's user-facing `Command '<id>' is not defined.` message and Req
    4.4 shortcut binding are wired with the menu/shortcut integration (DB.4).

## Task 3: External execution modes (Detached / Captured)

- [x] 3.1 Add `ff_shell::spawn_detached(program, args, working_dir)` returning a dropped `TaskHandle` seam (DB.10)
  - Satisfies: Req 3.2, 3.3
  - DONE: `ff_shell::spawn_detached` + `TaskHandle`/`ExecutionMode`/`ExternalOutcome`
    in `crates/ff-shell/src/executor/external.rs`; `ShellEngine::spawn_detached`
    applies the shell.mode gate and working_dir fallback.
- [x] 3.2 Route Captured External targets through the existing `shell.execute` async path into the Output_Panel (DB.10)
  - Satisfies: Req 3.4, shell-command Req 19
  - DONE: `ShellEngine::execute_external` (Captured) reuses `CommandExecutor` and
    appends an `OutputEntry` (stdout/stderr + exit code) to the Output_Panel.
- [x] 3.3 Implement `${workspace_root}` / `${file_dir}` placeholder expansion in program/args/working_dir
  - Satisfies: Req 3.6
  - DONE: `WorkbenchShell::expand_external_placeholders` (`shell/external_adapter.rs`);
    unresolved placeholder -> empty string + DEBUG log.
- [x] 3.4 Apply `shell.mode` gate + prompt-confirm to both modes; handle spawn-launch failure
  - Satisfies: Req 3.7, 3.8, 3.9
  - DONE: `run_external_target` gates on `shell.mode` (disabled refuses; prompt
    stages `pending_external`; enabled runs); the confirmation dialog in
    `shell/update.rs` runs on confirm / skips on cancel; launch failure surfaces
    a status message.
- [x] 3.5 Apply working-directory resolution (explicit, else shell.working_directory rules)
  - Satisfies: Req 3.5
  - DONE: explicit `working_dir` passed through; `None` falls back via the
    ff-shell `WorkingDirResolver` (project_root / active_file plumbed from shell state).
- [x] 3.6 Write unit/integration tests: detached spawns and returns without capture; captured shows stdout/stderr/exit in Output_Panel; disabled refuses; prompt-declined does not spawn; placeholder expansion; visible-workspace classification
  - Validates: Requirement 3.2, 3.4, 3.6, 3.7, 3.8, 3.10
  - DONE: 6 adapter tests in `shell/tests.rs` (disabled/prompt/enabled/captured/
    launch-failure/placeholder) plus the ff-shell engine tests.
  - NOTE: a full dockable Output_Panel view in ff-desktop is a follow-up UI task;
    captured output is captured into the engine's Output_Panel (asserted via
    `output_entry_count`) and the run reports a status pointing to it.

## Task 4: Command Configurator Context

- [x] 4.1 Add `WorkspaceKind::CommandConfigurator` (session-layer) and runtime `TabKind::CommandConfigurator`
  - Satisfies: Req 2.1, 2.7
  - DONE: `TabKind::CommandConfigurator` + `command_configurator()` constructor
    (`tab_state.rs`); `WorkspaceKind::CommandConfigurator` already present in
    `ff-session`; `context_name_for_kind` -> `"commands"`.
- [x] 4.2 Implement `COMMANDS` primary command to open the Context; F3/END returns to POM
  - Satisfies: Req 2.1, 2.8
  - DONE: `COMMANDS` arm in `handle_command` (transform-or-open, title
    `[COMMANDS]`); END/F3 sets `pending_return_to_pom` to transform back to POM.
- [x] 4.3 Render the definitions table (id, label, variant, external mode) in `command_config/render.rs`
  - Satisfies: Req 2.2
  - DONE: `render.rs` grid with id/label/variant/mode columns.
- [x] 4.4 Implement Add / Edit / Delete with the variant-specific target editor in `command_config/edit.rs`
  - Satisfies: Req 2.3, 2.6
  - DONE: `edit.rs` `EditForm`/`TargetVariant` variant-specific editor; delete
    confirmation window; Save applied by `shell/configurator.rs`.
- [x] 4.5 Implement definition validation (id rules, label, target deserialise, external program/mode)
  - Satisfies: Req 4.1, 4.2
  - DONE: `EditForm::build` field checks + `store::validate_definition`
    (id/label/reserved/external-program) on commit.
- [x] 4.6 Write unit tests: add/edit/delete round-trips through the store; validation rejects bad id/label/target/program/mode; title is [COMMANDS]
  - Validates: Requirement 2.3, 2.4, 2.5, 4.1, 4.2
  - DONE: 8 shell tests (`shell/tests.rs`) + `render.rs`/`edit.rs` unit tests.

## Task 5: Binding integration

- [x] 5.1 Allow a Menu_Option `command` value to resolve to a Command_Definition id (menu-workspace Req 10) (DB.4)
  - Satisfies: Req 4.3
  - DONE: menu-option dispatch (typed + click) resolves via `ShellTargetResolver`
    (`shell/target_dispatch.rs`); an inline `[options.target]` takes precedence
    (menu-workspace Req 10.6). Built-in verbs fall through unchanged (Req 10.2).
- [x] 5.2 Allow a keyboard Shortcut_Binding to target a Command_Definition id (DB.4)
  - Satisfies: Req 4.4
  - DONE: function-key and key-label-bar dispatch route through
    `dispatch_bound_command`; `run_command_definition` reports
    `Command '<id>' is not defined.` for a missing id (Req 4.5).
- [x] 5.3 Write tests: menu option runs the definition's target; shortcut runs the definition's target (DB.4)
  - Validates: Requirement 4.3, 4.4, 4.5
  - DONE: 11 tests across `shell/tests.rs` and `menu_workspace/loader.rs`.
  - NOTE: executing an External/Menu/CustomWorkspace/Macro target from a binding
    still reports a deferred status; those land with the `MENU` command and the
    ff-shell external desktop adapter (command-configurator Req 3).

## Task 6: Persistence and documentation

- [x] 6.1 Persist the Command Configurator Context as `CustomWorkspace { workspace_kind: CommandConfigurator }`; ensure Started_Tasks and captured output tabs are not persisted
  - Satisfies: Req 3.10, startup-and-session Req 21
  - DONE: `descriptor_for_tab` maps `TabKind::CommandConfigurator` to
    `CustomWorkspace { CommandConfigurator }`; `restore_workspace_descriptors`
    reopens it. Started_Tasks / captured runs are not tabs, so not persisted.
- [ ] 6.2 Update `docs/quality/TCR.md` -- set command-configurator rows to correct status
- [ ] 6.3 Update `docs/specs/project-master/tasks.md` -- mark command-configurator tasks complete

## Task 7: Alias Editor Context (CR-NR-113, Requirement 5)

Add an Alias Editor Workspace Context to create, edit, and SAVE command aliases
(including CR-NR-112 parameterised aliases) into a user-scoped alias store.
Modelled on the Menus Editor (`shell/menus_editor.rs`). Follow TDD (write the
failing test first); keep every touched file under 400 non-test lines.

BUILD-ORDER PREREQUISITE (hard dependency on CR-NR-112, design 9.0): the
template-dependent subtasks -- 7.2 (the serialiser's `command_template` handling),
the `command_template` field/edit in 7.3, and 7.6.2 (the end-to-end `ecob helloW`
expansion) -- MUST NOT be started until CR-NR-112 has landed
`AliasEntry.command_template`, the `AliasRowToml.command_template` field, and the
`&1..&N` / `&*` substitution in the pre-pass (`command_template` exists nowhere in
source today). These subtasks carry a CR-NR-112-LANDED note. The NON-template
subset (store path/load, surface+canonical+environment authoring, save, refresh,
focus, scope) has no such dependency and may be built first.

- [ ] 7.1 Define the User_Alias_Store location, format, and loader
  - [ ] 7.1.1 Add a user alias store path under the workbench data dir (`<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`), locale-independent, separate from `i18n/<locale>/aliases/`
    - Validates: Requirement 5.1
  - [ ] 7.1.2 Add a User_Alias_Store LOAD path to `LocaleAliasStore` as a THIRD overlay on top of the identity base + per-locale overlay, reading the user file via the NEW `parse_user_alias_toml` / `UserAliasRowToml` (design 9.4, not the 3-field `parse_alias_toml`) and routing each row into its target environment's AliasTable (`environment` field, default `ffcmd`), reusing the collision-rejecting overlay seam; skip-with-WARN a row whose surface equals a reserved built-in root (a `builtin_ffcmd_roots()` member -- a NEW CR-NR-113 pre-pass invariant, NOT command-framework Req 8.10); on a cross-layer surface clash with the per-locale overlay, no-op when the root is the SAME and skip-with-WARN (per-locale wins) when the root DIFFERS. EXTEND the re-seed path (`reload_active_alias_store_for_locale` / `seed_and_overlay_for_locale`) so one re-seed re-applies BOTH the per-locale overlay AND the user-store overlay
    - Validates: Requirement 5.6, 5.7
  - [ ] 7.1.3 Write failing unit tests: absent store -> empty + created on first save; a loaded user alias resolves through the resolver; an `environment = "ffedit"` row resolves in FFEDIT and a defaulted row in FFCMD; a built-in-root-shadowing row is skipped with a WARN; a cross-layer same-root clash is a no-op and a different-root clash keeps the per-locale surface
    - Validates: Requirement 5.1, 5.7

- [ ] 7.2 Alias serialiser (rows -> TOML) -- CR-NR-112-LANDED for the `command_template` handling (design 9.0)
  - [ ] 7.2.1 Write failing round-trip unit tests: a row with a template serialises to `[[alias]]` with `command_template` (CR-NR-112-LANDED); a pure verb row omits `command_template` and `localisation_key`; a non-default `environment` round-trips and `ffcmd` is the serialised/parsed default; serialise-then-parse THROUGH THE NEW `parse_user_alias_toml` / `UserAliasRowToml` (design 9.4) -- NOT the 3-field `parse_alias_toml`, which has no `environment` and would drop it -- yields the same rows
    - Validates: Requirement 5.1, 5.4
  - [ ] 7.2.2 Implement the serialiser AND the matching user-store row type/parser (none exists today; `alias_reader.rs` only reads via the 3-field `parse_alias_toml`). Add a NEW `UserAliasRowToml` (`surface`, `canonical`, `#[serde(default)] environment`, optional `command_template`, optional `localisation_key`) + `parse_user_alias_toml`, leaving the existing `AliasRowToml` / `parse_alias_toml` UNCHANGED; the serialiser emits `surface`, `canonical`, the `environment` when not the `ffcmd` default, and `command_template` when non-empty (the template emit is CR-NR-112-LANDED-gated)
    - Validates: Requirement 5.1, 5.4

- [ ] 7.3 Alias Editor state, pure render, and WorkspaceContext impl
  - [ ] 7.3.1 Add `AliasEditorState` + `AliasRowEdit` + `AliasEditorAction` (modelled on `MenusEditorAction`), held in a grouped shell sub-struct
    - Validates: Requirement 5.1, 5.3
  - [ ] 7.3.2 Implement the pure `alias_editor_panel::render` returning an `AliasEditorAction`; apply per-field edits directly on the working copy (mirroring B054)
    - Validates: Requirement 5.3
  - [ ] 7.3.3 Implement `WorkspaceContext for AliasEditorState` returning `InteriorFocus` with a stable first-control `egui::Id`; dispatch via the owned-panel swap in the central-panel arm
    - Validates: Requirement 5.10

- [ ] 7.4 Shell-side effects: open, apply, save, refresh
  - [ ] 7.4.1 Implement `open_alias_editor` (load rows, `navigate_to` a `CommandConfigurator` `editor = "aliases"` descriptor in place)
    - Validates: Requirement 5.2, 5.9
  - [ ] 7.4.2 Implement `apply_alias_editor_action` (AddRow / DeleteRow / Save)
    - Validates: Requirement 5.3, 5.5
  - [ ] 7.4.3 Implement `save_user_aliases`: validate (empty surface/canonical, within-SAME-environment surface->two-roots collision, built-in-root shadowing via `builtin_ffcmd_roots()`), leave the on-disk file unchanged on any validation/write failure, serialise, write, then refresh the live `LocaleAliasStore` in place via the EXTENDED `ACTIVE_ALIAS_STORE` reload seam (one re-seed re-applies BOTH the per-locale overlay AND the user-store overlay, 7.1.2)
    - Validates: Requirement 5.4, 5.5, 5.6, 5.7
  - [ ] 7.4.4 Write failing unit tests for the save path: each validation rejection; file-unchanged-on-failure; a valid save writes the file and the live store resolves the new alias; after save-then-reload the live store STILL carries BOTH the per-locale overlay AND the new user alias (re-seed drops neither layer)
    - Validates: Requirement 5.4, 5.6

- [ ] 7.5 Registration (configurator family, full sibling parity) and the open verb
  - NOTE: "no new WorkspaceKind" does NOT mean "no new TabKind" -- the Alias
    Editor needs the SAME wiring points as the Menus Editor (design 9.6), not
    only the descriptor arm.
  - [ ] 7.5.1 Add the runtime `TabKind::AliasEditor` + `KindTag::AliasEditor` (`tab_state.rs`), the `BuiltinKind` mapping (`workspace_kind/mod.rs`), the `session_manager.rs` non-persist arms, and the tab constructor (`tab_state_ctors.rs`)
    - Validates: Requirement 5.2, 5.9
  - [ ] 7.5.2 Add the `editor = "aliases"` descriptor arm in `shell/nav_stack.rs` (`descriptor_for_current_context`) and the reconstruct arm in `nav_reconstruct.rs` (`reconstruct_custom`); add the shell panel field for `AliasEditorState` in `construct.rs` and the dedicated central-panel render arm in `shell/render_body.rs` (mem::take -> `render_workspace_context` -> put back -> `apply_alias_editor_action`). No new WorkspaceKind, no new dispatcher, no new persistence format
    - Validates: Requirement 5.2, 5.9, 5.10
  - [ ] 7.5.3 Register the `ALIASES` verb (classified as a `CustomWorkspace` target through the single front door) and/or a menu option; show `[ALIASES]` in the tab title; END/F3 returns via the Navigation_Stack
    - Validates: Requirement 5.2, 5.9

- [ ] 7.6 Full-shell tests and end-to-end expansion
  - [ ] 7.6.1 Write the mandatory full-shell egui_kittest first-Tab test: first Tab from the command field lands on the reported first interior control (no phantom stop)
    - Validates: Requirement 5.10
  - [ ] 7.6.2 (CR-NR-112-LANDED) Write a full-shell behavioural test: author `ECOB`/`EDIT`/`FFWB.COBOL.MASTER(&1)` (environment `ffedit`) in the editor, Save, type `ecob helloW`, assert expansion to `EDIT FFWB.COBOL.MASTER(helloW)` (end-to-end with CR-NR-112; blocked until the CR-NR-112 template expansion is live, design 9.0)
    - Validates: Requirement 5.8

- [ ] 7.7 Scope guard and coverage bookkeeping
  - [ ] 7.7.1 (Verification, no code) Confirm the editor reads/writes ONLY the User_Alias_Store and never the per-locale `i18n/<locale>/aliases/` catalogues (CR-NR-111 non-overlap)
    - Covers: Requirement 5.11
  - [ ] 7.7.2 Update `docs/quality/TCR.md`: set the CR-NR-113 command-configurator Req 5.1-5.12 rows to their correct status as each test lands
    - Covers: Requirement 5 (all criteria)
