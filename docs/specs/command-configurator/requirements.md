# Requirements Document -- Command Configurator

## Introduction

The Command Configurator lets a user define, store, edit, and delete named
commands from within FileForge Workbench, and bind them to menu options and
keyboard shortcuts. Each definition is a named Command_Target
(command-framework Requirement 8) that may point at internal FFWB behaviour
(a menu, a custom workspace, an internal function, or a macro) or at an external
program (a PowerShell / batch / bash script, a Python script, or any executable).

Definitions are stored in a data-driven, hot-reloadable TOML file at
`<User_Data_Dir>/commands/commands.toml`, in the same spirit as the `menus/`
directory used by the Menu Workspace pattern. The Command Configurator itself is
a Custom Workspace (a new Context) that presents a form over that file.

External commands run in one of two modes:

- **Detached** -- a fire-and-forget Started_Task. FFWB spawns the process and
  forgets it: no output is captured, no Workspace is opened, and the task is
  never persisted or restarted. Monitoring a running Started_Task is out of
  scope (the operating system's own task manager owns process lifecycle).
- **Captured** -- FFWB runs the process asynchronously without blocking the UI,
  collects its stdout, stderr, and exit code, and displays them in the existing
  shell Output_Panel (shell-command Requirement 15).

All external execution is gated by the existing `shell.mode` configuration
(`disabled` | `prompt` | `enabled`, shell-command Requirement 2); the Command
Configurator introduces no separate external-execution security switch.

### Source References

- **[CR-NR-052]** = Change log entry for the Command Configurator requirement.
- **[CR-NR-051]** = Unified Command Target (command-framework Requirement 8).
- **[WB]** = Workbench Architecture Brief (command-driven, data-as-config).

### Cross-References

- **`command-framework`** Requirement 8 -- defines the Command_Target this
  sub-project stores and dispatches.
- **`shell-command`** Requirement 2 (shell.mode gate), Requirement 15
  (Output_Panel), Requirement 19 (External program execution) -- the external
  execution engine reused for the External target.
- **`menu-workspace`** Requirement 10 -- menu options may reference a
  user-defined command by its id.
- **`startup-and-session`** Requirement 21 -- the Command Configurator Context
  persists as a Custom_Workspace_Target; Started_Tasks do not persist.

---

## Glossary

| Term | Definition |
|------|-----------|
| **Command_Definition** | A named, user-authored entry pairing a Command_Id with a Command_Target and display metadata, stored in the Command_Store. |
| **Command_Store** | The TOML file `<User_Data_Dir>/commands/commands.toml` holding all Command_Definitions. |
| **Command_Configurator** | The Custom Workspace (Context) that lists, adds, edits, and deletes Command_Definitions. |
| **Started_Task** | A Detached external process spawned fire-and-forget: no capture, no Workspace, never persisted or restarted. |
| **Captured_Run** | A Captured external process run asynchronously with stdout/stderr/exit-code shown in the Output_Panel. |
| **Execution_Mode** | Detached or Captured -- an attribute of an External Command_Target. |

---

## Requirements

### Requirement 1: Command Store File Format

**User Story:** As a user, I want my custom commands saved in a readable file so
that I can back them up, share them, or edit them outside the app.

**Source:** [CR-NR-052], [WB]

#### Acceptance Criteria

1. THE Command_Store SHALL be a TOML file at `<User_Data_Dir>/commands/commands.toml` containing an array of Command_Definition tables under the key `[[command]]`.
2. EACH `[[command]]` entry SHALL contain: `id` (string, required, unique within the file, matching the Command_ID naming rule -- lowercase ASCII letters, digits, dots, underscores), `label` (string, required, human-readable), and a `[command.target]` table (required) that is a serialised Command_Target (command-framework Requirement 8.7).
3. EACH `[[command]]` entry MAY contain: `description` (string, optional) and `category` (string, optional, default `"user"`).
4. WHEN two entries share the same `id`, THE Command_Store loader SHALL keep the first, skip the duplicate, and log a WARN-level record naming the duplicate id.
5. WHEN the Command_Store file is absent, THE workbench SHALL treat the store as empty and SHALL create the `commands/` directory and an empty `commands.toml` on first save.
6. WHEN the Command_Store file contains invalid TOML or an entry is missing a required field, THE loader SHALL skip the invalid entry, retain all valid entries, and surface a load-error message identifying the offending entry.
7. WHEN a Command_Store file is written or modified on disk while the workbench is running, THE loaded definitions SHALL reload using the same hot-reload infrastructure as the Menu Workspace pattern (menu-workspace Requirement 4.3, 4.4).
8. THE Command_Store SHALL use only plain ASCII in its default/example content (documentation.md character rules).

---

### Requirement 2: Command Configurator Workspace

**User Story:** As a user, I want a screen inside FFWB where I can see all my
custom commands and add, edit, or remove them, so that I can extend the
workbench without hand-editing files.

**Source:** [CR-NR-052], [WB]

#### Acceptance Criteria

1. THE Command_Configurator SHALL be a Custom Workspace (Context) openable via a primary command (`COMMANDS`) and bindable as a Command_Target, displaying the list of Command_Definitions from the Command_Store.
2. THE Command_Configurator SHALL display, for each Command_Definition, its `id`, `label`, target variant (menu / custom-workspace / function / macro / external), and -- for External targets -- the Execution_Mode (Detached or Captured).
3. THE Command_Configurator SHALL provide an Add action that creates a new Command_Definition, a per-row Edit action that modifies an existing definition, and a per-row Delete action that removes it (with confirmation).
4. WHEN the user saves an Add or Edit, THE Command_Configurator SHALL validate the definition (Requirement 1.2 rules and Requirement 4 for External targets) and, on success, write the full Command_Store back to `commands.toml`; on validation failure it SHALL display the error and leave the file unchanged.
5. WHEN the user deletes a Command_Definition, THE Command_Configurator SHALL remove it from the Command_Store and write the file back.
6. THE Command_Configurator SHALL present the target editor as variant-specific fields: Menu (menu name), Custom Workspace (workspace kind + optional params), Function (command id + optional params), Macro (name or path), External (program, args, working directory, Execution_Mode).
7. WHEN the tab title is shown, THE Command_Configurator Context SHALL display `[COMMANDS]`.
8. WHEN F3 or `END` is entered in the Command_Configurator command field, THE shell SHALL return the Workspace to the Home Context (POM), consistent with other Custom Workspaces.

---

### Requirement 3: External Command Execution Modes

**User Story:** As a user, I want to run external scripts and programs from FFWB,
choosing whether to just launch them and forget, or to wait and see their
output, so that I can both kick off background tasks and run tools whose results
I need to read.

**Source:** [CR-NR-052], [WB]

#### Acceptance Criteria

1. AN External Command_Target SHALL carry a `program` (string, required), an `args` list (array of strings, optional), a `working_dir` (string, optional), and a `mode` that is exactly one of `detached` or `captured`.
2. WHEN an External Command_Target with `mode = detached` is executed AND `shell.mode` permits execution, THE workbench SHALL spawn the process as a Started_Task and return immediately without waiting, without capturing output, and without opening a Workspace.
3. WHEN a Started_Task is spawned, THE workbench SHALL NOT track, monitor, restart, or persist it; the process's lifecycle after spawn is owned by the operating system.
4. WHEN an External Command_Target with `mode = captured` is executed AND `shell.mode` permits execution, THE workbench SHALL run the process asynchronously (never blocking the GUI render thread), capture stdout and stderr, and display them together with the exit code in the shell Output_Panel (shell-command Requirement 15), reusing the same execution path as `shell.execute` (shell-command Requirement 19).
5. WHEN a `working_dir` is specified, THE workbench SHALL set the child process's working directory to that path; WHEN it is absent, THE workbench SHALL apply the `shell.working_directory` resolution rules (shell-command Requirement 11).
6. THE workbench SHALL expand `${workspace_root}` and `${file_dir}` placeholders in `program`, `args`, and `working_dir` to the active workspace root and active file directory respectively; an unresolved placeholder SHALL expand to an empty string and log a DEBUG-level record.
7. WHEN `shell.mode` is `disabled`, THE workbench SHALL refuse to execute any External Command_Target (Detached or Captured) and SHALL display the standard shell-disabled message (shell-command Requirement 2.2).
8. WHEN `shell.mode` is `prompt`, THE workbench SHALL display the shell confirmation dialog before executing an External Command_Target; IF the user declines, THEN the process SHALL NOT be spawned.
9. WHEN a Detached spawn fails to launch (program not found, permission denied), THE workbench SHALL display an error message identifying the program and SHALL NOT open a Workspace.
10. THE classification of an External Command_Target as a Visible_Workspace SHALL follow command-framework Requirement 8.9: Captured produces a Visible_Workspace (the Output_Panel result), Detached does not.

---

### Requirement 4: Definition Validation and Binding

**User Story:** As a user, I want the configurator to catch mistakes and let me
reuse my commands in menus and on keys, so that my custom commands are reliable
and reachable.

**Source:** [CR-NR-052], [CR-NR-051]

#### Acceptance Criteria

1. WHEN a Command_Definition is validated, THE workbench SHALL reject an empty `id`, an `id` that violates the Command_ID naming rule, an empty `label`, or a `[command.target]` that does not deserialise to a valid Command_Target, with a specific error message per failure.
2. WHEN an External target is validated, THE workbench SHALL reject an empty `program` and reject a `mode` that is not `detached` or `captured`.
3. A Command_Definition's `id` SHALL be usable as a Menu_Option `command` value (menu-workspace Requirement 10) so that selecting the option executes the definition's Command_Target.
4. A Command_Definition's `id` SHALL be bindable to a keyboard Shortcut_Binding (command-framework Requirement 8.5) so that a key or chord executes the definition's Command_Target.
5. WHEN a Menu_Option or Shortcut references a Command_Definition id that does not exist in the Command_Store, THE workbench SHALL display `Command '<id>' is not defined.` and take no further action.
6. THE workbench SHALL NOT allow a user-defined `id` to shadow a reserved built-in Command_ID; WHEN a definition uses a reserved id, THE workbench SHALL reject it with a message naming the conflict.

---

### Requirement 5: Alias Editor Context -- create, edit, and save command aliases (CR-NR-113; ISPF redesign CR-CH-068)

**REVISION NOTE (CR-CH-068, Gate 2).** Criteria 5.1-5.12 were authored under
CR-NR-113 for a FLAT free-text editor (surface / root / environment / template
text boxes plus a footer "Add alias" button). CR-CH-068 REDESIGNS the editor's
UI/interaction model into an ISPF-style editor (a `Command ===>` primary command
line, a CE/Root FILTER row, ISPF I/R/D line commands in a leading Act column,
cascading CE->Root dropdowns consuming CR-CH-067, and a scrollbar). The following
settled decisions REMAIN unchanged and are NOT contradicted: the on-disk
`UserAliasRowToml` storage SHAPE (5.1), the single command-dispatch front door
and `ALIASES` verb registration (5.2), the save-time validations (5.4), the
in-place live refresh (5.6), the resolver load/precedence model (5.7), the
expansion/dispatch delegation to CR-NR-112 (5.8), the per-tab Navigation_Stack
return (5.9), the `WorkspaceContext` single focus-latch contract (5.10), the v1
user-store-only scope (5.11), and the ASCII-keys/UTF-8-values posture (5.12).
Where a criterion's UI mechanics change, the change is noted INLINE below (5.3 is
superseded by the line-command model of 5.13-5.16; the footer Add button is
superseded by the `I` line command). New criteria 5.13-5.24 cover the redesign.

