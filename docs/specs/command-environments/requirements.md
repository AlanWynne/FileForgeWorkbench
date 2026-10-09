# Requirements Document -- Command Environments (CR-CH-053)

## Introduction

This sub-project defines the Command Environment model for FileForgeWorkbench,
modelled on the REXX/ISPF ADDRESS pattern: FFWB is not one giant command set but
a set of named environments, each owning the commands relevant to its context.
A user (or macro) works within an environment based on what they are doing; the
active Workspace Context selects the active environment; an always-present base
environment is the fallback; an explicit address targets a specific environment.

The FULL intended catalogue of FFWB environments (FFCMD, FFEDIT, FFLINE,
FFBROWSE, FFAMS, FFJES, FFJOB, FFSQL, FFCICS, FFVFS, FFADMIN, FFDEBUG, FFMON,
FFLIB) and their IBM equivalents are documented in `environments-vision.md` (a
vision/reference document). This requirements document is the AUTHORITATIVE home
for the environment MODEL and ROUTER, and it is deliberately PHASE-1 SCOPED.

### Phase-1 buildable scope

Only the environment FRAMEWORK plus the two environments that already exist in
the running app are BUILT by this gate:

- **FFCMD** -- the shell/workbench base environment. It ALREADY EXISTS: it is
  `ff_command::resolve_target` driven by `ShellTargetResolver` /
  `builtin_workspace_target_for`. No rewrite; CR-CH-053 names it FFCMD and treats
  it as the always-present base.
- **FFEDIT** -- the editor command-line verbs that bug B080 correctly left on the
  shell ladder (`commands_ladder_b2.rs`). CR-CH-053 gives them their home as the
  FFEDIT environment, migrated off the shared ladder.

**FFLINE** (prefix-area line commands) and **FFNAV** (the file-navigator
environment, whose FIND locates a file rather than searching a buffer) are
NAMED/MODELED only in phase 1. FFLINE's intake (prefix gutter ->
`ff-command-semantics` Command_Engine) already works and is not rebuilt; FFNAV's
file-navigator command handling stays wherever it is today and is not migrated.
Both are named so the model is coherent and the active-env derivation accounts
for them, but neither is BUILT as a first-class environment in phase 1. Every
OTHER environment in the catalogue is VISION-ONLY: no criterion here requires
building it; each registers when its consuming subsystem is built.

### Framework-conformance stance

This model BUILDS ON the existing framework and adds NO second dispatcher and NO
second navigation stack. FFCMD IS `resolve_target`. The router inserts ONE
active-environment step into the single B080 front door
(`dispatch_command_string`) BEFORE `resolve_target`; the existing prelude
(stage-1 current-menu Option_Key, EXIT family, POM/chained fastpaths) stays FIRST
and unshadowable. The `CommandTarget` enum is unchanged. This does NOT implement
CR-CH-052 (`=` / X / =X navigation-ladder semantics) and does NOT design the
database tool or any other future environment.

### Source references

- **[CR-CH-053]** = this change request (owner: the REXX ADDRESS multi-environment
  model for FFWB).
- **[B080]** = the single command-dispatch front door (`dispatch_command_string`),
  the plumbing this model generalises.
- **[WB]** = Workbench Architecture Brief, command-driven architecture.
- **design-proposal** = `.agents/tasks/command-environments/design-proposal.md`.

### Cross-references

- `command-framework` Req 2 (single dispatch), Req 5 (Shortcut_Registry), Req 6
  (Scripting_Bridge), Req 8 (CommandTarget + ordered Target_Resolution chain, 8.3
  / 8.4 / 8.10 shadowing), Req 9 (one verb/arg split), Req 10 (Navigation_Stack).
- `command-semantics` Req 1-3 (Command_Engine, scope resolution, Primary_Command
  parser) -- the EDITOR verb vocabulary and the FFLINE intake path.
- `lua-macro-engine` Req 11.11-11.15 (TSO / ISPEXEC / ISREDIT host command
  environments, `ADDRESS <env>`, `RC`) -- the macro addressing surface this model
  reconciles with (does not duplicate).
- `environments-vision.md` -- the FF* environment catalogue.

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Command_Environment** | A named resolver+executor owning a command vocabulary for a context; it may CLAIM a submitted command string (handle it) or fall through. | [CR-CH-053] |
| **Environment_Registry** | The shell-owned collection of environments: the FFCMD base plus zero or more Context environments; derives the Active_Environment and resolves a name for addressing. | [CR-CH-053] |
| **Active_Environment** | The environment selected by the currently FOCUSED Workspace Context. | [CR-CH-053] |
| **Base_Environment (FFCMD)** | The always-present fallback environment; it IS `resolve_target` / `ShellTargetResolver`. Owns the Context-opener/workbench verbs. | [B080] |
| **FFEDIT** | The editor command-line environment: LOCATE/FIND/CHANGE/EXCLUDE/SORT/profile/scroll verbs acting on the active editor buffer via the existing managers. | [CR-CH-053] |
| **FFLINE** | The prefix-area line-command environment (D/DD/M/C/CC/A/B...), a sibling of FFEDIT; intake is the prefix gutter -> Command_Engine, NOT the command line. Named/modeled only in phase 1. | [CR-CH-053] |
| **FFNAV** | The file-navigator environment (FilesPanel / FileExplorerPanel / catalog Contexts); owns navigator verbs incl. its OWN FIND (locate a file, not search a buffer). Named/modeled only in phase 1; its command handling is not migrated. | [CR-CH-053] |
| **Address** | Directing a command to a named environment regardless of the active one (REXX ADDRESS). Phase 1: macros only. | [lua-macro-engine Req 11.12] |
| **Alias_Map** | The mapping of mainframe environment names onto FF* names for addressing: TSO -> FFCMD, ISREDIT -> FFEDIT (future IDCAMS -> FFAMS, SDSF -> FFJES, ...). | [CR-CH-053] |
| **Record_Store_Contract** | The record-aware store entry on `BackendEnvironment` added alongside the byte `save`: it carries a Store_Target (dataset identity), a record stream, and record attributes (RECFM/LRECL/encoding), so a backend CE can store framed records rather than an opaque byte buffer. Host CEs do NOT implement it (they keep the byte `save`). | [CR-CH-060] |
| **Store_Target** | The dataset-identity object the Record_Store_Contract carries: DSN / catalog identity / owning-environment name -- NOT a host path. It is what lets the mainframe CE resolve the dataset through `ff-volume` and store via `ff_dscatalog::DatasetAccess`. | [CR-CH-060] |

---

## Requirements

### Requirement 1: Command Environment model

**User Story:** As a workbench developer, I want commands grouped into named
environments owned by their context, so that each context exposes only the
commands that make sense for it, mirroring the IBM mainframe multi-environment
model.

#### Acceptance Criteria

1. THE framework SHALL define a Command_Environment as a named resolver that, given
   a submitted command string, either CLAIMS it (executes it and reports an
   outcome) or declines (falls through to the next environment).
2. THE FFCMD environment SHALL be the always-present Base_Environment, and it
   SHALL BE the existing `resolve_target` / `ShellTargetResolver` chain -- no
   rewrite and no second classifier is introduced for it.
3. THE framework SHALL maintain an Environment_Registry containing the FFCMD base
   and zero or more Context environments, exposing: the Active_Environment for the
   focused Context, lookup of an environment by name (for addressing), and the
   FFCMD base as fallback.
4. THE Environment_Registry SHALL NOT be a second dispatcher: it SHALL feed the
   single front door (`dispatch_command_string`), not run a parallel dispatch path.

### Requirement 2: Active environment derivation

**User Story:** As a user, I want the command line to understand the commands of
whatever I am currently working in, without my having to declare it.

#### Acceptance Criteria

