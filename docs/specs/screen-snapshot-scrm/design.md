# Design Document -- Screen Snapshot Service + SCRM, and the ff-desktop decomposition it seeds (CR-NR-098)

## Status

DRAFT for owner review (design phase of the gate). Resolves the two design-level
open questions from requirements.md: Q3 (auto-capture transition events) and Q8
(PDF library). No code is written until this design and the accompanying tasks.md
are approved.

## 1. Design principles

1. Text-first, model-derived capture (requirements Req 1): capture reads a LOGICAL
   `Screen_Model` and renders it to selectable text; it never scrapes egui
   widgets. The authoritative artefact is always copy/paste-able text.
2. Plugin-shaped seam, core packaging (Req 2): the ONLY coupling between the app
   and the capture engine is the `ScreenProvider` trait plus the existing
   command-dispatch / WorkspaceContext / navigation seams. The engine lives in new
   crates compiled into the app.
3. Build ON the framework (framework-conformance.md): no new command dispatcher,
   no second navigation stack, no per-Context focus mechanism, no new persistence
   format. Every seam below is an EXISTING one, verified in code.
4. SCRM is the first extracted vertical of the ff-desktop decomposition (Req 19):
   the bulk of the new code lands OUTSIDE `ff-desktop`, and the `ScreenProvider`
   pattern becomes the template for later panel extractions.

## 2. Code-grounded seam survey (what SCRM plugs into)

Verified against the current tree:

- **Command dispatch.** `ff_command::resolve_target` -> `CommandTarget`
  (`crates/ff-command/src/command_target.rs`; variants `Menu`, `CustomWorkspace`,
  `Function`, `Macro`, `External`). The shell dispatches via
  `crates/ff-desktop/src/shell/target_dispatch.rs`
  (`resolve_and_dispatch_command` -> `dispatch_command_target`); a `Function`
  target runs through `handle_command` for an observably-identical result. Simple
  built-in verbs today are `if upper == "..."` intercepts in
  `shell/commands.rs` (stage 2, around L384 onward: EXIT, HELP, KINDS, FILES,
  SEARCH, LOG, PLUGINS, MACROS, ...).
- **Context-transition choke point (the Q3 hook).** `navigate_to(descriptor,
  push)` in `crates/ff-desktop/src/shell/nav_stack.rs` (~L280) calls
  `reconstruct_context(&descriptor)`. ALL context changes funnel through it:
  `nav_to_kind`, `nav_end`, `nav_return`, `start_new_workspace` ->
  `apply_start_command` -> `handle_command`. `reconstruct_context` (match in
  `shell/commands.rs` ~L2180) is the single place a tab's visible Context is
  rebuilt.
- **WorkspaceContext / InteriorFocus.** Trait in
  `crates/ff-desktop/src/shell/workspace_context.rs`:
  `render(&mut self, ui, services: &mut ShellServices) -> InteriorFocus`. A
  Context enqueues `ShellRequest`s (Command / Target / OpenFile / Status) drained
  after render; `InteriorFocus::single(id)` reports the focus contract. Minimal
  implementors already exist (help_context, config panel, search results).
- **New-Context wiring pattern (verified via EventLog/MacroLibrary/
  CommandConfigurator).** Adding a Context touches: a `WorkspaceKind` variant in
  `ff-session`; a `TabKind`; an `open_*_tab` constructor on the tab store; a
  `reconstruct_context` match arm; and the descriptor mapping in
  `session_manager.rs` + `nav_stack.rs`. Persistence is a
  `WorkspaceDescriptor::CustomWorkspace { workspace_kind, params }`.
- **Existing logical models.** Panels render straight to egui, but several already
  split state from render (`config_panel/{tree,render}`, `menu_workspace/*`,
  `search_results_panel/{state,render}`, `menus_editor_panel/render`). So a
  `ScreenProvider` is implemented from a panel's STATE, not scraped from egui.

## 3. Crate decomposition (Req 2, Req 19)

Two NEW workspace crates (owner-confirmed names, Q4):

- **`ff-screen-model`** (no egui, no SCRM logic). Owns:
  - `ScreenModel` and its elements: `Title`, `Field { label, value, attrs }`,
    `Table`, `Message`, `StatusBar`, `command_line`, `cursor`, `dimensions`
    (requirements Req 3). `attrs` include colour, highlight, protection, and a
    `sensitive: bool` (Req 13.5).
  - The `ScreenProvider` trait: `fn screen_model(&self) -> ScreenModel`.
  - The renderers (Req 4, 5): `PlainText`, `Ansi`, `Markdown`, `Html`, `Yaml`,
    behind a `SnapshotFormat` enum. Box-drawing default + ASCII fallback (Req 5.1,
    5.2). Pure functions `ScreenModel -> String`.
- **`ff-scrm`** (no egui; depends on `ff-screen-model`). Owns:
  - `ScreenCollection` / `ScreenCapture` data model (Req 17). `ScreenCapture`
    carries an OPTIONAL `TriggeringCommand { command_string, target_kind }`
    (Req 17.2a) -- the command, as resolved through the single dispatch path,
    that caused the transition INTO this capture. `Option`, not an empty-string
    sentinel, so a non-command cause (interval/rule) is explicitly absent
    (Req 9.12). This is the "recorder" delta that turns a screen-only timeline
    into a command + screen timeline (Req 9.10).
  - Persistence: the native zip archive (`collection.yaml`, `screens/`, `images/`,
    `metadata/`) (Req 12.5, 17.3) + crash-recovery journal (Req 18.4). The
    optional `TriggeringCommand` round-trips unchanged, present or absent
    (Req 17.2a).
  - Capture rules + masking rules (Req 9.7-9.9, Req 13).
  - Replay engine (sequence, first/prev/next/last, autoplay, speed, timing)
    (Req 10) -- pure state machine, no egui.
  - Exporters: text/md/html/pdf + protected-pdf + evidence package (Req 12, 14,
    20).

`ff-desktop` gains only the THIN wiring: the `ScreenProvider` impls per Context,
the `SNAPSHOT`/`CAPTURE` command handlers, the auto-capture hook, and the SCRM
viewer Context (egui). This keeps the heavy code out of the binary (Req 19.1,
19.2).

## 4. Q3 RESOLVED -- automatic-capture transition events

Automatic Capture Mode observes the framework's real screen-change choke points.
Enumerated against the code:

1. **Context transition**: a single hook at the end of
   `nav_stack::reconstruct_context` (reached by navigate_to / nav_end /
   nav_return / start_new_workspace). One hook covers POM<->panels, dialogs, and
   drill navigation. This is the primary "screen changed" signal.
2. **Command completion that changes the screen**: after `handle_command` returns,
   if the active Context's `ScreenModel` differs from the pre-command model
   (compare a cheap content hash), emit a capture. This covers the mainframe
   ENTER/PF/PA triggers (Req 9.2-9.4), because those keys dispatch through the
   same command path (`dispatch_key_command` -> `dispatch_bound_command` ->
   `handle_command`).
3. **Menu-option activation**: `open_menu_by_name` (`shell/commands.rs` ~L1529)
   -- already routed through navigate_to for in-place menus, so it is covered by
   (1); the explicit note is retained for the new-tab menu case (Req 9.5).

Design decision: implement (1) as the single guaranteed hook in v1, add (2) as a
model-diff guard so command-driven changes are caught without double-capturing
(1) and (2) for the same transition (dedupe by content hash + a per-frame
"already captured this transition" flag). The configurable interval (Req 9.6) and
rules (Req 9.7-9.9) filter what (1)/(2) emit. This introduces NO new navigation or
dispatch mechanism (framework-conformance).

### Recording the triggering command (Req 9.10-9.12, the recorder delta)

When a capture is emitted by cause (2) -- a command completion that changed the
screen -- the command just resolved by `ff_command::resolve_target` and
dispatched by `shell/target_dispatch.rs` is the triggering command. The dispatch
layer already holds both the canonical command string and the resolved
`CommandTarget`; the auto-capture path threads that pair onto the new capture's
`TriggeringCommand` (Req 9.10). No bespoke intercept is added -- the data is read
at the existing seam the command already passes through (Req 9.11,
framework-conformance). For cause (1) reached WITHOUT a command (there is no such
path in v1: every Context transition is command-driven via navigate_to) and for
interval/rule captures (Req 9.6/9.7), `TriggeringCommand` is left absent
(Req 9.12). Replay shows a "caused by: <command>" line only where the field is
present (Req 10.9-10.10). The schema is deliberately sufficient to later export
the ordered commands as a macro (Req 21.2); the exporter itself is deferred
(Req 21.1).

## 5. Q8 RESOLVED -- PDF library selection

Constraints: real SELECTABLE text (Q2, Req 12.6), AES encryption + permission
flags (Req 20.1, 20.2), optional user/open password (Req 20.2a), and ideally a
path to digital signatures (Req 20.4a). Candidates surveyed (Rust ecosystem):

- **printpdf** -- mature pure-Rust generator, real text/font embedding; good fit
  for the title page / TOC / index / page numbers (Req 12.4) and selectable text.
  Encryption support is limited/version-dependent.
- **lopdf** -- low-level PDF document model; can post-process a generated PDF to
  apply the encryption dictionary + permission flags and owner/user passwords.
- **Commercial/AGPL options (Aspose, rustpdf.dev)** -- full AES-256 + PAdES
  signing, but licence/cost implications the owner has not approved.

Design decision (v1): **generate with `printpdf` (real selectable text, layout),
then apply encryption + permission flags + optional user password via `lopdf`'s
document model** (RC4/AES per what lopdf supports at the pinned version). The
content HASH (Req 20.4) is computed over the source Collection and embedded in
both the PDF metadata and the evidence-package metadata -- this is the tamper-
EVIDENCE and does not depend on the PDF encryption strength. Digital signatures
(Req 20.4a) are deferred as a configurable add-on; if a pure-Rust PAdES path is
not viable at pin time, the add-on is documented as requiring an external signer,
surfaced to the owner before that slice is built (not part of v1).

Dependency versions will be pinned exactly (rust-standards / safety). The precise
crate versions are chosen at implementation time and recorded in tasks.md; if the
encryption capability of the chosen versions cannot meet Req 20.2 (edit-lock via
owner password), that is surfaced as a blocker before the protected-PDF slice,
not worked around.

### Pre-slice checkpoint RESULT (Wave 3, 2026-09-26): PASS

The mandatory checkpoint (tasks.md task 9.1 / Wave 3 task 1) is RESOLVED and the
protected-PDF slice is VIABLE:

- **printpdf** generates PDFs with embedded fonts and REAL selectable text
  (satisfies Q2 / Req 12.6). It is the layout + text generator.
- **lopdf** ships an `encryption` module (`lopdf::encryption`) exposing
  `EncryptionState`, `Permissions`, `PasswordAlgorithm`, `EncryptionVersion`, and
  `encrypt_object` / a `Document`-level encrypt path. `EncryptionState` has
  `permissions()`, `owner_value`/`user_value`, and `encode() -> Dictionary` to
  WRITE the `/Encrypt` dictionary (owner password + permission flags), not merely
  DECRYPT. This meets Req 20.1/20.2 (edit-lock via owner-password encryption +
  permission flags) and Req 20.2a (optional user/open password). Verified against
  the published lopdf `encryption` module docs.

Design REFINEMENT (Wave 3 implementation): the base PDF is emitted by a small
HAND-WRITTEN PDF 1.7 writer in `ff-scrm/src/pdf.rs` using the Base-14 `Courier`
font and real text content streams (`BT ... Tj ... ET`), rather than pulling
`printpdf`. Reason: the target is a monospaced text dump (title page + screen
index + one page per capture), which is a small, stable slice of PDF we can emit
deterministically and keep under our own control (mirroring the hand-written YAML
renderer); `printpdf`'s API has churned across recent releases and betting the
`-D warnings` gate on it was an unnecessary risk. The text is genuinely
selectable (Req 12.6) -- verified by tests asserting a `Tj` operator and the
`/BaseFont /Courier` (not an image). This keeps ff-scrm dependency-free for the
UNPROTECTED export. The PROTECTED slice still loads these bytes into a
`lopdf::Document` and applies an `EncryptionState` (owner password + `Permissions`
copy-allowed / modify-disallowed; optional user password) and saves.

