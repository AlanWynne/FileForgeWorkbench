# Symmetric box-plus-tables clipboard flavors for screen snapshots

This change makes every clipboard TEXT flavor of a screen snapshot carry both the box-drawn layout and the property/value tables, where before each flavor carried only one representation. The Markdown (`CF_UNICODETEXT`) path previously early-returned a tables-only render when a model supplied `markdown_sections`; it now emits the fenced box body first and then the tables. The rich (`CF_HTML`) path gains a new `tables_html` argument on `build_cf_html` whose content is appended after the monospace box paragraphs, so Word receives image, then box, then HTML tables. The table markup for both flavors is produced from the same logical source by renderers that live in the shared `ff-screen-model` crate, so any table-bearing screen model inherits the behavior. The PNG picture flavor is untouched.

Watch for: the ordering guarantees rest on byte-index assertions that are correct but coupled to the exact span style string (likely); the `clipboard_html.rs` test module is now ~307 lines, past the ~200-line split guideline, though that length predates this change (possible).

**Verdict**: APPROVED

## High-level view

The design keeps the fix at the right layer. The Markdown renderer refactors the fenced-body emission into a shared `push_fenced_body` helper used by both the generic and the `markdown_sections` paths, so the box always precedes the tables, and the new `render_tables_html` emitter in `ff-screen-model` is the HTML structural twin of the Markdown table output, derived from the identical source (`markdown_sections` when present, else `fields`/`tables`). This is what makes criterion 28.6 (same data, same structure, different syntax) and 28.7 (general, not Config-specific) hold: the logic is in the shared crate and keyed off model content, with Config appearing only in test fixtures.

The CF_HTML composite change is minimal and matches the one allowed signature change: `build_cf_html` gains a `tables_html: &str` third parameter, appended inside the fragment after the per-line monospace spans and before `EndFragment`, with the existing two-pass offset math recomputing the header. The single `ff-desktop` caller in `place_snapshot_composite` passes `render_tables_html(model)`. An empty `tables_html` leaves the box-only output byte-identical, which is both asserted and reasoned about in the doc comments.

The PNG flavor is proven independent of the table work by a test that rasterises a model with and without `markdown_sections` and asserts byte-identical PNGs; no picture-path code is touched in the diff.

TDD coverage is present and each test carries a `// Validates: Requirement 28.x` annotation across the two text flavors and the general-scope case (a `fields`-only model and a `tables`-only model also get both representations). The two pre-existing fidelity assertions that depended on a key appearing once in the Markdown flavor were correctly reconciled to expect twice (box body plus table row), with plain-text still once.

<details>
<summary>Issues (3)</summary>

1. **Test module length in clipboard_html.rs** (possible) -- the `tests` module is now ~307 lines, over the ~200-line split guideline. It was already large before this change (3 tests / ~69 lines added here), so not a regression; consider splitting into a `_tests.rs` sibling in a later pass. Non-blocking.
2. **Ordering assertions coupled to span style string** (likely) -- the box-before-tables ordering tests locate the box via the literal `<span style="font-family:'Consolas',...>` string. Correct today, but a future style tweak would break the test rather than the behavior. Acceptable for now.
3. **dual_representation module adds ~123 lines to commands_snapshot.rs test area** (possible) -- combined with `dialog_crop_tests` the file's test region is ~211 lines across two modules; non-test code stays ~375 (under 400). Informational.

</details>

<details>
<summary>Details</summary>

### Markdown flavor: box first, then tables (28.1, 28.4)

The prior `render_markdown` early-returned `render_markdown_sections` when `markdown_sections` was non-empty, producing a tables-only block and dropping the box. The fix hoists the fenced-body emission into `push_fenced_body` and calls it before `render_markdown_sections` on the sections path, and keeps the generic path (fenced body + fields table + `tables`) as before. This directly matches criterion 28.4 (box first, then tables) and 28.1 (both representations in one plain-text block). The ordering is proven by `markdown_sections_render_box_then_tables`, which asserts the `U+250C` glyph index precedes the `| Property | Value |` header index.

One behavioral subtlety is handled: a heading-less section previously separated itself with a blank line only when `idx > 0`. Now that a fenced box body always precedes the sections, the separator logic was changed to key off whether `out` already ends in a blank line (`!out.ends_with("\n\n")`) rather than the index. This keeps a heading-less intro table correctly spaced after the fenced block.

### HTML table emitter in the shared crate (28.2, 28.6, 28.7)

