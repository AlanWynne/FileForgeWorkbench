# Implementation Plan -- common-frame-fixes (CR-NR-115 follow-up, five defects)

Worktree: c:/workspace/apps/FileForgeWorkbench/.worktrees/common-frame-fixes
Crate: ff-desktop (src root `crates/ff-desktop/src`). Verification is SCOPED compile/lint only
(gate-policy + testing.md): `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`,
`cargo fmt -p ff-desktop -- --check`, each redirected to a UNIQUE `_ignore/logs/<name>-<ts>.log` and
read back. Single new tests may be run scoped (`cargo test -p ff-desktop <name>`) only if quick;
the owner runs the full gate. Do NOT run full `cargo test` / `cargo gate`.

## Findings that shape the plan (all read in the worktree)

- Key line (defect 1): `ff-screen-model/src/render/framed.rs::push_bottom_bands` ALREADY draws
  `footer_lines` as the LAST single-cell band after the status band. The menu path misses it because
  `screen_snapshot.rs::menu_workspace_screen_model_full` folds the PF line into `body_lines`
  (+ 2 blank rows) from `MenuScreenChrome.key_bar_lines`, and `common_frame.rs::apply_common_frame`
  skips the key line when `is_menu`. So the fix is in ff-desktop only; ff-screen-model is untouched.
- Command line (defect 2): the same builder folds `Command ===> {typed}{_ x119}` into `body_lines`
  from `&self.command_text` (still holding the triggering `ss` during dispatch). Non-menu Contexts
  use `apply_common_frame` -> `model.command_line = command_field_value(common_frame_command_field_text())`
  (empty field; the framed renderer owns the underscore fill to the border). Menu path is skipped by
  `if !is_menu`.
- Both defects have one root cause: `is_menu` branches in `apply_common_frame` plus the menu builder
  duplicating frame elements. Fix = delete the duplication; the menu builder returns CONTENT ONLY.
  This is automatically Menu-Kind generic (POM, Settings, `menus/<name>.toml`): there is no
  name-based branch anywhere in the menu capture path (it dispatches on `KindTag::MenuWorkspace`).
- Message routing (defect 3): `message_line: Vec<String>` is NEVER written by production code.
  `render_message_line` and the snapshot `messages` read it, but every message goes to
  `open_error` (shown in `render_status_bar`). `open_error = Some(..)` has ~80 sites in
  commands_ladder_a/b/c, commands_fastpath, commands_scrm, commands_snapshot, commands_session,
  commands_pfshow, commands.rs, actions.rs, dialog_capture.rs, external_adapter.rs, etc.
  `workspace-framework/design.md` (Message_Line section, ~line 373) already says `open_error` is
  ERROR-ONLY and was rejected as the info store; the code drifted. Also `finish_command_line`
  treats `open_error.is_some()` as failure and RESTORES the typed command, so every info message
  posted through `open_error` also wrongly keeps the command text in the field. Moving info
  messages out fixes that too (expect some existing tests to flip; update them, do not skip them).
- Capture timing (defect 4): typed path = `run_command_line` -> `begin_command_line` ->
  `note_action_dispatched()` (clears `message_line`, restamps `last_action_at`) -> dispatch ->
  `handle_snapshot` -> `active_screen_capture`. So the capture sees the CLEARED message line and the
  NEW Last time. Key-forward path = `dispatch_key_command` -> `begin_command_line` ->
  `dispatch_bound_command` -> `note_action_dispatched()` AGAIN (documented double-invoke). Menu
  click/shortcut = `dispatch_bound_command` only. Any naive "save pre-state inside
  note_action_dispatched" is overwritten by the second call: the stash MUST be idempotent per action.
- Zoom (defect 5): lives in `WorkbenchShell.zoom: ZoomState` (`shell/state.rs:247`, constructed in
  `construct.rs:278` from `ZoomConfig::default()`), read via `self.zoom.offset()` (`ZoomOffset::value()`).
  **NEGATIVE ZOOM IS SUPPORTED**: default `min_offset = -10`, `max_offset = +60`; config clamps
  `min_offset` to [-20, 0]; `parse_zoom_args("-2")` -> `SetAbsolute(-2)`; `ZoomIndicatorState`
  already formats `Zoom: +N` / `Zoom: -N` but returns `Hidden` at 0. The live bar currently shows
  zoom only when non-zero, as an accent label between Prof and the per-Kind cells
  (`render_status.rs` ~line 143); the snapshot band (`common_frame_status_bar_text`) has no zoom.
  Do NOT change `ff-zoom` (its tests assert Hidden at 0); add a shell-level formatter instead.
