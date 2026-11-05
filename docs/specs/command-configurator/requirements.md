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

9. WHEN the tab title is shown, THE Alias_Editor Context SHALL display `[ALIASES]`; WHEN F3 or `END` is entered in its command field, THE shell SHALL return the Workspace to the Context it was opened from via the per-tab Navigation_Stack (consistent with the Menus Editor, menu-workspace Requirement 13). **(AMENDED by CR-CH-071.)** The tab-title clause is UNCHANGED. The END / F3 RETURN BEHAVIOUR is amended to the ISPF model: END (and F3) SHALL first SAVE the Alias Editor's working copy and THEN return via the per-tab Navigation_Stack (ISPF END commits); the plain navigation-only END described above is superseded. The discard-on-leave verb is CANCEL / RETURN (command-environments Req 20a.3), not END. See command-environments Requirement 20a for the full editable-Context verb set (SAVE / END / CANCEL / RETURN / REFRESH) and its FFCMD-base ownership with the FFEDIT active-wins override. This amendment does NOT change the `WorkspaceContext` focus contract (5.10 / 5.23), the on-disk shape (5.1), or the dispatch front door (5.2).

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

14. COMMAND LINE. WHEN the Alias_Editor is rendered, THE Context SHALL present a `Command ===>` primary command field at the top (ISPF primary command line). THE field SHALL accept at least the primary commands `SAVE` (validate + write the working copy, criterion 5.4) and `CANCEL` / `END` (leave the editor via the per-tab Navigation_Stack, criterion 5.9). WHEN the user enters `SAVE` on the command line, THE Context SHALL first apply any pending line commands (criterion 5.16) and then perform the Save of criterion 5.4. THE command line SHALL ALSO accept `FILTER <ce> <root>` and `RESET` to set/clear the filter row of criterion 5.15. A locale switch is OUT of v1 scope. (CONFIRMED by owner, CR-CH-068 approval: v1 primary commands = SAVE, CANCEL/END, FILTER, RESET.) **(AMENDED by CR-CH-071.)** The command-line still accepts SAVE, CANCEL / END, FILTER, and RESET and still applies pending line commands first (criterion 5.22). TWO things are amended: (a) the SEMANTICS of SAVE / END / CANCEL follow the ISPF model of command-environments Requirement 20a -- SAVE persists and STAYS, END (and F3) SAVES then leaves, CANCEL / RETURN DISCARDS then leaves -- and the new `REFRESH` verb (discard working changes, reload the last-saved store, STAY) is ALSO accepted; and (b) the Alias Editor's PRIVATE interpretation of SAVE / CANCEL / END (the `apply_alias_command_line` arms in `shell/alias_editor_line_commands.rs`) is RETIRED: SAVE / CANCEL / REFRESH are resolved through the ONE command-dispatch path as FFCMD-base verbs handled per-focused-editable-Context (command-environments Req 20a.5-20a.6), so F3 / a globally-typed END and the typed `Command ===>` line reach the SAME handler (today they diverge -- the private interpreter sees the typed line but F3 does not). FILTER and RESET remain Alias-Editor-local `Command ===>` verbs (they act on the filter row, not a cross-Context concern). This amendment does NOT change the on-disk shape (5.1), the dispatch front door / `ALIASES` registration (5.2), or the focus contract (5.23).

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
    - **SUPERSEDED by CR-CH-072 (Requirement 6 below)**: the v1 user-store-only
      scope (criterion 5.11) is replaced -- the Alias_Editor now edits the
      per-locale LOCALISATION aliases (the active-locale effective table = shipped
      seed overlaid by the user per-locale override store). The on-disk ROW shape
      changes with the simplified model: the `localisation_key` command-surface
      field is REMOVED (the localised surface is stored directly as `surface`);
      the row becomes `surface` + `canonical` + optional `command_template` +
      optional `environment`. Criteria 5.13's L18N column is retired for command
      surfaces (there is no command-surface localisation key any more). The
      interaction model (ISPF command line, line commands, filter row, cascading
      dropdowns, focus contract) and the ISPF save semantics (CR-CH-071) are
      PRESERVED; only the STORE the editor reads/writes and the ROW shape change.

---

### Requirement 6: The Alias Editor edits the per-locale localisation aliases, active-locale effective view (CR-CH-072)

**User Story:** As a foreign-language user, I want the Alias Editor to show me the
aliases that localise MY menus for MY active locale, seeded with the shipped
defaults and fully editable, so I can customise any command surface into my own
language and see the menus reflect it.