1. THE Active_Environment SHALL be SUPPLIED BY the focused Workspace Context's
   KIND as a kind ATTRIBUTE (a named reference), NOT determined by a central
   hardcoded match in the command handler. The command handler SHALL ask the
   focused kind for its command-environment name and use it; it SHALL NOT
   enumerate the set of kinds. WHEN a kind supplies no environment, THE
   Active_Environment SHALL default to the FFCMD base. (This inverts the
   dependency: a new kind declares its environment and plugs in with NO change to
   the handler -- see Requirement 9.) The phase-1 kinds declare: editor Contexts
   (FileEditor / Untitled) -> FFEDIT; file-navigator Contexts (FilesPanel /
   FileExplorerPanel / catalog) -> FFNAV (NAMED now, see Requirement 7a); every
   other kind -> FFCMD base. These mappings are the KINDS declaring their
   environment, not the handler's knowledge.
2. WHERE the Workbench is split or a Workspace is detached, THE Active_Environment
   SHALL follow the FOCUSED region/window's Context (not an arbitrary tab).
3. THE FFCMD base SHALL remain reachable as the fallback regardless of the
   Active_Environment, so a workbench verb typed while another environment is
   active still resolves (the base-environment relay).

### Requirement 2a: Per-environment verb ownership (same name, different command)

**User Story:** As a mainframe-minded user, I want a verb like FIND to mean what
it should IN THE CONTEXT I am in -- find text in the open file in the editor, find
a file in the navigator -- the way the same verb differs between ISPF Edit and
other environments.

#### Acceptance Criteria

1. A verb NAME (e.g. FIND, LOCATE, SAVE, X) MAY be owned by MORE THAN ONE
   Command_Environment, each with its OWN implementation; there is NO single global
   definition of such a verb.
2. WHEN a verb owned by multiple environments is submitted, THE Active_Environment's
   implementation SHALL run (e.g. FIND in FFEDIT searches the open file; FIND in
   FFNAV locates a file; SAVE in FFEDIT writes the active buffer; SAVE in the Theme
   Editor Context writes the theme working copy). The environment IS the namespace.
3. THIS SHALL NOT change today's observable behaviour (Requirement 4), save for the
   explicitly-listed exceptions in Requirement 4: where a verb already behaves
   differently by context today, the environment model FORMALISES that
   context-dependence rather than altering it.
4. OWNERSHIP PRINCIPLE: a verb SHALL be owned by the Command_Environment in whose
   CONTEXT it is meaningful. A verb that acts on the active editor buffer (SAVE,
   CANCEL, UNDO, REDO, FIND, CHANGE, EXCLUDE, the profile/scroll verbs) is owned by
   FFEDIT; a verb that acts on a Context's working copy (a theme / menu / key map /
   kind config / configuration edit) is owned by THAT Context's environment; a verb
   that acts on the workbench (open a Workspace, navigate menus, FILES, CONFIG) is
   owned by FFCMD. The same name (SAVE) therefore resolves to different
   implementations by active environment; there is no context-free global SAVE.
5. FFCMD (the base) HAS NOTHING OF ITS OWN TO SAVE: saving is always SOME context's
   save (a buffer, a theme, a menu, a config), so there SHALL be no `FFCMD.SAVE`.
   SAVE exists only in environments that hold a working copy. WHEN SAVE is typed
   with no save-capable environment active, it SHALL NOT silently fall through to a
   base SAVE (there is none); it resolves to nothing / an unresolved-command result
   exactly as any other unowned verb.
6. WITHIN FFCMD, `X` and `RETURN` are ALIASES for `EXIT` (close the Workspace /
   exit when it is the last), resolved by FFCMD's own alias resolution the same way
   FFEDIT resolves X -> EXCLUDE. These are FFCMD-owned verbs: the active environment
   gets first crack (so bare `X` in the editor is FFEDIT EXCLUDE), and FFCMD
   receives `X` / `RETURN` / `EXIT` only when the active environment rejects them.
   The close-the-application effect MAY be performed by an upper layer FFCMD calls.

### Requirement 3: The front-door environment router

**User Story:** As a maintainer, I want environments resolved through the one
existing command front door, so there is a single dispatch path, not several.

#### Acceptance Criteria

1. THE command handler SHALL NOT special-case individual verbs. It SHALL take the
   command-line string and offer it to resolvers in a FIXED ORDER, each of which
   either CLAIMS the string (handles it) or REJECTS it (the handler continues to
   the next). The order is: (a) the `=` universal rule (criterion 3.2) FIRST; (b)
   the History_Record and the Menu_Context-only fastpaths (stage-1 current-menu
   Option_Key, POM / chained -- inert outside a menu Context); (c) the
   Active_Environment (e.g. FFEDIT when an editor Context is focused) -- the active
   environment gets FIRST CRACK at the verb; (d) the FFCMD base (`resolve_target`
   and the workbench verbs it owns), which RECEIVES ANY STRING THE ACTIVE
   ENVIRONMENT REJECTED; (e) the Command_Engine terminal as the final fallback.
   There is NO privileged "exit family" branch: `X` / `RETURN` / `EXIT` / `QUIT` /
   `LOGOFF` are ordinary FFCMD verbs that FFCMD receives when the active
   environment does not claim them. (The close-the-application side-effect of
   `EXIT` may be performed by an UPPER layer that FFCMD calls, but FFCMD is the
   resolver that RECEIVES the string -- it is not a handler special case.)
2. THE `=` prefix is a SINGLE UNIVERSAL RULE handled in ONE place, BEFORE the
   Active_Environment is consulted: WHEN `=` is the FIRST character of the command
   string, THE handler SHALL drop the current tab's Navigation_Stack (return to the
   POM top) and then dispatch the REMAINDER of the string as an FFCMD / base
   command from that top. Semantically `=` means "not your business, environment --
   this is FFCMD, go back to POM". The Active_Environment therefore NEVER receives a
   `=`-prefixed string (the environment is skipped for that command). This
   generalises the existing `=`-origin rule (command-framework Req 10.2) to all
   environments.
2a. CONSEQUENCE for FFEDIT (and every environment): because `=` is stripped and
   rerouted before the environment is consulted, a `=`-prefixed command in the
   editor behaves as the FFCMD/base command -- e.g. `=X` in the editor does the
   SAME as RETURN/EXIT (drop to POM, run `X` as the base verb), NOT FFEDIT's
   EXCLUDE. Only the BARE form (`X`) is environment-sensitive (FFEDIT EXCLUDE).
3. THE router SHALL insert the Active_Environment step into the SAME single front
   door, AFTER the `=`/history/menu-fastpath universal steps and BEFORE the FFCMD
   base; it SHALL NOT create a second dispatcher and SHALL NOT add a second
   navigation stack.
4. THE `CommandTarget` enum and `resolve_target` SHALL be unchanged by this model.

### Requirement 4: Backward compatibility

**User Story:** As a user, I want every command that works today to keep working
identically after environments are introduced.

#### Acceptance Criteria

1. THE introduction of Command_Environments SHALL NOT change the observable result
   of any command string that resolves today (mirrors command-framework Req 8.4),
   WITH ONE DELIBERATE EXCEPTION: bare `X` typed on an editor Context, which today
   is claimed as a close/exit (`X` resolves to the base exit) before the editor
   verbs are reached, SHALL now run FFEDIT's EXCLUDE (Req 5.1). This is an intended
   correction -- in the editor `X` is the ISPF EXCLUDE line/primary command, and
   the active environment must get first crack at it before FFCMD. The base
   exit/return remains reachable in the editor via `=X` (Req 3.2 / 3.2a / 5.2b) and
   via END/RETURN. On every NON-editor Context bare `X` is unchanged (it is
   rejected by the active environment and FFCMD receives it, close/exit as today).
