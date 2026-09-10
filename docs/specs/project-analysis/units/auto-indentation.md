# Analysis Record: auto-indentation (W1.13)

- **Wave**: 1 (Editor Core)
- **Backing crate**: `ff-auto-indent`
- **Spec files**: requirements.md (259 lines, 10 requirements), tasks.md
  (137 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 1 re-baseline pass (post CR-NR-057 reset)

---

## 1. Split candidacy

NOT a split candidate.

Assessment against the 2-of-4 rule:

- Requirement volume: 10 requirements, 259 lines. Below both thresholds. No signal.
- Responsibilities: ONE domain (language-aware indentation) with cohesive facets:
  maintain/smart indent on Enter, increase/decrease patterns, between-braces
  expansion, comment auto-continue, Indent/Unindent commands. All feed the same
  decision engine. Already split by concern into 13 files (block/comment/config/
  decision/indent_cmd/maintain/mode/patterns/service/smart/types).
- Crates: single crate; minimal deps (ff-logging + regex + thiserror only).
- Cohesion: high.

Zero firm signals. No split.

### Source file sizes -- ALL WITHIN CAP

Largest non-test file `comment.rs` at 234 non-test lines; `patterns.rs` 192,
`service.rs` 177, `smart.rs` 167. None approaches the 400 cap. Well-decomposed.
No PA-STD size item.

---

## 2. Cross-unit consistency

### PA-CONFLICT-002 (transaction model) -- CLEAN SEAM (decoupled)

The spec repeatedly says auto-indent modifications are "recorded as an
EditorTransaction" (Req 2.4, 4.4, 5.3, 6.5, 7.4, 8.5, 10.1, 10.5). This could
have made auto-indent a PARTY to PA-CONFLICT-002 (EditorTransaction vs
undo-redo Transaction). It is NOT:

- Cargo deps are ONLY `ff-logging`, `regex`, `thiserror`. There is NO dependency
  on ff-edit-operations, ff-undo(-redo), ff-document-model, ff-language-service,
  or ff-command (grep confirmed empty).
- The crate RETURNS plain data: `IndentDecision`, `CommentContinuation`,
  `BlockExpansion` (decision.rs). The CALLER (edit-operations / shell) wraps these
  into `EditorTransaction`. Req 10.4 makes this explicit: "operate purely on the
  document model ... the GUI shell triggers auto-indent through the edit-operations
  API; the subsystem returns the indentation to apply."

Consequence: auto-indentation is decoupled by design -- same pattern as
sequence-numbers (W1.9). It binds to whichever transaction model the caller uses.
Recorded as a CLEAN row (no new conflict). This is a THIRD data point (after
sequence-numbers and whitespace-guides) confirming the Wave 1 model crates favour
parameter/return-data injection over crate deps, which keeps them out of
PA-CONFLICT-002.

### Cross-references realized at caller layer

All 6 declared cross-refs (language-service, edit-operations, document-model,
configuration-system, command-framework, undo-redo-transactions) are realized by
PARAMETER INJECTION / caller responsibility, not crate edges. The crate takes
language rules, indent settings, and syntax state as inputs and returns decisions.
This is architecturally clean and consistent with Req 10.4. No dangling refs; no
duplicated types.

### Public types and ownership

- `IndentDecision`, `CommentContinuation`/`CommentKind`, `BlockExpansion`, the
  Auto_Indent mode enum, indent-pattern/config types -- sole-owned by
  `ff-auto-indent`. No duplication.
- `edit.indent` / `edit.unindent` commands (Tab / Shift+Tab): registered via
  command-framework by the caller; auto-indent provides the indent computation.
  Consistent with command ownership.
- Config keys `editor.auto_indent`, `editor.indent_size`, `editor.tab_size`,
  `editor.use_tabs` + per-language `[indent]`/`[comment]` TOML tables -- read from
  configuration-system + language-service. No key collision found.
- EditorConfig precedence (Req 1.6): defers to configuration-system's EditorConfig
  detection. Consistent (verify at configuration-system W0.3 record -- already
  analysed).

---

## 3. Completeness

Tracking: all 137 sub-tasks `[x]`. Implementation present across all 10 req areas
(maintain, smart increase/decrease, between-braces, comment-continue, indent/
unindent commands, TOML rules, edit integration). Tests in-file with
`// Validates:` annotations. EXCEPT the mandated logging (below).

### PA-INCOMPLETE-006 (HIGH) -- mandated logging STUBBED OUT

Two requirements MANDATE logging, and NEITHER is implemented:

- Req 9.7: "Invalid patterns SHALL be logged as WARN and the pattern SHALL be
  treated as non-matching." The ONLY ff-logging reference in the whole crate is a
  COMMENTED-OUT call at `patterns.rs:47`:
  `// ff_logging::log(LogLevel::Warn, "auto-indent", &format!("invalid regex: {}", source));`
  The invalid-pattern-as-non-matching behaviour appears present, but the mandated
  WARN log is disabled.
- Req 10.7: "THE auto-indentation subsystem SHALL emit a DEBUG-level log record
  for each auto-indent decision, including the reference line number, matched
  pattern (if any), and resulting indent level." NO code exists for this at all.

This is NOT the benign "zero-log defensible" case seen on other pure models --
here logging is an EXPLICIT acceptance criterion that is stubbed/absent. It is a
false-positive-tracking pattern (tasks 137/137 `[x]` while Req 9.7/10.7 logging is
not done), same class as PA-INCOMPLETE-002 (background-io). And it is squarely a
CR-NR-058 concern: Req 10.7's per-decision DEBUG log is EXACTLY the dev/debug
instrumentation the user prioritised. Recorded PA-INCOMPLETE-006 (HIGH):
uncomment/implement the Req 9.7 WARN and implement the Req 10.7 per-decision DEBUG
record, ideally under the `dev-logging` gate (logging Req 13, PA-CR058) so the
per-decision DEBUG is stripped in release.

### TCR gap (PA-TCR-007)

TCR.md has 1 row for `ff-auto-indent` (generic) against 10 reqs / ~60 criteria.
Thin coverage enumeration (like PA-TCR-002/003). Plus, once PA-INCOMPLETE-006 is
fixed, Req 9.7/10.7 need PASS rows. Recorded PA-TCR-007.

---

## 4. Logging audit

Scan of `crates/ff-auto-indent/src` (recursive):

- `ff_logging` / `log_*!` ACTIVE calls: 0 (the single reference is commented out,
  patterns.rs:47 -- see PA-INCOMPLETE-006)
- `ff-logging` Cargo dep: PRESENT and INTENDED (unlike PA-LOG-008/009 dead deps --
  here the dep is correct, the call is just disabled)
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

Unlike the last two units (dead ff-logging deps), here ff-logging is a LEGITIMATE
dependency whose use is mandated by Req 9.7 + 10.7 but currently stubbed. The
logging gap is tracked as PA-INCOMPLETE-006 (HIGH, mandated) rather than a
LOW/dev-logging suggestion -- this crate is the clearest Wave 1 example of the
CR-NR-058 dev/debug-logging requirement already being written into the spec.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-006 (HIGH, task-revision)**: implement the two mandated logs --
  Req 9.7 WARN on invalid regex (uncomment + wire `patterns.rs:47`), Req 10.7
  per-decision DEBUG (new code in the decision/service path). Gate the per-decision
  DEBUG behind `dev-logging` (logging Req 13) so it is release-stripped. Re-open
  the corresponding tasks (they are wrongly `[x]`). Owner/logging-Req-13 aware.
- **PA-STD-019 (ASCII, ACTIONABLE -- runtime string)**: 99 non-ASCII bytes in
  `.rs` (15 in non-comment code). `error.rs:12` has an em-dash INSIDE a runtime
  error string ("invalid mode '{value}' -- expected ..."). Others: arrows (->) in
  test comments (indent_cmd.rs "4 -> 8 spaces"), em/en dashes + box-drawing banners
  in doc comments. documentation.md mandates strict ASCII in `.rs`. Replace with
  `--`/`-`/`->`; convert banners. REFACTOR, no gate.
- **PA-TCR-007**: enumerate per-requirement TCR rows (1 row for 10 reqs today);
  add Req 9.7/10.7 rows once PA-INCOMPLETE-006 lands. No code (until then).

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

auto-indentation is a well-scoped, well-decomposed engine that -- like
sequence-numbers and whitespace-guides -- stays OUT of PA-CONFLICT-002 by
returning plain decision data (`IndentDecision` etc.) and letting the caller wrap
it in `EditorTransaction`; its only real deps are ff-logging + regex. Not a split
candidate; no size violation. The significant finding is PA-INCOMPLETE-006 (HIGH):
Req 9.7 (invalid-pattern WARN) and Req 10.7 (per-decision DEBUG) MANDATE logging,
but the WARN is commented out and the DEBUG is absent -- while tasks read 137/137.
This is the clearest Wave 1 case of CR-NR-058 dev/debug logging already being a
written acceptance criterion, and should be implemented under the `dev-logging`
gate. Minor items: an ASCII violation incl. a runtime error string (PA-STD-019)
and a thin TCR (PA-TCR-007).
