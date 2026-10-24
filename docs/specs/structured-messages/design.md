# Design Document -- Structured, Language-Aware Messages (ff-message)

## Overview

`ff-message` is a new GUI-free core crate that adds a structured, language-aware
messaging model to FileForgeWorkbench. A program emits a STABLE identity (a
Message Class plus a Message Number), a SEVERITY, and a small set of named
PARAMETERS; the human-readable, localized STRING is produced only at the
presentation boundary (a log sink, a notification/toast, or a dialog). This is
the mainframe BAPIRET2 / `sy-msgid`/`msgty`/`msgno`/`msgv1..4` spirit: pass the
structured tuple around, render it to a `String` only at the edge.

The crate is deliberately SMALL and ADDITIVE. It reuses the existing `ff-i18n`
catalogue for ALL text resolution (it does not rebuild i18n), it renders to a
`String` and calls the EXISTING `ff_logging::log(...)` for the log boundary, and
it converts to the EXISTING `Notification` for the notification boundary. No
existing call site is forced to change; new and important messages opt in.

This design is written against the REAL signatures of `ff-i18n`
(`crates/ff-i18n/src`), `ff-logging` (`crates/ff-logging/src`), and the
notification module (`crates/ff-desktop/src/notification/mod.rs`), read during
authoring.

## Framework-conformance statement

Per `framework-conformance.md`, this layer is ADDITIVE and builds ON existing
seams:

- It introduces a NEW cross-cutting DATA TYPE (`StructuredMessage` and its parts).
  That is an owner-level decision and is CONFIRMED in the locked owner decisions
  for this gate.
- It changes NO core framework mechanism: no new command-dispatch path, no second
  navigation stack, no change to the `ff-logging` public API or `LogRecord`, no
  new session-persistence format, no per-Context focus mechanism, and no change to
  the `WorkspaceContext` / `InteriorFocus` contract.
- It does NOT reshape any load-bearing public framework type. It depends on
  `ff-i18n` and `ff-logging` as they are, and the notification conversion depends
  on `Notification` as it is.

The only "all logging in message format" idea -- making every `LogRecord` carry a
class+number+severity identity -- is explicitly a possible FUTURE owner-confirmed
framework change and is NOT part of this design.

## Crate boundary (the decision that anchors everything)

The sharp boundary between the two cooperating crates:

- `ff-i18n` = TEXT RESOLUTION. It owns the Fluent catalogue, locale selection,
  the English Identity_Base, named-argument substitution (`MessageArgs`), and the
  process-wide `t` / `t_args` seam. Its key is a FREE-FORM STRING and it carries
  NO severity and NO stable class+number identity.
- `ff-message` = STABLE IDENTITY + SEVERITY + PARAMS carried across the "one
  program calls another" boundary. It adds exactly the two things `ff-i18n`
  deliberately omits -- a stable `{class, number}` identity and a severity axis --
  and it RENDERS via `ff-i18n` at the edge. `ff-message::Severity` is the source
  of truth and maps DOWN onto both `ff_logging::LogLevel` and the notification
  `NotificationLevel`.

So `ff-message` depends on `ff-i18n`; `ff-i18n` never depends on `ff-message`.
The catalogue DATA for structured messages lives in `ff-i18n`'s `.ftl` files,
keyed by the `{class, number}` -> key scheme below.

## Technology stack (locked once approved)

- Language/crate: a new library crate `crates/ff-message` in the existing Cargo
  workspace, `edition = "2021"`.
- Dependencies: `ff-i18n` (path) for render; `ff-logging` (path) for the log
  adapter and for the crate's own WARN/diagnostics; `thiserror` (workspace) for
  the crate's library error type. NO `egui`, NO `ff-desktop`, NO `fluent` direct
  dependency (it flows through `ff-i18n`).