This requirement SUPERSEDES the v1 user-store-only scope of Requirement 5.11
(CR-NR-113). It is the SINGLE in-app place to view and customise the localised
menu-command aliases. It builds ON the Alias Editor Context (Requirement 5, incl.
the CR-CH-068 ISPF redesign and the CR-CH-071 save semantics) and the simplified
one-layer alias model (localization Requirement 13); it adds NO new Context, NO
new dispatcher, and NO new WorkspaceKind.

#### Acceptance Criteria

6.1. THE Alias_Editor SHALL display the ACTIVE LOCALE's EFFECTIVE alias table for
   each environment: the shipped read-only seed (`i18n/<locale>/aliases/<ENV>.toml`)
   OVERLAID by the User_Locale_Override_Store for the active locale (localization
   Requirement 13.5). WHEN the active locale is `af`, THE editor SHALL show the
   shipped `af` surfaces (INSTELLINGS / KATALOGUSSE / LEERS / ... ) -- the explicit
   fix for the owner-reported "I don't see any Afrikaans aliases" -- not an empty
   or English-only list.

6.2. EACH displayed row SHALL present the localised SURFACE (the In_Row_Surface),
   the ROOT (canonical) command, the target ENVIRONMENT, and the optional
   parameter TEMPLATE. THE editor SHALL NOT present a command-surface localisation
   key column (the `verb-*` indirection is removed, localization Requirement 13.1);
   the surface IS the authoritative localised text.

6.3. WHEN the user edits a surface (or adds / deletes a row) and SAVEs, THE
   Alias_Editor SHALL write the change to the User_Locale_Override_Store for the
   ACTIVE locale and environment (localization Requirement 13.6), NEVER to the
   shipped seed; the shipped seed SHALL remain unmodified on disk.

6.4. AFTER a save, THE effective active-locale table SHALL reflect the edit and the
   corresponding MENU option SHALL render the edited surface (localization
   Requirement 13.7), reusing the existing reload seam so the change goes live
   without restart. An English-base user MAY likewise customise surfaces into his
   own language against the active locale.

6.5. THE editor's SAVE / END / CANCEL / RETURN / REFRESH semantics (CR-CH-071,
   command-environments Requirement 20a) SHALL operate over the
   User_Locale_Override_Store: SAVE persists the override and STAYS; END / F3 saves
   then leaves; CANCEL / RETURN discards the working changes and leaves; REFRESH
   discards the working changes and reloads the effective table (seed + override).

6.6. THE validation discipline of Requirement 5.4 / 5.19 SHALL apply to the override
   rows (empty surface, empty root, within-environment surface->two-roots
   collision, no-shadow of a reserved built-in root, valid (environment, root) via
   `contains_root`); where a user override surface equals a shipped-seed surface
   the override WINS (localization Requirement 13.5), which is the user's intended
   customisation, not a collision.

6.7. THE parameter template (CR-NR-112) SHALL remain editable on each row and SHALL
   expand unchanged when the aliased command is invoked (localization
   Requirement 13.8); removing the localisation-key column SHALL NOT affect the
   template feature.

---

### Requirement 7: Alias Editor on the shared ISPF Table Editor -- Description column + Sort (CR-CH-075); block commands, single-chrome, and arrow navigation (CR-CH-076 / 077 / 078)

**User Story:** As an ISPF user, I want the Alias Editor to be a true ISPF table
editor built on the shared table-editor component, with a Description column and
a Sort line command, so it looks and behaves like the Menu and Kinds editors and
I can describe and order my aliases.

**Source:** [CR-CH-075] (Description column + Sort + shared-component move);
[CR-CH-076] (block commands, criterion 7.11); [CR-CH-077] (single-chrome,
criterion 7.12); [CR-CH-078] (arrow navigation, criterion 7.13) -- all three
owner-confirmed 2026-11-01, `.agents/tasks/alias-editor-concerns/concerns.md`.
Builds ON Requirement 5 (the CR-NR-113 / CR-CH-068 Alias Editor and its CR-CH-071
save semantics), Requirement 6 (CR-CH-072 store/row shape), and the shared ISPF
Table Editor component (`docs/specs/ispf-table-editor/`). It REPLACES the Alias
Editor's bespoke I/R/D mini line-command flow (`alias_editor_line_commands.rs`)
with the shared component's engine-backed line commands. It adds NO new dispatcher, NO new
navigation stack, NO new session-persistence format, NO new line-command engine,
and NO new save path.

