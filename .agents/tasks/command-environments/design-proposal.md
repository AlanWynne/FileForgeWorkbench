# Design Proposal -- Command Environments (CR-CH-053)

READ-ONLY design investigation. No source or spec file was changed. This is a
PROPOSAL for owner review that will drive the requirements gate for change
request CR-CH-053 "Command Environments". Plain ASCII only (`--`, `->`, straight
quotes).

Framework-conformance stance (binding): this proposal BUILDS ON the existing
command framework. It adds NO second dispatcher and NO parallel navigation
stack. `ff_command::resolve_target` + `CommandTarget` already IS the SHELL
environment's resolver; this proposal generalises the ONE front door
(`dispatch_command_string`, B080) to N environments. It does NOT change the
`CommandTarget` enum, does NOT implement CR-CH-052 (`=`/X/`=X` ladder
semantics), and does NOT design the database tool.

---

## 0. Summary answer (for the owner, read this first)

The owner's REXX/ISPF ADDRESS analogy maps cleanly and almost exactly onto what
the codebase already has, with ONE honest gap it also fixes:

- The SHELL environment ALREADY EXISTS. It is `ff_command::resolve_target`
  (`crates/ff-command/src/command_target.rs:267`) driven by
  `ShellTargetResolver` / `builtin_workspace_target_for`
  (`crates/ff-desktop/src/command_config/mod.rs:130-210`). B080 Step 2 already
  migrated the Context-opener verbs (FILES, CONFIG, LOG, KEYS, ...) into it. It
  owns the verbs that classify to a `CommandTarget`.
- The EDITOR environment DOES NOT YET EXIST as a first-class resolver. Its verbs
  (LOCATE, FIND, CHANGE, EXCLUDE, SORT, CAPS, SCROLL, ...) live as hand-written
  `if` arms in the shell ladder, specifically `try_commands_b2`
  (`crates/ff-desktop/src/shell/commands_ladder_b2.rs:19-...`), each delegating
  to a manager (`nav_manager`, `find_manager`, `exclude_manager`,
  `edit_profile`, `scroll_amount`). B080 Steps 3-6 CORRECTLY deferred these
  because they are NOT expressible as any `CommandTarget` variant -- they are not
  Menu / CustomWorkspace / Function / Macro / External; they act on the active
  editor buffer.
- The PROPOSAL: a `CommandEnvironment` is a named resolver+executor for a command
  vocabulary. The ACTIVE environment is derived from the active workspace
  Context (`self.tabs.active_tab().kind`, `tab_state.rs:18`). The front door
  consults the ACTIVE environment FIRST, then falls back to the SHELL
  environment (= `resolve_target`), then to the existing engine terminal. The
  editor verbs move OUT of `commands_ladder_b2.rs` INTO an EDITOR environment.
- ADDRESSING already has a home: the lua-macro-engine spec ALREADY specifies
  REXX `ADDRESS <environment-name>` and `TSO`/`ISPEXEC`/`ISREDIT` host command
  environments (lua-macro-engine `requirements.md` Req 11.11-11.14, with tasks
  21-22 marked `[x]`). CR-CH-053 should RECONCILE with that notion, not invent a
  second one: the macro `address(<env>)` binding routes through the existing
  Scripting_Bridge (command-framework Req 6), and the environment NAMES
  (SHELL/EDITOR/future DB) become the ADDRESS targets. An interactive address
  PREFIX is RECOMMENDED but kept minimal and optional.

The single collision to flag: lua-macro-engine Req 11.11 already maps `TSO` to
"the workbench TSO command dispatcher (ff-command)". That is EXACTLY the SHELL
environment under a different (mainframe) name. The owner must decide the
canonical name set and the alias map (TSO -> SHELL, ISREDIT -> EDITOR) so the two
specs do not define two parallel vocabularies. See Open Question Q6.

---

## 1. Problem statement -- why editor verbs do not fit the shell CommandTarget model

### 1.1 The five CommandTarget variants are shell/workbench actions

`CommandTarget` (`crates/ff-command/src/command_target.rs:112-170`) has exactly
five variants: `Menu`, `CustomWorkspace`, `Function`, `Macro`, `External`.
Every one describes a WORKBENCH-level action: open a menu, open a Context, invoke
a registered Command_ID, run a macro, run a process. `resolve_target`
(`command_target.rs:267-301`) classifies a bare string into one of these.

### 1.2 The editor verbs are none of those five

The editor-action verbs act on the ACTIVE editor buffer through a manager, not on
the workbench. Concretely, from `commands_ladder_b2.rs`:

- Navigation / scroll: `LOCATE` (line 21, `nav_manager.locate`), `TOP` (line 32),
  `BOTTOM` (line 38), `UP`/`DOWN` (lines 48/63, `nav_manager.up/down` or
  `up_by_amount`/`down_by_amount` against `self.scroll_amount`), `LEFT`/`RIGHT`
  (lines 78/85), `SORT` (line 92, `nav_manager.sort`).
- Exclude / show filter: `EXCLUDE ALL` / `X ALL` (line 104), `EXCLUDE <text> [ALL]`
  / `X <text> [ALL]` (line 115), `SHOW ALL` / `INCLUDE ALL` (line 139),
  `SHOW <text>` / `INCLUDE <text>` (line 150), `RESET [EXCLUDED|ALL]` (line 165)
  -- all `self.exclude_manager.*` with a line snapshot closure.
- Find: `RFIND` (line 183), `RCHANGE` (line 196), `FIND <term>` (line 209),
  `CHANGE 'old' 'new'` (line 220) -- all `self.find_manager.*`.
- Profile: `CAPS [ON|OFF]` (lines 240/245/296), `NULLS [ON|OFF]` (line 306),
  `STATS [ON|OFF]` (line 320), `LOCK [ON|OFF]` (line 334), `PROFILE [kw [val]]`
  (lines 349/356), `HILITE [mode]` (line 383) -- all
  `self.tabs.active_tab_mut().edit_profile.*`.
- Scroll field: `SCROLL <amt>` (line 404, `self.scroll_amount`).

(One mixed case: `THEME <name>` at line 262 is a theme-APPLY action that stays on
the ladder; bare `THEME` is already a SHELL CustomWorkspace, classified by
`builtin_workspace_target_for` -- see `command_config/mod.rs:168-174`. THEME is a
SHELL verb, not an editor verb; it is noted only because it shares the file.)

None of these can be a `Function` target without first registering each as a
Command_ID with a handler that reaches the manager, AND the `Function` dispatch
arm (`target_dispatch.rs` `dispatch_command_target`) calls `handle_command(id)`
with NO params -- the same reason `function_target_with_arg`
(`dispatch.rs:function_target_with_arg`) is dead-code-allowed and B080 Step 3 was
DEFERRED: a param-carrying Function dispatch does not exist yet, so `FIND Foo`
could not carry `Foo`. They are not Menu / CustomWorkspace (they open nothing),
not Macro, not External. So B080 correctly left them on the ladder.

### 1.3 Why this is the correct home for them

The owner's insight is exactly right: these verbs belong to a DIFFERENT command
environment -- the EDITOR environment -- the way ISREDIT edit-macro services
belong to the ISREDIT host command environment, distinct from TSO. They are only
meaningful when an editor Context is active, they share one backing surface (the
active `TabState` + its managers), and they have their own argument grammar
(quoted strings for CHANGE, `ALL` suffix for EXCLUDE, scroll amounts for
UP/DOWN). A per-environment resolver is the natural structure the ladder is
hand-rolling today.

---

## 2. The Command Environment model

### 2.1 What a Command_Environment IS (in Rust terms)

Reuse the shape that already works: the `TargetResolver` trait pattern. A
Command_Environment is a NAMED pair of (resolve, execute):

```
pub trait CommandEnvironment {
    /// Stable environment name used by ADDRESS and diagnostics
    /// (e.g. "SHELL", "EDITOR", later "DB"). Case-insensitive match.
    fn name(&self) -> &'static str;

    /// Try to CLAIM a raw command string for this environment. Returns
    /// Some(outcome) when this environment owns the verb and handled it;
    /// None to let the router fall through to the next environment.
    /// (Mirrors the ladder's `try_commands_*` -> bool contract, but typed.)
    fn dispatch(&mut self, raw: &str, cx: &mut EnvContext) -> Option<EnvOutcome>;
}
```

Two deliberate design choices, each grounded in existing code:

1. The SHELL environment is NOT a new type. It is the EXISTING
   `resolve_target(raw, &ShellTargetResolver)` ->
   `dispatch_command_target(target)` pair. The router treats "SHELL" as the base
   fallback and runs that exact path -- no rewrite, no second classifier. This is
   the single most important reconciliation point: SHELL == resolve_target.
   (`command_target.rs:267`, `target_dispatch.rs` dispatch_command_target.)

