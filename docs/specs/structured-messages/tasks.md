# Implementation Tasks -- Structured, Language-Aware Messages (ff-message)

Task discipline (per `testing.md` and the execution split): every
implementation task writes its FAILING tests FIRST, each carrying a
`// Validates: Requirement X.Y` annotation, then the minimum code to pass. Agents
COMPILE-VERIFY with SCOPED, bounded commands and read their logs:
`cargo check -p ff-message`, `cargo clippy -p ff-message -- -D warnings`,
`cargo fmt -- --check`, and the single new scoped test (`cargo test -p
ff-message <test_name>`) where quick. The OWNER runs the full gate / whole test
suite manually. Do NOT run `cargo test`/`cargo gate`/`cargo full-gate` as an
agent verification step. Keep every source file under 400 non-test lines.

---

## Phase 1 -- The ff-message crate (additive; safe to implement after approval)

- [ ] 1. Scaffold the ff-message crate
  - [ ] 1.1 Create `crates/ff-message/Cargo.toml` depending on `ff-i18n` (path), `ff-logging` (path), and `thiserror` (workspace); add the crate to the workspace members (Req 9.1)
  - [ ] 1.2 Create a thin `src/lib.rs` with crate docs and re-exports only; confirm `cargo check -p ff-message` builds the empty crate (Req 9.4)
    - Note: no logic in lib.rs; it is a coordinator per the Rust standards.

- [ ] 2. Library error type
  - [ ] 2.1 Write failing tests for `MessageError` variants (invalid class reasons; number-out-of-range) with `// Validates: Requirement 1.5, 9.3`
  - [ ] 2.2 Implement `src/error.rs` with a `thiserror` `MessageError` enum (e.g. `InvalidClass { value, reason }`, `NumberOutOfRange { value }`); no unwrap/expect/panic (Req 1.5, 9.3)

- [ ] 3. Message identity (class, number, id, key mapping)
  - [ ] 3.1 Write failing tests: `MessageClass::try_new` accepts short uppercase ASCII and rejects empty/lowercase/non-ASCII/over-length; `MessageNumber::try_new` accepts `0..=9999` and rejects `> 9999` with `NumberOutOfRange`; `MessageId::catalogue_key` formats `vfs-0007` (lowercase class, fixed zero-pad width 4, with `9999` rendering exactly four digits) -- all with `// Validates: Requirement 1.1, 1.2, 1.5, 2.1, 2.2`
  - [ ] 3.2 Implement `src/identity.rs`: `MessageClass` newtype (validated `try_new`), `MessageNumber(u16)` newtype (fallible `try_new`, bounded `0..=9999`), `MessageId`, and `MessageId::catalogue_key` (pure/total) with `KEY_NUMBER_WIDTH` constant (Req 1.1, 1.2, 1.5, 2.1, 2.2)
    - Note: illegal states unrepresentable -- the only `MessageClass` constructor validates.

- [ ] 4. Severity axis, flow-effect contract, and LogLevel mapping
  - [ ] 4.1 Write failing tests: `Severity` ordering (Success < ... < Fatal); `flow_effect` for all six variants; `to_log_level` for all six variants (Req 4.1) -- with `// Validates: Requirement 3.1, 3.2, 3.3, 4.1`
  - [ ] 4.2 Implement `src/severity.rs`: six-variant ordered `Severity`, `FlowEffect` enum, `Severity::flow_effect` (DEFINED only, Req 3.4), `Severity::to_log_level` (Req 4.1, 4.3)
    - Note: `flow_effect` is declarative this gate; NOTHING reads it to change control flow (Req 3.4). Document the Abort/Fatal -> UI Error collapse (Req 3.5).

- [ ] 5. StructuredMessage type
  - [ ] 5.1 Write failing tests: `new` / `with_args` build a message; accessors `id()`/`severity()`/`args()` return the parts; type is `Clone + Send + Sync` (a compile-assert) -- with `// Validates: Requirement 1.3, 1.4, 1.6, 9.2`
  - [ ] 5.2 Implement `src/message.rs`: `StructuredMessage { id, severity, args: ff_i18n::MessageArgs }` with private fields + accessors + constructors; reuse `MessageArgs` directly (no wrapper) (Req 1.3, 1.4, 1.6, 9.2)

- [ ] 6. Render via ff-i18n
  - [ ] 6.1 Write failing tests using the in-test English Identity_Base pattern (`ff_i18n::Catalogue::load_identity_base` + `set_catalogue`): `render` returns catalogue text for a present key; returns the key string for an absent key (ff-i18n fallback); `{ $arg }` interpolation works; an embedded verb passed as an argument is NOT translated; `render_detail` returns `Some` for a present `<key>.detail` and `None` for an absent one (the test doc comment records the known false-negative: a `.detail` entry whose text equals its own key reads as absent) -- with `// Validates: Requirement 2.3, 2.4, 2.5, 2.6, 8.4, 8.5`
  - [ ] 6.2 Implement `src/render.rs`: `render` calls `ff_i18n::t_args(id.catalogue_key(), args)`; `render_title`; `render_detail` using the `<key>.detail` key-echo presence check (Req 2.3, 2.4, 2.5, 2.6)
    - Note: add NO second fallback; rely on ff-i18n's active -> Identity_Base -> key-echo+WARN chain (Req 2.4, 8.4).