- The notification conversion (`StructuredMessage` -> `Notification`) lives in
  `ff-desktop` (where `Notification` is defined), so `ff-desktop` depends on
  `ff-message`, never the reverse. This keeps the core crate GUI-free (Req 9.1,
  Req 6.4).

## Concrete Rust types (the decisions the implementer must not re-make)

### Identity

```rust
// identity.rs
pub struct MessageClass(/* short uppercase ASCII */ String);
pub struct MessageNumber(u16); // invariant: value <= 9999 (fixed four-digit space)
pub struct MessageId { class: MessageClass, number: MessageNumber }
```

Decisions:

- `MessageClass` is a NEWTYPE over a validated `String` (not a bare `&str`) so an
  identity cannot be confused with arbitrary text (Rust standards: newtypes over
  primitives). Validation: non-empty, ASCII, uppercase, short (a documented bound,
  e.g. 2..=8 chars). Construction is FALLIBLE: `MessageClass::try_new(&str) ->
  Result<Self, MessageError>` (Req 1.5). `try_new` is the ONLY constructor, so
  there is no silent invalid state.
- `MessageNumber` is a NEWTYPE over `u16`, but BOUNDED to the four-digit space
  `0..=9999` so the rendered key is always a fixed four-digit width (the owner and
  review decision: identity space is `0000..=9999`, not `0..=65535`). Construction
  is therefore FALLIBLE: `MessageNumber::try_new(u16) -> Result<Self, MessageError>`
  rejects any value `> 9999` with `MessageError::NumberOutOfRange { value }`. The
  render width is the documented constant `KEY_NUMBER_WIDTH = 4`; because the value
  can never exceed `9999`, `{:04}` always yields exactly four digits (never wider).
  This makes the "fixed-width" guarantee in Req 2.1 and the Glossary true by
  construction rather than by convention.
- `MessageId` composes the two. It derives `Debug, Clone, PartialEq, Eq, Hash` so
  it can key a map and be compared. It is small and cheap to clone.

Illegal states are unrepresentable: there is no way to hold a `MessageClass`
whose string failed validation, because the only constructor validates.

### Severity and the behaviour contract

```rust
// severity.rs
pub enum Severity { Success, Info, Warning, Error, Abort, Fatal } // ordered

pub enum FlowEffect { Continue, ContinueWithAttention, StopOperation, AbortTask, Terminate }

impl Severity {
    pub fn flow_effect(self) -> FlowEffect { /* defined mapping */ }
    pub fn to_log_level(self) -> ff_logging::LogLevel { /* Req 4.1 */ }
    // notification mapping lives at the desktop boundary (see below)
}
```

Decisions:

- `Severity` has EXACTLY six variants in ascending seriousness and derives
  `PartialOrd, Ord` (declaration order gives the ordering) so a consumer can pick
  the most-severe message in a batch (Req 3.1, 3.2).
- The Behaviour_Contract is a FIRST-CLASS enum `FlowEffect` returned by
  `Severity::flow_effect`, NOT free prose (Req 3.3). Mapping: `Success`/`Info` ->
  `Continue`; `Warning` -> `ContinueWithAttention`; `Error` -> `StopOperation`;
  `Abort` -> `AbortTask`; `Fatal` -> `Terminate`. `FlowEffect` is DEFINED here but
  nothing in this gate reads it to change control flow (Req 3.4); Phase 2 batch
  code will.
- `to_log_level` (Req 4.1): `Success`/`Info` -> `Info`; `Warning` -> `Warn`;
  `Error`/`Abort`/`Fatal` -> `Error`. Total; defined in the core crate because
  `ff-logging` is a core dependency.
- The notification mapping (Req 4.2) is DEFINED but its function lives at the
  desktop boundary (next section) to keep the core crate free of a desktop
  dependency (Req 4.4).

### Parameters

