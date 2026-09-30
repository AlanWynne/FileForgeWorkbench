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
  - `ScreenCollection` / `ScreenCapture` data model (Req 17).
  - Persistence: the native zip archive (`collection.yaml`, `screens/`, `images/`,
    `metadata/`) (Req 12.5, 17.3) + crash-recovery journal (Req 18.4).
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
Req 16->S6; Req 17->S3; Req 18->S9; Req 19->S8; Req 20->S5. tasks.md will carry
the per-criterion task breakdown and TCR rows.
