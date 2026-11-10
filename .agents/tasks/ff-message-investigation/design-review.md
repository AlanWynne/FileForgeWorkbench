# Design Review -- Structured, Language-Aware Messages (ff-message)

Reviewer: design-review subagent (fresh context; no authoring history).
Scope: requirements-gate review of three unwritten-crate deliverables plus the
investigation findings. No source code exists or should exist yet. No cargo /
build was run; this is a document review cross-checked against the REAL
signatures of the crates the design depends on.

Documents reviewed:
- docs/specs/structured-messages/requirements.md
- docs/specs/structured-messages/design.md
- docs/specs/structured-messages/tasks.md
- .agents/tasks/ff-message-investigation/findings.md (investigation input)

Source signatures read to verify the design's claims:
- crates/ff-logging/src/level.rs, crates/ff-logging/src/init/api.rs
- crates/ff-i18n/src/lib.rs, args.rs, seam.rs, catalogue.rs
- crates/ff-desktop/src/notification/mod.rs
- crates/ff-vfs/src/error.rs, crates/ff-command/src/error.rs

---

## Verdict

CHANGES_REQUESTED is NOT warranted on substance: the design is coherent,
grounded in the real signatures, and conforms to the locked owner decisions and
the framework/Rust/task-format/charset rules. There are ZERO HIGH or MEDIUM
findings. The only findings are NITs (documentation-accuracy polish), which by
the mechanical rule do not block.

Mechanical count: HIGH = 0, MEDIUM = 0, NIT = 3 -> APPROVED.

---

## Verified Assumptions (checked against real source)

1. ff_logging::LogLevel is EXACTLY Trace/Debug/Info/Warn/Error (5 variants,
   `#[non_exhaustive]`, derives PartialOrd/Ord). The design's
   `Severity::to_log_level` mapping (`Success`/`Info` -> `Info`; `Warning` ->
   `Warn`; `Error`/`Abort`/`Fatal` -> `Error`) names only real variants and is
   total. VERIFIED (crates/ff-logging/src/level.rs).

2. ff_logging::log has signature `log(level: LogLevel, module_path: &str,
   message: &str)`. The design's `log_message` body
   `ff_logging::log(msg.severity().to_log_level(), module_path, &msg.render())`
   type-checks: `render()` returns `String`, `&String` coerces to `&str`.
   VERIFIED (crates/ff-logging/src/init/api.rs). The Req 5.3 claim that the
   not-initialised / fallback path is unchanged is borne out: `log` early-returns
   on shutdown and on an uninitialised `SUBSYSTEM`, never panics.

3. ff_i18n::t_args has signature `t_args(key: &str, args: &MessageArgs) ->
   String`. The design's `render` body
   `ff_i18n::t_args(&self.id.catalogue_key(), &self.args)` type-checks
   (`catalogue_key()` returns `String`; `&String` coerces to `&str`). VERIFIED
   (crates/ff-i18n/src/seam.rs).

4. ff_i18n missing-key fallback is active-locale -> English Identity_Base -> the
   key itself with a WARN, and the pre-catalogue state also returns the key with
   a WARN. The design's reliance on this exact chain (Req 2.4, 8.4; no second
   fallback) is accurate, and the `render_detail` "key-echo equality" presence
   check is a legitimate consequence of this documented behaviour. VERIFIED
   (crates/ff-i18n/src/seam.rs, catalogue.rs).

5. ff_i18n::MessageArgs is `#[derive(Debug, Default, Clone)]` over
   `Vec<(String, ArgValue)>` (ArgValue = String | f64). Therefore it is
   auto-`Send + Sync`, so a `StructuredMessage` holding it (with a `Clone`
   `MessageId` and a `Copy` `Severity`) is `Clone + Send + Sync` as Req 1.3 / 9.2
   claim. The builder shape `MessageArgs::new().set(name, value)` the design
   mirrors with `with_args` exists. VERIFIED (crates/ff-i18n/src/args.rs).

6. ff_i18n::Catalogue::load_identity_base(&Path) and set_catalogue exist exactly
   as the design's test plan uses them; the seam is a process-wide swappable
   `RwLock<Option<Catalogue>>`. The Testability note that installing the SAME
   English Identity_Base concurrently is benign is consistent with the
   implementation (idempotent replace behind the lock). VERIFIED
   (crates/ff-i18n/src/catalogue.rs, seam.rs, lib.rs).

