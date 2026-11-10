# Implementation Plan -- Stream SCRM Phase 2C (capture lifecycle)

Worktree: `c:/workspace/apps/FileForgeWorkbench/.worktrees/scrm-2c-capture-lifecycle`
(branch `scrm-2c`). Do NOT merge to main in this run (owner reviews + runs the
full gate, then merges per gate-policy).

This plan implements C.5 (capture START per-transition metadata), C.6 (capture
STOP Save-As dialog with the ffwb-domain default), and C.7 (replay
save-to-file + list/open/select + export). It does NOT touch 2D (menu-from-POM
restructure) or 2E (dialog/message-box capture).

Everything builds ON the existing seams verified in the code during exploration:
the single dispatch front door (`handle_capture` -> SCRM verbs), the per-Context
`ScreenProvider` seam (`active_screen_capture`, already landed with providers for
all Contexts), the Phase 2B async window-screenshot path
(`snapshot.pending` / `snapshot.screenshot_requested` resolved on the
`Event::Screenshot` frame in `shell/update.rs`), `ff-scrm` persist/replay/export,
and the dialog-overlay pattern in `shell/update_dialogs.rs`. NO second dispatcher,
NO bespoke intercept, NO egui screengrab in the logical-model text path.

## Design decisions made during exploration (grounded in the code)

