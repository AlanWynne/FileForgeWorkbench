# Design Document -- Command Configurator

## 1. Overview

The Command Configurator adds a user-authored command layer on top of the
unified Command_Target (command-framework Requirement 8). It has three parts:

1. A **Command_Store** -- a hot-reloadable TOML file of named Command_Definitions.
2. A **Command Configurator Context** -- a Custom Workspace that edits the store.
3. An **External execution path** with two modes (Detached, Captured) that
   reuses the `ff-shell` engine and Output_Panel.

No new library crate is required. The store loader and the Context live in the
`ff-desktop` binary layer alongside `menu_workspace/`; external execution reuses
`ff-shell`. The Command_Target type itself is owned by `ff-command`
(command-framework), so this sub-project stores and dispatches targets but does
not redefine them.

---

## 2. Architectural Position

```
ff-desktop/src/
  command_config/
    mod.rs        -- CommandDefinition, CommandStore, public API, re-exports
    store.rs      -- commands.toml load/save, validation, hot-reload poll
    render.rs     -- egui rendering of the Command Configurator Context
    edit.rs       -- add/edit form state and the variant-specific target editor
```

- `CommandTarget` (from `ff-command`) is serialised inside each definition.
- External execution calls into `ff-shell` (Requirement 19 of shell-command),
  which already owns process spawning, async capture, timeout, and the
  Output_Panel.
- Hot-reload reuses the `MenuWorkspaceState::poll_reload` pattern (mtime check
  per frame), not a new file watcher.

---

## 3. Data Model

```rust
/// One user-authored command.
/// Validates: command-configurator Requirement 1.2, 1.3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_category")]
    pub category: String,
    pub target: ff_command::CommandTarget,   // [command.target]
}

fn default_category() -> String { "user".to_string() }

/// The loaded store plus load diagnostics.
/// Validates: command-configurator Requirement 1.1, 1.6, 1.7
pub struct CommandStore {
    pub file_path: PathBuf,
    pub definitions: Vec<CommandDefinition>,   // valid entries only
    pub load_error: Option<String>,
    pub last_modified: Option<SystemTime>,
}
```

TOML shape:

```toml
[[command]]
id = "build.release"
label = "Build Release"
description = "Build the workspace in Release mode"
category = "build"
[command.target]
kind = "external"
program = "pwsh"
args = ["-File", "scripts/build.ps1", "-Config", "Release"]
working_dir = "${workspace_root}"
mode = "captured"

[[command]]
id = "open.notes"
label = "Open Notes Menu"
[command.target]
kind = "menu"
name = "notes"
```

Store save writes the full definition list back (Requirement 2.4/2.5),
preserving order. Load skips invalid entries and records `load_error`
(Requirement 1.6); duplicate ids keep the first (Requirement 1.4).

---

## 4. Target Resolution Integration

`command_config::store` exposes the loaded definitions to
`ff_command::resolve_target` via a `UserCommandStore` view (command-framework
design, Target resolution step 1). When the user types or a menu option carries
a bare id equal to a definition's `id`, resolution returns that definition's
stored `CommandTarget`. Reserved built-in ids cannot be shadowed
(Requirement 4.6); the loader checks each definition id against the
`CommandRegistry` reserved set and rejects conflicts.

---

## 5. External Execution (Requirement 3)

External targets are dispatched by `execute_target` (command-framework) to a
thin adapter that calls `ff-shell`:

- **Detached** -> `ff_shell::spawn_detached(program, args, working_dir)`:
  spawns the child, drops the handle, returns immediately. No capture, no tab.
  A `TaskHandle` is defined by `ff-shell` and returned, but the desktop layer
  drops it -- this is the named seam for possible future Started_Task
  monitoring, which is explicitly out of scope now.
- **Captured** -> the existing `shell.execute` async path (shell-command
  Requirement 4, 13, 15): run async, stream stdout/stderr to the Output_Panel
  with a header and exit code. The only difference from a typed `SHELL` string
  is that program+args are explicit rather than parsed from a shell line.