2. WHEN an editor command-line verb (FIND, CHANGE, LOCATE, ..., and bare `X` =
   EXCLUDE per criterion 4.1) is submitted on an editor Context, THE result SHALL
   be identical to the current shell-ladder behaviour of that FFEDIT verb,
   including the B062 rule (verb matched case-insensitively, Argument_String case
   preserved).
3. WHEN a workbench verb (FILES, CONFIG, ...) is submitted, THE result SHALL be
   identical to today: the active environment rejects it and FFCMD = `resolve_target`
   handles it, unchanged.

### Requirement 5: Shadowing (active-wins)

**User Story:** As a user, I want the environment I am working in to take
precedence for a shared verb name, with the base reachable by addressing.

#### Acceptance Criteria

1. WHEN a verb name exists in BOTH the Active_Environment and the FFCMD base, THE
   Active_Environment SHALL claim it (active-wins), mirroring the earlier-stage-wins
   shadowing rule of command-framework Req 8.10. The showcase collision is bare
   `X`: in an editor Context the Active_Environment is FFEDIT, so bare `X` SHALL
   run FFEDIT's EXCLUDE; in a non-editor Context FFEDIT is not active (it rejects
   `X`), so FFCMD receives `X` and performs the close/exit exactly as today.
2. THE FFCMD instance of a shadowed verb SHALL remain reachable WITHOUT changing
   the focused Context by: (a) phase 1 -- an explicit macro address; AND (b) the
   `=` universal prefix -- `=X` (and any `=<verb>`) is stripped-and-rerouted to
   FFCMD/POM BEFORE the environment is consulted (Req 3.2), so `=X` always reaches
   FFCMD's `X` (close/exit) regardless of the active environment. This gives an
   in-editor keyboard escape hatch to the base verb without leaving the editor.
3. THE universal steps (criterion 3.2 / 3.1b: the `=` rule, the History_Record,
   and the Menu_Context-only fastpaths) SHALL take precedence over BOTH environments
   and SHALL NOT be shadowable. FFCMD's own verbs (including `X` / `RETURN` /
   `EXIT`) are NOT universal -- they are the base that receives whatever the active
   environment rejects, so the bare verb `X` IS shadowable by the Active_Environment
   (criterion 5.1); the `=`-prefixed form is routed past the environment by the
   universal `=` rule, not by any FFCMD precedence (criterion 5.2b).

### Requirement 6: The FFEDIT environment

**User Story:** As an editor user, I want the editor's command-line commands to
live in the editor's own environment rather than the shell command set.

#### Acceptance Criteria

1. THE framework SHALL provide an FFEDIT Command_Environment that owns the editor
   command-line verbs: LOCATE, TOP, BOTTOM, UP, DOWN, LEFT, RIGHT, SORT, EXCLUDE
   (and alias X), SHOW (and alias INCLUDE), RESET, FIND, RFIND, CHANGE, RCHANGE,
   CAPS, NULLS, STATS, LOCK, PROFILE, HILITE, SCROLL.
2. THE FFEDIT environment SHALL execute each verb against the active editor through
   the EXISTING managers (navigation, exclude/show, find, edit profile, scroll
   amount) -- it SHALL NOT reimplement their logic and SHALL NOT front the
   Command_Engine in phase 1.
2a. WHEN an editor Context is active, THE FFEDIT environment SHALL CLAIM its owned
   verbs (Requirement 6.1) at the active-environment step -- i.e. BEFORE the FFCMD
   base and BEFORE the Command_Engine terminal fallback -- preserving today's
   precedence in which the shell-ladder arm handled these verbs ahead of the
   engine. The Command_Engine remains the final fallback for input FFEDIT does not
   claim (e.g. the prefix-area/line-command path and any engine-only primary
   command), so FFEDIT and the Command_Engine are not two competing executors of
   the same command-line verb.
3. THE observable result of every FFEDIT verb SHALL be identical to its current
   shell-ladder arm (Requirement 4), including argument parsing (e.g. CHANGE's
   two-argument quoting) and the B062 case rule.
4. THE FFEDIT environment SHALL be the Active_Environment exactly when the focused
   Context is an editor Context (Requirement 2.1).

### Requirement 6a: Verb aliases resolve to a canonical verb (localization-ready)

**User Story:** As a user in a localized install, I want to type a command verb
in my own language (CHERCHER, SUCHEN) and have it behave exactly as the canonical
verb (FIND), so localization can extend to the VERBS without changing behaviour.

#### Acceptance Criteria

1. EACH Command_Environment SHALL resolve a typed surface form (alias) to a single
   CANONICAL verb via a per-environment ALIAS TABLE, BEFORE dispatch. The
   behaviour lives on the CANONICAL verb; the handler SHALL NOT see the alias, so
   `CHERCHER 'foo'` and `FIND 'foo'` take the IDENTICAL code path and produce the
   IDENTICAL result (behaviour-neutral). The existing one-behaviour aliases
   (EXCLUDE/X, SHOW/INCLUDE) ARE this mechanism (surface forms of one canonical
   verb).
2. THE CANONICAL verb SHALL be what is RECORDED, PERSISTED, and used internally
   (command history, macro `command=` values, menu option `command` values,
   keybindings, session/Workspace descriptors). Aliases are COMMAND-LINE INPUT
   convenience only: typing an alias records/persists the canonical verb (type
   `CHERCHER`, store `FIND`), so a macro or saved workspace does not break when
   the locale changes.
3. THE alias table SHALL be matched case-insensitively (B062) and SHALL reject
   collisions at load (an alias equal to another verb's canonical name or alias in
   the same environment is an error), using the same conflict discipline as the
   Shortcut_Registry.
4. Aliases SHALL respect the prelude/shadowing precedence: an alias SHALL NOT let
   a user shadow a prelude-owned verb (EXIT family, menu Option_Key), and
   active-wins shadowing (Requirement 5) applies to canonical verbs.
5. LOCALIZED alias sets (CR-NR-103) SHALL load into these SAME per-environment
   alias tables per locale, adding NO new mechanism -- localization extends to
   verbs purely as additional alias DATA, with no behaviour impact. (Phase 1
   builds the alias-resolution MECHANISM with the existing English aliases;
   per-locale alias data arrives with CR-NR-103.)
6. ALIAS DATA SHALL be organised as ONE catalogue PER LOCALE, selected and loaded
   when the active locale is chosen -- NOT a monolithic union of all locales
   preloaded. Selecting a different locale loads a DIFFERENT alias catalogue into
   the tables, so only the useful (active-locale) surface forms are resident. This
   has NO per-entry locale indicator: the locale is chosen once (the `ui.locale`
   config key), and that selection determines which catalogue loads; individual
   alias rows carry no language tag.
   A per-locale catalogue SHALL itself be organised PER ENVIRONMENT: because the
   alias table is per-environment (criterion 6a.1), a locale's catalogue is a SET
   of per-environment alias tables (e.g. French FFEDIT verbs, French FFCMD verbs,
   French FFNAV verbs), and each loads into the CORRESPONDING environment's table.
   The SAME surface form MAY map to a DIFFERENT canonical verb in a different
   environment within the same locale (as English `X` -> EXCLUDE in FFEDIT but
   EXIT in FFCMD), and the collision rule (criterion 6a.3) is scoped WITHIN one
   environment's table, not across environments. The dimension is therefore
   per-environment-per-locale, not one flat per-locale list.
7. THE English CANONICAL verb names SHALL remain resolvable in EVERY locale (as
   identity entries), with the active locale's surface forms LAYERED ON TOP (base
   English + locale overlay), NOT a full replacement of English. In a French
   install both `FIND` and `CHERCHER` resolve to the canonical `FIND`. A surface
   form that collides across the base and overlay layers is caught by the
   load-time collision rule (criterion 6a.3).

### Requirement 7: The FFLINE environment (named/modeled only in phase 1)

