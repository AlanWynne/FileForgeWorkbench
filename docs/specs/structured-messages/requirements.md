# Requirements Document -- Structured, Language-Aware Messages (ff-message)

## Introduction

This sub-project defines a STRUCTURED, LANGUAGE-AWARE messaging model for
FileForgeWorkbench, inspired by the mainframe pattern in which a program emits a
STABLE message IDENTITY plus a SEVERITY plus a small set of PARAMETERS, and the
human-readable, localized STRING is resolved only at the presentation boundary
(a log sink, a notification/toast, or a dialog). The structured value -- identity
plus severity plus parameters -- is what travels across the "one program calls
another" boundary and what gets stored or passed around; it is rendered to a
`String` only at the edge.

The deliverable is a NEW core crate `ff-message`. It is an ADDITIVE layer, not a
rewrite. It DEPENDS ON the existing `ff-i18n` crate for text resolution (do not
rebuild i18n) and renders structured messages to a `String` at the boundary,
feeding the EXISTING `ff-logging` public API UNCHANGED and the EXISTING
notification seam. Existing plain-string call sites keep working; new and
important messages opt in.

The investigation that motivated this gate is
`.agents/tasks/ff-message-investigation/findings.md`. It confirms that the
text-resolution half (catalogue, locale, named args, render-at-boundary) already
exists as `ff-i18n`, while the stable identity (class + number) and the
behaviour-bearing severity axis do NOT exist anywhere today.

### Locked owner decisions (design to these; do not re-litigate)

1. ADDITIVE layer, not a rewrite. `ff-message` is a new crate that depends on
   `ff-i18n` and renders at the log/notification/dialog boundary. The
   `ff-logging` public API and `LogRecord` are UNTOUCHED. The heavier "all
   logging moves to message format" change is OUT OF SCOPE for this gate (a
   possible future owner-confirmed framework change, not designed here).
2. Message IDENTITY is a stable Message Class + Message Number (e.g. class `VFS`,
   number `007`). The identity maps to an `ff-i18n` catalogue key so the TEXT is
   localizable: opening or rendering under a different locale yields the message
   in that language automatically. Parameters flow through the EXISTING
   `ff_i18n::MessageArgs` (do not invent a second parameter model). FFWB's rule
   that command verbs and identifiers stay English and are passed as
   non-translated arguments (localization Req 4) is preserved.
3. SEVERITY is a 6-level axis owned by `ff-message` as the source of truth:
   Success, Info, Warning, Error, Abort, Fatal (modeled on the mainframe
   S/I/W/E/A/X ladder). It MAPS DOWN onto `NotificationLevel` (4 levels) at the
   dialog/toast boundary and onto `ff_logging::LogLevel` (5 levels) at the log
   boundary; both mappings are purely additive. The BEHAVIOURAL MEANING of
   severity (continue / stop the current operation / abort the whole task-or-batch
   / terminate) is DEFINED here, but actually HONOURING that behaviour in
   batch/background/Command-Environment execution is a DISTINCT, LATER phase
   (scoped separately so defining the axis does not silently rewrite batch
   semantics). Dialog/interactive surfaces collapse Abort/Fatal visually into
   Error; background/batch surfaces use the full axis.

### Cross-references

- `localization` (CR-NR-103) -- the `ff-i18n` foundation (`Catalogue`,
  `set_catalogue`, `t` / `t_args`, `MessageArgs`, `Locale`, `IDENTITY_BASE_TAG`).
  `ff-message` DEPENDS ON this crate for all text resolution and realizes
  localization Req 9.3 (route `thiserror`-originated text through the seam at the
  display boundary).
- `logging-subsystem` -- the untouched sink. `ff-message` renders to a `String`
  and calls the EXISTING `ff_logging::log(level, module, &rendered)`. No change
  to the logging public API or `LogRecord`.
- `notification-system` -- the first consumer. `ff-message` adds a
  structured-message constructor path that renders `title` / `detail` and maps
  severity onto `NotificationLevel`.
- `batch-execution` -- the Phase 2 consumer that will HONOUR severity behaviour
  (Abort stops a batch run; Fatal terminates). Deferred; see Out of Scope.
- `framework-conformance` -- this layer introduces a new cross-cutting data type
  (owner-confirmed) but changes NO core framework mechanism.

## Glossary