Placeholder expansion (`${workspace_root}`, `${file_dir}`) happens in the
adapter before spawn (Requirement 3.6). The `shell.mode` gate and the
prompt-confirm dialog are applied by the shell engine, unchanged
(Requirement 3.7, 3.8; shell-command Requirement 2).

### Why reuse ff-shell rather than a new engine

`ff-shell` already implements platform shell detection, async execution,
timeout, cancellation, ConPTY/PTY, and the Output_Panel. Captured mode is a
strict subset of `shell.execute`. Detached mode adds only a spawn-and-drop
variant. This avoids a parallel process-management implementation and a second
security switch.

---

## 6. Command Configurator Context (Requirement 2)

A new `WorkspaceKind::CommandConfigurator` (session-layer) / runtime
`TabKind::CommandConfigurator` renders the store as a table: id, label, variant,
and (for external) mode. Add/Edit opens `edit.rs` form state with a
variant-selector; the form shows only the fields for the chosen variant
(Requirement 2.6). Save validates then writes the store. The Context title is
`[COMMANDS]`; F3/END returns to the POM (Requirement 2.7, 2.8), matching other
Custom Workspaces.

Opening command: `COMMANDS` (primary command), and the Context is itself a
`CustomWorkspace { workspace_kind: CommandConfigurator }` target, so it is
persisted by descriptor (startup-and-session Requirement 21) like any other
visible Workspace.

---

## 7. Session Persistence

The Command Configurator Context persists as
`CustomWorkspace { workspace_kind: CommandConfigurator }` (no params needed).
Started_Tasks (Detached runs) and Captured_Run Output_Panel results are NOT
persisted (command-framework Requirement 8.9; startup-and-session
Requirement 21). The `commands.toml` store is user data on disk and is loaded at
startup independent of session restore.

---

## 8. Security

External execution inherits shell-command Requirement 2 entirely:
`shell.mode = disabled` refuses all external runs; `prompt` confirms first;
`enabled` runs without prompting. There is no Command-Configurator-specific
security setting. When an External target is invoked from a macro, the combined
macro-security + shell.mode rule (shell-command Requirement 2.7) applies.

---

## 9. Alias Editor Context (Requirement 5, CR-NR-113)

The Alias Editor is a new Custom Workspace Context for creating, editing, and
saving command aliases -- including the parameterised aliases defined by
CR-NR-112. It is modelled on the Menus Editor Context
(`shell/menus_editor.rs` + `menus_editor_panel`), which already solves the same
problem for menu TOML: list definitions, hold a working copy, edit rows, validate
with the loader's own discipline, serialise, write, refresh. The technology stack
is fixed: Rust in `ff-desktop`, serde + `toml`, egui + the existing
`WorkspaceContext` framework; no new crate and no new dependency.

"Modelled on the Menus Editor" means the SAME SHAPE, not a smaller set of wiring
points: the Alias Editor needs every structural piece a sibling editor has (a new
runtime `TabKind` + `KindTag`, a `BuiltinKind` mapping, the session non-persist
arms, a tab constructor, a dedicated central-panel render arm, and a shell panel
field), enumerated in Section 9.6. It is NOT only a descriptor arm.

### 9.0 Build-order prerequisite -- HARD dependency on CR-NR-112

CR-NR-113 has a HARD build-order dependency on CR-NR-112. A repo-wide search
confirms `command_template` exists NOWHERE in live source today: `AliasRowToml`
(`shell/alias_reader.rs`) carries only `surface`, `canonical`,
`localisation_key`, and the pre-pass `rewrite_verb_to_root` does verb rewrite +
raw-arg re-append with NO `&1..&N` / `&*` substitution. CR-NR-112 is itself only
gate-authored (pending design review), so the field and `expand_template` are not
yet implementable dependencies.

Therefore the template-dependent parts of this design -- Requirement 5.1's
`command_template` field, 5.3's template edit, 5.8's expansion, and the
`expand_template` reference in 9.7 -- MUST NOT be implemented until CR-NR-112 has
landed `AliasEntry.command_template`, the `AliasRowToml.command_template` field,
and the `&1..&N` / `&*` substitution in the pre-pass. The NON-template subset
(surface/canonical authoring, environment selection, save, load, in-place
refresh, the focus contract, the scope guard) has no such dependency and may be
built first. Task 7 gates the template-dependent subtasks behind a CR-NR-112-landed
checkpoint (tasks.md Task 7 preamble + 7.2 / 7.6.2).