Implementation note (lopdf 0.36, Wave 3): the protected path is
`Document::load_mem(plain_bytes)` -> set `/Info` (Producer/Title/content-hash/
collection-id) -> set trailer `/ID` (a two-element array of 16-byte identifiers
derived from the content hash; REQUIRED because lopdf derives the file encryption
key from `/ID` and errors "missing the file /ID elements" without it, and our
hand-written base PDF emits none) -> `EncryptionState::try_from(
EncryptionVersion::V2 { document, owner_password, user_password, key_length: 16,
permissions })` (128-bit, revision 3) -> `doc.encrypt(&state)` -> `doc.save_to`.
`Permissions = PRINTABLE | COPYABLE | COPYABLE_FOR_ACCESSIBILITY |
PRINTABLE_IN_HIGH_QUALITY` (MODIFIABLE / ANNOTABLE / ASSEMBLABLE cleared =
edit-locked, copy-allowed). Content hash is SHA-256 (`sha2` crate), shared with
the evidence package. The content HASH (Req 20.4) is computed over the source Collection with a
SHA-256-class hash and embedded in the PDF metadata + evidence-package metadata;
it is the tamper-EVIDENCE and is independent of the PDF encryption strength.
Exact `printpdf` / `lopdf` / hash-crate versions are pinned in Cargo.toml at
implementation time. Digital signatures (Req 20.4a) remain a deferred, owner-
confirmed add-on (no pure-Rust PAdES path assumed).

## 6. SCRM viewer as a WorkspaceContext (Req 16)

- New `WorkspaceKind::ScrmViewer` (ff-session) + `TabKind::ScrmViewer`
  (ff-desktop) + `open_scrm_viewer_tab` on the tab store, mirroring EventLog.
- `ScrmViewerContext` implements `WorkspaceContext`; `render` draws the replay
  surface (the captured `ScreenModel` rendered as selectable text) + first/prev/
  next/last controls, and returns `InteriorFocus::single(<stable id of the first
  control>)`. Its first interior control gets a stable `egui::Id`
  (`egui::Id::new("scrm_viewer_first")`).
- Persists as `CustomWorkspace { workspace_kind: ScrmViewer, params: { collection
  id } }`; transient replay position is NOT persisted (Req 16.3).
- Mandatory full-shell first-Tab egui_kittest test
  (`full_shell_scrm_viewer_first_tab_focuses_...`) per workspace-conformance.

## 7. Commands (Req 11) -- all via the single dispatch path

`SNAPSHOT [TEXT|ANSI|MARKDOWN|MD|HTML|YAML|AI]` and `CAPTURE
START|STOP|SCREEN|STATUS|LIST|OPEN|SAVE|LOAD|REPLAY|EXPORT
TEXT|MD|HTML|PDF|PDF PROTECTED|PURGE`. Implemented as registered Command_IDs
resolving to `Function` targets (preferred over new hard-coded `if upper == "..."`
intercepts, to honour framework-conformance and command parity). Any menu/toolbar
affordance invokes the SAME command id (Req 11.4).

## 8. Decomposition waves (Req 19) -- each behaviour-preserving, each verify.ps1-green

- **Wave 0 (foundation, NEW code only, no ff-desktop churn):** create
  `ff-screen-model` (+ `ScreenProvider`, renderers) and `ff-scrm` (data model,
  persistence, replay, exporters) with full unit/proptest coverage. ff-desktop
  untouched. Lowest risk; largest line volume lands outside the binary.
- **Wave 1 (thin wiring):** SNAPSHOT command + one `ScreenProvider` impl (POM) +
  clipboard delivery. Proves the vertical end to end with minimal ff-desktop code.
- **Wave 2:** CAPTURE collection lifecycle + auto-capture hook at
  reconstruct_context + the ScrmViewer Context.
- **Wave 2R (recorder delta):** optional `TriggeringCommand` on `ScreenCapture`,
  populated from the dispatch seam at the auto-capture hook, round-tripped in
  persistence, and surfaced as a "caused by" line in replay (Req 9.10-9.12,
  10.9-10.10, 17.2a). Macro export (Req 21.1) is DEFERRED -- v1 only guarantees
  the schema is macro-sufficient (Req 21.2).
- **Wave 3:** exporters wired to commands, masking, evidence package, protected
  PDF.
- **Waves 4+ (pure refactor, separate CRs as they land):** use the ScreenProvider
  seam as the template to extract heavy panels (files_panel ~1998, editor_panel
  ~1512, catalog_manager_dialog ~1176, explorer_view ~1082) out of ff-desktop.
  These are behaviour-preserving refactors (no gate) measured before/after
  (Req 19.4); they are NOT part of the SCRM build and are listed here only to show
  the seam is the shared mechanism (Req 19.5).

Each wave keeps `verify.ps1` (full) green and adds no observable change to
existing behaviour (Req 19.3).

## 9. Risks / decisions to confirm

- PDF encryption strength at the pinned lopdf version (Section 5) -- confirmed as
  a pre-slice checkpoint, not worked around.
- Auto-capture volume: the model-diff guard + dedupe (Section 4) prevents capture
  floods; NFR Req 18.1 (>=10,000 captures) and Req 18.2 (no visible interruption)
  drive an async, off-frame capture write (ff-scrm persistence on a worker, not
  the egui thread).
- `ScreenProvider` coverage: v1 provides impls for the Contexts that matter first
  (POM, dialogs); a Context without an impl reports "not capturable" (Req 2.3)
  rather than producing an empty capture.

## 10. Traceability

Every requirement in requirements.md maps to a section here: Req 1->S1/S3; Req
2->S3; Req 3->S3; Req 4-6->S3/S7; Req 7-10->S3(ff-scrm)/S4; Req 11->S7; Req
12->S3/S5; Req 13->S3; Req 14->S3; Req 15->S3 (DIDL fields on ScreenCapture);
Req 16->S6; Req 17->S3; Req 18->S9; Req 19->S8; Req 20->S5; Req 9.10-9.12 /
10.9-10.10 / 17.2a (recorder delta)->S3/S4; Req 21 (macro export)->DEFERRED, not
designed in v1 beyond the schema-sufficiency guarantee (Req 21.2); Req 28
(dual-representation clipboard flavors, CR-CH-079)->S11; Req 29 (full-window
snapshot fidelity, CR-CH-115)->S12. tasks.md will carry the per-criterion task
breakdown and TCR rows.

## 11. Dual-representation clipboard text flavors (Req 28, CR-CH-079)

### Investigation verdict -- where each representation is generated today

