# Implementation Plan -- Common Workspace Frame (CR-NR-115)

Worktree: `c:/workspace/apps/FileForgeWorkbench/.worktrees/common-frame-impl`
(branch `common-frame-impl`). Git via `git -C <worktree>`. TDD-first. STOP before
merge: no merge, no push -- the owner runs the full `cargo gate` + manual check.

Agents COMPILE-VERIFY only (gate-policy.md / testing.md execution split): scoped
`cargo check -p <crate>`, `cargo clippy -p <crate> -- -D warnings`,
`cargo fmt -p <crate> -- --check`, and the SCOPED new unit tests where quick. Do
NOT run the full `cargo test` / `cargo nextest` / `cargo gate` -- that is the
owner's manual run.

## Scope note settled during exploration (read before starting)

- The per-action dispatch seam the Message_Line clears on and `last_action_at`
  stamps at is NOT a single function today. User actions reach the shell through
  `begin_command_line()` (shared by the typed `run_command_line` and the
  function-key `dispatch_key_command`) and through `dispatch_bound_command()`
  (menu-option clicks that do not pass `begin_command_line`). The plan adds ONE
  private helper `note_action_dispatched()` and calls it at the top of BOTH
  seams. Double-invocation on the key path (begin_command_line then
  dispatch_bound_command) is harmless: clearing an already-cleared Vec is a
  no-op and re-stamping `last_action_at` in the same action is correct. This is
  the "reuse the existing per-action dispatch seam" the design calls for; it is
  additive, no new dispatch path (Req 7.27).
- `ScreenModel` already carries `header_lines` / `body_lines` / `footer_lines` /
  `tables` / `status_bar` / `command_line` slots (ff-screen-model/src/model.rs).
  No public-type reshape is needed (Req 7.27, 30.17).