### 9.1 Why a new store, and where it lives

Alias loading today is read-only and lives in the LOCALISATION layer
(`shell/alias_reader.rs` -> `LocaleAliasStore`), reading per-locale DATA from the
shipped `i18n/<locale>/aliases/<env>.toml` tree. Two reasons that tree is the
WRONG home for user-authored aliases: (1) it is shipped data under the resolved
`i18n/` root, which may be a read-only install location; (2) it is per-LOCALE and
entangled with CR-NR-109/111 localisation intent, whereas the owner's `ecob` is a
language-INDEPENDENT shorthand with no locale.

DECISION (owner-confirmed): the Alias Editor reads and writes a NEW User_Alias_Store
-- a single locale-independent TOML file under the workbench user data directory,
the same base the macros directory uses (`dirs::data_dir()/FileForgeWorkbench/`,
see `shell/actions.rs`). Concretely `<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`,
holding `[[alias]]` rows with the four CR-NR-112 fields (`surface`, `canonical`,
optional `localisation_key`, optional `command_template`). This mirrors the
Command_Store's own `<User_Data_Dir>/commands/commands.toml` placement (Section 1),
so the two user-authored stores sit side by side under the same data root.

Alternatives considered and rejected:
- Writing into `i18n/<locale>/aliases/`: rejected -- shipped/possibly-read-only,
  per-locale, collides with CR-NR-111.
- A workspace-local `aliases/` dir: rejected for v1 -- user aliases are a
  per-user preference, not a per-project artifact; the user data dir matches the
  macros/commands precedent and is always writable.

### 9.2 Data model (modelled on MenusEditorAction)

```rust
// Validates: command-configurator Requirement 5.1, 5.3
/// One editable alias row in the Alias Editor working copy.
pub struct AliasRowEdit {
    pub surface: String,              // the alias name, e.g. "ECOB"
    pub canonical: String,            // the root command, e.g. "EDIT"
    pub environment: AliasEnvironment,// ffcmd | ffedit, default ffcmd (Req 5.1)
    pub command_template: String,     // "" = pure verb alias (CR-NR-112 14.6)
                                      // NOTE: template field + its editing/
                                      // expansion are CR-NR-112-gated (9.0).
    // localisation_key is intentionally omitted from the user editor: a user
    // shorthand is language-independent (left blank on save).
}

/// Which Command Environment's AliasTable a user row loads into (Req 5.1, 5.7).
/// Defaults to FFCMD (the workbench shorthand case); FFEDIT is chosen for an
/// editor-verb root such as EDIT (ECOB -> EDIT is an FFEDIT alias).
pub enum AliasEnvironment { Ffcmd, Ffedit }

impl Default for AliasEnvironment { fn default() -> Self { Self::Ffcmd } }

/// The Alias Editor UI state (held in a grouped shell sub-struct, not a flat
/// WorkbenchShell field). `first_interior_id` captures the first control's
/// stable egui::Id each frame for the focus contract.
pub struct AliasEditorState {
    pub rows: Vec<AliasRowEdit>,      // the working copy
    pub error: Option<String>,
    pub pending_action: AliasEditorAction,
    pub first_interior_id: Option<egui::Id>,
}

/// Structural actions the pure render returns; per-field edits are applied
/// directly on the mutable working copy by the render (mirroring B054 in the
/// Menus Editor), so only structural changes surface here.
/// Validates: command-configurator Requirement 5.3, 5.4, 5.5
pub enum AliasEditorAction {
    None,
    AddRow,
    DeleteRow(usize),
    Save,
}
```

### 9.3 The Context lifecycle (copying the Menus Editor)

- `open_alias_editor` -- load the User_Alias_Store into `rows`, then `navigate_to`
  a `CustomWorkspace { CommandConfigurator, params: { editor = "aliases" } }`
  descriptor IN PLACE (push onto the per-tab Navigation_Stack), so END/RETURN
  returns to the opener. Mirrors `open_menus_editor` exactly.
