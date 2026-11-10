# Findings: Structured, Language-Aware Messaging Core vs. Current FileForgeWorkbench

READ-ONLY investigation. No code changed. This report answers whether the
user's proposed SAP SE91/BAPIRET2-style structured message core (stable
identity + severity + parameters, rendered to localized text only at the
presentation boundary) already exists in `ff-logging` or elsewhere, and what a
new `ff-message` core crate would imply for logging, localization, and
notifications.

---

## Summary answer (the short version)

1. **Is any of this already built?** PARTIALLY, but NOT in the shape the user
   describes.
   - `ff-logging` is a plain-string, level-tagged file logger. It has NO message
     identity, NO class/number, NO parameter model, NO catalogue, NO
     render-time localization. It logs already-formatted `String` bodies.
   - The localization foundation the user half-remembers DOES exist, but it is a
     SEPARATE crate: `ff-i18n` (CR-NR-103). It provides exactly the
     "identity -> localized string at the presentation boundary" half of the
     idea: a keyed Fluent **Catalogue**, a process-wide lookup **seam**
     (`t("key")` / `t_args(key, args)`), an always-present English
     **Identity_Base**, named placeable **arguments**, and locale selection at
     render time. What it does NOT have is a SEVERITY tuple or a message
     CLASS+NUMBER identity bound to the key; the key is a free-form string and
     severity lives elsewhere (log level, notification level).
   - Cross-boundary errors ("one program calls another") are `thiserror` enums
     whose `Display` produces English strings with a `[domain]` prefix. There is
     a stable Rust-type identity (variant + fields) but NO machine-readable
     code/class+number and NO catalogue binding.
   - There is NO `ff-message` crate today (searched; none found).

2. **How much logging rework would a new `ff-message` core crate imply?**
   SUBSTANTIAL if "all logging moves to the message format" is taken literally,
   because every current call site logs a formatted `String` through
   level-based macros, and `ff-logging` has no dependency on identity/catalogue
   at all. A lighter, additive path exists: have `ff-message` own the
   identity+severity+params struct, render it to a `String` at the log boundary
   (reusing `ff-i18n` for the text), and feed that into the existing
   `ff-logging` API unchanged. That is a new ADDITIVE layer, not a rewrite of
   `ff-logging`.

3. **How should the proposed crate relate to the three specs?** `ff-message`
   would sit as a NEW foundation crate that DEPENDS ON `ff-i18n` (for the
   catalogue/locale render) and is CONSUMED BY the notification system and
   (optionally) the logging integration layer. It overlaps heavily with
   `ff-i18n` (do not rebuild the catalogue) and with `notification-system`'s
   `NotificationLevel` (the severity axis already exists there). It would need
   its own gate; it touches framework-conformant seams but introduces a new
   cross-cutting data type, which is an owner-level decision.

---

## Evidence by investigation point

### 1. Current `ff-logging` design and public API

Crate: `crates/ff-logging` (`Cargo.toml` deps: `chrono`, `crossbeam-channel`,
`thiserror`, `dirs`, `toml` — notably NO `ff-i18n`, NO fluent, no catalogue).

Public surface (from `crates/ff-logging/src/lib.rs` re-exports):

- Types: `LogLevel`, `LogRecord`, `LogConfig`, `LoggingError`, `LoggingStatus`,
  `ParseLogLevelError`, `BUILD_PROFILE_LEVEL`, trait `PluginLogHandle`.
- Functions: `init`, `init_default`, `reconfigure`, `shutdown`, `log`,
  `log_lazy`, `install_panic_hook`, `current_level`, `dropped_count`,
  `is_fallback`, `is_logging_available`, `create_plugin_handle`,
  `log_subsystem_error`.
- Macros: `log_trace!`, `log_debug!`, `log_info!`, `log_warn!`, `log_error!`.

The log record (`crates/ff-logging/src/record.rs`):

```rust
pub struct LogRecord {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub module_path: String,
    pub message: String,   // already-formatted, truncated, control-char-escaped
}
```

How a caller logs today (`crates/ff-logging/src/init/api.rs`):

```rust
pub fn log(level: LogLevel, module_path: &str, message: &str) { ... }
pub fn log_lazy(level: LogLevel, module_path: &str, f: impl FnOnce() -> String) { ... }
```

and the ergonomic form (`crates/ff-logging/src/macros.rs`):

```rust
ff_logging::log_info!("Application started successfully");
ff_logging::log_error!("Failed to open file: {}", err);
```

