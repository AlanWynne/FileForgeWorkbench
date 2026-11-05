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

---

## 9.10 Alias Editor ISPF redesign (Requirement 5.13-5.24, CR-CH-068, Gate 2)

### 9.10.0 What this redesign IS and is NOT

The CR-NR-113 Alias Editor (Sections 9.1-9.9) shipped a FLAT free-text grid:
each row is a `surface` / `canonical` / `environment-combo` / `command_template`
set of text boxes, with a footer "Add alias" button and a per-row "Delete"
button. The owner could not see rows or an obvious way to add the first one (the
Add-visibility bug, `.agents/tasks/alias-editor-workspace/add-row-bug-findings.md`,
whose root cause is a render-visibility / empty-store layout risk plus a likely
stale running binary).

CR-CH-068 REDESIGNS the UI/interaction model to the owner's ISPF mock. It is a
PURE UI/interaction redesign over the EXISTING data layer (`user_aliases.rs`
`UserAliasRowToml` + `parse_user_alias_toml` + `serialise_user_aliases`,
Section 9.4) and the Gate 1 API (CR-CH-067 `EnvironmentRegistry::root_commands` /
`contains_root`). It changes NO on-disk shape, NO dispatch front door, NO
WorkspaceKind, NO navigation stack, NO session-persistence format, and NO
focus-latch mechanism. The technology stack is fixed and unchanged: Rust in
`ff-desktop`, serde + `toml`, egui + the `WorkspaceContext` framework; no new
crate and no new dependency.

The footer Add/Delete buttons of 9.3 are SUPERSEDED by ISPF line commands; the
rest of 9.1-9.9 (store location, serialiser, load path, precedence, save
validation, live refresh, registration, expansion delegation) is RETAINED
verbatim. Where 9.10 and 9.1-9.9 differ, 9.10 governs the editor UI; everywhere
else 9.1-9.9 is authoritative.

### 9.10.1 The ISPF layout (the owner's mock)

```
FileForge Workbench Alias Editor
Command ===> ________________________________________________________________
Act CE       Root     L18N ALIAS        Parameters
    ________ ________ __   ____________ ____________________________________   <- FILTER row
I   FFCMD    EDIT     en   ECOB         FFWB.COBOL.MASTER(&1)                   <- data rows (filtered)
    ________ ________ __   ____________ ____________________________________
    ________ ________ __   ____________ ____________________________________
```

Top to bottom: a `Command ===>` primary command field; a column-heading row
(Act / CE / Root / L18N / ALIAS / Parameters); a FILTER row immediately under the
headings; then the data rows (the current aliases that match the filter), inside
a vertically scrolling area with a scrollbar on the right.

### 9.10.2 Column-to-model mapping (Req 5.13; on-disk shape UNCHANGED)

| Column | Backing field (`UserAliasRowToml`) | Notes |
|--------|------------------------------------|-------|
| Act | (none -- transient UI only) | A per-row line-command letter; never persisted. |
| CE | `environment` (`AliasEnvironment`) | Dropdown of registered CEs (9.10.5). |
| Root | `canonical` | Dropdown from `root_commands(CE)` (9.10.5). |
| L18N | `localisation_key` | Editable; defaults blank for a user shorthand; PERSISTS via `localisation_key` (round-trips, see DECISION below). |
| ALIAS | `surface` | The typeable alias name. |
| Parameters | `command_template` | CR-NR-112 template (`&1..&N` / `&*`). |

The existing `AliasRowEdit` struct (`alias_editor_panel/state.rs`: `surface`,
`canonical`, `environment`, `command_template`) ALREADY carries the backing
fields; the redesign adds a transient `act: String` per row for the line-command
letter, a `filter_ce: Option<AliasEnvironment>` + `filter_root: String` pair for
the FILTER row, and a `command_line: String` for the `Command ===>` field --
all on `AliasEditorState`, none persisted. `localisation_key` is NOT in
`AliasRowEdit` today.

DECISION (Req 5.13, review finding 1 -- option (a), L18N genuinely round-trips):
add a `localisation_key: String` EDIT field to `AliasRowEdit` and render the L18N
column as a narrow editable field that defaults blank. The L18N value PERSISTS via
the existing `UserAliasRowToml.localisation_key` on-disk field (no on-disk-shape
change -- the field already exists; Req 5.24 preserved). This requires a one-line
change to the SOLE save path: `validate_user_alias_rows`
(`crates/ff-desktop/src/shell/alias_editor.rs`, ~line 241) today constructs every
`UserAliasRowToml` with `localisation_key: String::new()` HARDCODED, discarding any
edited L18N value; it SHALL instead carry
`localisation_key: row.localisation_key.trim().to_string()`. A round-trip unit test
(a row with a non-empty L18N serialises and re-parses with that key) guards the
change. Task 8.9 owns both the save-path edit and the round-trip test. Rationale
for option (a) over a read-only L18N: the owner's mock explicitly shows an `en`
value typed into the L18N column, so the column must accept and keep a value;
because the backing on-disk field already exists, persisting it is in scope and is
NOT a model change.

### 9.10.3 The Command line (Req 5.14, 5.22)

`AliasEditorState` gains `command_line: String`. The pure render draws the
`Command ===>` field with a STABLE `egui::Id` (e.g.
`egui::Id::new("alias_editor_command_line")`) and, on Enter (the field's
`lost_focus()` + Enter key, mirroring the shell command line), returns a new
`AliasEditorAction::CommandLine(String)` carrying the typed text. The shell
`apply_alias_editor_action` parses it (case-insensitive verb):

- `SAVE` -> first apply pending line commands (9.10.4), then `save_user_aliases`
  (Section 9.4, unchanged).
- `CANCEL` / `END` -> leave via the Navigation_Stack (the existing END/F3 path,
  criterion 5.9 / Section 9.3). `CANCEL` discards the working copy; `END`
  (RECOMMENDED) behaves as today's return.
- `FILTER <ce> <root>` / `RESET` (RECOMMENDED, owner to confirm) -> set / clear
  `filter_ce` + `filter_root`.
- empty / unrecognised -> apply pending line commands only (ISPF: a bare Enter
  on the command line applies pending line commands).

This mirrors the Menus Editor command-handling shape (menu-workspace Req 13.1):
a pure render returns an Action, the shell command layer performs the effect.

### 9.10.4 Line commands I / R / D and the apply pass (Req 5.16, 5.22 -- the batch model)

Each data row renders a small Act `TextEdit` bound to `row.act`. Typing a letter
does NOT restructure the list mid-frame (that would fight egui's immediate-mode
iteration and the Tab/focus contract). Instead the letters accumulate as PENDING
state on the rows. The batch is applied when the user presses Enter on the
`Command ===>` line (Req 5.16 / 5.22), via a dedicated
`AliasEditorAction::ApplyLineCommands` emitted before/with the command-line
parse, handled shell-side in a single deterministic pass:

```
fn apply_pending_line_commands(&mut self) {
    // 1. Snapshot the Act letters (case-insensitive) with their row indices.
    // 2. Process by DESCENDING row index so inserts/repeats/deletes do not
    //    shift the indices of rows not yet processed:
    //      D -> remove row i
    //      I -> insert a blank AliasRowEdit AFTER row i
    //      R -> insert a clone of row i (its field values) AFTER row i
    // 3. Clear every Act field that was consumed.
}
```

Descending-index processing is the concrete answer to Req 5.16's "apply
deterministically": a delete at index 2 never invalidates a pending insert at
index 5, and an insert-after never reorders an earlier pending command. An `I`
yields a blank row (`AliasRowEdit::default()`); an `R` yields a `.clone()` of the
source row's persisted fields with its Act cleared.

EDGE CASES: an unrecognised Act letter is ignored and its field cleared (ISPF
tolerates and clears stray line commands); an `I` or `R` on the last row inserts
at the end; a `D` on a row that is the only row leaves zero rows but the
empty-store affordance (9.10.6) still renders an insertable row.

### 9.10.5 Cascading CE -> Root dropdowns (Req 5.18; consumes CR-CH-067)

CE and Root become `egui::ComboBox` dropdowns (replacing the free-text Root field
and extending the existing `AliasEnvironment` combo). The CE combo lists the
registered environments. For v1 the editor offers the two user-addressable alias
environments `AliasEnvironment::ALL` (`ffcmd`, `ffedit`) -- the same set the model
supports today -- so the CE list is `["ffcmd", "ffedit"]`; a future expansion to
the full registry membership is additive.

The Root combo is populated by the Gate 1 associated function:

```rust
// ACTUAL implemented signature (associated fn, NOT a &self method):
//   crates/ff-desktop/src/shell/environment_registry.rs
//   pub(crate) fn root_commands(env_name: &str) -> Vec<String>
let roots = EnvironmentRegistry::root_commands(row.environment.env_name());
```

`row.environment.env_name()` returns `"FFCMD"` / `"FFEDIT"` (the
`user_aliases::AliasEnvironment::env_name` already defined). When the user changes
`row.environment`, the render re-reads `root_commands` for the new CE on the next
frame (immediate mode makes this automatic), and the shell clears/revalidates
`row.canonical` if it is no longer in the new environment's root set
(`EnvironmentRegistry::contains_root(row.environment.env_name(), &row.canonical)`).
Because the Root combo offers ONLY `root_commands(CE)` entries, a chosen (CE, Root)
pair is valid BY CONSTRUCTION -- the dropdown cannot produce a pair that
`contains_root` would reject. The FILTER row's Root dropdown uses the same
`root_commands(filter_ce)` source.

`root_commands` is a cheap, deterministic, sorted list (CR-CH-067 Req 20.2); the
render may call it each frame without caching, but a per-CE memoised cache on
`AliasEditorState` is an acceptable optimisation if profiling shows cost.

### 9.10.6 Empty-store usability (Req 5.17 -- the Add-visibility bug fix by redesign)

The CR-NR-113 bug was that an empty store rendered column headings only, with the
Add control in a footer that could be clipped below the viewport. The redesign
removes the footer entirely. In the empty-store case the render ALWAYS draws the
`Command ===>` line, the column headings, the FILTER row, AND at least one empty
insertable data row (a blank `AliasRowEdit` rendered with an Act field). The
user creates the first alias either by typing directly into that blank row's
ALIAS/Root cells or by typing `I` in its Act field and pressing Enter. Because
the blank row is part of the normal, always-rendered row list (not a
footer-after-scroll-area widget), the empty-store layout is identical in
structure to the non-empty layout and cannot be clipped out of view. The
regression test (9.10.9) drives an `I` in the empty-store case and asserts a
visible editable row appears.

IMPLEMENTATION NOTE: the always-present blank row is a RENDER concern -- the
working copy `rows` may be genuinely empty; the render inserts a transient
"scratch" blank row for display when `rows.is_empty()`, and an edit/`I` on it
promotes it into `rows`. This keeps "empty store = zero persisted rows" true while
guaranteeing a visible insertion surface.

### 9.10.7 The filter (Req 5.15)

`AliasEditorState` gains `filter_ce: Option<AliasEnvironment>` and
`filter_root: String`. The render computes the DISPLAYED row set as the subset of
`rows` where `filter_ce.is_none() || row.environment == filter_ce` AND
`filter_root.is_empty() || row.canonical.eq_ignore_ascii_case(filter_root)`.
Filtering is DISPLAY-only -- it never mutates `rows`, so a filtered-out row is
preserved and saved. The filter matches CE + Root ONLY (Req 5.15); ALIAS and
Parameters are not filter inputs.

DECISION (RECOMMENDED, owner to confirm at CONFIRM): the filter is applied on
Enter / an explicit `FILTER`/`RESET` command rather than live-as-you-type, which
avoids re-filtering the list on every keystroke and matches ISPF's
command-driven filtering. Live-as-you-type is a trivial alternative (filter in
the render directly from the filter-field bindings); the design picks
applied-on-command for ISPF fidelity and defers to the owner.

A subtle index concern: because line commands (9.10.4) operate on `rows` by
index but the user sees the FILTERED subset, the Act field and its row must carry
the row's index INTO `rows` (not the display position). The render iterates the
filtered subset but captures each row's original `rows` index for the pending
line-command snapshot, so a `D` on a visible row deletes the correct backing row.

### 9.10.8 Focus / Tab reconciliation -- the hard problem (Req 5.21, 5.23)

This is the novel design problem: ISPF line commands are "type a letter in a cell,
apply on Enter", which must coexist with egui's immediate-mode widget model and
the app's single `WorkspaceContext::render -> InteriorFocus` Tab/focus contract
(workspace-conformance.md) WITHOUT a phantom Tab stop.

The reconciliation has four concrete parts:

1. **No mid-frame restructuring.** Line-command letters are PENDING state on the
   rows (9.10.4); the row list is only restructured in the shell's apply pass
   AFTER the frame, via the owned-panel swap (`mem::take` -> `render_workspace_context`
   -> put back -> `apply_alias_editor_action`). The render therefore iterates a
   stable `rows` for the whole frame -- egui assigns stable ids and the Tab ring
   is well-defined within the frame. This is exactly the existing
   `pending_action` discipline, extended with the new actions; it is NOT a new
   mechanism.