**User Story:** As a user, I want a screen inside FFWB where I can create, edit,
and save command aliases -- including parameterised aliases like `ECOB` ->
`EDIT FFWB.COBOL.MASTER(&1)` -- laid out like a mainframe/ISPF editor (a command
line, a filter row, and I/R/D line commands) with the command environment and
root command chosen from dropdowns, so that I can define my own shorthands
without hand-editing TOML files and without mistyping an invalid root.

**Source:** [CR-NR-113] (owner: "We need a workspace for creating and saving
aliases", follow-up to CR-NR-112). Builds ON the 4-field alias record and the
single-pass expansion of CR-NR-112 (command-semantics Requirement 14,
command-framework Requirement 19), the `WorkspaceContext` focus framework
(workspace-framework Requirement 1), and the Menus Editor Context pattern
(menu-workspace Requirement 13). Cross-references `docs/specs/localization/`
Requirement 11/12 (the per-locale alias catalogues, which this requirement does
NOT edit).

**BUILD-ORDER PREREQUISITE (hard dependency on CR-NR-112):** criteria 5.1 (the
`command_template` field), 5.3 (editing the template), and 5.8 (template
expansion and the end-to-end `ecob helloW` scenario) depend on CR-NR-112 having
landed `AliasEntry.command_template`, the `AliasRowToml.command_template` field,
and the `&1..&N` / `&*` substitution in the pre-pass. Those parts of this
requirement MUST NOT be implemented until CR-NR-112 is live in source. The
non-template subset (surface + canonical authoring, save, load, refresh, focus,
scope) has no such dependency and may proceed independently. See design
Section 9 for the sequencing note.

**NO-SHADOW RULE PROVENANCE:** the rule that a user alias must not equal a
reserved built-in root command (criteria 5.4, 5.7) is a NEW CR-NR-113 invariant
of the alias PRE-PASS, NOT inherited from command-framework Requirement 8.10.
Req 8.10 is a `resolve_target` stage-order rule (built-in command beats menu
beats macro) that does not cover aliases; aliases are rewritten in the
`rewrite_verb_to_root` / `canonical_verb` pre-pass that runs BEFORE
`resolve_target`, so without this new invariant a user alias could rewrite a
built-in verb away before 8.10 is ever consulted. "Reserved built-in root
command" is defined as the set returned by `builtin_ffcmd_roots()` (which
includes `command_config::BUILTIN_WORKSPACE_VERB_ROOTS`). This invariant is
analogous in INTENT to -- but mechanically distinct from -- Req 8.10.