7. NotificationLevel is EXACTLY Info/Success/Warning/Error (4 variants). The
   design's `notification_level_for` match arms name only real variants and are
   exhaustive. `Notification::new(level, title: String, detail: Option<String>)`
   matches the design's `from_structured` call
   `Notification::new(level, msg.render_title(), msg.render_detail())` exactly
   (`render_title()` -> `String`, `render_detail()` -> `Option<String>`). The
   non-blocking drop-if-full `try_send` semantics the design preserves (Req 6.3)
   are real. VERIFIED (crates/ff-desktop/src/notification/mod.rs).

8. CommandError::InvalidId and CommandError::HistoryIo DO carry a non-ASCII
   em-dash (U+2014) in their `#[error(...)]` strings, so the design's decision to
   leave them untouched (and defer `HasMessageId` adoption to their own gate) is
   grounded. VERIFIED (crates/ff-command/src/error.rs).

9. No non-ASCII character appears anywhere in the three spec documents
   (grep for `[^\x00-\x7F]` returned no matches). The documentation-charset rule
   is satisfied. VERIFIED.

10. tasks.md uses only `[ ]` checkboxes, every task line has a descriptive
    title, and every status note (BLOCKED, Note) is on an indented sub-bullet,
    never inside the brackets. task-format.md is satisfied. VERIFIED.

11. EARS: every acceptance criterion is numbered and uses a valid EARS form --
    either event-driven (`WHEN ... THE ... SHALL ...`, e.g. Req 1.5, 2.3, 2.4,
    8.3, 8.4) or ubiquitous (`THE ... SHALL ...`, e.g. Req 1.1-1.4, 3.1). Both
    are legitimate EARS patterns. VERIFIED.

---

## Unverified / Wrong Assumptions

1. WRONG (minor, documented in NIT 1 below): both requirements.md (Out of Scope)
   and design.md ("Optional MessageId on thiserror enums") assert that
   `VfsError`/`CommandError` BOTH contain pre-existing non-ASCII em-dashes in
   their `#[error(...)]` strings. `CommandError` does; `VfsError` does NOT -- its
   `#[error(...)]` strings are plain ASCII (`[vfs] {operation}: ...: {uri}`).
   This does not affect any scope decision (both enums are correctly left
   untouched this gate) but the stated justification is factually inaccurate for
   `VfsError`. Carried through from findings.md point 4, which itself attributed
   the em-dash only to `CommandError` -- the two specs over-generalised it to
   both.

2. NOT INDEPENDENTLY VERIFIED (out of review scope, correctly deferred): the
   claim that `VfsError`/`CommandError` adoption of `HasMessageId` is a
   separately-gated step in those crates' own specs. The design does not modify
   those enums this gate, which is the only thing that matters here; the future
   gating is an assertion about process, not code, and needs no verification now.

---

## Findings