- Existing tests that will need updating (they assert the old status text/zoom):
  `tests_common_frame_golden.rs::editor_context_status_text_keeps_ln_col_encoding_and_line_count`
  (`"Last: 11:02 | Prof: default | Ln 1, ..."` -> add `Zoom: 0`), the status assertions in
  `config_and_alias_status_band_is_last_and_prof_only`, the four Config/Alias goldens in
  `src/shell/golden/*.txt`, `screen_snapshot.rs` unit tests that use `sample_chrome(with_keys)` /
  `key_bar_lines` / `Command ===>` body row, and `tests_common_frame*.rs`.

## DECISIONS (record in the docs step)

D1. Error routing: ERRORS STAY on `open_error` (red, Status_Bar, still registered to
    `automation::ids::STATUSBAR_MESSAGE`, still drives `finish_command_line` Restore). Only
    INFORMATIONAL / CONFIRMATION messages move to the Message_Line. Rationale: matches the existing
    design.md text ("open_error is error-only"), keeps command-line failure semantics (Req 13.2/13.3)
    and automation unchanged, and nothing is silently dropped. Record as new workspace-framework
    Req 7.28 (info -> Message_Line; error -> Status_Bar error slot, cleared on next action) .
    Snapshots do not include `open_error` today; unchanged (note as a possible follow-up, not in scope).
D2. Classification rule for a poster: it is INFO if the command SUCCEEDED and the text is a
    result/confirmation/status report (snapshot copied/written, copy-copied, LOGOFF time, date/time
    report, status-routing report, replace summary, `Info` status kind); it is an ERROR if the text
    reports failure, refusal, bad argument, "not available/stub", "requires path", unknown
    subcommand, "no clipboard", "cannot be captured".
D3. Zoom is a COMMON status segment, always shown, directly after Prof; one formatter feeds the live bar
    and the snapshot band.
D4. Pre-action capture state is ADDITIVE shell state (not a framework change; see the flag below).

## PROMINENT FLAG (defect 4): seam change beyond "read pre-action state"

Reading pre-action state requires saving it BEFORE `note_action_dispatched()` clears/restamps. That is
an additive hook INSIDE the existing per-action seam (`note_action_dispatched`, called from
`begin_command_line` and `dispatch_bound_command`) plus one new shell field. It adds NO dispatch path,
NO navigation stack, NO public type reshape, and `active_screen_capture` stays the single capture seam,
so it is NOT a framework change under framework-conformance.md. But it DOES change the behaviour of
the shared per-action seam (idempotent stash + explicit end-of-action clear in three places) and the
`Last:` semantic for captures (a capture shows the Last time as it was when the command was issued).
Surface this to the owner in the final report and in the change-log note. If the owner does not want
the per-action seam touched, the only alternative is moving the clear to the end of the action, which
is a larger behaviour change; this plan does not do that.

---

# TASKS (ordered by dependency; tests first per TDD)