#### Glossary additions

- **User_Alias_Store**: a user-scoped, locale-INDEPENDENT alias file, SEPARATE
  from the per-locale `i18n/<locale>/aliases/` localisation catalogues, holding
  the user's own alias definitions. Located under the workbench user data
  directory (the same base as the macros directory). It is the store the Alias
  Editor reads and writes.
- **Alias_Definition**: one user alias row, carrying the FOUR CR-NR-112 parts
  (command-semantics Requirement 14.1) plus a target-environment selector: a ROOT
  command (canonical verb), an ALIAS name (the typeable surface, e.g. `ECOB`), an
  optional LOCALISATION KEY (left blank for a language-independent user
  shorthand), an optional PARAMETER TEMPLATE (e.g. `FFWB.COBOL.MASTER(&1)`), and
  an optional target ENVIRONMENT (`ffcmd` or `ffedit`, defaulting to `ffcmd`)
  selecting which Command Environment's AliasTable the row loads into.
- **Alias_Editor**: the Custom Workspace (Context) that lists, adds, edits,
  deletes, and saves Alias_Definitions in the User_Alias_Store.

#### Acceptance Criteria

1. THE User_Alias_Store SHALL be a TOML file under the workbench user data directory (locale-independent, SEPARATE from the shipped per-locale `i18n/<locale>/aliases/` catalogues), containing `[[alias]]` rows whose fields are `surface` (the alias name, required), `canonical` (the root command, required), optional `environment` (the target Command Environment the row loads into -- `ffcmd` or `ffedit`; WHEN absent it SHALL default to `ffcmd`), optional `localisation_key`, and optional `command_template` (the CR-NR-112 field; see the build-order prerequisite above). WHEN the file is absent, THE workbench SHALL treat the user alias store as empty and SHALL create it on first save.