2. **Stable ids, visual Tab order.** Every control is created in visual order so
   egui-native Tab traversal walks them in that order:
   `Command ===>` (stable id `alias_editor_command_line`) -> FILTER row (filter CE
   combo, filter Root combo) -> each data row's Act -> CE -> Root -> L18N ->
   ALIAS -> Parameters. Each per-row control keeps a stable salted id keyed by the
   row's backing index (e.g. `("alias_act", idx)`, `("alias_surface", idx)`),
   mirroring the existing `surface_id(i)` pattern so ids do not collide across
   rows.

3. **First interior = the FILTER row's first field.** The `WorkspaceContext::render`
   impl reports `InteriorFocus { first, last }` where `first` is the FILTER row's
   first focusable control (the filter CE combo's response id, captured fresh each
   frame -- a combo's id is captured from its `response.id`, per
   workspace-conformance.md, because a combo id must round-trip through egui
   focus), and `last` is the last data-row control (or the filter row when there
   are no data rows). The `Command ===>` line is the ENTRY control (the shell's
   command field analogue), so the first Tab FROM it lands on the FILTER row's
   first field (Req 5.23) -- not a phantom stop. The mandatory full-shell
   egui_kittest first-Tab test (9.10.9) asserts exactly this.

4. **Primary command vs line command ordering.** Enter on the `Command ===>` line
   emits `ApplyLineCommands` then `CommandLine(text)` (or a single combined action
   the shell sequences), so pending Act codes are applied before `SAVE` runs
   (Req 5.22). Focus after an apply returns to the `Command ===>` line (ISPF
   convention), which the shell can request via egui's `request_focus` on the
   command-line id after a structural change.

WHY NOT a per-row "insert" button instead of an Act field: a per-row button ring
would add N focus stops and diverge from the ISPF mental model the owner asked
for; the Act text field is ONE focusable control per row that carries the line
command, matching ISPF and keeping the Tab ring predictable.

`AliasEditorState` therefore gains (all transient, none persisted): `command_line:
String`, `filter_ce: Option<AliasEnvironment>`, `filter_root: String`, and a
per-row `act: String` on `AliasRowEdit`. `first_interior_id` / `last_interior_id`
remain the reported anchors; `first_interior_id` now tracks the FILTER row's first
field rather than the first row's surface / the Add button.

### 9.10.9 Testability (Req 5.17, 5.19, 5.23)

- PURE unit tests on `apply_pending_line_commands`: `I` inserts a blank row after
  the source; `R` inserts a field-copy after the source; `D` removes the row;
  multiple pending commands apply deterministically (descending-index) without
  index corruption; an unrecognised Act letter is cleared and ignored.
- PURE unit tests on the filter: empty filters show all; a CE filter restricts by
  environment; a Root filter restricts by canonical; CE+Root is an AND; ALIAS /
  Parameters are NOT filter inputs; filtering never mutates `rows`.
- PURE unit test on the CE->Root cascade: changing a row's CE re-sources the Root
  options from `root_commands(new_CE)` and a now-invalid Root is cleared /
  flagged; a (CE, Root) pair chosen from the dropdowns satisfies `contains_root`.
- Save-path unit tests (EXISTING, retained): the `validate_user_alias_rows` ->
  `contains_root` save-side hook rejects a root invalid in its declared
  environment (Req 5.19); empty surface / empty root / within-environment
  surface->two-roots / built-in-root shadow all rejected; file unchanged on
  failure.
- L18N round-trip unit test (Req 5.13, 9.10.2 DECISION): an `AliasRowEdit` with a
  non-empty `localisation_key` passed through `validate_user_alias_rows` yields a
  `UserAliasRowToml` carrying that trimmed value, and re-parsing the serialised
  store re-renders it. This is the guard for the save-path change that stops
  `localisation_key` being hardcoded empty (task 8.9.5).
- MANDATORY full-shell egui_kittest first-Tab test (workspace-conformance.md,
  Req 5.23): open the Alias Editor, assert the first Tab from the `Command ===>`
  field lands EXACTLY on the FILTER row's FILTER CE dropdown (the one deterministic
  first interior control; no phantom stop).
- MANDATORY full-shell regression test for the Add-visibility bug (Req 5.17):
  with an EMPTY store, render the editor, type `I` into the always-present Act
  field, press Enter on the `Command ===>` line, and assert a VISIBLE editable
  data row appears (its ALIAS field renders and accepts typed input that persists
  into `rows[0].surface`). This is the test the add-row-bug findings report
  identified as missing.

### 9.10.10 Gate 1 carry-over notes resolved (CR-CH-067 -> CR-CH-068)

Gate 1 (CR-CH-067) left two non-blocking notes for Gate 2:

(a) **`&self`-vs-associated-fn doc drift.** Earlier narrative referred to
`root_commands` / `contains_root` as instance methods. The ACTUAL implemented
signatures are ASSOCIATED functions keyed by environment NAME on the
`EnvironmentRegistry` type (verified in
`crates/ff-desktop/src/shell/environment_registry.rs`):

```rust
pub(crate) fn root_commands(env_name: &str) -> Vec<String>
pub(crate) fn contains_root(env_name: &str, root: &str) -> bool
```

They are called as `EnvironmentRegistry::root_commands("FFCMD")` /
`EnvironmentRegistry::contains_root("FFEDIT", "FIND")`, NOT `registry.root_commands(..)`.
This design and its tasks reference the associated-function form throughout
(9.10.5, 9.10.9). RESOLVED.

(b) **Per-environment no-shadow reconciliation.** The current no-shadow guard in
`validate_user_alias_rows` rejects a surface that equals a member of
`builtin_ffcmd_roots()` (an FFCMD-only reserved set), regardless of the row's
declared environment. A fully per-environment no-shadow rule would instead reject
a surface that equals a reserved root of the row's OWN environment (so an FFEDIT
row's surface is checked against FFEDIT's roots, an FFCMD row's against FFCMD's).

DECISION (RECOMMENDED, owner to confirm at CONFIRM): KEEP the current
`builtin_ffcmd_roots()`-based guard for CR-CH-068 and treat full per-environment
no-shadow reconciliation as a LATER follow-up, NOT in this redesign. Rationale:
(1) the Gate 1 `contains_root` save-side hook already enforces that a row's ROOT
is valid in its declared environment, which is the primary (environment, root)
integrity Gate 2 needs; (2) the `builtin_ffcmd_roots()` guard is a superset
safety net against shadowing the workbench base verbs (the highest-risk case);
(3) making no-shadow per-environment touches the load-path skip-with-WARN and the
save-path reject in lock-step and is better done as its own small gated change
with its own tests, rather than riding along with a UI redesign. If the owner
prefers per-environment no-shadow now, it is an additive change to
`validate_user_alias_rows` + `apply_user_rows` using the SAME `root_commands(env)`
data already available -- recorded here as the alternative.

### 9.10.11 What is explicitly NOT changing

- The on-disk `UserAliasRowToml` shape and the `user.toml` location (Section 9.1,
  9.4). The redesign reads/writes the SAME file via the SAME
  `parse_user_alias_toml` / `serialise_user_aliases`.
