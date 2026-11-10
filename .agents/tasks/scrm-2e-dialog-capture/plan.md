# Implementation Plan -- Stream SCRM Phase 2E: message/dialog box capture (Requirement 27)

Worktree: `c:/workspace/apps/FileForgeWorkbench/.worktrees/scrm-2e-dialog-capture` (branch `scrm-2e`).
Git: `git -C c:/workspace/apps/FileForgeWorkbench/.worktrees/scrm-2e-dialog-capture ...`.

## Design decisions (grounded in the code read)

1. **Redaction is a pure model transform in `ff-screen-model`.** The model already
   carries `FieldAttributes::sensitive` (model.rs), but NO renderer masks a sensitive
   field's value -- text/markdown/html/ansi emit `f.value` in clear, and yaml adds a
   `sensitive: true` marker yet STILL prints `value: hunter2`. So redaction does not
   exist yet. Rather than patch five renderers (and the raster/PDF paths), add ONE pure
   function `ScreenModel::redacted()` (and `Field::redacted_value()`) that returns a
   clone whose every `sensitive` field value is replaced by the ASCII placeholder
   `[REDACTED]`. Redacting the MODEL once, before any render, means the model, the
   direct-copy text, the composite TEXT/HTML flavors, and even the PNG raster are all
   redacted from the single source -- satisfying 27.4 "model AND clipboard" with no
   second mechanism and no renderer edits. The `sensitive` flag is the declared opt-out
   (27.5), analogous to `InteriorFocus::none()`.

2. **Frontmost-surface selection stays on the existing `active_screen_capture` seam.**
   `active_screen_capture()` (screen_provider.rs) returns the active Context's model via
   the per-`KindTag` dispatch. Add a check at the TOP: if a transient dialog/message
   overlay is open, return the DIALOG's `ScreenCapture::Model` (redacted); otherwise fall
   through to the existing Context dispatch unchanged. This is the one seam SNAPSHOT and
   CAPTURE already use, so 27.2 and 27.7 come for free -- SNAPSHOT already renders from
   the model (commands_snapshot.rs), never an egui grab (the window SCREENSHOT picture is
   the existing Phase 2B capability and is orthogonal to the text model).

3. **A `DialogScreen` source built from each dialog's OWN state.** Add a small trait
   `DialogScreen { fn screen_model(&self) -> ScreenModel; }` and a shell method
   `active_dialog_capture() -> Option<ScreenModel>` that inspects the SAME flags
   `modal_open` is computed from (update_input.rs: `show_about`, `pending_external`,
   `reset_bare_confirm`, `show_unsaved_workspace_dialog`, `show_history_list`,
   `show_swap_list`, `scrm_ui.save_as`, `scrm_ui.replay_picker`) plus a transient
   message/error popup, picks the frontmost open one, and builds its model from the
   dialog's own fields (title + body message + field labels/values + list rows + button
   labels). This mirrors the per-Context `screen_providers.rs` builders but for overlays.

4. **The direct COPY affordance is a new front-door `COPY` verb + a shared helper.**
   Add a `COPY` arm beside the `SNAPSHOT` arm in `commands_ladder_b.rs` (same ladder,
   single dispatch -- no second dispatcher, no bespoke intercept). It routes to
   `handle_copy()`, which takes the frontmost surface's model (dialog if open, else the
   active Context), renders the redacted plain-text, and places it on the clipboard via a
   text-only path on the existing `SnapshotClipboard` backend -- NOT the full SNAPSHOT
   screenshot machinery (27.3). A dialog may ALSO expose a Copy button / Ctrl+C that
   dispatches the same `COPY` verb (the UI affordance invokes the command, never the
   handler directly -- wiring-standard).

5. **Secret declaration lives on the dialog state, mapped to `sensitive` at model-build
   time.** A `DialogScreen` builder marks a field secret by setting
   `FieldAttributes::sensitive` on that field (the existing builder). No secret-bearing
   dialog exists in the parked/core set today, so FEAT-002 ships a synthetic
   `DialogScreen` fixture in the test module to prove the mechanism end to end.

6. **File-size discipline.** New dialog-model/redaction logic goes in its own modules
   (`ff-screen-model/src/redact.rs`; `ff-desktop/src/shell/dialog_screen.rs` +
   `dialog_capture.rs`). Keep each under 400 non-test lines; split test files before
   ~200 lines. ASCII-only source; the placeholder `[REDACTED]` is ASCII.

## Build / test commands (scoped, per gate-policy)