**User Story:** As an architect, I want the prefix-area line commands recognised
as their own environment so the model is coherent and a future macro can address
them, without rebuilding the working line-command path now.

#### Acceptance Criteria

1. THE framework SHALL NAME FFLINE as a Command_Environment that is a SIBLING of
   FFEDIT, conceptually active together with FFEDIT when an editor Context is
   focused.
2. THE FFLINE intake SHALL remain UNCHANGED in phase 1: line commands enter via the
   prefix area and run through the `ff-command-semantics` Command_Engine; FFLINE
   SHALL NOT be routed through the command-line front door.
3. THERE SHALL BE no line-command behaviour change in phase 1: FFLINE is a naming
   and modeling act only, so a future `ADDRESS FFLINE` has a defined target.

### Requirement 7a: The FFNAV environment (named/modeled only in phase 1)

**User Story:** As an architect, I want the file-navigator recognised as its own
environment (its FIND locates a file, distinct from FFEDIT's FIND), so the model
is correct, without migrating the navigator's command handling now.

#### Acceptance Criteria

1. THE framework SHALL NAME FFNAV as the Command_Environment for the
   file-navigator Contexts (FilesPanel / FileExplorerPanel / catalog), and the
   active-env derivation (Requirement 2.1) SHALL map those Contexts to FFNAV.
2. THE FFNAV command handling SHALL remain UNCHANGED in phase 1: navigator verbs
   (including its own FIND / LOCATE) stay wherever they are handled today; FFNAV
   is not built as a first-class resolver and no navigator behaviour changes.
3. FFNAV being named SHALL make the per-environment verb model (Requirement 2a)
   concrete: FFEDIT.FIND and FFNAV.FIND are distinct, each selected by the active
   Context, with no behaviour change in phase 1.

### Requirement 8: Addressing (macros; interactive prefix deferred)

**User Story:** As a macro author, I want to direct a command to a specific
environment the way REXX ADDRESS does.

#### Acceptance Criteria

1. WHEN a macro addresses an environment (REXX `ADDRESS <env>` / the Lua binding),
   THE command SHALL be routed to the named FFWB Command_Environment through the
   existing Scripting_Bridge (command-framework Req 6), reconciled with the
   already-specified TSO / ISPEXEC / ISREDIT host command environments
   (lua-macro-engine Req 11.11-11.14) via the Alias_Map (TSO -> FFCMD, ISREDIT ->
   FFEDIT).
2. WHEN the addressed environment EQUALS the Active_Environment, THE address SHALL
   be redundant: the command SHALL execute identically with or without it.
3. THE environment dispatch outcome SHALL carry a return code that the
   Scripting_Bridge maps to the macro `RC` (lua-macro-engine Req 11.15).
4. WHEN a command line begins with a token that happens to match an environment
   name, THE phase-1 front door SHALL treat the whole line as an ordinary command
   (NO interactive address-prefix parsing); interactive addressing is a deferred
   future extension and the `parse_address_prefix` seam SHALL be reserved but
   inert in phase 1. (This pins the deferral as a testable behaviour: an
   environment-named leading token is not special-cased at the command line.)

### Requirement 9: Future-context template

**User Story:** As a developer adding a future context (database tool, JES, ...),
I want a defined, minimal way to give it its own environment.

#### Acceptance Criteria

1. A new Workspace Context SHALL get its Command_Environment by declaring the
   environment NAME as a KIND ATTRIBUTE (Requirement 2.1); the command handler
   SHALL dispatch to whatever environment the focused kind supplies, with NO
   change to the handler or the single front door when a new kind is added.
2. THE kind's command-environment attribute SHALL be DATA (a name/reference), NOT
   executable code. Selecting or reconfiguring WHICH environment a kind uses
   (including a user reconfiguring it) is permitted and is recoverable via RESET
   BARE like other kind attributes.
3. THE built-in environments (FFCMD, FFEDIT) SHALL be CODE-ONLY and SHALL NOT be
   REPLACEABLE via configuration -- exactly as built-in menus/themes are code-only
   (CR-CH-021). A kind attribute MAY point at a different existing environment,
   but it SHALL NOT supply an environment's executable command implementations.
4. Authoring a NEW environment with new executable command behaviour SHALL be a
   PLUGIN capability (it registers an environment through the plugin API and is
   subject to the plugin permission/security model), NOT a configuration edit.
   (Owner: "if somebody wants the editor to have a different command interface,
   they copy the Editor Plugin and build their own.")
5. Per-VERB customization (adding or overriding specific verbs) SHALL be via a
   layered user environment that SHADOWS the base (Requirement 5), where the new
   verbs are themselves commands/macros subject to the existing command + plugin
   security model -- NOT by replacing a built-in environment wholesale.
6. THIS gate SHALL NOT build any environment beyond the framework, FFCMD, and
   FFEDIT (FFLINE/FFNAV named-only); every other environment in
   `environments-vision.md` is vision-only until its consuming subsystem is built.
### Requirement 10: Editor-buffer verb ownership (E9)

**User Story:** As an editor user, I want the commands that act on the open
buffer (save it, discard its changes, undo/redo its edits) to belong to the
editor's own environment, so they resolve by context like every other editor
verb -- not as workbench commands that happen to look at the active tab.

#### Acceptance Criteria

1. FFEDIT SHALL own the editor-BUFFER verb SAVE (write the active buffer to its
   file), by the ownership principle (Requirement 2a.4): it is meaningful only in
   an editor Context's buffer, not in FFCMD's workbench context. SAVE is
   DIRTY-AWARE and STAYS in the editor (never leaves):

   | Verb | Clean buffer | Dirty buffer |
   |------|--------------|--------------|
   | SAVE | NO-OP (nothing changed since last save; no write, no flag reset) | write the file, STAY in the editor, clear the dirty flag + save point |

   The Dirty state is `TabState.is_modified` (same flag the leave verbs use). The
   dirty write delegates to the existing save operation
   (`tab_manager::save_active_tab`, which writes, clears `is_modified`, and sets
   the document save point). IF the write FAILS (read-only / disk error), SAVE
   STAYS in the editor and surfaces the error (it stays regardless, so no leave to
   suppress). SAVE is NOT a Confirmable_Command (saving is not destructive). The
   only change from today is the CLEAN no-op guard (skip the write when
   `!is_modified`).
2. FFEDIT SHALL own the editor-leave verbs END / CANCEL / RETURN when an editor
   Context is active (its own dirty-aware versions), per this table. The Dirty
   state is the active tab's modified-since-save flag (`TabState.is_modified`; a
   never-saved buffer with content counts as dirty):

   | Verb | Clean buffer | Dirty buffer |
   |------|--------------|--------------|
   | END | return up one level (`nav_end`) | SAVE, then return up one level (no dialog) |
   | CANCEL | return up one level (`nav_end`) | CONFIRM dialog ("changes will be lost"): confirm -> up one level WITHOUT saving; cancel -> stay in editor |
   | RETURN | return to top / POM (`nav_return`) | CONFIRM dialog ("changes will be lost"): confirm -> to top WITHOUT saving; cancel -> stay in editor |

   On a CLEAN buffer all three delegate to the EXISTING navigation primitive
   (unchanged from today). This REPLACES the earlier draft criterion that END /
   RETURN stay plain navigation: in an editor Context they are dirty-aware FFEDIT
   verbs; everywhere else they remain FFCMD / universal navigation (same name,
   environment-specific behaviour -- Requirement 2a).
3. WHEN END is run on a DIRTY buffer, THE save SHALL be attempted first; IF the
   save FAILS (e.g. read-only file, write error), THE command SHALL STAY in the
   editor, surface the error, and NOT leave (never lose changes by leaving after a
   failed save). IF the save succeeds, THE command leaves (up one level).
4. THE CANCEL / RETURN dirty-discard is a Confirmable_Command (command-framework
   Requirement 16): interactive + no switch -> the confirm dialog; `-Y` -> discard
   + leave headless; `-N` -> stay; non-interactive + no switch -> assume cancel
   (stay, do not discard, record "needs -Y"), continue. (The `-Y`/`-N` + interactive
   -flag infrastructure is command-framework Req 16 / Phase confirmable-commands;
   until it lands, the INTERACTIVE dialog path is implemented and is the behaviour
   for typed use.)
5. FFCMD SHALL NOT own a buffer SAVE: there is no context-free buffer to save
   (Requirement 2a.5). WHEN no editor Context is active, SAVE resolves to nothing /
   unresolved like any unowned verb, and END / RETURN are plain FFCMD / universal
   navigation exactly as today (FFEDIT not active -> not claimed).
6. UNDO and REDO are DEFERRED from this slice (owner-flagged scope finding): UNDO
   today is a keyboard-only inline handler (Ctrl+Z in `editor_panel/input.rs`), not
   a reusable command -- claiming it as an FFEDIT verb is an EXTRACTION refactor;
   REDO does NOT EXIST (no redo stack / logic / handler) -- adding it is a NEW
   feature requiring its own gate (undo-redo-transactions). Neither is built in
   this slice; both are recorded for a separate owner decision. (The ownership
   principle still says they BELONG to FFEDIT once they exist as commands.)
7. THIS slice SHALL be TDD'd (red before green) per table row and per verb; the
   SAVE verb and the CLEAN-buffer END/CANCEL/RETURN paths SHALL be behaviour-
   preserving relative to today; the DIRTY-buffer END (save-then-leave) and
   CANCEL/RETURN (confirm-discard) are the NEW behaviour this requirement adds.

### Requirement 11: Per-Context SAVE + command chaining (E10)

**User Story:** As a user, I want SAVE to persist whatever I am editing (a theme,
a menu, a key map, a kind, a configuration -- or a buffer), and I want to chain
several commands on one line and have each run in turn.

#### Acceptance Criteria

1. EACH editing Context (Theme Editor, Menus Editor, Keys Editor, Kinds Editor,
   Config/Settings editor) SHALL own a SAVE in its OWN Command_Environment that
   persists THAT Context's working copy (the theme, menu TOML, key map, kind
   config, or configuration edit). SAVE therefore resolves to the
   context-appropriate implementation by Active_Environment (Requirement 2a.2/2a.4);
   there is no global SAVE and no FFCMD SAVE (Requirement 2a.5).