| Term | Definition |
|------|-----------|
| **Message_Class** | A short, stable, uppercase ASCII identifier naming the subsystem/domain that owns a message family (e.g. `VFS`, `CMD`). The SE91-analogue message class. |
| **Message_Number** | A stable numeric identifier within a class, bounded to the fixed four-digit space `0..=9999` and rendered zero-padded to that fixed width (e.g. `0007`). Numbers are reserved per class so they do not collide. |
| **Message_Id** | The `{class, number}` tuple: the STABLE identity carried across boundaries and mapped to an `ff-i18n` catalogue key. |
| **Severity** | The 6-level axis owned by `ff-message`: Success, Info, Warning, Error, Abort, Fatal. The source of truth for message severity. |
| **Behaviour_Contract** | The DEFINED (not yet enforced) meaning of each severity for flow control: continue / stop current operation / abort task-or-batch / terminate. |
| **Structured_Message** | The value `{ id, severity, args }` passed around and rendered to a `String` only at the presentation boundary. |
| **Catalogue_Key** | The `ff-i18n` message key a Message_Id maps to, via a documented string scheme (e.g. `vfs-0007`). |
| **Render_Boundary** | The point (log sink, notification, dialog) where a Structured_Message is resolved to a localized `String` via `ff-i18n`. |

---

## Requirements

### Requirement 1: Structured-message data model

**User Story:** As a subsystem author, I want to emit a stable message identity
with a severity and parameters, so that the human text is resolved later at the
edge rather than baked in at the call site.

#### Acceptance Criteria

1. THE `ff-message` crate SHALL define a `MessageClass` newtype wrapping a short
   uppercase ASCII identifier and a `MessageNumber` newtype wrapping a numeric
   identifier bounded to the four-digit space `0..=9999`, so a message identity
   cannot be confused with an arbitrary string or integer and the rendered key
   width is fixed.
2. THE `ff-message` crate SHALL define a `MessageId` type composed of a
   `MessageClass` and a `MessageNumber`, which is the STABLE identity carried
   across boundaries.
3. THE `ff-message` crate SHALL define a `StructuredMessage` type composed of a
   `MessageId`, a `Severity`, and a parameter carrier, and it SHALL be
   `Clone + Send + Sync` and cheap to pass by value or reference.
4. THE parameter carrier SHALL REUSE `ff_i18n::MessageArgs` (wrapped or held
   directly) rather than introducing a second parameter/substitution model, so
   named placeable arguments flow through the existing i18n mechanism.
5. WHEN a `MessageClass` is constructed from a string that is not short, uppercase
   ASCII, THE constructor SHALL reject it with a recoverable error rather than
   producing an invalid identity; AND WHEN a `MessageNumber` is constructed from a
   value greater than `9999`, THE constructor SHALL reject it with a recoverable
   error so every valid identity maps to a fixed-width four-digit key.
6. THE `StructuredMessage` type SHALL expose read access to its `MessageId`,
   `Severity`, and arguments so a consumer (log adapter, notification adapter,
   dialog) can route on identity and severity without rendering text.

### Requirement 2: Class+number to ff-i18n key mapping and localized render

**User Story:** As a user, I want a message to appear in my selected language
automatically, keyed by its stable identity.

#### Acceptance Criteria

1. THE `ff-message` crate SHALL map a `MessageId { class, number }` to an
   `ff-i18n` Catalogue_Key via a single documented, deterministic string scheme:
   the lowercased class, a hyphen, then the number zero-padded to the fixed width
   of four digits (for example `MessageId { VFS, 7 }` maps to `vfs-0007`). Because
   a `MessageNumber` is bounded to `0..=9999` (Req 1.1, Req 1.5), every key the
   scheme produces is exactly four digits wide.
2. THE key-mapping function SHALL be pure and total (it SHALL produce a key for
   any valid `MessageId`), so the same identity always resolves to the same key.
3. WHEN a `StructuredMessage` is rendered, THE crate SHALL resolve its text by
   calling `ff_i18n::t_args(key, args)` with the mapped Catalogue_Key and the
   message's arguments, so the active-locale string is produced at the boundary.
4. WHEN the mapped Catalogue_Key is absent in both the active locale and the
   English Identity_Base, THE render SHALL return the `ff-i18n` key-echo fallback
   (the key string) and `ff-i18n` SHALL log its existing WARN, so a missing
   catalogue entry is diagnosable and never blank. `ff-message` SHALL add no
   second fallback mechanism.
5. THE crate SHALL expose a `render(&self) -> String` method producing the single
   localized line, and SHALL ALSO expose `render_title(&self) -> String` and
   `render_detail(&self) -> Option<String>` for notification/dialog consumers that
   need a separate title and detail.
6. THE render path SHALL keep command verbs and stable identifiers English by
   passing them as non-translated `MessageArgs` values (localization Req 4.2, Req
   9.2); `ff-message` SHALL NOT translate any argument value.

### Requirement 3: The six-level severity axis and its behavioural meaning

**User Story:** As a background/batch author, I want a severity axis richer than
the UI's four levels, with a defined meaning for program flow.

#### Acceptance Criteria