**SCHEMA DECISION -- SETTLED (option (a), implemented).** The owner's mock
replaces the L18N column (retired by CR-CH-072, Requirement 6.2) with a
DESCRIPTION column. The owner CONFIRMED option (a): an additive optional
`#[serde(default, skip_serializing_if = "...")] description` field on the alias
row (user-store row and per-locale override row), editable and persisting, with
empty values omitted on serialise so existing files round-trip unchanged. Per
`.kiro/steering/framework-conformance.md` an additive optional field is NOT a
framework change (no core-type reshape). This is no longer an open decision: the
Description column SHIPPED in the Group B merge (commit 1532a5f, on main) and is
live. Criteria 7.2 / 7.3 are therefore SETTLED on option (a); option (b)
(display-only, no field) was NOT chosen and is not carried forward.

#### Acceptance Criteria

7.1. WHEN the Alias_Editor is rendered, THE Context SHALL draw its surface via the
   shared ISPF Table Editor component (`docs/specs/ispf-table-editor/`
   Requirement 1), presenting the `Command ===>` line, the filter row, the Act
   column, and the scrolling data rows through that one component rather than a
   bespoke per-editor surface. THE focus contract, command-line semantics, filter
   behaviour, and save-seam routing SHALL be those the shared component defines
   (ispf-table-editor Requirements 2, 3, 6, 7).

7.2. THE Alias_Editor column order SHALL be: Act, CE (Command Environment), Command
   (root / canonical), Alias (surface), Parameters (parameter template),
   Description. THE column-to-model mapping SHALL be: Act = the shared component's
   transient line-command cell (NOT persisted); CE = `environment`; Command =
   `canonical`; Alias = `surface`; Parameters = `command_template`; Description =
   the row's `description` field (SETTLED option (a), implemented -- see the
   SCHEMA DECISION above). The L18N column is NOT present (retired by CR-CH-072,
   Requirement 6.2).

7.3. (SETTLED option (a), implemented -- commit 1532a5f on main.) THE Description column SHALL be an
   editable field that PERSISTS: WHEN a non-empty description is entered and the
   working copy is saved, THE saved alias row SHALL carry that value (trimmed) in
   an optional `description` field, and re-loading the store SHALL render the same
   description (a round-trip). THE new field SHALL be optional and
   `#[serde(default)]` so an alias file written before this field existed loads
   unchanged (no on-disk-shape break), consistent with the stable-schema rule.

7.4. THE Alias_Editor's enabled Line_Command_Set SHALL be `I` (Insert), `D`
   (Delete), `C` (Copy), and SORT. Move (`M`) SHALL NOT be enabled, because an
   alias set is not an inherently ordered list (the resolver keys by surface, not
   position), so moving a row has no observable effect; SORT (by a chosen column)
   is the useful ordering operation for a long alias list. (RATIONALE recorded per
   the brief: move is nonsensical for aliases; sort is useful.)

7.5. WHEN the user enters `SORT <column> [ASC|DESC]` on the `Command ===>` line,
   THE Alias_Editor SHALL reorder its working-copy rows by that column (stable,
   ascending default) via the shared component's Sort (ispf-table-editor
   Requirement 5), and the reordering SHALL persist on Save. At least the Alias
   and Command (root) columns SHALL be declared sortable.