- The pure render (`alias_editor_panel::render`) draws the row list + per-field
  editors and returns an `AliasEditorAction`; the `WorkspaceContext::render` impl
  stashes it on `pending_action` and returns `InteriorFocus` (first = the first
  row's surface field or the Add button when empty, captured on
  `first_interior_id`). Dispatched via the owned-panel swap
  (`mem::take` -> `render_workspace_context` -> put back -> apply action). No
  hand-written focus latch (workspace-conformance.md).
- `apply_alias_editor_action` -- the shell-side effect handler: `AddRow` pushes a
  blank `AliasRowEdit`; `DeleteRow(i)` removes row i; `Save` validates then
  writes (9.4).

### 9.4 Save: validate -> serialise -> write -> refresh

```rust
// Validates: command-configurator Requirement 5.4, 5.6, 5.7
fn save_user_aliases(&mut self) {
    // 1. Validate (reusing the overlay's collision discipline + the NEW
    //    no-shadow pre-pass invariant)
    //    - reject empty surface / empty canonical
    //    - reject a surface mapped to two different canonicals WITHIN THE SAME
    //      target environment in the working set (the same check
    //      AliasCatalogue::load_into_keyed enforces, scoped per environment)
    //    - reject a surface that equals a reserved built-in root, i.e. a member
    //      of builtin_ffcmd_roots() -- a NEW CR-NR-113 pre-pass invariant (9.5),
    //      NOT command-framework Req 8.10 (that rule governs resolve_target
    //      stage order and does not see aliases)
    // 2. Serialise rows -> [[alias]] TOML text (NEW serialiser; none exists today)
    // 3. create_dir_all(user_aliases_dir); fs::write(user.toml, text)
    //    On any failure in steps 1-3 the on-disk file is left UNCHANGED (Req 5.4).
    // 4. Refresh the live resolver IN PLACE via the EXISTING reload seam, which
    //    must be EXTENDED to re-apply BOTH the per-locale overlay AND the
    //    User_Alias_Store overlay in one re-seed (9.5) so the alias is live now
    //    and the per-locale overlay is not dropped.
}
```