The parameter carrier REUSES `ff_i18n::MessageArgs` directly (Req 1.4). The
design does NOT wrap it in a new type: `MessageArgs` is already `Debug + Clone`,
is the exact type `ff_i18n::t_args` consumes, and already encodes the
"verb-as-non-translated-argument" rule. A thin wrapper would add surface area for
no gain, so `StructuredMessage` holds a `MessageArgs` field directly.

### The structured message

```rust
// message.rs
pub struct StructuredMessage {
    id: MessageId,
    severity: Severity,
    args: ff_i18n::MessageArgs,
}
```

Decisions:

- Fields are private with accessors (`id()`, `severity()`, `args()`) per the Rust
  standards (prefer accessors over `pub` fields), giving read access for routing
  without rendering (Req 1.6).
- Constructors: `StructuredMessage::new(MessageId, Severity)` (no args) and a
  builder-style `with_args(mut self, MessageArgs) -> Self`, mirroring the
  `MessageArgs` builder shape already in `ff-i18n`.
- `Clone + Send + Sync`: `MessageId` is `Clone`, `Severity` is `Copy`, and
  `MessageArgs` derives `Clone` while being AUTO-`Send + Sync` (the `Send`/`Sync`
  auto-traits are inferred from its owned `String`/`f64`-backed fields, NOT an
  explicit or manual impl). `StructuredMessage` therefore derives `Clone` and
  gets `Send + Sync` AUTOMATICALLY from its fields -- the implementer must NOT
  write a manual or `unsafe` `Send`/`Sync` impl. So `StructuredMessage` is
  `Clone + Send + Sync` and cheap to pass across the "one program calls another"
  boundary (Req 1.3, 9.2).

## The {class, number} -> ff-i18n key convention

Single, documented, deterministic scheme (Req 2.1, 2.2):

```
key = format!("{}-{:0width$}", class.to_ascii_lowercase(), number, width = 4)
// MessageId { class: "VFS", number: 7 }  ->  "vfs-0007"
// MessageId { class: "CMD", number: 12 } ->  "cmd-0012"
```

The width-4 zero-pad is a documented constant `KEY_NUMBER_WIDTH`. Because a
`MessageNumber` is bounded to `0..=9999` at construction (see Identity above),
`{:04}` always produces exactly four digits -- the key width is genuinely fixed,
not merely a minimum. The function `MessageId::catalogue_key(&self) -> String` is
pure and total (Req 2.2). It lives on `MessageId` so identity and key-derivation
stay together.

Rationale for a convention over a runtime registry: the identity is already a
typed `MessageId`; the catalogue is `ff-i18n` `.ftl` data keyed by exactly this
string. A heavy runtime registry would duplicate what the catalogue file already
is. Number-space reservation is therefore a LIGHTWEIGHT DOCUMENTED CONVENTION: a
registry NOTE (a Markdown table in this spec and a header comment block in the
`.ftl` data) listing each reserved `MessageClass` and its allocated number ranges
(Req 8.2). The authoring-time convention is the authority that prevents a
`{class, number}` collision (Req 8.3); `ff-message` never merges or overwrites
catalogue entries.

If, in the future, dynamic class registration (e.g. plugin-authored classes) is
needed, a runtime registry can be ADDED then; it is intentionally out of this
gate because nothing here requires it.

## Render API and how it calls ff-i18n

```rust
// render.rs
impl StructuredMessage {
    pub fn render(&self) -> String {
        ff_i18n::t_args(&self.id.catalogue_key(), &self.args)
    }
    pub fn render_title(&self) -> String { self.render() } // deliberate alias (see rationale)
    pub fn render_detail(&self) -> Option<String> { /* see below */ }
}
```

Decisions:

- `render` maps the id to the key and calls `ff_i18n::t_args(key, args)` (Req
  2.3). The active-locale string is produced at call time, so the SAME structured
  message rendered under a different locale yields that locale's text (the core
  language-aware behaviour).
