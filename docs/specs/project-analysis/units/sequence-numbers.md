# Analysis Record: sequence-numbers (W1.9)

- **Wave**: 1 (Editor Core)
- **Backing crate**: `ff-seqnum` (spec title says `ff-sequence-numbers`; actual
  crate dir is `ff-seqnum` -- naming drift, see PA-DOC below)
- **Spec files**: requirements.md (434 lines, 14 requirements), tasks.md
  (22 top-level tasks, all `[x]`), design.md present
- **Analysed**: Wave 1 re-baseline pass (post CR-NR-057 reset)

---

## 1. Split candidacy

NOT a split candidate. Assessment against the 2-of-4 rule:

- Requirement volume: 14 requirements, 434 spec lines. Above the 12-req line but
  the reqs are tightly cohesive (all sequence-number lifecycle: config ->
  detect -> strip -> number -> show -> save). Single count only.
- Responsibilities: ONE coherent domain (legacy sequence-number handling).
  Detection, strip, number, show, save are stages of one pipeline, not separable
  concerns. Does not meet 3+ responsibilities.
- Crates: single crate `ff-seqnum`; all upstream deps
  (ff-document-model, ff-language-service, ff-command, ff-undo, ff-config) are
  reached via traits in `traits.rs` -- no second crate embedded.
- Cohesion: high. The side-table (state.rs) ties detection, strip, show, and
  save-restore together; splitting would fragment that shared state.

Only 1 of 4 signals (req volume) -- below the 2-of-4 threshold. No split.

### Source file size watch (PA-STD-012)

Non-test line counts (largest first; 400-line cap is NON-TEST lines only):

| File | non-test | note |
|------|----------|------|
| `number_cmd.rs` | 355 | largest; approaching cap -- WATCH |
| `detector.rs` | 221 | ok |
| `state.rs` | 209 | ok |
| `types.rs` | 193 | ok |
| `strip.rs` | 178 | ok |
| `unnum.rs` | 149 | ok |
| `number.rs` | 140 | ok |
| all others | <=123 | ok |

No file currently EXCEEDS 400 non-test lines, so no split is forced. `number_cmd.rs`
at 355 is the one to watch: if Req 6/7 NUMBER parsing grows (e.g. more FORMAT
variants), split argument-parsing from execution. Logged as PA-STD-012 (WATCH,
not yet actionable).

---

## 2. Cross-unit consistency

Public types and ownership:

- `ColumnRange`, `SequenceFormat`, `DetectionResult`, `SeqNumState`,
  `SideTableEntry`, `SeqNumConfig`, `AutoStripResult`, `SeqNumIndicator`,
  `NumberEngine`/`NumberResult`, `OverlayEntry` -- all sole-owned by `ff-seqnum`.
  No duplication found against other Wave 0/1 units.
- Command IDs `sequence.unnum`, `sequence.number`, `sequence.number_show`
  (+ aliases `NUM`, `AUTONUM`) -- sole owner; registered via `CommandRegistry`
  trait from ff-command. Consistent with command-framework ownership.
- Config keys `editor.sequence_numbers.*` (detection_threshold, sample_size,
  highlight_columns, default_format, restore_on_save + per-language overrides) --
  sole owner; consumed from configuration-system layered model. Consistent.

### PA-CONFLICT-002 relevance (undo transaction model) -- CLEAN SEAM

`undo_integration.rs` does NOT use the concrete undo-redo `Transaction` type NOR
edit-operations' `EditorTransaction`. It defines its own `ColumnChange` struct
and drives an `UndoRecorder` **trait** (`begin_sequence_transaction` /
`record_column_change` / `commit` / `abort`). The concrete transaction type is
injected at runtime by whoever wires the recorder.

Consequence: sequence-numbers is NOT a party to the EditorTransaction-vs-Transaction
conflict (PA-CONFLICT-002). It is decoupled by design and will bind to whichever
model the shell selects when it provides the `UndoRecorder` impl. Recorded as a
CLEAN row in the consistency matrix (no new conflict), with a note that the shell
wiring must supply a recorder backed by the winning transaction model.

### BOUNDS / CaseFolder cross-refs

- BOUNDS (Req 10): sequence-numbers is a READER only -- it never mutates BOUNDS
  (state owned by navigation-commands, single-owner confirmed W1.7). Overlap
  warning logic in `bounds.rs` reads active bounds; no ownership conflict.
- No CaseFolder usage -- PA-CONFLICT-003 (find-and-replace duplicate) does not
  extend here.

### Cross-reference integrity

