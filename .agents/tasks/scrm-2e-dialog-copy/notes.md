# Stream SCRM Phase 2E -- on-dialog direct COPY affordance (screen-snapshot-scrm Req 27.3)

## Gap fixed
Phase 2E shipped COPY only as a front-door command verb (`COPY` -> handle_copy).
Req 27.3 requires EVERY message/dialog box to offer a DIRECT copy affordance
(button or Ctrl+C) so copying the dialog's text needs no typed command. The SCRM
Save-As dialog showed only Save/Cancel -- no Copy. This change adds the
affordance to every modal overlay, routed to the single existing redacted copy
path.

## What changed
- `shell/dialog_capture.rs`: renamed the single copy path `handle_copy` ->
  `copy_frontmost_surface` (the one clipboard path / one redaction point).
  Added `any_dialog_open()` (mirrors the frontmost_dialog_model precedence set).
- `shell/commands_ladder_b.rs`: the `COPY` verb now calls
  `copy_frontmost_surface`.
- On-dialog Copy BUTTON added to every overlay bound by Req 27:
  - SCRM Save-As (`scrm_save_as_dialog.rs`, new `Outcome::Copy`)
  - SCRM replay picker (`scrm_session_picker.rs`, new `Outcome::Copy`)
  - About (`about_dialog.rs`, render now returns `copy_clicked: bool`)
  - History, SWAP, External-exec (`shell/update_dialogs.rs`)
  - RESET BARE, Unsaved-workspace (`shell/update_dialogs_confirm.rs`, NEW file --
    split out of update_dialogs.rs to keep it under the 400-line rule)
  Each button routes to `copy_frontmost_surface` and LEAVES the dialog open.
- Ctrl+C while a dialog is open: `shell/update_input.rs`
  `handle_dialog_copy_shortcut` (called from `shell/update.rs` before overlays).
  Scoped to `any_dialog_open()` AND `!ctx.wants_keyboard_input()` so it never
  steals Ctrl+C from a focused text field or from the UI when no dialog is up.
  The Ctrl+C / Copy events are consumed.
- SCRM overlay outcome handling factored into `apply_scrm_save_as_outcome` /
  `apply_scrm_replay_outcome` so the Copy-leaves-open wiring is unit-testable.
- i18n: new key `ff-desktop-shell-dialogs-copy` in en/de/fr/af
  (Copy / Kopieren / Copier / Kopieer). ASCII-only.

## Constraints honoured
- One clipboard path, one redaction point (copy_frontmost_surface); no second
  clipboard path, no duplicated redaction, no egui pixel grab (text model only).
- No source .rs over 400 non-test lines: update_dialogs.rs 380 -> 297 after the
  split; update_dialogs_confirm.rs new (~210); others well under.
- ASCII-only; focus contract untouched (first_tab tests green).

## Verification (scoped, agent compile-verify; owner runs full gate)
- cargo check -p ff-desktop: OK
- cargo clippy -p ff-desktop --tests -- -D warnings: clean
- cargo fmt -p ff-desktop -- --check: clean
- cargo test -p ff-desktop dialog: 34 passed
- cargo test -p ff-desktop clipboard_backend: 8 passed
- cargo test -p ff-desktop scrm: 65 passed
- cargo test -p ff-desktop first_tab: 15 passed (no focus regression)

## Owner re-test
Open the SCRM Save-As dialog (CAPTURE STOP with a non-empty collection), click
Copy, paste into Notepad -- the dialog's text pastes without typing a command.
Also try Ctrl+C while the dialog is open.
