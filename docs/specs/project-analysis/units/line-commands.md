# Analysis Record: line-commands

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-line-commands`
- **Spec folder**: `docs/specs/line-commands/`
- **Analysed**: Wave 1, task W1.6 (CR-NR-057/058 re-baseline)
- **Verdict**: COMPLETE (215/215 tasks `[x]`, 13 TCR rows PASS). NOT a split
  candidate. Corroborates PA-CONFLICT-002 (produces EditorTransaction; ff-undo-redo
  dep UNUSED = PA-DEP-001). One 400-cap refactor (resolution.rs 422). Zero-log
  defensible.
- **CR-NR-057/058 impact**: NONE (line-commands specs untouched). Re-verified.

---

## 1. Scope summary

`ff-line-commands` is the GUI-independent ISPF/PDF prefix-area line-command
engine: parsing, validation, block pairing, pending-state, execution. 15
requirements (delete D/Dn/DD; insert I/In; repeat R/Rn/RR; copy C/CC; move M/MM;
after/before A/B; exclude X/Xn/XX; tag/untag T/TT/U/UU; shift-right >/>n/>>;
shift-left </<n/<<; bounds-aware shift; block pairing; compatibility matrix;
pending state; additional ISPF O/W/F/L/]/S).

338 req lines, 15 requirements, single backing crate. Well-organised
(commands/ + execution/ subdirs).

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 338 lines (just under); 15 reqs (>12) | YES (reqs) |
| 3+ distinct responsibilities | ONE responsibility: the line-command engine (the 15 reqs are command variants of the same prefix model) | No |
| 2+ crates | one crate | No |
| low-cohesion clusters | high cohesion (parser/resolution/pending/execution pipeline) | No |
| file-size pressure | `resolution.rs` 422 non-test over cap; command.rs 354 near | Yes (1) |

Only reqs-count + one oversized file. **NOT a split candidate.** Fix is the
refactor (PA-STD-010).

## 3. Consistency / conflict

- Public types owned here (LineCommand, BlockCommand, PendingCommand,
  ImmediateCommand, SourceMarker/TargetMarker, CommandCompatibilityMatrix, the
  parser/resolution/pending engine) -- sole owner. Added to consistency-matrix.
- **PA-CONFLICT-002 corroboration**: ff-line-commands produces `EditorTransaction`
  FROM `ff-edit-operations` for all undoable commands (execute_delete ->
  `Result<EditorTransaction>`). It declares `ff-undo-redo` in Cargo.toml but uses
  it ZERO times in src (grep 0). Confirms EditorTransaction is de-facto canonical
  at the command layer; undo-redo's model is unwired. Logged PA-DEP-001.
- Consumer relationships correct: document-model (mutation primitives),
  display-line-mapping (X/XX exclusion via canonical `DisplayLineMapping` --
  confirmed W1.4), exclude-show-filter (X sets `excluded`, W1.11),
  command-semantics (line-command collection step, W3.1), navigation-commands
  (BOUNDS for bounds-aware shift, W1.7), configuration-system (ShiftWidth,
  invalid_line_command_policy). No conflict.
- Session-state vs undoable boundary consistent: X/T/U/F/L/S/W produce NO
  transaction (Req 7.5, 8.7, 15.11, 15.12); D/I/R/C/M/>/</)/( and O produce a
  single EditorTransaction. Matches undo-redo Req 10 (non-undoable operations).

## 4. Completeness

- Tasks: 215 `[x]`, 0 `[ ]`. 13 `ff-line-commands` TCR rows all PASS (parser,
  resolution, execution per command family incl. Req 15 O/W/F/L/]/S).
  Well-modularised (per-command execution files).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` declared (wired, unused). DEFENSIBLE: pure
  command engine. Invalid/incomplete commands are surfaced via PendingCommand
  retention + display messages (Req 5/12/14) and `LineCommandError` (Result) to
  the caller -- not logged. No requirement mandates a log.
- CR-NR-058 relevance: line-command dispatch/resolution would be good `log_trace!`/
  `log_debug!` candidates under the new `dev-logging` gate; optional, folds into
  the PA-LOG-002 re-scope.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a pure command engine)**.

## 6. Findings logged

- **PA-CONFLICT-002 (corroborating evidence)**: line-commands produces
  edit-operations `EditorTransaction` for all undoable commands and never uses its
  declared `ff-undo-redo` dependency. Register PA-CONFLICT-002 row already carries
  this; no new row.
- **PA-DEP-001** (UNUSED DEPENDENCY -- REFACTOR, carried/re-confirmed):
  `ff-line-commands` declares `ff-undo-redo` in Cargo.toml but no source uses it.
  Remove the dependency (or wire it in if undo-redo becomes canonical per
  PA-CONFLICT-002). No gate.
- **PA-STD-010** (REFACTOR -- 400-line cap): `resolution.rs` is 422 non-test lines
  (command.rs 354 near). Split resolution.rs by concern (resolution_block /
  resolution_target; keep dispatch). REFACTOR, no gate.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-line-commands contributes
  matches (doc-comment em-dashes, ISPF command glyphs in prose, separators).
  Rolled into project-wide PA-LOG-001.