- The single command-dispatch front door: `ALIASES` ->
  `builtin_workspace_target_for` -> `custom("aliases")` ->
  `dispatch_command_target` -> `open_alias_editor` (Section 9.6). The
  `Command ===>` line inside the editor is the editor's OWN primary-command field
  (like the Menus Editor's), NOT a second global dispatcher.
- `WorkspaceKind::CommandConfigurator` + `editor = "aliases"`, the runtime
  `TabKind::AliasEditor`, and all sibling-parity wiring (Section 9.6).
- The per-tab Navigation_Stack return (Section 9.3), the `pending_action` /
  owned-panel-swap effect pattern, and the single `WorkspaceContext` focus-latch
  path (Section 9.3, 9.9).
- The resolver load/precedence model, the live in-place refresh after save, and
  the expansion/dispatch delegation to CR-NR-112 (Sections 9.5, 9.7).
- v1 scope: the editor authors the User_Alias_Store ONLY; the per-locale
  `i18n/<locale>/aliases/` catalogues remain out of scope (Req 5.11).

### 9.10.12 Framework-conformance statement (CR-CH-068)

The redesign builds ON the framework: one dispatch front door, one navigation
stack, one `WorkspaceContext` focus-latch path, one persistence model, code-only
built-ins. It consumes the CR-CH-067 additive read-only API. It reshapes NO core
framework type (`CommandTarget`, `WorkspaceContext`, `InteriorFocus`,
`WorkspaceDescriptor`, `KindConfig`, the `BackendEnvironment` store contract). By
the framework-conformance definition this is NOT a framework change -- it is a
UI/interaction redesign of one Context plus consumption of an already-approved
additive API. No new owner-confirm framework item surfaces; the five OPEN
DECISIONS (R semantics, filter timing + scope, primary-command set, per-environment
no-shadow timing, v1 CE dropdown membership) are UI/scope choices recorded for the
gate CONFIRM step, not framework changes.

### 9.10.13 Responses to Gate 2 design review (design-review.md, iteration 2)

The Gate 2 review returned CHANGES_REQUESTED (2 MEDIUM + 2 NIT). All four findings
were ADDRESSED docs-only; none required a framework change.

- Finding 1 (MEDIUM -- L18N save-path contradiction): ADDRESSED via review option
  (a). The L18N column genuinely round-trips: `AliasRowEdit` gains an editable
  `localisation_key` field (9.10.2 DECISION), and the sole save path
  `validate_user_alias_rows` is changed to carry `row.localisation_key.trim()`
  instead of the hardcoded `String::new()` (tasks 8.9.4 round-trip test + 8.9.5
  save-path change). Because the on-disk `UserAliasRowToml.localisation_key` field
  ALREADY exists, this is NOT an on-disk-shape change (Req 5.24 preserved). Chosen
  over option (b) read-only because the owner's mock shows an editable `en` value.
- Finding 2 (MEDIUM -- ambiguous first-interior target): ADDRESSED. Req 5.23 now
  names EXACTLY ONE first interior control -- the FILTER row's filter CE dropdown
  (its `response.id` captured fresh each frame) -- matching 9.10.8 part 3, so the
  mandatory first-Tab egui_kittest asserts a single deterministic `egui::Id`. Tasks
  8.7.1 / 8.7.2 already target the filter CE combo.
- Finding 3 (NIT -- CE membership vs "registered command environments"): ADDRESSED.
  Req 5.18 now notes v1 CE membership = the two user-addressable alias environments
  (`AliasEnvironment::ALL`), with full-registry enumeration a deferred additive
  follow-up; recorded as OPEN DECISION 5 in the gate-summary for owner confirm.
  9.10.5 already scoped the v1 CE list to the two environments.
- Finding 4 (NIT -- Tab-order prose): ADDRESSED. Req 5.21(c) reworded to separate
  the FILTER row field set (filter CE -> filter Root) from each data row's field set
  (Act -> CE -> Root -> L18N -> ALIAS -> Parameters).

---

## Design Delta: FFCMD `X -> EXIT` seed alias, written first run + rebuilt by Reset bare (Requirement 8, CR-CH-116)

This delta adds ONE genuine seed alias, `X -> EXIT`, to the FFCMD alias layer and
materialises it to disk under the write-through config model (configuration-system
Requirement 23, startup-and-session Requirement 23). It builds on Requirement 5 (the
User_Alias_Store, its serialiser, and the three-layer resolver) and introduces NO
new alias mechanism, dispatcher, or resolution path.

### Technology stack (locked)

Rust + the existing `shell/alias_reader.rs` (seed + overlay), `shell/user_aliases.rs`
(the `[[alias]]` TOML serialiser and the `user_alias_override_base()` seam),
`shell/environment.rs` (`AliasTable`), and the shared Config_Rebuild_Helper
(configuration-system design). `toml` + `serde`. No new dependency.

### What is written and where

The seed file is `aliases/<locale>/FFCMD.toml` (ENV in the filename, per the
existing model), `<locale>` = the active `ui.locale` (default `en`). It contains the
single genuine row `surface = "X", canonical = "EXIT"` (environment `ffcmd` implied
by the filename). The writer reuses `user_aliases::serialise_user_aliases(&rows)`
exactly as the Alias Editor's save path does, so the on-disk shape is identical to a
user-authored FFCMD override and the row shows up in the Alias Editor's FFCMD view
(Requirement 8.6). The write goes through `user_alias_override_base()`, which honours
`dir_overrides.user_aliases` / `FFWB_USER_ALIASES_DIR` (Requirement 8.3), so tests
isolate it.

Design choice -- seed the USER override file (not a new shipped read-only seed
layer): the owner wants `X -> EXIT` VISIBLE and EDITABLE like any alias and restored
by Reset bare. Writing it into the user override store (the third resolver layer,
Requirement 5.7) achieves exactly that with the existing serialiser and reload path;
a new shipped `i18n/<locale>/aliases/FFCMD.toml` read-only seed would NOT be editable
and would not match "expose everything in config files". So the seed is a
first-run-written USER override row, rebuilt by Reset bare.

### First-run and Reset-bare hooks

Both call the shared Config_Rebuild_Helper (configuration-system design):
- First run (`construct.rs`, `Mode::FirstRunMissingOnly`): if
  `aliases/<locale>/FFCMD.toml` is ABSENT, write it with the `X -> EXIT` row BEFORE
  `seed_and_overlay_for_locale` runs, so the resolver's user-overlay layer reads it.
- Reset bare (`reset_bare.rs`, `Mode::RebuildOverwrite`): after `aliases/` is
  archived (configuration-system Requirement 19.16), rewrite the seed file.

### Resolution / coexistence (Requirement 8.5)

`X -> EXIT` lands in the FFCMD `AliasTable` via the EXISTING user-overlay layer
(Requirement 5.7). FFEDIT's base `X -> EXCLUDE` is a different `AliasTable` keyed by
a different ENV, so the two coexist (the overlay already permits the same surface in
different ENVs). The active-wins order (prelude in `commands.rs`: Active_Environment
first, then FFCMD) ensures an editor EXCLUDEs on `X` and every non-editor Context
EXITs on `X`. No change to `ffedit_english()` or any existing alias.

### Error handling

- Seed write failure at first run (Degraded_Mode / unwritable dir): skipped;
  resolver falls back to identity (`X -> X`), so bare `X` would report unknown-command
  until the dir is writable or Reset bare succeeds. Non-fatal, logged.
- User deletes the seed row via the Alias Editor: allowed (it is a normal row);
  `X` reverts to identity until Reset bare restores the seed. Documented consequence.

### Testability

- Unit: the seed-row builder produces exactly one `X -> EXIT` FFCMD row; the
  serialised file round-trips through the loader to that row.
- Integration (full-shell, temp seams): first run writes `aliases/en/FFCMD.toml`
  with `X -> EXIT`; the resolver resolves bare `X` (non-editor) to `EXIT`; an FFEDIT
  editor resolves bare `X` to `EXCLUDE`; Reset bare rebuilds the seed after
  archiving a user-edited aliases dir. Agent compile-verifies scoped; owner full gate.

---

## 9.11 Alias Workspace Kind: Content_Area + Workspace_Status_Segment (Requirement 7.14-7.28, per-Kind gate under CR-NR-115)

### 9.11.0 Overview

> **RECONCILED ON REBASE (CR-NR-116, rebased onto main fd35620).** This section
> was first authored off the old main a6aacae. Main has since gained: the common
> workspace frame's frame-owned rule between the Command line and content plus the
> grid-merge (CR-NR-115, screen-snapshot-scrm 30.29/30.30 on main, referenced here
> as 30.34/30.35), the Menu Kind box-drawing content grid + Option|Command|
> Description clipboard table (CR-NR-117), the common `Zoom:` status cell, and the
> write-through config model with the `X -> EXIT` FFCMD seed row (CR-CH-116). The
> gate's criteria, fixture, and decisions below are reconciled to those: the grid
> top rule now MERGES with the frame rule, the status segment follows
> `Last | Prof | Zoom`, the grid builds on the shared framed renderer (not a
> bespoke grid), the clipboard uses the shared `markdown_sections` mechanism, and
> the file-backed seed row is accounted for in the grid.

The common workspace frame (CR-NR-115, merged a6aacae) is shell-owned. This
section defines what the Alias Workspace owns on top of it: ONE Content_Area
(the grid) and ONE Workspace_Status_Segment, in the live panel and in the
snapshot/capture. It is the FIRST per-Kind gate, so the grid-in-frame form, the
cell conventions and the trailing scratch row are designed as SHARED pieces
(`ispf-table-editor` Requirement 12, `screen-snapshot-scrm` Requirement
30.32-30.40) and the Alias Kind contributes only data: its columns' cell metadata,
its row-to-cells mapping, its field widths, and its status cells. The decisions
below were settled by reading the code, not assumed.

### 9.11.1 Target Content_Area (normative layout, snapshot)

Rendered at the pfshow-off, no-message case so the box is easy to read (the
glyphs are runtime product output, exempt from the ASCII-only rule). With pfshow
on, the frame is as wide as the key line and the last (Description) column's
right edge is simply the frame border; nothing else changes. The data shows the
mandatory write-through `X -> EXIT` FFCMD seed row (Requirement 8, CR-CH-116;
always present and editable), the shipped SCRM `SS` alias row (empty Parameters /
Description), a row with real values, and the trailing scratch row. The Act
column shows its pending-letter text.

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                         Alias Editor                                                          │
├───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Command ===> ________________________________________________________________________________________________________________ │
├──────┬────────────┬──────────────────────┬──────────────────┬──────────────────────────────┬──────────────────────────────────┤
│ Act  │ CE         │ Command              │ Alias            │ Parameters                   │ Description                      │
├──────┼────────────┼──────────────────────┼──────────────────┼──────────────────────────────┼──────────────────────────────────┤
│ ____ │ [ffcmd v]  │ [EXIT v]             │ X                │ ____________________________ │ Exit the application             │
│ ____ │ [ffcmd v]  │ [SNAPSHOT v]         │ SS               │ ____________________________ │ ________________________________ │
│ ____ │ [ffcmd v]  │ [CAPTURE START v]    │ CS               │ &1                           │ Session Recording start          │
│ ____ │ [ffcmd v]  │ [ v]                 │ ________________ │ ____________________________ │ ________________________________ │
├──────┴──────┬─────┴──────┬────┴───────┬───┴──────────────┴──────────────────────────────┴──────────────────────────────────┤
│ Last: 12:06 │ Prof: def  │ Zoom: 100% │ Aliases: 3 | Rows 1-3 visible                                                    │
└─────────────┴────────────┴────────────┴──────────────────────────────────────────────────────────────────────────────────┘
```

Reading it: everything from the centered title to the `Command ===>` line and the
`Last:`/`Prof:`/`Zoom:` cells is the common frame (CR-NR-115). The frame draws its
OWN rule between the `Command ===>` line and the first Content_Area row
(screen-snapshot-scrm Requirement 30.34); because that first content row is the
Alias grid's header band, the grid's own top rule and the frame rule MERGE into
one rule line (CR-NR-115 grid-merge, screen-snapshot-scrm Requirement 30.35), so
there is exactly one rule between the Command line and the header - carrying the
header's tee-down junctions. The Alias Content_Area is exactly the header band,
the data bands and the scratch band with their rules. The status band's Alias
cells (`Aliases: 3`, `Rows 1-3 visible`) are APPENDED after the Common_Status_
Segments `Last:`/`Prof:`/`Zoom:` (screen-snapshot-scrm Requirement 30.10). The
first data row is the mandatory `X -> EXIT` FFCMD seed row (Requirement 8); the
fourth row is the scratch row (blank-row conventions); the `SS` row has empty
Parameters/Description because that is its real data (see 9.11.2, D5). The fixture was generated from the rules in 9.11.3 by
a throw-away script (not committed), so column widths and junctions are exact.

### 9.11.2 Investigation findings (read against the code on branch alias-kind-gate, off a6aacae)

D2 / issue 1 - why a fields block AND a grid. `screen_providers.rs::alias_editor_screen_model`
builds `model.fields` first (one `Field::new(surface, canonical[ template])` per
row - the pre-A2 representation, the `SS . . . : SNAPSHOT` line, rendered by
`render/framed.rs::collect_rows` as a blank spacer row plus `label . . . : value`),
and THEN `model.tables.push(self.alias_table())` (the A2 representation added in
Group A, `screen_providers_tables.rs::alias_table`). A2 added the table without
removing the fields, so the same row is emitted twice. The clipboard `| Property |
Value |` table comes from `render/markdown.rs::render_markdown`: when
`markdown_sections` is empty it appends a Property/Value table built from
`model.fields` (and `html.rs::render_tables_html` mirrors it). It is therefore the
SAME stale `fields` block surfacing in the structured flavor; deleting the Alias
`fields` loop removes the dot-leader line AND the Property/Value table in one move.
The Markdown path then still appends each `tables` element as a GFM table (the
deliberate Req 28 dual representation) - addressed by decision g (9.11.5).

D5 / issue 3 - why Parameters and Description are "empty". They are empty
because the working-copy rows ARE empty; nothing is lost or mis-mapped. Evidence:
(1) `alias_table()` maps `r.command_template` -> Parameters and `r.description` ->
Description (correct, Req 7.2 order); (2) the same snapshot's dot-leader line
shows `SNAPSHOT` with no template suffix, and that code appends the template
whenever `command_template` is non-empty, so the row's template really is empty;
(3) the `SS` row is a shipped SCRM alias row (its template is `None`;
`alias_reader.rs` seed -> `effective_display_rows`), distinct from the CR-CH-116
`X -> EXIT` seed row (Requirement 8); its description comes
from `user_alias_descriptions()` (empty unless a user override carries one);
(4) the live panel's `(&1)` and `note` are egui `hint_text` PLACEHOLDERS on the
empty Parameters and Description TextEdits (`alias_editor_panel/rows.rs`:
`.hint_text("(&1)")`, `.hint_text("note")`, plus `"name"` on Alias and `"_"` on
Act). They are rendered in a faint colour and are not values. The snapshot is
faithful; the live panel only LOOKS populated. Consequence for the requirements:
placeholders are never snapshot content (7.19, ispf-table-editor 12.4), and an
empty text cell is `____` (7.20). If the owner did store a real template on a row,
the same `command_template` field feeds both widget and snapshot, so it would
appear; a quick confirmation is to type `&1` into Parameters and snapshot again.
Round-trip coverage of the description already exists
(`tests_alias_editor.rs`, save/reload with `description`).

Cell conventions are NOT applied by the provider today. The `[value v]` / `[ v]` /
`____` strings exist only in `ff-screen-model` renderer tests (`text.rs`
`alias_like_table`) and in Req 29.8/29.9; `alias_table()` emits raw strings
(`ffcmd`, empty), which is why the owner capture shows bare `ffcmd` and blank
cells. The conventions must therefore be implemented in the PROVIDER side (shared
helpers, 9.11.3), not the renderer.

Issue 2 - why a nested box. In the framed path (`render/framed.rs::collect_rows`)
each `model.tables` entry is emitted as `Row::Text("")` (blank spacer),
`Row::Text(caption)` (the `Aliases` heading) and then the lines of
`text.rs::render_table_grid` as plain text rows. `render_table_grid` draws a
complete box (top/mid/bottom rules, corners) inside a content row, hence the
`┌─┬─┐` box nested in the frame, with the frame's right border far to its right.
The frame's own band machinery (`Row::Band` cells + `Row::Rule` with tee/cross
junctions computed from the neighbouring band's dividers - already used for the
menu bar, tab bar and status bar) is not used for content.

Issue 7 - stray rows. Same cause: the blank spacer before `fields`, the blank
spacer and caption before each table. Removing `fields` and drawing tables as
bands removes all three.

Scratch row (issue 5). The live scratch row is NOT always present. In
`shell/ispf_table_editor/render.rs::render_ispf_table` the scratch row is
rendered only `if rows.is_empty()` (ispf-table-editor Req 4.6); a list with one or
more rows shows exactly those rows. The owner's mental model (and screen-snapshot
Req 29.7) is an always-present trailing blank row. Mirroring live literally would
give NO scratch row in the owner's SS example. Decision f resolves this.

Filter row. `render_ispf_table` draws the header, then a FILTER row (the host's
`render_filter_cells`: CE and Root comboboxes showing `--` when unset), then a
separator. These are input controls whose unset state carries no data.

Status data sources. `workspace_status_cells()` (`render_status.rs`) is the single
per-Kind seam read by BOTH the live bar (`render_status_bar` loop) and the
snapshot (`common_frame.rs::common_frame_status_bar_text`); it currently returns
an empty Vec for every Kind except FileEditor/Untitled. Alias data: the working
rows are `self.alias_editor_panel.rows`; the filtered set is
`alias_editor_panel::filter::displayed_indices(rows, filter_ce, &filter_root)`;
the active filter is `filter_ce` / `filter_root`. No dirty/baseline tracking
exists on `AliasEditorState`, so a `Modified` cell is NOT specified (a follow-up
would need a baseline snapshot of the rows).

### 9.11.3 Design

Technology: Rust, existing crates (`ff-desktop`, `ff-screen-model`), no new crate
or dependency, egui untouched in the provider (logical state only, Req 29.13).

Shared pieces (owned by `ispf-table-editor` / `screen-snapshot-scrm`, designed in
their design.md):
- `ff-screen-model` framed renderer lays `model.tables` out as frame-ruled bands
  (30.32-30.39).
- `shell/ispf_table_editor/snapshot.rs` (new, pure): `CellKind {Text, Combo}`,
  `combo_cell`, `text_cell`, `sanitise_cell`, and
  `snapshot_table(columns, rows, scratch) -> Table` / `raw_table(...)`.
  `ColumnSpec` gains `cell: CellKind` and `snapshot_width: usize` with builder
  `with_snapshot(kind, width)` (const; default Text, header width).
- The trailing scratch row in `render_ispf_table` (12.5).

Alias-specific pieces:
- `alias_editor_panel/host.rs`: `ALIAS_COLUMNS` declares the cell metadata:

  | Column | CellKind | snapshot_width | Raw value source |
  |--------|----------|----------------|------------------|
  | Act | Text | 4 | `row.act` |
  | CE | Combo | 10 | `row.environment.label()` |
  | Command | Combo | 20 | `row.canonical` |
  | Alias | Text | 16 | `row.surface` |
  | Parameters | Text | 28 | `row.command_template` |
  | Description | Text | 32 | `row.description` |

  The widths follow the live widget proportions (28/84/96/120/260/180 px) and the
  owner's sketch; they are minimums - a longer value widens, never truncates.
  A new `alias_snapshot_cells(&AliasRowEdit) -> Vec<String>` returns the six RAW
  values in that order (used for data rows, the scratch row via
  `AliasRowEdit::default()` - CE `ffcmd` because `AliasEnvironment` derives
  `Default` = Ffcmd - and the raw markdown section).
- `shell/screen_providers_tables.rs::alias_table()`: builds the displayed set with
  `displayed_indices(&p.rows, p.filter_ce, &p.filter_root)`, maps rows through
  `alias_snapshot_cells`, appends the scratch cells, and calls the shared
  `snapshot_table(...)`; `caption` stays `Some("Aliases")` for the structured
  flavors. A sibling `alias_markdown_section()` builds the RAW section.
- `shell/screen_providers.rs::alias_editor_screen_model()`: delete the `fields`
  loop; push `alias_table()` and the markdown section; keep `p.error` ->
  `model.messages`. (`command_line` is set by `apply_common_frame` afterwards.)
- `shell/workspace_status_alias.rs` (new, <400 lines): `alias_status_cells(&self)
  -> Vec<String>`; `render_status.rs::workspace_status_cells()` gains a
  `KindTag::AliasEditor => return self.alias_status_cells()` arm before the
  FileEditor/Untitled early return. New `ff_i18n` keys follow
  `ff-desktop-shell-status-*` (English `Aliases: { $count }`, `Rows 1-{ $shown }
  visible`, `Rows 0 visible`, `Filter: { $ce }/{ $root }`).

Live panel: no visual change except the trailing scratch row (12.5), which lives
in the shared surface. The live Alias widgets, ids and the first-Tab target (filter
CE combo) are unchanged.

### 9.11.4 The status segment

`Aliases: <n>` = `rows.len()` (scratch excluded). `Rows 1-<m> visible` = number of
rows passing the filter - the LOGICAL visible set. The owner's phrase "visible" is
honoured, but the snapshot is egui-free (29.13) so it cannot know the scroll
viewport; with no scroll state the range is always `1-<m>` and its information
content is the filtered count (differs from `Aliases` only when filtered). The
third Alias cell `Filter: ffcmd/--` appears only while a filter is active and is
how a filtered snapshot announces itself (decision b). Cells contain no ` | `. The
Alias cells are APPENDED after the Common_Status_Segments `Last:`/`Prof:`/`Zoom:`
(CR-NR-115, screen-snapshot-scrm Requirement 30.10): `Zoom` is now a common cell
owned by the shell, not an Alias cell. Example bands: unfiltered `Last: 12:06 |
Prof: default | Zoom: 100% | Aliases: 2 | Rows 1-2 visible`; filtered `... | Zoom:
100% | Aliases: 40 | Rows 1-6 visible | Filter: ffcmd/SNAPSHOT`.

### 9.11.5 Clipboard flavors (decision g)

`render_markdown` with a non-empty `markdown_sections` emits the fenced box then
the sections and nothing else; `render_tables_html` does the same. So the Alias
model sets ONE section `### Aliases` with the RAW six-column table. Result in the
clipboard: box first (with cell conventions, scratch row, status), then
`### Aliases` + a GFM table of real values (no `[v]`, no `____`, no padding, no
scratch row), and no Property/Value table. If the owner prefers the structured
table to mirror the box strings, drop the section and let `tables` feed the
structured flavors (they would then carry `[ffcmd v]`/`____`/the scratch row, and
29.7a stays literally true). Interaction noted: 29.7a (HTML scratch `<tr>`) holds
only for models WITHOUT `markdown_sections`; 30.40 records this.

### 9.11.6 Error handling (concrete)

- `AliasEditorState.error` present: recoverable; pushed to `model.messages`
  (Message_Line band), NOT logged by the provider (the validation path already
  logs); the grid is still rendered.
- Row with control characters in any field: provider replaces tab/CR/LF with a
  space (`sanitise_cell`); no error, no log.
- Value containing ` | `: kept inside its cell (cells are discrete strings, 30.36).
- Value longer than the field width: column widens (frame widens if needed); never
  truncated; no error.
- Zero working rows: the grid shows the header and the scratch row only; status
  `Aliases: 0` / `Rows 0 visible`.
- Filter hides every row: grid shows header + scratch row; `Rows 0 visible`,
  `Filter:` cell present.
- A model with `tables` but a broken column count (row shorter than headers):
  the shared builder pads missing cells with empty-convention strings; a longer
  row is truncated to the header count (defensive; callers build both from the
  same column list so it should not happen).
- Status cells cannot fail (pure reads of in-memory state); `ff_i18n` missing key
  falls back to the key text per the existing i18n behaviour.

### 9.11.7 Input validation

The provider reads only in-memory working-copy state; there is no external input
at snapshot time. The only user-controlled text is alias field values, handled by
`sanitise_cell` (control chars -> space), no length limit (widen-on-overflow),
and `Filter:` echoing `filter_root` (a value chosen from the registry's root list
or empty; sanitised the same way). Markdown cell escaping of `|` and newlines is
the existing `escape_cell` in `render/markdown.rs`.

### 9.11.8 Framework verdict (framework-conformance.md)

BUILD ON existing seams; NO framework change; nothing here reshapes a public
framework type.
- Dispatch, navigation, descriptor, persistence, save path: untouched.
- `WorkspaceContext` / `InteriorFocus`: untouched; the filter CE combo stays the
  first interior stop; the scratch row is appended LAST in the Tab/arrow ring.
- Common frame (mechanism 8): not reshaped. The Alias Kind fills only the
  Content_Area and its status cells, via the existing `workspace_status_cells`
  seam (which needs no new trait: it is a `match` arm on a shell method).
- `ScreenModel` / `ScreenProvider` / `Table` / `MarkdownSection`: public shapes
  UNCHANGED. Tables carry the cell strings; `markdown_sections` is an existing
  field with precedent (Config panel).
ADDITIVE items flagged for the owner at the CONFIRM step (none is a framework
change, but each is behaviour/shared-code reach):
1. Framed renderer draws ALL Contexts' `model.tables` in frame rules (Theme, Menu,
   Keys, Kinds snapshots change appearance; their goldens/tests are regenerated in
   the implementation change). Renderer-internal, `ff-screen-model` public API
   unchanged.