`LogLevel` (`crates/ff-logging/src/level.rs`) is the five-value severity enum
`Trace < Debug < Info < Warn < Error`. The standard subsystem error helper
(`crates/ff-logging/src/integration.rs`) just concatenates strings:

```rust
pub fn log_subsystem_error(level, subsystem, operation, error: &dyn Display) {
    log(level, subsystem, &format!("{operation}: {error}"));
}
```

**Verdict (point 1):** `ff-logging` logs PLAIN STRINGS tagged with a severity
level and a module path. It has NO structured identity, NO message
class/number, NO parameter/substitution model, and NO catalogue/locale concept.
The "message" is already-rendered English text by the time it reaches the
subsystem. The user's proposed model is NOT built into `ff-logging`.

### 2. Any existing structured-message / identity / class-number / catalogue / severity-tuple concept anywhere in `crates/`?

What EXISTS and is relevant:

- **`ff-i18n`** (`crates/ff-i18n`) — the localization catalogue mechanism
  (CR-NR-103). This is the strongest match for HALF of the user's idea:
  - `Catalogue` (`src/catalogue.rs`): per-locale Fluent bundles + an
    always-present English `Identity_Base`; `resolve(key)` and
    `resolve_args(key, &MessageArgs)` resolve active-locale -> Identity_Base ->
    the key itself (with a WARN) as fallback.
  - `set_catalogue` / `t` / `t_args` (`src/seam.rs`): a process-wide swappable
    catalogue behind an `RwLock`; free-function lookup seam so render code calls
    `t("key")` with no threading. Locale is chosen at render time.
  - `MessageArgs` (`src/args.rs`): named placeable arguments (`set(name, value)`
    / `set_number`) backing Fluent `{ $arg }` interpolation — this is the
    PARAMETER/substitution model (`&1..&n` analogue), with the explicit rule
    that an embedded command verb is passed as a NON-translated argument.
  - `Locale` + `IDENTITY_BASE_TAG = "en"` (`src/locale.rs`): BCP-47 locale,
    English as the always-present base.
  - DATA: `crates/ff-i18n/i18n/en/*.ftl` — e.g. `main.ftl` shows keyed messages
    with placeables (`welcome-user = Welcome, { $name }.`) and verb-embedding
    (`profile-locked = Profile is locked -- use { $verb } to unlock.`).
  - Live consumer example: `crates/ff-catalog-dialog` resolves its dialog text
    through `ff_i18n::t(...)` / `t_args(...)` and installs the English catalogue
    in tests via `ff_i18n::Catalogue::load_identity_base` + `set_catalogue`.

- **Severity enums** (two, both plain, neither tied to a message identity):
  - `ff_logging::LogLevel` (Trace/Debug/Info/Warn/Error).
  - `crates/ff-desktop/src/notification/mod.rs` `NotificationLevel`
    (Info/Success/Warning/Error). This is the closest to the SAP S/I/W/E/A type
    axis, but it has only four values (no "Abort/terminate") and carries plain
    `title`/`detail` strings, not an identity.

What DOES NOT exist (searched broadly for `message_class`, `message_number`,
`msgid`/`msgno`/`msgty`, `MessageClass`, `catalogue`, `BAPIRET`, `SE91`, `&1`):