- [ ] 1. Docs gate: update requirements, design, tracking (docs only, ASCII only, no source edits)
      - workspace-framework/requirements.md: Req 7 criterion 16 and 17 in place: Common_Status_Segments =
        `Last: HH:MM`, `Prof: <name>`, `Zoom: <n>` (always shown, directly after Prof, formats
        `Zoom: 0`, `Zoom: +14`, `Zoom: -3`). Update glossary row Common_Status_Segment (line ~244) and the
        criterion-23 reference (common = Last + Prof + Zoom). Add the "context help is a HOVER TOOLTIP only
        (status segments are non-focusable, B055); zoom is never a Tab stop" sentence. Add new criterion
        7.28 (info vs error routing, D1/D2) and 7.29 (a snapshot/capture reproduces the frame as it was
        when the triggering command was issued: pre-action Message_Line and Last, empty command field).
        Check the highest criterion number in the file first (27 at plan time) and number after it.
      - screen-snapshot-scrm/requirements.md: Req 30.10 in place (Last + Prof + Zoom + Kind cells),
        reword 30.6/30.21 note that this holds for EVERY Context including Menu Kinds, add 30.27 (menu
        path uses the same frame: key line last as ONE cell, command line empty with underscores to the
        border, message band above status, for every Menu Kind) and 30.28 (pre-action capture timing).
        Check the next free 30.x number first (30.26 at plan time).
      - view-zoom/requirements.md Req 7 criteria 1-3 and the Zoom_Indicator glossary/summary line 14:
        supersede "hidden at zero / after encoding before Ln:Col" with a pointer to workspace-framework
        7.16 (common segment, always shown, after Prof). Edit in place and note the change.
      - workspace-framework/design.md (status segments table ~line 301, Message_Line section ~346-376,
        common segments ~334) and screen-snapshot-scrm/design.md (status text ~839, helper steps ~855,
        sample frame ~893, Menu note): add the Zoom segment, the routing rule, `PreActionFrame`
        (task 6), and "the menu builder returns content only".
      - docs/status/change-log.md: add a note UNDER the existing CR-NR-115 entry (do not add a new CR):
        the five defects, decisions D1-D4, the flagged seam change, and "no POM-specific branch".
      - Coverage record + tasks: add NOT-COVERED rows for each new criterion and tasks in the
        affected tasks.md files (`[ ]`/`[x]` only, titled). Find the coverage record by grepping
        `docs/` for the existing 30.10 row; do not invent a new file.
      Verify: `Select-String` the ASCII check from documentation-charset.md over the edited md files
      returns no match (run via the pwsh7 wrapper, read the output); no cargo needed.

- [ ] 2. Write the failing tests first (all `// Validates: Requirement X.Y`, full-text goldens)
      Files: `crates/ff-desktop/src/shell/tests_common_frame_golden.rs` (reuse `fixed_shell`/`fixture`;
      split new tests into a new `tests_common_frame_menu.rs` registered in `shell/mod.rs` if the golden
      file would exceed ~200 test lines), new fixtures under `src/shell/golden/`.
      a. `menu_kind_pom_snapshot_matches_golden_pfshow_on_with_message`: fixture
         `menu_pom_message_pfshow_on.txt`. Build a deterministic POM: fixed `last_action_at`, default
         profile, two tabs, pfshow ON, `message_line = ["Snapshot copied to clipboard: picture + Markdown."]`.
         The calendar block is date-dependent: the golden must not contain live dates. Either inject a
         fixed calendar via a test seam on `menu_screen_chrome` (preferred: split the right-column
         builder so the test passes a fixed `year/month/day/hour/minute` to `calendar_right_column`,
         which already takes components), or build the POM from a calendar-less test menu. Assert:
         key line is the LAST band after the status band as ONE cell; status band is
         `Last: 11:02 | Prof: default | Zoom: 0`; command row is `Command ===>` + underscores to the
         right border (no `ss`); message band sits between content and status with its own rules; no
         blank spacer rows before the key line.
      b. `menu_kind_settings_snapshot_matches_golden_pfshow_on`: fixture `menu_settings_pfshow_on.txt`
         (calendar-less built-in `settings` menu, no message): proves identical frame behaviour.
      c. `menu_kind_user_menu_snapshot_matches_golden`: a user menu loaded from a temp
         `menus/<name>.toml` (use the temp-dir menu-loading pattern from `tests_menu_workspace.rs`;
         fixture `menu_user_pfshow_off.txt`, pfshow OFF to also cover "no key line"). If loading a user
         menu in the harness is not cheap, document why in a comment and keep (a)+(b); do not weaken them.
      d. `menu_kind_frame_has_no_menu_name_branch`: capture POM and Settings and assert the non-content
         rows (title bar, menu bar, tab bar, command row, status band, key line) are identical
         except the title and body; structural proof that no name branch exists.
      e. Message-routing tests (new `tests_message_routing.rs`): `info_messages_go_to_message_line_not_open_error`
         drives the REAL seams (`dispatch_command_string` for LOGOFF-free info verbs such as the
         date/time/status report verbs, `place_snapshot_composite` result, the dialog Copy path) and asserts
         `message_line` has the text and `open_error` is None; `error_messages_stay_on_open_error`
         drives bad-argument cases (`SNAPSHOT bogus`, `PFSHOW bogus`) and asserts `open_error` is Some
         and `message_line` is empty; `info_message_does_not_restore_command_field` asserts the
         `finish_command_line` Clear outcome for a successful info command.
      f. Capture-timing tests (new `tests_capture_timing.rs`) driving the real seam, not a helper:
         `snapshot_captures_message_present_before_the_command`: set `message_line = ["prev"]`, set
         `command_text = "SNAPSHOT"`, call `run_command_line("SNAPSHOT")`/`dispatch_key_command`, read
         `shell.snapshot.pending.text` (the Markdown flavour stashed by `stash_composite_request`) and
         assert it contains `prev` in the message band and NOT the typed command in the command row;
         `second_snapshot_contains_first_snapshots_confirmation`: run SNAPSHOT, post the confirmation as
         the frame loop does (call the new `post_info` with the same i18n text), run SNAPSHOT again, assert
         the second pending text contains `Snapshot copied to clipboard: picture + Markdown.` and the
         live `message_line` was cleared by the action; `first_snapshot_does_not_contain_its_own_confirmation`;
         `double_invoke_key_forward_path_keeps_pre_action_state` (F-key path calls `note_action_dispatched`
         twice; the stash must still hold the original message); `capture_outside_an_action_uses_live_state`.
      g. Zoom tests (new `tests_zoom_segment.rs`): `zoom_label_zero_is_zoom_colon_0`,
         `zoom_label_positive_has_plus_sign` (+14), `zoom_label_negative` (-3; legal because default
         `min_offset` is -10; set via `shell.zoom` using the same `ZoomState` API the ZOOM command uses),
         `status_band_order_is_last_prof_zoom_then_kind_cells` (editor Context:
         `Last: 11:02 | Prof: default | Zoom: 0 | Ln 1, Col 1 | UTF-8 | 1 lines`),
         `live_status_bar_and_snapshot_band_use_one_formatter` (call the same `format_zoom_label()`
         from both and assert the snapshot band contains exactly the label the live bar would draw).
         Tooltip test: the zoom label in the live bar is not a Tab stop (reuse the B055 pattern in
         `tests_focus.rs`) and has hover help; if hover text cannot be read headlessly from
         `egui_kittest`, mark the criterion MANUAL with that reason in the coverage record (allowed
         exception: hover tooltip text), keep the not-a-tab-stop assertion automated.
      h. Update existing assertions listed in "Findings" to the new expected values.
      Verify: `cargo check -p ff-desktop --tests` compiles; the new tests are expected RED. Run one or
      two of them scoped (`cargo test -p ff-desktop menu_kind_pom_snapshot`) only if quick; else leave
      to the owner. Fixtures may be created after task 7 by running the tests and hand-reviewing output
      (goldens must be reviewed text, not blindly accepted).

