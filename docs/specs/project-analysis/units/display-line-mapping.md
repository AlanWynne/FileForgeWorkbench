# Analysis Record: display-line-mapping

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-display-line-mapping`
- **Spec folder**: `docs/specs/display-line-mapping/`
- **Analysed**: Wave 1, task W1.4 (CR-NR-057/058 re-baseline)
- **Verdict**: COMPLETE (87/87 tasks `[x]`, TCR PASS). NOT a split candidate.
  Resolves PA-CONFLICT-001 (viewport's duplicate `DisplayLineMapper` trait vs the
  canonical `DisplayLineMapping` owned here). One 400-cap refactor
  (contraction_state.rs 614). Zero-log defensible.
- **CR-NR-057/058 impact**: NONE (specs untouched). Re-verified.

---

## 1. Scope summary

`ff-display-line-mapping` maintains the bidirectional doc-line <-> display-line
mapping (hidden lines = 0 display lines; wrapped lines = many). Adapts Scintilla
ContractionState/Partitioning into Rust with O(log n) lookups (Fenwick tree). 10
requirements: doc<->display mapping, line exclusion/hiding, code folding (nested),
word-wrap mapping, O(log n) lookup performance, incremental updates, integration
points + the public `DisplayLineMapping` trait (Req 7.10), large-document 64-bit
indexing, lazy allocation + one-to-one optimisation, dual hiding (ISPF exclude +
code folding coexist).

242 req lines, 10 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 242 lines; 10 reqs | No |
| 3+ distinct responsibilities | one responsibility: the doc<->display mapping data structure (folding/exclude/wrap are facets of the same Contraction_State) | No |
| 2+ crates | one crate | No |
| low-cohesion clusters | high cohesion | No |
| file-size pressure | `contraction_state.rs` 614 non-test over cap | Yes (1) |

Only the file-size criterion. **NOT a split candidate.** Fix is the refactor
(PA-STD-009).

## 3. Consistency / conflict -- PA-CONFLICT-001 resolved

- Canonical trait `DisplayLineMapping` (Send + Sync) is defined HERE in traits.rs
  per Req 7.10, implemented by `ContractionState`. Correctly consumed by
  `ff-line-commands` (exclude/show), `ff-exclude-show-filter` (ExclusionEngine),
  and `ff-desktop` -- the spec's stated consumers.
- **PA-CONFLICT-001 (resolved to a finding)**: `ff-viewport-scrolling` does NOT
  consume `DisplayLineMapping`. It DEFINES its own `DisplayLineMapper` trait
  (display_mapper.rs, W1.2) with an IdentityMapper and never imports
  `ff_display_line_mapping` (grep 0 in viewport). The two traits differ in NAME
  and are NOT bridged. viewport Req 11.1 says the viewport SHALL accept "a
  `DisplayLineMapper` FROM the display-line-mapping crate" -- but this crate
  exposes `DisplayLineMapping`, not `DisplayLineMapper`. So the viewport uses a
  DUPLICATE local abstraction rather than the canonical trait. Owner decision:
  (a) rename/consume the canonical trait in viewport (drop the local one), OR
  (b) provide an explicit adapter impl and document the thin viewport-facing
  trait as intentional. Either way the current state diverges from the spec.
- Public types owned here (ContractionState, DisplayLineMapping trait, DocLine,
  DisplayLine, SubLine, DisplayLineCountChange, ListenerHandle, DisplayMappingError)
  -- sole owner. Added to consistency-matrix.
- Fold-level ownership (Req 10.7): this crate stores ONLY per-line visibility +
  expanded flags; fold levels/nesting/extents belong to syntax-highlighting /
  language-service. Consistent -- confirm clean when syntax-highlighting is
  analysed (W1.10).
- Change notifications (Req 7.9) via ListenerHandle -- consumed by viewport /
  scrollbar; consistent with viewport Req 4/11.

## 4. Completeness

- Tasks: 87 `[x]`, 0 `[ ]`. TCR PASS (logical-to-visual mapping, wrap mode);
  extensive test suite (folding/visibility/wrap/incremental/one-to-one/large-doc/
  property tests). No std::fs/tokio::fs (pure data-structure crate).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` NOT a dependency. DEFENSIBLE: pure
  data-structure/algorithm crate (Partitioning / Fenwick tree, O(log n) lookups);
  invalid inputs return `false` or clamp (Req 2.7, 4.7). No requirement mandates a
  log record. Same class as viewport / caret-and-selection / encoding.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (correctly zero-log for a pure data-structure
  crate)**.

## 6. Findings logged

- **PA-CONFLICT-001** (CONSISTENCY -- cross-unit; resolved to a finding here):
  viewport-and-scrolling defines a local `DisplayLineMapper` trait instead of
  consuming the canonical `DisplayLineMapping` from ff-display-line-mapping,
  contrary to viewport Req 11.1. Not bridged. Owner decision: consume the
  canonical trait (drop the duplicate) OR document the thin viewport-facing trait
  + provide an adapter. Doc + possible small code change; surface for owner.
- **PA-STD-009** (REFACTOR -- 400-line cap): `contraction_state.rs` is 614
  non-test lines. Split by concern (contraction_visibility / contraction_fold /
  contraction_wrap; keep contraction_state.rs core + trait impl). REFACTOR, no
  gate.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-display-line-mapping
  contributes matches (doc-comment fold-indicator glyphs + box-drawing separators).
  Rolled into project-wide PA-LOG-001.