- Missing-entry fallback is `ff-i18n`'s existing behaviour: active locale ->
  English Identity_Base -> the key itself with a WARN (Req 2.4). `ff-message` adds
  NO second fallback.
- Title vs detail (Req 2.5): the simplest, catalogue-driven decision is that
  `render_title` renders the primary key, and `render_detail` renders an OPTIONAL
  companion key `"<key>.detail"` (e.g. `vfs-0007.detail`) ONLY when that key is
  present. To avoid the key-echo fallback masquerading as real detail text,
  `render_detail` first checks presence: it renders the detail key and returns
  `None` when the result equals the detail key string (i.e. the entry is absent).
  This keeps a one-line message detail-free and lets a message opt into a detail
  line purely by adding a `.detail` catalogue entry -- no code change. (This
  presence-by-echo check is the one place the design leans on the key-echo
  contract; it is documented and unit-tested.) KNOWN false-negative: a `.detail`
  entry whose rendered text is byte-identical to its own key (e.g.
  `vfs-0007.detail = vfs-0007.detail`) is indistinguishable from an absent entry
  and would be reported as `None`. This is a pathological authoring case; authors
  MUST NOT author such an entry, and the planned `render_detail` unit test
  documents this limit in its doc comment so the behaviour is intentional.
- `render_title` is a DELIBERATE exact alias of `render`: the primary key IS the
  title, so a message's one-line form and its title are the same string. There is
  intentionally NO `<key>.title` companion key (unlike the optional `<key>.detail`);
  a message carries one primary line plus an optional detail, not a separate title
  entry. This is a documented decision, not an oversight.
- Verbs stay English: any verb/identifier is already a non-translated
  `MessageArgs` value supplied by the caller (Req 2.6, 6-localization). `ff-message`
  never inspects or translates argument values.

## Adapters

### Log adapter (core crate)

```rust
// adapters/log.rs
pub fn log_message(msg: &StructuredMessage, module_path: &str) {
    ff_logging::log(msg.severity().to_log_level(), module_path, &msg.render());
}
```

- Renders once, maps severity to `LogLevel` (Req 4.1), calls the EXISTING
  `ff_logging::log(level, module_path, &str)` (Req 5.1). No change to
  `ff-logging`, `LogRecord`, or logging behaviour (Req 5.2).
- Because it adds no logic beyond render-then-log, the not-initialised / fallback
  behaviour is identical to a direct `ff_logging::log` call today (Req 5.3), and it
  renders exactly once (Req 5.4).

### Notification adapter (desktop boundary)

Lives in `crates/ff-desktop/src/notification/` (a small addition in a LATER,
separately tracked step, since it touches `ff-desktop`; this gate DESIGNS it and
lists it as a task but the core crate does not depend on desktop):

```rust
// in ff-desktop, depends on ff-message
fn notification_level_for(sev: ff_message::Severity) -> NotificationLevel {
    match sev {
        Severity::Success => NotificationLevel::Success,
        Severity::Info    => NotificationLevel::Info,
        Severity::Warning => NotificationLevel::Warning,
        Severity::Error | Severity::Abort | Severity::Fatal => NotificationLevel::Error,
    }
}

impl Notification {
    pub fn from_structured(msg: &ff_message::StructuredMessage) -> Self {
        Notification::new(notification_level_for(msg.severity()),
                          msg.render_title(), msg.render_detail())
    }
}
```

- Sets level from the severity-to-NotificationLevel mapping (Req 4.2), `title`
  from `render_title`, `detail` from `render_detail` (Req 6.1). Produces a plain
  `Notification` accepted by the existing `NotificationSender` / `NotificationQueue`
  unchanged (Req 6.2), preserving the non-blocking drop-if-full semantics (Req 6.3).
- Dependency direction is desktop -> ff-message only (Req 4.4, 6.4). This is why
  the mapping function physically lives in `ff-desktop`: naming `NotificationLevel`
  in the core crate would pull a desktop dependency into a GUI-free core.