- [ ] 3. Defects 1 and 2: make the menu builder content-only and remove the `is_menu` branches
      Files: `screen_snapshot.rs` (menu_workspace_screen_model_full, MenuScreenChrome, unit tests),
      `shell/screen_provider.rs` (active_context_capture menu arm, menu_screen_chrome),
      `shell/common_frame.rs` (apply_common_frame).
      - `menu_workspace_screen_model_full`: remove the `Command ===>` body row and the two blank
        spacers + PF footer loop. Keep title, option rows, two-column body, load-error message.
        Keep the signature but drop `command_line`/`key_bar_lines` usage: preferred change is to REMOVE the
        `command_line` parameter and `MenuScreenChrome.key_bar_lines` (both are ff-desktop-internal,
        not ScreenModel/Table/WorkspaceContext, so this is permitted) and update the ~10 call sites
        in `screen_snapshot.rs` tests. `menu_screen_chrome` then only builds `right_column`.
      - `apply_common_frame`: delete `let is_menu` and both `!is_menu` guards, so command line
        (`command_field_value(self.common_frame_command_field_text())`, empty field, renderer fills
        underscores to the border) and the pfshow key line (`footer_lines`) are set for EVERY Context.
        Update the module/doc comments that describe the menu reconciliation (lines 1-40, 60-70).
      - Delete the now-unneeded `KindTag` import in common_frame.rs if unused.
      - Do NOT branch on menu name anywhere; confirm with `grep_search` for `"pom"|"POM"|"settings"`
        in the changed regions.
      - Generic across Menu Kinds: nothing else changes; Settings/user menus pass through the same arm.
      Verify: `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`,
      `cargo fmt -p ff-desktop -- --check`; tests 2a-2d go green when run scoped.