7.5a. AUTOMATIC SORT-ON-SAVE (owner-confirmed CR-CH-075). WHEN the Alias_Editor
   persists the working copy (any SAVE or END), THE system SHALL first reorder the
   rows into the canonical sort order and write the Command_Store in that SORTED
   order, so the on-disk alias file is always deterministically ordered
   regardless of the order rows were inserted/edited. THE canonical sort key SHALL
   be the RESOLVER'S MATCH KEY: the `Alias` (surface) string, compared
   case-INSENSITIVELY (uppercased, matching `AliasTable::canonical_verb`'s
   `eq_ignore_ascii_case` on the uppercased surface), scoped PER Command
   Environment. Because the store is already one file per environment
   (`<locale>/<ENV>.toml`), this means each environment's rows SHALL be sorted by
   uppercased surface ascending; `Command` (root) is a stable tie-breaker only if
   two identical surfaces ever coexist (validation normally forbids this). THE
   automatic sort-on-save SHALL use the SAME ordering the manual `SORT` command
   (7.5) produces, so manual and automatic sorting are consistent, stable for
   already-sorted input (a no-op reorder), and SHALL NOT alter any field value,
   only row order.
   THIS is an EDITOR-ONLY change (ordering rows the editor owns before
   serialising). It SHALL NOT change the alias RESOLVER: `AliasTable` remains a
   linearly-scanned `Vec` and `canonical_verb` is unchanged. Per the
   alias-resolver investigation the set stays small at runtime (only the ACTIVE
   locale's per-CE table is loaded and scanned, not all locales), so the O(n)
   scan is negligible and reshaping `AliasTable` to a map/binary-search is an
   unwarranted core/framework change that is NOT undertaken. Sorting by the
   resolver's match key keeps the on-disk order aligned with lookup and leaves a
   FUTURE binary-search switch cheap, without coupling to it now.

7.6. THE `I` / `D` / `C` line commands SHALL behave per the shared component
   (ispf-table-editor Requirement 4): `I` inserts a blank alias row after the
   row; `D` deletes the row; `C` copies the row's field values to a target
   designated by an `A` (after) / `B` (before) marker, using the shared
   component's `ff-line-commands`-backed pairing. THE previous CR-CH-068 `R`
   (repeat) MAY be retained as an alias of a single-row Copy, but the primary
   duplication path is `C`; this REPLACES the bespoke I/R/D flow of criterion 5.16.

7.7. THE filter row SHALL match on CE and Command (root) only, exactly as
   criterion 5.15 / Requirement 6, now realised through the shared component's
   filter (ispf-table-editor Requirement 3); the CE -> Root cascading dropdowns of
   criterion 5.18 SHALL be preserved as the per-row and filter-row widgets the
   host declares to the component.

7.8. THE Alias_Editor SHALL report its interior focus through the single
   `WorkspaceContext::render -> InteriorFocus` path with the first interior
   control being the filter row's filter CE dropdown (criterion 5.23 unchanged),
   proven by the existing MANDATORY full-shell egui_kittest first-Tab test
   (workspace-conformance.md). Moving to the shared component SHALL NOT weaken or
   remove this test.

7.9. THE Alias_Editor's displayed columns (7.2 order) SHALL be the authoritative
   column set its `ScreenProvider` populates into a `ff-screen-model::Table` for
   snapshot/capture (Group A), so a captured snapshot shows Act / CE / Command /
   Alias / Parameters / Description in that order
   (ispf-table-editor Requirement 7.3).
   (REVISED 2026-11-04 by the Alias per-Kind gate: the SHAPE of that captured
   table - frame rules, cell conventions, scratch row, single representation - is
   now specified by 7.14-7.28.)

7.10. THE CR-CH-075 redesign SHALL NOT change the dispatch front door / `ALIASES`
   registration (criterion 5.2), the `WorkspaceKind::CommandConfigurator` +
   `editor = "aliases"` descriptor, the per-tab Navigation_Stack return behaviour
   (criterion 5.9), the `pending_action` / owned-panel-swap effect pattern, or the
   CR-CH-072 store the editor reads/writes (Requirement 6). The ONLY persisted
   shape change is the OPTIONAL `description` field of 7.3, flagged above for owner
   confirmation; everything else is presentation + line-command behaviour moved
   onto the shared component.

7.11. BLOCK LINE COMMANDS (CR-CH-076). THE Alias_Editor SHALL opt IN to the shared
   component's block Line_Command_Set (ispf-table-editor Requirement 9): its
   enabled set becomes `I`, `D`, `C`, SORT (criterion 7.4) PLUS the block variants
   `DD` (delete block), `CC` (copy block), and -- for completeness of the uniform
   pattern -- `RR` (repeat block); `MM` (move block) remains DISABLED because Move
   is not enabled for an alias set (criterion 7.4, the resolver keys by surface,
   not position). A matched doubled pair marks an INCLUSIVE range and the
   operation applies to the whole block; an unterminated block yields a
   non-blocking "block not closed" notice and NO structural change
   (ispf-table-editor Requirement 9.1, 9.3, 9.4). RATIONALE (owner, 2026-11-01):
   block commands are rarely useful on an unordered alias list, but the Alias
   Editor opts in so the PATTERN is supported uniformly and is copyable to the
   Menu / Kinds / future ISPF editors. A `CC` block still requires an `A`/`B`
   destination exactly as a single `C` does (criterion 7.6, ispf-table-editor
   Requirement 9.3).

7.12. SINGLE-CHROME LAYOUT (CR-CH-077). THE Alias Context SHALL show exactly ONE
   Title line (the Context / Kind descriptive title) and ONE `Command ===>` line
   -- the SHELL's APP-LEVEL chrome (the shell Title_Line + the shell command
   field), per the owner-confirmed option-A (shell-level) design -- and the Alias
   Context SHALL NOT draw its OWN duplicate title / heading or `Command ===>` line
   on top of the shell's (ispf-table-editor Requirement 10.1, 10.2). WHEN the
   Alias Context is ACTIVE THE shell command field SHALL ROUTE the entered command
   line into the Alias command-environment handler: `run_command_line` applies any
   pending Act line commands, then interprets FILTER / RESET / SORT, then FALLS
   THROUGH to the single FFCMD front door for SAVE / END / RETURN / REFRESH
   (ispf-table-editor Requirement 10.2). The previously duplicated body heading
   ("FileForge Workbench Alias Editor") and the duplicated command line SHALL be
   REMOVED from the Alias Context render arm. THE Alias Context's `ScreenProvider`
   snapshot SHALL render the SAME single-chrome layout -- INCLUDING the single
   Title line, the `Command ===>` line, and the column header row (today the
   command line and header are missing from the Alias snapshot) -- in the 7.2
   column order, so the live Alias panel and its captured snapshot AGREE
   (ispf-table-editor Requirement 10.3). This changes NO dispatch, navigation,
   descriptor, effect pattern, or save path (ispf-table-editor Requirement 10.4).
   (NOTE 2026-11-01: this criterion was reconciled IN PLACE from its original
   wording -- which had the shared COMPONENT own the one `Command ===>` line -- to
   the owner-confirmed option-A shell-level design implemented under CR-CH-077.)
   (REVISED 2026-11-04: since CR-NR-115 the `Command ===>` line and the single
   Title are COMMON-FRAME output owned by the shell in the snapshot too; the
   Alias provider contributes neither (7.14), and the remainder of the snapshot
   layout is governed by 7.15-7.22.)