## Optional MessageId on thiserror enums

```rust
// has_id.rs
pub trait HasMessageId {
    fn message_id(&self) -> Option<MessageId>;
}
```

- A new OPTIONAL trait (Req 7.1). An error type that does not implement it is
  unaffected; implementing it does NOT touch the type's `Display` /
  `#[error(...)]` output (Req 7.2).
- This gate only DEFINES the trait. Adopting it on `VfsError` / `CommandError`
  (mapping each variant to a reserved `{class, number}`) is a later, separately
  gated step IN THOSE crates' specs (Req 7.3); no source enum is modified here.
- The payoff (Req 7.4, localization Req 9.3): a display-boundary consumer takes the
  error's `message_id()`, builds a `StructuredMessage` (supplying the error's
  context as `MessageArgs`), and renders it -- localizing error text at the edge
  rather than at the error source. The known pre-existing non-ASCII em-dash in
  two `CommandError` `#[error(...)]` strings (variants `InvalidId` and
  `HistoryIo`) is a SOURCE issue and is NOT fixed here (`VfsError`'s
  `#[error(...)]` strings are plain ASCII).

## Catalogue-entry conventions

- Structured-message texts are ordinary `ff-i18n` `.ftl` entries in the English
  Identity_Base (`crates/ff-i18n/i18n/en/`) and per-locale overlays, keyed by the
  `{class, number}` scheme (Req 8.1). A dedicated file per class is the recommended
  organisation, e.g. `crates/ff-i18n/i18n/en/messages-vfs.ftl` containing
  `vfs-0007 = ...` (and optional `vfs-0007.detail = ...`). Example entry:

  ```
  # messages-vfs.ftl (English Identity_Base)
  vfs-0007 = Resource not found: { $uri }
  vfs-0007.detail = The { $operation } operation could not locate { $uri }.
  ```

  This gate creates ONLY `crates/ff-i18n/i18n/en/messages-msg.ftl` (the `MSG`
  class, exercised by the render unit tests -- see the seed-registry table below
  and Task 9.1). The `messages-vfs.ftl` / `vfs-0007` example above is ILLUSTRATIVE
  of the convention, NOT a file created in this gate; `VFS`/`CMD` are reserved
  placeholders for their later, separately gated adoption.

- The reserved-number registry is a LIGHTWEIGHT NOTE: a Markdown table in this
  spec plus a header comment in each `messages-<class>.ftl` file listing the class
  and its allocated number ranges (Req 8.2). It is the authoring-time authority
  against collisions (Req 8.3).
- Missing key -> `ff-i18n` key-echo + WARN (Req 8.4). Placeables come from
  `MessageArgs`; an embedded verb is a non-translated argument (Req 8.5).

### Initial reserved classes (seed registry note)

| Message_Class | Owner / domain | Reserved number range | Notes |
|---------------|----------------|-----------------------|-------|
| `MSG` | ff-message self-test / examples | 0001-0099 | Proves the mechanism in unit tests. |
| `VFS` | ff-vfs (future adoption) | 0001-0999 | Reserved for the eventual `VfsError` adoption (separate gate). |
| `CMD` | ff-command (future adoption) | 0001-0999 | Reserved for the eventual `CommandError` adoption (separate gate). |

Only `MSG` is EXERCISED by this gate (unit-test catalogue entries). `VFS`/`CMD`
are reserved placeholders so a later adoption does not collide.

## Module layout (respecting the 400-line rule)

```
crates/ff-message/
  Cargo.toml
  src/
    lib.rs            -- crate docs + re-exports only (thin)
    error.rs          -- MessageError (thiserror); identity validation errors
    identity.rs       -- MessageClass, MessageNumber, MessageId, catalogue_key
    severity.rs       -- Severity, FlowEffect, to_log_level, flow_effect
    message.rs        -- StructuredMessage + constructors/accessors
    render.rs         -- render / render_title / render_detail (calls ff-i18n)
    has_id.rs         -- HasMessageId trait
    adapters/
      mod.rs          -- re-exports
      log.rs          -- log_message (calls ff_logging::log)
    tests.rs          -- unit tests (split further if > ~200 lines)
```