2. WHEN SAVE is typed on an editing Context whose command environment owns SAVE,
   THE command-line SAVE SHALL perform the SAME persistence as that Context's
   existing Save affordance (button/menu), through the SAME code path (command
   parity): the typed SAVE and the Save button are one path, not two.
3. THE command line SHALL support CHAINING multiple commands separated by `;`.
   THE SPLITTER AND THE PER-SEGMENT LOOP ARE HANDLER-OWNED: the handler splits the
   submitted string into segments ONCE and dispatches each segment, in
   left-to-right order, through the SAME single front door (env -> FFCMD -> engine)
   as if each had been typed and entered on its own. A Command_Environment NEVER
   receives more than ONE already-split segment and NEVER owns the split or the
   "advance to the next segment" bookkeeping -- it remains a pure claim-or-reject
   resolver on a single segment. (This keeps ONE splitter with consistent quoting
   for every environment, rather than each environment splitting its own way.)
4. THE `;` splitter SHALL be QUOTE-AWARE: a `;` inside a quoted argument (e.g.
   `FIND ';'` or `CHANGE 'a;b' 'c'`) SHALL NOT be treated as a segment separator.
5. THE Active_Environment SHALL be RE-EVALUATED per segment, because an earlier
   segment can change the focused Context: WHEN a segment navigates away (e.g.
   `...; =0` returns to the POM), the LATER segments SHALL resolve in the
   environment that was SWITCHED TO (the now-active Context), NOT the environment
   that was active when the line was submitted. A segment is always dispatched
   against whatever environment is active AT THE MOMENT it runs.
6. THE `=` universal rule (Requirement 3.2) SHALL scope to its OWN `;` segment: a
   `=`-prefixed segment drops the ladder and runs that segment from the POM base;
   it SHALL NOT force the remaining segments of the chain to the base.
7. THE chain error policy SHALL be: a segment that fails (e.g. a FIND that reports
   NOT FOUND, or a rejected/unresolved verb) SHALL be recorded (its status
   surfaced) and the chain SHALL CONTINUE with the next segment (best-effort,
   ISPF-like), UNLESS a later-specified explicit stop modifier is provided (no stop
   modifier is defined in this slice; continue-on-error is the phase default).
8. CHAINING SHALL add NO second dispatcher: it is a pre-split that feeds the one
   existing front door once per segment; `CommandTarget`, the navigation stack, and
   the environment model are unchanged by it.
### Requirement 12: FFEDIT CUA editing verbs -- COPY / CUT / PASTE / SELECT ALL / UNDO / REDO (CR-CH-054)

**User Story:** As an editor user, I want the standard editing commands (copy,
cut, paste, select-all, undo, redo) to be FFEDIT verbs that act on the active
buffer, usable both by their reserved keys (command-framework Req 17) and by
typing, and that honour BOTH the cursor selection and the ISPF line-command
markers.

**Source:** [CR-CH-054]. Owner: "copy and cut can also take input from the line
commands, c cc ... if there is no selection in the cursor context they should
look at the line command context ... paste should have a similar action ... take
the paste direction from an 'A' or 'B' in the line command space." Builds on the
ownership principle (Requirement 2a.4) and Cursor_Context (command-framework
Req 12); the keyboard binding is command-framework Req 17.

#### Acceptance Criteria

1. FFEDIT SHALL own the CUA editing verbs COPY, CUT, PASTE, SELECT ALL, UNDO,
   REDO; they act on the active editor buffer and resolve by Active_Environment
   (Requirement 2a.4). Outside an editor Context they are not FFEDIT verbs.
2. COPY/CUT SELECTION SOURCE precedence: (a) IF a Cursor_Context selection is
   present -> operate on that selection (stream/character granular); (b) ELSE IF a
   pending line-command block (`C`/`CC` markers) selects one or more lines ->
   operate on those WHOLE lines; (c) ELSE no-op (status: nothing selected). COPY
   writes the taken content to the clipboard; CUT writes it to the clipboard AND
   deletes it from the buffer (marking the buffer dirty and pushing an undo entry).
3. WHEN COPY/CUT consume a `C`/`CC` line block, the block is the CLIPBOARD SOURCE
   (option a): the marked lines are copied/cut to the OS clipboard -- this is
   distinct from the ISPF in-document `C`/`CC` + `A`/`B` move/copy. The `C`/`CC`
   markers SHALL be CLEARED after a COPY or CUT that consumed them.
4. PASTE DESTINATION precedence: (a) IF the cursor is in the editing space ->
   insert the clipboard at the cursor (stream insert); (b) ELSE IF a pending
   line-command destination marker (`A` = after / `B` = before) is set -> insert
   the clipboard content as WHOLE LINE(S) after/before the marked line; (c) ELSE
   no-op. The `A`/`B` paste is ALWAYS line-granular (clipboard text split on
   newlines, inserted as whole lines, regardless of how it was copied). The
   cursor-in-editing-space destination ALWAYS WINS the tie-break over a pending
   `A`/`B` marker. The `A`/`B` marker SHALL be CLEARED after the paste; the
   CLIPBOARD CONTENT is RETAINED (paste again is allowed).
5. THE selection source and paste destination SHALL be read from the existing
   context inputs -- the Cursor_Context (command-framework Req 12) for the cursor
   selection/position, and the line-command (FFLINE) state for the `C`/`CC` source
   and `A`/`B` destination markers. The verbs take NO explicit area parameter.
   This requires the line-command layer to EXPOSE accessors for the pending
   `C`/`CC` source block and the pending `A`/`B` destination marker (a small,
   deliberate FFLINE -> FFEDIT coupling; the markers are context inputs, same
   category as the cursor selection).