7.13. ARROW-KEY FOCUS NAVIGATION (CR-CH-078). THE Alias_Editor SHALL inherit the
   shared component's full arrow-key focus navigation (ispf-table-editor
   Requirement 11): Up/Down move focus to the same column in the adjacent row
   (clamped), Left/Right move focus to the adjacent column in the same row
   (clamped), owned by the shared component so the Alias Editor needs no bespoke
   arrow handling. This SHALL NOT weaken the mandatory first-Tab egui_kittest test
   (criterion 7.8 / 5.23): Tab / Shift+Tab and the stable `egui::Id`s are
   unchanged; the arrow walk is additive (ispf-table-editor Requirement 11.4). The
   focused `Command ===>` field keeps its CR-NR-096 Up/Down history gesture
   (ispf-table-editor Requirement 11.5).

#### Acceptance Criteria -- Alias Workspace Kind: Content_Area + Workspace_Status_Segment (per-Kind gate, CR-NR-116, building on CR-NR-115)

> NOTE ON TAGGING: the gate criteria below carry the `[CR-NR-115]` tag because
> they EXTEND the common-frame family (the convention also used by CR-NR-117).
> The owning change-log id for this gate is CR-NR-116; `[CR-NR-115]` marks the
> framework family these criteria build on, not their ownership.


**Source:** the FIRST per-Workspace-Kind gate under the common workspace frame
(`workspace-framework` Requirement 7.23-7.25; `screen-snapshot-scrm` Requirement
30). Owner issues of 2026-11-03/04 (`.agents/tasks/snapshot-fidelity-impl/
owner-review-vs-live.md` D2/D5, `.agents/tasks/alias-editor-concerns/
snapshot-fidelity-gap.md`, and the 2026-11-04 owner capture). The change-log
CR id is CR-NR-116 (this gate). This block defines ONLY what the
Alias Workspace owns: its CONTENT AREA and ONE STATUS SEGMENT, in BOTH the live
panel and the snapshot/capture. It REVISES the snapshot half of 7.9 and 7.12
(whose "today the command line and header are missing" wording described the
pre-CR-NR-115 snapshot; the command line and header are now frame/grid output).
Criteria marked `[OWNER CONFIRM]` carry a recommended default that the owner
accepts or flips at the gate CONFIRM step (decision points a-g, design 9.11.9).

7.14. OWNERSHIP BOUNDARY. THE Alias Workspace SHALL own ONLY (i) its Content_Area
   (the region between the shell `Command ===>` line and the Message_Line) and
   (ii) its Workspace_Status_Segment cells. It SHALL NOT draw, compose, or
   capture any Common_Frame element (window title bar, menu bar, tab bar,
   centered title, `Command ===>` line, Message_Line, `Last:`/`Prof:`/`Zoom:`
   status cells, key line) in either the live panel or the `ScreenProvider` model
   (workspace-framework Requirement 7.23; screen-snapshot-scrm Requirement
   30.15). `[CR-NR-115]`

7.15. SINGLE REPRESENTATION (owner issue 1 / D2). WHEN the Alias Context is
   snapshotted or captured, THE `ScreenProvider` SHALL present each alias row's
   data EXACTLY ONCE, as one row of the content grid. It SHALL NOT populate
   `ScreenModel.fields` (no `SS . . . : SNAPSHOT` dot-leader line), and NO
   clipboard flavor SHALL contain a `| Property | Value |` table built from alias
   rows (workspace-framework Requirement 7.24; screen-snapshot-scrm Requirement
   28.4 is unchanged for Contexts that legitimately use it). `[CR-NR-115]`

