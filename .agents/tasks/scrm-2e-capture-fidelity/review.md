# Capture-fidelity fixes on scrm-2e: dialog Copy composite, faithful models, side-effect-free SNAPSHOT

Three capture-fidelity defects on `scrm-2e-dialog-capture` are fixed across five commits. FIX 1 makes the on-dialog Copy affordance (visible Copy button on every Req-27 overlay and Ctrl+C-while-open) and the `COPY` verb place the SAME multi-flavor composite (picture + Markdown text + CF_HTML) as bare `SNAPSHOT`, reusing the one composite path rather than the earlier text-only copy. FIX 2 replaces the thin Config ScreenModel (Filter + Unsaved-edits only) with a faithful settings-tree capture built from the same `visible_rows` builder the render/keyboard driver use, and locks the SCRM Save-As model's full field values; the other dialog models build from their own state structs. FIX 3 roots out the double-consume where the command-line Enter also drove the Config tree's keyboard driver, collapsing the focused node during SNAPSHOT.

Watch for: nothing blocking. The composite Copy path, single redaction point, single dispatch seam, and side-effect-free SNAPSHOT are all confirmed in the diff, with deterministic TDD tests that cite the correct requirements and never touch a real clipboard or print a secret. One pre-existing (out-of-scope) non-ASCII condition in `about_dialog.rs` comments is noted below but was not introduced by this change.

**Verdict**: APPROVED

## High-level view

FIX 1 is implemented by extracting `stash_composite_request(model)` from the bare-SNAPSHOT handler in `commands_snapshot.rs` and calling it from `copy_frontmost_surface` in `dialog_capture.rs`. Both SNAPSHOT and dialog Copy now render the two text flavors from the (already-redacted) model and arm the async window screenshot; the frame loop assembles and places the composite on the `Event::Screenshot` frame. There is no second clipboard path: the text-only `place_text` trait method and its two backend impls were removed. Every surface Copy affordance (the per-dialog buttons via `Outcome::Copy`/return-bool, the `COPY` verb via the ladder, and Ctrl+C via `handle_dialog_copy_shortcut`) routes to the one `copy_frontmost_surface`, which applies redaction once through `active_dialog_capture().redacted()`. The dialog stays open because arming a screenshot touches no overlay flag.

FIX 2 rewrites `config_panel_screen_model` (moved to the new `screen_providers_config.rs` for the 400-line rule) to walk the real schema entries through the panel's own `visible_rows(entries, filter, collapsed)` builder, emitting a group row per namespace with its expanded/collapsed state and a key row per visible key carrying its current effective value (staged working value when edited, else persisted), marked `(unsaved)` when staged. Because it reads every schema entry, categories such as Accessibility appear when present, respecting the live filter and collapse state. The SCRM Save-As `screen_model` carries the full Name/Folder/Type values (locked by a test asserting a full typed filename round-trips), and the external-exec, RESET BARE, History, SWAP, Unsaved, About, and replay-picker models each build from real state (command string, profile list, tab list, session list). All are logical-model-first (read STATE structs, never egui widgets) and run through the single redaction point.

FIX 3 changes both command-field submit paths (`render_command_field` and the shared `render_command_field_body`) from `ctx.input(key_pressed(Enter))` to `ctx.input_mut(consume_key(NONE, Enter))`, so the Config tree's per-frame keyboard driver no longer reads the same Enter and collapses the focused node. A regression test drives the owner's exact repro (`SNAPSHOT` + Enter with the command line focused in the Config panel) and asserts no namespace group collapses.

<details>
<summary>Issues (1)</summary>

1. **Pre-existing non-ASCII in about_dialog.rs comments (out of scope)** — `about_dialog.rs` carries en-dashes and box-drawing in `///` and `//` comment separators on unchanged lines (5, 59, 70, 75). The change did not introduce them (added lines use ASCII), so this does not block; worth a future cleanup when that file is next touched under the extension/ASCII sweep.

</details>

<details>
<summary>Details</summary>

## FIX 1 -- dialog Copy places the composite, reusing the one path

The core move is the extraction of `stash_composite_request(&model)` in `commands_snapshot.rs`. Bare `SNAPSHOT` now resolves its Context model and calls it; `copy_frontmost_surface` resolves the frontmost surface (dialog if open via `active_dialog_capture`, else the active Context via `active_screen_capture().into_model()`) and calls the same helper. The helper renders the Markdown text flavor and the plain box-drawn `pre_text` (the CF_HTML `<pre>` body) from the supplied model, stashes them in `snapshot.pending`, and sets `snapshot.screenshot_requested`. The window screenshot the frame loop takes naturally includes the open dialog on top, which is exactly what the owner wanted in Word. This is one assembly path in `shell/update.rs`; the caller supplies an already-redacted model so redaction stays a single point.

