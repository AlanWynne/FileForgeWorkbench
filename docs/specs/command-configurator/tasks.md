# Implementation Plan -- Command Configurator

Spec status: DRAFT, pending approval (Phase DB gate). No source is written until
the gate is approved and a separate implementation instruction is given. All
tasks are `[ ]` (pending). Each references the acceptance criteria it satisfies.

---

## Task 1: Command Store file format and loader

- [ ] 1.1 Define `CommandDefinition` and `CommandStore` types in `command_config/mod.rs`
  - Satisfies: Req 1.2, 1.3
- [ ] 1.2 Implement `commands.toml` load in `command_config/store.rs` (skip invalid entries, keep-first on duplicate id, populate load_error)
  - Satisfies: Req 1.1, 1.4, 1.6
- [ ] 1.3 Implement store save (write full definition list, create commands/ dir + empty file on first save)
  - Satisfies: Req 1.5, 2.4, 2.5
- [ ] 1.4 Implement hot-reload poll (mtime check) reusing the Menu Workspace pattern
  - Satisfies: Req 1.7
- [ ] 1.5 Write unit tests: valid round-trip, invalid entry skipped, duplicate id kept-first, absent file = empty store, ASCII-only default content
  - Validates: Requirement 1.1, 1.4, 1.5, 1.6, 1.8

## Task 2: Target resolution integration

- [ ] 2.1 Expose loaded definitions to `ff_command::resolve_target` as a `UserCommandStore` view
  - Satisfies: command-framework Req 8.3
- [ ] 2.2 Reject definitions whose id shadows a reserved built-in Command_ID
  - Satisfies: Req 4.6
- [ ] 2.3 Write unit tests: id resolves to stored target; reserved-id conflict rejected; unknown id reports `Command '<id>' is not defined.`
  - Validates: Requirement 4.5, 4.6, command-framework Requirement 8.3

## Task 3: External execution modes (Detached / Captured)

- [ ] 3.1 Add `ff_shell::spawn_detached(program, args, working_dir)` returning a dropped `TaskHandle` seam
  - Satisfies: Req 3.2, 3.3
- [ ] 3.2 Route Captured External targets through the existing `shell.execute` async path into the Output_Panel
  - Satisfies: Req 3.4, shell-command Req 19
- [ ] 3.3 Implement `${workspace_root}` / `${file_dir}` placeholder expansion in program/args/working_dir
  - Satisfies: Req 3.6
- [ ] 3.4 Apply `shell.mode` gate + prompt-confirm to both modes; handle spawn-launch failure
  - Satisfies: Req 3.7, 3.8, 3.9
- [ ] 3.5 Apply working-directory resolution (explicit, else shell.working_directory rules)
  - Satisfies: Req 3.5
- [ ] 3.6 Write unit/integration tests: detached spawns and returns without capture; captured shows stdout/stderr/exit in Output_Panel; disabled refuses; prompt-declined does not spawn; placeholder expansion; visible-workspace classification
  - Validates: Requirement 3.2, 3.4, 3.6, 3.7, 3.8, 3.10

## Task 4: Command Configurator Context

- [ ] 4.1 Add `WorkspaceKind::CommandConfigurator` (session-layer) and runtime `TabKind::CommandConfigurator`
  - Satisfies: Req 2.1, 2.7
- [ ] 4.2 Implement `COMMANDS` primary command to open the Context; F3/END returns to POM
  - Satisfies: Req 2.1, 2.8
- [ ] 4.3 Render the definitions table (id, label, variant, external mode) in `command_config/render.rs`
  - Satisfies: Req 2.2
- [ ] 4.4 Implement Add / Edit / Delete with the variant-specific target editor in `command_config/edit.rs`
  - Satisfies: Req 2.3, 2.6
- [ ] 4.5 Implement definition validation (id rules, label, target deserialise, external program/mode)
  - Satisfies: Req 4.1, 4.2
- [ ] 4.6 Write unit tests: add/edit/delete round-trips through the store; validation rejects bad id/label/target/program/mode; title is [COMMANDS]
  - Validates: Requirement 2.3, 2.4, 2.5, 4.1, 4.2

## Task 5: Binding integration

- [ ] 5.1 Allow a Menu_Option `command` value to resolve to a Command_Definition id (menu-workspace Req 10)
  - Satisfies: Req 4.3
- [ ] 5.2 Allow a keyboard Shortcut_Binding to target a Command_Definition id
  - Satisfies: Req 4.4
- [ ] 5.3 Write tests: menu option runs the definition's target; shortcut runs the definition's target
  - Validates: Requirement 4.3, 4.4

## Task 6: Persistence and documentation

- [ ] 6.1 Persist the Command Configurator Context as `CustomWorkspace { workspace_kind: CommandConfigurator }`; ensure Started_Tasks and captured output tabs are not persisted
  - Satisfies: Req 3.10, startup-and-session Req 21
- [ ] 6.2 Update `docs/quality/TCR.md` -- set command-configurator rows to correct status
- [ ] 6.3 Update `docs/specs/project-master/tasks.md` -- mark command-configurator tasks complete
