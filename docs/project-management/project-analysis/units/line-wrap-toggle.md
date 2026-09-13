# Analysis Record: line-wrap-toggle (W1.14)

- **Wave**: 1 (Editor Core)
- **Backing crate**: `ff-wrap` (spec title says `ff-line-wrap-toggle`; actual
  crate dir is `ff-wrap` -- naming drift)
- **Spec files**: requirements.md (290 lines, 13 requirements), tasks.md
  (189 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 1 re-baseline pass (post CR-NR-057 reset)

---

## 1. Split candidacy

NOT a split candidate.

Assessment against the 2-of-4 rule:

- Requirement volume: 13 requirements, 290 lines. Marginally above the 12-req
  line but the reqs are cohesive (wrap mode + WRAP command + boundary + indent +
  markers + display-line integration + scrollbar + status/menu + persistence +
  config + rendering -- all one display property). (1 weak signal)
- Responsibilities: ONE concern (per-document line-wrap display state). The
  status-bar/menu/scrollbar/session facets are integration points, not separable
  responsibilities.
- Crates: single crate; deps are only thiserror + serde + serde_json (pure
  data/state model, all cross-refs caller-injected -- see below).
- Cohesion: high.

At most 1 weak signal. No split. Already split into 13 focused files.

### Source file sizes -- ALL WITHIN CAP

Largest non-test file `layout.rs` at 294 non-test lines; `commands.rs` 212,
`config.rs` 183. None approaches the 400 cap. No PA-STD size item.

---

## 2. Cross-unit consistency

### PA-CONFLICT-004 (NEW) -- duplicate WrapIndentMode + wrap visual flags

line-wrap-toggle (Req 5 Wrap Indent, Req 10 Wrap Visual Flags) and
whitespace-and-guides (W1.12: Req 7 Wrap Indentation, Req 6 Wrap Visual Markers)
BOTH define the SAME Scintilla concepts, and BOTH crates declare their own types:

- `WrapIndentMode` (Fixed/Same/Indent/DeepIndent):
  - `ff-wrap::indent.rs:16` `pub enum WrapIndentMode`
  - `ff-whitespace-guides::wrap_indent_mode.rs:10` `pub enum WrapIndentMode`
- Wrap visual flags (None/End/Start/Margin):
  - `ff-wrap::visual_flags.rs:13` `pub enum WrapVisualFlags` (enum)
  - `ff-whitespace-guides::wrap_visual_flag.rs:11` `pub struct WrapVisualFlag(u8)`
    (bitfield) -- DIFFERENT SHAPE (enum vs bitfield newtype)

Neither crate imports the other (ff-wrap deps = thiserror/serde only; no
ff-whitespace-guides dep). So these are TWO UNBRIDGED parallel definitions of the
same domain concept -- the same anti-pattern class as PA-CONFLICT-001 (viewport
duplicate DisplayLineMapper) and PA-CONFLICT-003 (find-and-replace duplicate
CaseFolder). The wrap-flags divergence (enum vs bitfield) is worse: they are not
even structurally compatible, so a shared render path would have to convert.

Ownership question to resolve (owner-gated): whitespace-and-guides Req 6-7 frame
wrap markers + wrap indent as ITS concern (2 of its 4 named concerns), and Req
10.7 of line-wrap-toggle says the flags are "rendered using the
whitespace-and-guides rendering infrastructure". That points to
whitespace-and-guides as the SINGLE OWNER of the wrap-indent-mode + wrap-visual-flag
VALUE TYPES, with line-wrap-toggle owning only the wrap MODE state
(None/Word/Character) and the WRAP command. Recorded PA-CONFLICT-004: pick one
owner for `WrapIndentMode` + wrap-visual-flag type (recommend
whitespace-and-guides), have the other consume it; reconcile the enum-vs-bitfield
shape. Code change + owner decision.

### PA-CONFLICT-001 relevance -- display-line-mapping consumed correctly

Req 6 drives display-line-mapping via `set_height(doc_line, height)`. ff-wrap has
NO dep on ff-display-line-mapping; the height computation output is applied by the
caller/shell. So line-wrap-toggle does NOT duplicate the DisplayLineMapping trait
(unlike viewport, PA-CONFLICT-001) -- it stays a pure state model and the caller
bridges to display-line-mapping. CLEAN on that axis.

### Wrap-inactive -> no-markers gate (W1.12 PA-WATCH) -- CONFIRMED consistent

W1.12 raised a WATCH: whitespace-guides `compute_wrap_markers` returns None when
`sub_line_count <= 1`, and Req 6.9 says no markers when wrap is inactive. Here
line-wrap-toggle Req 10.5 ("no wrap visual flags -> no markers") + Req 6.2 (wrap
None -> set_height 1 for every line, so sub_line_count collapses to 1). The two
specs agree: when Wrap_Mode is None, every line has height 1, so
whitespace-guides' `sub_line_count <= 1 -> None` guard fires and no markers
render. The gate is consistent across both specs. W1.12 PA-WATCH RESOLVED
(pending the PA-CONFLICT-004 type unification, which does not change the gate
logic).

### Public types and ownership

- `WrapMode` (None/Word/Character), `WrapBoundary` (Viewport/Column(n)),
  per-document wrap state, WRAP command parsing, scrollbar-visibility rule,
  status indicator, persistence record -- sole-owned by `ff-wrap`. No duplication
  of these.
- `WrapIndentMode` + wrap visual flags -- DUPLICATED (PA-CONFLICT-004 above).
- WRAP command (`view.wrap`) registered via command-framework by caller;
  non-undoable, not in history (Req 3.12/3.13) -- consistent with viewport
  non-undoable display commands (W1.2).
- Config `[view.wrap]` keys (default_mode/wrap_column/indent_mode/indent_amount/
  visual_flags) -- read from configuration-system. NB potential key overlap with
  whitespace-guides `editor.wrap_*` keys (W1.12): line-wrap-toggle uses
  `[view.wrap]` table, whitespace-guides uses `editor.wrap_visual_flags` /
  `editor.wrap_indent_mode`. TWO config surfaces for the same settings -- folds
  into PA-CONFLICT-004 (unify the config key ownership too).

### Cross-reference integrity

All 9 declared cross-refs resolve to existing sub-projects. No dangling refs. All
realized by caller injection (ff-wrap has no ff-* deps).

---

## 3. Completeness

Tracking: all 189 sub-tasks `[x]`. Implementation present across all 13 req areas
(mode, command, boundary, indent, display-height layout, scrollbar, indicator,
menu, persistence, config, visual_flags, rendering-support). Tests in-file. No
false-positive test pattern detected. No PA-INCOMPLETE raised.

### TCR gap (PA-TCR-008)

TCR.md has 1 row for `ff-wrap` against 13 reqs / ~80 criteria. Thin coverage
enumeration (like PA-TCR-002/003/007). Recorded PA-TCR-008.

---

## 4. Logging audit

Scan of `crates/ff-wrap/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: ABSENT (deps = thiserror + serde + serde_json only)
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

Here the logging gap is a SPEC-VS-IMPL mismatch: FOUR requirements MANDATE a
"configuration warning" via the logging-subsystem -- Req 4.7 (wrap_column out of
range), Req 5.8 (wrap_indent_amount out of range), Req 11.3 (unrecognized
persisted mode), Req 12.2 (any invalid config value "emit a configuration warning
via the logging-subsystem"). The crate emits NO log and has NO ff-logging
dependency. It DOES return structured warning DATA (config.rs builds warning
messages like "out of range ... using default"), so the values are validated and
a message is produced -- but it is returned to the caller, not logged. This is
the same "return-warning-data, caller-logs" seam as other model crates, EXCEPT
Req 12.2 explicitly says "via the logging-subsystem", implying ff-wrap (or its
immediate caller) should log. Recorded PA-LOG-010 (MEDIUM): either (a) add
ff-logging and emit the mandated config warnings here, or (b) document that the
config-loading CALLER is responsible for logging the returned warning data --
and make that explicit in the spec. Also a dev-logging candidate for WRAP command
state changes (CR-NR-058).

---

## 5. Task revision proposals

- **PA-CONFLICT-004 (owner-gated, HIGH)**: unify `WrapIndentMode` + wrap-visual-flag
  type between ff-wrap and ff-whitespace-guides. Recommend whitespace-and-guides
  as sole owner (its Req 6-7 + line-wrap Req 10.7 "rendered using
  whitespace-and-guides infrastructure" point that way); ff-wrap consumes. Reconcile
  the enum-vs-bitfield shape and the duplicate config keys (`[view.wrap]` vs
  `editor.wrap_*`). Code change + owner decision.
- **PA-LOG-010 (MEDIUM)**: resolve the mandated-config-warning gap (Req 4.7/5.8/
  11.3/12.2): add ff-logging + emit here, OR make caller-logging explicit in the
  spec. Plus dev-logging for WRAP command (CR-NR-058).
- **PA-STD-020 (ASCII, ACTIONABLE -- runtime strings)**: 68 non-ASCII bytes in
  `.rs` (14 non-comment). MANY are em-dashes/en-dashes inside RUNTIME warning +
  error strings in config.rs (lines 99/115/133/147/153/172) and error.rs
  (14/22) -- e.g. "out of range (0-10000) -- using default". Non-ASCII in
  user-facing output is a genuine defect. Replace with `--`/`-`; convert doc-comment
  banners. REFACTOR, no gate.
- **PA-TCR-008**: enumerate per-requirement TCR rows (1 row for 13 reqs). No code.
- **PA-DOC (naming)**: spec calls the crate `ff-line-wrap-toggle`; actual crate is
  `ff-wrap`. Align spec text or record alias. Doc-only.

No requirement CHANGE proposed for line-wrap-toggle itself; but PA-CONFLICT-004
implies a design.md note in whichever crate becomes the type owner.

---

## Summary

line-wrap-toggle (`ff-wrap`) is a clean, cohesive per-document wrap-state model
with no coupling deps (all cross-refs caller-injected) and no size violation. The
significant finding is PA-CONFLICT-004 (NEW): `WrapIndentMode` and wrap-visual-flag
types are DUPLICATED across ff-wrap and ff-whitespace-guides, unbridged, and the
visual-flag shapes even diverge (enum vs bitfield) -- same anti-pattern as
PA-CONFLICT-001/003; recommend whitespace-and-guides as sole owner. The W1.12
wrap-inactive -> no-markers gate is CONFIRMED consistent (Wrap None -> height 1 ->
whitespace-guides' sub_line_count<=1 guard). Other items: a mandated-config-warning
logging gap (PA-LOG-010, MEDIUM -- 4 reqs say "via the logging-subsystem" but the
crate has no ff-logging dep), an ASCII violation including runtime warning/error
strings (PA-STD-020), a thin TCR (PA-TCR-008), and a crate-name doc drift.