All 8 declared cross-refs (document-model, edit-operations, navigation-commands,
configuration-system, command-framework, language-service, undo-redo-transactions,
file-operations) resolve to existing sub-projects. No dangling references.

---

## 3. Completeness

Tracking: all 22 top-level tasks (1-22, incl. Phase BY aliases) are `[x]`.
Implementation is present for every requirement area: `detector.rs`,
`strip.rs`/`auto_strip.rs`, `number.rs`/`number_cmd.rs`, `unnum.rs`,
`number_show.rs`, `save_handler.rs`, `profiles.rs`, `profile_config.rs`,
`commands.rs`, `indicators.rs`, `bounds.rs`, `undo_integration.rs`. Tests exist
in-file (`#[cfg(test)]` in every module) plus `tests/integration_tests.rs` and
`tests/property_tests.rs`.

No PA-INCOMPLETE raised: the implementation matches the tasks and reqs; no
false-positive test pattern detected (unlike PA-INCOMPLETE-002 in background-io).

### TCR gap (PA-TCR-003)

TCR.md has only 1 generic row for `ff-seqnum` (line 120: "Sequence number
parsing, renumbering") plus 2 Phase BY alias rows (Req 6.7a, Req 8 alias). The
other 14 requirements / ~90 criteria are NOT enumerated in TCR despite tests
existing in-crate. This is a TCR RECORDING gap, not a test gap -- same pattern as
PA-TCR-002 (find-and-replace). Recorded PA-TCR-003 (add per-requirement rows
citing existing tests; no code change).

---

## 4. Logging audit

Scan results in `crates/ff-seqnum/src`:

- `ff_logging` / `log_*!` macros: 0
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0 (pure model, GUI/IO-independent)

Zero-log is DEFENSIBLE here, consistent with the other pure GUI-independent
Wave 0/1 models (viewport, caret-and-selection, edit-operations, etc.): errors
surface via `Result<_, SeqNumError>`, and the crate does no I/O of its own.

HOWEVER two requirements SPECIFY WARN-level log records that are currently NOT
emitted (the spec text says "emit a WARN-level log record"):

- Req 1.4: malformed `sequence_cols_front`/`_back` -> WARN + ignore key.
- Req 2.8: detection_threshold out of [50,100] -> clamp + WARN.

Current code clamps/ignores correctly but emits NO log. This is a minor
spec-vs-impl gap. It is also the natural home for CR-NR-058 dev-logging: command
start/params/completion for UNNUM/NUMBER/NUMBER SHOW should be instrumented at
DEBUG under the `dev-logging` gate once logging Req 13 lands. Recorded as
PA-LOG-006 (LOW/dev-logging): wire ff-logging, emit the two mandated WARNs, and
add dev-logging command instrumentation.

---

## 5. Task revision proposals

- **PA-STD-013 (REFACTOR, if triggered)**: strip the non-ASCII characters from
  the crate source. Scan found 60 non-ASCII bytes across 16 files: em-dash
  (U+2014), en-dash (U+2013), right-arrow (U+2192), and box-drawing separators
  (U+2500 range) in doc comments and section banners. documentation.md is
  explicit: `.rs` files are STRICT ASCII (box-drawing is allowed in `.md` only,
  NOT in `.rs`). This is a real standards violation. Fix: replace `--`/`-`/`->`,
  and convert `// ___ Section ___` banners to `// === Section ===`. REFACTOR (no
  gate; no behaviour change). Recorded PA-STD-012 as the file-size WATCH and
  PA-STD-013 as this ASCII fix.
- **PA-TCR-003**: enumerate per-requirement TCR rows (no code).
- **PA-LOG-006**: wire ff-logging for the two mandated WARNs (Req 1.4, 2.8) plus
  CR-NR-058 dev-logging command instrumentation (owner/logging-Req-13 gated).
- **PA-DOC (naming)**: spec calls the crate `ff-sequence-numbers`; the actual
  crate is `ff-seqnum`. Align the spec text (or record the alias) -- doc-only.

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

sequence-numbers is a well-scoped, single-responsibility, fully-tracked crate.
Not a split candidate. No new cross-unit CONFLICT (undo is trait-decoupled --
clean seam for PA-CONFLICT-002). Open items are all LOW/mechanical: an ASCII
source cleanup (PA-STD-013, genuine documentation.md violation), a TCR
enumeration gap (PA-TCR-003), a small logging gap for two mandated WARNs plus
CR-NR-058 dev-logging (PA-LOG-006), and a crate-name doc drift.