6. CLIPBOARD operations SHALL be backed by the `ff-clipboard` crate (wiring in a
   currently-orphan crate), not a bespoke clipboard access path.
7. SELECT ALL SHALL select the whole buffer (as the Cursor_Context selection), so
   a subsequent COPY/CUT operates on the entire document via criterion 2(a).
8. UNDO SHALL undo the last buffer edit; today the undo logic is an inline
   keyboard handler (Ctrl+Z in `editor_panel/input.rs`) -- this requirement
   EXTRACTS it into the UNDO verb so the key and the verb share one path. REDO is
   a NEW feature (no redo stack exists today); it SHALL be built as part of this
   requirement's slice 2, coordinating with undo-redo-transactions. Until REDO
   exists, Ctrl+Y resolves to REDO but has nothing to redo.
9. THESE verbs SHALL be reachable by both the reserved keys (command-framework
   Req 17.2) and by typing; the key and the typed verb produce the identical
   result. No second dispatcher; dispatched through the one front door.

---

## Phase-1 maturation extension (CR-CH-053, owner-approved DESIGN-BRIEF)

Requirements 13-16 EXTEND CR-CH-053 with the ONE owner-approved core change from
the consolidated DESIGN-BRIEF (file-system-aware VFS + Command Environments +
adaptive editor; `.agents/tasks/mainframe-dataset-emulation/DESIGN-BRIEF.md`,
section 5). They COMPLETE the Environment_Registry this spec already DESCRIBES
(design.md "The Environment_Registry", jobs 1-3) and Req 1.3 already NAMES, plus
the address-by-name seam Req 8/Task 10 already depend on. They do NOT contradict
Requirements 1-12: FFCMD stays `resolve_target`; the active-env step stays the one
front-door/shared-ladder stage; `CommandTarget`, the per-tab Navigation_Stack, and
the WorkspaceContext focus latch are UNCHANGED. No second dispatcher and no second
navigation stack are introduced.

Framework-conformance stance (per `framework-conformance.md`): this is the single
owner-confirmed framework EXTENSION that completes a designed, phase-1-deferred
capability (the built registry + address-by-name + plugin environments of Req
9.4). It is NOT a new mechanism: it replaces the CLOSED set (fixed
`EnvironmentKind` enum, hardcoded `environment_for_kind` match, single hardcoded
`== FfEdit` claim gate) with the OPEN built registry the design already called for.

MODIFIES-vs-ADDITIVE marking (see each criterion):
- MODIFIES working behaviour: Requirement 10.1 SAVE routing (SAVE stops being a
  direct local-FS byte write and is ADDRESSED to the owning environment). The host
  FS environment's SAVE is byte-identical to today, so NATIVE editing is
  unchanged; the modification is the routing path, flagged in Req 14.
- ADDITIVE (behaviour-preserving with only FFEDIT + FFCMD + the host FS env
  registered): the built registry (Req 13), the address-by-name entry point (Req
  14 mechanism), the tab owning-environment binding defaulting to the host FS env
  (Req 15), and the ff-ce-* family modelled with light host-FS CEs (Req 16).

New glossary terms (extend the Glossary above):

| Term | Definition | Source |
|------|-----------|--------|
| **Built Environment_Registry** | The shell-owned collection of `dyn CommandEnvironment` into which environments REGISTER (a plugin capability, Req 9.4), replacing the closed `EnvironmentKind` enum + hardcoded `environment_for_kind` match + hardcoded `== FfEdit` claim gate. Active-env derivation and the claim gate READ FROM it. | [CR-CH-053 / DESIGN-BRIEF 5.1] |
| **Owning_Environment** | The Command Environment of the file system that OWNS a tab's edited resource; the destination FFEDIT addresses store-affecting verbs (SAVE, future CREATE/REPLACE-member, record validation) to. Captured at open from the originating catalog/provider; default = the Host_FS_Environment. | [CR-CH-053 / DESIGN-BRIEF D2, 5.3] |
| **dispatch_to_environment** | The registry's address-by-name entry point (REXX ADDRESS applied internally): given an environment NAME and a raw command, route the command to that named environment. The SAME seam the deferred macro ADDRESS (Req 8 / Task 10) needs; FFEDIT SAVE forwarding is its first consumer. | [CR-CH-053 / DESIGN-BRIEF 5.1(b)] |
| **FS_Command_Environment (ff-ce-*)** | A Command Environment that represents a FILE SYSTEM (not a platform): `ff-ce-ntfs`, `ff-ce-posix` (future `ff-ce-apfs`), and the mainframe CE housed in `ff-idcams`. Owns its file system's store semantics (SAVE write-back, record validation, path/case rules). | [CR-CH-053 / DESIGN-BRIEF D4a] |
| **Host_FS_Environment / Native role** | The FS Command Environment that `ff-ce-host-fs` (a DECIDER, not a file system) resolves the "native" ROLE to for the current host at startup (Windows -> ff-ce-ntfs; Linux/macOS -> ff-ce-posix). "Native" is a ROLE, not a CE; any FS CE is emulatable in a non-native context. | [CR-CH-053 / DESIGN-BRIEF D4a] |
| **Live_Provider_Registry** | The `ff-vfs` `ProviderRegistry` registered in the running shell at startup. Built/tested today but NOT registered live; the prerequisite without which a plugin-provided `VfsProvider` (e.g. the mainframe VFS provider) has nowhere to land. | [CR-CH-053 / DESIGN-BRIEF 5 prerequisite] |

### Requirement 13: Built, pluggable Environment_Registry

**User Story:** As a workbench developer, I want the Environment_Registry to be a
real built collection that environments register into, so that active-environment
derivation and the claim gate read from one open registry rather than a closed
enum and hardcoded match, and a new environment plugs in (per Req 9.4) without
editing a central dispatch site.

**Marking:** ADDITIVE / behaviour-preserving with only FFEDIT + FFCMD (+ the host
FS environment, Req 16) registered. Completes the registry design.md already
describes (jobs 1-3) and Req 1.3 names.

#### Acceptance Criteria

1. THE Environment_Registry SHALL be a BUILT shell-owned collection of
   environments (each a `dyn CommandEnvironment`, Req 1.1), REPLACING the closed
   `EnvironmentKind` enum, the hardcoded `environment_for_kind` match, and the
   single hardcoded `== FfEdit` claim gate as the source of truth for which
   environments exist.
2. AN environment SHALL REGISTER itself INTO the registry (built-in environments
   registered in code at startup; a new executable environment registering as a
   PLUGIN capability per Req 9.4), rather than being enumerated by a fixed enum
   variant and a central match arm.
3. THE active-environment derivation (Req 2.1) SHALL read the Active_Environment
   FROM the registry by the focused kind's command-environment name, and THE claim
   gate (the active-wins step, Req 5.1) SHALL consult the registry's resolved
   active environment -- NEITHER SHALL use the former hardcoded `environment_for_kind`
   match or the `== FfEdit` literal.
4. WHEN the focused kind supplies no environment name, OR names one not present in
   the registry, THE registry SHALL return the FFCMD base (Req 2.1 default), so an
   unresolved name degrades to the base rather than erroring.