Two genuinely NEW pieces (everything else is a copy of the Menus Editor):
1. An alias SERIALISER (`rows -> TOML`). None exists today -- `alias_reader.rs`
   only READS; the only writer is a `#[cfg(test)]` helper. The serialiser emits
   `[[alias]]` tables with `surface`, `canonical`, the `environment` field WHEN it
   is not the `ffcmd` default (so the FFEDIT routing a user row needs survives a
   round-trip, aligning with Task 7.2.2), and `command_template` when non-empty
   (omitting the `ffcmd`-default `environment`, an empty `command_template`, and
   the always-blank `localisation_key` so a plain `ffcmd` pure-verb alias
   round-trips to today's minimal shape).
2. A User_Alias_Store LOAD path in `LocaleAliasStore` that reads the user file
   through a NEW user-store row type / parser (9.5), NOT the existing 3-field
   `parse_alias_toml`.

Validation reuses the loader's discipline so the editor cannot produce a file the
reader rejects -- the same contract the Menus Editor's `validate_menu` provides.

**User-store parser (why not `parse_alias_toml`).** The existing
`parse_alias_toml` deserialises into `AliasRowToml`, which carries only
`surface`, `canonical`, and `localisation_key` -- it has NO `environment` field
(and no `command_template`), so routing a user row and preserving it on a
round-trip through `parse_alias_toml` is impossible: a non-default `environment`
would be silently dropped. The user store is therefore read by a NEW dedicated
row type `UserAliasRowToml` and its parser `parse_user_alias_toml`, declared
alongside `AliasRowToml` in `alias_reader.rs`:

```rust
// Validates: command-configurator Requirement 5.1, 5.3, 5.7
#[derive(serde::Deserialize, serde::Serialize)]
struct UserAliasRowToml {
    surface: String,
    canonical: String,
    #[serde(default)]                 // absent -> AliasEnvironment::Ffcmd
    environment: AliasEnvironment,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    command_template: String,         // CR-NR-112-gated (9.0); blank until then
    #[serde(default, skip_serializing_if = "String::is_empty")]
    localisation_key: String,         // always blank for a user shorthand
}
```

`AliasEnvironment` derives `serde::Deserialize`/`Serialize` with
`#[serde(rename_all = "lowercase")]` so the TOML surface is `ffcmd` / `ffedit`.
This is ADDITIVE: the existing `AliasRowToml` and `parse_alias_toml` (the
per-locale catalogue reader) are UNCHANGED, so the CR-NR-112 four-field record and
the shipped `i18n/` layout are untouched. The serialiser emits
`UserAliasRowToml` and the round-trip test (9.8) reads it back through
`parse_user_alias_toml`, not `parse_alias_toml`.

### 9.5 Resolver load path and precedence

`LocaleAliasStore` gains a user-store load layered as a THIRD overlay:

```
seed identity base (built-in roots)            // existing
  + per-locale overlay (i18n/<locale>/aliases) // existing, CR-NR-109/111
  + User_Alias_Store overlay                    // NEW (CR-NR-113)
```

**Store-to-environment mapping (Req 5.1, 5.7).** The user store is read by the
NEW `parse_user_alias_toml` / `UserAliasRowToml` (9.4), NOT the existing 3-field
`parse_alias_toml`, because the per-environment routing needs an `environment`
field that `AliasRowToml` does not carry. The overlay seam
(`load_into_keyed` -> `apply_locale_overlay_keyed`) still loads rows into a SINGLE
per-environment `AliasTable` (FFCMD, FFEDIT, ...), so the user-store load reads
each `UserAliasRowToml` and routes it into its `environment`'s table through that
same keyed seam. Each `[[alias]]` row carries an optional `environment` field
(`ffcmd` | `ffedit`) that defaults to `ffcmd`. The loader routes a row into that
environment's table: `ECOB -> EDIT` is authored as `environment = "ffedit"` (EDIT
is an FFEDIT root); a bare workbench shorthand defaults to `ffcmd`. The editor exposes the environment as a per-row selector
(Req 5.3). Rationale for a per-row field over inferring the environment from the
canonical's owning environment: inference needs a reverse lookup that does not
exist as a stable seam today, and a root verb could in principle be valid in more
than one environment; an explicit field is unambiguous and serialises cleanly.

**No-shadow invariant (Req 5.4, 5.7) -- NEW, not Req 8.10.** A user alias MUST
NOT equal a reserved built-in root command, where "reserved built-in root" is the
set returned by `builtin_ffcmd_roots()` (which folds in
`command_config::BUILTIN_WORKSPACE_VERB_ROOTS`). This is a NEW CR-NR-113 invariant
of the alias PRE-PASS, enforced at save (reject) and at load (skip-with-WARN). It
is NOT inherited from command-framework Req 8.10: Req 8.10 orders the stages of
`resolve_target` (built-in command > menu > macro) and never sees aliases, which
are rewritten in the `rewrite_verb_to_root` / `canonical_verb` pre-pass that runs
BEFORE `resolve_target`. Without this new invariant a user alias equal to a
built-in verb would be rewritten away before 8.10 could apply, i.e. it COULD
shadow a built-in -- the opposite of what 8.10 constrains. The invariant is
analogous in INTENT to 8.10 but mechanically distinct and lives in a different
layer.

**Within-store vs cross-layer collisions (Req 5.4, 5.7).** Two collision scopes,
handled distinctly:
- WITHIN the User_Alias_Store, in the SAME environment: a surface mapped to two
  different roots is rejected at save (Req 5.4) and at load via the existing
  `apply_locale_overlay_keyed` within-environment collision discipline.