- `cargo check -p ff-screen-model` / `-p ff-desktop`
- `cargo clippy -p ff-screen-model -- -D warnings` / `-p ff-desktop -- -D warnings`
- `cargo fmt -p ff-screen-model -- --check` / `-p ff-desktop -- --check`
- Scoped tests: `cargo test -p ff-screen-model redact`, `cargo test -p ff-desktop dialog_capture`,
  `cargo test -p ff-desktop dialog_screen`. The authoritative full `cargo gate` is the owner's.

---

## Ordered steps (TDD-first)

- [ ] 1. Add a failing unit test for model redaction in `ff-screen-model`.
      Create `crates/ff-screen-model/src/redact.rs` with a `#[cfg(test)]` test
      `redacted_masks_only_sensitive_field_values` annotated `// Validates: screen-snapshot-scrm Req 27.4, 27.5`:
      a `ScreenModel` with a plain field (`User`=`alan`) and a sensitive field
      (`Password`=`hunter2`, `FieldAttributes::plain().sensitive()`); assert the redacted
      clone's sensitive value equals `[REDACTED]` and the secret string `hunter2` is ABSENT,
      while the non-secret `alan` is still present (27.5). Add `pub const REDACTED_PLACEHOLDER: &str = "[REDACTED]";`.
      Files: `crates/ff-screen-model/src/redact.rs`, `crates/ff-screen-model/src/lib.rs` (add `mod redact;` + re-export).
      Verify: `cargo test -p ff-screen-model redact` -- test FAILS to compile/assert (red).

- [ ] 2. Implement `ScreenModel::redacted()` and `Field::redacted_value()`.
      In `redact.rs` add `impl ScreenModel { pub fn redacted(&self) -> ScreenModel }` returning a
      clone whose every field with `attrs.sensitive == true` has its `value` replaced by
      `REDACTED_PLACEHOLDER` (label and attrs preserved). Keep the function under ~40 lines.
      Files: `crates/ff-screen-model/src/redact.rs`.
      Verify: `cargo test -p ff-screen-model redact` -- test PASSES (green); `cargo clippy -p ff-screen-model -- -D warnings` clean.

- [ ] 3. Add a failing test that EVERY text flavor of a redacted model omits the secret.
      In `redact.rs` tests add `redacted_model_omits_secret_in_every_text_format` annotated
      `// Validates: screen-snapshot-scrm Req 27.4`: render the redacted model via
      `render(.., fmt, RenderOptions::default())` for PlainText, Ansi, Markdown, Html, Yaml and
      assert none contains `hunter2` and each contains `[REDACTED]`. (yaml.rs currently prints the
      value in clear, so this proves redaction happens at the MODEL level, upstream of yaml.)
      Files: `crates/ff-screen-model/src/redact.rs`.
      Verify: `cargo test -p ff-screen-model redact` -- new test PASSES (model is already redacted before render).

- [ ] 4. Add a failing test for the `DialogScreen` trait + a synthetic secret-bearing dialog.
      Create `crates/ff-desktop/src/shell/dialog_screen.rs` with `pub(crate) trait DialogScreen { fn screen_model(&self) -> ff_screen_model::ScreenModel; }`
      and a `#[cfg(test)]` synthetic `struct FakeSecretDialog` implementing it (title + a body
      message + a plain `Host` field + a `Password` field marked
      `FieldAttributes::plain().sensitive()` + a `[OK]` button). Test
      `dialog_screen_marks_declared_secret_field` annotated `// Validates: screen-snapshot-scrm Req 27.1, 27.4, 27.5`:
      build the model, assert title/message/button present and the password field's `attrs.sensitive` is true
      while a non-secret field is copyable (value present, not redacted at this stage).
      Files: `crates/ff-desktop/src/shell/dialog_screen.rs`, `crates/ff-desktop/src/shell/mod.rs` (add `mod dialog_screen;`).
      Verify: `cargo test -p ff-desktop dialog_screen` -- FAILS to compile (trait/module absent) then PASSES once the trait + fixture compile (red->green for the trait shape).

- [ ] 5. Implement real `DialogScreen` builders for the live transient overlays.
      In `dialog_screen.rs` implement `screen_model()` for each open-overlay source that exists in the
      core build: the RESET BARE confirmation (`reset_bare_confirm` target: heading + profile rows +
      Confirm/Cancel buttons), the external-run confirmation (`pending_external`: body + command string
      + Run/Cancel), the unsaved-workspace dialog, the About dialog, the history list, the SWAP picker,
      and the SCRM Save-As / replay-picker (title + name/folder/type fields + Save/Cancel). Build each
      from the dialog's OWN state struct (NOT egui). Each builder is a small free fn or `impl`; keep the
      file under 400 lines (split into `dialog_screen_scrm.rs` if it approaches the limit).
      Files: `crates/ff-desktop/src/shell/dialog_screen.rs` (+ optional `dialog_screen_scrm.rs`).
      Verify: `cargo check -p ff-desktop`; `cargo clippy -p ff-desktop -- -D warnings` clean.

