# Analysis Record: exclude-show-filter (W1.11)

- **Wave**: 1 (Editor Core)
- **Backing crate**: `ff-exclude-show-filter` (NOT `ff-filter` -- earlier summary
  note corrected)
- **Spec files**: requirements.md (245 lines, 10 requirements), tasks.md
  (120 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 1 re-baseline pass (post CR-NR-057 reset)

---

## 1. Split candidacy

NOT a split candidate. Assessment against the 2-of-4 rule:

- Requirement volume: 10 requirements, 245 lines. Below both thresholds (12 reqs
  / ~350 lines). No signal.
- Responsibilities: ONE coherent concern -- logical line-visibility (exclusion)
  state and the EXCLUDE/SHOW/RESET/X commands that drive it. Placeholder-text
  generation and find/change scope iterators are facets of the same model.
- Crates: single crate; correctly consumes `ff-display-line-mapping` for storage
  (see below). No embedded second crate.
- Cohesion: high; all reqs revolve around the exclusion boolean layer.

Zero of 4 signals. No split.

### Source file size violation (PA-STD-016) -- ACTIONABLE

Non-test line counts (400-line cap is NON-TEST lines only):

| File | non-test | status |
|------|----------|--------|
| `exclusion_engine.rs` | 535 | **EXCEEDS 400 -- must split** |
| `registration.rs` | 287 | ok (WATCH) |
| `types.rs` | 275 | ok |
| `text_matcher.rs` | <=120 | ok |
| `error.rs`, `lib.rs` | small | ok |

`exclusion_engine.rs` at 535 non-test lines is well over the hard cap. Split by
concern: e.g. command execution (EXCLUDE/SHOW/RESET dispatch) vs query/iterators
(is_excluded / visible_lines_iter / excluded_lines_iter) vs
block-enumeration/placeholder generation vs line-command (X/Xn/XX) resolution.
REFACTOR (no gate; no behaviour change). Recorded PA-STD-016.

---

## 2. Cross-unit consistency

### SHOW-restore / visibility-storage boundary -- CONFIRMED CLEAN (W1.4/W1.6)

The open note from W1.4/W1.6 ("exclude-show-filter authoritative for SHOW
restore") is confirmed and refined:

- exclude-show-filter OWNS the LOGICAL exclusion layer: EXCLUDE/SHOW/RESET primary
  commands, X/Xn/XX line commands, exclusion-block enumeration, placeholder text,
  and find/change scope iterators (Req 2-9).
- It does NOT store visibility itself. `exclusion_engine.rs` imports
  `ff_display_line_mapping::{DisplayLineMapping, DocLine}` and drives all state
  through the CANONICAL trait: `set_visible` / `get_visible` / `hidden_lines` /
  `show_all` (Req 1.1-1.5, 4.4). display-line-mapping owns the storage
  (ContractionState), exclude-show-filter owns the command semantics.

Critically, exclude-show-filter consumes the SAME canonical `DisplayLineMapping`
trait that PA-CONFLICT-001 flags viewport for DUPLICATING. exclude-show-filter
does it RIGHT -- it is a positive reference implementation of how consumers should
use the canonical trait. This strengthens PA-CONFLICT-001 (viewport is the outlier;
exclude-show-filter and line-commands both consume the canonical trait correctly).

Boundary: CLEAN, single-owner-per-layer. No conflict.

### Exclusion vs code-folding coexistence

Both exclusion (flat) and folding (nested levels) write visibility into the SAME
display-line-mapping layer (Req: "exclusion state coexists with code-folding
visibility"). Flat exclusion has no fold levels; folding (syntax-highlighting
computes levels, W1.10) uses them. They share the visibility bit but are distinct
mechanisms. No ownership conflict -- display-line-mapping arbitrates the combined
visible/hidden result. Recorded as a WATCH to confirm no double-toggle race when
a line is both folded and excluded (Wave 4 shell/viewport integration).

### Public types and ownership

- `ExclusionEngine`, exclusion-block/placeholder types, scope iterators, error
  type -- sole-owned by `ff-exclude-show-filter`. No duplication.
- EXCLUDE/X, SHOW/INCLUDE, RESET primary commands + X/Xn/XX line commands
  registered via command-framework (`registration.rs`), consumed by
  command-semantics (matrix) and line-commands (X/Xn/XX resolution). Consistent
  with command ownership.
- FIND/CHANGE EXCLUDED|VISIBLE scope (Req 8): exclude-show-filter EXPOSES
  `visible_lines_iter` / `excluded_lines_iter`; find-and-replace CONSUMES them.
  Consistent single-owner-many-readers -- matches the W1.8 find-and-replace record.

### Cross-reference integrity

All 7 declared cross-refs (display-line-mapping, command-semantics, line-commands,
find-and-replace, document-model, viewport-and-scrolling, navigation-commands)
resolve to existing sub-projects. No dangling references.

---

## 3. Completeness

Tracking: all 120 sub-tasks `[x]`. Implementation present: `exclusion_engine.rs`
(the core), `text_matcher.rs` (literal/regex line matching for EXCLUDE/SHOW),
`registration.rs` (command-framework metadata + registration), `types.rs`,
`error.rs`. Tests are in-file (`#[cfg(test)]`). Implementation matches reqs; no
false-positive test pattern (contrast background-io PA-INCOMPLETE-002). No
PA-INCOMPLETE raised.

### TCR gap (PA-TCR-005) -- TOTAL ABSENCE

TCR.md has ZERO rows for `ff-exclude-show-filter` (grep = 0). All 10 requirements
/ ~70 criteria are untracked in TCR despite in-file tests existing. This is the
most complete TCR absence seen so far in Wave 1 (find-and-replace had 1,
syntax-highlighting had Req 16 only; this has none). RECORDING gap, not a test
gap. Recorded PA-TCR-005: add per-requirement TCR rows citing the existing tests.

---

## 4. Logging audit

Scan of `crates/ff-exclude-show-filter/src` (recursive):

- `ff_logging` / `log_*!` macro CALLS: 0
- `ff_logging` import/use: 0 -- BUT `ff-logging` IS declared as a Cargo dependency
  (Cargo.toml line 12). So the crate depends on ff-logging and never uses it: a
  DEAD DEPENDENCY.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0 (pure logical layer)

The many "status message" requirements (Req 2.8/2.9, 3.7/3.8, 4.7, 5.7 -- e.g.
"N line(s) excluded") are STATUS-AREA reports, not log records; they are returned
to the caller, not logged. Zero-log is otherwise defensible for this pure model.

However this crate is a strong CR-NR-058 dev-logging candidate: EXCLUDE/SHOW/RESET
are exactly the kind of session-state commands whose start/params/result would
help testing/debugging the filter workflows (EXCLUDE ALL + FIND ALL + SHOW). The
dead ff-logging dep signals the intent was there. Recorded PA-LOG-008
(LOW/dev-logging): either drop the unused dep OR (preferred) wire dev-logging
command instrumentation for the three commands under the `dev-logging` gate once
logging Req 13 (PA-CR058) lands.

---

## 5. Task revision proposals

- **PA-STD-016 (REFACTOR, ACTIONABLE)**: split `exclusion_engine.rs` (535 non-test
  lines, over the 400 cap). Split by concern (command dispatch / query+iterators /
  block+placeholder / X-XX line-command resolution). No gate, no behaviour change.
- **PA-STD-017 (ASCII, ACTIONABLE)**: 67 non-ASCII bytes in `.rs`. Two classes:
  (a) box-drawing section banners + em/en dashes in doc comments (prohibited in
  `.rs`); (b) NOTABLE -- `error.rs` line 18 puts an en-dash (U+2013) inside a
  RUNTIME error string: `"invalid line range {start}-{end}"`. A non-ASCII char in
  user-facing output is a genuine defect, not just a comment-style issue. Replace
  all with ASCII `-`/`--` and convert banners to `// === ... ===`. REFACTOR.
- **PA-TCR-005**: add per-requirement TCR rows (TCR currently has zero rows for
  this crate). No code.
- **PA-LOG-008**: drop the dead `ff-logging` dep OR wire dev-logging command
  instrumentation (preferred; owner/logging-Req-13 gated).
- **PA-WATCH-010**: confirm exclusion + folding do not double-toggle the shared
  display-line-mapping visibility bit (Wave 4 viewport integration).

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

exclude-show-filter is a clean, single-responsibility logical visibility layer.
The W1.4/W1.6 open note is CONFIRMED: it owns EXCLUDE/SHOW/RESET semantics and
correctly delegates visibility storage to the CANONICAL `DisplayLineMapping`
trait -- making it a positive counter-example to PA-CONFLICT-001 (viewport's
duplicate). Not a split candidate. Actionable items: `exclusion_engine.rs`
exceeds the 400 cap (PA-STD-016), an ASCII violation that includes a non-ASCII
char in a runtime error string (PA-STD-017), total TCR absence (PA-TCR-005), and
a dead ff-logging dependency that should become dev-logging instrumentation
(PA-LOG-008). One coexistence WATCH (exclusion+folding, PA-WATCH-010).