1. THE `ff-message` crate SHALL define a `Severity` enum with exactly six
   variants in ascending order of seriousness: `Success`, `Info`, `Warning`,
   `Error`, `Abort`, `Fatal`, and this enum SHALL be the source of truth for
   message severity.
2. THE `Severity` enum SHALL be totally ordered so consumers can compare and find
   the most-severe message in a collection.
3. THE crate SHALL DEFINE a Behaviour_Contract documenting the intended flow-control
   meaning of each severity: `Success`/`Info` continue; `Warning` continues with
   attention; `Error` stops the current operation; `Abort` aborts the whole
   task-or-batch; `Fatal` terminates. This definition SHALL be expressed as a
   documented method or associated enum on `Severity` (not free prose only).
4. THE Behaviour_Contract SHALL be DECLARATIVE ONLY in this gate: defining it
   SHALL NOT change the control flow of any existing batch, background, or
   Command-Environment execution path. Enforcement is Phase 2 (Out of Scope here).
5. THE crate SHALL document that dialog/interactive surfaces collapse `Abort` and
   `Fatal` visually into the UI `Error` level (four levels suffice at the UI),
   while background/batch surfaces use the full six-level axis.

### Requirement 4: Additive severity-to-LogLevel and severity-to-NotificationLevel mappings

**User Story:** As a maintainer, I want the six-level axis to map cleanly onto the
existing logging and notification levels without changing either.

#### Acceptance Criteria

1. THE `Severity` type SHALL provide a total function to `ff_logging::LogLevel`
   with this mapping: `Success`/`Info` -> `Info`; `Warning` -> `Warn`; `Error`,
   `Abort`, `Fatal` -> `Error`.
2. THE `Severity` type SHALL provide a total function to the notification severity
   (`NotificationLevel`, 4 levels) with this mapping: `Success` -> `Success`;
   `Info` -> `Info`; `Warning` -> `Warning`; `Error`, `Abort`, `Fatal` -> `Error`.
3. BOTH mappings SHALL be purely additive: they SHALL NOT require any change to
   `ff_logging::LogLevel`, to `NotificationLevel`, or to the existing variants or
   behaviour of either.
4. WHERE `ff-message` cannot name `NotificationLevel` directly without creating a
   dependency cycle or an unwanted dependency, THE design SHALL define the
   notification mapping at the consuming boundary (the notification adapter) so the
   core crate stays free of a GUI/desktop dependency; the mapping values in 4.2
   SHALL hold regardless of where the function physically lives.

### Requirement 5: Log adapter

**User Story:** As a subsystem author, I want to log a structured message with one
call that renders the localized text and routes to the existing logger.

#### Acceptance Criteria

1. THE crate SHALL provide a log adapter that, given a `StructuredMessage` and a
   module path, renders the message to a `String` and calls the EXISTING
   `ff_logging::log(level, module_path, &rendered)` with the level obtained from
   the severity-to-LogLevel mapping (Req 4.1).
2. THE log adapter SHALL NOT change the `ff-logging` public API, `LogRecord`, or
   any logging behaviour; it SHALL only call the existing entry point.
3. WHEN the logging subsystem is not initialised or is in fallback, THE adapter
   SHALL behave exactly as a direct `ff_logging::log` call does today (no panic,
   no new error path), because it adds no logic beyond render-then-log.
4. THE adapter SHALL render exactly once per log call, so the localized text in
   the log reflects the active locale at log time.

### Requirement 6: Notification adapter (first consumer)

**User Story:** As a subsystem author, I want to raise a structured message as a
toast/event-log notification with the right level and localized title/detail.

#### Acceptance Criteria

1. THE design SHALL define a conversion from a `StructuredMessage` to the existing
   `Notification` (`notification-system`) that sets the notification level from
   the severity-to-NotificationLevel mapping (Req 4.2), the `title` from
   `render_title`, and the `detail` from `render_detail`.
2. THE conversion SHALL produce a value accepted by the existing
   `NotificationSender` / `NotificationQueue` without changing their signatures or
   behaviour.
3. THE conversion SHALL preserve the existing non-blocking, drop-if-full semantics
   of `NotificationSender` (notification-system Req 3.3); it SHALL add no blocking
   behaviour.
4. WHERE the conversion lives in the `ff-desktop` notification module (because
   `Notification` is defined there), THE `ff-message` core crate SHALL remain free
   of a dependency on `ff-desktop`; the adapter direction is
   desktop-depends-on-ff-message, never the reverse.

### Requirement 7: Optional MessageId on thiserror error enums

**User Story:** As a subsystem author, I want cross-boundary errors to carry a
stable message identity without changing their current `Display` output.

#### Acceptance Criteria

1. THE `ff-message` crate SHALL define a trait (e.g. `HasMessageId`) with a method
   returning an `Option<MessageId>` (or a `MessageId`), so an error type can expose
   a stable identity additively.
