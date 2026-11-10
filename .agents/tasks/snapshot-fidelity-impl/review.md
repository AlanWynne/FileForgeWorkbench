# Full-window snapshot fidelity (Req 29, CR-CH-115)

This change makes the box-drawn snapshot/capture a faithful top-to-bottom mirror of the whole framed window for every capturable Context, not a thin collapsed box. It lands in two shared places exactly as the design prescribes: a new `ff-desktop` chrome helper (`shell/screen_chrome.rs`) that prepends the window title bar + `[XXX]` tab tag, folds the top `Command ===>` line, sets the bottom info bar, and emits the F-key line only when visible; and a grid-aware `ff-screen-model` text renderer that replaces the old `join("  ")` with a `|`-separated fixed-width grid padded out to the frame width. Cell conventions (`[value v]` / `[ v]` / `____`) and the always-present scratch row are provider-owned cell strings, so they flow identically to the HTML `<td>` flavor with no flavor-specific code. Every per-Context builder and the POM/menu path route through the one helper, and snapshot and capture share the one `ScreenProvider` seam, so the fidelity reaches both paths through a single renderer. No public `ScreenModel`/`Table` type is reshaped, no `CellKind` enum, no new dispatcher/nav/persistence/clipboard flavor.

Watch for: (confirmed) the code cites "Req 29.4b" in several places but no 29.4b criterion exists in requirements.md or design.md — a traceability label mismatch, not a behavioral defect (the "each chrome element exactly once" intent is really Req 29.3 + 29.15). (possible) the Alias builder still emits per-row `fields` (the `SS . . . : SNAPSHOT` dot-leader lines) in addition to the grid table, so alias row data appears twice in the box; this is pre-existing and outside the Req 29 diff.

**Verdict**: APPROVED

## High-level view

The top chrome is correct against the real renderer. Because `render_plain` emits `model.title` in the border, then `body_lines` directly under the separator, then `header_lines` *below* the body, the helper deliberately prepends the title bar + tab tag + command line as the first three `body_lines` rather than as `header_lines`. The workspace title stays in the border, emitted once. `model.command_line` is cleared after folding so the renderer does not also echo it below the frame — this directly fixes the owner-observed "Command ===> ss" floating under the box.

The grid is a dumb fixed-width `|` renderer: per-column width is max(header, every cell including the scratch row), a wider value widens the column and pads every row (no truncation), and the whole frame pads out to `max(widest content line, model.dimensions.columns)` so a narrow table still fills the 80-col default frame. The combo/editable/scratch conventions are not invented by the renderer; the provider (`alias_table()`) emits them as cell text, which is the mechanism that makes the plain-text and HTML flavors identical without touching `Table`.

Bottom chrome tracks live state: the info bar is always a defined non-empty line sourced from the shell status (settled-but-tunable), and the F-key line is appended only when `key_bar_visible`, reusing the same `key_bar_line` builder the POM path uses. The POM/menu path is reconciled to single chrome — it regains the title bar + `[POM]` tag (which Bug-2 had removed) via `apply_menu_window_chrome`, which prepends only those two lines and leaves the menu's own command line and PF footer untouched, so nothing is duplicated.

Framework conformance holds: no public type reshape, no `CellKind` enum (explicitly rejected in design and not present), resolver/dispatch/nav/persistence untouched, PNG and property-table flavors unchanged. All changed source files are well under 400 non-test lines and the changed source + tasks.md are ASCII-clean.

The TDD evidence is present and well-annotated. Phase 2G ships failing-test-first unit tests for the grid renderer and the HTML flavor in `ff-screen-model`, chrome-helper unit tests in `ff-desktop`, and full-shell `egui_kittest` fidelity tests covering a table Context (Alias), a `markdown_sections` Context (Config), an F-key visibility toggle, the HTML flavor, the POM single-chrome reconciliation, and a capture-path test asserting the capture seam renders byte-identical to the snapshot seam. Every test carries a `// Validates: Requirement 29.x` annotation. The commit records scoped `cargo test`/`check`/`clippy`/`fmt` all green, with the full gate correctly deferred to the owner per gate-policy.

<details>
<summary>Issues (2)</summary>

1. **"Req 29.4b" cites a non-existent criterion** — `screen_chrome.rs`, its doc comments, several test names, and the task prompt refer to "Req 29.4b (single chrome)", but requirements.md and design.md define no 29.4b. The behavior is correct and covered by Req 29.3 ("exactly ONCE and is not duplicated") and 29.15 (Bug-2 reconciliation). Relabel the citations to 29.3/29.15, or add the 29.4b sub-criterion to requirements.md so the annotation resolves. Non-blocking traceability fix.
2. **Alias box renders row data twice** — `alias_editor_screen_model` populates `model.fields` (dot-leader lines) *and* the grid table, so each alias row appears both as a `SS . . . : SNAPSHOT` field line and as a grid row. Pre-existing (not introduced by this diff) and not a Req 29 violation, but it works against the "faithful mirror" intent; worth a follow-up to decide whether the field lines should be dropped for table Contexts.

</details>

<details>
<summary>Details</summary>

### Top chrome and the renderer-order reconciliation (29.1-29.4)

`apply_full_window_chrome` prepends three lines ahead of the Context body — the window title bar (`# | FileForge Workbench ... _ [] X`), the active `[XXX]` tab tag from `BuiltinKind::default_title()`, and the folded `Command ===> <value><underscores>` — as the first three `body_lines`. This is the right call against `render_plain`, which emits `body_lines` directly under the title separator but `header_lines` below the body; putting the chrome in `header_lines` would have failed "begin with the title bar". The command line is folded at the shared `COMMAND_INPUT_WIDTH` (119) via `command_line_body_line`, the same helper the POM builder now uses, satisfying 29.4's "same run length across every Context". After folding, `model.command_line` is cleared so the renderer does not echo it outside the frame — the concrete fix for the owner's floating command line. `tests_screen_directive.rs` was updated to assert exactly this (empty `command_line`, command as a top body row).

