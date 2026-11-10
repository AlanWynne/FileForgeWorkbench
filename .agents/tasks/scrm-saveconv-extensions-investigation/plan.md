# Implementation Plan -- Extensions Spec + Save Convention + SCRM + Master Screen Directive

This is a PLAN ONLY. No code or requirements docs are written in this run; the
owner approves before any gate. All work follows the requirements GATE first
(`.kiro/steering/workflow.md`), builds ON existing framework seams
(`framework-conformance.md`, `wiring-standard.md`), is ASCII-only
(`documentation-charset.md`), and respects the scoped-per-change / full-gate-
before-merge cadence (`gate-policy.md`). Agents compile-verify; the owner runs
the full gate.

The three concerns overlap on the SAME screens/Contexts, so the plan sequences a
SHARED central directive FIRST, then two parallel implementation streams that
each touch a given screen ONCE for both contracts.

---

## Phase 0 -- Requirements gate (docs only, no source)

Per workflow + framework-conformance, all requirements must be settled and
owner-approved before code. These are the docs to create/update, in order.

- [ ] 0.1 Author the MASTER SCREEN DESIGN DIRECTIVE (new central requirements doc).
      Create `docs/specs/master-screen-directive/requirements.md` (EARS) +
      `design.md`. It mandates, for EVERY Workspace Context, compliance with
      THREE cross-cutting contracts: (1) the existing WorkspaceContext /
      InteriorFocus focus contract (reference `.kiro/steering/workspace-
      conformance.md`); (2) the ISPF save convention for EDITABLE Contexts
      (reference master Req 11 + `.kiro/steering/save-convention.md`); (3)
      snapshot + capture support via a per-Context `ScreenProvider`
      (reference screen-snapshot-scrm Req 2). Define the contract: an editable
      Context MUST wire the CR-CH-071 seam; EVERY Context MUST provide a
      `ScreenProvider` (or an explicit documented "no interior screen" opt-out,
      mirroring `InteriorFocus::none()`). Include a retrofit table (section
      "Core Context retrofit matrix" below).
      Files: `docs/specs/master-screen-directive/requirements.md`,
      `docs/specs/master-screen-directive/design.md`,
      `docs/specs/master-screen-directive/tasks.md`.
      Rationale: both "every screen" contracts share the same screens; one
      directive lets each screen be touched once for both. This is the natural
      sibling of the focus contract already enforced per-arm.
      Verify (gate): owner review + approval; no source touched.

- [ ] 0.2 Fold the master directive into the Extensions spec.
      Update `docs/specs/extension-integration/requirements.md` Req 5 (Workspace
      Context contribution) to REQUIRE an extension-contributed Context satisfy
      the master screen directive (focus + save-convention-if-editable +
      ScreenProvider), not only the focus latch. Add a cross-reference, do not
      duplicate the criteria. Record the re-phrasing in the change log.
      Files: `docs/specs/extension-integration/requirements.md` (edit Req 5 +
      add a reference row).
      Verify (gate): owner review; criteria consistent, no contradiction with
      the existing D1-D4 decisions.

- [ ] 0.3 Update the SCRM requirements for the desired end-state (EARS).
      Edit `docs/specs/screen-snapshot-scrm/requirements.md` to ADD/clarify:
      full-screen model (chrome/title bar/menu bar/tab header/title line/command
      line value/POM option rows/calendar/PF-key footer per pfshow) as the POM
      provider's required content (extends Req 3.8); a SNAPSHOT format-picker
      when no argument is given (new Req); picture snapshot formats (bmp/png/
      jpeg and/or pdf/odt/odp -- see 0.6 decision) with combined picture+ANSI/
      TEXT output where the format supports it (extends Req 4/5.5); capture-START
      per-transition bitmap+ANSI+markdown + triggering-command metadata (extends
      Req 9 + confirms Wave 2R Req 9.10-9.12); capture-STOP Save-As dialog with
      type choice (new Req); replay OPEN + SELECT-from-saved-sessions UI (extends
      Req 10); and the TOML-driven SCRM menu family (extends Req 22 -- bare-POM
      SCRM entry + a "Reset Bare SCRM Menu" default + the bar dropdown).
      Files: `docs/specs/screen-snapshot-scrm/requirements.md`,
      and refresh `docs/specs/screen-snapshot-scrm/tasks.md`.
      Verify (gate): owner review; every new capability expressed on an existing
      framework seam (no new dispatcher / nav stack).