`render_tables_html` renders only the tabular content, no title or chrome, because the composite already carries the box as monospace spans. It mirrors the Markdown source selection exactly: `markdown_sections` when present, else the `fields` property table followed by each `tables` element. The shared `push_data_table` helper gives every table identical markup (caption from the group heading, a `Property`/`Value` header row, one `<tr>` per row), satisfying 28.6's "same data and group structure." It returns an empty string for a box-only model, so a Context with no tabular content is unaffected (28.7). The generality claim is backed by `render_tables_html_falls_back_to_fields_and_tables` (a `fields` + `tables` model, no `markdown_sections`), which is the non-Config path.

### CF_HTML composite: image, box, tables, offsets intact (28.2, 28.4, 28.5)

`build_cf_html` and its `fragment_html` helper gain `tables_html: &str`, appended after the per-line monospace spans with a `\r\n` separator, only when non-empty. The per-line `<span>` monospace treatment (23.5f) is untouched, so 28.5's "do not regress the monospace treatment" holds. The two-pass offset computation is unchanged and operates on the now-longer body, and `cf_html_offsets_round_trip_with_tables_present` asserts all four header offsets still index the fragment markers and that the tables live inside the bracketed fragment. The image/box/tables order is asserted by byte index in `cf_html_carries_image_box_then_tables_in_order`.

The box-only invariant (28.3/28.7) is covered by `cf_html_box_only_unchanged_when_no_tables`, which checks no `<table>` markup leaks in and the body still ends at the last line paragraph exactly as before. The doc comment also states the empty-`tables_html` output is byte-identical to pre-Req-28, consistent with the append-only guard.

### PNG picture flavor untouched (28.3)

No code on the rasterise/encode path is in the diff. `png_picture_flavor_unchanged_by_tables` makes the independence explicit: it rasterises the fixture and a clone with `markdown_sections` cleared and asserts the encoded PNGs are byte-equal, confirming the picture depends only on the model geometry, not the text-flavor tables.

### Single-format consumers reconciled (28.8)

The shared-renderer change means `SNAPSHOT MARKDOWN` single-file output now also carries the box body, so a key that previously appeared once in the Markdown flavor now appears twice. The two `tests_config_snapshot_fidelity` assertions were updated accordingly: the plain-text flavor still asserts exactly once (it is the box-only body, unchanged), and the Markdown flavor now asserts exactly twice (box body plus its table row). The per-group structural assertions (one `### [Group]` heading per namespace, one table per section) are retained, so the former duplicate-TABLE defect this suite guarded against is still guarded. This is the right reconciliation: the doubled key count is the intended symmetric behavior, not the old defect.

### Framework and constraint conformance

No public or framework type is reshaped. The only signature change is the internal `build_cf_html` `tables_html` parameter with its single `ff-desktop` caller, exactly as the task permits. `render_tables_html` is an additive `pub` function re-exported from `ff-screen-model`, not a reshape of an existing type. Non-test line counts stay under 400 for all four touched source files (clipboard_html ~201, commands_snapshot ~375, html ~221, markdown ~131). ASCII-only holds in the docs and pattern-matched source; the box-drawing glyphs appear only inside rendered snapshot content and test fixtures, which the task explicitly allows. The coder's recorded results report scoped `check`, `clippy -D warnings`, `fmt --check`, and the relevant test suites all green.

</details>

<details>
<summary>File map</summary>

- `crates/ff-screen-model/src/render/markdown.rs` -- hoist fenced-body into `push_fenced_body`; emit box before `markdown_sections` tables; blank-line separator keyed off trailing newline; 3 new tests.
- `crates/ff-screen-model/src/render/html.rs` -- new `render_tables_html` emitter + shared `push_data_table` helper; 3 new tests.
- `crates/ff-screen-model/src/render.rs` / `lib.rs` -- re-export `render_tables_html`.
- `crates/ff-desktop/src/clipboard_html.rs` -- `build_cf_html`/`fragment_html` gain `tables_html` appended after box; 3 new tests; existing callers updated to pass `""`.
- `crates/ff-desktop/src/shell/commands_snapshot.rs` -- compute `tables_html` via `render_tables_html`; pass into `build_cf_html`; new `dual_representation_flavor_tests` module.
- `crates/ff-desktop/src/shell/state_groups.rs` -- `PendingSnapshot` gains `tables_html` field + doc updates.
- `crates/ff-desktop/src/shell/tests_config_snapshot_fidelity.rs` -- reconcile once/twice key-count assertions for plain-text vs Markdown.
- `docs/specs/screen-snapshot-scrm/tasks.md` -- Phase 2F tasks marked complete.

Full diff: `git -C .worktrees/snapshot-clipboard-impl diff main...HEAD`

</details>
