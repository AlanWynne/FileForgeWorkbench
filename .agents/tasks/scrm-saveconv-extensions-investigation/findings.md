# Investigation Findings -- Extensions Spec, Save Convention, SCRM

Read-only investigation. No source changed, no gate run. All paths absolute from
the repo root `c:/workspace/apps/FileForgeWorkbench`. ASCII-only per
documentation-charset steering.

## Summary answer (read this first)

1. EXTENSIONS SPEC -- a near-complete keystone spec ALREADY EXISTS at
   `docs/specs/extension-integration/` (CR-CH-074). It has `requirements.md`
   (11 requirements + NFRs), `design.md`, and `tasks.md`. It is DRAFTED and
   GATED-NOT-STARTED (every task unchecked). It consolidates and supersedes
   `docs/specs/plugin-architecture/` and `docs/specs/plugin-manager-ui/` (the
   `ff-plugin` crate is BUILT + unit-tested in isolation; the Plugin/Extension
   Manager Context is BUILT but has no live registry behind it). It is NOT yet a
   "complete Extensions Requirements Specification" for the user's broader intent:
   it defines how an extension plugs into the framework but does NOT yet fold in
   the cross-cutting "every screen must comply" contracts (save convention,
   snapshot/capture). That folding-in is the main missing piece.

2. SAVE CONVENTION -- defined in `.kiro/steering/save-convention.md` (B081) and
   as the central requirement `docs/project-management/project-master/requirements.md`
   Requirement 11 (the centralized document ALREADY EXISTS). Five editable
   Contexts are COMPLIANT on main (Alias, Theme, Menus, Keys, Kinds via
   `crates/ff-desktop/src/shell/editable_context.rs`). The unmerged branch
   `config-save-convention` (1 commit, `0bcddf3`) brings Config/Settings into
   compliance. The ONLY remaining non-compliant editable Context after that
   branch merges is the Command Configurator.

3. SCRM -- substantially BUILT in-tree (crates `ff-screen-model`, `ff-scrm`, and
   `crates/ff-desktop/src/screen_snapshot.rs` + `shell/commands_scrm.rs`), with a
   comprehensive spec at `docs/specs/screen-snapshot-scrm/` (CR-NR-098, 22
   requirements). The spec already covers most of the user's desired end-state
   (menu-bar surfacing Req 22, auto-capture Req 9, replay Req 10, PDF/protected
   PDF Req 12/20, command+screen recorder Req 9.10). The real GAPS are: (a)
   snapshot captures a TRIMMED model (menu options only), not the full screen
   chrome; (b) capturable Contexts are limited to Menu_Workspaces, which is the
   root of "cannot be captured yet"; (c) no bitmap/picture snapshot format and no
   format-picker; (d) no Save-As dialog on capture STOP; (e) TOML-driven SCRM
   menu family / bare-POM SCRM entry not yet added. ANSI and TEXT output are
   EFFECTIVELY IDENTICAL today for menu Contexts -- confirmed in code.

---

## Concern 1 -- Extensions Requirements Specification

### What exists and where

- `docs/specs/extension-integration/requirements.md` -- the keystone contract.
  Requirements 1-11 + NFRs. Introduction states it CONSOLIDATES and supersedes
  `plugin-architecture` and `plugin-manager-ui`. Authored in the "extension"
  vocabulary per `.kiro/steering/extension-terminology.md`. Key points:
  - Req 1-2: extension host + live registry in `ff-desktop`; discovery/load/
    enable/disable; persisted `extensions.disabled` set.
  - Req 3-7: contribution seams -- command (runtime-extensible root registry),
    menu/menu-bar, WorkspaceContext, Workspace Kind, viewer/VFS.
  - Req 8: Extension Manager Context wired to the live registry.
  - Req 9: Catalogs/Files return as an extension.
  - Req 10-11: framework conformance + the runtime-extensible root registry
    (FFCMD) with two owner-confirmed additive framework extension points.
- `docs/specs/extension-integration/design.md` -- carries four owner decision
  points D1 (rename plugin->extension), D2 (Catalogs/Files as an extension), D3
  (in-tree feature-gated registry load model), D4 (SCRM stays core).