- [ ] 0.4 Confirm the save-convention central requirement is complete.
      master Req 11 already EXISTS. Add (if the owner wants forward-enforcement)
      a one-line forward directive: "every FUTURE editable Context MUST wire the
      CR-CH-071 seam before it ships" and cross-link it from the master directive
      (0.1). Update the per-Context conformance table to reflect current reality
      (Theme/Menus/Keys/Kinds COMPLIANT on main; Config/Settings pending the
      branch merge; Command Configurator outstanding).
      Files: `docs/project-management/project-master/requirements.md` (Req 11
      table refresh).
      Verify (gate): owner review.

- [ ] 0.5 Run the Extensions D1-D4 owner gate.
      Decide D1 (plugin->extension rename go-ahead), D2 (Catalogs/Files as an
      extension), D3 (in-tree feature-gated registry load model), D4 (SCRM stays
      core). These unblock `extension-integration/tasks.md`. SCRM D4 = core is
      consistent with this plan (SCRM is core-packaged).
      Verify (gate): recorded owner decisions in `design.md`.

- [ ] 0.6 DECISION: which picture/doc formats to implement FIRST (owner input).
      Recommendation below. Record in the SCRM design.
      Verify (gate): owner picks the first format set.

### Decision 0.6 -- first snapshot picture/doc format (RECOMMENDED: PNG first)

Options the user listed: bmp, png, jpeg, pdf, odt, odp.
- RECOMMEND PNG first. It is lossless (faithful to a monospaced screen),
  universally pasteable, has a mature pure-Rust encoder (`image`/`png`), and is
  the natural "picture alongside ANSI/TEXT" artefact. jpeg (lossy) is poor for
  crisp text; bmp is large and legacy.
- Add PDF SECOND for snapshots ONLY by reusing the EXISTING `ff-scrm/pdf.rs`
  collection PDF path (already a dependency, already produces selectable text),
  so a single-screen "snapshot to PDF" is a thin wrapper, not new infrastructure.
- Defer odt/odp (ODF) -- heavier, less-used; revisit after PNG+PDF land.
Rationale: PNG covers the "include a picture-format rendering AND the ANSI/TEXT
rendering" requirement with the least new dependency surface; PDF reuses built
code. Rendering the picture: rasterise the monospaced TEXT render to a bitmap
(fixed-width font), NOT an egui screengrab -- keeps the logical-model-first
constraint (screen-snapshot-scrm Req 1) intact.

---

## Phase 1 -- Implementation streams (worktrees)

RECOMMENDATION: TWO implementation worktrees/streams, sequenced, each conforming
to the master directive so every screen is touched ONCE for both contracts. A
small Phase-1A shared step lands the `ScreenProvider` contract both streams need.

Stream order and rationale:
1. Phase 1A (shared, SMALL): land the per-Context `ScreenProvider` contract +
   the master-directive conformance test harness. Both streams depend on it.
2. Stream SAVE (save-convention retrofit) -- smaller, lower risk, merges first.
3. Stream SCRM (snapshot/capture end-state) -- larger, builds on 1A.

Why not one big worktree: the two contracts have different blast radii and test
harnesses; separate worktrees keep each merge small and the full gate green per
merge (gate-policy). Why not three: the Extensions implementation (CR-CH-074) is
its own large, independently-gated program (D1-D4) and should remain a SEPARATE
future worktree, not entangled with these two retrofits -- the Extensions spec
only needs the 0.2 cross-reference now.

### Phase 1A -- shared ScreenProvider contract (prerequisite)

- [ ] 1A.1 Define the per-Context `ScreenProvider` seam in `ff-desktop`.
      Replace the hard-coded menu-only `active_screen_model()`
      (`crates/ff-desktop/src/shell/commands_scrm.rs`) with a dispatch that asks
      the ACTIVE Context for its `ScreenModel`, defaulting to the existing
      menu_workspace provider for Menu_Workspaces and an explicit opt-out for
      Contexts that genuinely have no screen. Build on the existing
      `ff_screen_model::ScreenProvider` trait (screen-snapshot-scrm Req 2.1).
      Files: `crates/ff-desktop/src/shell/commands_scrm.rs`,
      `crates/ff-desktop/src/screen_snapshot.rs`.
      Verify: `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D
      warnings`, `cargo fmt -p ff-desktop -- --check`; scoped unit test that the
      POM still snapshots and one previously-"not capturable" Context now
      returns a model. Full suite is the owner's gate.