2. Grid-merge under the frame rule (30.35, building on main's CR-NR-115 30.34 /
   30.30): the frame draws its OWN rule between the Command line and the first
   content row, and when that first row is the grid header the grid's top rule
   MERGES with the frame rule into one line carrying the header junctions. (This
   reconciles the gate's earlier "no frame-added rule, grid rule sits directly
   under the Command line" wording with the common-frame rule that landed on main
   since: the single rule the owner sees is now the MERGED rule, not a
   frame-suppressed one.)
3. Live behaviour change: trailing scratch row for NON-empty lists (12.5), shared
   by every table editor.
4. `ColumnSpec` gains two fields (ff-desktop-internal struct, `Copy`, const
   builders; the sole current host is Alias).
5. Markdown/HTML-table flavors use `markdown_sections` (raw values).

### 9.11.9 Decision points (recommendations)

(a) Combo marker in header vs per cell. RECOMMEND per cell (`[ffcmd v]`,
`[SNAPSHOT v]`, `[ v]`), header plain. Per-cell is the already-approved Req
29.8/30.13 convention, marks exactly which cells are dropdowns (an empty combo
`[ v]` is distinguishable from an empty text cell `____`, which a header marker
cannot express per row), and is identical in the HTML flavor. The sketch's `CE   v`
header marker plus `FFCMD__` cells loses the empty-combo distinction and
re-uses `_` fill (an edit-field cue) on a non-edit control. Both CE and Command
are combos in the live panel, so both get the marker (the sketch shows Command
unmarked; live shows a combo).
(b) Filter row in the snapshot. RECOMMEND omit the row; report an ACTIVE filter as
the `Filter: <ce>/<root>` status cell. It is an input, an unset filter is `--`
noise (the stray-row complaint), and an extra non-data row in `model.tables` would
need a row-type the Table model does not have (reshape). The status cell keeps the
information that matters (the list is filtered).
(c) Grid header rule directly under the Command line. RECOMMEND yes (owner
sketch), reconciled to the common frame that landed on main since this gate was
authored: CR-NR-115 (main 30.29/30.30) now has the SHELL draw a frame-owned rule
between the `Command ===>` line and the first Content_Area row, and MERGE it with
a grid's own top rule when the first content row is a grid. So the single rule the
owner sees under the Command line is the MERGED frame+grid rule (screen-snapshot-
scrm 30.34/30.35), produced by the same rule/junction machinery as the
menu/tab/status bands and carrying the header's tee-down junctions. (BASIS CHANGED
BY REBASE: the gate's original wording was "no frame-added rule, the rule is the
grid's own"; the frame now owns that rule and merges it.)
(d) Status wording. RECOMMEND `Aliases: <n>` and `Rows 1-<m> visible` (owner's
words), plus the conditional `Filter: <ce>/<root>` cell, APPENDED after the common
`Last:`/`Prof:`/`Zoom:` cells. Sources in 9.11.4. No `Modified` cell (no baseline
tracking exists). (BASIS CHANGED BY REBASE: since CR-NR-115 landed, `Zoom:` is a
COMMON status cell owned by the shell, so the Alias segment now follows
`Last | Prof | Zoom`, not `Last | Prof`; the Alias Kind still contributes only
`Aliases`/`Rows`/`Filter`.)
(e) Value fidelity. RECOMMEND mirror live VALUES exactly as stored (`ffcmd` not
`FFCMD`); there is no `(&1)` value to mirror - it is a placeholder (D5). The
sketch's `&1` / `FFCMD` were illustrative data. If a template `(&1)` is typed, it
appears as `(&1)`. (BASIS CHANGED BY REBASE: since CR-CH-116 the FFCMD alias store
is file-backed and seeded with `X -> EXIT` on first run, so the Alias grid now
ALWAYS contains at least that seed row - `Act ____`, `CE [ffcmd v]`, `Command
[EXIT v]`, `Alias X`, empty Parameters, its seed Description - and the value
mirrored for it is exactly the stored seed, same as any user row. The store is no
longer empty-until-the-user-adds.)
(f) Scratch row. RECOMMEND make the LIVE panel show the trailing scratch row for
non-empty lists too (12.5) so snapshot = live = owner's mental model. The
alternative (snapshot-only scratch row) breaks the mirror-live rule; the other
alternative (no scratch row unless empty) contradicts Req 29.7 and the owner's
issue 5. This is the one criterion that changes live behaviour of every table
editor, hence flagged.
(g) Clipboard structured table. RECOMMEND keep the `### Aliases` GFM table (Req 28
dual-flavor, box first) but with RAW values via `markdown_sections`, and no
Property/Value table. Alternative: structured flavors mirror the box strings.