- `docs/specs/extension-integration/tasks.md` -- 13 task groups, ALL unchecked
  (`[ ]`). Explicitly GATED: "none may start until the owner confirms the package
  at the gate (decision points D1-D4)". Task 13 is the plugin->extension rename
  staging (waves A-D).

The older source specs `docs/specs/plugin-architecture/` and
`docs/specs/plugin-manager-ui/` still exist (named in `.kiro/steering/specs.md`);
task 13.1 renames them to `extension-architecture` / `extension-manager-ui`.

### State

DRAFTED, GATED, NOT STARTED. The requirements and design are mature and
framework-aligned. No implementation tasks are checked.

### Gap to a complete "Extensions Requirements Specification"

The existing spec answers "how is an extension built and how does it plug in".
What is MISSING for the user's stated goal:

- It does NOT yet reference or require the cross-cutting "every screen must
  comply" contracts. The user wants the Extensions spec to be the home (or the
  referrer) for: (a) the save convention every screen must follow, and (b) the
  snapshot+capture support every Context must provide. Neither is cross-linked
  from `extension-integration` today.
- There is no "master screen design directive" doc that an extension-contributed
  Context (Req 5) must satisfy alongside the existing WorkspaceContext/
  InteriorFocus focus contract. Req 5 enforces the FOCUS contract but is silent
  on save-convention and snapshot/capture compliance.
- The D1-D4 owner gate is still open, so the rename (plugin->extension) and the
  load model are not yet confirmed.

### Terminology

`.kiro/steering/extension-terminology.md`: "plugin" is being renamed to
"extension" gradually, in place, no per-rename confirmation, riding along with
real work. The crate is still `ff-plugin` in the tree; `KindTag::PluginManager`
and `TabKind::PluginManager` still use the old name
(`crates/ff-desktop/src/tab_state.rs`). The spec is already authored in the new
vocabulary.

---

## Concern 2 -- Save Convention

### Where it is defined (central doc EXISTS)

- `.kiro/steering/save-convention.md` -- the standing owner rule (B081):
  SAVE = commit + stay; END = commit + exit; RETURN = discard + exit; working
  copy; no autosave.
- `docs/project-management/project-master/requirements.md` Requirement 11 --
  the CENTRAL, EARS-form requirement ("Workspace Save Convention -- ISPF-Style
  Explicit Commit"), 10 acceptance criteria + a per-Context conformance table.
  This is the centralized requirement the user asked whether exists: IT EXISTS.
  Each editable Context's own spec carries a reference row back to it.

### The shared seam (how compliance is realised)

`crates/ff-desktop/src/shell/editable_context.rs` is the CR-CH-071 editable-
Context seam. `editable_context_kind()` maps the active `KindTag` to an
`EditableKind`, and `editable_persist` / `editable_discard` / `editable_reload`
route SAVE/END/RETURN/REFRESH to the owning Context. The ladder arms are in
`crates/ff-desktop/src/shell/commands_ladder_a.rs` (END persists-then-exits,
stays-open-on-error; RETURN discards; CANCEL discards + nav; REFRESH discard +
reload + stay).

### Which Contexts comply vs need retrofitting

On `main` (`editable_context.rs` wires these):

| Context | Save-convention state (main) |
|---------|------------------------------|
| Alias Editor | COMPLIANT (reference impl) |
| Theme Editor | COMPLIANT (wired; preview reverts on RETURN) |
| Menus Editor | COMPLIANT (wired; delegates to existing save) |
| Keys Editor | COMPLIANT (wired) |
| Kinds Editor | COMPLIANT (wired) |
| Config / Settings | NOT on main; fixed on branch `config-save-convention` |
| Command Configurator | NON-COMPLIANT (per-row writes incl. delete) |

The `_ => None` arm of `editable_context_kind()` explicitly documents the
Config/Settings TODO (CR-CH-071 fast-follow, command-environments task 34.1) and
that non-wired Contexts fall through to plain navigation (Req 20a.14).

### The unmerged commit (inspected read-only via `git show 0bcddf3`)

Branch `config-save-convention`, worktree
`.worktrees/config-save-convention`, head `0bcddf3`, 1 ahead / 1 behind main.
Single commit: "fix: wire Config/Settings into ISPF explicit-save seam (B081,
master Req 11)". It:
- adds a working-copy buffer + `persist_error` to `ConfigPanelState`
  (`config_panel/mod.rs`): `WorkingEdit` enum (Set/Reset), stage/clear/take
  helpers;