2. THE trait SHALL be OPTIONAL to implement: an error type that does not implement
   it continues to work exactly as today, and implementing it SHALL NOT change the
   type's `Display` / `thiserror` `#[error(...)]` output.
3. WHERE `VfsError` or `CommandError` adopt the trait (a later, separately gated
   step in their own specs), the adoption SHALL map each variant to a reserved
   `{class, number}` without altering the variant's fields or `Display`. This gate
   SHALL NOT modify those source enums; it only DEFINES the trait they can adopt.
4. THE trait SHALL enable a consumer to obtain the localized text for an error at
   the DISPLAY boundary by taking the error's `MessageId`, building a
   `StructuredMessage`, and rendering it, realizing localization Req 9.3 without
   translating at the error source.

### Requirement 8: Catalogue-entry conventions and missing-entry fallback

**User Story:** As a translator/maintainer, I want structured-message text to live
as ordinary `ff-i18n` catalogue data with a predictable key convention, and a
safe fallback when an entry is missing.

#### Acceptance Criteria

1. THE structured-message texts SHALL live as ordinary `ff-i18n` `.ftl` catalogue
   entries in the English Identity_Base (and per-locale overlays), keyed by the
   `{class, number}` -> key scheme of Req 2.1 (e.g. `vfs-0007 = ...`), so adding or
   translating a message is a catalogue-data edit, not a code change.
2. THE key convention SHALL reserve the per-class number space by a LIGHTWEIGHT
   CONVENTION (a documented registry note listing each reserved `MessageClass` and
   its allocated number ranges), not a heavy runtime registry, unless a runtime
   registry is justified in the design.
3. WHEN two messages would collide on the same `{class, number}`, THE convention
   (registry note) SHALL be the authority that prevents the collision at authoring
   time; `ff-message` SHALL NOT silently merge or overwrite catalogue entries.
4. WHEN a structured message's Catalogue_Key is missing, THE render SHALL use the
   `ff-i18n` key-echo fallback with its existing WARN (Req 2.4); no catalogue entry
   is fabricated and no panic occurs.
5. THE catalogue entries for structured messages MAY carry named placeables
   (`{ $arg }`) populated from the message's `MessageArgs`, and an embedded command
   verb SHALL be a non-translated argument (localization Req 9.2).

### Requirement 9: Non-functional requirements

#### Acceptance Criteria

1. THE `ff-message` core crate SHALL be GUI-free: it SHALL depend only on
   `ff-i18n`, `ff-logging`, and small utility crates, and SHALL NOT depend on
   `egui` or `ff-desktop`, so any crate and any test can build structured messages.
2. THE core types (`MessageClass`, `MessageNumber`, `MessageId`, `Severity`,
   `StructuredMessage`) SHALL derive `Debug`, and SHALL derive `Clone`;
   `StructuredMessage` SHALL be `Send + Sync`.
3. THE crate SHALL use `thiserror` for its own library error type (identity
   validation errors), per the Rust standards, and SHALL NOT `unwrap`/`expect`/
   `panic!` in library code.
4. EACH source file SHALL stay under the 400-non-test-line limit; the crate SHALL
   be split by concern (identity, severity, message, render, adapters) per the
   FFWB Rust standards.
5. ALL new catalogue entries and documentation SHALL be plain ASCII per the
   documentation character-set rule.

---

## Out of Scope (this gate)

- ANY change to the `ff-logging` public API or `LogRecord`. The logger is the
  untouched sink; `ff-message` only renders and calls the existing entry point.
- The global "all logging moves to the message format" change (every log line
  carrying a class+number+severity identity). That is a high-blast-radius
  framework change to a Wave-1 crate and, if ever wanted, must be a separate
  owner-confirmed gate -- not designed here.
- PHASE 2: HONOURING the severity Behaviour_Contract in batch, background, and
  Command-Environment execution (Abort stops a batch run; Fatal terminates; Error
  stops the current operation). This gate DEFINES the axis and its meaning only;
  enforcing it is behavioural, higher blast radius, and deferred to separate tasks
  that reference `batch-execution` and require owner re-confirmation.
- Rebuilding or changing `ff-i18n`. `ff-message` depends on it unchanged.
- Modifying `VfsError` / `CommandError` source (including the pre-existing
  non-ASCII em-dash in two `CommandError` `#[error(...)]` strings -- variants
  `InvalidId` and `HistoryIo` -- a known source issue; `VfsError`'s
  `#[error(...)]` strings are plain ASCII). This gate only DEFINES the optional
  `HasMessageId` trait those enums MAY later adopt under their own gates.
- Authoring non-English catalogue content for structured messages. English
  Identity_Base entries prove the mechanism; other-locale overlays follow the
  localization capstone.