Settled against the code (NOT assumed), reading
`crates/ff-screen-model/src/render/{text,markdown,html}.rs`,
`crates/ff-screen-model/src/model.rs`,
`crates/ff-desktop/src/shell/screen_providers_config.rs`,
`crates/ff-desktop/src/shell/commands_snapshot.rs`, and
`crates/ff-desktop/src/clipboard_html.rs`:

- The clipboard SNAPSHOT places THREE OS flavors in one transaction
  (`OsClipboard::place` in `clipboard_backend`): `CF_DIB`/`CF_DIBV5` (the real
  window PNG), `CF_UNICODETEXT` (the plain-text flavor), and `CF_HTML` (the
  composite rich flavor). This is the Req 23 three-flavor model and is NOT
  changing in count.
- The PLAIN-TEXT (`CF_UNICODETEXT`) flavor is produced by
  `stash_composite_request` as `render_snapshot(model, Markdown)` -- the Markdown
  renderer (`render/markdown.rs::render_markdown`). For a Context that populates
  `model.markdown_sections` (the Config panel does, via
  `config_capture_markdown_sections`), `render_markdown` takes the
  `markdown_sections` branch and `return`s EARLY: it emits ONLY the per-group
  `### [Group] (state)` heading + 2-column `Property | Value` GFM tables and
  NEVER renders the fenced box-drawn body. => Notepad gets tables, no box.
  (Confirmed as the owner-observed behaviour.)
- The COMPOSITE RICH (`CF_HTML`) flavor is built by
  `clipboard_html::build_cf_html(png, pre_text)`, where `pre_text` is
  `render(model, PlainText, default)` -- i.e. the box-drawn screen from
  `render/text.rs::render_plain` (which renders `body_lines` verbatim inside the
  frame). `build_cf_html` emits the PNG `<img>` paragraph followed by ONE
  monospace `<span>` paragraph per box line (Req 23.5f). It does NOT render
  `markdown_sections`, `tables`, or `fields` as HTML tables at all. => Word gets
  image + box, no tables.
- So the asymmetry is structural: the two text flavors use DIFFERENT renderers
  (Markdown vs PlainText-into-CF_HTML), and for a `markdown_sections` Context the
  Markdown renderer deliberately swaps the box body OUT for tables, while the
  CF_HTML path only ever carries the box body and never the tables.

### Chosen design -- make both flavors carry both, in the shared path

The fix is GENERAL (Req 28.7), implemented in the shared `ff-screen-model`
renderers and the shared `clipboard_html` composite, so every table-bearing
Context inherits it; the Config provider is NOT special-cased.

PLAIN-TEXT flavor (`CF_UNICODETEXT`), satisfying Req 28.1/28.4/28.6:
- Change `render/markdown.rs::render_markdown` so that when `markdown_sections`
  is present it emits the box-drawn body FIRST (the fenced Unicode-box block, as
  the generic path already does via `text::render_plain` inside a code fence),
  THEN the `markdown_sections` tables -- instead of early-returning tables only.
  The box-first-then-tables order (Req 28.4) matches the generic Markdown path,
  which already emits the fenced body then the property table then table
  elements.
- A Context with NO `markdown_sections` is unchanged (Req 28.7): it still gets
  the fenced body + the `fields` property table + each `tables` element, which
  already carries both representations.

COMPOSITE RICH flavor (`CF_HTML`), satisfying Req 28.2/28.4/28.5/28.6:
- Extend the composite so that, after the PNG `<img>` paragraph and the per-line
  monospace box `<span>` paragraphs, it appends HTML `<table>` markup for the
  same property/value content. Two viable approaches were considered:
  - (A) Have `build_cf_html` take an extra "tables HTML" argument produced by the
    HTML renderer from the model's tabular content, and append it inside the
    fragment after the box paragraphs.
  - (B) Render the whole model via `render/html.rs::render_html` and inject it
    into the fragment.
  CHOSEN: (A). `render_html` renders `fields`/`tables` but NOT
  `markdown_sections`, and it also re-emits the title/body/command-line chrome,
  which would duplicate what the box `<pre>` already shows. (A) keeps the box
  body exactly as Req 23.5f tuned it (monospace per-line `<span>`, which Word
  honours) and appends ONLY the tables, derived from the SAME content the
  Markdown flavor uses. Concretely: add a small HTML-table emitter (either a new
  `render/html.rs` helper that renders `markdown_sections`/`tables` as `<table>`
  with `<caption>` group headings, or reuse the existing `ffwb-table` emitter)
  and pass its output to `build_cf_html` as a new `tables_html` parameter
  appended after the box paragraphs, inside the fragment and before
  `<!--EndFragment-->`.
- The CF_HTML header byte offsets are already recomputed from the assembled
  `doc_prefix`/`body`/`doc_suffix` lengths in `build_cf_html`; appending the
  tables to `body` keeps the two-pass offset math correct automatically
  (Req 28.5). Existing offset round-trip tests
  (`cf_html_offsets_round_trip_after_monospace_style`) extend to cover the
  tables-present case.

Shared table content (Req 28.6): both flavors derive their tables from the SAME
logical source the Markdown renderer already uses -- `model.markdown_sections`
when present, else `model.fields`/`model.tables`. For the Config panel this is
the heading-less intro table + one `[Group] (state)` + `Property | Value` table
per namespace group, so Word's HTML tables and Notepad's Markdown tables show
identical data and grouping, differing only in syntax.

### Error handling and edge cases
- A model with box layout but NO tabular content: both flavors carry just the box
  (no empty table block emitted) -- Req 28.7. The emitters guard on
  `markdown_sections`/`tables`/`fields` being non-empty, exactly as the Markdown
  renderer already does.
- A model with tables but no `body_lines` and no generic framed body: the box
  block is whatever `render_plain` produces for that model (its generic framed
  body); the tables follow. No new failure mode -- both are pure
  `ScreenModel -> String` functions that cannot fail.
- A collapsed Config group: its `markdown_sections` entry has an empty child
  table; the HTML emitter emits the heading with an empty table body, MIRRORING
  the Markdown flavor (which already does this) -- consistent across flavors.
- Secret redaction is unchanged: redaction happens on the model BEFORE either
  renderer runs (single-point, Req 27.4), so both the box body and the new tables
  carry only the redacted placeholder.