- [ ] 4. Defect 5 (part 1): zoom formatter and status band
      Files: `shell/actions.rs` (add `format_zoom_label(&self) -> String` beside `format_last_action` /
      `active_profile_label`), `shell/common_frame.rs` (`common_frame_status_bar_text`),
      `shell/render_status.rs` (`render_status_bar`), `crates/ff-i18n/i18n/*/ff-desktop-shell-status-*.ftl`.
      - `format_zoom_label`: `match ZoomIndicatorState::from_offset(self.zoom.offset()) { Visible{text,..} => text, Hidden => "Zoom: 0".into() }`
        (reuses the ff-zoom formatter for the sign; `ff-zoom` untouched).
      - `common_frame_status_bar_text`: `vec![last, prof, zoom]` then `workspace_status_cells()`.
      - `render_status_bar`: replace the old "zoom when non-zero" block with an always-shown
        `ui.label(self.format_zoom_label()).on_hover_text(ff_i18n::t("ff-desktop-shell-status-zoom-help"))`
        directly after Prof + separator. Plain `ui.label` only (never SelectableLabel / click sense, B055).
        Update the comments in render_status.rs and `workspace_status_cells` docs ("common = Last, Prof, Zoom").
      - i18n: add `ff-desktop-shell-status-zoom-help` to the `en` ftl with the help text (what zoom
        offset means, ZOOM IN/OUT/RESET, Ctrl+= / Ctrl+-, Ctrl+0); add the key to fr/de/af too if the
        repo requires key parity across locales (check `ff-i18n` tests for a parity check before
        deciding; ASCII-escape non-ASCII as the existing ftl files do with `{ "\u00e9" }`).
      Verify: check/clippy/fmt on ff-desktop (and ff-i18n if its ftl tests are touched);
      tests 2g go green.

- [ ] 5. Defect 3: message routing
      Files: new `shell/message_routing.rs` (small, <400 lines; declare in `shell/mod.rs`), then the
      poster files listed under Findings.
      - Add `pub(crate) fn post_info(&mut self, text: impl Into<String>)` that pushes to `self.message_line`
        (no dedup; multiple messages expand the band per Req 7.13). Errors keep `self.open_error = Some(..)`.
        Do not add a second store, a channel, or touch `NotificationQueue`.
      - Inventory EVERY `open_error = Some(` site with `grep_search` over `crates/ff-desktop/src/**/*.rs`
        (excluding tests*) and classify with rule D2. Known INFO posters to move: `place_snapshot_composite`
        (`snapshot-both-copied`, `snapshot-copied`; the `snapshot-no-clipboard` one is an ERROR, leave),
        `snapshot_single_format_to_file` / `snapshot_to_pdf` `Ok` arms (`snapshot-written`, `snapshot-pdf-written`;
        their `Err` arms stay errors), `dialog_capture.rs` `msg-copy-copied` (the `copy-no-clipboard` stays),
        the `LOGOFF` message in `commands_ladder_a.rs`, the date/time and status-routing reports in
        `commands_ladder_c.rs` (~lines 150-176), `actions.rs` replace summary (`msg-replace-summary`;
        `replace-failed` stays), and `commands.rs` `StatusKind::Info` (currently DROPS the text with
        `open_error = None`: route `status.text` to `post_info` if non-empty, otherwise leave as is, so
        nothing is silently lost). Re-check `commands_scrm.rs` lines ~82 and ~106 (messages after
        capture save/export: success -> info, failure -> error) and `commands_session.rs`.
      - Where a poster is on the async frame-loop path (`place_snapshot_composite` runs from
        `update.rs` after the screenshot event) the message lands in `message_line` AFTER the action's
        clear, so it survives until the next action, which is the required lifetime (Req 7.12).
      - The per-detached-window swap in `commands_session.rs` (~134-167) saves/restores `open_error`;
        `message_line` is shell-wide so it needs no swap. Verify that is still correct.
      - Keep `finish_command_line` unchanged.
      - Update the old tests that asserted `open_error` for info texts to assert `message_line`
        (do not delete them).
      Verify: check/clippy/fmt; tests 2e go green. Also `grep_search` shows no remaining
      `open_error = Some(` with a success/confirmation i18n key (list the classified sites in the report).