5. FFEDIT SHALL be a REAL `CommandEnvironment` OBJECT registered in the registry
   (replacing today's methods-on-the-shell `ffedit_claim`), claiming its owned
   verbs (Req 6.1) through the trait, with its observable result unchanged
   (Req 4.2, 6.3).
6. WITH only FFEDIT and FFCMD (and the host FS environment, Req 16) registered,
   THE observable result of every command SHALL be identical to the pre-registry
   behaviour (Req 4) -- the registry is an open structure around the SAME
   resolution order, not a behaviour change.
7. THE registry SHALL NOT be a second dispatcher or a second navigation stack
   (Req 1.4, 3.3): it feeds the one front door / shared ladder path and leaves
   `CommandTarget` and the per-tab Navigation_Stack unchanged.

### Requirement 14: Address-by-name routing (dispatch_to_environment) and FFEDIT SAVE forwarding

**User Story:** As a macro author and as the editor, I want to direct a
store-affecting command to the environment that OWNS the edited resource (REXX
ADDRESS applied internally), so that FFEDIT stays universal for in-buffer editing
while SAVE (and future store verbs) are performed by the owning file system's
Command Environment.

**Marking:** The address-by-name MECHANISM is ADDITIVE (the registry gains an
entry point). The SAVE REROUTING MODIFIES working behaviour and is flagged against
Requirement 10.1 below: FFEDIT stops executing SAVE as a direct local-FS byte
write and instead ADDRESSes it to the Owning_Environment. The host FS
environment's SAVE is byte-identical to today, so native editing is unchanged.

#### Acceptance Criteria

1. THE Environment_Registry SHALL expose an address-by-name entry point
   `dispatch_to_environment(name, raw)` that routes a raw command to the named
   environment regardless of the Active_Environment (REXX ADDRESS applied
   internally). WHEN the named environment equals the active one, the result SHALL
   be identical with or without addressing (consistent with Req 8.2).
2. `dispatch_to_environment` SHALL be the SAME seam the deferred macro ADDRESS
   (Req 8, Task 10) uses: the macro `ADDRESS <env>` binding and FFEDIT SAVE
   forwarding SHALL both route through this one entry point, not two parallel
   paths. The dispatch outcome SHALL carry a return code (Req 8.3).
3. FFEDIT SHALL classify its verbs into IN-BUFFER verbs and STORE-AFFECTING verbs.
   THE in-buffer verbs (LOCATE, FIND, CHANGE-in-buffer, CAPS, SORT, EXCLUDE,
   NUMBER, UNNUM, BNDS, COLS, and the other verbs of Req 6.1) SHALL be handled by
   FFEDIT DIRECTLY and SHALL NOT be addressed to another environment.
4. THE store-affecting verb SAVE (today the only one; FUTURE CREATE / REPLACE
   member / save-time record validation) SHALL be ADDRESSED by FFEDIT, via
   `dispatch_to_environment`, to the tab's Owning_Environment (Req 15) rather than
   executed by FFEDIT as a direct store write.
5. **(MODIFIES Requirement 10.1 -- flagged.)** FFEDIT SAVE SHALL no longer perform
   a direct local-FS byte write (today `tab_manager::save_active_tab` ->
   `LocalFsProvider` byte write). INSTEAD, FFEDIT SHALL address SAVE to the
   Owning_Environment, whose SAVE performs the write. The dirty-awareness contract
   of Requirement 10.1 (clean = no-op; dirty = write + stay + clear flag + save
   point; write-fail = stay + surface error; not Confirmable) SHALL be PRESERVED
   -- only the EXECUTOR of the write moves from FFEDIT to the Owning_Environment.
6. THE DEFAULT Owning_Environment SHALL be the Host_FS_Environment (Req 16), whose
   SAVE SHALL be BYTE-IDENTICAL to today's `save_active_tab` local-FS write. THEREFORE
   editing a NATIVE file and typing SAVE SHALL produce the identical on-disk result
   and the identical dirty/save-point state as before this change (native behaviour
   unchanged; the only change is the routing path).
7. OWNERSHIP PRINCIPLE: FFEDIT SHALL own in-buffer editing; the Owning_Environment
   SHALL own all store reads/writes and store-dependent validation (e.g. LRECL /
   RECFM enforcement when a non-host FS environment is the owner). This formalises
   Req 2a.4 for the store boundary and introduces no store knowledge into FFEDIT.
8. THIS routing SHALL add no second dispatcher and no second navigation stack: it
   is the registry's address-by-name feeding the one front door / claim path;
   `CommandTarget` and the per-tab Navigation_Stack are unchanged.

### Requirement 15: Tab-to-owning-environment binding (captured at open)

**User Story:** As the editor, I want each tab to record which file system / CE
owns its content, captured when the file was opened, so that FFEDIT can address
store-affecting verbs to the right environment instead of guessing or always
writing to the local FS.

**Marking:** ADDITIVE / behaviour-preserving (default = Host_FS_Environment, so
every file open today binds to the native env and SAVE stays native).

#### Acceptance Criteria

1. A tab SHALL record its Owning_Environment as tab state (an environment NAME /
   reference on `TabState`), identifying the file system / CE that OWNS the tab's
   content.
2. THE Owning_Environment SHALL be CAPTURED AT OPEN from the originating catalog /
   provider (e.g. the `CatalogType` the navigator already knows at open time, which
   is DISCARDED today), threaded through the open command (a `file.open`
   `CommandParams` entry) rather than inferred later.
3. WHEN no originating environment is supplied at open (a plain host-path open as
   today), THE Owning_Environment SHALL DEFAULT to the Host_FS_Environment (Req
   16), so existing opens are behaviour-preserving.
4. FFEDIT SHALL READ the tab's Owning_Environment to choose the target of a
   store-affecting verb (Req 14.4): SAVE on a tab bound to the host FS env writes
   via the host FS env; SAVE on a tab bound to a non-host FS env is addressed to
   that env.
5. THE Owning_Environment binding SHALL persist/restore CONSISTENTLY with the
   existing `WorkspaceDescriptor` model (a tab reopened from its descriptor SHALL
   recapture the same Owning_Environment from its origin); it SHALL NOT introduce
   a new persistence format (framework-conformance mechanism 6).
6. CAPTURING and reading the Owning_Environment SHALL NOT change the observable
   open or SAVE behaviour for a native/host-path file (the default binding keeps
   today's path).

### Requirement 16: The ff-ce-* file-system Command Environment family and ff-ce-host-fs decider

**User Story:** As an architect, I want each file system to be its own Command
Environment (named by the ff-ce-* convention) and a host-fs decider that resolves
the "native" role to the right concrete FS CE per host, so the model generalises
cleanly to NTFS, POSIX, future file systems, and the mainframe dataset CE without
hardcoding a platform.

**Marking:** ADDITIVE / behaviour-preserving. The host-fs decider + light NTFS /
POSIX CEs supply a byte-write SAVE identical to today; the mainframe CE (housed in
ff-idcams) is the first CE that genuinely diverges and is built in a later phase.
Reconciles with Req 9.4 (a new executable environment is a plugin capability) and
DESIGN-BRIEF D4a.

#### Acceptance Criteria

1. THE model SHALL define a Command Environment PER FILE SYSTEM (an
   FS_Command_Environment), named by the `ff-ce-*` convention -- `ff-ce-ntfs`,
   `ff-ce-posix` (future `ff-ce-apfs`), and the mainframe CE housed in `ff-idcams`
   -- each owning its file system's store semantics (SAVE write-back, record
   validation, path/case rules). A file system is a file system regardless of
   which OS hosts it; a CE represents a FILE SYSTEM, not a platform.
2. `ff-ce-host-fs` SHALL be a DECIDER / pass-through (NOT a file system) that at
   startup detects the host platform and RESOLVES the "native" ROLE to the
   matching concrete FS CE (Windows -> `ff-ce-ntfs`; Linux / macOS -> `ff-ce-posix`
   / future `ff-ce-apfs`). "Native" SHALL be a ROLE, not a CE; whatever the decider
   resolves to IS the Host_FS_Environment / default Owning_Environment (Req 15.3).
3. ANY FS_Command_Environment SHALL be EMULATABLE in a non-native context (e.g.
   NTFS-on-Linux, mainframe-dataset anywhere) because the CE owns the semantics,
   not the platform -- the registry (Req 13) holds the family and designates one as
   native per host rather than hardcoding a platform file system.
4. THE INITIAL build SHALL register the host-fs decider + `ff-ce-ntfs` + `ff-ce-posix`
   (both deliberately LIGHT -- relying on the OS for controls, permissions, and
   file attributes, supplying a byte-write SAVE == today's behaviour plus cheap FS
   defaults such as POSIX case-sensitivity vs NTFS case-insensitivity) + the
   mainframe CE housed in `ff-idcams`. Each `impl CommandEnvironment` and registers
   into the Environment_Registry (Req 13.2).
5. THE light NTFS / POSIX CEs' SAVE SHALL be byte-identical to today's local-FS
   write (Req 14.6), so with the host FS env as the default Owning_Environment,
   native SAVE is unchanged.
6. DEEP NTFS / POSIX / APFS semantic emulation and cross-emulation (NTFS-on-Linux,
   etc.) SHALL be DEFERRED -- enabled by this abstraction, not built in the initial
   phase; no criterion here requires building them.
7. A NEW FS_Command_Environment with new executable store behaviour SHALL be a
   PLUGIN capability (registered through the plugin API under the plugin
   permission model, Req 9.4), consistent with the built-in environments being
   code-only and not configuration-replaceable (Req 9.3).

### Requirement 17: Live provider-registry prerequisite (dependency note)

**User Story:** As a developer wiring the mainframe (and other non-host) file
systems, I need the `ff-vfs` `ProviderRegistry` registered live in the running
shell at startup, because without it a plugin-provided `VfsProvider` has nowhere
to land and a non-host Owning_Environment cannot read/write its store.

**Marking:** ADDITIVE shell wiring (register the already-built provider stack at
startup). This is a PREREQUISITE/dependency for the non-host parts of Req 14-16,
NOT a change to the registry or dispatch mechanisms. It is called out explicitly
per the DESIGN-BRIEF section 5 "NON-NEGOTIABLE PREREQUISITE".

#### Acceptance Criteria

1. THE shell SHALL register an `ff-vfs` `ProviderRegistry` LIVE at startup (the
   provider stack is built/tested today but NOT registered live), so that a
   provider can be looked up at runtime by the Owning_Environment.
2. THE Live_Provider_Registry SHALL be the seam a plugin-provided `VfsProvider`
   (e.g. the mainframe VFS provider) registers into; without it, such a provider
   SHALL have nowhere to land (the dependency this criterion records).
3. REGISTERING the provider registry live SHALL be ADDITIVE shell wiring and SHALL
   NOT change the observable behaviour of host-path file access (the host FS
   provider continues to serve native opens/saves exactly as today).
4. THE non-host parts of Requirements 14-16 (addressing SAVE to a mainframe or
   other non-host Owning_Environment that must reach a provider) SHALL DEPEND ON
   this criterion; the host-FS default path (Req 14.6, 15.3) SHALL NOT depend on
   it (native editing works whether or not a non-host provider is registered).

### Requirement 18: Record-aware BackendEnvironment store contract (CR-CH-060)

**User Story:** As the editor saving a Fixed/Variable (mainframe) document, I want
the owning Command Environment's store call to carry the editor's framed RECORDS
plus the dataset identity and RECFM/LRECL/encoding, rather than an opaque byte
buffer, so a mainframe CE can store records through `ff_dscatalog::DatasetAccess`
while native byte saves stay exactly as they are.

**Marking:** OWNER-DIRECTED FRAMEWORK CHANGE (framework-conformance.md).
`ff-vfs::BackendEnvironment` is a load-bearing core framework type, so reshaping
its store contract requires express owner confirmation; the owner confirmed the
DIRECTION ("move forward with both CR-CH-058 and RC.B.8 Part 2", "design the
contract once", "A and B"). This is a SINGLE contract authored ONCE to serve
BOTH CR-CH-058 (the editor SAVE walk) and CR-CH-059 RC.B.8 Part 2 prerequisite
(a) (the mainframe editor SAVE). The drafted contract SHAPE is still subject to
explicit owner approval before any code (a later step). ADDITIVE: the byte
`save` is retained unchanged; native/host SAVE stays byte-identical.

#### Acceptance Criteria

18.1 THE `ff-vfs::BackendEnvironment` trait SHALL RETAIN its existing byte store
   entry `save(&self, path: &Path, bytes: &[u8]) -> io::Result<()>` unchanged, so
   that a light host-FS Command Environment (the `ff-ce-host-fs` decider's
   resolved `ff-ce-ntfs` / `ff-ce-posix`) performs the SAME OS-backed byte write
   as today and native SAVE is byte-identical (Req 16.5; document-model Req 12.12).
   [CR-CH-060]

18.2 THE `ff-vfs::BackendEnvironment` trait SHALL ADD a record-aware store entry
   (the Record_Store_Contract) that carries (a) a Store_Target identifying the
   dataset by DSN / catalog identity / owning-environment -- NOT a host path, (b)
   the editor's framed RECORDS as an object-safe record stream, and (c) record
   attributes (RECFM, LRECL, encoding). The record-aware entry SHALL be ADDITIVE
   (alongside, not replacing, the byte `save`). [CR-CH-060]

18.3 THE record-aware store entry SHALL have a PROVIDED default such that an
   existing host-FS Command Environment compiles and behaves UNCHANGED without
   implementing it (the default declines / reports not-record-capable); only a
   record-aware Command Environment (the mainframe CE) SHALL override it. A
   backend SHALL be able to advertise whether it is record-capable. [CR-CH-060]

18.4 THE `BackendEnvironment` trait SHALL REMAIN object-safe (used as
   `dyn BackendEnvironment` in the Environment_Registry): the record-aware entry
   SHALL take `&self` and non-generic, non-`Self`-returning parameters (an
   object-safe `&dyn` record stream), so `Box<dyn BackendEnvironment>` stays
   valid. [CR-CH-060]

18.5 WHEN the editor saves a document whose owning Command Environment advertises
   a `Delimited` RecordFormat (native/host), THE SAVE SHALL call the BYTE
   `save(path, bytes)` entry (18.1); WHEN the owning CE advertises a `Fixed` or
   `Variable` RecordFormat, THE SAVE SHALL call the record-aware entry (18.2) with
   the re-framed records. The selection SHALL be driven by the owning CE's
   advertised RecordFormat, which the CE supplied at OPEN (document-model Req 11.2)
   -- the save SHALL NOT re-derive record framing independently. [CR-CH-060]

18.6 THE record-aware store call SHALL ride the SINGLE existing SAVE-addressing
   seam (CR-CH-053 Task 20/21: FFEDIT addresses SAVE to the owning Command
   Environment via `dispatch_to_environment`); it SHALL NOT introduce a parallel
   save path or a second dispatcher. [CR-CH-060, framework-conformance.md]

18.7 THE mainframe Command Environment (housed in `ff-idcams`, Req 16.1 / 16.4)
   SHALL implement the record-aware store entry over
   `ff_dscatalog::DatasetAccess` (open -> put each record -> close), so the RECFM
   codec frames the record bytes on close; it SHALL resolve the dataset location
   through `ff-volume` and perform I/O over the single `ff-vfs::StorageProvider`
   seam, keeping the dependency DAG acyclic (`ff-idcams` -> `ff-dscatalog` ->
   `ff-volume` -> `ff-vfs`). [CR-CH-060, CR-CH-059]

18.8 THE record-aware store outcome SHALL carry a return code (mirroring
   `BackendOutcome`) that the addressing caller maps to a status / macro `RC`
   (consistent with Req 14 / lua-macro-engine Req 11.15), including the mainframe
   failure semantics `DatasetAccess` already surfaces (e.g. x37 space-full
   abends). [CR-CH-060]