- CROSS-LAYER (a user surface equals a per-locale overlay surface already loaded):
  the loader compares roots. SAME root = no-op (the mappings agree, nothing to
  reconcile). DIFFERENT root = the loader REJECTS the user row with a WARN and
  RETAINS the per-locale surface (the per-locale overlay is authoritative). This
  resolves the earlier "coexist vs reject" ambiguity: user and per-locale
  surfaces coexist whenever the SURFACES differ; when the same surface maps two
  ways, the per-locale layer wins and the user row is dropped-with-WARN rather
  than silently overriding. v1 keeps the two in SEPARATE files and the editor
  only touches the user file (Req 5.11), so a cross-layer clash arises only from
  an out-of-band per-locale authoring, not from the editor.

**Live refresh after save (Req 5.6) -- the reload seam is EXTENDED, not merely
reused.** The existing `reload_active_alias_store_for_locale` rebuilds via
`seed_and_overlay_for_locale`, which today applies ONLY the Identity_Base and the
`i18n/<locale>/aliases/` overlay and reads NO user store; reloading it unchanged
after a save would DROP the just-saved user alias. So the re-seed path is EXTENDED
to apply the User_Alias_Store overlay as the third layer in the SAME re-seed that
re-reads `i18n/`. One re-seed therefore re-applies BOTH overlays: a save never
drops the per-locale aliases (F6 hazard) and the new user alias goes live without
restart (the same `ACTIVE_ALIAS_STORE` slot a `ui.locale` change already uses). A
unit test asserts that after save-then-reload the live store carries BOTH the
per-locale overlay AND the new user alias.

### 9.6 Registration (configurator family, no new Kind -- but a new TabKind)

The Theme / Menus / Keys / Kinds editors all reuse
`WorkspaceKind::CommandConfigurator` with an `editor` marker param, so NO new
`WorkspaceKind` is introduced. IMPORTANT: "no new WorkspaceKind" does NOT mean "no
new runtime TabKind" -- each sibling editor is a full set of wiring points, not
only a descriptor arm. To match the Menus Editor one-to-one the Alias Editor
needs ALL of the following (verified against the live sibling wiring):

1. A new runtime `TabKind::AliasEditor` variant and a matching `KindTag::AliasEditor`
   (`tab_state.rs`).
2. A `BuiltinKind` mapping for the new `KindTag` (`workspace_kind/mod.rs`, e.g.
   `KindTag::AliasEditor -> BuiltinKind::...` alongside the Menus arm).
3. The non-persist arms in `session_manager.rs` (the `KindTag::AliasEditor => None`
   arm plus the two match lists the sibling editors appear in).
4. A tab constructor (`tab_state_ctors.rs`, e.g. `alias_editor(...)`).
5. The descriptor arm `editor = "aliases"` in `shell/nav_stack.rs`
   (`descriptor_for_current_context`) and the matching reconstruct arm in
   `nav_reconstruct.rs` (`reconstruct_custom`) that calls `open_alias_editor`.
6. A DEDICATED central-panel render arm in `shell/render_body.rs` doing the
   owned-panel swap: `mem::take` the panel -> `render_workspace_context` -> put it
   back -> `apply_alias_editor_action`.
7. A shell-held panel field for `AliasEditorState` in `construct.rs` (inside a
   grouped sub-struct, not a flat `WorkbenchShell` field).

A command verb `ALIASES` (classified as a `CustomWorkspace` target by the shell
resolver, or a menu option) opens it through the single front door. No new
dispatcher and no new session-persistence format -- the Context still persists as
the existing `CustomWorkspace { CommandConfigurator }` descriptor (Section 7); the
new `TabKind`/`KindTag` are runtime-only wiring that map back onto that one
descriptor, exactly as the sibling editors do.

### 9.7 Dispatch and expansion are unchanged (CR-NR-112 owns them)

The Alias Editor is purely an AUTHORING + PERSISTENCE surface. Once a definition
is saved and the resolver refreshed, invoking it (`ecob helloW`) runs the
CR-NR-112 pre-pass (`rewrite_verb_to_root` -> `expand_template` ->
`EDIT FFWB.COBOL.MASTER(helloW)` -> `resolve_target`) with NO new behaviour
(Req 5.8). The editor adds no expansion logic and no dispatch path. NOTE: the
`expand_template` step and the `command_template` field it consumes are provided
by CR-NR-112 and do not exist in source yet (9.0); this end-to-end path is only
buildable once CR-NR-112 has landed.