- D1 -- Save-As default captures directory = the EXISTING `scrm_dir()`.
  `WorkbenchShell::scrm_dir()` (`shell/commands_session.rs`) already resolves to
  `<User_Data_Dir>/screen-collections/` via `ff_session::UserDataDir::resolve(None)`
  (the app's domain-path authority; base `<config>/ffworkbench`), with a
  test-only override `dir_overrides.scrm`. This IS the discoverable ffwb-domain
  captures directory the owner asked for. The Save-As dialog therefore PRE-FILLS
  `scrm_dir()` and lets the user navigate elsewhere; a chosen name/dir maps
  through the EXISTING `resolve_scrm_path(arg, stem, ext)`. I do NOT invent a
  second scheme (e.g. a `.ffworkbench/captures/` under a project root) -- that
  would be a parallel path convention; `scrm_dir()` is the established one and
  already the default for CAPTURE SAVE/EXPORT/LOAD. (Owner directive satisfied:
  discoverable in the ffwb domain + navigable away.)

- D2 -- Per-transition screenshot tradeoff (C.5). A REAL window screenshot is
  ASYNC in egui 0.33 (the Phase 2B path needs a frame round-trip:
  request -> `ViewportCommand::Screenshot` -> later `Event::Screenshot`). Taking
  one synchronously inside `auto_capture_active_context` (called from the
  Context-transition hook) is not feasible and would interrupt navigation
  (violating Req 18.2). DECISION: per transition, record RELIABLY the logical
  `ScreenModel` (authoritative selectable text; ANSI and Markdown are rendered
  on demand from it via `ff_screen_model::render`, so they need no stored copy)
  PLUS the `TriggeringCommand` metadata. The per-capture `image` (bitmap) is NOT
  populated per transition in this phase: the live window grab is async-only, and
  a text-rasterised image is explicitly pointless per Req 1.3/23.2. This matches
  Req 9.13's "where the capture configuration enables it" for the picture and
  satisfies Req 9.14 (no egui screengrab in the per-transition path). The picture
  is included where it already is: the bare-SNAPSHOT clipboard composite
  (Phase 2B) and any future enable-flag. Documented as the chosen tradeoff.

- D3 -- Capture STOP Save-As is a MODAL OVERLAY, not a Workspace Context.
  Rationale: it is a transient name/type/location picker consistent with the
  existing `egui::Window`/`egui::Modal` dialogs in `shell/update_dialogs.rs`
  (RESET BARE, Unsaved-workspace, External-exec). A modal avoids a new TabKind +
  WorkspaceDescriptor + focus-latch + first-Tab test for an ephemeral picker, and
  Req 24.6 explicitly allows the modal alternative ("IF the Save-As dialog is a
  Workspace Context ... ELSE keep it consistent with the existing dialog
  pattern"). Full dialog-capture is Phase 2E, so the modal itself need not be
  capturable now. The modal DISPATCHES the existing `CAPTURE SAVE` /
  `CAPTURE EXPORT ...` verbs through the single front door (`handle_command`) --
  command parity, no new persistence path (Req 24.4).

- D4 -- Replay picker is also a MODAL OVERLAY that lists `.ffscrm` files in
  `scrm_dir()` and, on selection, dispatches `CAPTURE REPLAY <file>` through the
  front door. The ScrmViewer Context itself is UNCHANGED and stays a
  `WorkspaceContext` (it already has `InteriorFocus` + a first-interior id). The
  only Context-facing change is making `CAPTURE REPLAY <file>` load-then-open (it
  currently ignores its arg and only replays the active in-memory collection).
  This keeps the mandatory first-Tab egui_kittest obligation satisfied by the
  existing viewer; the picker is a modal like D3. No new Context, no phantom-stop
  risk.

- D5 -- `TriggeringCommand` lives in `ff-scrm/model.rs` as a new serializable
  type on `ScreenCapture`, representable as ABSENT via `Option` (Req 9.12), and
  round-trips through the zip header (persist.rs serializes the whole
  `ScreenCollection` to `collection.yaml`, so adding a `serde` field is
  automatic -- verified in persist.rs). It carries the canonical command string
  AND the resolved target kind (Req 9.10, 17.2a). The kind is stored as a plain
  `String` label (e.g. "Function", "CustomWorkspace") to avoid an `ff-scrm ->
  ff-command` dependency; the shell maps the `CommandTarget` to that label.

## Commit 1 -- C.5: capture START per-transition metadata (model + hook)

Maps to: screen-snapshot-scrm Req 9.10, 9.11, 9.12, 9.13 (text/markdown only),
9.14, 17.2, 17.2a.

- [ ] 1. Add the `TriggeringCommand` type and field to the SCRM model.
      Add `pub struct TriggeringCommand { pub command: String, pub target_kind:
      String }` (derive Debug/Clone/PartialEq/Eq/Serialize/Deserialize) and a
      field `pub triggering_command: Option<TriggeringCommand>` on `ScreenCapture`,
      defaulted to `None` in `ScreenCapture::new`. Add a small constructor/setter
      so the session can attach it. Keep `image` as-is (unchanged). Re-export
      `TriggeringCommand` from `ff-scrm` lib.rs.
      Files: `crates/ff-scrm/src/model.rs`, `crates/ff-scrm/src/lib.rs`
      Verify: `cargo test -p ff-scrm model` -- new unit test
      `capture_carries_triggering_command_and_round_trips_absent_and_present`
      passes; the existing persist round-trip test still passes (the field
      serializes through the zip header automatically).

- [ ] 2. Thread the triggering command into the capture call.
      Extend `ScrmSession::capture` (`scrm_session.rs`) to accept an
      `Option<TriggeringCommand>` (or add a sibling `capture_with_command`) and
      attach it to the appended `ScreenCapture`. Keep the existing `capture`
      signature working for `CAPTURE SCREEN` (manual capture = no command) by
      passing `None`, OR update its two call sites (`handle_capture` SCREEN arm
      and `auto_capture_active_context`). Prefer one method taking the option to
      avoid duplication.
      Files: `crates/ff-desktop/src/scrm_session.rs`
      Verify: `cargo test -p ff-desktop scrm_session` -- new test
      `capture_records_triggering_command_when_present` passes; existing
      `capture_appends_sequentially` / `capture_auto_starts_default_collection`
      still pass.

- [ ] 3. Record the triggering command at the auto-capture hook, from the single
      dispatch seam (NO bespoke intercept).
      The hook `auto_capture_active_context` runs on Context transition. Capture
      the command that CAUSED the transition from the already-resolved dispatch
      result: stash the last dispatched canonical command + its resolved
      `CommandTarget` kind on the shell during dispatch (reuse the existing
      front-door path in `shell/dispatch.rs` / `target_dispatch.rs` -- add a
      small `last_dispatched: Option<(String, String)>` field, set where
      `resolve_target` / `dispatch_command_target` already run, read+clear in the
      hook). A transition caused by a timed interval or a rule leaves it `None`
      (Req 9.12). Map the `CommandTarget` variant to its label string in the
      shell (so `ff-scrm` stays free of `ff-command`).
      Files: `crates/ff-desktop/src/shell/commands_scrm.rs` (hook), the dispatch
      seam file that owns `resolve_target` result (`shell/dispatch.rs` and/or
      `shell/target_dispatch.rs`), a grouped field on the shell state
      (`shell/state_groups.rs` or `shell/state.rs` -- add to an existing SCRM/
      dispatch sub-struct, not a new flat god-struct field, per wiring-standard).
      Verify: `cargo test -p ff-desktop` scoped to the new test
      `auto_capture_records_triggering_command_for_command_transition` (a
      command-caused transition records the command; an interval/rule capture
      records `None`). Also `cargo clippy -p ff-desktop -- -D warnings` and
      `cargo fmt -p ff-desktop -- --check` clean.

- [ ] 4. Surface the triggering command in replay (selectable "caused by" line).
      In the ScrmViewer, when the current capture carries a `triggering_command`,
      show a selectable "caused by: <command>" line above the screen text; omit
      it when absent (Req 10.9, 10.10). Render from the staged collection's
      current capture.
      Files: `crates/ff-desktop/src/scrm_viewer_panel.rs`
      Verify: `cargo test -p ff-desktop` -- new viewer test
      `replay_shows_cause_line_only_when_command_present` passes; existing viewer
      tests still pass.

## Commit 2 -- C.6: capture STOP Save-As modal (with the ffwb-domain default)

Maps to: screen-snapshot-scrm Req 24.1, 24.2, 24.3, 24.4, 24.5, 7.6; 12.1-12.5.

- [ ] 5. Add the capture-STOP Save-As modal state.
      Add a grouped SCRM-UI state (e.g. `ScrmUiState { save_as: Option<SaveAsState> }`)
      to the shell state sub-structs (NOT a flat field). `SaveAsState` holds: the
      session NAME buffer (pre-filled from the active collection name), the
      DIRECTORY buffer (pre-filled from `scrm_dir()` as a string, editable so the
      user can navigate away), and the chosen output TYPE (enum: Ffscrm archive,
      Text, Markdown, Html, Pdf -- the IMPLEMENTED subset; ODT/ODP listed as
      greyed/deferred or omitted per Req 24.2).
      Files: `crates/ff-desktop/src/shell/state_groups.rs` (new sub-struct),
      `crates/ff-desktop/src/shell/state.rs` (field + accessor if needed)
      Verify: `cargo check -p ff-desktop` compiles; `cargo fmt -p ff-desktop
      -- --check` clean.

- [ ] 6. Change CAPTURE STOP to open the modal instead of stopping silently.
      In `handle_capture`'s `"STOP"` arm (`commands_scrm.rs`): still STOP the
      session (Req 7.6 -- `self.scrm.stop()` ends auto-capture and keeps the
      in-memory collection), THEN, when a non-empty collection exists, open the
      Save-As modal (set `scrm_ui.save_as = Some(SaveAsState::new(default_name,
      scrm_dir()))`). When there is no collection to save, keep the current
      status message and open no modal.
      Files: `crates/ff-desktop/src/shell/commands_scrm.rs`
      Verify: `cargo test -p ff-desktop` -- new test
      `capture_stop_opens_save_as_modal_when_collection_present` and
      `capture_stop_without_collection_opens_no_modal` pass.

- [ ] 7. Render the Save-As modal and dispatch the chosen output via the front
      door (command parity, Req 24.4).
      New module `crates/ff-desktop/src/scrm_save_as_dialog.rs` renders the
      modal (pattern-matched to the existing `egui::Window`/`egui::Modal`
      dialogs in `update_dialogs.rs`): a name field, a directory field
      (editable), the output-type selector, and Save / Cancel buttons. The modal
      is INVOKED from `render_overlays` in `update_dialogs.rs` guarded by
      `scrm_ui.save_as.is_some()` (keep the file under 400 lines -- the render
      lives in the new module, `update_dialogs.rs` only calls it). On SAVE, build
      the full path from the name+dir and dispatch the matching EXISTING verb
      through `self.handle_command(...)`: Ffscrm -> `CAPTURE SAVE <path>`;
      Text/Markdown/Html/Pdf -> `CAPTURE EXPORT <FMT> <path>`. This reuses the
      existing export/save code paths (Req 24.4) and embeds the per-capture
      bitmap alongside text where the format already supports it (HTML/PDF via the
      existing exporters -- Req 24.3; no new embedding code needed here).
      Files: `crates/ff-desktop/src/scrm_save_as_dialog.rs` (new),
      `crates/ff-desktop/src/shell/update_dialogs.rs` (call the new render + apply
      outcome), `crates/ff-desktop/src/lib.rs` or `main.rs` (register `mod
      scrm_save_as_dialog;` wherever the other top-level modules are declared)
      Verify: `cargo test -p ff-desktop` -- new test
      `save_as_save_dispatches_capture_save_for_ffscrm` and
      `save_as_save_dispatches_capture_export_for_pdf` (assert the dispatched
      command string and that the file lands under a TempDir `dir_overrides.scrm`).
      `cargo clippy -p ff-desktop -- -D warnings` clean.

- [ ] 8. CANCEL stops the capture but writes nothing (Req 24.5).
      On Cancel (or Escape / dimmed-background click per the modal pattern),
      clear `scrm_ui.save_as` and write no file; the in-memory collection remains
      available (already stopped in step 6).
      Files: `crates/ff-desktop/src/scrm_save_as_dialog.rs`,
      `crates/ff-desktop/src/shell/update_dialogs.rs`
      Verify: `cargo test -p ff-desktop` -- new test
      `save_as_cancel_writes_nothing_but_capture_is_stopped` passes.

## Commit 3 -- C.7: replay save-to-file + list/open/select + export

Maps to: screen-snapshot-scrm Req 10.11, 10.12, 10.13, 10.14, 11.1, 16.1, 16.2.

- [ ] 9. Make CAPTURE REPLAY <file> / CAPTURE LOAD <file> open a saved session
      for replay (not only the active in-memory collection).
      In `handle_capture`: when `REPLAY` has a non-empty arg, LOAD the archive
      first (reuse the existing `handle_capture_load` -> `ff_scrm::load_archive`
      into `self.scrm.load(...)`), then `open_scrm_viewer()`. `CAPTURE LOAD
      <file>` already loads; keep it, and have REPLAY-with-arg reuse that load
      path so there is one loader. `CAPTURE REPLAY` with NO arg opens the picker
      (step 10). The viewer stages `self.scrm.active_collection()` as today
      (`render_body.rs`), so once loaded it replays the saved session unchanged.
      Files: `crates/ff-desktop/src/shell/commands_scrm.rs`
      Verify: `cargo test -p ff-desktop` -- new test
      `capture_replay_with_file_loads_then_opens_viewer` (load a saved .ffscrm
      from a TempDir, assert the active collection is the loaded one and the
      active tab is the ScrmViewer).

- [ ] 10. Add the saved-session picker modal (list .ffscrm in scrm_dir, select ->
      CAPTURE REPLAY <file>).
      New module `crates/ff-desktop/src/scrm_session_picker.rs`: a modal listing
      the `*.ffscrm` files found in `scrm_dir()` (read the dir; show file stems;
      empty-state label when none). On selection, dispatch `CAPTURE REPLAY
      <absolute-path>` through `self.handle_command(...)` (front door, command
      parity) and close. Cancel/Escape closes with no change. Bare `CAPTURE
      REPLAY` (step 9) sets `scrm_ui.replay_picker = Some(...)`; the modal is
      rendered from `render_overlays` guarded by that option.
      Files: `crates/ff-desktop/src/scrm_session_picker.rs` (new),
      `crates/ff-desktop/src/shell/state_groups.rs` (add `replay_picker` to the
      SCRM-UI sub-struct from step 5), `crates/ff-desktop/src/shell/commands_scrm.rs`
      (bare REPLAY opens the picker), `crates/ff-desktop/src/shell/update_dialogs.rs`
      (render + apply), module registration alongside step 7's module.
      Verify: `cargo test -p ff-desktop` -- new tests
      `replay_picker_lists_ffscrm_files_in_scrm_dir` (seed two .ffscrm files in a
      TempDir override, assert both listed) and
      `replay_picker_select_dispatches_capture_replay_with_path`. `cargo clippy
      -p ff-desktop -- -D warnings` clean.

- [ ] 11. Confirm saved-session export to PDF works end-to-end (reuse, no new
      path).
      Verify a loaded/active collection exports via the EXISTING `CAPTURE EXPORT
      PDF` path (`handle_capture_export` -> `ff_scrm::export_pdf`), keeping screen
      content as selectable text (Req 10.13, 12.6). ODT/ODP remain deferred
      (recorded NOT-COVERED). No code change expected beyond ensuring the picker/
      Save-As type list offers PDF; add a test only if a gap is found.
      Files: (verification only) `crates/ff-desktop/src/shell/commands_scrm.rs`
      Verify: `cargo test -p ff-desktop` -- test
      `saved_session_exports_to_pdf` (load a .ffscrm, dispatch `CAPTURE EXPORT
      PDF <path>` into a TempDir, assert a non-empty .pdf is written). Existing
      `ff-scrm` pdf tests still pass.

## Final verification (agent scope -- compile-verify + scoped tests)

- [ ] 12. Scoped gate on the touched crates and read the logs back.
      Run, reading each log per terminal-discipline:
      `cargo check -p ff-scrm -p ff-desktop`,
      `cargo clippy -p ff-scrm -p ff-desktop -- -D warnings`,
      `cargo fmt -p ff-scrm -p ff-desktop -- --check`,
      and the scoped SCRM tests
      `cargo test -p ff-scrm` and `cargo test -p ff-desktop scrm`.
      Confirm no source .rs exceeds 400 non-test lines (the new dialog + picker
      modules keep `commands_scrm.rs`/`update_dialogs.rs` under the limit).
      The authoritative full `cargo gate` is the OWNER's manual run (gate-policy);
      do NOT run the whole-workspace suite as an agent.

## Digest

Build/test commands (gate-policy, scoped; owner runs full gate):
- `cargo check -p ff-scrm -p ff-desktop`
- `cargo clippy -p ff-scrm -p ff-desktop -- -D warnings`
- `cargo fmt -p ff-scrm -p ff-desktop -- --check`
- `cargo test -p ff-scrm`
- `cargo test -p ff-desktop scrm`

Files and the exact blocks to change:

- `crates/ff-scrm/src/model.rs` (currently ~190 lines incl. tests)
  - ADD `pub struct TriggeringCommand { pub command: String, pub target_kind:
    String }` with the standard derives.
  - ADD field `pub triggering_command: Option<TriggeringCommand>` to
    `pub struct ScreenCapture { ... }`; set `None` in
    `impl ScreenCapture { pub fn new(...) -> Self }`.
  - Tests: extend `mod tests` (append; well under 200 lines). Covered today:
    `append_capture_assigns_sequential_numbers`, `capture_carries_dialog_state`.
    Verified these pass currently (model is self-contained, no shell).

- `crates/ff-scrm/src/lib.rs`
  - Change `pub use model::{ScreenCapture, ScreenCollection};` to also export
    `TriggeringCommand`.

- `crates/ff-scrm/src/persist.rs` (serializes the whole `ScreenCollection` to the
  zip `collection.yaml` header via `serde_json::to_vec_pretty`) -- NO code change;
  the new serde field round-trips automatically. Covered by
  `archive_round_trips_collection_header` (passes today).

- `crates/ff-desktop/src/scrm_session.rs` (~250 lines incl. tests)
  - `pub fn capture(&mut self, model, screen_name)` -> accept an
    `Option<TriggeringCommand>` (or add `capture_with_command`); attach to the
    appended capture via `collection.screens.last_mut()`. Tests covering today:
    `capture_appends_sequentially`, `capture_auto_starts_default_collection`
    (both pass).

- `crates/ff-desktop/src/shell/commands_scrm.rs` (~310 lines; keep under 400)
  - `fn handle_capture(&mut self, rest)` -- `"STOP"` arm: stop then open Save-As
    modal when a collection exists; `"REPLAY"` arm: with arg -> load-then-open,
    no arg -> open picker.
  - `fn auto_capture_active_context(&mut self)` -- read+clear the shell's
    `last_dispatched` command and pass it to `scrm.capture(...)`.
  - Reuse `handle_capture_load` / `resolve_scrm_path` / `scrm_dir` (unchanged).

- dispatch seam (`crates/ff-desktop/src/shell/dispatch.rs` and/or
  `shell/target_dispatch.rs`) -- set `last_dispatched: Option<(String, String)>`
  where `resolve_target` / `dispatch_command_target` already run (map
  `CommandTarget` -> label string here). Field added to an existing SCRM/dispatch
  sub-struct in `shell/state_groups.rs`.

- `crates/ff-desktop/src/shell/state_groups.rs` -- ADD a grouped `ScrmUiState`
  (`save_as: Option<SaveAsState>`, `replay_picker: Option<ReplayPickerState>`)
  and the `last_dispatched` field; mirror existing sub-struct style
  (`SnapshotState`, `FocusState`). `state.rs` holds the field + accessor.

- `crates/ff-desktop/src/scrm_save_as_dialog.rs` (NEW) -- `SaveAsState` + a
  `render(ctx, &mut SaveAsState) -> Outcome` modal mirroring
  `about_dialog::render` / the RESET BARE modal. Outcome drives
  `handle_command("CAPTURE SAVE|EXPORT ...")`.

- `crates/ff-desktop/src/scrm_session_picker.rs` (NEW) -- list `*.ffscrm` in
  `scrm_dir()`, modal select -> `handle_command("CAPTURE REPLAY <path>")`.

- `crates/ff-desktop/src/shell/update_dialogs.rs` (~330 lines; keep under 400)
  - In `fn render_overlays(&mut self, ctx)` ADD two guarded blocks that call the
    new modules' render fns and apply their outcomes (same shape as the existing
    `if self.show_about { ... }` / RESET BARE blocks). Note the old SNAPSHOT
    format-picker is already retired here (comment present).

- `crates/ff-desktop/src/scrm_viewer_panel.rs` (~230 lines incl. tests)
  - `fn current_screen_text` / `render` -- prepend a selectable "caused by:
    <command>" line when the current capture has `triggering_command`
    (Req 10.9/10.10). ScrmViewer stays a `WorkspaceContext` with its existing
    `InteriorFocus { first, last: first }` -- the mandatory first-Tab
    egui_kittest obligation is already met by the existing viewer; NOT re-added.

- module registration -- add `mod scrm_save_as_dialog;` and `mod
  scrm_session_picker;` wherever `scrm_viewer_panel` / `about_dialog` are declared
  (top-level `lib.rs`/`main.rs` module list).

- `crates/ff-desktop/src/shell/render_body.rs` -- NO change; the
  `TabKind::ScrmViewer` arm already stages
  `self.scrm.active_collection().cloned()` onto the viewer each frame, so a
  loaded saved session replays with no viewer-staging change.

What I verified by reading code (not assumed):
- `scrm_dir()` -> `<User_Data_Dir>/screen-collections/` via
  `UserDataDir::resolve(None)` with a `dir_overrides.scrm` test override
  (`shell/commands_session.rs`, `shell/state_groups.rs`) -- this is the ffwb
  domain captures dir; no second scheme needed.
- `active_screen_capture` provider seam + providers for every Context already
  landed (`shell/screen_provider.rs`); C.1-C.3 are DONE, so C.5 only adds command
  metadata, not provider work.
- The Phase 2B window-screenshot path is async (`SnapshotState.pending` +
  `screenshot_requested`, resolved on `Event::Screenshot` in `shell/update.rs`)
  -> per-transition real picture is not feasible synchronously (D2).
- `ScreenCapture` already has `image`, `screen`, `metadata`; ANSI/Markdown are
  rendered on demand from `screen` (`ff_screen_model::render`), so no stored
  per-flavor copies are needed (D5 rationale).
- `handle_capture` STOP currently just calls `self.scrm.stop()`; REPLAY ignores
  its arg and opens the viewer on the active collection only -- the two behaviours
  C.6 and C.7 change.
- `ff_scrm::{save_archive, load_archive, export_pdf, export_text/markdown/html}`
  and `ReplaySession` exist and are the reuse points (lib.rs); persist serializes
  the whole collection so the new field round-trips.
- Dialog pattern: `egui::Window`/`egui::Modal` overlays driven by shell state
  flags, rendered in `render_overlays` with a `modal_open` guard
  (`shell/update_dialogs.rs`) -- the Save-As and picker modals follow it.