7.16. NO STRAY ROWS (owner issue 7). THE Alias Content_Area SHALL begin with the
   grid's own top rule MERGED with the frame-owned rule that the shell draws
   between the `Command ===>` line and the first Content_Area row
   (screen-snapshot-scrm Requirement 30.34 / 30.35, CR-NR-115 grid-merge) and
   SHALL contain NO blank row before the grid, NO `Aliases` caption/heading row,
   and NO blank row between the last grid row and the Content_Area's closing
   rule. An inline panel error (`AliasEditorState.error`) SHALL be carried in
   `model.messages` (the shell Message_Line band, screen-snapshot-scrm
   Requirement 30.25), not as a Content_Area row. `[CR-NR-115]`

7.17. GRID IN THE FRAME'S RULES (owner issue 2). THE Alias grid SHALL be drawn
   with the FRAME'S OWN rules and full-width dividers, NOT as a separate nested
   box: a top rule (tee-down at each column divider), the header band, a
   header/data rule (crosses), one band per data row, and then the
   Content_Area's closing rule (tee-up at each column divider), exactly as in the
   design 9.11.1 fixture. This behaviour is SHARED and defined in
   `screen-snapshot-scrm` Requirement 30.32-30.39 and `ispf-table-editor`
   Requirement 12.1-12.3; the Alias Kind only supplies the table
   (`model.tables[0]`). It BUILDS ON the shared box-drawing content grid and the
   framed renderer delivered by CR-NR-115/117 (screen-snapshot-scrm Requirement
   30.12-30.14); it introduces NO bespoke grid. `[CR-NR-115]`

7.18. COLUMNS AND HEADER. THE grid columns SHALL be, in order, Act, CE, Command,
   Alias, Parameters, Description (criterion 7.2), with plain-text header cells
   (the combo marker lives in the CELLS, not the header - decision a, design
   9.11.9). `[OWNER CONFIRM a]` `[CR-NR-115]`

7.19. LIVE-VALUE FIDELITY (owner issues 3 / D5). THE snapshot grid SHALL mirror
   the live panel's data EXACTLY: (i) the data rows SHALL be the live DISPLAYED
   set (after the CE + Command filter, criterion 7.7) in BACKING order, one per
   working-copy row -- INCLUDING the write-through `X -> EXIT` FFCMD seed row
   (Requirement 8), which appears in the grid like any other alias and is
   editable (the Alias store is now file-backed and seeded, not empty-until-user-
   adds); (ii) each cell SHALL be derived from the same
   `AliasRowEdit` field the live widget edits: Act = `act`, CE =
   `environment.label()`, Command = `canonical`, Alias = `surface`, Parameters =
   `command_template`, Description = `description`. PLACEHOLDER HINT TEXT that
   the live widgets show on an EMPTY field (`_` in Act, `name`, `(&1)`, `note`) is
   NOT a value and SHALL NOT appear in the snapshot; an empty text field is
   shown as underscores (criterion 7.20). The value is mirrored as stored, with
   its original case (e.g. `ffcmd`, not `FFCMD`). `[OWNER CONFIRM e]`
   `[CR-NR-115]`

7.20. CELL CONVENTIONS AND WIDTHS (owner issue 4; screen-snapshot-scrm
   Requirement 29.8, 29.9, 30.13). THE Alias grid cells SHALL use these
   per-column conventions, each padded to the column's snapshot field width
   (Act 4, CE 10, Command 20, Alias 16, Parameters 28, Description 32 - declared
   once as host data, design 9.11.3) and WIDENING (never truncating) when a value
   is longer:
   - Act: TEXT cell. Empty = `____`; a pending line-command letter is shown
     padded (e.g. `D   `).
   - CE: COMBO cell. `[ffcmd v]` / `[ffedit v]`.
   - Command: COMBO cell. `[SNAPSHOT v]`; an empty selection = `[ v]`.
   - Alias, Parameters, Description: TEXT cells. Empty = a run of underscores
     filling the field width; filled = the value padded to the field width.
   `[OWNER CONFIRM a]` `[CR-NR-115]`