### NIT 1 -- Inaccurate "VfsError em-dash" justification
Location: requirements.md "Out of Scope" bullet ("Modifying `VfsError` /
`CommandError` source (including their pre-existing non-ASCII em-dash
`#[error(...)]` strings...") and design.md "Optional MessageId on thiserror
enums" ("The known pre-existing non-ASCII em-dash in `VfsError`/`CommandError`
`#[error(...)]` strings is a SOURCE issue...").

Problem: `VfsError`'s `#[error(...)]` strings are plain ASCII; only
`CommandError` (variants `InvalidId` and `HistoryIo`) carries the em-dash. The
docs over-attribute the charset issue to both enums.

Fix: narrow the parenthetical to `CommandError` only, e.g. "(including the
pre-existing non-ASCII em-dash in two `CommandError` `#[error(...)]` strings --
`InvalidId` and `HistoryIo`)". Do NOT change the scope decision; both enums stay
untouched this gate regardless.

### NIT 2 -- `MessageArgs` Send+Sync is asserted, not shown to be derived
Location: design.md "The structured message" / Decisions summary
("`MessageArgs` is `Clone + Send + Sync` (it holds owned `String`/`f64`)").

Problem: the statement is CORRECT (it is auto-`Send + Sync`), but a reader could
misread it as claiming an explicit derive. `MessageArgs` derives only
`Debug, Default, Clone`; `Send + Sync` are auto-traits inferred from its fields.
The `StructuredMessage` type will likewise `#[derive(Debug, Clone)]` and get
`Send + Sync` automatically.

Fix: add half a sentence: "...`Send + Sync` are AUTO-derived (auto-traits) from
its owned `String`/`f64` fields, not an explicit derive; `StructuredMessage`
therefore gets them automatically." This removes any chance the implementer
writes a redundant/incorrect `unsafe impl` or a manual bound.

### NIT 3 -- `MessageClass::new_static` returns Result despite the "infallible" framing
Location: design.md "Identity" ("An infallible `const`-friendly path for
compile-time-known classes is provided as `MessageClass::new_static(&'static
str)` which still validates and returns `Result`...").

Problem: the prose calls it "an infallible ... path" in the same sentence that
says it "returns `Result`". That is self-contradictory on a quick read, and it
is not referenced by any task (tasks 3.1/3.2 only mention `try_new`), so an
implementer has no acceptance criterion telling them whether `new_static` is in
scope this gate.

Fix: either (a) drop `new_static` from the design for this gate (only `try_new`
is tested in tasks 3.1/3.2, so nothing needs it), or (b) keep it but rename the
framing to "a `&'static str` convenience constructor that still validates and
returns `Result`" (remove the word "infallible") AND add a sub-note to task 3.2
that it is implemented and tested. Recommended: (a), to keep the surface minimal
and the tasks and design in lockstep.

---

## Conformance checks (all pass)

- Locked owner decision (a) ADDITIVE: design.md "Framework-conformance
  statement" and "Crate boundary" state ff-message depends on ff-i18n, leaves
  ff-logging public API and `LogRecord` UNCHANGED, and marks the global
  "all logging to message format" move explicitly OUT OF SCOPE (requirements Out
  of Scope + tasks "Explicitly NOT a task"). PASS.
- Locked owner decision (b) SAP-like {class, number} identity mapping to an
  ff-i18n key: the scheme `format!("{}-{:04}", class.to_lowercase(), number)` is
  single, documented, deterministic, total; the missing-entry fallback reuses
  ff-i18n's key-echo+WARN; parameters reuse `ff_i18n::MessageArgs` (no second
  model); verbs stay English as non-translated args. Req 2.1-2.6, 8.1-8.5. PASS.
- Locked owner decision (c) six-level Severity as source of truth WITH
  behavioural meaning AND mapping-down functions: `Severity` (6 ordered
  variants) + `FlowEffect` + `to_log_level` (to 5-level LogLevel) +
  `notification_level_for` (to 4-level NotificationLevel), Abort/Fatal collapsing
  to Error at interactive surfaces, full axis at batch/background. Req 3.1-3.5,
  4.1-4.4. PASS.
- Locked owner decision (d) every enumerated design question resolved: concrete
  `MessageClass`/`MessageNumber`/`MessageId` newtypes; `Severity`;
  `MessageArgs` carrier reused directly; `StructuredMessage` Clone+Send+Sync;
  render/render_title/render_detail API; log + notification adapters; optional
  `HasMessageId` trait leaving `Display` unchanged; catalogue-entry
  location/naming (`crates/ff-i18n/i18n/en/messages-<class>.ftl`) and class
  reservation registry note. design.md "Decisions summary" and "Concrete Rust
  types". PASS.
- tasks.md Phase 1 vs Phase 2 separation: Phase 1 (crate + adapters + catalogue
  conventions) carries TDD notes and SCOPED `cargo check`/`clippy`/`fmt`
  verification with the owner running the full gate; Phase 2 (honouring severity
  behaviour in batch/background/Command-Environment) is clearly marked, BLOCKED,
  references docs/specs/batch-execution, and flagged as higher blast radius
  needing owner re-confirmation. PASS.
- framework-conformance: additive new DATA TYPE only; no new dispatch path, no
  second nav stack, no new session-persistence format, no change to ff-logging
  API/LogRecord, no change to WorkspaceContext/InteriorFocus. design.md
  "Framework-conformance statement". PASS.
- rust-standards: 400-line split by concern (module layout section), `thiserror`
  `MessageError`, newtypes over primitives, illegal states unrepresentable
  (validated-only constructors), no unwrap/expect/panic in library code. PASS.
- Cross-reference notes to localization / logging-subsystem / notification-system
  are present and light-touch; future cross-spec changes are marked PROPOSED
  (design.md "Cross-spec relationships"). PASS.
- documentation-charset: no non-ASCII in any of the three docs. PASS.
- task-format: only `[ ]`; all task lines titled; status notes on sub-bullets.
  PASS.

---

## Summary

The three documents are implementation-ready. Every signature the design leans
on was checked against the real crate source and matches. The locked owner
decisions and all workspace rules are honoured. The three NITs are
documentation-accuracy polish (a mis-attributed em-dash, an auto-trait wording
clarification, and a self-contradictory "infallible ... returns Result" phrase
for an untasked constructor); none blocks the gate. Addressing them would tighten
the docs but is not required before owner approval.