2. The EDITOR environment is a resolver+executor that OWNS the verbs now in
   `try_commands_b2`. Its `dispatch` is the SAME `verb -> manager` matching that
   lives there today, moved verbatim behind the trait. Because the editor verbs
   mutate shell-entangled state (`self.tabs`, `self.nav_manager`,
   `self.find_manager`, `self.exclude_manager`, `self.scroll_amount`), the
   EDITOR environment cannot be a free resolver like `ShellTargetResolver`; it is
   either (a) a thin struct holding `&mut`-borrowed manager refs for the frame,
   or (b) a set of methods ON `WorkbenchShell` behind an "editor env" marker that
   the router calls. Option (b) is RECOMMENDED first (least churn, mirrors how
   the Files Panel keeps its own `files_panel_cmd` redirect per wiring-standard),
   with (a) as the eventual clean form once the managers are grouped into a
   sub-struct (rust-standards god-struct guidance).

`EnvContext` / `EnvOutcome` are small shims so the environment can report the
same things the ladder arms report today: set `open_error`, set a status, and
indicate claimed-vs-fell-through. They are NOT a new dispatch result type for the
whole framework -- they are local to the environment router, converted into the
existing `open_error` / Command_Line_Outcome at the boundary (so Req 13.1 stays
the one decision point, as in `commands.rs:run_command_line`).

### 2.2 The Environment_Registry

A small ordered registry owned by the shell: the always-present SHELL base plus
zero or more context environments. Minimal shape:

```
struct EnvironmentRegistry {
    // SHELL is implicit/base, not stored as a trait object -- it is resolve_target.
    editor: EditorEnvironment,          // phase 1
    // future: database: DatabaseEnvironment, ... keyed by name
}
```

The registry's only jobs: (1) given the active Context, return the ACTIVE
environment name; (2) given a name (for ADDRESS), return that environment; (3)
expose SHELL as the base. It is NOT a second dispatcher -- it feeds the one front
door.

### 2.3 The ACTIVE environment (derived from the active Context)

The active environment is a pure function of `self.tabs.active_tab().kind`
(`tab_state.rs:18`, `TabKind`):

- `TabKind::FileEditor` and `TabKind::Untitled` -> ACTIVE = EDITOR.
- Every other `TabKind` (MenuWorkspace, ConfigPanel, FilesPanel,
  FileExplorerPanel, SearchResults, PluginManager, EventLog, MacroLibrary,
  CommandConfigurator, ThemeEditor, MenusEditor, KeysEditor, KindsEditor, ...)
  -> ACTIVE = SHELL (there is no bespoke environment for them yet; SHELL owns
  their Context-opener verbs via `resolve_target`).
- Future: a Database Context -> ACTIVE = DB (registers its own environment).

This mapping is the ONLY new piece of "which environment is active" logic, and it
mirrors the existing `WorkspaceContext` Kind registry mapping (one Context kind
-> one behaviour set).

### 2.4 SHELL as the always-present fallback base

SHELL is never "the active environment that can be absent". It is the base every
command can fall back to. This matches REXX: the initial/host environment is
always reachable. Concretely: a FIND typed while a menu is active falls through
the (SHELL-active) front door to the engine terminal exactly as today; a FILES
typed while the editor is active is NOT claimed by the EDITOR environment, so the
router falls back to SHELL (= `resolve_target`) which classifies FILES to
`CustomWorkspace("file_explorer")`. This is the ISPF "command relayed to the base
environment" behaviour.

---

## 3. The router -- one front door generalised to N environments

### 3.1 Today's front door (B080, exact)

`dispatch_command_string` (`crates/ff-desktop/src/shell/dispatch.rs:47-82`) runs,
in order:

1. `run_command_prelude(raw, &upper)` -- history record, stage-1 current-menu
   Option_Key (`try_current_menu_option`), the EXIT family (`try_exit_family`),
   the POM / chained fastpaths (`commands.rs:run_command_prelude`,
   `commands.rs:96-176`).
2. `resolve_target(raw, &ShellTargetResolver)` -> `dispatch_command_target` on
   `Ok` (`dispatch.rs:70-74`). THIS is the SHELL environment.
3. `run_command_ladder(raw, &upper)` -- the stage-2+ ladder, which TODAY still
   contains `try_commands_b2` (the editor verbs) before the engine terminal
   (`commands.rs:run_command_ladder`, `commands.rs:178-...`).

### 3.2 Proposed front door (minimal insertion, same single path)

Insert ONE active-environment step BETWEEN the prelude and `resolve_target`:

```
dispatch_command_string(raw):
    upper = raw.trim().to_uppercase()
    if run_command_prelude(raw, upper): return          // UNCHANGED (stage 1, EXIT, POM, chained)
    if let Some(env, rest) = parse_address_prefix(raw):  // NEW, optional (section 4)
        return dispatch_to_named_env(env, rest)          //   explicit ADDRESS wins
    if active_env_is_editor():                           // NEW: derived from active Tab kind
        if editor_env.dispatch(raw): return              //   ACTIVE environment first
    if let Ok(t) = resolve_target(raw, &ShellTargetResolver): // SHELL environment (UNCHANGED)
        dispatch_command_target(t); return
    run_command_ladder(raw, upper)                       // transitional fallback -> engine terminal
```

Key properties, each a reconciliation:

- There is still ONE front door. `active_env.dispatch` is NOT a second
  dispatcher; it is a resolver stage inserted into the SAME ordered chain the
  front door already owns, exactly like `builtin_workspace_target` was inserted
  in B080 Step 2.
- SHELL is still `resolve_target`. The step is literally unchanged
  (`dispatch.rs:70-74`). The generalisation is "consult the ACTIVE env, then
  consult SHELL (= resolve_target)".
- The prelude keeps its precedence. Stage-1 current-menu Option_Key, the EXIT
  family, and the POM/chained fastpaths run BEFORE any environment, as today
  (`run_command_prelude`). This is intentional: those are workbench-global and
  must not be shadowable by an environment verb (Req 8.3 stage 1 is first).
- The ladder is the transitional fallback. Until the editor verbs are fully
  migrated into the EDITOR environment, `run_command_ladder` (containing
  `try_commands_b2`) stays as the final fallback before the engine terminal.
  Each migration step deletes the arm from `try_commands_b2` once the EDITOR
  environment claims it -- the same per-step rollback discipline as B080 (keep
  both reachable; delete the ladder arm once the env entry is test-proven).

### 3.3 Precisely what changes vs today

- TODAY: prelude -> `resolve_target` (SHELL) -> ladder (which still holds editor
  verbs in `try_commands_b2`) -> engine terminal.
- PROPOSED: prelude -> [optional explicit ADDRESS] -> ACTIVE env (EDITOR when an
  editor Context is active) -> `resolve_target` (SHELL) -> ladder (shrinking as
  editor verbs migrate out) -> engine terminal.

The ONLY new ordering decision is "ACTIVE env before SHELL". That is the ISPF/
REXX rule (the addressed/active environment gets first crack; the base is the
fallback) and it is the `active-wins` shadowing rule the owner described
("already in that environment = address redundant"). See Q4 for the precedence
nuance when a verb name exists in BOTH.

---

## 4. Addressing -- the explicit ADDRESS mechanism

### 4.1 Macros (primary, already specified) -- reconcile, do not reinvent

The lua-macro-engine spec ALREADY defines this surface:

- Req 11.11: a `TSO` host command environment routes unrecognised commands to
  "the workbench TSO command dispatcher (ff-command)" and returns `RC`.
- Req 11.12: `ADDRESS <environment-name>` switches the default host command
  environment for subsequent commands in the exec.
- Req 11.13 / 11.14: `ISPEXEC` and `ISREDIT` environment names within ADDRESS.
- Tasks 21-22 (`lua-macro-engine/tasks.md`) are marked `[x]` for ISREDIT/ISPEXEC
  host environments and `ADDRESS <environment-name>` / `TSO` routing.

CR-CH-053's macro story is therefore RECONCILIATION, not new invention: the Lua
`address(<env>) "cmd"` / `ADDRESS <env>` binding sends `cmd` into the named FFWB
Command_Environment through the Scripting_Bridge (command-framework Req 6.1,
`workbench.execute(...)`, `crates/ff-command/src/scripting.rs:66-96`). The
mapping to propose:

- `ADDRESS TSO`  -> the SHELL environment (TSO already means "ff-command
  dispatcher" per Req 11.11 -- SHELL is that dispatcher's front door).
- `ADDRESS ISREDIT` -> the EDITOR environment (ISREDIT already means edit-macro
  services per Req 11.14 -- EDITOR is the FFWB-native name for the same thing).
- `ADDRESS SHELL` / `ADDRESS EDITOR` -> the same two, under the FFWB-native
  names, so a non-mainframe macro author uses the obvious names.

This gives ONE environment set with a documented alias map (TSO=SHELL,
ISREDIT=EDITOR), rather than two competing vocabularies. The owner must confirm
the canonical names and the alias direction (Q6).

### 4.2 Interactive address prefix (recommended, minimal, in scope)

RECOMMEND a minimal, optional interactive prefix so a user at the command line
can force an environment regardless of active Context, mirroring ADDRESS:

- Syntax: a leading environment token followed by a colon or whitespace, e.g.
  `EDITOR: FIND Foo` or `SHELL FILES`. Keep it conservative: only recognised
  environment NAMES (SHELL/EDITOR/aliases) are treated as a prefix; anything else
  is an ordinary verb (so `FIND` is never mis-read as an environment).
- Parsed at `parse_address_prefix` in the front door (section 3.2), BEFORE the
  active-env step. An explicit prefix picks the named environment directly.
- RECOMMENDATION: keep syntax to ONE form to avoid ambiguity; defer the exact
  separator to the owner (Q1). This is the only genuinely NEW user-facing syntax
  and is deliberately small.

### 4.3 "Already in that environment -> address redundant"

Directly implementable and should be a stated criterion: WHEN the addressed
environment EQUALS the active environment, the address prefix is a no-op wrapper
-- the command runs identically with or without it. This is automatic in the
router: an explicit `EDITOR: FIND Foo` while the editor is active resolves the
same as `FIND Foo` (both hit the EDITOR environment first). No special code; it
is a consequence of active-env-first + explicit-env-pick converging on the same
environment.

---

## 5. Migration path -- editor verbs out of the ladder, behaviour-preserving

Each step is scoped-test-green (`cargo test -p ff-desktop`), per B080's
per-family discipline. The EDITOR environment is introduced empty, then
populated family-by-family; each family's arms are deleted from
`commands_ladder_b2.rs` only once the environment claims them with a passing
test.

- Step E0 -- scaffold: add `CommandEnvironment` trait, the EDITOR environment
  (empty `dispatch` returning `None`), the Environment_Registry, the active-env
  derivation from `TabKind`, and the front-door insertion (section 3.2). With an
  empty EDITOR dispatch and no address prefix, behaviour is unchanged (pure
  indirection). Prove no shell test regresses.
- Step E1 -- navigation family: move LOCATE / TOP / BOTTOM / UP / DOWN / LEFT /
  RIGHT / SORT (`commands_ladder_b2.rs:21-101`) into the EDITOR environment,
  delegating to `nav_manager` exactly as now (including `up_by_amount` /
  `down_by_amount` against `self.scroll_amount` for bare UP/DOWN, CR-NR-087).
  Delete those arms from `try_commands_b2`.
- Step E2 -- exclude/show family: EXCLUDE/X [ALL], SHOW/INCLUDE [ALL], RESET
  variants (`commands_ladder_b2.rs:104-181`), delegating to `exclude_manager`
  with the same snapshot closure. The ALL-suffix / text sub-parse moves inside
  the one env entry (removes the D9-style ordering hazard).
- Step E3 -- find family: FIND / RFIND / CHANGE / RCHANGE
  (`commands_ladder_b2.rs:183-237`), delegating to `find_manager`.
  BEHAVIOUR-PRESERVING argument handling: CHANGE's `parse_two_args` quoting and
  the case-preserved search term (B062 / Req 8.4) are carried verbatim; the verb
  token is matched case-insensitively, the Argument_String keeps its case
  (`split_verb_arg` rule, `dispatch.rs:verb_arg` tests).
- Step E4 -- profile family: CAPS / NULLS / STATS / LOCK / PROFILE / HILITE
  (`commands_ladder_b2.rs:240-402`), delegating to `edit_profile`.
- Step E5 -- scroll: SCROLL <amt> (`commands_ladder_b2.rs:404-...`) into the
  EDITOR environment (`scroll_amount`).
- Step E6 -- prefix-area line commands (C/CC/M/D/DD/...): these are entered in the
  prefix area, not the command line, so they are collected via Session_State and
  run through the ff-command-semantics `Command_Engine`
  (`command-semantics/requirements.md` Req 1.1-1.2, Req 2 scope). RECOMMEND the
  EDITOR environment OWNS them conceptually (they are editor commands) but their
  INTAKE path (prefix area) is separate from the command-line router. Flag as
  Q2: are line commands part of the EDITOR environment or a sibling PREFIX
  environment? See Open Questions.
- Step E7 -- after the families are migrated, `try_commands_b2` is empty and is
  deleted; `run_command_ladder` loses that call. This composes with (does not
  block) B080 Step 7 (deleting the Step-2 superseded SHELL arms).

SHELL keeps its Context-opener verbs unchanged throughout: they already resolve
via `resolve_target` / `builtin_workspace_target_for` (`command_config/mod.rs`),
which this proposal does not touch.

A FUTURE Context (database tool) registers its own environment by: adding its
`TabKind`, adding an arm to the active-env derivation (its kind -> DB), and adding
a `DatabaseEnvironment: CommandEnvironment` to the registry. No front-door change
-- this is the template the proposal establishes. (database-tool is noted as a
future CONSUMER only; it is NOT designed here.)

---

## 6. Reconciliation and non-goals

Explicit non-goals (framework-conformance):

- NO second navigation stack. The per-tab Navigation_Stack (command-framework
  Req 10, `nav_stack.rs`) is untouched. Environments route COMMANDS; they do not
  navigate.
- NO parallel dispatcher. The ACTIVE-env step is inserted into the ONE front door
  `dispatch_command_string`, between the existing prelude and the existing
  `resolve_target` step. There is exactly one path.
- NO change to `CommandTarget`. The five variants are unchanged. The editor verbs
  do NOT become new `CommandTarget` variants; they live in the EDITOR
  environment, which is why they could not be migrated in B080.
- Does NOT implement CR-CH-052 (`=` ladder-drop, X / =X semantics). That is
  independent and composes on top of the same front door (the `=` prefix is
  handled in the prelude's fastpaths; environments are orthogonal to it).
- Does NOT design the database tool.

What REMAINS on the shared ladder as a transitional fallback: `run_command_ladder`
(and within it the not-yet-migrated editor arms in `try_commands_b2`, plus the
other `try_commands_a/b1/c` families B080 is separately retiring) stays as the
final fallback before the engine terminal, shrinking per migration step. Relation
to B080 Step 7: B080 Step 7 deletes the SHELL Step-2 superseded arms; CR-CH-053
Steps E1-E7 delete the EDITOR arms. They are independent deletions of different
arm families and can proceed in either order.

Reconciliation with the engine terminal: `cmd_engine.execute_command_line(cmd)`
(`commands.rs:run_command_ladder` terminal) remains the LAST resort. Note the
ff-command-semantics `Command_Engine` (`command-semantics/requirements.md` Req 1,
Primary_Command pipeline) is itself effectively the editor-command engine for the
prefix-area/line-command path; the EDITOR environment and the Command_Engine
should be reconciled so they are not two editor-command executors. RECOMMEND the
EDITOR environment DELEGATES primary editor verbs to the same managers the engine
uses (as the ladder does today), and the owner decides at the gate whether the
EDITOR environment eventually subsumes the Command_Engine's command-line role or
sits in front of it (Q5).

---

## 7. Open questions for the owner

- Q1 (interactive address syntax): what prefix form? Options: `ENV: cmd`
  (colon), `ADDRESS ENV cmd` (REXX-literal), or `ENV cmd` (bare). RECOMMEND
  `ENV: cmd` for an unambiguous separator that cannot collide with a two-word
  verb. Confirm, or decide to omit the interactive prefix entirely in phase 1
  (macros-only addressing).
- Q2 (line commands): are prefix-area line commands (C/CC/M/D/DD/RR/XX/TT, etc.)
  part of the EDITOR environment or a separate PREFIX environment? They enter via
  the prefix area (Session_State), not the command line, and are already handled
  by the Command_Engine scope algorithm (`command-semantics` Req 2.1). RECOMMEND
  conceptually-EDITOR but intake-separate (do NOT route them through the
  command-line front door); confirm.
- Q3 (split regions / detached windows): which environment is active when the
  focus is a split region or a Detached Workspace? RECOMMEND the ACTIVE
  environment follows the FOCUSED Context (`focused_group` ->
  `active_tab().kind`, `tab_manager.rs:121`), so a detached editor's command line
  is EDITOR-active and a detached menu is SHELL-active. Confirm the focus source
  of truth for detached windows (`update_keys.rs:83` path).
- Q4 (precedence when a verb exists in BOTH active and shell environments):
  RECOMMEND active-wins (the ISPF/REXX rule, mirroring the Req 8.10 shadowing
  rule's "earlier stage wins"), with a documented shadowing note: when the EDITOR
  environment and SHELL both define a verb name, the ACTIVE environment claims it;
  the user reaches the shell one via an explicit `SHELL:` address. Confirm, and
  confirm whether any current SHELL verb name collides with an editor verb (none
  found in this pass: the `X`/`=X`/EXIT family is claimed by the PRELUDE's
  `try_exit_family` BEFORE either environment, so EXCLUDE's `X` alias in the
  editor env does not conflict with the shell EXIT `X` -- but this is exactly the
  kind of collision the owner should ratify).
- Q5 (EDITOR environment vs Command_Engine): does the EDITOR environment delegate
  to managers (as the ladder does) and leave the ff-command-semantics
  Command_Engine owning the prefix-area/scope pipeline, or does it eventually
  front the Command_Engine for primary editor commands too? RECOMMEND phase 1 =
  delegate to managers (behaviour-preserving); defer any Command_Engine
  consolidation.
- Q6 (canonical environment names + alias map): confirm SHELL and EDITOR as the
  FFWB-native names and the alias map to the already-specified mainframe names
  (TSO -> SHELL per lua-macro-engine Req 11.11; ISREDIT -> EDITOR per Req 11.14;
  ISPEXEC maps to the ISPF dialog service layer per Req 11.13 -- is ISPEXEC a
  THIRD environment or a SHELL sub-service?). This is the one place CR-CH-053 and
  lua-macro-engine Req 11 MUST be reconciled to avoid two parallel vocabularies.
- Q7 (RC / return code): REXX ADDRESS sets `RC` (lua-macro-engine Req 11.15).
  Should the FFWB Command_Environment `dispatch` return a structured outcome that
  the Scripting_Bridge maps to `RC`, so a macro's `address(EDITOR) "FIND Foo"`
  gets a found/not-found code? RECOMMEND yes (fold into `EnvOutcome`); confirm the
  code convention.

---

## 8. Proposed EARS acceptance criteria (DRAFT, for the gate)

Numbering is illustrative; the gate assigns final numbers in the chosen
sub-project. Each is phrased to BUILD ON the existing framework.

Environment model and router:

- E.1 THE framework SHALL define a Command_Environment as a named resolver that
  may CLAIM a submitted command string for a command vocabulary, where the SHELL
  environment is the existing `resolve_target` / CommandTarget chain and is the
  always-present base.
- E.2 THE framework SHALL maintain an Environment_Registry containing the SHELL
  base and zero or more Context environments, and SHALL derive the ACTIVE
  environment from the active Workspace Context.
- E.3 WHEN a command string is submitted, THE single front door
  (`dispatch_command_string`) SHALL, AFTER the existing prelude (current-menu
  Option_Key, EXIT family, POM/chained fastpaths), consult the ACTIVE
  environment FIRST, THEN fall back to the SHELL environment
  (`resolve_target`), preserving the existing ordered chain and adding no second
  dispatcher.
- E.4 THE introduction of Command_Environments SHALL NOT change the observable
  result of any command string that resolves today (backward compatibility,
  mirroring command-framework Req 8.4).
- E.5 WHEN a verb name exists in BOTH the active environment and the SHELL
  environment, THE active environment SHALL claim it (active-wins shadowing),
  and the SHELL instance SHALL be reachable only via an explicit address
  (mirrors Req 8.10 earlier-stage-wins).

Editor environment:

- E.6 THE framework SHALL provide an EDITOR Command_Environment that owns the
  editor-action verbs (LOCATE, TOP, BOTTOM, UP, DOWN, LEFT, RIGHT, SORT,
  EXCLUDE/X, SHOW/INCLUDE, RESET, FIND, RFIND, CHANGE, RCHANGE, CAPS, NULLS,
  STATS, LOCK, PROFILE, HILITE, SCROLL) and SHALL execute them against the active
  editor through the existing managers, with identical observable results to the
  current shell ladder (including case-preserved Argument_Strings per B062).
- E.7 WHEN the active Context is an editor Context (TabKind FileEditor or
  Untitled), THE ACTIVE environment SHALL be EDITOR; otherwise it SHALL be SHELL.

Addressing:

- E.8 THE framework SHALL support an explicit address that routes a command to a
  named Command_Environment regardless of the active Context.
- E.9 WHEN a macro addresses an environment (Lua `address(<env>)` / REXX
  `ADDRESS <env>`), THE command SHALL be routed to that FFWB Command_Environment
  through the Scripting_Bridge, reconciled with the already-specified
  TSO/ISPEXEC/ISREDIT host command environments (lua-macro-engine Req 11.11-11.14)
  via a documented alias map (TSO=SHELL, ISREDIT=EDITOR).
- E.10 WHEN the addressed environment equals the active environment, THE address
  SHALL be redundant: the command SHALL execute identically with or without it.
- E.11 (IF the interactive prefix is approved, Q1) WHEN a command line begins with
  a recognised environment name followed by the chosen separator, THE front door
  SHALL route the remainder to that named environment; an unrecognised leading
  token SHALL be treated as an ordinary verb.

Future-context template:

- E.12 A new Workspace Context MAY register its own Command_Environment by mapping
  its Context kind to that environment in the Environment_Registry, with NO change
  to the single front door.

### Which spec sub-project do these belong in?

RECOMMENDATION: create a NEW sub-project `docs/specs/command-environments/`
(already anticipated in the CR-CH-053 change-log "Affects" line and the specs.md
sub-project list has room for it). Rationale: the environment model + router is a
framework CONCERN that spans command-framework (dispatch), command-semantics
(editor verb vocabulary), and lua-macro-engine (ADDRESS) -- giving it its own
folder keeps the cross-cutting model in one authoritative place and lets each
existing sub-project reference it, rather than smearing the model across three
requirements files. Specifically:

- `docs/specs/command-environments/requirements.md` (NEW): E.1-E.5, E.8, E.10-E.12
  (the model, router, addressing, template) -- the authoritative home.
- `docs/specs/command-framework/`: a SMALL delta noting the front door now
  consults the active environment before `resolve_target` (reconciles Req 2.1 /
  8.3), cross-referencing command-environments. No change to Req 8's CommandTarget.
- `docs/specs/command-semantics/`: E.6-E.7 (the EDITOR vocabulary + manager
  delegation) as a cross-reference, since those verbs are the ISPF primary
  commands this sub-project already governs.
- `docs/specs/lua-macro-engine/`: E.9 as a reconciliation note on Req 11.11-11.14
  (the alias map), NOT a new parallel ADDRESS mechanism.

---

## 9. Evidence index (file:symbol citations used above)

- `crates/ff-desktop/src/shell/dispatch.rs:47-82` -- `dispatch_command_string`,
  the single front door (prelude -> resolve_target -> ladder).
- `crates/ff-desktop/src/shell/commands.rs` -- `run_command_line`,
  `run_command_prelude`, `run_command_ladder`, `handle_command` (prelude +
  ladder definition; `try_commands_b2` call site in `run_command_ladder`).
- `crates/ff-desktop/src/shell/commands_ladder_b2.rs:19-...` --
  `try_commands_b2`, the editor-action verbs and their manager delegations
  (nav/exclude/find/profile/scroll).
- `crates/ff-desktop/src/command_config/mod.rs:130-210` --
  `builtin_workspace_target_for` + `ShellTargetResolver` (the SHELL environment's
  resolver today).
- `crates/ff-command/src/command_target.rs:112-301` -- `CommandTarget` enum,
  `TargetResolver` trait, `resolve_target` ordered chain.
- `crates/ff-desktop/src/shell/target_dispatch.rs` --
  `resolve_and_dispatch_command`, `dispatch_bound_command`,
  `dispatch_key_command`, `dispatch_command_target` (the key/menu seam + variant
  routing).
- `crates/ff-command/src/scripting.rs:66-96` -- `ScriptingBridge::execute`
  (command-framework Req 6, the macro -> command path).
- `crates/ff-command/src/dispatch.rs:80-...` -- `CommandDispatch::execute_command`
  (the `(id, params)` entry; no `arg` folding -- the Req 9.2 gap B080 Step 3
  deferred on).
- `crates/ff-desktop/src/tab_state.rs:18-...` -- `TabKind` (FileEditor / Untitled
  = editor Contexts; all others = SHELL-active).
- `crates/ff-desktop/src/tab_manager.rs:117-121` -- `focused_group` /
  `active_tab()` focus resolution (Q3 active-env-follows-focus).
- `docs/specs/command-framework/requirements.md` -- Req 2 (single dispatch), Req 5
  (Shortcut_Registry), Req 6 (Scripting_Bridge, `workbench.execute`), Req 8
  (CommandTarget + ordered Target_Resolution chain, 8.3/8.4/8.10 shadowing),
  Req 9 (one verb/arg split at the boundary), Req 10 (per-tab Navigation_Stack).
- `docs/specs/command-semantics/requirements.md` -- Req 1 (Command_Engine
  pipeline), Req 2 (scope resolution for line commands), Req 3 (Primary_Command
  parser).
- `docs/specs/lua-macro-engine/requirements.md` -- Req 11.11-11.15
  (TSO/ISPEXEC/ISREDIT host command environments, `ADDRESS <env>`, `RC`); tasks
  21-22 marked `[x]`.
- `docs/status/change-log.md` -- CR-CH-053 (this change; owner framing), CR-CH-052
  (independent, composes).
- `.agents/tasks/dispatch-unify/design-delta.md`, `divergences.md` -- B080
  front-door design and the enumerated seam divergences; confirms the editor
  verbs stayed on the ladder and the single-front-door plumbing is in place.