- [ ] 6. Add a failing test for `active_dialog_capture()` frontmost selection.
      Create `crates/ff-desktop/src/shell/dialog_capture.rs` with a `#[cfg(test)]` test
      `dialog_open_captures_dialog_not_context_behind` annotated `// Validates: screen-snapshot-scrm Req 27.2`:
      make a shell on a POM Context, open a transient dialog (e.g. set `pending_external` via the real
      path or `reset_bare_confirm`), call the new `active_dialog_capture()` and assert it returns `Some`
      with the DIALOG's title/body, distinct from the POM Context model. Also test
      `no_dialog_open_returns_none`.
      Files: `crates/ff-desktop/src/shell/dialog_capture.rs`, `crates/ff-desktop/src/shell/mod.rs` (add `mod dialog_capture;`).
      Verify: `cargo test -p ff-desktop dialog_capture` -- FAILS (method absent).

- [ ] 7. Implement `active_dialog_capture()` and route `active_screen_capture()` through it.
      In `dialog_capture.rs` add `impl WorkbenchShell { pub(super) fn active_dialog_capture(&self) -> Option<ff_screen_model::ScreenModel> }`
      that checks the frontmost open overlay (same precedence as the `modal_open` OR-chain in
      update_input.rs), builds its `DialogScreen` model, and returns `Some(model.redacted())` (redaction
      applied HERE so every caller gets the masked model); `None` when no overlay is open. Then at the TOP
      of `active_screen_capture()` (screen_provider.rs): `if let Some(m) = self.active_dialog_capture() { return ScreenCapture::Model(Box::new(m)); }`
      before the existing `KindTag` match (which is left byte-for-byte unchanged).
      Files: `crates/ff-desktop/src/shell/dialog_capture.rs`, `crates/ff-desktop/src/shell/screen_provider.rs`.
      Verify: `cargo test -p ff-desktop dialog_capture` -- PASSES (green); existing `screen_provider` tests still pass (`cargo test -p ff-desktop screen_provider`).

- [ ] 8. Add a failing test that SNAPSHOT captures the dialog surface (redacted) via the Phase 2B path.
      In `dialog_capture.rs` tests add `snapshot_text_for_active_captures_frontmost_redacted_dialog`
      annotated `// Validates: screen-snapshot-scrm Req 27.2, 27.4, 27.7`: with a synthetic secret dialog
      wired as the frontmost surface (inject via a test hook that sets the overlay state), call the pure
      `snapshot_text_for_active("")` (commands_snapshot.rs) and assert the returned text contains the
      dialog title + `[REDACTED]` and does NOT contain the secret value -- proving SNAPSHOT renders from
      the dialog's logical model (not the Context, not an egui grab).
      Files: `crates/ff-desktop/src/shell/dialog_capture.rs`.
      Verify: `cargo test -p ff-desktop dialog_capture` -- PASSES (no production change needed; step 7 already routed `active_screen_capture`). If a test-only overlay hook is needed, add it behind `#[cfg(test)]`.

- [ ] 9. Add a failing test for the direct COPY helper (text-only, no screenshot).
      Add a text-only method to the clipboard abstraction: extend `SnapshotClipboard` with
      `fn place_text(&mut self, text: &str) -> Result<(), String>` (default impl builds a text-only
      `SnapshotPayload`-style path OR a dedicated `CF_UNICODETEXT`/`arboard` write), and give
      `OsClipboard` a real impl. In `clipboard_backend.rs` tests add `place_text_offers_text_only`
      (FakeClipboard records the text) annotated `// Validates: screen-snapshot-scrm Req 27.3`.
      Then in `dialog_capture.rs` add `handle_copy()` test `copy_places_frontmost_redacted_text_without_screenshot`
      annotated `// Validates: screen-snapshot-scrm Req 27.3, 27.4`: assert the copied text equals the
      redacted plain-text of the frontmost surface and that no `snapshot.screenshot_requested` flag was set.
      Files: `crates/ff-desktop/src/clipboard_backend.rs`, `crates/ff-desktop/src/shell/dialog_capture.rs`.
      Verify: `cargo test -p ff-desktop clipboard_backend dialog_capture` -- FAILS (method/handler absent).