- [ ] 6. Defect 4: capture the pre-action state through the single seam
      Files: `shell/state.rs` (+ `shell/construct.rs` init): new field
      `pre_action_frame: Option<PreActionFrame>` where `PreActionFrame { message_line: Vec<String>, last_action_at: Option<DateTime<Local>> }`
      (define in `shell/message_routing.rs` or a grouped sub-struct in `state_groups.rs`; do NOT add
      more than this one flat field, per wiring-standard State Integration);
      `shell/actions.rs` (`note_action_dispatched`); `shell/commands_line.rs` (`finish_command_line`/`run_command_line`);
      `shell/target_dispatch.rs` (`dispatch_bound_command`, `dispatch_key_command`); `shell/update.rs`;
      `shell/common_frame.rs` + `shell/screen_provider.rs`.
      - `note_action_dispatched`: FIRST `self.pre_action_frame.get_or_insert_with(|| PreActionFrame{ message_line: self.message_line.clone(), last_action_at: self.last_action_at })`
        (idempotent, so the documented double-invoke on the key-forward path cannot overwrite it), THEN
        clear/restamp as today.
      - End-of-action clear (`self.pre_action_frame = None`): at the end of `run_command_line`
        (after `finish_command_line`), at the end of `dispatch_key_command`, at the end of
        `dispatch_bound_command` ONLY when it is the outermost call (record `let outermost = self.pre_action_frame.is_none();`
        before calling `note_action_dispatched`, clear only if `outermost`), and as a safety net at the top
        of each frame in `update.rs` before input handling. So auto-capture or any capture outside an
        action sees live state.
      - `apply_common_frame`: read messages from `self.frame_message_lines()` and the Last label from
        `self.frame_last_action_label()`, helpers that prefer `pre_action_frame` when `Some`, else the live
        fields. Command field stays `common_frame_command_field_text()` (empty = field as it was before the
        triggering command). Zoom and Prof are not changed by the capture action, so they read live.
      - `active_screen_capture` remains THE single seam (CAPTURE SCREEN, SNAPSHOT, dialog paths all pass
        through it); do not add a second capture function.
      - Live rendering (`render_message_line`, `render_status_bar`) keeps reading the live fields.
      - Explicitly confirm in the report that PNG and CR-CH-079 clipboard dual representation are unchanged
        (only the text model feeding `stash_composite_request` changes; the PNG is the live window grab).
      Verify: check/clippy/fmt; tests 2f green (scoped run if quick).

- [ ] 7. Regenerate goldens and finish
      Files: `src/shell/golden/config_pfshow_on.txt`, `config_message_pfshow_off.txt`, `alias_pfshow_on.txt`,
      `alias_message_pfshow_off.txt`, `alias_message_pfshow_off_ascii.txt`, plus the new menu fixtures.
      Config/Alias goldens only change in the status band (`Last: 11:02 | Prof: default | Zoom: 0`; band
      widths/dividers shift, so regenerate rather than hand-edit): produce the new text from the
      harness output, REVIEW it line by line against Req 30.18-30.26 (key line still last, one cell, no
      junctions; no blank row before the status band), then write with the file-writing tool (ASCII
      goldens; the Unicode box glyphs in the existing goldens are test fixtures, unchanged style). Do
      not blindly accept output.
      Then: confirm every touched `.rs` file is <= 400 non-test lines (the `screen_provider.rs`,
      `common_frame.rs`, `actions.rs`, `commands_snapshot.rs`, `render_status.rs` files are the candidates;
      split per rust-standards if needed as a pure refactor), update the coverage record rows from
      NOT-COVERED to the test names, tick task boxes, and ASCII-check the docs.
      Verify: `cargo check -p ff-desktop --tests`, `cargo clippy -p ff-desktop --tests -- -D warnings`,
      `cargo fmt -p ff-desktop -- --check`, logs read back. Hand off the full `cargo gate --build`
      to the owner (gate-policy: required green before merging to main).

## Constraints checklist (the coder must restate in its report)
- No framework change; no ScreenModel / Table / WorkspaceContext reshape; `shell/environment.rs` untouched;
  `ff-screen-model` and `ff-zoom` untouched; CR-CH-079 clipboard dual representation and the PNG unchanged.
- Menu-Kind generic: confirm NO POM- or name-specific branch in the changed code (grep evidence).
- Menu CONTENT items (description-bracket removal, colon alignment, Option/Command/Description table,
  calendar position) are NOT done here; they stay for the Menu per-Kind gate.
- Extension-terminology: do not introduce any new "plugin"-named item (note `KindTag::PluginManager`
  exists; do not touch it).