- [ ] 1A.2 Add the master-directive conformance test harness.
      A reusable full-shell egui_kittest helper asserting, per Context: the
      first-Tab focus contract (existing), and SNAPSHOT returns a model (no
      "cannot be captured yet"). Mirrors the per-arm focus test in
      workspace-conformance steering.
      Files: a new `crates/ff-desktop/src/shell/tests_screen_directive.rs`.
      Verify: scoped compile + the harness runs for the POM; owner gate for full.

### Stream SAVE -- save-convention retrofit (worktree 1)

Merge/rebase first; smallest change set.

- [ ] S.1 Rebase + merge the existing `config-save-convention` branch.
      Worktree `.worktrees/config-save-convention`, head `0bcddf3` (1 ahead / 1
      behind main). Rebase onto current main (low collision: config_panel/*,
      editable_context.rs), re-run the 8 `tests_config_save_convention` tests
      scoped, then the owner's full gate before merge.
      Files (already in the commit): `config_panel/{mod,commit,render,widgets}.rs`,
      `shell/commands_config_save.rs`, `shell/editable_context.rs`,
      `shell/tests_config_save_convention.rs`, docs.
      Verify: `cargo test -p ff-desktop config_save_convention` (scoped); full
      `cargo gate` CLEAN before merge (gate-policy).

- [ ] S.2 Retrofit the Command Configurator into the CR-CH-071 seam.
      The LAST non-compliant editable Context. Add
      `EditableKind::CommandConfigurator` to
      `crates/ff-desktop/src/shell/editable_context.rs`; give the panel a
      whole-store working copy; defer all writes (incl. delete) to persist;
      discard drops the working copy; END persists-then-exits (stays-open-on-
      error), RETURN discards. Follow the Menus/Keys/Kinds pattern already wired.
      Files: `crates/ff-desktop/src/shell/editable_context.rs`, the command
      configurator panel module, new save-convention tests.
      Verify: scoped `cargo test -p ff-desktop` for the new tests + the trio;
      owner full gate before merge.

- [ ] S.3 Fix Bug A (Help F3 END "command not yet implemented") in this stream.
      Root cause (findings): the live F3 path
      (`update_keys::resolve_function_key_command` ->
      `target_dispatch::dispatch_bound_command` ->
      `resolve_and_dispatch_command`) lets the Help Context's END fall through to
      the CommandEngine stub (`ff-command-semantics/engine.rs`), whereas the
      `handle_command`/ladder path (tested in `tests_nav.rs`) handles it. Make
      the `dispatch_bound_command` path reach the SAME END navigation
      (`nav_end`/`close_help`) for a non-editable navigational Context. END
      semantics for Help per master Req 11: Help is NON-editable, so END is pure
      navigation (pop Navigation_Stack back to the opener), NOT persistence.
      Files: likely `crates/ff-desktop/src/shell/target_dispatch.rs` and/or the
      ladder reachability; confirm with a LIVE repro first.
      Verify: NEW full-shell egui_kittest that presses F3 (not
      `handle_command("END")`) in the Help Context and asserts navigation back +
      no error; scoped trio; owner full gate.

- [ ] S.4 Confirm every editable Context now complies + table refresh.
      Update master Req 11 table rows to COMPLIANT. Run the 1A.2 harness across
      editable Contexts.
      Verify: owner full gate CLEAN before merge to main.

### Stream SCRM -- snapshot/capture end-state (worktree 2)

Builds on Phase 1A. Larger; sequence internally by dependency.

- [ ] C.1 Enrich the POM/menu `ScreenProvider` to the FULL screen.
      Populate the `ScreenModel` with window chrome/title bar, menu bar, tab
      header, title line, the Command line WITH its value (already present), the
      POM option rows WITH command values (present), the calendar, and the
      PF-key footer shown/hidden per `key_bar_visible`/`key_bar_scope`
      (pfshow). Extend `menu_workspace_screen_model`
      (`crates/ff-desktop/src/screen_snapshot.rs`) and the `ScreenModel`
      element set if needed (`crates/ff-screen-model/src/model.rs`).
      Files: `crates/ff-desktop/src/screen_snapshot.rs`,
      `crates/ff-screen-model/src/model.rs`, renderers as needed.
      Verify: scoped tests that the snapshot text contains the calendar + a
      PF-key line when pfshow is on and omits the PF-key line when off; owner gate.

- [ ] C.2 Add per-Context `ScreenProvider` impls (retrofit).
      For each core Context in the retrofit matrix below, provide a `ScreenModel`
      so SNAPSHOT/CAPTURE work everywhere (closes Bug B generally). Same change
      pattern across many Contexts = ONE feature, files listed. Where a Context
      legitimately has no screen, record the documented opt-out.
      Files: each Context panel module under `crates/ff-desktop/src/` (config_
      panel, theme editor, menus/keys/kinds editors, alias editor, files panel,
      search results, event log, macro library, command configurator, help
      context, scrm viewer, editor).
      Verify: the 1A.2 harness asserts each Context returns a model; owner gate.

- [ ] C.3 Fix Bug B (generalised) is satisfied by 1A.1 + C.2.
      The "cannot be captured yet" error now only appears for a documented
      opt-out Context. Confirm the error path still exists for a true opt-out.
      Verify: scoped test; owner gate.

- [ ] C.4 Add picture snapshot format(s) + the format picker.
      Per decision 0.6: add PNG as a `SnapshotFormat` (rasterise the monospaced
      TEXT render; new encoder dep pinned exact), then a thin snapshot-to-PDF
      reusing `ff-scrm/pdf.rs`. Bare `SNAPSHOT` (no arg) pops a format picker
      dialog; when the chosen format supports it, produce BOTH the picture and
      the ANSI/TEXT rendering. Keep logical-model-first (no egui screengrab).
      Files: `crates/ff-screen-model/src/{lib,render}.rs` (new format + encoder),
      `crates/ff-desktop/src/shell/commands_scrm.rs` (picker + handle_snapshot),
      a new picker dialog module.
      Verify: scoped tests (PNG bytes produced; picker dispatches the chosen
      format via the single front door); owner gate.

- [ ] C.5 Capture START: per-transition bitmap + ANSI + markdown + command meta.
      Extend `auto_capture_active_context`
      (`crates/ff-desktop/src/shell/commands_scrm.rs`) to store, per capture, the
      bitmap (C.4 encoder) + ANSI + markdown variants, AND the triggering command
      from the dispatch seam (`resolve_target` result) per screen-snapshot-scrm
      Req 9.10-9.12 / 17.2a (Wave 2R tasks 11-13). Reuse the existing single
      dispatch seam; NO bespoke intercept.
      Files: `crates/ff-scrm/src/model.rs` (TriggeringCommand + image/ansi/md
      fields if absent), `crates/ff-desktop/src/shell/commands_scrm.rs`.
      Verify: scoped tests (command-caused transition records the command;
      interval/rule capture records none); owner gate.

- [ ] C.6 Capture STOP: Save-As dialog.
      On `CAPTURE STOP`, show a Save-As dialog to name the session and choose
      save-as type (ANSI/TEXT, HTML, Markdown, PDF, ODT, ODP -- initially the
      implemented subset), embedding the bitmap alongside the monospaced text
      where the type supports it. Reuse existing export functions
      (`ff-scrm/export.rs`, `pdf.rs`); the dialog invokes the EXISTING
      `CAPTURE SAVE`/`CAPTURE EXPORT` commands (command parity), not a new path.
      Files: `crates/ff-desktop/src/shell/commands_scrm.rs`, a Save-As dialog
      module, dialog wiring in the update/overlay cluster.
      Verify: scoped tests (dialog choice dispatches the matching export command);
      owner gate.

- [ ] C.7 Replay: open + SELECT a saved session.
      Extend the `ScrmViewer` Context (and/or a session-list picker) so the user
      can OPEN a saved `.ffscrm` and SELECT from a list of saved sessions, not
      only replay the active collection. Reuse `ff-scrm/persist.rs` load +
      `replay.rs`; the Context stays a `WorkspaceContext` (InteriorFocus, first-
      Tab test) per workspace-conformance.
      Files: `crates/ff-desktop/src/` SCRM viewer module,
      `crates/ff-desktop/src/shell/commands_scrm.rs` (CAPTURE REPLAY / OPEN).
      Verify: full-shell first-Tab egui_kittest for the picker; scoped tests;
      owner gate.

- [ ] C.8 TOML-driven SCRM menu family (build ON menu_workspace/defaults.rs).
      Add a code-only `DEFAULT_SCRM_MENU_TOML` + a `recovery_scrm_menu()` parse
      fn in `crates/ff-desktop/src/menu_workspace/defaults.rs` (NEVER written to
      disk, CR-CH-021). Add an SCRM entry to `DEFAULT_POM_TOML` (bare-POM) and
      turn the bar's existing dynamic `SNAPSHOT` entry into the full SCRM
      dropdown (Snapshot / Capture start / Capture stop / Capture replay / SCRM
      Help). Every option's `command` is an EXISTING SCRM verb resolved through
      `resolve_target` -> `dispatch_command_target` / `open_menu_by_name`. The
      keys/labels below are a STARTING SUGGESTION to refine at the gate.
      Suggested bare POM: S Settings / SCRM / H Help / X Exit. Suggested Reset
      Bare SCRM Menu: Snapshot (SS), Capture start (Cr), Capture stop (Cs),
      Capture replay (Cp), SCRM Help (H), Exit (X).
      NOTE conflict to resolve at gate: CR-CH-073 fixes the bare POM at EXACTLY
      0/3/X; adding an SCRM entry to the bare POM is a deliberate change to that
      invariant and MUST be owner-confirmed as a framework-touching decision
      (framework-conformance). The bar-only surfacing already exists and needs no
      invariant change -- recommend expanding the BAR dropdown first (no
      invariant change) and treating the bare-POM SCRM entry as the owner-
      confirmed follow-up.
      Files: `crates/ff-desktop/src/menu_workspace/defaults.rs`,
      `crates/ff-desktop/src/shell/render_menu_bar.rs` (dynamic options).
      Verify: scoped unit tests (new default parses, ASCII-only, POM invariant
      handled per the gate decision, each SCRM option dispatches its verb through
      the single front door) + a full-shell click test; owner gate.

- [ ] C.9 Menu-bar accelerator hotkeys (DECISION below).
      Add accelerators to menu-bar items via the EXISTING key map framework
      (menu-and-statusbar Req 17, function-keys-and-history, menu-workspace
      Req 17), not a new mechanism.
      Files: `crates/ff-desktop/src/shell/render_menu_bar.rs`, key map defaults.
      Verify: scoped tests (accelerator resolves to the item's command via the
      active key map); owner gate.

---

## Core Context retrofit matrix (save-convention AND snapshot/capture)

One table so each screen is touched ONCE for both contracts. "Editable" = must
wire the CR-CH-071 save seam; "Snapshot" = must provide a `ScreenProvider`.
Source: `crates/ff-desktop/src/tab_state.rs` (KindTag) + `editable_context.rs` +
`commands_scrm.rs`.

| Context (KindTag) | Save convention (current -> needed) | Snapshot/capture (current -> needed) |
|---|---|---|
| MenuWorkspace (POM/Settings/menus) | n/a (not editable) | capturable, but TRIMMED -> enrich to full chrome/calendar/PF-keys (C.1) |
| AliasEditor | COMPLIANT | not capturable -> add provider (C.2) |
| ThemeEditor | COMPLIANT | not capturable -> add provider |
| MenusEditor | COMPLIANT | not capturable -> add provider |
| KeysEditor | COMPLIANT | not capturable -> add provider |
| KindsEditor | COMPLIANT | not capturable -> add provider |
| ConfigPanel (Settings/Config) | pending branch merge (S.1) | not capturable -> add provider (ANSI colour matters here) |
| CommandConfigurator | NON-COMPLIANT -> retrofit (S.2) | not capturable -> add provider |
| FilesPanel (Virtual Catalog Mgr) | n/a (not an editable text store) | not capturable -> add provider |
| FileExplorerPanel | n/a | not capturable -> add provider |
| SearchResults | n/a (read-only) | not capturable -> add provider |
| PluginManager (-> ExtensionManager) | n/a | not capturable -> add provider |
| EventLog | n/a (read-only) | not capturable -> add provider |
| MacroLibrary | n/a | not capturable -> add provider |
| HelpContext | n/a (navigational) | not capturable -> add provider; FIX Bug A END (S.3) |
| ScrmViewer | n/a | not capturable (or documented opt-out) -> decide at gate |
| FileEditor / Untitled (editor) | FFEDIT owns SAVE (active-wins) | not capturable -> add provider when editor re-enters (parked) |

---

## Framework seams the work MUST build on (do NOT invent mechanisms)

Per framework-conformance.md + wiring-standard.md:

- SINGLE command dispatch: every SCRM/snapshot/save verb resolves through
  `ff_command::resolve_target` and `shell/target_dispatch.rs`
  (`dispatch_command_target` / `dispatch_bound_command`). No `if upper == "..."`
  intercept, no second dispatcher. (Bug A fix is making the key path reach the
  SAME dispatch, not adding a new one.)
- MENU defaults: code-only `DEFAULT_*_TOML` in
  `crates/ff-desktop/src/menu_workspace/defaults.rs`, parsed by
  `recovery_*_menu()` / `default_menubar_menu()`, NEVER written to disk
  (CR-CH-021). The SCRM menu family (C.8) adds a new code-only default the same
  way; the dynamic SCRM dropdown mirrors the existing `SNAPSHOT`/`THEME LIST`
  dynamic-source pattern (`render_menu_bar::dynamic_menu_options`).
- WorkspaceContext + InteriorFocus single focus latch: any new/affected Context
  (SCRM replay picker, format picker if a Context) reports focus through
  `render_workspace_context` / `InteriorFocus`; ships the mandatory first-Tab
  egui_kittest test (workspace-conformance).
- Save seam: editable Contexts wire the CR-CH-071 `editable_context.rs` seam
  (persist/discard/reload), never a per-panel autosave.
- ScreenProvider seam: the per-Context provider (1A.1) is the ONE place SNAPSHOT/
  CAPTURE obtain a screen; it reads the logical model, never egui widgets
  (screen-snapshot-scrm Req 2.2).
- Session persistence: any new Context persists as a `WorkspaceDescriptor`
  (CustomWorkspace), transient replay position not persisted.

---

## Decision -- menu-bar accelerator hotkey scheme (RECOMMENDED: mnemonic letter)

User is undecided between Alt+0..Alt+9 per item vs a mnemonic letter per item.

- Alt+digit (Alt+0..9): positional, language-neutral, but opaque (the user must
  learn which number is which item) and collides conceptually with the POM
  numeric option keys (0 Settings / 3 Help) -- confusing to overload digits for
  two different selection models. Caps at 10 items.
- Mnemonic letter (Alt+<letter>, e.g. Alt+S Settings, Alt+H Help, Alt+C SCRM):
  self-documenting (the letter matches the label), scales past 10 items, and the
  menu options ALREADY carry a `key` letter field (Settings `0`, Help `3`,
  SNAPSHOT `S` in `DEFAULT_MENU_BAR_TOML`) that a mnemonic can reuse. It aligns
  with the existing per-option `key` mnemonic model and with common desktop menu
  conventions.

RECOMMENDATION: mnemonic letter accelerators (Alt+<mnemonic>), reusing each
option's existing `key` where it is a letter and assigning a distinct mnemonic
where it is a digit (Settings -> Alt+S, Help -> Alt+H, SCRM -> Alt+C). Resolve
them through the existing key map framework (a modified-key binding per bar
item), not a new accelerator mechanism. Caveat for the gate: ensure the chosen
mnemonics do not collide with existing Alt-bindings in the active key profile.

---

## Sequencing summary

1. Phase 0 gate (docs): master directive (0.1) -> Extensions cross-ref (0.2) ->
   SCRM EARS update (0.3) -> save-convention table (0.4) -> Extensions D1-D4
   (0.5) -> format decision (0.6). Owner approves before any code.
2. Phase 1A (shared): ScreenProvider contract + conformance harness.
3. Stream SAVE (worktree 1): merge Config/Settings (S.1) -> Command Configurator
   (S.2) -> Bug A Help END (S.3) -> confirm + table (S.4). Full gate, merge.
4. Stream SCRM (worktree 2): full-screen model (C.1) -> per-Context providers +
   Bug B closure (C.2/C.3) -> picture formats + picker (C.4) -> capture START
   meta (C.5) -> capture STOP Save-As (C.6) -> replay picker (C.7) -> TOML SCRM
   menu family (C.8) -> menu-bar accelerators (C.9). Full gate, merge.
5. (Separate, later) Extensions implementation (CR-CH-074) once D1-D4 confirmed
   -- its own worktree, consuming the master directive via the 0.2 cross-ref.

Bug A rides Stream SAVE (END semantics on a screen); Bug B rides Stream SCRM
(every-screen capture) -- exactly as the user asked.

## Open items for the owner at the gate

- 0.6 first picture/doc format (recommend PNG, then PDF-reuse; defer ODF).
- C.8 whether the bare POM gains an SCRM entry (changes the CR-CH-073 0/3/X
  invariant -- framework-touching) or SCRM stays bar-only (recommended first
  step: bar-only, no invariant change).
- C.9 accelerator scheme (recommend mnemonic letters).
- SCRM menu keys/labels in C.8 (starting suggestion given; refine at gate).
- ScrmViewer self-capture (capturable vs documented opt-out).
