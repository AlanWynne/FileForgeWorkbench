# Analysis Record: text-decorations (W1.15)

- **Wave**: 1 (Editor Core) -- spec labels it a Wave 6 (UI/Rendering) component;
  analysed here as the last Wave 1 editor-core unit
- **Backing crate**: `ff-text-decorations`
- **Spec files**: requirements.md (366 lines, 15 requirements), tasks.md
  (155 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 1 re-baseline pass (post CR-NR-057 reset)

---

## 1. Split candidacy

WATCH -- not a spec split, but one file exceeds the cap.

Assessment against the 2-of-4 rule:

- Requirement volume: 15 requirements, 366 lines. Above the 12-req line. (1 signal)
- Responsibilities: several internal layers -- indicator styles (Req 1-2), RLE
  storage (Req 3-4), search/diagnostic/history/bookmark producers (Req 5-8),
  general line-marker system (Req 9), DPI rendering contract (Req 10), hover
  (Req 11), gutter (Req 12), allocation (Req 13), render pipeline (Req 14), theme
  (Req 15). These are facets of ONE decoration model sharing the RunStyles storage
  and indicator/marker registries. Cohesive; already split into 20 files. (partial)
- Crates: single crate. Deps: ff-logging, ff-command, ff-config (see logging note).
- Cohesion: high (all producers write into the shared Decoration_List).

1 firm + 1 partial signal -- borderline but below a confident 2-of-4 spec split.
No spec split. The internal decomposition is adequate.

### Source file size violation (PA-STD-021) -- ACTIONABLE

`run_styles.rs` = 410 non-test lines, just OVER the 400 hard cap. It is the RLE
core (RunStyles + fill_range/insert_space/delete_range/value_at/start_run/end_run).
Split by concern: e.g. the RunStyles container/mutation vs the range-query/
iteration API. REFACTOR (no gate; no behaviour change). Recorded PA-STD-021.

---

## 2. Cross-unit consistency

### syntax-highlighting peer boundary (`under` property) -- CONFIRMED CLEAN

W1.10 flagged text-decorations as a PEER of syntax-highlighting (independent
storage; the `under` property orders indicators vs syntax colours). Confirmed:

- Storage is fully independent: text-decorations owns `RunStyles` (Req 3) --
  UNIQUE to this crate (grep: no `RunStyles` defined in any other crate). It does
  NOT reuse syntax-highlighting's style buffer, and vice versa. No duplication.
- The `under` property (Req 2.2) + the layer order (Req 14.1: background markers
  -> under-indicators -> text+syntax -> over-indicators -> selection -> margin)
  is the exact composition contract W1.10 described from the syntax side (Req
  15.3-15.5 there). The two specs agree.
- Indicator allocation (Req 13): indicators 0-7 reserved for lexer/syntax-highlighting,
  8-31 container/plugins, 32-35 IME, 36-43 history. syntax-highlighting owns 0-7;
  text-decorations forbids container writes below 8. Clean namespace split -- no
  collision with syntax-highlighting's style slots (which are a SEPARATE u8 space).

No conflict. The W1.10 peer relationship is CONFIRMED clean.

### find-and-replace producer boundary (search highlighting) -- consistent

Req 5: find-and-replace is a PRODUCER that calls `fill_range` on
`INDICATOR_SEARCH_CURRENT` / `INDICATOR_SEARCH_ALL`. This is the storage backing
for find-and-replace's highlight-all (W1.8) and the HILITE FIND mode
(syntax-highlighting Req 16.4 / PA-WATCH-009, W1.10). Single owner
(text-decorations owns the indicator storage), find-and-replace and the HILITE
command are readers/writers via `fill_range`. Consistent. Ties into PA-WATCH-009:
HILITE FIND ultimately toggles these search indicators.

### Public types and ownership

- `IndicatorStyle` (23 variants), `Indicator` props, `Decoration`/`DecorationList`,
  `RunStyles`, `LineMarker`/`MarkerSymbol`/`MarkerMask`, `DecorationRenderer`
  (trait), indicator/marker number constants -- all sole-owned by
  `ff-text-decorations`. No duplication found (RunStyles verified unique).
- Bookmark commands (Req 8.9: toggle/next/previous/clear-all) registered via
  command-framework (ff-command dep present, 2 refs). Consistent with command
  ownership.
- Change-history markers (Req 7) + modified-line gutter (Req 12) driven by the
  same data source (Req 12.7) -- internally consistent; the undo/redo edit-sync
  (Req 4.5-4.6) is caller-driven via insert_space/delete_range.
- Theme `[decorations]`/`[indicators]` section (Req 15): sole reader; theme owns
  the values. Consistent with theme ownership.
- Rendering: Req 14.5/14.6 keep drawing OUT of this crate -- it exposes a
  `DecorationRenderer` trait and data; the viewport draws. GUI-independent model.

### Cross-reference integrity

All 8 declared cross-refs (find-and-replace, theme-and-appearance,
document-model, undo-redo-transactions, display-line-mapping, syntax-highlighting,
viewport-and-scrolling, configuration-system) resolve to existing sub-projects.
No dangling references.

---

## 3. Completeness

Tracking: all 155 sub-tasks `[x]`. Implementation present across all 15 req areas
(20 source files: indicator styles, RunStyles RLE, decoration list, edit-sync,
line markers, bookmark/history/search producers, allocation registry, render
contract, theme integration). Tests in-file. No false-positive pattern detected.
No PA-INCOMPLETE raised for functional behaviour.

### TCR gap (PA-TCR-009) -- TOTAL ABSENCE

TCR.md has ZERO rows for `ff-text-decorations` (grep = 0) across 15 reqs /
~130 criteria, despite in-file tests. THIRD total-absence in Wave 1
(exclude-show-filter PA-TCR-005, whitespace-guides PA-TCR-006, now this).
RECORDING gap, not test gap. Recorded PA-TCR-009.

---

## 4. Logging audit

Scan of `crates/ff-text-decorations/src` (recursive):

- `ff_logging` / `log_*!` macro CALLS: 0
- `ff-logging` Cargo dep: PRESENT (Cargo.toml:10) -- but 0 uses. DEAD (same
  pattern as PA-LOG-008/009).
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

Req 15.8 MANDATES logging: invalid theme values (alpha/stroke_width/style out of
range) "fall back to defaults, logging a warning". The validation + fallback
appear present but NO log is emitted -- so the mandated WARN is unimplemented and
ff-logging sits unused. This is the same class as line-wrap PA-LOG-010 (mandated
warning, no active logging) combined with the dead-dep pattern. Recorded
PA-LOG-011 (LOW-MED): wire ff-logging to emit the Req 15.8 WARN on invalid theme
values (drop the dep only if that path is intentionally caller-logged); plus
dev-logging is a fit for decoration edit-sync / allocation-exhaustion trace under
the `dev-logging` gate (CR-NR-058).

---

## 5. Task revision proposals

- **PA-STD-021 (REFACTOR, ACTIONABLE)**: split `run_styles.rs` (410 non-test
  lines, over the 400 cap) into container/mutation vs range-query API. No gate,
  no behaviour change.
- **PA-STD-022 (ASCII, ACTIONABLE -- runtime strings)**: 71 non-ASCII bytes in
  `.rs` (3 non-comment). `error.rs:26,30` have en-dashes inside runtime `#[error]`
  strings ("lexer range (0-7)", "numbers (8-31)"). Non-ASCII in user-facing output
  is a genuine defect. Remainder: doc-comment em/en dashes + box-drawing banners +
  a `->` in a test comment (run_styles.rs:467). Replace with `--`/`-`/`->`; convert
  banners. REFACTOR, no gate.
- **PA-TCR-009**: enumerate per-requirement TCR rows (0 rows for 15 reqs today).
  No code.
- **PA-LOG-011**: implement the Req 15.8 mandated WARN (invalid theme value) via
  ff-logging; add dev-logging edit-sync/allocation trace under the `dev-logging`
  gate (owner/logging-Req-13 aware).

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

text-decorations is a well-structured decoration model that CONFIRMS the W1.10
peer boundary with syntax-highlighting: fully independent RunStyles storage
(unique to this crate), a matching layer-order/`under` composition contract, and
a clean indicator-number namespace split (0-7 lexer/syntax, 8-31 container,
32-43 IME/history) with no collision against syntax slot indices. It is also the
storage backing for find-and-replace search highlighting + HILITE FIND
(PA-WATCH-009). Not a spec split candidate. Actionable items: `run_styles.rs`
exceeds the 400 cap (PA-STD-021), an ASCII violation incl. runtime error strings
(PA-STD-022), total TCR absence (PA-TCR-009, third in Wave 1), and a dead
ff-logging dep whose Req 15.8 mandated theme-validation WARN is unimplemented
(PA-LOG-011).
