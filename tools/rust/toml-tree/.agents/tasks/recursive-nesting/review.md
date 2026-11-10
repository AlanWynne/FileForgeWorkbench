# Recursive nesting for toml-tree

The toml-tree CLI previously rendered only two levels: each top-level `[Section]` as a branch and its keys as leaves, collapsing any nested sub-table or array-of-tables inline via `render_value` as `{ ... }` / `[ ... ]`. This change replaces the flat Section/Leaf model with a recursive `Node` tree (`Leaf | Branch`) so nested sub-tables expand into their own indented child branches and arrays-of-tables expand into indexed element nodes (`[0]`, `[1]`, ...). Scalars and scalar arrays still render inline on the right. Rendering is a two-pass traversal: build `Line` rows (connector decisions made once), then align the colon column to the widest left label so the value/count column stays aligned across every depth. The single-file CLI, usage message, error paths, and exit codes are untouched, and the crate stays out of the parent cargo workspace.

Watch for: a global colon-alignment column means a deeply nested section widens the value column of the flat sections in the same file (confirmed; this is the intended "aligned across the whole tree" behaviour, but it does shift flat sections' column position in a mixed file). Flat-only files remain byte-for-byte identical (confirmed by test).

**Verdict**: APPROVED

## High-level view

The data model moved from a two-level structure to a genuinely recursive one. `collect_nodes` walks the top-level table; `expands()` is the single decision point that classifies a value as a branch (a table, or a non-empty array whose every element is a table) versus an inline leaf (scalars, scalar arrays, empty arrays). `node_for` recurses, labelling sub-tables `[key]` and array-of-tables elements `[0]`, `[1]`. This cleanly satisfies the nested-sub-table, array-of-tables, and scalar-stays-inline requirements, and the array-of-tables labelling choice is documented in both the README and the commit.

Rendering keeps two distinct layouts on purpose. A top-level section whose children are all leaves is drawn by `draw_legacy_leaves`, which reproduces the original quirky two-level glyphs byte-for-byte (first leaf joins with an L-corner+tee, later leaves hang beneath). A section containing real nesting is drawn by `draw_children`, a clean recursive layout with tee vs L-corner connectors and vertical-pipe continuation under ancestors that still have following siblings. This is the mechanism that keeps existing flat output unchanged while letting nesting expand.

Alignment is computed once over the whole tree: the colon column is the widest `Row` left label plus one space. That guarantees the right-hand column lines up across all depths, and it is why, in a file that mixes flat and deeply-nested sections, the flat sections' value column widens to match the deepest label. The task calls for alignment across the whole tree, so this is correct, but it is the one way a flat section's rendered column position is not literally unchanged in a mixed file.

The fixture and docs match the behaviour: `sample.toml` now carries `[Server]`, `[Server.http]`, `[Server.http.limits]`, and two `[[Server.routes]]` elements; the README gains a recursive-nesting section, an expanded ASCII sketch, and keeps the honest Unicode-vs-ASCII approximation note. Prose is ASCII-first.

<details>
<summary>Issues (2)</summary>

1. **Global alignment shifts flat columns in mixed files** (possible) -- a deeply nested section widens the shared colon column, so flat sections in the same file render with a wider value column than they would alone. This matches the "aligned across the whole tree" requirement and the byte-for-byte guarantee only strictly holds for flat-only files; confirm this is the intended reading of "unchanged except where real nesting now expands". No code change required unless the intent was per-section alignment.
2. **No test for the empty-array / mixed-array edge** (possible) -- `expands()` correctly keeps an empty array and a mixed scalar/table array inline, but no test pins that behaviour; a future change to `expands()` could regress it silently. Optional hardening.

</details>

<details>
<summary>Details</summary>

## The `expands()` decision point

All branch-vs-leaf classification funnels through one predicate:

```rust
fn expands(v: &Value) -> bool {
    match v {
        Value::Table(_) => true,
        Value::Array(a) => a.iter().all(|e| e.is_table()) && !a.is_empty(),
        _ => false,
    }
}
```

Tables always expand; an array expands only when it is non-empty and every element is a table. This is the right boundary for the three value-shape requirements at once: nested sub-tables expand, arrays-of-tables expand, and scalar arrays, empty arrays, and mixed arrays stay inline through `render_value`. Centralising the decision means `collect_nodes` and `node_for` share identical logic, so top-level and nested classification cannot drift apart. The empty-array and mixed-array fall-through to inline is correct but untested, so a later edit to this predicate could regress it without a failing test.

## Two layouts: legacy flat vs recursive nested

`draw_section` chooses the layout per top-level section:

```rust
if children.iter().all(|c| matches!(c, Node::Leaf { .. })) {
    draw_legacy_leaves(children, gutter, lines);
} else {
    draw_children(children, gutter, lines);
}
```

`draw_legacy_leaves` reproduces the original connector quirks exactly: a lone leaf gets an L-corner, the first of several gets an L-corner+tee, and later leaves hang beneath with a plain corner or tee. The `flat_sections_render_unchanged` test asserts the full string byte-for-byte, which is the strongest possible guard on the "existing flat output unchanged" requirement. `draw_children` is the clean recursive path: tee for a child with a following sibling, L-corner for the last child, and the gutter extended with a vertical pipe (sibling follows) or three spaces (last child) as it descends. The section header glyph retains the down-tee for both layouts while nested children use a plain tee/corner; this minor glyph difference is intentional and matches the README sketch.

## Colon alignment across depth

```rust
let label_width = lines.iter()
    .filter_map(|l| match l { Line::Row { left, .. } => Some(left.chars().count()), Line::Raw(_) => None })
    .max().unwrap_or(0) + 1;
```

Width is taken over every `Row` in the whole tree and uses `chars().count()`, so multi-byte box-drawing glyphs in the left label are measured by column, not byte. `pad_to` likewise counts chars. The `deep_nesting_draws_correct_connectors` test verifies both the connector glyphs at depth and that every data line's colon sits at the same column. The one consequence worth flagging: because the column is global, a deep section sets the width for the flat sections too, so a flat section's value column in a mixed file is wider than when that section is rendered alone. That is the intended "aligned across the whole tree" behaviour, not a bug, but it is the sense in which flat output is not literally byte-identical inside a mixed file.

## Fixture and docs

`sample.toml` exercises every new path: a scalar leaf under `[Server]`, a one-level sub-table `[Server.http]`, a two-level sub-table `[Server.http.limits]`, and a two-element array-of-tables `[[Server.routes]]`. The README's recursive-nesting section documents the sub-table-as-`[key]` and array-of-tables-as-indexed-`[n]` choices, keeps the scalar-array-inline rule explicit, and retains the "real output uses Unicode box-drawing glyphs; the ASCII sketch is an approximation" note. Source uses `\u{...}` escapes for glyphs so both `main.rs` and `README.md` stay ASCII, consistent with the recorded ASCII scan.

## CLI surface and workspace isolation

`main` is unchanged in shape: one positional path argument, `usage: toml-tree <path-to-toml>` to stderr with exit code 2 on missing argument, exit code 1 on read or parse failure, success otherwise. `Cargo.toml` still carries an empty `[workspace]` table, keeping the crate out of the FileForge Workbench build. The recursive-nesting commit touched only `src/main.rs`, `sample.toml`, and `README.md`; the uncommitted `diff HEAD` contains only unrelated spec-doc edits outside this crate.

## Recorded verification

The commit message records a manifest-scoped run: `cargo fmt -- --check` clean, `cargo clippy -- -D warnings` clean, `cargo check` ok, `cargo test` 5 passed / 0 failed, an ASCII scan with no non-ASCII, and the rendered `sample.toml` output. The five tests map directly onto the task's TDD requirements (flat unchanged, sub-table expands, array-of-tables indexed, deep connectors + alignment, scalar array inline), each carrying a `Validates:` comment, and the commit states red-first was confirmed via a missing-`collect_nodes` compile error. Per instructions the suites were not re-run here.

</details>

<details>
<summary>File map</summary>

- `src/main.rs` -- recursive `Node` tree replacing flat Section/Leaf; `collect_nodes`/`node_for`/`expands` build it; `render` two-pass with global colon alignment; `draw_section` splits legacy vs recursive layout; 5 TDD tests added.
- `sample.toml` -- adds `[Server]`, `[Server.http]`, `[Server.http.limits]`, two `[[Server.routes]]` elements to exercise nesting.
- `README.md` -- recursive-nesting section, expanded ASCII sketch, Unicode-approximation note retained.

Full diff: `git -C tools/rust/toml-tree show a2f10fd`

</details>
