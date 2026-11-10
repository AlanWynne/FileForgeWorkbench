# RESUME -- ff-message (structured, language-aware messages)

Last updated: 2026-10-22. Written so work can resume cleanly after a restart.

## One-line status

Design gate for a new `ff-message` core crate is APPROVED and CLOSED. Phase 1 is
ready to implement. NO code has been written yet. Phase 2 is deferred/BLOCKED.

## What this is

A new GUI-free core crate `ff-message` that brings a SAP SE91 / BAPIRET2-style
structured, language-aware messaging model to FileForgeWorkbench: a program
emits a STABLE identity (Message Class + Message Number) + a SEVERITY + named
PARAMETERS, and the localized human-readable STRING is resolved only at the
presentation boundary (log sink / notification / dialog). Pass the structured
tuple around; render to a String only at the edge.

## The five locked owner decisions (do NOT re-litigate)

1. ADDITIVE layer, not a rewrite. `ff-message` depends on `ff-i18n` (text
   resolution -- do NOT rebuild it) and renders at the boundary, feeding the
   EXISTING `ff_logging::log(...)` API UNCHANGED and the EXISTING notification
   seam. `ff-logging` public API and `LogRecord` stay untouched. The heavier
   "ALL logging in message format" change is OUT OF SCOPE (possible future
   owner-confirmed framework change).
2. Identity is SAP-like: Message Class + Message Number, mapping to an `ff-i18n`
   catalogue key so text localizes automatically per locale. Parameters REUSE
   `ff_i18n::MessageArgs` (no second param model). Command verbs/identifiers stay
   English, passed as non-translated args.
3. Severity is a 6-level axis owned by `ff-message` (source of truth):
   Success, Info, Warning, Error, Abort, Fatal. Maps DOWN onto `NotificationLevel`
   (4) at the dialog boundary and `ff_logging::LogLevel` (5) at the log boundary
   (purely additive). Behavioural meaning is DEFINED via a first-class `FlowEffect`
   enum but NOT enforced this gate.
4. Key scheme: `MessageId { class, number }` -> `format!("{}-{:04}", class.to_lowercase(), number)`,
   e.g. `{VFS,7}` -> `vfs-0007`. `MessageNumber` bounded `0..=9999` (fallible
   `try_new`) so the 4-digit key width is guaranteed by construction.
5. Phase 2 (severity ACTUALLY driving batch/background/Command-Environment control
   flow) is DEFERRED and BLOCKED, needs its own gate against `docs/specs/batch-execution`
   and separate owner re-confirmation.

## Deliverables already produced (approved docs)

- c:\workspace\apps\FileForgeWorkbench\docs\specs\structured-messages\requirements.md  (9 EARS requirement groups)
- c:\workspace\apps\FileForgeWorkbench\docs\specs\structured-messages\design.md
- c:\workspace\apps\FileForgeWorkbench\docs\specs\structured-messages\tasks.md  (Phase 1 tasks 1-12; Phase 2 tasks 13-15 BLOCKED)

Supporting artifacts:
- c:\workspace\apps\FileForgeWorkbench\.agents\tasks\ff-message-investigation\findings.md         (read-only investigation of current state)
- c:\workspace\apps\FileForgeWorkbench\.agents\tasks\ff-message-investigation\design-review.md / .json  (review: APPROVED, 0 HIGH / 0 MEDIUM)
- c:\workspace\apps\FileForgeWorkbench\.agents\tasks\ff-message-investigation\approval-summary.md

## Gate status

- Design workflow wf_9564c9602d289a59 ("design-ff-message-structured-messages"): COMPLETE.
- Owner APPROVED. Gate CLOSED.
- Three documentation NITs were applied (doc-accuracy only; no design change):
  1. Em-dash note narrowed to CommandError only (VfsError is plain ASCII).
  2. StructuredMessage Send+Sync clarified as auto-derived (do NOT write a manual/unsafe impl).
  3. MessageClass::new_static dropped; sole constructor is the fallible try_new.
- All three docs verified plain ASCII (Select-String log empty).

## Key facts about the existing codebase (grounding, verified during investigation)

