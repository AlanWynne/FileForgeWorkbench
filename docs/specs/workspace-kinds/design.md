# Design Document -- Configurable Workspace Kinds (CR-NR-090)

## 1. Overview

Make a Workspace Kind's presentation + profile DATA (a `Kind_Config`) resolved
through a `Kind_Registry`, and let a user create a new Kind "modelled on" a
built-in base. Single-layer (no Def layer): runtime stays Kind -> live Workspace
tab. This document covers Slice B.1 (model + registry + built-in defaults +
title-from-config); B.2-B.4 design deltas are added at their gates.

## 2. Where it lives

- New module `crates/ff-desktop/src/workspace_kind/` (or a small `ff-workspace-kind`
  helper module) owning `KindConfig`, `BaseKind`, `KindProfile`, and `KindRegistry`.
  Chosen home: `ff-desktop` (the shell owns TabKind and the registry consumers);
  no new library crate for B.1. Revisit if B.4 needs sharing.
- User Kind files: `<User_Data_Dir>/workspace-kinds/<name>.toml`.

## 3. Data model (B.1)

```rust
/// What a Kind is modelled on. Open model: Builtin resolvable in v1; External is
/// schema-accepted but unresolved in v1 (future Lua/REXX kinds).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseKind {
    Builtin(BuiltinKind),  // BuiltinKind mirrors the compiled TabKind set by stable name
    External(String),      // future: e.g. "ext:my-lua-kind"; unresolved in v1
}

/// Profile attributes carried by a Kind (extensible).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindProfile {
    pub edit_profile: EditProfileDefaults, // CAPS/NULLS/STATS/LOCK/HILITE defaults
    pub tab_size: u8,
    pub line_end_mode: LineEndModeName,
    // extensible: theme override, default view/edit mode, working dir ... (future)
}

/// The configurable record for a Workspace Kind (built-in default or user file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindConfig {
    pub name: String,             // stable id (built-in: "editor"...; user: "mainframe-editor")
    pub modelled_on: BaseKind,    // built-in base in v1 (built-in Kind: Builtin(self))
    pub title: String,            // tab / Title_Line label, e.g. "[CATALOGS]"
    pub menu_bar: Option<String>, // named menu; None => base default (B.2 wires it)
    pub key_list: Option<String>, // named keymap; None => base default (B.2 wires it)
    pub profile: KindProfile,     // applied on open (B.3 wires it)
}
```

Serde/TOML: `modelled_on` round-trips as a string tag -- `"editor"` for
`Builtin(Editor)`, `"ext:<name>"` for `External("<name>")` -- so the on-disk
schema is stable and external-ready. (A dedicated `#[serde]` adapter maps the
string form to/from `BaseKind`.)

`BuiltinKind` is a small enum keyed by the SAME stable names as
`context_name_for_kind` today (`editor`, `files`, `config`, `search`, `plugins`,
`log`, `macros`, `menu`, `commands`, `pom`, `theme`, `menus`, `keys`), so it maps
1:1 to `TabKind` (+ the `pom`/`menu` split the shell already makes via `is_home`).
The registry maps a Kind name -> `BuiltinKind` -> the existing open/dispatch path;
no command routing changes in B.1.

## 4. Kind registry (B.1)

```rust
pub struct KindRegistry { /* built-in defaults + loaded user Kinds, by name */ }

impl KindRegistry {
    pub fn with_builtin_defaults() -> Self;      // compiled defaults for every built-in
    pub fn load(user_dir: &Path) -> Self;        // built-ins + user files (user overrides built-in by name)
    pub fn effective(&self, name: &str) -> &KindConfig; // always resolves (falls back to a built-in)
    pub fn resolve_base(&self, name: &str) -> BuiltinKind; // single-hop base resolution
}
```

- Built-in defaults are compiled (a `builtin_kind_configs()` table), so the
  registry is never empty and RESET BARE (B.4) restores from it.
- User files override a built-in by matching `name` (CR-CH-021 pattern);
  unparseable user files are skipped with a non-blocking notice; a user Kind
  whose `modelled_on` is another user Kind (chain) is rejected-with-message
  (Req 1.3); an `External` base that cannot resolve falls back to a safe built-in
  with a notice (Req 1.2).
- B.1 loads the registry once at startup and holds it on the shell (like
  `command_store`); hot-reload is a B.4 concern.

## 5. Title derived from the Kind config (B.1)