- IMPORTANT: Phase 2G of screen-snapshot-scrm (Req 29, CR-CH-115 -- the "shared
  chrome helper" Phase 2H says it extends) is NOT implemented on this branch.
  The non-menu `*_screen_model()` builders in `screen_providers*.rs` do NOT add
  the window-title / menu-bar / tab-bar / status chrome, and the text renderer
  (`ff-screen-model/src/render/text.rs`) renders `model.tables` as
  `row.join("  ")` with no box-drawing grid. Only the menu/POM path
  (`menu_workspace_screen_model_full`) composes any chrome, and it deliberately
  omits the title/menu/tab block. Therefore the snapshot-reproduction work in
  this gate must CREATE the shared common-frame chrome helper (the 2G
  foundation) scoped to exactly the eight-element common frame Req 30 requires,
  plus the box-drawing grid in the text + html renderers. This is in-scope for
  CR-NR-115 (Req 30 reproduces the Req 7 frame); it is NOT a re-litigation of
  all of Req 29. Do not pull in Req 29 extras beyond the common-frame elements.
- The in-frame window-title text MUST mirror the ONE live title constant
  `"FileForge Workbench"` from `main.rs` (brand token, untranslated). Do not
  hardcode a second copy; reference a single shared `const`/helper (add one in
  `screen_snapshot.rs` or reuse `main.rs`'s constant exported as `pub const`).
- Preserve stable egui ids the full-shell focus tests depend on
  (`command_field_input`, panel ids), the single `render_workspace_context`
  focus latch, ISPF explicit-commit save, and single command dispatch. No source
  `.rs` over 400 non-test lines; split a test file before ~200 lines. ASCII-only
  in docs and pattern-matched source; box-drawing allowed in rendered-frame
  fixtures/tests only.

---

## A. Shell additive state + Status_Bar common-segment model (Phase CF.1-CF.5)

- [ ] 1. Add `message_line: Vec<String>` and `last_action_at:
      Option<chrono::DateTime<chrono::Local>>` additive fields to the shell
      state, plus a `note_action_dispatched()` helper that clears `message_line`
      and stamps `last_action_at = Some(Local::now())`. Write failing unit tests
      FIRST: setting messages makes them readable; calling the per-action seam
      clears them before the action; `last_action_at` is `None` initially and
      `Some` after an action.
      Files: `crates/ff-desktop/src/shell/state.rs` (fields + doc comments),
      `crates/ff-desktop/src/shell/actions.rs` (the `note_action_dispatched`
      helper near `format_session_start`),
      `crates/ff-desktop/src/shell/construct.rs` (init both fields:
      `message_line: Vec::new()`, `last_action_at: None`),
      `crates/ff-desktop/src/shell/tests_session.rs` (new unit tests).
      Verify: `cargo test -p ff-desktop message_line` and
      `cargo test -p ff-desktop last_action` -- new tests pass; `cargo check -p
      ff-desktop` clean.
      Validates: Requirement 7.11, 7.12, 7.13, 7.14, 7.16, 7.27.

- [ ] 2. Call `note_action_dispatched()` at the top of `begin_command_line()`
      and at the top of `dispatch_bound_command()` so every user action clears
      the Message_Line and stamps the time before the action runs. Write a
      failing test that seeds `message_line`, runs a command through
      `handle_command`/`run_command_line`, and asserts the Message_Line is empty
      afterward unless the action re-populated it.
      Files: `crates/ff-desktop/src/shell/commands_line.rs` (begin_command_line),
      `crates/ff-desktop/src/shell/target_dispatch.rs` (dispatch_bound_command),
      `crates/ff-desktop/src/shell/tests_session.rs` (test).
      Verify: `cargo test -p ff-desktop message_line_cleared` passes; existing
      command-line tests still pass (`cargo test -p ff-desktop command_line`).
      Validates: Requirement 7.12, 7.14, 7.16.

- [ ] 3. Render the Message_Line as a new `TopBottomPanel::bottom("message_line")`
      rendered AFTER `render_status_bar` in `update()` so it stacks directly
      ABOVE the Status_Bar; absent (early-return) when `message_line` is empty;
      one row per message (may expand). Add a `render_message_line(ctx)` method.
      Write a headless test rendering the panel and asserting it is absent when
      empty and shows N rows when N messages are set (mirror the
      `render_status_bar` headless test in tests_focus.rs).
      Files: `crates/ff-desktop/src/shell/render_status.rs` (new
      `render_message_line`), `crates/ff-desktop/src/shell/update.rs` (call it
      after `render_status_bar`), `crates/ff-desktop/src/shell/tests_focus.rs`
      or a new `tests_message_line.rs` (headless test; new file if tests_focus
      would exceed ~200 added lines).
      Verify: `cargo test -p ff-desktop message_line_panel` passes; `cargo
      clippy -p ff-desktop -- -D warnings` clean.
      Validates: Requirement 7.8, 7.13, 7.14.

- [ ] 4. In `render_status_bar`, remove the `RUNNING`/phase-label segment
      entirely (the `phase_label` match and its `ui.label(phase_label)`), replace
      the `Started: HH:MM` segment with `Last: HH:MM` sourced from
      `last_action_at` (fall back to a blank/`Last: --:--` when `None`), and
      shorten `active_profile_label()` from `Profile: <name>` to `Prof: <name>`.
      Add a pure `format_last_action()` helper mirroring `format_session_start`.
      Write failing tests FIRST: `format_last_action()` yields `Last: HH:MM` (and
      the `None` fallback); `active_profile_label()` yields `Prof: <name>` /
      `Prof: default`; a headless status-composition assertion that the
      registered `STATUSBAR_MESSAGE`/composed text contains no `RUNNING`.
      Files: `crates/ff-desktop/src/shell/actions.rs` (`format_last_action`,
      shorten `active_profile_label`), `crates/ff-desktop/src/shell/render_status.rs`
      (drop phase segment; use `format_last_action`),
      `crates/ff-desktop/src/shell/tests_session.rs` +
      `crates/ff-desktop/src/shell/tests_misc.rs` (update
      `active_profile_label_reflects_active_profile` and
      `format_session_start_produces_started_hhmm` expectations; add the new
      tests).
      Verify: `cargo test -p ff-desktop prof_`, `cargo test -p ff-desktop
      last_action`, `cargo test -p ff-desktop status` pass; no test still
      asserts `Started:` or `RUNNING` for the live status bar.
      Validates: Requirement 7.15, 7.18, 7.19, 7.20.

- [ ] 5. Add the Workspace_Status_Segment composition point in
      `render_status_bar`: after the common segments, append a Kind-owned
      segment string that is EMPTY by default (the per-Kind content is a
      follow-up gate). Implement it as a shell method
      `workspace_status_segment() -> String` returning `String::new()` for every
      Kind for now (single composition point; no Kind-registry trait method). If
      a Kind-registry trait method turns out to be required, STOP and surface it
      to the owner per design.md -- do NOT add it unilaterally. Write a failing
      test proving the status composition includes the slot (empty is acceptable)
      alongside the common segments.
      Files: `crates/ff-desktop/src/shell/render_status.rs`
      (`workspace_status_segment` + its render), `crates/ff-desktop/src/shell/tests_session.rs`
      (test).
      Verify: `cargo test -p ff-desktop workspace_status_segment` passes; `cargo
      check -p ff-desktop` clean.
      Validates: Requirement 7.17, 7.21, 7.27.

## B. Common-frame single-chrome regression guard (Phase CF.6)

- [ ] 6. Write a full-shell egui_kittest test (via `harness_shell()`) asserting
      the eight common-frame elements each render once: the command field
      (`command_field_input`) present, the Status_Bar and the new Message_Line
      are DISTINCT bottom panels, the key bar follows `pfshow`
      (`key_bar_visible`), and no active Workspace re-draws a frame element in
      its content (reuse the `full_shell_alias_editor_is_single_chrome` pattern
      -- assert the redundant surface title count stays 0). Confirm the existing
      full-shell first-Tab tests remain green (behaviour-preserving around the
      new Message_Line panel) -- do not modify them.
      Files: `crates/ff-desktop/src/shell/tests_focus.rs` or
      `tests_common_frame.rs` (new file if needed for the 200-line rule).
      Verify: `cargo test -p ff-desktop common_frame` passes; `cargo test -p
      ff-desktop full_shell_first_tab` still passes.
      Validates: Requirement 7.1, 7.5, 7.6, 7.22, 7.23, 7.24, 7.27.

## C. Snapshot/capture reproduction of the common frame (Phase 2H; creates the 2G chrome-helper foundation)

- [ ] 7. Box-drawing content grid in the ff-screen-model text renderer. Change
      `render/text.rs` to render each `model.tables` entry as a box-drawn grid:
      a header rule, a header/data separator rule, box-drawing vertical
      separators between fixed-width cells, and a bottom rule -- keeping the
      `[value v]` / `[ v]` / `____` / padded-value cell conventions and the
      full-frame width. Keep the ASCII fallback (`+ - |`) when
      `RenderOptions.unicode_box` is false (Req 5.2). Apply the equivalent cell
      conventions in `render/html.rs` (`<td>` with `[value v]` / `____`). Write
      failing unit tests FIRST in each renderer's test module.
      Files: `crates/ff-screen-model/src/render/text.rs`,
      `crates/ff-screen-model/src/render/html.rs`.
      Verify: `cargo test -p ff-screen-model box_draw` and `cargo test -p
      ff-screen-model render` pass; `cargo clippy -p ff-screen-model -- -D
      warnings` clean.
      Validates: Requirement 30.12, 30.13, 30.14.

- [ ] 8. Create the shared common-frame chrome helper in ff-desktop that composes
      the eight frame elements onto a `ScreenModel` from live logical state
      (never egui): `header_lines` = Window_Title_Bar (`FileForge Workbench` +
      `_ [] X` from the single shared title const), Menu_Bar (live menu echo,
      e.g. `Settings | Help | SCRM`), FULL Tab_Bar (ALL open `TabManager` tabs
      with their `[XXX]` `BuiltinKind::default_title()` tags + an ASCII `x` close
      marker -- not active-tag-only); the centered Workspace_Title stays
      `model.title`; the top `Command ===>` line stays `model.command_line`; the
      Message_Line lines (from the shell `message_line` state, omitted when
      empty) and the Status_Bar text (`Last:`/`Prof:`/Ln,Col/encoding/line count
      + Kind segment, NO `RUNNING`) and the pfshow key-bar line go into
      `footer_lines`/`status_bar`. One helper
      `apply_common_frame(&self, &mut ScreenModel)` called from the per-Context
      dispatch in `screen_provider.rs` so EVERY capturable Context gains the
      frame (Req 30.15) and SNAPSHOT + CAPTURE share it (Req 30.16). Write
      failing ff-desktop unit tests FIRST for: full tab bar (all tags), menu-bar
      line, message line present-vs-omitted, common+per-Kind status with no
      RUNNING.
      Files: new `crates/ff-desktop/src/shell/common_frame.rs` (the helper; keep
      under 400 lines), `crates/ff-desktop/src/shell/mod.rs` (register the
      module), `crates/ff-desktop/src/shell/screen_provider.rs` (call
      `apply_common_frame` in the dispatch, after building each Context model;
      reconcile the menu path so it is not double-framed),
      `crates/ff-desktop/src/screen_snapshot.rs` (single shared title const if
      not exported from main.rs), test module alongside `common_frame.rs`.
      Verify: `cargo test -p ff-desktop common_frame_helper` passes; `cargo
      check -p ff-desktop` clean.
      Validates: Requirement 30.1, 30.2, 30.3, 30.4, 30.5, 30.6, 30.7, 30.8,
      30.9, 30.10, 30.11, 30.15, 30.16, 30.17.

- [ ] 9. General-scope proof + capture parity (full-shell). Write full-shell
      egui_kittest tests: (a) a SNAPSHOT over the live ALIAS Editor (a table
      Context) contains the menu-bar line, BOTH open tab tags (e.g. `[ALIASES]`
      and `[CONFIG]` -- open a second tab in the test), the box-drawing grid, the
      `Last:` / `Prof:` status segments, and no `RUNNING`; (b) a SNAPSHOT over
      the live CONFIG panel (a `markdown_sections` Context) contains the menu-bar
      line, the `[CONFIG]` tag, the `Command ===>` line, the status segments, and
      no `RUNNING` -- proving the frame is GENERAL, not Alias-specific; (c) a
      capture-path parity test asserting the CAPTURE seam yields the IDENTICAL
      reproduced frame text as the SNAPSHOT seam for the same Context.
      Files: `crates/ff-desktop/src/shell/tests_common_frame_snapshot.rs` (new
      file; mirror `tests_config_snapshot_fidelity.rs` harness usage).
      Verify: `cargo test -p ff-desktop common_frame_snapshot` passes.
      Validates: Requirement 30.1, 30.3, 30.4, 30.10, 30.12, 30.15, 30.16.

## D. Steering amendments (docs, ASCII-only; cite the REAL code names created)

- [ ] 10. Amend the three steering files, citing the actual implemented names
      (`crates/ff-desktop/src/shell/common_frame.rs::apply_common_frame`,
      `message_line`, `last_action_at`, `note_action_dispatched`,
      `workspace_status_segment`).
      Files:
      `.kiro/steering/framework-conformance.md` -- ADD core mechanism
      `8. Common Workspace Frame (CR-NR-115)`: the shell owns the eight frame
      elements drawn once around every Workspace; a Workspace owns only its
      content area + one status-bar segment; DO NOT redraw a frame element in
      content; DO NOT draw chrome per-Context; cite the chrome helper +
      message_line/last_action_at + status-segment seam names. ADD to the "what
      counts as a framework change" list: reshaping the common frame or the
      common-vs-unique boundary (or adding/removing a frame element) needs owner
      confirmation.
      `.kiro/steering/wiring-standard.md` -- in `View Integration` /
      `When adding a NEW workspace`, ADD that a new workspace fills ONLY the
      content area and contributes ONE status-bar segment; it never draws the
      title bar / menu bar / tab bar / command line / message line / status bar /
      PF-key line itself -- the shell frame provides those.
      `.kiro/steering/workspace-conformance.md` -- ADD a short cross-reference
      tying the focus contract to the common-frame contract (both are "every
      workspace MUST ...").
      Verify: `Select-String -Path .kiro/steering/*.md -Pattern
      '[^\x00-\x7F\u2500-\u257F]'` returns no new violations (ASCII-only).
      Validates: documentation of Requirement 7.23, 7.27 in steering.

## STOP

After items 1-10 compile-verify clean (scoped check/clippy/fmt + scoped new
tests), STOP. Do NOT run the full suite, do NOT merge, do NOT push. Report the
compile-verify evidence and hand off to the owner for the full `cargo gate` +
manual visual check of the framed window and the Alias/Config snapshots.

---

## Digest

Files and the exact blocks to change, the test that covers each, and what was
verified during exploration.

### `crates/ff-desktop/src/shell/state.rs`
- ADD fields on `WorkbenchShell` near `session_start:
  chrono::DateTime<chrono::Local>` (line ~392): `message_line: Vec<String>` and
  `last_action_at: Option<chrono::DateTime<chrono::Local>>` (both additive, Req
  7.27). Covered by new tests in `tests_session.rs`.

### `crates/ff-desktop/src/shell/construct.rs`
- In `WorkbenchShell::new()` struct literal (ends ~line 326 with
  `session_start: chrono::Local::now()`), ADD `message_line: Vec::new()` and
  `last_action_at: None`. No test; compile-gated.

### `crates/ff-desktop/src/shell/actions.rs`
- Near `format_session_start` (line 140) and `active_profile_label` (line 149):
  ADD `note_action_dispatched(&mut self)` (clears `message_line`, stamps
  `last_action_at`) and `format_last_action(&self) -> String` (`Last: HH:MM`,
  `None` -> blank). SHORTEN `active_profile_label` from `format!("Profile:
  {name}")` / `"Profile: default"` to `Prof:`. Covered by
  `format_last_action_*`, `active_profile_label_*` tests.

### `crates/ff-desktop/src/shell/commands_line.rs`
- `begin_command_line` (line ~60, currently sets `pending_command_line_outcome =
  None; open_error = None`): ADD `self.note_action_dispatched();` at the top.
  Covered by the message-line-cleared test.

### `crates/ff-desktop/src/shell/target_dispatch.rs`
- `dispatch_bound_command` (line 89): ADD `self.note_action_dispatched();` at the
  top (covers menu-option clicks that bypass `begin_command_line`). Covered by
  the message-line-cleared test.

### `crates/ff-desktop/src/shell/render_status.rs`
- `render_status_bar` (line 106): REMOVE the `phase_label` match + its
  `ui.label(phase_label)` (the RUNNING segment); REPLACE
  `ui.label(self.format_session_start())` (line 133) with
  `ui.label(self.format_last_action())`; the `active_profile_label()` call (line
  ~139) now yields `Prof:`. ADD a `workspace_status_segment()` label after the
  common segments. ADD a new `render_message_line(ctx)` method (a
  `TopBottomPanel::bottom("message_line")` that early-returns when
  `self.message_line.is_empty()`, else one row per message). Covered by status
  composition tests + the message-line headless test + full-shell CF.6 test.

### `crates/ff-desktop/src/shell/update.rs`
- `update()` panel sequence (lines 318-320): currently
  `render_command_field` -> `render_key_label_bar` -> `render_status_bar`. ADD
  `self.render_message_line(ctx);` AFTER `render_status_bar` so the message panel
  stacks directly above the status bar (egui bottom panels stack upward in call
  order). Verified: key_bar is called before status_bar today, so status_bar
  already sits ABOVE the key bar -- the required order; the message line added
  last sits above the status bar. Compile/full-shell-gated.

### `crates/ff-desktop/src/shell/common_frame.rs` (NEW, < 400 lines)
- `apply_common_frame(&self, model: &mut ScreenModel)`: compose `header_lines`
  (window title bar from the single `FileForge Workbench` const + `_ [] X`; menu
  bar from the live menu echo; full tab bar from `self.tabs` tabs +
  `BuiltinKind::default_title()` + `x` close marker), `footer_lines` (pfshow key
  bar from `self.key_bar_visible`/`key_bar_scope`), `status_bar.text` (Last:/
  Prof:/Ln,Col/encoding/line count + `workspace_status_segment`, no RUNNING), and
  the Message_Line lines from `self.message_line` (omitted when empty). Covered
  by the new `common_frame.rs` test module + the full-shell snapshot tests.

### `crates/ff-desktop/src/shell/screen_provider.rs`
- `active_screen_capture` (line 88): after building each per-Context model, call
  `self.apply_common_frame(&mut model)` so every `ScreenCapture::Model(..)` is
  framed (Req 30.15/30.16). Reconcile the `KindTag::MenuWorkspace` arm (lines
  ~101-116) so the menu path is not double-framed (menu already composes its own
  body; add only the shared title/menu/tab/status/message chrome, not a second
  command line / PF footer). Covered by `menu_workspace_dispatches_to_provider`
  (must stay green) + new snapshot tests.

### `crates/ff-desktop/src/shell/mod.rs`
- ADD `mod common_frame;` near the other `mod screen_*;` declarations (lines
  ~96-99). Compile-gated.

### `crates/ff-screen-model/src/render/text.rs`
- `render_plain` table loop (currently `lines.push(table.headers.join("  "))`
  and `lines.push(row.join("  "))`, lines ~82-90): REPLACE with a box-drawn grid
  (header rule, separator rule, vertical cell separators using `Glyphs`, bottom
  rule), fixed-width cells preserving `[value v]`/`[ v]`/`____`/padded
  conventions and full-frame width; ASCII fallback via the existing `Glyphs`
  ASCII branch. Covered by new `render/text.rs` test-module tests.

### `crates/ff-screen-model/src/render/html.rs`
- `<td>` cells carry the same `[value v]` / `____` conventions. Covered by new
  html tests. (Read this file before editing -- not yet opened; it exists under
  `render/`.)

### Tests (new / updated)
- `crates/ff-desktop/src/shell/tests_session.rs` -- message_line store, clear on
  action, last_action_at, format_last_action. (Existing
  `format_session_start_produces_started_hhmm` / `session_start_*` stay; add new,
  do not break them unless the status bar no longer shows Started -- update only
  the STATUS-BAR expectation, keep `session_start` field intact for logoff.)
- `crates/ff-desktop/src/shell/tests_misc.rs` -- update
  `active_profile_label_reflects_active_profile` to expect `Prof:`.
- `crates/ff-desktop/src/shell/tests_focus.rs` (or new `tests_common_frame.rs`)
  -- message-line headless panel test + CF.6 single-chrome guard, modelled on
  `render_status_bar` headless test and `full_shell_alias_editor_is_single_chrome`.
- `crates/ff-desktop/src/shell/common_frame.rs` test module -- helper unit tests.
- `crates/ff-desktop/src/shell/tests_common_frame_snapshot.rs` (new) -- full-shell
  Alias + Config snapshot + capture-parity, modelled on
  `tests_config_snapshot_fidelity.rs` (uses `harness_shell()`).

### Verified during exploration
- Build/test commands (gate-policy.md): scoped `cargo check -p <crate>`,
  `cargo clippy -p <crate> -- -D warnings`, `cargo fmt -p <crate> -- --check`,
  `cargo test -p <crate> <name>`. Full `cargo gate` is the OWNER's manual run.
- Worktree clean on branch `common-frame-impl`.
- All three steering files and `.agents/tasks/common-frame-impl` exist.
- `ScreenModel` slots (`header_lines`/`body_lines`/`footer_lines`/`tables`/
  `status_bar`/`command_line`) already present -- no type reshape.
- Phase 2G shared chrome helper does NOT exist yet (confirmed: non-menu
  `*_screen_model` builders add no window/menu/tab/status chrome; text renderer
  tables use `join("  ")`); item 8 creates it scoped to the common frame.
- The one live title constant is `"FileForge Workbench"` in `main.rs`
  (`ViewportBuilder::with_title` + `run_native`).
- `harness_shell()` + `cmd_field_id()` (= `egui::Id::new("command_field_input")`)
  are the full-shell test entry points (tests_common.rs).