2. THE Alias_Editor SHALL be a Custom Workspace (Context) openable via a primary command (`ALIASES`) and bindable as a Command_Target, displaying the list of Alias_Definitions from the User_Alias_Store. The Context SHALL be registered the framework-conformant way (a `WorkspaceKind::CommandConfigurator` descriptor carrying `editor = "aliases"`, mirroring the Theme / Menus / Keys / Kinds editors); NO new WorkspaceKind and NO new dispatcher SHALL be introduced.

3. THE Alias_Editor SHALL display, for each Alias_Definition, its alias name (surface), its root command (canonical), its target environment (`ffcmd` or `ffedit`), and its parameter template (when present), and SHALL provide a way to ADD a row, DELETE a row (with confirmation), and per-field edit of the surface, the root command, the target environment, and the parameter template. (REVISED by CR-CH-068: the ADD/DELETE actions are now the ISPF `I` / `D` line commands of criteria 5.13-5.16 rather than a footer "Add alias" button + a per-row Delete button; the per-field edit of CE and Root is now via dropdowns per 5.18. The OBSERVABLE capability -- add, delete, per-field edit -- is unchanged; only the interaction mechanism changes.)

4. WHEN the user saves, THE Alias_Editor SHALL VALIDATE the working definitions -- rejecting (a) an empty surface, (b) an empty root command, (c) a surface that maps to two different roots within the SAME target environment in the store (the overlay's existing within-environment collision discipline), and (d) a surface that equals a reserved built-in root command (the set returned by `builtin_ffcmd_roots()`, a NEW CR-NR-113 pre-pass invariant -- see NO-SHADOW RULE PROVENANCE above, NOT command-framework Req 8.10) -- and, on success, serialise the rows and write the full User_Alias_Store back to disk; on validation failure it SHALL display the specific error and leave the file on disk UNCHANGED. The within-environment collision check (c) is scoped to collisions WITHIN the User_Alias_Store itself; a user surface that equals a per-locale overlay surface is governed by the load-time rule in Requirement 5.7 (SAME root = no-op; DIFFERENT root = the loader rejects the user row with a WARN and retains the per-locale surface), not by this save-time check.

5. WHEN the user deletes an Alias_Definition, THE Alias_Editor SHALL remove it from the working copy and, on save, write the file back without that row.

6. WHEN an Alias_Definition is saved, THE workbench SHALL refresh the live alias resolver (`LocaleAliasStore`) IN PLACE through the existing `ACTIVE_ALIAS_STORE` reload seam, so that the saved alias resolves and expands immediately WITHOUT an application restart. Because the existing reload path (`reload_active_alias_store_for_locale` -> `seed_and_overlay_for_locale`) today re-applies ONLY the Identity_Base and the per-locale `i18n/<locale>/aliases/` overlay and reads no user store, THE reload/re-seed path SHALL be EXTENDED to ALSO apply the User_Alias_Store overlay as a third layer; after a save-then-reload the live store SHALL carry BOTH the per-locale overlay AND the newly saved user alias (the re-seed SHALL NOT drop either layer).

7. THE resolver SHALL load the User_Alias_Store as a THIRD overlay layered on top of the Identity_Base and the per-locale overlay, routing each row into the AliasTable of its target environment (Requirement 5.1: `ffcmd` or `ffedit`, defaulting to `ffcmd`), such that a user alias resolves through the EXISTING single pre-pass (`rewrite_verb_to_root` -> `resolve_target`, command-framework Requirement 19) with NO new dispatch path. A user alias SHALL NOT shadow a reserved built-in root command (the `builtin_ffcmd_roots()` set; a NEW CR-NR-113 pre-pass invariant, not command-framework Req 8.10): Requirement 5.4 rejects it at save time, and the loader SHALL skip such a row with a WARN if it reaches the file by other means. WHEN a user row's surface equals a per-locale overlay surface already mapped to the SAME root within the same environment, THE loader SHALL treat it as a no-op (the mappings agree); WHEN it maps to a DIFFERENT root, THE loader SHALL reject the user row with a WARN and retain the per-locale surface (the per-locale overlay is authoritative on a cross-layer collision). A user alias and a per-locale localisation surface are different axes and coexist whenever their surfaces differ.

8. WHEN a parameterised Alias_Definition authored in the Alias_Editor is invoked -- e.g. `ECOB` with root `EDIT` and template `FFWB.COBOL.MASTER(&1)`, typed as `ecob helloW` -- it SHALL expand to `EDIT FFWB.COBOL.MASTER(helloW)` and dispatch exactly as CR-NR-112 specifies (command-semantics Requirement 14.9, command-framework Requirement 19.1); the Alias_Editor SHALL author the definition but SHALL NOT introduce any new expansion or dispatch behaviour.

9. WHEN the tab title is shown, THE Alias_Editor Context SHALL display `[ALIASES]`; WHEN F3 or `END` is entered in its command field, THE shell SHALL return the Workspace to the Context it was opened from via the per-tab Navigation_Stack (consistent with the Menus Editor, menu-workspace Requirement 13).

10. THE Alias_Editor SHALL report its interior focus through the single `WorkspaceContext::render -> InteriorFocus` path (workspace-framework Requirement 1): the FIRST interactive control SHALL have a stable `egui::Id`, and the first Tab from the command field SHALL land EXACTLY on that control (no phantom stop), proven by a full-shell egui_kittest first-Tab test.

11. v1 SCOPE: THE Alias_Editor SHALL author the User_Alias_Store ONLY. Editing the per-locale localisation alias catalogues (`i18n/<locale>/aliases/`, CR-NR-109/111) SHALL be OUT OF SCOPE for this requirement, so the Alias_Editor SHALL NOT read from or write to the shipped per-locale catalogues (avoiding overlap with the in-flight CR-NR-111 gate).

12. THE User_Alias_Store SHALL use only plain ASCII in its default/example content for the structural keys; alias field VALUES are user data and MAY be any valid UTF-8, consistent with the alias DATA posture (localization alias catalogues are localisation DATA).

#### Acceptance Criteria -- ISPF redesign (CR-CH-068, Gate 2)

The following criteria REDESIGN the Alias_Editor's UI/interaction model. They
consume the CR-CH-067 `EnvironmentRegistry::root_commands(env_name)` /
`contains_root(env_name, root)` associated functions (command-environments
Requirement 20), follow the Menus Editor Context pattern (menu-workspace
Requirement 13) for the command line + working-copy + validate-on-save flow, and
preserve the `WorkspaceContext` focus contract (workspace-conformance.md). They do
NOT change the on-disk `UserAliasRowToml` shape (5.1), the dispatch front door
(5.2), the WorkspaceKind, or the v1 user-store-only scope (5.11).

13. COLUMN LAYOUT. WHEN the Alias_Editor is rendered, THE Context SHALL present the alias list with columns in this left-to-right order: Act, CE (Command Environment), Root (root command), L18N (locale key), ALIAS (surface), Parameters (parameter template). THE column-to-model mapping SHALL be: Act = a transient per-row line-command field (NOT persisted); CE = `UserAliasRowToml.environment`; Root = `canonical`; L18N = `localisation_key` (blank for a language-independent user shorthand); ALIAS = `surface`; Parameters = `command_template`. THE L18N column SHALL be an editable field that PERSISTS: WHEN a non-empty L18N value is entered and the working copy is saved, THE saved `UserAliasRowToml.localisation_key` SHALL carry that value (trimmed), and re-loading the store SHALL render the same L18N value (a round-trip). This does NOT change the on-disk `UserAliasRowToml` shape (criterion 5.24): the `localisation_key` field already exists in the on-disk row; the editor merely stops discarding it. (The current save path `validate_user_alias_rows` hardcodes `localisation_key: String::new()`; criterion 5.13 requires that to carry the edited row value -- see design 9.10.2 and task 8.9.)

14. COMMAND LINE. WHEN the Alias_Editor is rendered, THE Context SHALL present a `Command ===>` primary command field at the top (ISPF primary command line). THE field SHALL accept at least the primary commands `SAVE` (validate + write the working copy, criterion 5.4) and `CANCEL` / `END` (leave the editor via the per-tab Navigation_Stack, criterion 5.9). WHEN the user enters `SAVE` on the command line, THE Context SHALL first apply any pending line commands (criterion 5.16) and then perform the Save of criterion 5.4. THE command line SHALL ALSO accept `FILTER <ce> <root>` and `RESET` to set/clear the filter row of criterion 5.15. A locale switch is OUT of v1 scope. (CONFIRMED by owner, CR-CH-068 approval: v1 primary commands = SAVE, CANCEL/END, FILTER, RESET.)

15. FILTER ROW. WHEN the Alias_Editor is rendered, THE Context SHALL present a FILTER ROW immediately below the column headings, into which the user MAY enter a CE and/or a Root. WHEN both filter fields are empty, THE Context SHALL list ALL aliases from the User_Alias_Store. WHEN a CE and/or Root filter is set, THE Context SHALL list ONLY the alias rows whose environment matches the CE filter (when set) AND whose root matches the Root filter (when set). THE filter SHALL match on CE and Root ONLY (NOT on ALIAS or Parameters). WHEN a CE is chosen in the FILTER row, THE Root filter's dropdown SHALL be constrained to that environment's root commands (`root_commands(selected_CE)`), consistent with criterion 5.18. THE filter SHALL be applied on the `FILTER` command / Enter (ISPF fidelity), NOT live-as-you-type. (CONFIRMED by owner, CR-CH-068 approval: applied-on-command, filter scope CE + Root only.)

16. LINE COMMANDS (I / R / D). WHEN the user types a line command letter into a data row's Act field, THE Context SHALL recognise `I` (insert a new blank editable row), `R` (repeat -- duplicate the current row into a new editable row), and `D` (delete the row). THE pending line commands SHALL be APPLIED as a batch WHEN the user presses Enter on the `Command ===>` line (or activates an equivalent Apply/refresh action), consistent with ISPF semantics, after which each applied Act field SHALL be cleared. WHEN an `I` is applied, THE inserted row SHALL be blank (empty surface / root / template, default environment). WHEN an `R` is applied, THE new row SHALL be a copy of the source row's field values. WHEN multiple line commands are pending in one apply pass, THE Context SHALL apply them deterministically (RECOMMENDED: process by row position so that inserts/repeats/deletes do not corrupt one another's indices). (CR-CH-068 SUPERSEDES the CR-NR-113 footer "Add alias" button with the `I` line command and the per-row Delete button with the `D` line command.)

17. EMPTY-STORE USABILITY. WHEN the User_Alias_Store is absent or empty (no rows), THE Alias_Editor SHALL still render the column headings and the FILTER row, AND SHALL present at least one empty insertable row (or an obvious, visible affordance to insert the first row, e.g. an `I` typed into an always-present Act field), so that the user can create the FIRST alias without hand-editing a file. (This is the explicit fix, by redesign, for the Add-visibility bug recorded in `.agents/tasks/alias-editor-workspace/add-row-bug-findings.md`.)

18. CASCADING CE -> ROOT DROPDOWNS. WHEN a data row (or the filter row) renders its CE and Root fields, THE CE field SHALL be a dropdown listing the registered command environments, AND THE Root field SHALL be a dropdown populated by `EnvironmentRegistry::root_commands(selected_CE)` (CR-CH-067, command-environments Requirement 20.1). WHEN the user changes a row's CE selection, THE Context SHALL re-populate that row's Root dropdown from the new environment's `root_commands` AND SHALL clear or re-validate a Root that is no longer valid in the new environment. THE dropdowns SHALL enforce the validated (environment, root) anchor by CONSTRUCTION (a user can only pick a Root that `contains_root(selected_CE, root)` would accept). (v1 CE MEMBERSHIP, CONFIRMED by owner CR-CH-068 approval: for v1 the CE dropdown lists the two user-addressable alias environments `AliasEnvironment::ALL` = `ffcmd` / `ffedit`, the only environments the alias model can route a row into today; enumerating the FULL EnvironmentRegistry membership is a deferred, purely additive follow-up and NOT part of this redesign.)

19. SAVE VALIDATION REUSE (CR-CH-067). WHEN the user saves, THE Alias_Editor SHALL validate each row's (environment, root) pair using `EnvironmentRegistry::contains_root(env_name, root)` (CR-CH-067, command-environments Requirement 20.8) via the existing `validate_user_alias_rows` save-side hook, IN ADDITION to the CR-NR-113 validations of criterion 5.4 (empty surface, empty root, within-same-environment surface->two-roots collision, no-shadow of a reserved built-in root). WHEN any validation fails, THE Context SHALL display the specific error inline on the `Command ===>`/error surface AND SHALL leave the on-disk file UNCHANGED (criterion 5.4). The load path already skips an invalid user row with a WARN (CR-CH-067, command-environments Requirement 20.7).

20. SCROLLBAR. WHEN the number of listed (filtered) data rows exceeds the visible row area, THE Alias_Editor SHALL present a vertical scrollbar on the right so the user can scroll through the list, while the `Command ===>` line, the column headings, and the FILTER row remain usable.

21. LINE-COMMAND FOCUS RECONCILIATION. THE Alias_Editor SHALL reconcile the ISPF Act-field line-command model with egui's immediate-mode widget model and the app's Tab/focus contract such that: (a) each row's Act field is an ordinary focusable control; (b) typing a line-command letter records a PENDING code without immediately restructuring the row list mid-frame (the batch is applied only on the criterion 5.16 Enter/apply); and (c) the Tab order flows `Command ===>` line -> FILTER row (filter CE dropdown -> filter Root dropdown) -> each data row (Act -> CE -> Root -> L18N -> ALIAS -> Parameters) with NO phantom stop (consistent with criterion 5.10 and criterion 5.23). The FILTER row carries ONLY the filter CE and filter Root fields (no Act / L18N / ALIAS / Parameters); the Act / L18N / ALIAS / Parameters field set belongs to each DATA row.

22. PRIMARY-COMMAND / LINE-COMMAND PRECEDENCE. WHEN the user presses Enter on the `Command ===>` line with BOTH a primary command typed AND pending Act line commands, THE Context SHALL apply the pending line commands FIRST and THEN interpret the primary command (so e.g. `SAVE` saves the post-line-command working copy), consistent with ISPF primary-vs-line command ordering.

23. FIRST-TAB FOCUS (MANDATORY). THE Alias_Editor SHALL report its interior focus through the single `WorkspaceContext::render -> InteriorFocus` path, with a STABLE `egui::Id` on the FIRST interior control, AND the FIRST Tab from the `Command ===>` field SHALL land EXACTLY on that first interior control, with NO phantom stop. THE first interior control SHALL be the FILTER row's filter CE dropdown (its `response.id` captured fresh each frame, per workspace-conformance.md), matching design 9.10.8 part 3. (This names exactly ONE deterministic target so the mandatory first-Tab test asserts a single `egui::Id`; the filter Root field is NOT the first interior control.) THIS SHALL be proven by a full-shell egui_kittest first-Tab test (workspace-conformance.md; a specialisation of criterion 5.10 for the redesigned layout).

24. MODEL + FRAMEWORK UNCHANGED. THE CR-CH-068 redesign SHALL NOT change the on-disk `UserAliasRowToml` shape (criterion 5.1), the single command-dispatch front door and `ALIASES` verb registration (criterion 5.2), the `WorkspaceKind::CommandConfigurator` + `editor = "aliases"` descriptor, the per-tab Navigation_Stack return behaviour (criterion 5.9), the `pending_action` / owned-panel-swap effect pattern, or the v1 user-store-only scope (criterion 5.11). IT SHALL add NO second dispatcher and NO new session-persistence format.