The text-only path is gone, not merely bypassed. `clipboard_backend.rs` dropped the `place_text` trait method and both the OS and arboard implementations, and the `FakeClipboard` test seam was trimmed to the composite path. `place()` now calls `raw::empty()` on open so a composite never leaves stale flavors from a prior publish, with `raw::set_without_clear` adding each format to the one open transaction.

Every Copy affordance converges on `copy_frontmost_surface`:

```
COPY verb (commands_ladder_b.rs) ----\
per-dialog Copy button --------------+--> copy_frontmost_surface --> stash_composite_request
  Save-As / replay: Outcome::Copy    |        (active_dialog_capture().redacted())
  About: render() -> bool            |
  History/SWAP/External/Reset/Unsaved|
Ctrl+C (handle_dialog_copy_shortcut)-/
```

The Ctrl+C shortcut in `update_input.rs` is scoped by `any_dialog_open()` AND `!wants_keyboard_input()`, so it fires only when a Req-27 overlay is up and no text field (e.g. the Save-As name field) owns the copy; the event is consumed. Each dialog's Copy outcome in `update_dialogs*.rs` calls the helper and does not clear its overlay flag, so the dialog stays open. Tests assert `screenshot_requested` and a stashed `pending` (the composite contract) rather than a text-only place, and assert the overlay flag survives.

## FIX 2 -- faithful field-value capture for dialog and Config models

The Config provider was the main defect (the owner saw only Filter + Unsaved-edits). `config_panel_screen_model` now builds the real tree:

```
schema entries --> TreeEntry list --> visible_rows(entries, p.filter, p.collapsed)
  Namespace row --> "[Editor]" = expanded|collapsed
  Key row       --> "editor.font_size" = <effective value> [(unsaved)]
```

Reusing `visible_rows` (the same pure builder the render and keyboard driver use) guarantees the capture matches the screen's ordering, filter, and collapse state. The effective value comes from `config_key_display_value`: a staged `Set` shows the edited value, otherwise `get_with_provenance` resolves the persisted/default value, formatted by `config_value_to_string` (scalars, arrays, nested-table key count, and a `Debug` fallback for the `#[non_exhaustive]` enum so no value is silently dropped). Because it enumerates all schema entries, Accessibility and every other category are captured when the schema has them. The builder moved to `screen_providers_config.rs`, leaving `screen_providers.rs` at ~302 non-test lines.

The SCRM Save-As `DialogScreen::screen_model` emits Name/Folder/Type as field values from `SaveAsState` directly, and a test (`save_as_model_carries_full_current_field_values`) locks the full typed filename against the earlier truncation. The other overlays build real content from their own state: external-exec carries the resolved command string, RESET BARE lists each profile to be reset, History/SWAP list real rows, Unsaved names the active workspace, About emits its heading/version/credits, and the replay picker lists the enumerated `*.ffscrm` sessions. All read STATE structs, never egui. The single redaction point remains `active_dialog_capture` -> `ScreenModel::redacted()`, proven for the Context path too since `active_screen_capture` checks the dialog capture first; the `redact.rs` tests confirm the secret is absent and the placeholder present across PlainText/Ansi/Markdown/Html/Yaml.

## FIX 3 -- side-effect-free SNAPSHOT in the Config workspace

Root cause was a double-consume of the same Enter: the command field detected submit with `key_pressed(Enter)` without removing the event, so the Config tree's per-frame keyboard driver (which owns the keyboard whenever a non-tree widget like the command field holds focus) also read that Enter and ran its collapse/activate gesture. The captured screenshot therefore showed a mutated (collapsed) screen. The fix switches both submit sites to `ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter))`, so only the command line acts on the Enter -- the same class of fix as the menu-bar arrow double-consume. The regression test reproduces the owner's scenario (command line focused in the Config panel, `SNAPSHOT` + Enter) and asserts no namespace group collapses as a side effect, with a precondition that none was collapsed on entry.

## Seam reuse, file sizes, ASCII, focus contract, test quality