The shared title derivation added in CR-CH-034/B050 (`title_line_text` +
`render_tab_bar`'s base-title helper) currently returns hard-coded per-`TabKind`
bracket labels for panel Kinds. B.1 routes those through the registry:

- Add `WorkbenchShell::kind_title(tab) -> String` that returns the active tab's
  effective `KindConfig.title` from the registry (keyed by the tab's Kind name;
  for a Menu Workspace it stays the loaded-menu-derived label per Req 17.10, and
  Home stays the POM banner / `[POM]`).
- `title_line_text` and the tab-bar header helper call `kind_title` for the panel
  Kinds instead of the hard-coded strings. `workspace_name` precedence and the
  editor path are unchanged.
- Because the label is recomputed from the registry each render, a reconfigured
  Kind's title updates live (Req 17.10 consistency).

This is the B.1 user-visible payoff: it fixes the Catalogs/`[FILES]` smell (the
Catalog Explorer Kind's compiled default title becomes `[CATALOGS]`).

NOTE: `title_line_text` is currently a free function taking only `&TabState`; to
read the registry it must either become a `&self` method or take the registry.
The design makes it a shell method (`kind_title`) and has `title_line_text`
delegate, keeping a thin free-function shim for the existing unit tests that call
it without a shell (those assert built-in defaults, which the shim can supply
from `builtin_kind_configs()` without a full shell).

## 6. Correctness properties (B.1)

- **P1 Registry totality:** `effective(name)` returns a `KindConfig` for ANY
  input (built-in name, known user name, or unknown -> a safe built-in default);
  never panics.
- **P2 Single-hop resolution:** `resolve_base(name)` terminates in one hop; a user
  Kind resolves to a `Builtin(...)`, never to another user Kind.
- **P3 Title round-trip:** a `KindConfig` written to TOML and re-read yields an
  equal record, including `modelled_on` in both `Builtin` and `External` forms.
- **P4 Distinct built-in titles:** the compiled defaults give Catalog Explorer and
  File Explorer DISTINCT titles (`[CATALOGS]` vs `[FILES]`).

## 7. No behaviour change beyond titles (B.1)

B.1 changes ONLY the title source (data-driven) and adds the model/registry.
Command dispatch, central-panel rendering, navigation, keymaps, and menu bars are
UNCHANGED in B.1 (menu bar + key list are wired in B.2; profile in B.3; the dialog
in B.4). Every existing title/keymap/menu test continues to pass (title tests
updated only where a built-in default label is intentionally corrected, e.g.
Catalogs -> `[CATALOGS]`).

## 8. External-ready (future, not built in B.1)

`BaseKind::External(name)` is accepted by the schema and the model but has no
resolver in v1. When Lua/REXX Kind providers land, a resolver maps an external
base to its command/behaviour; nothing in the B.1 schema or registry API changes.
This is the owner's "build for built-in, flexible for external later".

## 9. Slice B.2 delta -- per-Kind menu bar + key list

B.2 wires the two presentation fields the B.1 `KindConfig` already carries
(`menu_bar`, `key_list`) into the two existing resolver seams. No new model.

### Menu bar seam

`render_chrome.rs::resolve_menu_bar_menu(&self)` today always resolves the single
`DEFAULT_MENU_BAR_NAME` ("MB-POM"). B.2:

```rust
fn resolve_menu_bar_menu_for(&self, tab: &TabState) -> MenuFile {
    let kind_name = BuiltinKind::from_tab_kind(tab.kind, tab.is_home).stable_name();
    let bar_name = self.kind_registry.effective(kind_name).menu_bar
        .clone()
        .unwrap_or_else(|| DEFAULT_MENU_BAR_NAME.to_string());
    let slug = menu_bar_slug(&bar_name);
    load_menu_file(menus_dir/<slug>.toml).unwrap_or_else(|_| default_menubar_menu())
}
```

`render_menu_bar` (primary) passes the ACTIVE tab; `render_detached_menu_bar`
passes the detached tab (it is called inside the CR-CH-036 swap, so "active" IS
the detached tab -- it can keep calling the active-tab form). The existing
`resolve_menu_bar_menu()` becomes a thin wrapper over `_for(active_tab)` so no
call site outside changes. Fallback to the compiled default bar is unchanged
(Req 4.1). Behaviour-preserving because every built-in Kind's default
`menu_bar` is `None` -> `DEFAULT_MENU_BAR_NAME` (Req 4.5).

### Key list seam

The active key map is chosen by a context NAME fed to
`KeyMapResolver::set_context(name)` (render_chrome.rs sets it on tab activation;
the resolver picks `context_maps[name]` else the global map). Today the name is
`context_name_for_tab(tab)` (the base kind context / `pom`). B.2 introduces:

```rust
fn key_list_context_for_tab(&self, tab: &TabState) -> Option<String> {
    let kind_name = BuiltinKind::from_tab_kind(tab.kind, tab.is_home).stable_name();
    let cfg = self.kind_registry.effective(kind_name);
    cfg.key_list.clone()                      // Kind's configured key list, if any
        .or_else(|| context_name_for_tab(tab).map(str::to_string)) // else base context
}
```

Every site that calls `set_context(context_name_for_tab(tab))` (tab activation in
render_chrome.rs, plus the navigate/START focus paths) routes through
`key_list_context_for_tab` instead. A `key_list` naming a context with no loaded
`keymaps/<name>.toml` map falls back to the global map via the resolver's
existing precedence (Req 4.3) -- no new failure path. Behaviour-preserving
because built-in defaults have `key_list = None` -> the base context name, i.e.
today's behaviour (Req 4.5).

### "Modelled on" inheritance (Req 4.4)

A user Kind with `menu_bar: None` / `key_list: None` resolves to
`DEFAULT_MENU_BAR_NAME` / its base context name -- i.e. the SAME as its base Kind
(since the base also leaves them unset by default). An override on the user Kind
wins. So inheritance is "unset => same as base; set => override", achieved purely
by the `unwrap_or(base default)` in both seams; no explicit base-merge needed for
these two fields in B.2.

### Testability (B.2)

Headless: assert `resolve_menu_bar_menu_for(tab)` picks the Kind's configured bar
(and the default when unset); assert `key_list_context_for_tab(tab)` returns the
Kind's `key_list` when set and the base context when unset; a full-shell test
that switching to a Kind with a configured bar renders that bar's top-level
items. The real menu-bar visual + a live keystroke remain covered by existing
menu-bar / keymap tests (behaviour-preserving for unset Kinds).

## 10. Slice B.3 delta -- per-Kind profile attributes applied on open

B.3 applies the `KindProfile` (from B.1) to a newly-opened Workspace. The
applicable per-tab attributes are the ones that HAVE a per-tab home on
`TabState`: `edit_profile: EditProfile` and `line_end_mode: LineEndMode`.
`tab_size` has no per-tab field (it is the global `editor.tab_size` config key),
so it is carried in the schema (dialog-editable in B.4) but NOT applied to a
per-tab value in B.3 (Req 5.3, documented deferral).

### Application seam

Add `WorkbenchShell::apply_kind_profile_to_active(&mut self)`:

```rust
fn apply_kind_profile_to_active(&mut self) {
    let kind_name = { let t = self.tabs.active_tab();
        BuiltinKind::from_tab_kind(t.kind, t.is_home).stable_name() };
    let profile = self.kind_registry.effective(kind_name).profile.clone();
    let tab = self.tabs.active_tab_mut();
    match tab.kind {
        TabKind::Untitled => {                      // new buffer: edit profile + line-end default
            tab.edit_profile = profile.edit_profile.clone();
            tab.line_end_mode = line_end_from_name(&profile.line_end_mode);
        }
        TabKind::FileEditor => {                     // loaded file: edit profile only; keep detected line-end
            tab.edit_profile = profile.edit_profile.clone();
        }
        _ => {}                                      // non-editor Kinds: edit profile is irrelevant
    }
}
```

`line_end_from_name("default"|"unicode")` maps to `LineEndMode` (default for any
other value). Called ONCE right after a tab is created and made active, at the
shell open seams:
- `update.rs` frame handlers: `pending_open` (file open), `pending_new_file`
  (new untitled), and the `cli_files` startup-open loop;
- the search-result "OpenMatch" open (render.rs) and the toolchain diagnostic
  open reuse `open_file` -> also apply.

To keep it to ONE call site per open rather than scattering it, wrap the two
TabManager creators used by the shell in thin shell methods
`shell_open_file(path)` and `shell_new_untitled()` that call the TabManager
method then `apply_kind_profile_to_active()`, and route the shell's open sites
through them. (Direct `self.tabs.open_file` calls in the shell are replaced by
`self.shell_open_file`.) This gives a single, testable application point and
avoids per-call-site duplication.

Applying ONCE at open (not on activation) means a user's later per-tab toggle
(`CAPS OFF`) is preserved (Req 5.1). Loaded files keep their detected
`line_end_mode` (Req 5.2). Built-in Kinds carry the neutral default profile, so
opening is behaviour-preserving (Req 5.4/5.5).

### Testability (B.3)

Headless: a Kind whose profile sets `CAPS On` -> a new/opened editor tab of that
Kind starts with `edit_profile.caps == On`; a subsequent `CAPS OFF` is not
re-clobbered on the next frame; a NEW buffer of a Kind with `line_end_mode =
"unicode"` opens Unicode while a LOADED file keeps its detected mode; a built-in
Kind (default profile) opens with `EditProfile::default()` (unchanged).

## 11. Slice B.4 delta -- Kinds Editor Context + command + Settings + RESET BARE

B.4 makes B.1-B.3 user-facing. Modelled directly on the Keys Workspace
(`keys_editor_panel` + `shell/keys_editor.rs`): a pure-render Context that stashes
an action the shell applies, migrated to the `WorkspaceContext` trait.

### New Context

- `TabKind::KindsEditor` (new variant) + `BuiltinKind::KindsEditor` (stable name
  `kinds`, title `[KINDS]`); its compiled default `KindConfig` is added to the
  built-in table (Req 6.6). `from_tab_kind`/`context_name_for_kind`/
  `title_line_text` gain the arm (title now via the registry, so only the
  built-in default table + the TabKind mapping change).
- `crate::kinds_editor_panel` module: `KindsEditorState` (selected kind name,
  the editable `KindConfig` working copy, the list of kind names, a "new kind"
  sub-form: name + base selector, `first_interior_id`/`last_interior_id`,
  `pending_action: KindsEditorAction`), a pure `render(ui, &mut state) ->
  KindsEditorAction`, and `impl WorkspaceContext` returning `InteriorFocus`.
- `KindsEditorAction`: `None | SelectKind(name) | EditField(...) | NewKind{name,
  base} | Save`.

### Shell wiring (`shell/kinds_editor.rs`)

- `open_kinds_editor()`: refresh the kind list from the registry, default the
  selection, transform-in-place on Home else open/activate a `KindsEditor` tab
  (same pattern as `open_keys_editor`).
- `apply_kinds_editor_action(action)`: `Save` -> serialise the working
  `KindConfig` to `KindConfigToml`, write `<workspace-kinds>/<name>.toml`
  (a `workspace_kinds_dir()` resolver with a test override mirroring
  `menus_dir_override`), then reload `self.kind_registry` from that dir so the
  change is live (Req 6.3). `NewKind{name, base}` -> seed the working copy from
  the base's compiled default with the new name + `modelled_on = Builtin(base)`.
- `KINDS` command arm in `handle_command` -> `open_kinds_editor()` (Req 6.4,
  command parity). A Settings Recovery_Baseline option row (e.g. `W` -> `KINDS`)
  dispatches the same command (menu == typed).
- Central-panel dispatch arm for `TabKind::KindsEditor` (owned-panel swap through
  `render_workspace_context`, like Theme/Menus/Keys), draining `pending_action`
  into `apply_kinds_editor_action`.

### Persistence dir

`workspace_kinds_dir()`: test override else `<User_Data_Dir>/workspace-kinds/`
(mirrors `menus_dir`/`keymaps` resolvers). The B.1 startup registry load and the
B.4 save/reload both use it.

### RESET BARE (Req 7)

- `reset_in_memory_to_baseline` (shell/reset_bare.rs) also does
  `self.kind_registry = KindRegistry::with_builtin_defaults()` (drop user
  overrides -> compiled defaults, Req 7.1/7.3).
- The profile-archive step (configuration-system Req 19.5) already moves the
  profile's data dirs; add `workspace-kinds/` to the archived set so user Kind
  files are archived, not deleted (Req 7.2).

### Testability (B.4)

Headless: `open_kinds_editor` sets the active tab kind to `KindsEditor`;
`apply_kinds_editor_action(Save)` writes the file and the reloaded registry
reflects the edited title (assert via `kind_registry.effective`); `NewKind` seeds
a copy modelled on the base; a full-shell first-Tab test (workspace-conformance
rule) that the Kinds Editor reports its first interior control; RESET BARE resets
the registry to built-in defaults. The dialog's visual layout is MANUAL only for
pixel-exact appearance (the behaviour -- selection, edit, save, reload, focus --
is harness-tested).

### Slicing note

B.4 is the largest slice. It MAY be delivered as B.4a (the Kinds Editor Context +
KINDS command + Settings entry + Save/reload + New-Kind) and B.4b (RESET BARE
integration), gated together but committed/reviewed in two steps if it aids
review. The requirements (Req 6, 7) and tasks below cover the whole slice.