### 9.11.10 Shared vs Alias-specific

| Concern | Shared (ispf_table_editor / ff-screen-model) | Alias-specific |
|---------|----------------------------------------------|----------------|
| Grid in frame rules, junctions, widths, no caption/spacer | framed.rs table bands (30.32-30.39) | none |
| Cell helpers + conventions | `ispf_table_editor/snapshot.rs` | none |
| Column cell kind + width metadata | `ColumnSpec` fields + builder | the six values in `ALIAS_COLUMNS` |
| Row -> cells | `snapshot_table` / `raw_table` builders | `alias_snapshot_cells` |
| Trailing scratch row (live) | `render_ispf_table` (12.5) | `AliasRowEdit::default()` baseline |
| Hint text excluded | helper API has no hint param | n/a |
| Filter row omitted + status cell | rule (12.6) | `Filter:` cell wording |
| Status cells | `workspace_status_cells` seam | `alias_status_cells` + i18n keys |
| Clipboard raw section | `markdown_sections` mechanism | `alias_markdown_section` |

Menu, Kinds, Keys and Theme inherit everything in the Shared column; each declares
only its Alias-style column.

### 9.11.11 Testability

- Unit, pure (ff-screen-model): framed table bands - junction glyphs, widths,
  coalesced rules, no caption/spacer, ASCII fallback, cell with ` | ` stays in its
  cell; golden regeneration for `alias_*` fixtures.
