# Analysis Record: display-line-mapping

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-display-line-mapping`
- **Spec folder**: `docs/specs/display-line-mapping/`
- **Analysed**: Wave 1, task W1.4
- **Verdict**: COMPLETE (87/87 tasks `[x]`, TCR PASS). NOT a split candidate.
  One 400-cap refactor (contraction_state.rs 614). PA-WATCH-007 RESOLVED into a
  real consistency finding: viewport defines its OWN `DisplayLineMapper` trait
  instead of consuming the canonical `DisplayLineMapping` (PA-CONFLICT-001).

---

## 1. Scope summary

`ff-display-line-mapping` maintains the bidirectional doc-line <-> display-line
mapping (hidden lines = 0 display lines; wrapped lines = many). Adapts Scintilla
ContractionState/Partitioning into Rust with O(log n) lookups (Fenwick tree).
10 requirements:

- Req 1 doc<->display mapping; Req 2 line exclusion/hiding; Req 3 code folding
  (nested); Req 4 word-wrap mapping (set_height/sub-lines); Req 5 O(log n) lookup
  performance; Req 6 incremental updates (insert/delete lines); Req 7 integration
  points + the public `DisplayLineMapping` trait (Req 7.10); Req 8 large-document
  64-bit indexing; Req 9 lazy allocation + one-to-one optimisation; Req 10 dual
  hiding (ISPF exclude + code folding coexist).

242 req lines, 10 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 242 lines; 10 reqs | No |
| 3+ distinct responsibilities | one responsibility: the doc<->display mapping data structure; folding/exclude/wrap are all facets of the same Contraction_State | No |
| 2+ crates | one crate `ff-display-line-mapping` | No |
| low-cohesion clusters | high cohesion; the trait + ContractionState + partitioning are one unit | No |
| file-size pressure | `contraction_state.rs` 614 non-test over cap | Yes (1) |

Only the file-size criterion. **NOT a split candidate.** Fix is the refactor
(PA-STD-007), not a split.

## 3. Consistency / conflict -- PA-CONFLICT-001 (PA-WATCH-007 resolved)

- Canonical trait `DisplayLineMapping` (Send + Sync) is defined HERE in
  `traits.rs` per Req 7.10, implemented by `ContractionState`. Correctly consumed
  by `ff-line-commands` (exclude/show execution) and `ff-exclude-show-filter`
  (ExclusionEngine<D: DisplayLineMapping>) -- the spec's stated consumers.
- **PA-CONFLICT-001 (real inconsistency, PA-WATCH-007 resolved to a finding)**:
  `ff-viewport-scrolling` does NOT consume `DisplayLineMapping`. Instead it
  DEFINES ITS OWN trait `DisplayLineMapper` (display_mapper.rs) with an
  `IdentityMapper` fallback, and the viewport stores `Option<Box<dyn
  DisplayLineMapper>>`. The two traits:
  - differ in NAME (`DisplayLineMapping` canonical vs `DisplayLineMapper` local);
  - are NOT bridged -- viewport never imports `ff_display_line_mapping`
    (grep: 0 references), and no adapter impls `DisplayLineMapper` for a
    `DisplayLineMapping`.
  viewport-and-scrolling Req 11.1 explicitly says the viewport SHALL accept "a
  `DisplayLineMapper` (FROM the `display-line-mapping` crate)". The crate exposes
  `DisplayLineMapping`, not `DisplayLineMapper`, so the spec intent is unmet: the
  viewport uses a DUPLICATE local abstraction rather than the canonical trait.
  Owner decision: (a) rename/consume the canonical trait in viewport (drop the
  local one), or (b) provide an explicit adapter impl and document the thin
  viewport-facing trait as intentional. Either way the current state diverges
  from the spec. Supersedes PA-WATCH-007.
- Public types owned here (ContractionState, DisplayLineMapping trait, DocLine,
  DisplayLine, SubLine, DisplayLineCountChange, ListenerHandle,
  DisplayMappingError) -- sole owner. Added to matrix.
- Fold-level ownership boundary (Req 10.7): this crate stores ONLY per-line
  visibility + expanded flags; fold levels/nesting/extents belong to
  syntax-highlighting / language-service. Consistent -- watch confirmed clean
  when syntax-highlighting is analysed (W1.10).
- Change notifications (Req 7.9) via ListenerHandle -- consumed by viewport/
  scrollbar; consistent with viewport Req 4/11.

## 4. Completeness

- Tasks: 87 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-display-line-mapping` row PASS (logical-to-visual mapping, wrap mode);
  extensive test suite (folding/visibility/wrap/incremental/one-to-one/large-doc/
  property tests).
- No std::fs/tokio::fs (pure data-structure crate).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` IS a declared dependency (wired, unused).
- Defensible: this is a pure data-structure/algorithm crate (Partitioning /
  Fenwick tree, O(log n) lookups). Invalid inputs return `false` or clamp
  (Req 2.7, 4.7) -- no failure paths that mandate logging; no requirement
  mandates a log record. Same class as viewport / caret-and-selection / encoding.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Logging verdict: **adequate (correctly zero-log for a pure data-structure
  crate)**.

## 6. Findings logged

- **PA-CONFLICT-001** (CONSISTENCY -- cross-unit; supersedes PA-WATCH-007):
  viewport-and-scrolling defines a local `DisplayLineMapper` trait instead of
  consuming the canonical `DisplayLineMapping` from ff-display-line-mapping,
  contrary to viewport Req 11.1 ("from the display-line-mapping crate"). Not
  bridged. Owner decision: consume the canonical trait (drop the duplicate) OR
  document the thin viewport-facing trait + provide an adapter. Doc + possible
  small code change; surface for owner. No gate for the analysis record.
- **PA-STD-007** (REFACTOR -- 400-line cap): `contraction_state.rs` is 614
  non-test lines, over the cap. Split by concern (e.g. `contraction_visibility.rs`,
  `contraction_fold.rs`, `contraction_wrap.rs`; keep contraction_state.rs as the
  ContractionState core + trait impl). REFACTOR, no gate.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-display-line-mapping
  contributes 8 matches (doc-comment fold-indicator glyphs and box-drawing
  separators). Note: the fold indicators in Req 3.10 prose are legitimate; the
  `.rs` matches are separators -- ASCII-ise during the project-wide cleanup.
  Rolled into PA-LOG-001.