- NO SAP-style message CLASS + NUMBER stable identity.
- NO severity-as-part-of-message-identity tuple (`msgid/msgty/msgno/msgv1..4`).
- NO `ff-message` crate (file search: none).
- The only `message_class` hit is `crates/ff-scrm/src/rules.rs`
  (`Condition::MessageClass`) — that is a MAINFRAME SCREEN-capture rule field
  (match on an emulated screen's message class), entirely unrelated to a logging
  message-identity model. The `ff-command-semantics` `MSGID` hit is a TSO
  `PROFILE MSGID` operand, also unrelated.

**Verdict (point 2):** The CATALOGUE + LOCALE + PARAMETER half of the proposal
already exists as `ff-i18n`. The MESSAGE-IDENTITY (class+number) + SEVERITY-TUPLE
half does NOT exist anywhere. The key in `ff-i18n` is a free-form string, not a
class/number, and it carries no severity.

### 3. The notification seam (`Arc<Mutex<NotificationQueue>>`)

Defined in `crates/ff-desktop/src/notification/mod.rs` (notification-system
spec). It carries a STRUCT, not a bare string, but the struct's payload is plain
text:

```rust
pub enum NotificationLevel { Info, Success, Warning, Error }

pub struct Notification {
    pub level: NotificationLevel,
    pub title: String,
    pub detail: Option<String>,
    pub timestamp: String,
}

pub struct NotificationQueue { entries: VecDeque<Notification>, unread: usize }
pub struct NotificationSender { tx: mpsc::SyncSender<Notification> } // Clone + Send
```

Wiring: `crates/ff-desktop/src/shell/construct.rs` builds a bounded
`sync_channel::<Notification>(64)` and an
`Arc<Mutex<NotificationQueue>>`; the queue is held on the shell
(`shell/state.rs: notification_queue`) and exposed to Contexts via
`ShellServices.notifications` (`shell/workspace_context.rs`). UI consumers render
it in `crates/ff-desktop/src/event_log_panel.rs` (the Event Log Context) and as
status-bar/toast surfaces. `NotificationQueue` caps at 1000 entries, tracks
unread Warning/Error for the status-bar badge, supports `filter_by_level`.

**Verdict (point 3):** The notification carries a 4-level severity plus two
pre-rendered strings (`title`, `detail`). No identity, no params, no catalogue.
Text is already localized/English at emit time — the toast/log render just
displays the stored strings. This is the single best INSERTION POINT for a
structured message (it already has a severity axis and a struct payload).

### 4. Cross-boundary error surfacing (connectors / Command Environments / VFS)

Errors are `thiserror` enums with `Display` strings carrying a `[domain]`
prefix, NOT structured machine-readable identities:

- `crates/ff-vfs/src/error.rs` — `VfsError` (`#[non_exhaustive]`): variants
  `NotFound`, `PermissionDenied`, `AlreadyExists`, `NotADirectory`,
  `UnsupportedOperation`, `InvalidUri`, `ProviderUnavailable`, `Timeout`, `Io`,
  `DuplicateScheme`. Each `#[error("[vfs] {operation}: ...: {uri}")]` renders a
  fixed English string with interpolated context.
- `crates/ff-command/src/error.rs` — `CommandError` (`#[non_exhaustive]`):
  `NotFound`, `Disabled`, `DuplicateId`, `InvalidId`, `ShortcutConflict`,
  `ShortcutReserved`, `ExecutionFailed`, `UndoFailed`, `RedoFailed`,
  `HistoryIo`, `ScriptingError`; plus a `ScriptingError` enum. Same
  `[command] operation: description` convention (cross-cutting Requirement 8).

So the identity that travels across the "one program calls another" boundary is
the Rust ENUM VARIANT (+ typed fields) — structured within Rust, but:
- NO stable external code (no class+number) a catalogue or a non-Rust consumer
  could key on;
- the human text is baked into the `#[error("...")]` attribute in English, i.e.
  rendered at the SOURCE, not at the presentation boundary.

Localization Req 9.3 (see point 5) explicitly anticipates this gap: a message
whose text originates in a `thiserror` enum is meant to be routed through the
`ff-i18n` seam at the DISPLAY boundary — but that routing is not yet done for
these enums.

(Minor, out-of-scope observation: `CommandError::InvalidId` and `HistoryIo`
`#[error]` strings use a non-ASCII em-dash; the documentation-charset rule is
about docs/pattern-matched source, so noting only.)

**Verdict (point 4):** Cross-boundary errors pass STRUCTURED Rust enums, but
there is NO machine-readable message identity (class/number/code) and the text
is English-at-source. This is exactly the seam a SAP-style identity would
formalize.

### 5. Existing spec folders — current state and scope

- **`docs/specs/logging-subsystem`** (`requirements.md`, `design.md`,
  `tasks.md`): the `ff-logging` spec. Scope = file-based structured logging:
  init ordering, record FORMAT (timestamp + 5-char level + `[module]` +
  message), level config (`logging.level`), file location/rotation/retention,
  flush/shutdown, GUI-independent process model, thread-safety/overflow,
  platform-core integration patterns (Req 9), plugin logging handle (Req 10),
  runtime `reconfigure` (Req 11), and a logging-inventory tool (Req 12). The
  "structured" in its title means "fixed-field line format", NOT message
  identity. NO localization, NO catalogue, NO message class/number. Recent
  CR-NR-086 additions (Req 8.7, 9.6-9.8) push MORE instrumentation into the
  existing string-logging model (VFS dispatch, connector obligations), not an
  identity model.

- **`docs/specs/localization`** (CR-NR-103): `requirements.md`, `design.md`,
  `tasks.md`, `localization-vision.md`, plus `gate-review.{md,json}`. This IS
  the message-catalogue foundation, already partly implemented as `ff-i18n`.
  Scope: `ui.locale` config key (default `en`, hot-reloadable); the Fluent
  Message_Catalogue as per-locale `.ftl` DATA; the `t`/`t_args`
  Catalogue_Lookup_Seam; the always-present English Identity_Base; the verb/text
  split (verbs stay English, prose translates); per-locale per-environment alias
  catalogues into the existing `AliasTable`; help + menu localization; plugin
  localization; and argument-bearing/verb-embedding messages via Fluent
  placeables. Req 9.3 explicitly says `thiserror`-originated text should be
  localized AT THE DISPLAY BOUNDARY through the seam — i.e. localization already
  assumes a catalogue foundation and a render-time resolution model. Its shape:
  a FREE-FORM STRING KEY -> localized text, with named args. It does NOT model
  severity or a class+number identity; it is a presentation layer by explicit
  design (Req 10: additive, no new framework mechanism). Worktree
  `.worktrees/l10n-phase2-task9` is actively doing Phase 2 incremental
  extraction (converting hardcoded render literals to `t("key")` across shell,
  menus, help, search-results, etc.). Its git log shows many
  `feat(l10n): task8 ...` commits; a `git diff` confirms that worktree has NOT
  modified `ff-logging` at all (localization is touching render sites, not the
  logger).

- **`docs/specs/notification-system`** (`requirements.md`, `design.md`,
  `tasks.md`): the `Notification` / `NotificationQueue` / `NotificationSender`
  seam. Scope: toast notifications (4 levels, Info/Success auto-dismiss,
  Warning/Error sticky, stacked, "N more"), the Event Log Context (`LOG`
  command / bell icon), the non-blocking `NotificationSender` API, and
  status-bar unread badge. It defines the SEVERITY axis (4 levels) and a
  structured event payload, but the payload is `title`/`detail` STRINGS with no
  identity/params/catalogue. It is the natural consumer of a structured message.

**Verdict (point 5):** `localization` already provides the catalogue + locale +
render-boundary foundation (and assumes `thiserror` text localizes at display).
`logging-subsystem` and `notification-system` are both plain-string + severity
designs with no identity model. NONE of the three describes a SAP-style
class+number message identity or "all logging in message format".

### 6. `.worktrees/l10n-phase2-task9`

- Branch is mid-flight localization Phase 2 Task 9 work (git log:
  `feat(l10n): task8 ...` sweeps across shell-status, shell-dialogs,
  shell-nav-ops, menu-workspace, help-context, search-results, pom-calendar,
  about, scrm-viewer, clear-log). This is INCREMENTAL STRING EXTRACTION:
  replacing hardcoded egui literals with `ff_i18n::t("key")` calls and adding
  `.ftl` keys. It is building out catalogue DATA and converting render sites — it
  is the ongoing realization of CR-NR-103, exactly the "catalogue/locale
  infrastructure" the brief suspected.
- Its `crates/ff-logging/` is BYTE-IDENTICAL in file inventory to main
  (lib.rs 5392 bytes etc.) and a `git diff -- crates/ff-logging` against its HEAD
  is EMPTY: the localization effort has NOT touched the logger. So no competing
  structured-logging design is being built in that worktree.

**Verdict (point 6):** The worktree is advancing localization (catalogue + seam
adoption), NOT building a message-identity or logging-format change. It confirms
`ff-i18n` is the live, growing localization foundation and that `ff-logging` is
untouched by it.

---

## Conclusions

### (a) Is any of this already built?

- The **render-at-the-boundary localized-text half** is built: `ff-i18n`
  (catalogue, locale, Identity_Base fallback, named args, process-wide seam),
  actively being adopted via the l10n Phase 2 worktree. The localization spec
  even anticipates routing `thiserror` error text through the seam at display
  time (Req 9.3).
- The **severity axis** is built twice: `ff_logging::LogLevel` (5 levels) and
  `NotificationLevel` (4 levels: Info/Success/Warning/Error). Neither matches
  SAP's A/S/E/W/I exactly; there is no "Abort/terminate" notification level.
- The **stable message IDENTITY (class + number) + severity-tuple** half is NOT
  built anywhere. Keys in `ff-i18n` are free-form strings with no severity.
  Cross-boundary identity today is the Rust enum variant, English-at-source,
  with no external code.
- `ff-logging` itself does NOT contain any of the proposed model — it logs
  pre-rendered strings. The user's recollection that "this may already be in
  ff-logging" is NOT borne out; the relevant pieces live in `ff-i18n` and
  `notification-system`, not the logger.

### (b) How much logging rework would a new `ff-message` crate imply?

Two materially different interpretations:

- **Literal "ALL logging moves to the message format":** LARGE. Every log call
  site (hundreds, across ~90 crates — see the logging-inventory tooling in
  logging-subsystem Req 12) currently passes a formatted `String` through
  `log_*!` macros. Making every record carry a class+number+severity+params
  identity means: a new identity registry/catalogue, converting call sites,
  and extending `LogRecord` (currently `{timestamp, level, module_path,
  message}`) and the formatter. `ff-logging` has NO catalogue dependency today
  (by design it depends only on `chrono`/`crossbeam`/`thiserror`/`dirs`/`toml`),
  so this is a foundational change to a Wave-1 crate that everything depends on.
  High blast radius; would need an owner-confirmed framework change.

- **Additive "structured messages render to a string at the boundary":** SMALL
  to MODERATE and recommended. `ff-message` owns the
  identity+severity+params type and a `render(&self) -> String` that resolves
  through `ff-i18n`. At the LOG boundary, `ff-message` renders to a `String`
  and calls the EXISTING `ff_logging::log(level, module, &rendered)` — no change
  to `ff-logging` at all (the message's severity maps to `LogLevel`). At the
  NOTIFICATION boundary, a `Notification` gains an optional structured-message
  constructor that renders `title`/`detail` via the same path. Existing
  plain-string call sites keep working; new/important messages opt in. This is
  the SAP BAPIRET2 spirit (pass the structured tuple around, render only at the
  edge) without rewriting the logger.

### (c) How the proposed crate should relate to the three specs

- **Depend on `ff-i18n`, do NOT rebuild it.** The catalogue, locale selection,
  Identity_Base fallback, and named-argument substitution (`&1..&n`) are already
  there. A SAP "message class + number" is best modeled as a disciplined
  CONVENTION over `ff-i18n` keys (e.g. a key like `vfs-e-007` or a
  `{class, number}` that maps to a key), with `ff-message` adding the SEVERITY
  and the stable IDENTITY that `ff-i18n` deliberately omits. Align with
  localization Req 4 (verbs/identifiers stay English; only prose localizes) and
  Req 9.3 (localize `thiserror` text at the display boundary) — `ff-message` is
  effectively the formalization of Req 9.3's "route error identity through the
  seam at display".
- **Consume into `notification-system`, reuse its severity.** The 4-level
  `NotificationLevel` is the existing severity axis; `ff-message`'s severity
  should map onto it (and onto `LogLevel`). The notification struct is the
  cleanest first consumer because it already carries a severity + a struct
  payload; add a structured-message path that renders `title`/`detail`.
- **Keep `logging-subsystem` as the sink, layered not rewritten.** Prefer the
  additive boundary-render approach so `ff-logging`'s public API and `LogRecord`
  are untouched. If the owner truly wants every log line to carry identity, that
  is a framework-level change to a Wave-1 crate and must be an explicit,
  owner-confirmed gate (per framework-conformance), not an implementation side
  effect.
- **New gate required.** `ff-message` is a new cross-cutting foundation type and
  a new crate; it needs its own requirements/design/tasks under
  `docs/specs/` and should be cross-referenced from `localization`,
  `logging-subsystem`, and `notification-system`. It overlaps enough with
  `ff-i18n` that the gate's first job is to draw the boundary: `ff-i18n` = text
  resolution; `ff-message` = identity + severity + params carried across
  boundaries and rendered via `ff-i18n` at the edge.

### Recommended shape (fact-based, not a requirements proposal)

A minimal, framework-conformant `ff-message` would:
- define `MessageId` (stable identity; a class+number OR a catalogue key),
  `Severity` (mapping to both `LogLevel` and `NotificationLevel`), and a
  params/args carrier (wrapping or mirroring `ff_i18n::MessageArgs`);
- hold a `StructuredMessage { id, severity, args }` that is `Clone + Send + Sync`
  and cheap to pass across the "one program calls another" boundary;
- expose `render(&self) -> String` (or `render_title`/`render_detail`) that
  resolves through `ff_i18n::t_args` at the presentation boundary;
- offer adapters: `-> ff_logging::log(level, module, &rendered)` and
  `-> Notification`;
- let `thiserror` enums optionally expose a `MessageId` so cross-boundary errors
  gain a stable identity without changing their current `Display` behaviour.

This delivers the user's BAPIRET2 intent (structured identity+severity+params
passed around, localized text only at the edge) while reusing `ff-i18n` and
leaving the `ff-logging` Wave-1 foundation intact unless the owner explicitly
elects the heavier "all logging in message format" change.

Content was rephrased for compliance with licensing restrictions where external
(SAP) concepts are summarized.
