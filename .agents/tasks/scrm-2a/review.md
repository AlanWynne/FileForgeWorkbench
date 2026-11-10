# Stream SCRM Phase 2A: full-screen menu snapshot and per-Context ScreenProviders

Phase 2A does two things. C.1 enriches the POM/menu snapshot from a bare option list into the FULL screen the user sees: window title bar, menu bar, tab header, title line, the calendar, and the PF-key footer gated on pfshow. This is done by adding ordered `header_lines` / `footer_lines` chrome to `ScreenModel`, threading them through all five renderers, and having the shell build the chrome from its logical state (never egui). C.2/C.3 route every core Context (the master-directive Req 5 retrofit matrix) through the existing Phase 1A `ScreenProvider` dispatch seam, each returning a `ScreenModel` built from its own panel/state struct. The single remaining documented opt-out is the parked editor Context (FileEditor / Untitled). The conformance harness is rewritten to walk every retrofitted Context and assert it is capturable.

Watch for: a confirmed cosmetic bug in the snapshot calendar's "today" marker that corrupts double-digit day numbers (confirmed); the harness now exercises Contexts via raw tab-kind swap rather than the real command path, which drops the dispatch-path coverage Phase 1A had (confirmed); and the menu-bar chrome reads `resolve_menu_bar_menu()` while the option list is `mw`, a two-source split worth confirming is intended (possible).

**Verdict**: APPROVED

## High-level view

The `ScreenModel` extension is additive: two new `Vec<String>` fields (`header_lines`, `footer_lines`), both `#[serde(default)]` so existing serialized models stay compatible, with builder methods and consistent handling across text, html, yaml, and `visible_text_fragments` (ansi/markdown delegate through `render_plain`). The existing menu-snapshot path does not change shape; the fuller output is spec-aligned growth against Req 3.9-3.13, as the task frames it.

The chrome is built in `menu_screen_chrome` from live logical state (menu-bar options, tab header from `menu_name_label`, title via `title_line_text`, a date-derived calendar, and the PF-key line only when `key_bar_visible`). This honours the no-egui-screengrab constraint and the pfshow gating is correct: an empty `key_bar_lines` means the footer is absent from the capture.

The per-Context providers live in a new `screen_providers.rs` (327 lines, under the 400 limit) with one small `*_screen_model()` builder each, dispatched by `KindTag` in `screen_provider.rs`. The dispatch is a single exhaustive `match` with no catch-all, so adding a Kind forces a compile decision. Every builder reads only fields that exist and are safe to index (`commands[0]` is a `[String; 4]` array), and the two parked-domain panels (Catalog Explorer, File Explorer) record an "unavailable in core" message rather than an empty model.

The ScrmViewer decision (provider, not opt-out) is justified: it has logical content (replay position plus the current captured screen text). The editor opt-out is the lone documented exception and shares one `PARKED_EDITOR_REASON` constant between dispatch and harness so they cannot drift.

The weak spot is the calendar "today" marker, which truncates to two characters and mangles day numbers 10-31 in the captured text. Snapshot-only and cosmetic, but it ships a visibly wrong value.

<details>
<summary>Issues (3)</summary>

1. **Calendar today-marker truncation** (confirmed) -- `format!("*{day}").chars().take(2).collect()` renders day 25 as `*2`, day 10 as `*1`, etc. The marker corrupts the day number for every double-digit day. Snapshot-only and non-blocking, but should be fixed (mark single-digit days only, widen the cell, or use a trailing marker that keeps both digits).
2. **Harness no longer drives the real command path** (confirmed) -- `assert_capture_contract` switched from `dispatch_command_string(command)` to `set_active_tab_context(kind, ...)`. It still proves every Context is capturable through the seam (the Req 5 ask), but no longer proves the command path opens each Context, which Phase 1A's version did. Add a follow-up test or a note so the coverage change is intentional.
3. **Menu-bar chrome sourced separately from the option list** (possible) -- `menu_screen_chrome` builds `menu_bar` from `self.resolve_menu_bar_menu()` while the option rows come from `mw`. If those ever resolve different menus, the captured menu bar and body could describe different screens. Confirm the global-bar semantics are intended.

</details>

<details>
<summary>Details</summary>

### ScreenModel chrome: additive and compatible

The two new fields are ordered `Vec<String>` with `#[serde(default)]`, so a previously serialized `ScreenModel` still deserializes. All five render targets handle the new elements: text inserts header chrome directly under the title and footer chrome below the body (both inside the frame), html emits `ffwb-header` / `ffwb-footer` paragraphs with escaping, yaml emits `header:` / `footer:` sequences, and `visible_text_fragments` extends with both (filtering empties). ansi and markdown ride on `render_plain`, so they inherit the behaviour. This is the kind of change where a renderer is easy to miss; all five were updated.

### Chrome built from logical state, pfshow gated correctly

`menu_screen_chrome` reads the menu bar from `resolve_menu_bar_menu().options` (bar-visible descriptions), the tab header from `menu_name_label()` (falling back to `tab_title()`), the title from `title_line_text(tab)`, and the calendar only when the loaded menu's `show_calendar` is set. The PF-key footer is produced only inside `if self.key_bar_visible`, so when pfshow is off the footer vector is empty and the renderers omit it. The `full_snapshot_pfkey_footer_follows_pfshow_state` test asserts both directions (F1 Help present when on, absent when off) and `full_snapshot_text_contains_calendar` asserts the month header appears; together these satisfy the task's explicit "calendar + PF-key on/off" requirement, annotated to Req 3.9/3.10. No egui is read in the chrome path.