- [ ] 10. Implement `handle_copy()` and the `COPY` front-door verb.
       Add `impl WorkbenchShell { pub(super) fn handle_copy(&mut self) }` in `dialog_capture.rs`: take the
       frontmost model (`active_dialog_capture()` else `active_screen_capture().into_model()`), render
       redacted plain-text via `render_snapshot(.., PlainText)` (the model is already redacted), and place
       it with `OsClipboard.place_text(..)`; set a success/failure status via an i18n key. Add the `COPY`
       arm in `commands_ladder_b.rs` beside the `SNAPSHOT` arm (`if upper == "COPY" { self.handle_copy(); return true; }`).
       Files: `crates/ff-desktop/src/shell/dialog_capture.rs`, `crates/ff-desktop/src/shell/commands_ladder_b.rs`,
       plus the i18n `.ftl` for the `COPY` status keys (locate via existing `ff-desktop-shell-msg-scrm-snapshot-*` keys).
       Verify: `cargo test -p ff-desktop dialog_capture clipboard_backend` -- PASSES (green); `cargo clippy -p ff-desktop -- -D warnings` clean.

- [ ] 11. Add a full-shell COPY integration test and confirm the composite SNAPSHOT is redacted.
       In `dialog_capture.rs` (or a sibling `dialog_capture_tests.rs` if the file nears 200 test lines) add
       `full_shell_copy_command_copies_dialog_text` using the egui_kittest `harness_shell()` (model on the
       SCRM harness pattern): open a dialog via the real command path, run `handle_command("COPY")` through
       the front door, and assert the copied text (captured via a test clipboard seam or the pure helper)
       is the dialog's redacted text. Add `composite_text_flavor_for_dialog_is_redacted` asserting the
       Markdown flavor built from the frontmost (secret) dialog omits the secret and contains `[REDACTED]`
       (27.4 on the composite). Both annotated `// Validates: screen-snapshot-scrm Req 27.3, 27.4`.
       Files: `crates/ff-desktop/src/shell/dialog_capture.rs` (or `dialog_capture_tests.rs`).
       Verify: `cargo test -p ff-desktop dialog_capture` -- PASSES.

- [ ] 12. Context-dialog contract regression check (27.6) + final scoped gate.
       Confirm NO change to the focus-latch / InteriorFocus path: a Context that happens to be a dialog-like
       editor still satisfies its first-Tab contract. Run the existing focus tests
       (`full_shell_config_first_tab_focuses_filter_field`, `..._theme_editor_...`) UNCHANGED to prove 27.6
       (this Req only ADDS the transient-overlay binding; Context behaviour is untouched). Then run the full
       scoped trio on both crates.
       Files: none (verification only).
       Verify: `cargo test -p ff-desktop first_tab` passes unchanged; `cargo check -p ff-screen-model -p ff-desktop`,
       `cargo clippy -p ff-screen-model -p ff-desktop -- -D warnings`, `cargo fmt -p ff-screen-model -p ff-desktop -- --check` all clean.
       Hand off to the owner's full `cargo gate --build`.

---

## Digest

### `crates/ff-screen-model/src/lib.rs`
- Add `mod redact;` and `pub use redact::REDACTED_PLACEHOLDER;` near the other `mod`/`pub use` lines (file currently declares `SnapshotFormat` and re-exports `render`/`RenderOptions`/model types).
- Verified: `render(model, format, options)` is the single render entry (render.rs); every text renderer emits `f.value` in clear; `SnapshotFormat` variants PlainText/Ansi/Markdown/Html/Yaml/Png/Pdf.

### `crates/ff-screen-model/src/redact.rs` (NEW)
- `pub const REDACTED_PLACEHOLDER: &str = "[REDACTED]";`
- `impl ScreenModel { pub fn redacted(&self) -> ScreenModel }` -- clone; for each field with `attrs.sensitive`, set `value = REDACTED_PLACEHOLDER.to_string()`.
- Tests: `redacted_masks_only_sensitive_field_values` (27.4, 27.5), `redacted_model_omits_secret_in_every_text_format` (27.4).
- Verified from model.rs: `Field { label, value, attrs }`, `FieldAttributes { .., sensitive }`, builder `FieldAttributes::plain().sensitive()`; from render.rs: `render(&ScreenModel, SnapshotFormat, RenderOptions)`.

### `crates/ff-desktop/src/shell/screen_provider.rs`
- Change block: top of `pub(super) fn active_screen_capture(&self) -> ScreenCapture` (currently opens with `let tab = self.tabs.active_tab(); match tab.kind.tag() { .. }`).
- Insert BEFORE the `let tab`: `if let Some(m) = self.active_dialog_capture() { return ScreenCapture::Model(Box::new(m)); }`.
- Test covering it: existing `menu_workspace_dispatches_to_provider` / `non_menu_context_now_provides_a_model` (unchanged, prove the Context path still works when no dialog is open) + new `dialog_capture` tests.
- Verified: `ScreenCapture::{Model(Box<ScreenModel>), OptOut(&'static str)}`, `into_model()`.