- Unit, pure (ff-desktop): `combo_cell` / `text_cell` / `sanitise_cell`;
  `alias_snapshot_cells` (placeholder text absent, empty -> underscores, filled
  padded, non-truncating); filtered rows in backing order; scratch row last;
  `alias_editor_screen_model` has NO `fields`; `markdown_sections` raw;
  `alias_status_cells` (count, filtered `Rows`, `Filter:` only when active).
- Existing tests to update: `tests_editor_snapshot_tables.rs`,
  `tests_screen_directive.rs`, `tests_common_frame_snapshot.rs`,
  `tests_common_frame_order.rs`, `tests_common_frame_helper.rs`, the three alias
  goldens.
- egui_kittest: trailing scratch row present for a non-empty list; typing in it
  promotes it and a new scratch appears; first-Tab test unchanged (still the
  filter CE combo).
- Not drivable headlessly: pixel appearance of the live scratch row (MANUAL, owner
  eyeball; the logical presence is test-covered).

### 9.11.12 Files expected to change in the implementation step (this gate edits docs only)

`crates/ff-screen-model/src/render/framed.rs` (+ tests file),
`crates/ff-desktop/src/shell/ispf_table_editor/{state.rs,render.rs,snapshot.rs}`,
`crates/ff-desktop/src/alias_editor_panel/host.rs`,
`crates/ff-desktop/src/shell/{screen_providers.rs,screen_providers_tables.rs,
render_status.rs,workspace_status_alias.rs}`, i18n catalogue, the three goldens and
the tests listed in 9.11.11. Each stays under 400 non-test lines.