The title-bar marker, glyphs, and status source are all marked settled-but-tunable in the code, consistent with the spec's 29.1/29.10 language. `WINDOW_TITLE` is a constant mirroring `main.rs`; the comment correctly notes it must be re-sourced if the window title becomes localized (the owner raised this question), so the follow-up is anticipated rather than silently baked in.

### Grid renderer (29.5-29.9, 29.16)

`render_grid_table` computes per-column widths as max(header, every cell), pads each cell, and joins with ` | `. A value wider than its header widens the column and every row pads to it — the no-truncation overflow rule, with a unit test (`table_cell_wider_than_header_widens_column`). Frame width is `max(widest content line, model.dimensions.columns)`, so `narrow_grid_pads_to_frame_width` asserts an 80-col-minimum rectangular frame — this is the distinct 29.6a behavior layered on top of the 29.6 per-column sizing. The ASCII fallback keeps `|` because the grid separator is content, not frame (`table_grid_uses_pipe_in_ascii_fallback`). A box-only model with no table is unchanged (`box_only_model_without_table_is_unchanged`), covering 29.16.

The cell conventions live in the provider: `combo_cell`/`editable_cell` in `screen_providers_tables.rs` produce `[value v]` / `[ v]` / `____` / verbatim value, and `alias_table()` appends a full row of empty cells as the scratch row. This is the design's key decision — the renderer stays dumb, so the same strings reach the HTML `<td>` emitter. Confirmed by reading `render/html.rs`: the table path emits `<td>{escape(cell)}</td>` per row with no transformation, and the new `html_grid_carries_provider_cell_strings_and_scratch_row` test asserts `<td>[FFCMD v]</td>`, `<td>____</td>`, and the scratch `<tr>`. No html.rs production change was needed, which is correct given the pre-existing verbatim emitter.

### Single shared path to both snapshot and capture (29.12, 29.14)

`active_screen_capture()` is the one seam; `snapshot_text_for_active` (snapshot) and `active_screen_model()` -> `into_model()` (capture) both consume it and render through the same `ff-screen-model` renderer. `full_shell_capture_path_renders_identical_box_as_snapshot` drives the real shell and asserts the two render byte-identical, which is the strongest available proof that no divergent capture renderer exists. Generality is proven by running the full-shell fidelity assertions over both the Alias table Context and the Config `markdown_sections` Context.

### POM reconciliation to single chrome (29.15)

`apply_menu_window_chrome` prepends only the title bar + `[POM]` tag and sets the info bar; it does not add a command line or F-key line because the menu builder already owns those, avoiding duplication. `full_shell_pom_snapshot_regains_title_bar_and_pom_tag_single_chrome` counts `# | FileForge Workbench`, `_ [] X`, `[POM]`, and `Command ===>` each exactly once, and correctly matches the title-bar marker rather than the bare brand word so the POM's descriptive border title ("FileForge Workbench -- Primary Option Menu") doesn't cause a false double-count.

### Framework conformance and hygiene

Verified absences: PNG (Req 23) and property-table (Req 28) flavors are not in the diff; the rejected `CellKind` enum does not appear. Changed source file sizes: `screen_chrome.rs` 160, `screen_providers.rs` ~351, `screen_snapshot.rs` ~276 non-test, `text.rs`/`html.rs` ~186/~223 non-test — all under 400. ASCII scan of the changed source and tasks.md is clean (box-drawing only appears in rendered runtime output, the documented exemption).

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/shell/screen_chrome.rs` (new) — shared `apply_full_window_chrome` / `apply_menu_window_chrome` helpers; title bar, tab tag, top command line, info bar, F-key-when-visible.
- `crates/ff-desktop/src/shell/screen_chrome_tests.rs` (new) — unit tests for the helper (first-two-body-rows, top command line, info bar + F-key visibility, exactly-once, pure logical state).
- `crates/ff-desktop/src/shell/tests_snapshot_fidelity.rs` (new) — full-shell egui_kittest fidelity proofs: Alias table, Config markdown_sections, F-key toggle, HTML flavor, capture==snapshot, POM single chrome.
- `crates/ff-desktop/src/shell/screen_providers.rs` — route every per-Context builder through the chrome helper; `active_tab_tag` helper.
- `crates/ff-desktop/src/shell/screen_providers_config.rs` — Config builder routes through chrome.
- `crates/ff-desktop/src/shell/screen_providers_tables.rs` — provider-owned `combo_cell`/`editable_cell` conventions + scratch row in `alias_table()`.
- `crates/ff-desktop/src/shell/screen_provider.rs` — POM path calls `apply_menu_window_chrome`.
- `crates/ff-desktop/src/screen_snapshot.rs` — shared `command_line_body_line` at the common field width.
- `crates/ff-desktop/src/shell/mod.rs` — register `screen_chrome` + `tests_snapshot_fidelity` modules.
- `crates/ff-desktop/src/shell/tests_editor_snapshot_tables.rs`, `tests_screen_directive.rs` — updated assertions for combo cells, scratch row, folded command line.
- `crates/ff-screen-model/src/render/text.rs` — grid-aware `render_grid_table`; frame width from `dimensions.columns`.
- `crates/ff-screen-model/src/render/html.rs` — test proving the HTML `<td>` flavor carries the identical provider cell strings + scratch row.
- `docs/specs/screen-snapshot-scrm/tasks.md` — Phase 2G tasks marked complete.

Full diff: `git -C .worktrees/snapshot-fidelity-impl diff main...HEAD`.

</details>
