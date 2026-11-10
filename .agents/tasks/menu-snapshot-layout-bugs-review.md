# POM/menu snapshot reworked to the owner's ISPF mock (3 bugs)

Two commits on `scrm-2e` (`e505035` adds a verbatim `body_lines` path to the shared `ScreenModel`; `d4f10e6` rewrites the menu/POM snapshot to use it) make the menu capture match the owner's authoritative ISPF mock. The menu Context now owns its own body: it composes the `Command ===>` line, a two-column options-left / calendar-right body, blank spacers, and the pfshow-gated PF-key footer as exact text lines, instead of going through the generic label/value `fields` path. The former Phase 2A "header chrome" (window title, menu-bar echo, `[POM]` tag, duplicated title) is deleted for menus. Calendar/time/day-of-year are pulled from the same `chrono::Local::now()` plus `primary_option_menu` helpers the live POM uses.

Watch for: the option-row short name is not internally padded to a common width, so the left column's `:` separators won't line up the way the owner mock draws them (`Settings` / `Help    ` / `SCRM    `) — cosmetic, content is correct (likely). The `Command ===>` input width is a hard-coded `119` magic constant (confirmed). Both are minor; neither blocks the fix.

**Verdict**: APPROVED

## High-level view

Bug 2 (superfluous header block) is fully removed. `MenuScreenChrome` dropped its `window_title` / `menu_bar` / `tab_header` / `title_line` fields; `menu_screen_chrome` in `screen_provider.rs` stopped building them; the old `with_header_line` population is gone. A test asserts `header_lines` and `footer_lines` are empty and that neither `[POM]` nor a duplicated title line appears in the body. The title survives only in `ScreenModel::title`, drawn in the frame border.

Bug 1 (command line) is fixed by emitting `Command ===> {typed}{underscores}` as the first `body_line`, which the text renderer places directly under the title separator (row index 3: top border, title, separator, first body row). `ScreenModel::command_line` is deliberately left empty so the generic renderer does not also echo a command line outside the frame, avoiding a double render.

Bug 3 (two-column layout) is handled by `compose_two_column_body`, which pads each option row to the widest left row plus a fixed gap and joins the right-column block (calendar + `Time` + `Day of Year`) on the same line. When the menu hides the calendar the right column is empty and the body collapses to a single left column. Tests assert a single line carries both an option and the calendar, that the right column begins at a consistent origin, and that the left text ends before that origin (no overlap).

The live-POM reuse is genuine: `menu_screen_chrome` computes the date from `chrono::Local::now()`, derives the ordinal via `primary_option_menu::day_of_year`, and feeds `calendar_right_column`, which builds on the pre-existing `calendar_lines`. No second calendar/day-of-year format was invented in the snapshot path.

Scope is clean. The diff touches only the six files of the menu-model/layout and the generic renderer path. The dialog capture, crop, redaction (`ScreenModel::redacted`), and CF_HTML/monospace clipboard code (`clipboard_backend.rs`, `clipboard_html.rs`, `commands_snapshot`) are untouched. Phase 2A tests that asserted the old header-chrome/field layout were rewritten to the new ISPF layout rather than deleted, and new deterministic tests (clock/date stubbed) cover each bug, all cited `// Validates: screen-snapshot-scrm Req 3.8-3.13`.

<details>
<summary>Issues (2)</summary>

1. **Short-name column not padded** — option-row short names are emitted with a single space (`X X : ...`), so the `:` separators in the left column do not align as the owner mock draws them. Cosmetic; the key/short-name/description/(command) content is correct. Consider padding the short name to a shared width if exact mock fidelity is wanted.
2. **Hard-coded command input width** — `COMMAND_INPUT_WIDTH = 119` is a magic constant matching the owner's mock line length. Documented, but brittle if the frame width changes. Non-blocking.

</details>

<details>
<summary>Details</summary>

### Bug 2 -- the header-chrome block is gone

`MenuScreenChrome` now carries only `right_column` and `key_bar_lines`; the four header fields (`window_title`, `menu_bar`, `tab_header`, `title_line`) are deleted from the struct, from the shell's `menu_screen_chrome` builder, and from `menu_workspace_screen_model_full`, which no longer calls `with_header_line` at all. The standalone `menu_workspace_screen_model` (the option-field model) was removed entirely, so the full builder is the only menu path and it owns the body.

`full_model_is_two_column_body_without_header_chrome` locks this down: `model.header_lines.is_empty()`, `model.footer_lines.is_empty()`, `model.fields.is_empty()`, `model.command_line.is_empty()`, no `[POM]` substring in the body, and zero body lines equal to the title. This is the direct inverse of the removed `full_model_includes_chrome_and_option_rows` assertion, so the supersede is explicit rather than silently dropped.