- [ ] 7. Log adapter (in-core)
  - [ ] 7.1 Write failing tests for the mapping surface: `log_message` selects the `to_log_level` level for each severity (assert via the mapping; the ff_logging side effect is covered by ff-logging's own tests) -- with `// Validates: Requirement 5.1, 5.4`
  - [ ] 7.2 Implement `src/adapters/log.rs` + `src/adapters/mod.rs`: `log_message(&StructuredMessage, module_path)` renders once and calls the EXISTING `ff_logging::log(level, module_path, &rendered)` (Req 5.1, 5.2, 5.3, 5.4)
    - Note: no change to ff-logging public API or LogRecord; adapter adds no logic beyond render-then-log.

- [ ] 8. Optional HasMessageId trait
  - [ ] 8.1 Write a failing test implementing `HasMessageId` on a local dummy error and asserting its `Display` is unchanged by the impl -- with `// Validates: Requirement 7.1, 7.2`
  - [ ] 8.2 Implement `src/has_id.rs`: `HasMessageId` trait returning `Option<MessageId>` (Req 7.1, 7.2)
    - Note: this gate only DEFINES the trait; it does NOT touch `VfsError`/`CommandError` source (Req 7.3).

- [ ] 9. Catalogue conventions and seed data
  - [ ] 9.1 Add the English Identity_Base seed file `crates/ff-i18n/i18n/en/messages-msg.ftl` with example `MSG` class entries (e.g. `msg-0001`, `msg-0001.detail`) used by the render tests; plain ASCII only (Req 8.1, 8.5, 9.5)
  - [ ] 9.2 Record the reserved-number registry NOTE: the class/number-range table in `design.md` and a header comment block in `messages-msg.ftl`; reserve `MSG`, `VFS`, `CMD` ranges (Req 8.2, 8.3)
    - Note: lightweight convention, not a runtime registry; authoring-time authority against collisions.

- [ ] 10. Crate-level verification (agent scope)
  - [ ] 10.1 Run SCOPED `cargo check -p ff-message`, `cargo clippy -p ff-message -- -D warnings`, `cargo fmt -- --check`, redirect each to a log and READ the log (per terminal-discipline)
  - [ ] 10.2 Run the new scoped unit tests where quick (`cargo test -p ff-message <test_name>`); leave the full-suite run to the owner
    - Note: agent definition-of-done = tests written + crate compiles clean + lint/format clean + scoped new tests pass. The authoritative full gate is the owner's manual run.

- [ ] 11. Notification adapter (desktop boundary; small, additive edit in ff-desktop)
  - [ ] 11.1 Write a failing test in `ff-desktop` asserting `Notification::from_structured` sets the mapped level (Success/Info/Warning/Error, and Abort/Fatal collapse to Error), title from `render_title`, detail from `render_detail` -- with `// Validates: Requirement 4.2, 6.1`
  - [ ] 11.2 Implement `notification_level_for(ff_message::Severity) -> NotificationLevel` and `Notification::from_structured(&ff_message::StructuredMessage)` in `crates/ff-desktop/src/notification/`; add `ff-message` to `ff-desktop`'s dependencies (Req 4.2, 4.4, 6.1, 6.2, 6.3, 6.4)
    - Note: dependency direction is ff-desktop -> ff-message ONLY; the core crate stays GUI-free. No change to NotificationSender/NotificationQueue signatures or the non-blocking drop-if-full semantics.

- [ ] 12. Register the spec in the roadmap
  - [ ] 12.1 Note (per specs.md) that `docs/specs/structured-messages` should be added to the specs registry / roadmap; defer the actual management-doc edit to the owner's roadmap pass
    - Note: this task does not edit management docs; it records the registration need.

---

## Phase 2 -- HONOURING severity behaviour (BEHAVIOURAL; HIGHER BLAST RADIUS; DO NOT IMPLEMENT until owner re-confirms)

Phase 2 changes OBSERVABLE control flow in batch/background/Command-Environment
execution and therefore requires a SEPARATE owner re-confirmation and its own
requirements-gate pass against `docs/specs/batch-execution`. Nothing in Phase 2
is implemented as part of this gate. These tasks exist so the behavioural scope
is visible and explicitly separated from the additive Phase 1.

- [ ] 13. Gate Phase 2 against batch-execution (requirements-first; no code)
  - [ ] 13.1 Author/extend acceptance criteria in `docs/specs/batch-execution` for honouring `FlowEffect`: `Error` stops the current operation, `Abort` aborts the whole task-or-batch run, `Fatal` terminates (Req 3.3 behaviour; Phase 2)
    - BLOCKED: requires owner re-confirmation that behavioural honouring is wanted; this is a framework-adjacent behavioural change, not additive.

- [ ] 14. Honour FlowEffect in batch/background execution (behavioural)
  - [ ] 14.1 Write failing tests that a batch run STOPS on `Abort` and TERMINATES on `Fatal`, reading `Severity::flow_effect` at the execution boundary
  - [ ] 14.2 Implement the flow-control honouring in the batch/background executor (references `batch-execution`), reusing `Severity::flow_effect` (no new severity model)
    - BLOCKED: do not start until Task 13 is approved and owner re-confirms. Higher blast radius: changes batch semantics.

- [ ] 15. Honour FlowEffect in Command-Environment execution (behavioural)
  - [ ] 15.1 Write failing tests for a Command Environment aborting/terminating per `FlowEffect`
  - [ ] 15.2 Implement honouring at the Command-Environment execution seam
    - BLOCKED: do not start until Task 13 is approved and owner re-confirms.

---

## Explicitly NOT a task in any phase

- Changing the `ff-logging` public API or `LogRecord`.
- The global "all logging moves to the message format" change (every log record
  carrying a class+number+severity identity) -- a possible future owner-confirmed
  framework change, not planned here.
- Rebuilding or changing `ff-i18n`.
- Modifying `VfsError` / `CommandError` source (including the pre-existing
  non-ASCII em-dash in two `CommandError` `#[error(...)]` strings -- `InvalidId`
  and `HistoryIo`; `VfsError` is plain ASCII) -- adoption of `HasMessageId` on
  those enums is their own future gate.