- `lib.rs` is a thin coordinator (re-exports, crate docs) per the Rust standards.
- Each file is single-concern and well under 400 non-test lines. The notification
  conversion is NOT in this crate; it is the small `ff-desktop` addition described
  above.
- `MessageError` is a `thiserror` enum (e.g. `InvalidClass { value, reason }` and
  `NumberOutOfRange { value }`) per the library-error standard; no
  `unwrap`/`expect`/`panic!` in library code (Req 9.3).

## Error handling (concrete, per operation)

- `MessageClass::try_new`: FAILS on empty, non-ASCII, non-uppercase,
  or over-length input. Returns `Err(MessageError::InvalidClass { value, reason })`.
  Recoverable; the caller receives the error. Not logged by the constructor (the
  caller decides). This and `MessageNumber::try_new` are the two fallible identity
  constructors.
- `MessageNumber::try_new`: FAILS on a value `> 9999`, returning
  `Err(MessageError::NumberOutOfRange { value })`. Recoverable; the caller receives
  the error. Not logged by the constructor. This bound keeps every rendered key at
  the fixed four-digit width (Req 1.5, Req 2.1).
- `MessageId::catalogue_key`: INFALLIBLE and total; a valid id always yields a key.
- `render` / `render_title`: INFALLIBLE; a missing catalogue entry degrades to the
  `ff-i18n` key-echo + WARN (logged by `ff-i18n`, at WARN). Never panics, never
  blank.
- `render_detail`: INFALLIBLE; returns `None` when the `.detail` key is absent
  (detected by the key-echo equality check). Never panics. Documented
  false-negative: a `.detail` entry whose text equals its own key reads as absent
  (authors must not author one).
- `log_message`: INFALLIBLE; delegates entirely to `ff_logging::log`, whose own
  not-initialised/fallback/overflow handling is unchanged and non-panicking.
- Notification conversion (desktop): INFALLIBLE; delegates to the existing
  `Notification::new` and the non-blocking `try_send`, which drops if full.

## Input validation (per external input)

- `MessageClass` string: REQUIRED; type `&str`; limits non-empty, ASCII, uppercase,
  2..=8 chars; behaviour on failure = `MessageError::InvalidClass`.
- `MessageNumber` value: REQUIRED; type `u16`; limit `0..=9999` (the fixed
  four-digit identity space); behaviour on failure = `MessageError::NumberOutOfRange
  { value }`. A value `> 9999` is rejected so no key ever renders wider than four
  digits.
- `MessageArgs`: OPTIONAL; defaulting to empty; values are owned `String`/`f64`;
  never validated or translated by `ff-message`.

## Invariant ownership

- "A `MessageClass` is always valid": OWNED by `MessageClass`'s fallible
  constructor (the only way to make one). No other layer can bypass it.
- "A `MessageNumber` is always in `0..=9999` (fixed-width key)": OWNED by
  `MessageNumber`'s fallible `try_new` (the only way to make one). This is what
  makes `catalogue_key` width genuinely fixed rather than a minimum.
- "A `MessageId` always maps to a key": OWNED by `MessageId::catalogue_key`
  (pure/total). The catalogue data's EXISTENCE for that key is NOT an `ff-message`
  invariant -- it is the authoring-time registry-note convention plus `ff-i18n`'s
  fallback.
- "Severity is the single source of truth and maps down, never up": OWNED by
  `Severity::to_log_level` and the desktop `notification_level_for`; both are total
  and additive, so neither `LogLevel` nor `NotificationLevel` is altered.

## Testability

Unit-testable WITHOUT egui (the core crate is GUI-free, Req 9.1):