### Bug 1 -- `Command ===>` as the first body row

```rust
let typed = command_line.trim_end();
let underscores = COMMAND_INPUT_WIDTH.saturating_sub(typed.chars().count());
model = model.with_body_line(format!("Command ===> {typed}{}", "_".repeat(underscores)));
```

Because `body_lines` are pushed immediately after the title and the text renderer frames them as the first content rows under the title separator, the command line lands at output row 3. `rendered_menu_has_command_line_as_first_body_row` asserts `rows[3]` starts with `Command ===> ` and contains an underscore, and that a typed value (`SNAPSHOT`) appears before the underscores. Leaving `ScreenModel::command_line` empty suppresses the generic outside-the-frame echo (`body_lines_render_verbatim_as_first_body_rows` asserts no `\nCommand ===> \n` leak).

### Bug 3 -- two-column body with aligned calendar

`compose_two_column_body` computes `left_width` as the max option-row width, then for each row pads the left text to `left_width + MENU_COLUMN_GAP` before appending the right-column entry; rows past the shorter column are padded or left bare. `calendar_right_column` appends `Time` and `Day of Year` below the calendar with the labels left-padded to a fixed width so the colons align:

```rust
lines.push(format!("{:<17} : {:02}:{:02}", "Time", hour, minute));
lines.push(format!("{:<17} : {:>3}", "Day of Year", day_of_year));
```

`two_column_calendar_right_edge_is_aligned` recomputes the expected right-column origin and asserts the month header begins exactly there and the left option text ends at or before it, which is the no-overlap guarantee. `calendar_less_menu_renders_single_column` covers the Settings-style menu (empty right column → single left column, no calendar artefacts).

The short-name alignment deviation is here: the mock draws the short names padded to a common width inside the left column, but `option_row_text` emits `{key} {command} : ...` with a single space. The content matches the mock; only the internal `:` alignment differs.

### Live-POM reuse

`menu_screen_chrome` is the single place the snapshot's calendar is built, and it uses the live source:

```rust
let now = chrono::Local::now();
let date = now.date_naive();
let doy = crate::primary_option_menu::day_of_year(date.year(), date.month(), date.day());
crate::screen_snapshot::calendar_right_column(date.year(), date.month(), date.day(),
    now.hour(), now.minute(), doy)
```

`calendar_right_column` wraps the existing `calendar_lines`, so the month grid, weekday header, and today-marker are the same code the live POM renders. Tests pass fixed components (`calendar_right_column(2026, 10, 25, 12, 58, 298)`) to stay deterministic — the production clock is read only in the shell, not in the model, so the tests never touch the real clock.

### Renderer path and scope

`ScreenModel` gains `body_lines` (serde `default`, so existing serialized models are unaffected) and `with_body_line`. The text renderer emits `body_lines` verbatim as the first body rows with no leading blank; html wraps them in a monospaced `<pre class="ffwb-body">`; yaml emits a `body:` sequence; ansi inherits the text path. A Context that leaves `body_lines` empty renders exactly as before, so no other Context's capture changes.

The six-file diff is confined to the menu model, the shell's chrome builder, and the generic renderer. `redacted()`, the dialog crop, and the CF_HTML/monospace clipboard builders are not in the change set, confirming the "menu snapshot only" scope. The `screen_provider` full-shell test was updated from asserting `!model.fields.is_empty()` to `!model.body_lines.is_empty()`, consistent with the new body path.

</details>

<details>
<summary>Files changed (6)</summary>

- `crates/ff-desktop/src/screen_snapshot.rs` — removed header-chrome fields and the old `menu_workspace_screen_model`; added `option_row_text`, `compose_two_column_body`, `calendar_right_column`; rewrote `menu_workspace_screen_model_full` to compose the two-column body; rewrote/added tests.
- `crates/ff-desktop/src/shell/screen_provider.rs` — `menu_screen_chrome` now builds only `right_column` (from live clock + `primary_option_menu::day_of_year`) and `key_bar_lines`; full-shell test updated to `body_lines`.
- `crates/ff-screen-model/src/model.rs` — added `body_lines` field + `with_body_line`; added carries-body-lines test.
- `crates/ff-screen-model/src/render/text.rs` — emits `body_lines` as first body rows; added verbatim-body test.
- `crates/ff-screen-model/src/render/html.rs` — emits `body_lines` in a monospaced `<pre>`.
- `crates/ff-screen-model/src/render/yaml.rs` — emits a `body:` sequence.

Full diff: `git -C .worktrees/scrm-2e-dialog-capture diff 4e33654 d4f10e6`.

</details>