### The calendar today-marker bug

```rust
for day in 1..=total {
    if day == today {
        // Mark today (ASCII only). Trim to keep the two-char grid aligned.
        cells.push(format!("*{day}").chars().take(2).collect());
    } else {
        cells.push(format!("{day:>2}"));
    }
}
```

For single-digit days the marked cell is `*1`..`*9` (two chars, aligned). For double-digit days `*{day}` is three chars and `take(2)` keeps `*1` (10-19), `*2` (20-29), or `*3` (30/31): the day number is corrupted. A capture taken on the 25th shows `*2` in the grid, reading as the 2nd. The live on-screen calendar is unaffected (separate render path), so this is a snapshot-only cosmetic defect, but it produces a wrong value in the artifact the feature exists to produce. Keeping both digits (trailing marker, single-digit-only marking, or three-char cells) would resolve it. Non-blocking.

### Per-Context providers read only safe, existing state

Each `*_screen_model()` builder was checked against the real struct fields: `config_panel.{namespace_filter, filter, is_dirty(), working, persist_error}`, `keys_editor_panel.rows[].{key, commands}` (array, safe index), the editor panels' `selected`/`available`/`error`, `command_configurator_panel.displayed(...)`, `search_results_panel.{query, results, regex_error}`, `plugin_manager_panel.plugins`, `event_log_panel.{level_filter, text_filter}` plus the notification queue, `macro_library_panel.{filter, entries}`, `help.context_panel.current_topic_text()`, and `scrm_viewer.{replay_summary(), current_screen_snapshot_text()}`. The notification-queue lock in `event_log_screen_model` is taken and dropped within the function with no await across it. The parked Catalog/File Explorer providers emit a message model rather than an empty one, so a capture is faithful.

### Dispatch and opt-out are single-sourced

`active_screen_capture` is one exhaustive `match` on `KindTag` with no catch-all, so a future Kind will not compile until it is added to the dispatch. `retrofit_matrix_is_covered` is a `#[allow(dead_code)]` compile-time reminder listing the covered Kinds, belt-and-braces alongside the exhaustive match. The shared `PARKED_EDITOR_REASON` constant is used by both the dispatch and the harness assertion, so the opt-out text cannot drift between them.

### Conformance harness as the Req 5 checklist

`conformance_every_retrofitted_context_is_capturable` walks all fifteen matrix Contexts and asserts `Capturable` at both the seam (`ScreenCapture::Model`) and the user-visible SNAPSHOT (non-empty, no "cannot be captured yet"). `conformance_parked_editor_reports_documented_opt_out` pins the two editor Kinds to the opt-out at both layers. This is the authoritative Req 5 walk. The regression from Phase 1A: `assert_capture_contract` now sets the tab kind via `set_active_tab_context` instead of opening each Context through `dispatch_command_string`, dropping the incidental proof that the command path reaches each Context. The simplification is defensible (not every matrix Kind has a trivial open-command, and the test's purpose is the capture contract), but the coverage change should be a conscious one.

### Constraints

WorkspaceContext / InteriorFocus files are untouched by the diff. No new dispatcher; everything builds on the Phase 1A seam. No egui is read in any capture path. ASCII-only scan of the changed source files is clean. `screen_providers.rs` is 327 non-test lines and the grown `screen_provider.rs` is ~217 non-test lines, both under 400. The coder's recorded scoped results (check/clippy/fmt clean; 16 screen + 33 scrm + 18 snapshot + 24 ff-screen-model tests passing) are internally consistent with the code read here; per instructions these were not re-run.

</details>

<details>
<summary>File map</summary>

- `crates/ff-screen-model/src/model.rs` -- adds `header_lines` / `footer_lines` to `ScreenModel` with builders and a unit test.
- `crates/ff-screen-model/src/render.rs` -- `visible_text_fragments` extended with header/footer lines.
- `crates/ff-screen-model/src/render/{text,html,yaml}.rs` -- render header/footer chrome in each format.
- `crates/ff-desktop/src/screen_snapshot.rs` -- `MenuScreenChrome`, `menu_workspace_screen_model_full`, `calendar_lines`, `key_bar_line`, plus three C.1 tests.
- `crates/ff-desktop/src/primary_option_menu.rs` -- `month_name` helper (un-dead-codes `MONTH_NAMES`).
- `crates/ff-desktop/src/shell/screen_provider.rs` -- full-screen menu capture, per-Context dispatch, `menu_screen_chrome`, `PARKED_EDITOR_REASON`.
- `crates/ff-desktop/src/shell/screen_providers.rs` -- new: one `*_screen_model()` builder per Context plus the coverage reminder.
- `crates/ff-desktop/src/shell/mod.rs` -- register the new module.
- `crates/ff-desktop/src/shell/tests_screen_directive.rs` -- rewritten conformance harness walking every retrofitted Context.
- `crates/ff-desktop/src/help_context.rs` -- `current_topic_text` accessor.
- `crates/ff-desktop/src/scrm_viewer_panel.rs` -- `replay_summary` + `current_screen_snapshot_text` accessors.

Full diff: `git -C .worktrees/scrm-2a-fullscreen-providers diff main...HEAD`

</details>