### Framework check (framework-conformance.md)
This builds ON the existing SCRM clipboard flavors (Req 23) and the
`ff-screen-model` renderer/`Table`/`MarkdownSection` model. NO new clipboard
flavor, NO new framework mechanism, NO change to `resolve_target` / dispatch, NO
change to the `ScreenProvider` trait or `ScreenModel` public type. The changes
are: (1) `render_markdown` no longer early-returns when `markdown_sections` is
present (it emits box-then-tables); (2) `build_cf_html` gains a `tables_html`
parameter and an HTML-table emitter is added to `render/html.rs`. Both are
additive within existing crates. `build_cf_html`'s signature change is INTERNAL
to `ff-desktop` (not a cross-crate public contract) -- see the gate summary's
owner-decision note; it is called from exactly one place
(`place_snapshot_composite`). No public API of `ff-screen-model` or `ff-scrm` is
reshaped. => No framework change; no owner framework confirmation required beyond
approving this gate.

### Testability
- Unit (ff-screen-model): `render_markdown` on a model WITH `markdown_sections`
  now contains BOTH a Unicode box glyph (U+250C family) AND the `| Property |
  Value |` table header (box-before-table ordering asserted by index).
- Unit (ff-screen-model): the new HTML-table emitter renders
  `markdown_sections`/`tables` as `<table>`/`<tr>`/`<td>` with group headings.
- Unit (ff-desktop): `build_cf_html` output for a Config fixture contains the
  `<img ... base64`, the monospace box `<span>` paragraphs, AND `<table>` markup,
  in that order; the four CF_HTML offsets still round-trip.