### 9.8 Testability

- PURE unit tests on the alias serialiser: a row with a template round-trips to
  `[[alias]]` with `command_template`; a pure verb row omits `command_template`
  and `localisation_key`; a non-default `environment = "ffedit"` row round-trips
  (emitted and read back) while an `ffcmd`-default row omits `environment` on
  serialise and parses back to `ffcmd`; serialise-then-parse THROUGH THE NEW
  `parse_user_alias_toml` / `UserAliasRowToml` (9.4) -- NOT the 3-field
  `parse_alias_toml`, which has no `environment` and would drop it -- yields the
  same rows.
- Save-path unit tests: empty surface/canonical rejected; within-same-environment
  duplicate surface -> two roots rejected; a surface equal to a built-in root
  (a `builtin_ffcmd_roots()` member) rejected; a valid set writes the file and the
  live store resolves the new alias.
- Reload-layering unit test: after save-then-reload, the live `LocaleAliasStore`
  STILL carries BOTH the per-locale `i18n/<locale>/aliases/` overlay AND the newly
  saved user alias (guards the extended re-seed against dropping either layer).
- Environment-routing unit test: a row with `environment = "ffedit"` loads into
  the FFEDIT table and resolves there; a row defaulting to `ffcmd` loads into FFCMD.
- A full-shell egui_kittest first-Tab test (MANDATORY, workspace-conformance.md):
  open the Alias Editor, assert the first Tab from the command field lands on the
  first interior control (no phantom stop).
- A full-shell behavioural test: author `ECOB`/`EDIT`/`FFWB.COBOL.MASTER(&1)` in
  the editor, Save, then type `ecob helloW` and assert it expands to
  `EDIT FFWB.COBOL.MASTER(helloW)` (end-to-end with CR-NR-112).

### 9.9 Framework-conformance statement

The Context uses the existing `WorkspaceContext` + `render_workspace_context`
single focus-latch path, the existing `CommandConfigurator` Kind + descriptor,
and the single command-dispatch front door. No core framework type is reshaped.
The ONE new mechanism is the User_Alias_Store location + its resolver load path
(9.1, 9.5) -- an ADDITIVE new persistence/load path, not a reshape of
`CommandTarget` / `WorkspaceContext` / `InteriorFocus` / `WorkspaceDescriptor` /
`LocaleAliasStore`'s store contract / the navigation stack. The reload-seam
EXTENSION (9.5) adds a third overlay to an existing function; it does not change
the shape of any published type. By the framework-conformance definition this is
NOT a framework change.

OWNER-CONFIRM POINTS still open at the gate CONFIRM step (do not resolve silently):

1. **User store path/shape** -- the new user-writable store at
   `<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`, its `[[alias]]` row
   shape (read/written by the NEW `UserAliasRowToml` / `parse_user_alias_toml`,
   distinct from the existing 3-field `AliasRowToml`), and the per-row
   `environment` selector added in this revision (9.1, 9.2, 9.4, 9.5). This is a
   NEW TOML shape beyond the CR-NR-112 four-field record and differs from the
   shipped per-environment-FILE layout (where environment comes from the
   filename); the single-`user.toml`-with-per-row-`environment` choice and the
   new row type are concrete design decisions this revision makes to resolve the
   store-to-environment routing gap. They are flagged here for owner sign-off at
   the CONFIRM step rather than treated as settled.
2. **Resolver precedence + no-shadow-built-in-root rule** -- the user store as a
   THIRD overlay, the SAME-root no-op / DIFFERENT-root per-locale-wins cross-layer
   rule, and the NEW no-shadow pre-pass invariant grounded on
   `builtin_ffcmd_roots()` rather than Req 8.10 (9.5).
3. **v1 scope = user file only** -- the editor authors the User_Alias_Store ONLY;
   the per-locale `i18n/<locale>/aliases/` catalogues are out of scope
   (Req 5.11), avoiding CR-NR-111 overlap.

These are recorded for owner sign-off because they introduce a new user-writable
store, a new resolver load order, and a new (additive) no-shadow invariant.
