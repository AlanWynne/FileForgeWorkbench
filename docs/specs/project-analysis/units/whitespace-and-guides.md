# Analysis Record: whitespace-and-guides (W1.12)

- **Wave**: 1 (Editor Core)
- **Backing crate**: `ff-whitespace-guides`
- **Spec files**: requirements.md (213 lines, 9 requirements), tasks.md
  (100 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 1 re-baseline pass (post CR-NR-057 reset)

---

## 1. Split candidacy

NOT a split candidate; also the BEST-decomposed crate seen in Wave 1 so far.

Assessment against the 2-of-4 rule:

- Requirement volume: 9 requirements, 213 lines. Below both thresholds. No signal.
- Responsibilities: FOUR named visual concerns (whitespace visibility, indent
  guides, edge column, wrap markers) -- but they are all facets of ONE domain
  (non-content visual decorations) sharing a single `WhitespaceSettings`
  aggregate, and the crate already separates them cleanly into submodules
  (modes/ query/ indent/). This is internal decomposition done right, not a
  split trigger. (partial signal at most)
- Crates: single crate; GUI-independent (Req 9 verified: 0 egui/winit/wgpu).
- Cohesion: high; all four concerns feed the same settings model and query API.

At most 1 partial signal. No split.

### Source file sizes -- ALL WITHIN CAP (exemplary)

23 source files across 4 submodules. Largest non-test file is `types.rs` at 127
non-test lines. NO file approaches the 400 cap. This crate is a model for how the
400-line rule + one-concern-per-file should look (contrast PA-STD-014/016 where
single files hit 462/535). No PA-STD size item.

---

## 2. Cross-unit consistency

### display-line-mapping "Consumer" relationship -- CLEAN (parameter-injected)

The cross-ref lists display-line-mapping as providing "wrap height / sub-line
identification". The crate has NO Cargo dependency on ff-display-line-mapping and
0 references to it. This is CORRECT, not a gap: `query/wrap_markers.rs`
`compute_wrap_markers(sub_line_count: u32, ...)` takes the sub-line count as a
PARAMETER. The GUI shell obtains sub-line info from display-line-mapping and
passes it in. The crate stays purely data/GUI-independent (Req 9.4: "given a
document line's content and current settings, return positions of visual
elements"). The "Consumer" relationship is realized at the caller layer, not via
a crate edge. Recorded as a CLEAN consistency row.

### Public types and ownership

- `WhitespaceSettings`, the mode enums (`WhitespaceVisibility`, `TabDrawMode`,
  `IndentGuideMode`, `EdgeMode`, `WrapVisualFlag`, `WrapVisualLocation`,
  `WrapIndentMode`), `WrapMarkerInfo`/`WrapIndentInfo`, `EdgeProperties`, query
  result types -- all sole-owned by `ff-whitespace-guides`. No duplication.
- Config keys `editor.whitespace_mode`, `editor.tab_draw_mode`,
  `editor.whitespace_size`, `editor.indent_guides`, `editor.edge_mode`,
  `editor.edge_column`, `editor.edge_columns`, `editor.edge_colour`,
  `editor.wrap_visual_flags`, `editor.wrap_visual_location`,
  `editor.wrap_indent_mode`, `editor.wrap_start_indent` -- sole owner; consumed
  from configuration-system with hot-reload. Consistent (no key collisions found
  with other Wave 0/1 units).
- Toggle commands (`ToggleWhitespace`, `ToggleIndentGuides`, `ToggleEdgeColumn`)
  registered with command-framework, persist to user config layer (Req 8.4).
  Consistent with command ownership.
- Theme resolution (Req 2.7/3.6/4.4/5/6.8): crate emits NO colours; all visual
  attributes resolved by theme-and-appearance at render time. Clean layering.
- line-wrap-toggle "Related" (Req 6.9: no wrap markers when wrap inactive): the
  wrap-active guard is enforced by `sub_line_count <= 1 -> None` and the caller's
  wrap-mode check. Confirm the wrap-mode gating at line-wrap-toggle (W1.14).

### Cross-reference integrity

All 6 declared cross-refs (theme-and-appearance, configuration-system,
display-line-mapping, document-model, command-framework, line-wrap-toggle)
resolve to existing sub-projects. No dangling references.

---

## 3. Completeness

Tracking: all 100 sub-tasks `[x]`. Implementation present and thorough: mode
enums (8 files under modes/), query engines (whitespace / indent_guides / edge /
wrap_markers under query/), indent scanning (indent/), settings aggregate,
command registration, config keys, theme colour resolution. Tests are in-file
(`#[cfg(test)]`) with `// Validates: Requirement X.Y` annotations (verified in
wrap_markers.rs). Implementation matches reqs; no false-positive pattern. No
PA-INCOMPLETE raised.

### TCR gap (PA-TCR-006) -- TOTAL ABSENCE

TCR.md has ZERO rows for `ff-whitespace-guides` (grep = 0) across 9 reqs / ~60
criteria, despite well-annotated in-file tests. Second total-absence in a row
(exclude-show-filter PA-TCR-005 was the first). RECORDING gap, not test gap.
Recorded PA-TCR-006: add per-requirement TCR rows citing existing tests.

---

## 4. Logging audit

Scan of `crates/ff-whitespace-guides/src` (recursive):

- `ff_logging` / `log_*!` macro CALLS: 0
- `ff_logging` import/use: 0 -- BUT `ff-logging` IS a declared Cargo dependency
  (Cargo.toml:12). DEAD DEPENDENCY (same pattern as exclude-show-filter
  PA-LOG-008).
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0 (pure settings/metadata model)

Zero-log is defensible for this pure data model. The Req 1.6/2.4/3.7/5.8/7.4
hot-reload + the config-error variants in `error.rs` (invalid mode/column ->
"using default") are exactly where a WARN would belong when a bad config value is
coerced -- but no log is emitted. Recorded PA-LOG-009 (LOW/dev-logging): drop the
dead dep OR wire ff-logging to emit a WARN on config-value coercion (the
`error.rs` variants already describe the fallback) plus optional dev-logging on
toggle-command state changes, under the `dev-logging` gate once logging Req 13
(PA-CR058) lands.

---

## 5. Task revision proposals

- **PA-STD-018 (ASCII, ACTIONABLE -- runtime strings)**: 39 non-ASCII bytes in
  `.rs`. Of these, 9 are in NON-comment code, and 7 are em-dashes (U+2014) INSIDE
  runtime `#[error("...")]` strings in `error.rs` (lines 12-59, e.g. "invalid
  whitespace mode '{value}' -- using default 'invisible'"). Non-ASCII in
  user-facing error output is a genuine defect (same class as exclude-show-filter
  PA-STD-017). The remainder are em/en dashes + arrows (the Req 8 "A -> B -> C"
  cycle descriptions) + box-drawing banners in doc comments. Replace all with
  ASCII `--`/`-`/`->`; convert banners to `// === ... ===`. REFACTOR, no gate.
- **PA-TCR-006**: add per-requirement TCR rows (TCR currently zero rows). No code.
- **PA-LOG-009**: drop dead `ff-logging` dep OR wire config-coercion WARN +
  dev-logging toggle instrumentation (owner/logging-Req-13 gated).
- **PA-WATCH (wrap gating)**: confirm the wrap-mode-inactive -> no-markers gate
  (Req 6.9) is enforced by the line-wrap-toggle caller at W1.14. Low priority.

No requirement CHANGE proposed; the spec is internally consistent and complete.
NO size split needed -- the crate is exemplary on the 400-line rule.

---

## Summary

whitespace-and-guides is a clean, exemplary crate: 9 cohesive requirements, a
best-in-Wave-1 module decomposition (23 files, none near the 400 cap), and a
correctly GUI-independent design where the display-line-mapping "Consumer"
relationship is realized by parameter injection (sub_line_count) rather than a
crate dependency -- so no conflict and no dangling dep. Not a split candidate.
Open items are minor: an ASCII violation that (like exclude-show-filter) includes
non-ASCII chars inside runtime error strings (PA-STD-018), a total TCR absence
(PA-TCR-006), and a dead ff-logging dependency best converted to config-coercion
WARN + dev-logging (PA-LOG-009).