- `config_panel/commit.rs`: `commit_value` now STAGES (validates first), no store
  write on bare edit (Req 11.2);
- `config_panel/widgets.rs` + `render.rs`: widgets edit the working copy; a Save
  button (enabled when dirty) + unsaved indicator + persist-error banner;
- `shell/commands_config_save.rs` (new): `config_panel_persist` flushes the
  buffer in one commit, returns `Result` (END stays-open-on-error);
  `config_panel_discard/reload` drop the buffer;
- `shell/editable_context.rs`: adds `EditableKind::ConfigPanel`;
- `shell/tests_config_save_convention.rs` (new): 8 tests (11.1-11.5, 11.8);
- docs: master Req 11 Config/Settings row -> COMPLIANT; a Cross-Cutting Save
  Convention section in configuration-system requirements.

The commit is self-contained, scoped to Config/Settings, built on the single
dispatch seam. 10 files, +575/-23. It is 1 behind main, so it needs a rebase
before merge; the touched files (config_panel/*, editable_context.rs) are low
collision risk against current main.

### What still needs doing for "all core workspaces comply"

- Merge/rebase `config-save-convention` (Config/Settings).
- Retrofit the Command Configurator (the last non-compliant editable Context):
  whole-store working copy, defer all writes incl. delete to SAVE/END, RETURN
  discards. Add `EditableKind::CommandConfigurator` to the seam.
- Decide the convention's relationship to NON-editable Contexts (Help, SCRM
  viewer, menus, panels). These are read-only / navigational and do not edit a
  backing store, so they correctly fall through as plain navigation. END/RETURN
  there are pure navigation, not persistence (relevant to the Help END bug
  below).
- Centralize forward-looking compliance so FUTURE screens follow it: see the
  proposed master screen design directive (plan).

---

## Concern 3 -- SCRM (snapshot / capture / replay)

### Current implementation state (BUILT, beyond what tasks.md shows)

The spec `docs/specs/screen-snapshot-scrm/tasks.md` marks only Wave S (task 14,
menu-bar surfacing) complete, but the CODE is far ahead of the task list. Present
in-tree:

- `crates/ff-screen-model/` -- `ScreenModel`, `ScreenProvider`-shaped model, and
  renderers `text.rs`, `ansi.rs`, `markdown.rs`, `html.rs`, `yaml.rs`. Format
  enum `SnapshotFormat` = PlainText / Ansi / Markdown / Html / Yaml (5 formats;
  `render.rs`).
- `crates/ff-scrm/` -- collection/capture `model.rs`, zip `persist.rs`,
  `replay.rs`, `rules.rs`, `export.rs`, `pdf.rs`, `pdf_protected.rs`,
  `evidence.rs`, `error.rs`.
- `crates/ff-desktop/src/screen_snapshot.rs` -- builds a `ScreenModel` from a
  Menu_Workspace and renders it.
- `crates/ff-desktop/src/shell/commands_scrm.rs` -- SNAPSHOT + the full CAPTURE
  family (START/STOP/SCREEN/STATUS/LIST/PURGE/REPLAY/EXPORT/EVIDENCE/SAVE/
  LOAD/OPEN), auto-capture hook, protected-PDF export, PFSHOW.
- `crates/ff-desktop/src/menu_workspace/defaults.rs` -- `DEFAULT_MENU_BAR_TOML`
  with the SCRM `SNAPSHOT` dynamic entry already on the menu bar.

So SNAPSHOT, CAPTURE, REPLAY, EXPORT, and protected PDF already work for
Menu_Workspace Contexts. This matches the user's observation that SNAPSHOT
produces output.

### ANSI vs TEXT finding (user saw them identical -- CONFIRMED in code)

`crates/ff-screen-model/src/render/ansi.rs::render_ansi` does:
`let mut out = text::render_plain(model, options);` -- i.e. the ENTIRE framed
body is the plain-text render. It then appends an SGR colour legend ONLY for
fields where `attrs.highlight` is set or `attrs.colour` is non-default.

`crates/ff-desktop/src/screen_snapshot.rs::menu_workspace_screen_model` builds
each field with `Field::new(key, value)` and NO attributes -- so every field is
`Colour::Default`, no highlight. Therefore, for the POM and every menu Context,
NO field is coloured, the legend is empty, and ANSI output is BYTE-IDENTICAL to
TEXT. This is exactly the user's experience.

Where ANSI WOULD differ: any Context whose `ScreenModel` fields carry colour or
highlight attributes. None do today. The Settings/config screens the user
mentioned would only show an ANSI difference once their provider sets field
colour attributes (e.g. error/changed/dirty fields). This is latent, not wired.

### Gap to the desired end-state

The spec already covers much of the desired end-state in EARS; the GAPS are:

1. FULL-SCREEN capture. `menu_workspace_screen_model` captures only the menu
   title, the command line, and one field per menu option
   (`"description (command)"`). It does NOT capture window chrome / title bar,
   the menu bar, the tab header, the title line as the user sees it, the calendar,
   or the PF-key footer (pfshow on/off). The user's current-vs-expected mock gap
   is real: command-line value IS captured (`model.command_line`), but the
   calendar and PF-key display are NOT. Spec Req 3.8 lists the logical elements
   but the POM provider does not populate chrome/calendar/PF-keys. This needs a
   richer provider for the POM/menu Contexts.

2. Picture formats + format picker. `SnapshotFormat` has no bmp/png/jpeg/pdf/
   odt/odp. PDF exists only as a COLLECTION export (`ff-scrm/pdf.rs`), not as a
   SNAPSHOT format. Running `SNAPSHOT` with no args renders the default PlainText
   and copies to clipboard (`commands_scrm.rs::handle_snapshot` ->
   `snapshot_text_for_active("")`); there is NO format-picker popup. The spec
   does not yet require a picker or picture formats for SNAPSHOT.

3. Capture START bitmap+ANSI+markdown per transition + command metadata. The
   auto-capture hook `auto_capture_active_context` captures ONE `ScreenModel`
   per transition (text model only). It does NOT currently render a bitmap, nor
   store ANSI+markdown variants per capture. The command+screen RECORDER
   (TriggeringCommand) is SPEC'd (Req 9.10-9.12, 17.2a) and task-listed (Wave 2R
   tasks 11-13) but those tasks are unchecked -- so the triggering-command
   metadata is NOT yet populated at the hook.

4. Capture STOP Save-As dialog. `CAPTURE STOP` calls `self.scrm.stop()`; there is
   NO Save-As dialog to name the session or pick an output type. CAPTURE SAVE
   writes a `.ffscrm` zip to a path argument. No GUI picker.

5. Replay selection UI. `CAPTURE REPLAY` opens the `ScrmViewer` Context for the
   ACTIVE collection only (`open_scrm_viewer`). There is no "open a saved
   session and select from a list" UI; sessions are loaded only by
   `CAPTURE LOAD <file>`.

6. TOML-driven SCRM menu family. Today SCRM is surfaced ONLY as a single dynamic
   `SNAPSHOT` entry on `DEFAULT_MENU_BAR_TOML` (menu_workspace/defaults.rs). The
   bare POM (`DEFAULT_POM_TOML`) has NO SCRM entry (it is exactly 0/3/X per
   CR-CH-073). There is no "Reset Bare SCRM Menu" default. The user's requested
   SCRM dropdown family and bare-POM SCRM entry are not yet present.

### How the default-menu mechanism works (the plan must build ON this)

`crates/ff-desktop/src/menu_workspace/defaults.rs`:
- `DEFAULT_POM_TOML` -- code-only barebones POM (0 Settings / 3 Help / X),
  NEVER written to disk; `recovery_pom_menu()` parses it.
- `DEFAULT_SETTINGS_TOML` -- code-only Settings menu; `recovery_settings_menu()`.
- `DEFAULT_MENU_BAR_TOML` -- code-only menu bar (Settings / Help / SNAPSHOT /
  hidden X); `default_menubar_menu()` parses it; `DEFAULT_MENU_BAR_NAME` =
  "MB-POM". A user `menus/<slug>.toml` OVERRIDES the compiled default.
- The `SNAPSHOT` bar entry is a DYNAMIC source (see
  `shell::render_menu_bar::dynamic_menu_options`), mirroring `THEME LIST`: its
  children are generated at runtime, each dispatched via the single front door
  `dispatch_command_string` -> `resolve_target`.
- Built-in menus are code-only (CR-CH-021); `ensure_menus_dir` only creates an
  empty `menus/` directory and never materialises defaults.

So adding an SCRM menu family means: add a new code-only `DEFAULT_SCRM_MENU_TOML`
(+ parse function), optionally add an SCRM entry to `DEFAULT_POM_TOML`, and make
the bar's SCRM dropdown list the SCRM verbs -- all code-only defaults resolved
through `resolve_target` / `open_menu_by_name`. No disk writes, no bespoke
intercept, no second dispatcher.

### Menu-bar hotkeys (user undecided: Alt+0..9 vs mnemonic letter)

Current menu options already carry a `key` field (e.g. Settings `0`, Help `3`,
SNAPSHOT `S`, X `X` in `DEFAULT_MENU_BAR_TOML`). The keys framework
(`docs/specs/menu-and-statusbar` Req 17, `function-keys-and-history`,
`menu-workspace` Req 17) resolves function/modified keys via the active key map.
Tradeoffs are presented in the plan; recommendation there.

---

## The two bugs (root cause only; NO fix now)

### Bug A -- F3 in Help: "Error in END: command not yet implemented"

Root cause chain (evidence):

- The error text originates in
  `crates/ff-command-semantics/src/engine.rs::execute_command_line`:
  `StatusMessage::runtime_error(&name, "command not yet implemented")`. This is
  the CommandEngine STUB terminal for any `ParsedCommand::Command` it receives.
- It is reached from
  `crates/ff-desktop/src/shell/commands.rs::run_command_ladder` ONLY as the
  final "stage 5: unresolved" fall-through:
  `let status = self.cmd_engine.execute_command_line(cmd);`.
- END is normally handled BEFORE that terminal, in
  `crates/ff-desktop/src/shell/commands_ladder_a.rs` (`if upper == "END"` ->
  `nav_end()`), reached via `try_commands_a`. And a passing test
  `crates/ff-desktop/src/shell/tests_nav.rs` (HELP -> `handle_command("END")`)
  proves the LADDER path returns Help to its parent Context.
- The discrepancy: the LIVE F3 path does NOT go through the same entry as the
  test. F3 is resolved by
  `crates/ff-desktop/src/shell/update_keys.rs::resolve_function_key_command`
  (reads the ACTIVE key map) and dispatched by
  `crates/ff-desktop/src/shell/update.rs` -> `dispatch_key_command` ->
  `crates/ff-desktop/src/shell/target_dispatch.rs::dispatch_bound_command` ->
  `resolve_and_dispatch_command`. The test uses `handle_command`/
  `dispatch_command_string` (the ladder). These are TWO different resolution
  entry points.
- Therefore END from the Help Context, on the live key path, is not being caught
  by the editable/ladder END arm and falls through to the CommandEngine stub.
  The Help Context has NO Command Environment entry in
  `crates/ff-desktop/src/shell/environment.rs` (grep for `HelpContext` -> no
  match), so no environment claims END; the question for the fixing stream is why
  the `dispatch_bound_command` -> `resolve_and_dispatch_command` path does not
  reach the ladder's END arm for the Help Context while the `handle_command`
  path does. The most probable point is that `resolve_and_dispatch_command`
  classifies/forwards the Help Context's END differently from
  `run_command_ladder`, letting it reach the CommandEngine terminal.

Save-convention END semantics for Help: Help is a NON-editable, navigational
Context (no working copy, no backing store). Per master Req 11 it is NOT an
editable Context, so END must be PURE NAVIGATION: pop the Navigation_Stack back
to the Context Help was opened from (the `close_help` / `nav_end` behaviour in
`crates/ff-desktop/src/shell/help.rs` and the ladder). The fix is to ensure the
live F3/END path reaches that navigation, not the CommandEngine stub. This is
NOT a save-convention persistence change; it is a dispatch-routing fix that the
save-convention stream should carry because it concerns END semantics on a
screen.

Recommendation: the fixing stream should add a live full-shell egui_kittest test
that presses F3 (not `handle_command("END")`) in the Help Context and asserts it
navigates back with no "command not yet implemented" error -- the existing test
masks the bug because it uses the wrong entry point.

### Bug B -- "SNAPSHOT: the active workspace cannot be captured yet"

Root cause (direct):

- `crates/ff-desktop/src/shell/commands_scrm.rs::snapshot_text_for_active`
  returns `Err("SNAPSHOT: the active workspace cannot be captured yet.")` when
  `self.active_screen_model()` is `None`.
- `active_screen_model()` returns `Some` ONLY when
  `self.tabs.active_tab().kind.menu_workspace()` is `Some` -- i.e. ONLY for
  Menu_Workspace Contexts (the POM, Settings, and other TOML menus). For every
  other Context it returns `None`.
- `CAPTURE SCREEN` has the same guard (`ff-desktop-shell-msg-scrm-screen-not-
  capturable`, same `active_screen_model()` check).

So the error fires on EVERY non-menu Context. Affected Contexts (from
`crates/ff-desktop/src/tab_state.rs` `KindTag`): FileEditor, Untitled,
FilesPanel, ConfigPanel, FileExplorerPanel, SearchResults, PluginManager,
EventLog, MacroLibrary, CommandConfigurator, ThemeEditor, MenusEditor,
KeysEditor, KindsEditor, AliasEditor, HelpContext, ScrmViewer. Only
MenuWorkspace is capturable. Despite spec Req 2 defining a `ScreenProvider`
trait, the live code uses a HARD-CODED menu-only path, not a per-Context
provider -- so no Context can opt in without editing `active_screen_model()`.

This is the same gap as the "every screen must support snapshot+capture"
requirement: the fix is a per-Context `ScreenProvider` contract (a master screen
design directive) plus provider impls for each Context, which the SCRM stream
should carry.

---

## Conclusions and recommendations (detail in plan.md)

- The Extensions spec exists and is strong; the work is to (a) run the D1-D4
  gate, and (b) fold in / cross-reference the two "every screen" contracts. Do
  NOT rewrite it.
- The save-convention central requirement EXISTS (master Req 11). The remaining
  work is merge the Config/Settings branch, retrofit the Command Configurator,
  and add a forward-looking "every editable screen complies" directive.
- SCRM is mostly built; the work is enriching the screen model (full chrome +
  calendar + PF-keys), adding a per-Context `ScreenProvider` so every Context is
  capturable, adding picture formats + a format picker, a capture-STOP Save-As
  dialog, a replay session picker, and the TOML-driven SCRM menu family on the
  existing default-menu mechanism.
- BOTH "every screen" contracts (save convention + snapshot/capture) belong
  together as a single MASTER SCREEN DESIGN DIRECTIVE, a sibling of the existing
  WorkspaceContext/InteriorFocus focus contract (workspace-conformance steering),
  referenced from the Extensions spec Req 5 so extension-contributed Contexts
  must satisfy all three. This lets each screen be retrofitted ONCE for both
  contracts.
- Both bugs are routing/coverage gaps, not persistence logic; sequence Bug A into
  the save-convention stream and Bug B into the SCRM stream, as the user asked.