### `crates/ff-desktop/src/shell/dialog_screen.rs` (NEW)
- `pub(crate) trait DialogScreen { fn screen_model(&self) -> ff_screen_model::ScreenModel; }`.
- `screen_model()` builders for: `reset_bare_confirm` target, `pending_external`, `show_unsaved_workspace_dialog` (+ `active_workspace.name`), `show_about`, `show_history_list`, `show_swap_list`, `scrm_ui.save_as` (`SaveAsState`: name/directory/output_type), `scrm_ui.replay_picker`. Each sets `FieldAttributes::sensitive` on any declared-secret field.
- `#[cfg(test)]` `FakeSecretDialog` fixture. Test: `dialog_screen_marks_declared_secret_field` (27.1, 27.4, 27.5).
- Verified dialog state from update_dialogs.rs / update_dialogs_scrm.rs / scrm_save_as_dialog.rs (`SaveAsState { name, directory, output_type }`, `OutputType`).

### `crates/ff-desktop/src/shell/dialog_capture.rs` (NEW)
- `impl WorkbenchShell`:
  - `pub(super) fn active_dialog_capture(&self) -> Option<ff_screen_model::ScreenModel>` -- frontmost overlay by the same precedence as the `modal_open` OR-chain in update_input.rs; returns `Some(model.redacted())`, else `None`.
  - `pub(super) fn handle_copy(&mut self)` -- frontmost model (dialog else Context), render redacted PlainText via `screen_snapshot::render_snapshot`, place with `OsClipboard.place_text`, set i18n status.
- Tests (27.2/27.3/27.4/27.7): `dialog_open_captures_dialog_not_context_behind`, `no_dialog_open_returns_none`, `snapshot_text_for_active_captures_frontmost_redacted_dialog`, `copy_places_frontmost_redacted_text_without_screenshot`, `full_shell_copy_command_copies_dialog_text`, `composite_text_flavor_for_dialog_is_redacted`. Split into `dialog_capture_tests.rs` if > ~200 test lines.
- Verified: `snapshot_text_for_active(&self, arg)` and `snapshot.screenshot_requested` in commands_snapshot.rs/state_groups.

### `crates/ff-desktop/src/shell/mod.rs`
- Add `mod dialog_screen;` and `mod dialog_capture;` beside the other `mod` lines.

### `crates/ff-desktop/src/shell/commands_ladder_b.rs`
- Change block: inside `try_commands_b1`, immediately after the `SNAPSHOT` arm (`if upper == "SNAPSHOT" || upper.starts_with("SNAPSHOT ") { .. self.handle_snapshot(arg); return true; }`), add `if upper == "COPY" { self.handle_copy(); return true; }`.
- Test: `full_shell_copy_command_copies_dialog_text` (front-door dispatch).

### `crates/ff-desktop/src/clipboard_backend.rs`
- Add to `trait SnapshotClipboard`: `fn place_text(&mut self, text: &str) -> Result<(), String>;` (Windows: open once, `CF_UNICODETEXT` via `unicode_text_bytes`; non-Windows: `arboard` `set_text`). `FakeClipboard` records it.
- Test: `place_text_offers_text_only` (27.3).
- Verified: existing `SnapshotPayload`, `OfferedFlavors`, `OsClipboard`, `unicode_text_bytes`, `FakeClipboard`.

### i18n `.ftl`
- Add `ff-desktop-shell-msg-copy-copied` / `-no-clipboard` keys alongside the existing `ff-desktop-shell-msg-scrm-snapshot-*` keys (locate the `.ftl` via grep on an existing snapshot key).

### What was verified today
- Read Req 27 (27.1-27.7) and master-directive Req 4.6; the redaction mechanism does NOT exist (renderers emit sensitive values in clear; yaml only adds a marker). The `sensitive` flag and builders DO exist.
- `active_screen_capture` is the single capture seam; SNAPSHOT (commands_snapshot.rs) renders from the model, never egui; the window screenshot is a separate Phase 2B picture flavor.
- `modal_open` is the canonical frontmost-overlay signal (update_input.rs OR-chain); dialog state structs read.
- SNAPSHOT dispatches via `commands_ladder_b.rs`; COPY follows the same ladder arm.
- Build/test commands per gate-policy: scoped `cargo check/clippy/fmt/test -p <crate>`; the authoritative full gate is the owner's.
- Did NOT run any build/test (planning only).