- Unit (ff-desktop): a shell-level test over a Config `ScreenModel` fixture
  asserting the `CF_UNICODETEXT` flavor string contains both the box block and
  the Markdown tables, and the `CF_HTML` flavor contains both the box block and
  HTML tables (the owner's two acceptance checks, Req 28.1/28.2).
- The real-paste-into-Word confirmation (Req 23.5d/23.5f style) remains a MANUAL
  check recorded in the TCR, since it cannot be driven headlessly.

## 12. Full-window snapshot FIDELITY (Req 29, CR-CH-115)

### Investigation verdict -- where the box is rendered today and WHY it is thin

Settled against the code (NOT assumed), reading
`crates/ff-screen-model/src/render/text.rs`,
`crates/ff-screen-model/src/model.rs`,
`crates/ff-desktop/src/screen_snapshot.rs`,
`crates/ff-desktop/src/shell/screen_provider.rs`,
`crates/ff-desktop/src/shell/screen_providers.rs`,
`crates/ff-desktop/src/shell/screen_providers_config.rs`,
`crates/ff-desktop/src/shell/commands_snapshot.rs`,
`crates/ff-desktop/src/shell/commands_scrm.rs`, and
`crates/ff-desktop/src/workspace_kind/mod.rs`:

- **One renderer, one model.** The box-drawn text is produced by
  `render/text.rs::render_plain(model, options)` from a single `ScreenModel`.
  The model already carries all the slots fidelity needs: `title` (rendered in
  the top border), `header_lines` (emitted under the title), `body_lines`
  (emitted verbatim inside the frame directly under the title separator),
  `fields`, `tables`, `messages`, `buttons`, `footer_lines`, `command_line`,
  `status_bar`, `cursor`, `dimensions`. So NO new model type is needed -- the
  thinness is in WHAT each per-Context builder populates and in two renderer
  details, not in the model's expressiveness.

- **Why the Alias (and every editor/panel) box is thin.** The per-Context
  builders in `screen_providers.rs` (`alias_editor_screen_model`,
  `theme_editor_screen_model`, `menus_editor_screen_model`,
  `keys_editor_screen_model`, `kinds_editor_screen_model`,
  `command_configurator_screen_model`, `search_results_screen_model`, ...)
  populate ONLY `model.fields` (producing the `SS . . . : SNAPSHOT` dot-leader
  lines seen in the owner's paste), optionally ONE `model.tables` entry, and
  `model.command_line = self.command_text`. They set NO `header_lines` (no window
  title bar, no `[XXX]` tab tag), NO `footer_lines` (no info bar, no F-key bar),
  and they leave the command line in `model.command_line`, which `render_plain`
  emits OUTSIDE the frame at the BOTTOM (the owner-observed "Command ===> ss"
  below the box). That is the entire gap: the editor/panel builders produce a
  title + fields + one table, with no chrome and a bottom-placed command line.

- **Why the table reads without separators.** `render/text.rs` renders a
  `Table` as `headers.join("  ")` and `row.join("  ")` -- space-joined, NOT
  `|`-separated, NOT fixed-width-padded, NOT full-width. So even the one table a
  builder DOES provide collapses to an un-separated column list (the owner's
  `Act  CE  Command  Alias ...` line). There is NO `[value v]` / `____` /
  padded-value cell convention anywhere today.

- **The POM/menu Context is the one rich builder, but deliberately lost its
  chrome.** `screen_snapshot.rs::menu_workspace_screen_model_full` +
  `screen_provider.rs::menu_screen_chrome` build a faithful two-column body (the
  `Command ===>` row FIRST as a `body_line`, option rows left + calendar right,
  the PF-key footer when `key_bar_visible`). But the 2026-11-01 Bug-2 correction
  deliberately REMOVED the window title bar / menu-bar echo / `[POM]` tag / title
  line from the menu chrome (see the doc comment on `menu_screen_chrome`). The
  owner's 2026-11-03 direction RE-ADDS the title bar + tab tag to ALL snapshots
  (Req 29.15), so the menu path must regain those two lines while keeping its
  two-column body.

### Where the chrome, tab tag, and F-key visibility come from (settled)

- **Window title bar text.** The application window title is the static
  `"FileForge Workbench"` set in `main.rs` (`ViewportBuilder::with_title(...)`
  and the `run_native` app name). The title-bar line (Req 29.1) is composed from
  this name + the plain-ASCII control glyphs `_ [] X`; it is a constant plus
  fixed glyphs, built in the shared helper (no egui read).
- **Tab tag `[XXX]`.** The active Context's terse bracketed tag comes from
  `workspace_kind::BuiltinKind::default_title()` (e.g. `[POM]`, `[THEME]`,
  `[CONFIG]`, `[ALIASES]`), reachable from the active tab via
  `BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)` -- the SAME source
  `screen_providers::active_display_title` already uses for the DISPLAY title,
  so the tag is one method call away in the shared helper. (The Alias Kind's tag
  is `[ALIASES]`, confirmed via `helpers.rs` key-map context mapping and the
  Kind registry.)
- **Workspace title line.** `screen_providers::active_display_title(self)` (the
  descriptive display title already used as `model.title`). Keep it in the border
  to avoid duplication (Req 29.3).
- **Command line value.** `self.command_text` (the live command field), already
  read by every builder; it must be folded into a TOP `body_line`
  `Command ===> <value><underscores>` (as the menu builder already does) rather
  than left in `model.command_line` (Req 29.4).
- **Information / status bar.** The live status text the shell shows; it maps to
  `model.status_bar.text`. (render_plain already emits `status_bar` below the
  frame; Req 29.10 keeps the info bar at the bottom. The shared helper fills it
  from the shell's status source.)
- **F-key bar visibility.** `self.key_bar_visible` (bool) plus
  `self.key_bar_scope` -- EXACTLY the state `menu_screen_chrome` already reads to
  decide whether to emit the PF-key footer. The shared helper reuses
  `screen_snapshot::key_bar_line(active_key_map, scope_segment, modifier)` and
  emits the footer ONLY when `key_bar_visible` is true (Req 29.11), mirroring the
  pfshow rule (Req 3.10).

### Snapshot vs capture: ONE shared path (settled)

Snapshot and capture do NOT diverge. `SNAPSHOT` (`commands_snapshot.rs`) and
`CAPTURE SCREEN` / auto-capture (`commands_scrm.rs::auto_capture_active_context`)
BOTH obtain their model from the SAME seam:
`active_screen_capture()` -> `ScreenCapture::Model` (snapshot) and its thin
`into_model()` wrapper `active_screen_model()` (capture). Both then render through
the SAME `ff-screen-model` renderer. There is NO separate capture renderer and NO
distinct capture code path for the box. Therefore improving the shared per-Context
builders + the shared `render/text.rs` grid rendering AUTOMATICALLY delivers the
same fidelity to capture (Req 29.14). Capture is "recording / possibly multi-frame"
only in that it appends successive models to a Collection; each model is produced
by the identical provider seam, so each recorded frame is the same faithful box.

### Chosen design -- a shared chrome helper + a grid-aware renderer

The fix is GENERAL (Req 29.12) and lives in TWO shared places:

1. **A shared shell helper `apply_full_window_chrome(model, ...)` (in
   `ff-desktop`, e.g. in `screen_providers.rs` or a new small
   `screen_chrome.rs`).** Every per-Context `*_screen_model()` builder calls it
   after populating its body. The helper:
   - PREPENDS the window-title-bar line and the `[XXX]` tab-tag line as the
     FIRST TWO `body_lines` (Req 29.1, 29.2) -- NOT as `header_lines`. This is a
     correction forced by the real renderer order (see "Renderer-order
     reconciliation" below): `render/text.rs::render_plain` emits `model.title`
     (the border) first, then `body_lines` directly under the title separator,
     then `header_lines` BELOW the body. So to make the rendering BEGIN with the
     title bar and place the tab tag directly under it, the two chrome lines must
     be the FIRST body lines, inserted ahead of any body the Context already
     produced (and ahead of the TOP command-line body line below). `header_lines`
     is NOT used for them.
   - folds `self.command_text` into a TOP `body_line`
     `Command ===> <value><underscores>` (inserted after the title-bar + tab-tag
     lines, before the data body) and clears `model.command_line` so the renderer
     does not also echo it at the bottom (Req 29.4). The underscore run matches
     the POM/menu command-line field width (Req 29.4) so it is consistent across
     Contexts.
   - sets `model.status_bar.text` from the live status (Req 29.10); the bottom
     info bar is always a defined line even when the status text is empty.
   - appends the F-key bar line to `footer_lines` ONLY when `key_bar_visible`
     (Req 29.11), reusing `screen_snapshot::key_bar_line`.
   The title stays in `model.title` (the border), emitted once (Req 29.3). This
   is pure logical-state composition, no egui (Req 29.13).

   **Renderer-order reconciliation (Req 29.1/29.2 against the real renderer).**
   Verified against `crates/ff-screen-model/src/render/text.rs::render_plain`:
   it emits, in order, the top border, the `model.title` row, the title
   separator, then each `model.body_lines` line (directly under the separator,
   no leading blank), then (after a blank) each `model.header_lines` line, then
   fields, tables, messages, buttons, and footer_lines, then the bottom border,
   then `command_line` and `status_bar` OUTSIDE the frame. Therefore appending
   the title bar + tab tag as `header_lines` would render them BELOW the body,
   failing "begin with the title bar" (29.1) and "directly under the title bar"
   (29.2). The chosen fix is option (b) of the two the review raised: fold the
   title-bar and tab-tag lines as the FIRST TWO `body_lines` (they already render
   directly under the title separator), keep the workspace title in the border
   emitted once (29.3), and leave `render_plain`'s section order UNCHANGED. No
   renderer reordering is required and no other Context's `header_lines`/
   `footer_lines` usage is disturbed. A unit test asserts the first in-frame body
   row is the title bar and the second is the tab tag (task 2G.2.1).

2. **A grid-aware `Table` rendering in `ff-screen-model/src/render/text.rs`
   (and the matching HTML emitter path, `render/html.rs`).** The text table
   renderer changes from `join("  ")` to a FIXED-WIDTH, `|`-separated,
   full-width grid (Req 29.5, 29.6, 29.6a) with:
   - a per-column width = max(header width, widest cell width in that column
     INCLUDING the scratch row) (Req 29.6); a `[value v]` or filled value wider
     than its header WIDENS the column and every row (header, data, scratch) pads
     to the widened width -- no truncation (Req 29.6 overflow rule);
   - `|` separators between cells (ASCII `|` is also the fallback, Req 5.2);
   - the always-present empty SCRATCH row (Req 29.7) -- see the provider-owned
     scratch-row decision below;
   - cell CONVENTIONS: a combo cell renders `[value v]` (empty `[ v]`) padded to
     the column width (Req 29.8); an empty editable text cell renders `____` to
     the field width, a filled one renders its padded value (Req 29.9).

   **Frame-width source (Req 29.6a).** The current `render_plain` derives
   `inner_width` from the WIDEST content line -- exactly the "collapse to the
   widest single content line" 29.6a forbids. The fix threads the frame width
   from `model.dimensions.columns` (the model already carries `Dimensions`,
   default 24x80; verified in `crates/ff-screen-model/src/model.rs`): the
   renderer takes `inner_width = max(widest content line, model.dimensions.columns)`
   so a narrow table still pads out to the frame width, while a table wider than
   the nominal columns still fits. A Context that knows its live panel column
   count MAY thread it onto `model.dimensions.columns`; absent that, the 80-column
   default applies. This keeps `Dimensions` PUBLIC SHAPE UNCHANGED (it is an
   existing field) and gives 29.6a a deterministic source. A unit test asserts a
   narrow grid pads to `model.dimensions.columns`, not to the widest content line
   (task 2G.1.1/2G.1.2).

   **Provider-owned scratch row AND cell conventions (makes the HTML flavor
   identical for free -- Req 29.7a, 29.8, 29.9).** To carry the combo-vs-text
   distinction AND the scratch row without reshaping the public `Table` type
   (framework-conformance), BOTH are applied by the PROVIDER when it builds
   `model.tables[*].rows`: the provider (which already knows which column is a
   combo) emits the `[value v]` / `[ v]` / `____` / padded-value STRINGS as the
   cell text, and appends the empty SCRATCH row (a full row of those empty-cell
   strings) as the LAST entry in `rows`. The text renderer stays a dumb
   fixed-width `|` grid and the HTML emitter stays a dumb `<td>` emitter; neither
   invents the conventions or the scratch row. Because both the plain-text
   renderer (`render/text.rs`) AND the HTML `<td>` emitter (`render/html.rs`,
   verified: it emits `<td>{cell}</td>` per `table.rows` cell at the
   `for row in &table.rows` loop) consume the SAME provider-owned `rows`, the
   `[value v]` / `____` strings and the trailing scratch row appear IDENTICALLY
   in both flavors with NO flavor-specific code (Req 29.7a, 29.8, 29.9). This is
   the key correction over an earlier draft that appended the scratch row inside
   the TEXT renderer only (which would never reach the HTML `<td>` path). This
   keeps `ScreenModel` / `Table` PUBLIC SHAPES UNCHANGED. (Alternative
   considered: add a `CellKind` enum to `Table` -- REJECTED for this gate because
   it reshapes a public type and is not needed; the provider-owned convention is
   sufficient and keeps both renderers generic. Recorded so the implementer does
   not reach for the enum.)

   NOTE: the grid work changes `render/text.rs` and `render/html.rs` so the text
   renderer emits `|`-separated fixed-width cells rather than `join("  ")`. The
   HTML `<td>` emitter already renders each provider cell verbatim, so the HTML
   flavor needs NO structural change to carry the conventions and scratch row --
   only the Phase 2F HTML-table path (Req 28) continues to render these rows, now
   with the fidelity cell strings in them. A Phase 2G test asserts the HTML
   flavor of a grid Context contains a `<td>` `[value v]`, a `<td>` `____`, and
   the trailing scratch-row `<tr>` (task 2G.1.6).

#### Combo-cell representation: best-practice rationale (owner question)

The owner's original `[---- ]v` was a sketch and asked for a recommendation. The
canonical convention is `[value v]`: square brackets signal a bounded
selection control (a closed set of choices), and the trailing `v` is the
universal down-chevron affordance for a dropdown, kept ASCII (`v`, not a Unicode
chevron) so it survives a plain-text paste and the ASCII-fallback frame. An empty
combo is `[ v]` so an un-selected dropdown still reads as a dropdown, not as an
empty text field (which is `____`). This cleanly distinguishes the THREE cell
kinds a user sees -- a dropdown (`[value v]`), an empty editable field (`____`),
and a filled editable field (the padded value) -- in monospaced text, in BOTH the
plain-text and HTML flavors identically (Req 29.8, 29.9).

### Error handling and edge cases

- A Context with NO tabular content (Req 29.16): the shared chrome helper still
  adds the title bar, tab tag, title, top command line, info bar, and F-key bar;
  the body is whatever fields/messages the Context renders. No empty grid is
  emitted.
- A Context whose status text is empty: the info-bar line is still a defined
  (possibly blank) bottom line; the renderer must not drop the bottom chrome just
  because the status text is empty (distinct from the F-key bar, which IS omitted
  when hidden).
- A ragged table (a row with fewer cells than headers): the renderer pads missing
  cells to empty (`____`-free) blanks so the grid stays aligned; the scratch row
  is a full row of empty cells.
- Secret redaction (Req 27.4) is unchanged: redaction happens on the model BEFORE
  the renderer runs, so both the grid cells and the chrome carry only redacted
  text.
- The POM/menu path (Req 29.15): it already composes its own body_lines and
  top command line; the shared helper adds ONLY the title-bar + tab-tag header
  lines (and leaves the menu's existing two-column body + PF footer intact), so
  the menu regains the two chrome lines Bug-2 removed without disturbing its
  body.

### Framework check (framework-conformance.md)

This BUILDS ON the existing SCRM/ff-screen-model rendering. NO new framework
mechanism: dispatch (`resolve_target` / `target_dispatch`) is untouched; the
`ScreenProvider` seam, `ScreenModel`, and `Table` PUBLIC SHAPES are UNCHANGED
(the combo/text cell convention is provider-owned cell text, not a new type); no
new navigation stack, no new persistence format, no new clipboard flavor. The
changes are: (1) a new shared shell helper that composes the common chrome from
LIVE LOGICAL STATE; (2) a grid-aware `render/text.rs` table rendering (fixed-width
`|` grid + scratch row) and the matching HTML-table grid treatment; (3) each
per-Context builder calls the helper. All additive within existing crates. =>
NO framework change; no owner framework confirmation required beyond approving
this gate. (If, during implementation, the combo/text distinction turns out to
NEED a `Table` reshape, that becomes a flagged framework change -- but the design
above avoids it.)

### Testability

- Unit (ff-screen-model): `render_plain` of a `Table` emits a `|`-separated,
  fixed-width header + rows + a trailing empty scratch row; a combo cell string
  `[value v]` and an empty editable cell `____` render verbatim and aligned.
- Unit (ff-screen-model): a NARROW `Table` pads out to `model.dimensions.columns`
  (the frame width) rather than to the widest content line (Req 29.6a); a value
  wider than its column header widens the column and pads every row including the
  scratch row (Req 29.6 overflow).
- Unit (ff-screen-model): the HTML flavor (`render/html.rs`) of the SAME grid
  model contains a `<td>` with `[value v]`, a `<td>` with `____`, and the
  trailing scratch-row `<tr>` of empty cells -- proving the conventions and
  scratch row are identical across flavors because the cell text is
  provider-owned (Req 29.7a, 29.8, 29.9).
- Unit (ff-screen-model / ff-desktop): the FIRST in-frame body row is the
  window-title-bar line and the SECOND is the `[XXX]` tab tag (Req 29.1, 29.2),
  confirming the title-bar + tab tag are the first two `body_lines` and the
  renderer order is honoured.
- Unit (ff-desktop): the shared chrome helper, applied to a table Context
  (Alias) model, yields `header_lines` containing the `FileForge Workbench ... _ [] X`
  title bar and the `[ALIASES]` tag; a TOP `body_line` `Command ===>`; a bottom
  info bar; and an F-key line ONLY when `key_bar_visible` is true.
- Unit (ff-desktop): the SAME assertions over a `markdown_sections` Context
  (Config) model, proving the fidelity is GENERAL (title bar, `[CONFIG]` tag,
  top command line, info bar, F-key-when-visible).
- Unit (ff-desktop): a CAPTURE-path test -- `active_screen_model()` (the capture
  seam) for the same Context yields a model that renders to the IDENTICAL
  faithful box as the SNAPSHOT seam, proving Req 29.14 (shared path).
- Full-shell egui_kittest (ff-desktop): over the live Alias Editor, a SNAPSHOT's
  box text contains the title-bar line, the `[ALIASES]` tag, a `|`-separated
  header, the scratch row, the info bar, and the F-key line per `key_bar_visible`.
- The real on-screen visual match remains a MANUAL check recorded in the TCR
  (pixel-exact appearance cannot be driven headlessly).

### Feeds the workspace-architecture alignment (recorded, Req 29.17)

This work is intended to FEED the upcoming workspace-architecture alignment, and
is a first-class input to that model. A Context that renders a faithful
full-window box is, by construction, a Context whose logical screen model is
complete: it exposes its title, its tab identity, its command line, its full
data grid, its status, and its key bar through the one `ScreenProvider` seam.
That is exactly the per-Context completeness the workspace-architecture model
needs to describe a workspace uniformly. So the common chrome helper + the grid
conventions defined here are not only a snapshot-fidelity fix; they are a probe
that proves each Context can describe its whole screen through the shared seam,
which the workspace-architecture alignment can then build on. Keeping the chrome
helper and grid rendering in the SHARED path (not per-Context) is what makes this
reusable as that model lands.

### Design-review responses (CR-CH-115 revision 1)

The design review `.agents/tasks/snapshot-fidelity-gate/design-review.json`
(verdict CHANGES_REQUESTED, 0 HIGH / 4 MEDIUM / 3 NIT) was addressed as follows.

- **Finding 1 (MEDIUM -- HTML-flavor identity of cell conventions + scratch row
  not designed or tested): ADDRESSED.** The scratch row is now a PROVIDER-OWNED
  row in `model.tables[*].rows` (Req 29.7 / 29.7a), not a text-renderer-only
  append, so both the plain-text `|` grid and the HTML `<td>` emitter (verified
  `render/html.rs` emits `<td>{cell}</td>` per row) carry the SAME `[value v]` /
  `____` strings and the trailing scratch-row `<tr>` with no flavor-specific
  code. The grid-renderer design bullet states this; Testability adds an HTML
  unit test; tasks add 2G.1.6 (ff-screen-model HTML `<td>`/`<tr>` test) and
  2G.4.3 (full-shell HTML-flavor assertion over the Alias Context).

- **Finding 2 (MEDIUM -- "begins with window title bar" not reconciled with the
  renderer order): ADDRESSED.** Verified `render/text.rs::render_plain` emits
  title (border) then `body_lines` then `header_lines` (below the body). Chosen
  option (b): the title-bar and tab-tag lines are the FIRST TWO `body_lines`
  (not `header_lines`), so they render directly under the title separator and the
  rendering BEGINS with the title bar. Req 29.1/29.2 now say "first/second
  in-frame content line" with an explicit renderer-order note; design.md adds a
  "Renderer-order reconciliation" subsection; task 2G.2.1 asserts the first body
  row is the title bar and the second is the tab tag; `render_plain`'s section
  order is left UNCHANGED.

- **Finding 3 (MEDIUM -- "fill the full panel/frame width" undefined, conflicts
  with per-column-max): ADDRESSED.** Split into two criteria: 29.6 = per-column
  width (max header/cells, with a widen-on-overflow rule); NEW 29.6a = the frame
  width, sourced from `model.dimensions.columns` (verified existing field,
  default 24x80), with `inner_width = max(widest content line,
  model.dimensions.columns)`. design.md adds a "Frame-width source" paragraph;
  task 2G.1.1a asserts a narrow grid pads to the frame width, not the widest
  content line.

- **Finding 4 (MEDIUM -- stale Wave F tasks contradict revised requirements):
  ADDRESSED.** Task 16.1 and 17.2 now specify a REAL WINDOW SCREENSHOT (Req 1.3 /
  23.2), not a text raster; task 16.3 now places the composite with NO picker
  (Req 4.1 / 4.7 RETIRED). The underlying requirement contradiction was also
  fixed: Req 9.14 is revised to a real window screenshot (was "rasterise the
  text render"), aligning it with Req 1.3 / 23.2.

- **Finding 5 (NIT -- command-line underscore width unspecified): ADDRESSED.**
  Req 29.4 now sets the underscore run to the existing POM/menu command-line
  field width; task 2G.2.2 restates it.

- **Finding 6 (NIT -- combo/text overflow unspecified): ADDRESSED.** Req 29.6
  (and 29.8/29.9) now state a value wider than its header WIDENS the column and
  every row (header, data, scratch) pads to the widened width -- no truncation.

- **Finding 7 (NIT -- gate-summary "open owner decisions" vs "settled" EARS):
  ADDRESSED.** Req 29.1 (title-bar glyph literal) and Req 29.10 (info-bar status
  source) now carry an explicit SETTLED-BUT-TUNABLE note (as Req 25.3 does), so
  an approver knows these are confirmable, not locked. The gate summary is
  updated to cross-reference the settled-but-tunable wording.