- Identity: `MessageClass::try_new` accept/reject cases; `MessageNumber::try_new`
  accepting `0..=9999` and rejecting `> 9999` with `NumberOutOfRange`;
  `catalogue_key` formatting (`vfs-0007`), including the fixed four-digit zero-pad
  (assert `9999` renders `9999`, not wider) and lowercasing.
- Severity: ordering; `to_log_level` all six variants; `flow_effect` all six
  variants; the documented collapse of Abort/Fatal into UI Error (asserted via the
  notification mapping values).
- Render: install an English Identity_Base catalogue in-test (the
  `ff_i18n::Catalogue::load_identity_base` + `set_catalogue` pattern used by
  `ff-catalog-dialog` tests), then assert `render` returns the catalogue text for a
  present key and the key string for an absent key; assert `render_detail` returns
  `Some` for a present `.detail` key and `None` for an absent one; assert
  argument interpolation (a `{ $uri }` placeable) and that an embedded verb passed
  as an argument is NOT translated.
- Test isolation note (process-wide seam): `ff_i18n::set_catalogue` is a GLOBAL,
  swappable handle, and `cargo test` runs tests in parallel by default. The render
  tests all install the SAME shipped English Identity_Base (exactly the catalogue
  the existing `ff-catalog-dialog` tests install, which that code documents as
  "idempotent and safe to call from multiple tests"), so concurrent installation
  is benign. If a FUTURE test installs a DIFFERENT catalogue (e.g. a non-English
  overlay to prove locale switching), that test MUST serialise seam access behind a
  shared test mutex rather than relying on the global handle, so it cannot race the
  English-base render tests.
- Log adapter: assertable by construction/mapping (the `to_log_level` mapping is
  the testable surface); the `ff_logging::log` side effect is covered by the
  logging crate's own tests. Mapping tests suffice here.
- Notification conversion (desktop test, in `ff-desktop`): `from_structured` sets
  the mapped level, title, and detail; Abort/Fatal collapse to `Error`.

Integration-tested: the notification path in `ff-desktop` through the existing
`NotificationSender` / `NotificationQueue` is covered by the notification module's
tests once the conversion lands there.

A design note on testability: because render is a thin call into `ff-i18n` and the
log adapter is a thin call into `ff-logging`, the crate's logic that NEEDS testing
is concentrated in pure functions (key derivation, severity mappings, detail
presence), which is exactly what keeps it easy to test.

## Cross-spec relationships

- `localization` (CR-NR-103): `ff-message` DEPENDS ON `ff-i18n` and is the
  concrete realization of localization Req 9.3 (route error/structured text through
  the seam at the display boundary). A short cross-reference note is added to the
  localization design pointing here. PROPOSED future cross-spec change: adding
  `messages-<class>.ftl` data files and a per-locale overlay for structured-message
  text is catalogue DATA growth under localization's incremental extraction; it is
  marked PROPOSED because the content beyond the English seed is future work.
- `logging-subsystem`: the sink is UNTOUCHED. `ff-message` renders and calls the
  existing `ff_logging::log`. A short cross-reference note is added to the logging
  design stating the additive boundary-render integration and that the public API /
  `LogRecord` do not change. PROPOSED future cross-spec change (explicitly NOT in
  this gate): the heavier "every log record carries a structured identity" model,
  which would be an owner-confirmed framework change to the Wave-1 logger.
- `notification-system`: the FIRST consumer. A short cross-reference note is added
  to the notification design describing the `Notification::from_structured`
  constructor path and the severity-to-NotificationLevel mapping. This is additive;
  the `Notification` / queue / sender signatures do not change.
- `batch-execution`: the PHASE 2 consumer that will HONOUR the `FlowEffect`
  contract (Abort stops a batch; Fatal terminates). PROPOSED and deferred; no
  change is made to that spec here beyond this design naming the relationship.