- ASCII only in docs, steering and source; `// === Section ===` separators; every test has `// Validates:`.

## Digest

- `shell/common_frame.rs` (~190 lines): `apply_common_frame(&self, &mut ScreenModel)` (line 73) has
  `let is_menu = ... KindTag::MenuWorkspace` (lines ~100) and two `!is_menu` guards (command line, key bar);
  `common_frame_command_field_text` (line 120, returns `String::new()`);
  `common_frame_status_bar_text` (last fn, `vec![format_last_action, active_profile_label]` + `workspace_status_cells`).
  Covered by `tests_common_frame_golden.rs`, `tests_common_frame.rs`, `tests_common_frame_helper.rs`.
- `screen_snapshot.rs`: `menu_workspace_screen_model_full(state, command_line, chrome)` line 150 builds the
  `Command ===>` body row (COMMAND_INPUT_WIDTH=119) and the spacer + PF footer; `MenuScreenChrome { right_column, key_bar_lines }`
  line 49; `command_field_value` (non-menu) ~line 70; `key_bar_line` ~line 267; unit tests from ~line 320.
- `shell/screen_provider.rs`: `active_screen_capture` (line 88) = `active_context_capture` + `apply_common_frame`
  for `ScreenCapture::Model`; menu arm passes `&self.command_text` and `menu_screen_chrome(mw)` (line 197,
  builds right column + `key_bar_lines`). Tests at the bottom (3 tests, seeded POM helper `seeded_pom_shell`).
- `shell/actions.rs`: `note_action_dispatched` line 146 (clear + stamp), `format_last_action` 158,
  `active_profile_label` 171.
- `shell/render_status.rs`: `render_status_bar` line 105 (Last, Prof, old zoom-if-nonzero block ~143-150,
  `workspace_status_cells`, `open_error` red label + `STATUSBAR_MESSAGE` automation), `workspace_status_cells`
  ~line 262, `render_message_line` line 291 (early return when empty).
- `shell/update.rs` ~line 320-325: `render_key_label_bar` -> `render_status_bar` -> `render_message_line` order (unchanged).
- `shell/commands_line.rs`: `run_command_line` 28, `begin_command_line` 60 (calls `note_action_dispatched`),
  `finish_command_line` ~78 (Restore if `open_error.is_some()`).
- `shell/target_dispatch.rs`: `dispatch_bound_command` line 90 (calls `note_action_dispatched`),
  `dispatch_key_command` line 122 (`begin_command_line` then `dispatch_bound_command` = double invoke).
- `shell/commands_snapshot.rs`: `snapshot_composite_to_clipboard` (~line 126, `stash_composite_request`, sets
  `snapshot.pending`), `place_snapshot_composite` (~line 190-236, posts the confirmation via `open_error`),
  `snapshot_single_format_to_file` (~275-310) and `snapshot_to_pdf` (~340) post `snapshot-written` /
  `snapshot-pdf-written` via `open_error`.
- `shell/state.rs`: `message_line` (line ~400), `last_action_at` (~406), `zoom: ZoomState` (247);
  `shell/construct.rs`: inits at ~326-330 and 278.
- `crates/ff-screen-model/src/render/framed.rs`: `push_bottom_bands` (117) already orders rule, messages,
  status band, key line last as a single cell; `collect_rows` (72) prints `command_line` via `Row::Command`
  (renderer owns underscore fill). No change needed.
- `crates/ff-zoom`: `ZoomIndicatorState::from_offset` (Hidden at 0), `ZoomConfig` min -10 / max +60
  (clamp min to [-20, 0]); negative zoom IS supported and tested (`parse_zoom_args("-2")`).
- Docs: `docs/specs/workspace-framework/requirements.md` Req 7 criteria 16-21 (lines ~300-335),
  `docs/specs/screen-snapshot-scrm/requirements.md` Req 30.10 (line ~1420) and 30.26 (last), `view-zoom/requirements.md`
  Req 7, both `design.md` files, `docs/status/change-log.md` CR-NR-115 entry (line 25).
- Verified: read-only exploration only (no build or test was run in this planning step; terminal output is
  proxied and the files above were read with the read/grep tools). Unverified: current pass/fail state of
  the ff-desktop tests, exact line counts of the files near the 400-line limit, whether ff-i18n enforces
  cross-locale key parity, and whether a user `menus/<name>.toml` can be loaded cheaply in the kittest harness.