7.21. SCRATCH ROW (owner issue 5). THE grid SHALL end with ONE always-present
   empty insertable SCRATCH row after the last displayed data row, built from
   the SAME row-to-cells mapping as a data row applied to the blank row
   (`AliasRowEdit::default()`: Act `____`, CE `[ffcmd v]`, Command `[ v]`,
   Alias/Parameters/Description underscores). The live panel SHALL show the same
   trailing scratch row (ispf-table-editor Requirement 12.5), so the snapshot
   mirrors live. The scratch row is NOT counted in `Aliases: <n>` or
   `Rows 1-<n> visible`, and is never persisted unless the user edits it.
   `[OWNER CONFIRM f]` `[CR-NR-115]`

7.22. FILTER ROW. THE snapshot grid SHALL NOT include the live FILTER row (the
   CE / Command filter dropdowns under the headers): it is an input control, not
   data. An ACTIVE filter SHALL instead be reported by the status cell of
   criterion 7.25, so a snapshot of a filtered list is never mistaken for the
   whole list. `[OWNER CONFIRM b]` `[CR-NR-115]`

7.23. CELL SAFETY. WHEN a cell value contains a control character (tab, CR,
   LF) THE provider SHALL replace it with a single space; a cell value is carried
   as a discrete string and SHALL NOT be re-split on the frame's ` | ` band
   separator (a Description containing ` | ` stays inside its own cell). No cell
   is truncated (screen-snapshot-scrm Requirement 29.6, 30.14). `[CR-NR-115]`

7.24. CLIPBOARD FLAVORS. THE Alias model SHALL feed the Markdown and HTML-table
   clipboard flavors from `ScreenModel.markdown_sections` (one `Aliases` section
   with RAW values: no `[v]` marker, no underscore fill, no padding), using the
   SAME `markdown_sections` mechanism as the Menu Kind's Option|Command|
   Description clipboard table (CR-NR-117), so the box carries the cell
   conventions while the structured flavor stays pasteable data
   (screen-snapshot-scrm Requirement 28.6, 30.40). The clipboard SHALL contain
   the box FIRST and the `Aliases` table once; it SHALL NOT contain the
   Property/Value table of criterion 7.15. `[OWNER CONFIRM g]` `[CR-NR-115]`

7.25. WORKSPACE_STATUS_SEGMENT CONTENT. THE Alias Kind's `workspace_status_cells`
   (workspace-framework Requirement 7.17, 7.21) SHALL be APPENDED after the
   Common_Status_Segments `Last: HH:MM | Prof: <name> | Zoom: <pct>`
   (screen-snapshot-scrm Requirement 30.10, CR-NR-115) and SHALL return, in order:
   (1) `Aliases: <n>` where `<n>` = the number of working-copy rows
   (`AliasEditorState.rows.len()`, scratch row excluded);
   (2) `Rows 1-<m> visible` where `<m>` = the number of rows passing the CE +
   Command filter (`displayed_indices(..).len()`); `Rows 0 visible` when none
   (the range is the LOGICAL visible set; it SHALL NOT read egui scroll state);
   (3) ONLY WHILE a filter is active, `Filter: <ce>/<root>` using `--` for an
   unset half (e.g. `Filter: ffcmd/--`).
   None of the cells SHALL contain the band separator ` | `. The cell strings are
   localisable `ff_i18n` messages following the existing
   `ff-desktop-shell-status-*` pattern. `[OWNER CONFIRM d]` `[CR-NR-115]`

7.26. STATUS LIVE/SNAPSHOT PARITY. THE live Status_Bar and the snapshot/capture
   status band SHALL both obtain the Alias cells from the ONE method
   `WorkbenchShell::workspace_status_cells()` (a new `KindTag::AliasEditor` arm),
   so they are identical by construction; the Alias Kind SHALL add NO other
   status text and SHALL NOT contribute `Ln`/`Col`/encoding/line count.
   `[CR-NR-115]`

7.27. NO FRAMEWORK CHANGE. THE Alias content-area and status work SHALL change
   NO dispatch path, NO navigation, NO `WorkspaceKind`/descriptor, NO focus
   contract (the first-Tab test of 7.8 is unchanged), NO save path, and NO public
   `ff-screen-model` or `WorkspaceContext` type. The additive pieces (renderer
   form of 7.17, trailing scratch row of 7.21, `ColumnSpec` cell metadata) are
   flagged in design 9.11.8 and do not reshape a public framework type.
   `[CR-NR-115]`

7.28. VERIFICATION. THE Alias goldens (`golden/alias_pfshow_on.txt`,
   `alias_message_pfshow_off.txt`, `alias_message_pfshow_off_ascii.txt`) SHALL be
   regenerated to the design 9.11.1 layout and compared line for line; unit
   tests SHALL cover the row-to-cells mapping (value fidelity incl. placeholder
   exclusion, the `X -> EXIT` seed row present, filter, scratch row), the status
   cells (count, filtered range, active-filter cell), and the absence of
   `fields`; the existing Alias snapshot tests that assert raw cell strings
   (`tests_editor_snapshot_tables.rs`, `tests_screen_directive.rs`) SHALL be
   updated to the cell conventions. `[CR-NR-115]`