One composite clipboard path (`stash_composite_request` + the `update.rs` assembly), one dispatch seam (the `COPY` verb goes through `handle_command`/the ladder like every built-in verb; buttons and Ctrl+C converge on `copy_frontmost_surface`). No second navigation stack or dispatcher. `active_screen_capture` is byte-for-byte unchanged for the no-overlay case, so the Context ScreenProvider contract and first-Tab focus behaviour are untouched (the commit records `first_tab` 15 passed). File sizes: `dialog_capture.rs` 511 total with `mod tests` at line 239 (~238 non-test), `dialog_screen.rs` 180, `screen_providers_config.rs` ~143, `update_dialogs_confirm.rs` 202, all under 400 non-test; the split kept `update_dialogs.rs` and `screen_providers.rs` under the limit. New source files scanned clean for non-ASCII. Tests carry `// Validates:` with the cited requirements (27.3/27.4/27.5/27.8, 4.4a, 13.1-13.8, 24.x), are deterministic (capture seam + logical model, no OS clipboard, no printed secret), and the Save-As/replay Copy outcomes and `any_dialog_open` scoping are unit-tested without an egui frame.

</details>

<details>
<summary>File map</summary>

- `crates/ff-screen-model/src/redact.rs` (new) -- `ScreenModel::redacted()` + `REDACTED_PLACEHOLDER`, the single secret-masking transform.
- `crates/ff-screen-model/src/lib.rs` -- export `redact` module and placeholder.
- `crates/ff-desktop/src/shell/commands_snapshot.rs` -- extracted `stash_composite_request` shared by SNAPSHOT and dialog Copy.
- `crates/ff-desktop/src/shell/dialog_capture.rs` (new) -- `active_dialog_capture`, `copy_frontmost_surface`, `any_dialog_open`, per-overlay models + unit tests.
- `crates/ff-desktop/src/shell/dialog_screen.rs` (new) -- `DialogScreen` trait + impls for PendingExternal / ResetBareTarget / SaveAsState.
- `crates/ff-desktop/src/shell/screen_providers_config.rs` (new) -- faithful Config settings-tree model.
- `crates/ff-desktop/src/shell/screen_providers.rs` -- removed thin Config model; `active_display_title` made `pub(super)`.
- `crates/ff-desktop/src/shell/screen_provider.rs` -- `active_screen_capture` returns the frontmost dialog model when one is open.
- `crates/ff-desktop/src/clipboard_backend.rs` -- removed `place_text`; `raw::empty()` on open; composite test.
- `crates/ff-desktop/src/shell/render_command_line.rs` -- consume the submit Enter in both command-field paths (FIX 3).
- `crates/ff-desktop/src/shell/update_input.rs` -- `handle_dialog_copy_shortcut` (Ctrl+C-while-open).
- `crates/ff-desktop/src/shell/update.rs` -- call the Ctrl+C copy shortcut before rendering overlays.
- `crates/ff-desktop/src/shell/update_dialogs.rs` -- Copy buttons on About/History/SWAP/External; extracted confirm overlays.
- `crates/ff-desktop/src/shell/update_dialogs_confirm.rs` (new) -- RESET BARE + Unsaved overlays with Copy.
- `crates/ff-desktop/src/shell/update_dialogs_scrm.rs` -- `apply_scrm_save_as_outcome` / `apply_scrm_replay_outcome` handle `Outcome::Copy`.
- `crates/ff-desktop/src/shell/commands_ladder_b.rs` -- `COPY` verb routes to `copy_frontmost_surface`.
- `crates/ff-desktop/src/about_dialog.rs` -- `render` returns `copy_clicked`; adds Copy button (pre-existing non-ASCII comments noted).
- `crates/ff-desktop/src/scrm_save_as_dialog.rs` -- `Outcome::Copy` + Copy button.
- `crates/ff-desktop/src/scrm_session_picker.rs` -- `Outcome::Copy` + Copy button.
- `crates/ff-desktop/src/shell/tests_config_snapshot_fidelity.rs` (new) -- Config faithful-model + FIX 3 regression tests.
- `crates/ff-desktop/src/shell/tests_dialog_copy.rs` (new) -- full-shell Ctrl+C + per-dialog Copy outcome tests.
- `crates/ff-desktop/src/shell/tests_scrm.rs` -- COPY composite + no-overlay Context capture tests.
- `crates/ff-desktop/src/shell/mod.rs` -- register new modules.
- `crates/ff-i18n/i18n/{en,de,fr,af}/*.ftl` -- `ff-desktop-shell-dialogs-copy` key (ASCII).

Full diff: `git -C .worktrees/scrm-2e-dialog-capture diff main`
</details>