Per `specs.md`, `docs/specs/structured-messages` SHOULD be added to the specs
registry / roadmap. This design NOTES that; it does not edit management docs.

## Decisions summary (so nothing is left to the implementer)

1. Identity is a validated `MessageClass` newtype + a `MessageNumber(u16)` newtype
   bounded to `0..=9999` + a composing `MessageId`; both newtype constructors are
   fallible (`MessageClass::try_new` validates the string, `MessageNumber::try_new`
   rejects `> 9999`), so no invalid identity and no over-wide key can exist.
2. Key scheme is `format!("{}-{:04}", class.to_lowercase(), number)` via
   `MessageId::catalogue_key`; pure and total.
3. Severity is the six-variant ordered enum, source of truth; `FlowEffect` is a
   first-class enum (defined, not enforced this gate); `to_log_level` lives in the
   core crate; `notification_level_for` lives at the desktop boundary.
4. Parameters REUSE `ff_i18n::MessageArgs` directly (no wrapper).
5. Render calls `ff_i18n::t_args(key, args)`; detail is an optional `<key>.detail`
   entry detected by the key-echo equality check; missing -> `ff-i18n` fallback.
6. Log adapter in-core renders then calls `ff_logging::log`; notification
   conversion in `ff-desktop` builds a plain `Notification`.
7. `HasMessageId` is an optional additive trait; no source enum changes this gate.
8. Catalogue data lives in `ff-i18n` `.ftl` files; number reservation is a
   documented registry note, not a runtime registry.

## Responses to design review (iteration 2)

Review: `.agents/tasks/ff-message-investigation/design-review.md` /
`design-review.json`, verdict CHANGES_REQUESTED (1 MEDIUM, 4 NITs). Every finding
is addressed below; none required changing a locked owner decision or a core
framework mechanism.

- Finding 1 (MEDIUM -- `MessageNumber` width self-contradictory): ADDRESSED via the
  review's recommended Option A. `MessageNumber` is now BOUNDED to `0..=9999` and
  its constructor is FALLIBLE (`MessageNumber::try_new` rejecting `> 9999` with a
  new `MessageError::NumberOutOfRange { value }`). The "fixed four-digit width"
  guarantee is now true by construction: because no value exceeds `9999`, `{:04}`
  always yields exactly four digits. All three docs were reconciled -- requirements
  Req 1.1, Req 1.5, Req 2.1 and the Glossary now state the bound; the design's
  Identity, key-scheme, Error-handling, Input-validation, Invariant-ownership,
  Testability, and Decisions-summary sections agree; tasks 2 and 3 now test and
  implement the fallible `try_new` and the `NumberOutOfRange` variant.
- Finding 2 (NIT -- `render_detail` key-echo false-negative): ADDRESSED. The Render
  API and Error-handling sections now state the known false-negative (a `.detail`
  entry whose rendered text is byte-identical to its own key reads as absent;
  authors must not author one), and Task 6.1 records it in the unit-test doc
  comment so the behaviour is intentional.
- Finding 3 (NIT -- `render_title == render` redundancy): ADDRESSED. The Render API
  section now carries the rationale: the primary key IS the title, there is
  deliberately no `<key>.title` companion, so a message's one-line form and its
  title are the same string.
- Finding 4 (NIT -- test isolation against the process-wide i18n seam): ADDRESSED.
  Testability now states that render tests install the SAME shipped English
  Identity_Base (the idempotent `ff-catalog-dialog` pattern), so parallel install
  is benign, and that any future per-locale render test must serialise seam access
  behind a shared test mutex.
- Finding 5 (NIT -- single seed `.ftl` file): ADDRESSED. The Catalogue-entry
  conventions section now states explicitly that this gate creates ONLY
  `crates/ff-i18n/i18n/en/messages-msg.ftl` (MSG class), and that the
  `messages-vfs.ftl` / `vfs-0007` example is illustrative of the convention, not a
  file created here.