---

### Requirement 8: FFCMD seed alias `X -> EXIT`, written on first run and rebuilt by Reset bare (CR-CH-116)

> **FRAMEWORK CHANGE (owner-confirmed 2026-11-04).** Part of the write-through
> config model (configuration-system Requirement 23). Today the FFCMD alias layer
> starts from an IDENTITY seed (every root maps to itself) assembled in memory and
> never written; the FFEDIT base carries the genuine aliases `X -> EXCLUDE` and
> `INCLUDE -> SHOW`. Under CR-CH-116 a single genuine SEED alias `X -> EXIT` is
> added to FFCMD and is MATERIALISED to disk on first run (so it is visible and
> editable like any user alias) and rebuilt by Reset bare. No OTHER verb becomes a
> seed alias in this change (owner decision 2026-11-04).

**User Story:** As a user, I want `X` to act as EXIT everywhere by default, defined
as a visible, editable command alias rather than hidden logic, so that I can see it
in the Alias Editor and it is restored when I reset to barebones.

**Source:** [CR-CH-116] owner decision; builds on Requirement 5 (the alias store
and resolver) and the write-through model (configuration-system Requirement 23).

#### Acceptance Criteria

1. THE FFCMD seed alias set SHALL include exactly one genuine seed alias in
   addition to the identity mappings: `X -> EXIT` (surface `X`, canonical `EXIT`,
   environment `ffcmd`). No other built-in verb SHALL be turned into a seed alias
   by this change.
2. WHEN the workbench starts AND the FFCMD seed alias file for the active locale is
   ABSENT, THE workbench SHALL perform a First_Run_Write (configuration-system
   Requirement 23.3) of `aliases/<locale>/FFCMD.toml` containing the `X -> EXIT`
   row (plus any identity rows the writer emits), BEFORE the resolver seeds, so the
   resolver reads the just-written file. The locale `<locale>` SHALL be the active
   `ui.locale` (default `en`).
3. THE seed alias write SHALL go through the SAME alias base seam the resolver and
   the Alias Editor use (`dir_overrides.user_aliases` / the `FFWB_USER_ALIASES_DIR`
   env seam, configuration-system Requirement 23.5), so the shell test suite can
   isolate it to a temporary directory.
4. WHEN Reset bare runs (configuration-system Requirement 19.16/19.17), THE shell
   SHALL archive the existing `aliases/` directory and then REBUILD the
   `aliases/<locale>/FFCMD.toml` seed file (containing `X -> EXIT`) from the
   Recovery_Source via the shared Config_Rebuild_Helper, so a barebones reset
   always restores `X -> EXIT`.
5. THE `X -> EXIT` FFCMD seed SHALL COEXIST with the FFEDIT base alias
   `X -> EXCLUDE` (Requirement 5): the two live in DIFFERENT Command Environments
   (ENV encoded in the file name / the row's environment), and the active-wins
   resolution order (the Active_Environment interpreter first, then FFCMD) SHALL
   ensure that bare `X` in an editor (FFEDIT) resolves to EXCLUDE while bare `X` on
   every non-editor Context resolves through FFCMD to EXIT (menu-workspace
   Requirement 14.13). This requirement SHALL NOT alter the FFEDIT `X -> EXCLUDE`
   base alias.
6. THE `X -> EXIT` row SHALL appear in the Alias Editor's FFCMD view (Requirement 5
   / 6) like any other alias, and SHALL be editable/removable by the user exactly
   like a user-authored row (the write-through model exposes it; it is not hidden
   logic). Removing it is the user's prerogative; Reset bare restores it
   (criterion 4).
7. THE resolver SHALL load the seed `X -> EXIT` through the EXISTING overlay path
   (Requirement 5.7 -- the User_Alias_Store overlay routed into the FFCMD
   AliasTable), introducing NO new resolution mechanism, dispatcher, or intercept.
   `X` remains NOT a registered command (there is no command named `X`); it resolves
   as an alias of the existing `EXIT` command (startup-and-session Requirement 20.3).

#### Out of scope (Requirement 8)

- Turning any verb other than `X` into a seed alias.
- Changing the FFEDIT `X -> EXCLUDE` base alias or any other existing alias.
- The X / END / RETURN / EXIT command semantics themselves (menu-workspace
  Requirement 14, navigation-commands Requirement 21) -- this requirement only
  defines the SEED alias row and its first-run/Reset-bare lifecycle.