- `crates/ff-logging`: plain level-tagged string logger. `LogRecord { timestamp, level, module_path, message }`.
  Public API `log(level, module_path, &str)`, macros `log_info!`/`log_error!` etc.
  5-level `LogLevel` (Trace/Debug/Info/Warn/Error). DO NOT CHANGE THIS API.
- `crates/ff-i18n`: the catalogue/locale/param foundation (CR-NR-103). `Catalogue`,
  `set_catalogue`, free fns `t("key")` / `t_args(key, &MessageArgs)`, `MessageArgs`
  (named args), English Identity_Base (`IDENTITY_BASE_TAG="en"`), `.ftl` data in
  `crates/ff-i18n/i18n/en/`. REUSE; `ff-message` depends on it, never the reverse.
  Actively being adopted in worktree `.worktrees/l10n-phase2-task9` (does NOT touch ff-logging).
- Notification seam `crates/ff-desktop/src/notification/mod.rs`: `NotificationLevel`
  (Info/Success/Warning/Error, 4 levels); `Notification { level, title, detail, timestamp }`;
  `NotificationQueue` (cap 1000); `NotificationSender` (non-blocking, drop-if-full).
  First consumer -- add `Notification::from_structured` HERE (ff-desktop -> ff-message dep direction).
- Cross-boundary errors are `thiserror` enums (`crates/ff-vfs/src/error.rs` VfsError,
  `crates/ff-command/src/error.rs` CommandError), English-at-source Display, no machine-readable id.
  Only CommandError has pre-existing non-ASCII em-dashes (InvalidId, HistoryIo) -- DO NOT fix this gate.

## Planned ff-message module layout (from design.md)

```
crates/ff-message/
  Cargo.toml          (deps: ff-i18n path, ff-logging path, thiserror workspace)
  src/
    lib.rs            thin coordinator / re-exports only
    error.rs          MessageError (thiserror): InvalidClass, NumberOutOfRange
    identity.rs       MessageClass (validated try_new), MessageNumber(u16, 0..=9999 try_new), MessageId, catalogue_key
    severity.rs       Severity (6 variants, Ord), FlowEffect, to_log_level, flow_effect
    message.rs        StructuredMessage { id, severity, args: ff_i18n::MessageArgs } + accessors/ctors
    render.rs         render / render_title (alias of render) / render_detail (Option, <key>.detail)
    has_id.rs         HasMessageId trait -> Option<MessageId>
    adapters/
      mod.rs
      log.rs          log_message(&StructuredMessage, module_path) -> ff_logging::log
    tests.rs
```
Notification adapter is a SMALL addition in `crates/ff-desktop/src/notification/` (NOT in the core crate).
Seed catalogue: `crates/ff-i18n/i18n/en/messages-msg.ftl` (MSG class only; VFS/CMD reserved placeholders).

## NEXT STEP (what to do after restart)

Phase 1 is approved and ready. When the owner says go, LAUNCH AN IMPLEMENT-AND-REVIEW
WORKFLOW for Phase 1 (tasks 1-12 in tasks.md):
- TDD: failing tests first, each with `// Validates: Requirement X.Y`.
- Agents COMPILE-VERIFY scoped only: `cargo check -p ff-message`, `cargo clippy -p ff-message -- -D warnings`,
  `cargo fmt -- --check`, and scoped new tests where quick. OWNER runs the full gate / whole suite manually.
- Two-model semantic review at the end; keep file sizes under 400 non-test lines.
- No worktree strictly required (new crate + small ff-desktop edit + one .ftl seed), but a worktree is fine.

DO NOT:
- Change `ff-logging` public API or `LogRecord`.
- Rebuild or change `ff-i18n`.
- Implement Phase 2 (tasks 13-15) -- BLOCKED, needs separate owner re-confirmation + batch-execution gate.
- Modify VfsError/CommandError source (HasMessageId adoption is a later, separate gate).
- Edit management/roadmap docs beyond noting structured-messages should be registered (specs.md).

Also still open (optional, owner's call): register `docs/specs/structured-messages` in the specs registry / roadmap.

## Workflow launch setting reminder

Session setting is "When asked" -- do not auto-launch the Phase 1 implementation
workflow; wait for the owner to initiate it.
