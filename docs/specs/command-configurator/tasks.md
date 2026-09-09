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
- [ ] 3.3 Implement `${workspace_root}` / `${file_dir}` placeholder expansion in program/args/working_dir
  - Satisfies: Req 3.6
- [ ] 3.4 Apply `shell.mode` gate + prompt-confirm to both modes; handle spawn-launch failure
  - Satisfies: Req 3.7, 3.8, 3.9
- [ ] 3.5 Apply working-directory resolution (explicit, else shell.working_directory rules)
  - Satisfies: Req 3.5
- [ ] 3.6 Write unit/integration tests: detached spawns and returns without capture; captured shows stdout/stderr/exit in Output_Panel; disabled refuses; prompt-declined does not spawn; placeholder expansion; visible-workspace classification
  - Validates: Requirement 3.2, 3.4, 3.6, 3.7, 3.8, 3.10

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
