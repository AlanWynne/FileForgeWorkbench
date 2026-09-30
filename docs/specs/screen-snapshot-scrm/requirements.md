# Requirements Document -- Screen Snapshot Service + Screen Collection and Replay Manager (SCRM), delivered as the first vertical of the ff-desktop decomposition (CR-NR-098)

## Introduction

This document consolidates TWO owner source documents into a single, gated
requirements + acceptance-criteria set:

- `docs/source-documents/FileForgeWorkbench-Screen-Snapshot-Architecture.md`
  -- the LOGICAL screen model + multi-format TEXT renderer proposal (ISPF-style:
  separate the visual rendering from the logical screen representation).
- `docs/source-documents/Screen-Collection-and-Replay-Manager-(SCRM).md`
  (FFWB-SRS-SCRM-001) -- the capture / collection / replay / export / evidence
  subsystem inspired by IBM PCOM Print Screen Collection.

It ALSO folds in the owner-requested optimisation of `ff-desktop`: the crate is a
~49,000-line monolithic binary (measured: 88 `.rs` files, ~48,935 lines; largest
`shell/tests.rs` 9,672, `shell/render.rs` 2,616, `tab_manager.rs` 2,211,
`shell/commands.rs` 2,119, `files_panel.rs` 1,998, `shell/mod.rs` 1,613,
`editor_panel.rs` 1,512, `shell/update.rs` 1,503, plus many 400-1,200-line
panels/dialogs). Every panel edit recompiles the whole crate (including the
9.6k-line test file) and relinks the `ffwb` binary against ~35 path dependencies,
which is the primary compile/link-time cost. SCRM is used here as the FIRST
extracted vertical: it introduces the clean "screen provider" seam that both (a)
lets the capture engine read a logical model without touching egui and (b) is one
of the seams that lets panels move out of the monolith. This ties the two owner
goals into one program of work.

### The hard owner constraint (first-class requirement)

Capture MUST be COPY/PASTE-ABLE TEXT derived from a LOGICAL screen model, NOT a
raster image (no jpeg/png as the primary artefact). A bitmap MAY be produced as an
OPTIONAL secondary artefact, but the authoritative capture is always selectable,
copyable text. This constraint drives Requirement 3 (Screen Model) and
Requirement 5 (Text-first renderers) and constrains all export requirements.

### Plugin vs core (owner decision, recorded)

SCRM is built LIKE a plugin, packaged LIKE core: the capture engine couples to the
rest of the app ONLY through a small `ScreenProvider` trait plus the existing
command-dispatch and `WorkspaceContext` seams; its logic lives in ordinary
workspace crates compiled into the app (always present, no install step, covered
by the normal gate). The clean trait seam means SCRM CAN later be lifted into a
true dynamic plugin with no rewrite, but v1 does not pay the dynamic-plugin cost
(ABI, discovery, versioning). See Requirement 2 and Requirement 12.

### Relationship to the current FFWB framework (conformance)

Per `framework-conformance.md`, SCRM builds ON the framework and introduces no
parallel mechanism:

- **Single command dispatch (CommandTarget).** Every `SNAPSHOT ...` and
  `CAPTURE ...` verb is a registered `Function` command resolved by
  `ff_command::resolve_target` and dispatched through `shell/target_dispatch.rs`.
  No bespoke `if upper == "..."` intercepts (Requirement 11).
- **WorkspaceContext + InteriorFocus single focus latch (CR-NR-078, CR-CH-023).**
  The Replay/Collection viewer is a Workspace Context implementing
  `WorkspaceContext`, dispatched via `render_workspace_context`, reporting its
  interior focus through `InteriorFocus` (Requirement 8, Requirement 13).
- **WorkspaceDescriptor session persistence (CR-CH-012).** A restored SCRM
  Context restores as a `CustomWorkspace { kind, params }` descriptor; transient
  replay position is not persisted (Requirement 8, Requirement 10).
- **Per-tab navigation, code-only built-ins, per-instance chrome** are untouched.

No framework CHANGE is proposed. If, during design, any criterion here cannot be
met without altering a core mechanism, that is a FRAMEWORK CHANGE to be surfaced
and owner-confirmed before proceeding (it is not implied by this document).

### Glossary

- **Screen_Model**: The authoritative logical representation of a Context's
  current screen: title, fields (label/value/attributes), tables, messages,
  status bar, command line, cursor position, dimensions. Independent of egui.
- **ScreenProvider**: The trait a Context implements to yield its Screen_Model on
  demand. The capture engine consumes only this; it never reads egui widgets.
- **Snapshot**: A single Screen_Model captured at a point in time, rendered to one
  or more text formats (and optionally a bitmap).
- **Capture**: A persisted Snapshot appended to a Collection (SCRM term).
- **Collection**: A named, persisted, ordered group of Captures.
- **Replay_Session**: Sequential review of a Collection's Captures.
- **Evidence_Package**: A documentation package generated from a Collection
  (user, date, session, collection, screens, optional test-case id + pass/fail).
- **Dialog_State (DIDL)**: A logical dialog state identifier, recorded with a
  Capture where available.
- **Snapshot_Format**: One of PlainText, Ansi, Markdown, Html, Yaml (AI).
- **ff-screen-model**: NEW crate -- Screen_Model types, `ScreenProvider` trait,
  and the text/ANSI/Markdown/HTML/YAML renderers (no egui, no SCRM logic).
- **ff-scrm**: NEW crate -- Collection/Capture data model, persistence, replay
  engine, exporters, evidence packages (depends on ff-screen-model; no egui).

### Source traceability

Each requirement below cites the originating source id(s): `[ARCH]` for the
Screen-Snapshot-Architecture doc, and `[SCRM-nnn]` for FFWB-SRS-SCRM-nnn.

---

## Requirement 1 -- Logical-model capture, not raster (the hard constraint)

**User story:** As a user documenting or reporting a screen, I want the capture to
be the actual on-screen TEXT so I can paste it into Word, VS Code, a defect
report, a Git commit, or an AI prompt and edit it, rather than an image I cannot
select.

#### Acceptance Criteria
1.1 WHEN a screen is captured THE system SHALL produce a text representation
    derived from the Screen_Model whose visible characters can be selected and
    copied as text. `[ARCH]`
1.2 THE system SHALL NOT require a raster image (jpeg/png/bmp) as the authoritative
    capture artefact. `[ARCH]` `[SCRM-026]`
1.3 WHERE a bitmap rendering is produced THE system SHALL treat it as an OPTIONAL
    secondary artefact stored alongside the text, never as a replacement for it.
    `[SCRM-026]`
1.4 WHEN a Capture is exported in any text-bearing format THE exported screen
    content SHALL remain selectable text (not an embedded image of text). `[ARCH]`

---

## Requirement 2 -- ScreenProvider seam (plugin-shaped, core-packaged)

**User story:** As the architecture owner, I want capture to read a logical model
through one small trait so it never fights egui and so any current or future
Context (POM, editor, dialogs, future CICS/ISPF emulation) can be captured
uniformly.

#### Acceptance Criteria
2.1 THE system SHALL define a `ScreenProvider` trait whose method returns the
    current `Screen_Model` for a Context. `[ARCH]`
2.2 THE capture engine SHALL obtain screen content ONLY via `ScreenProvider` and
    SHALL NOT inspect egui widget trees. `[ARCH]`
2.3 WHERE a Context does not implement `ScreenProvider` THE system SHALL report
    that the active Context is not capturable rather than capturing an empty or
    partial screen. `[ARCH]`
2.4 THE `ScreenProvider` trait and Screen_Model types SHALL live in a crate
    (`ff-screen-model`) that does NOT depend on egui or on `ff-scrm`, so the seam
    can later back a dynamic plugin without rework. (Plugin-like build.)
2.5 THE SCRM capability SHALL be compiled into the application (core packaging),
    available with no install step. (Core-like packaging.)

---

## Requirement 3 -- Screen Model content preservation

**User story:** As a user, I want the capture to preserve everything meaningful on
the screen so the record is faithful.

#### Acceptance Criteria
3.1 THE Screen_Model SHALL preserve all screen text. `[SCRM-019]`
3.2 THE Screen_Model SHALL preserve field attributes. `[SCRM-020]`
3.3 THE Screen_Model SHALL preserve colour attributes. `[SCRM-021]`
3.4 THE Screen_Model SHALL preserve highlighting attributes. `[SCRM-022]`
3.5 THE Screen_Model SHALL preserve field protection attributes. `[SCRM-023]`
3.6 THE Screen_Model SHALL record cursor position (row, column / focused field).
    `[SCRM-024]`
3.7 THE Screen_Model SHALL record screen dimensions. `[SCRM-025]`
3.8 THE Screen_Model SHALL represent title, fields (label/value), tables,
    messages, status bar, and command line as distinct logical elements. `[ARCH]`

---

## Requirement 4 -- Snapshot command (single-screen, no collection required)

**User story:** As a user, I want a one-shot `SNAPSHOT` of the current screen in
the format I choose, without having to start a collection.

#### Acceptance Criteria
4.1 WHEN the user issues `SNAPSHOT` THE system SHALL render the current
    Screen_Model to the default text format and place it where the user can copy
    it (see Requirement 6). `[ARCH]`
4.2 WHEN the user issues `SNAPSHOT TEXT` THE system SHALL render plain text.
    `[ARCH]`
4.3 WHEN the user issues `SNAPSHOT ANSI` THE system SHALL render text with ANSI
    colour escape sequences. `[ARCH]`
4.4 WHEN the user issues `SNAPSHOT MARKDOWN` (alias `SNAPSHOT MD`) THE system
    SHALL render Markdown (screen text in a fenced code block; tabular data as a
    Markdown table where a table element is present). `[ARCH]`
4.5 WHEN the user issues `SNAPSHOT HTML` THE system SHALL render HTML preserving
    colours and layout. `[ARCH]`
4.6 WHEN the user issues `SNAPSHOT YAML` (alias `SNAPSHOT AI`) THE system SHALL
    render a structured YAML document (title, fields as name/value, tables,
    buttons, cursor). `[ARCH]`

---

## Requirement 5 -- Text-first renderers (box drawing + fallback + colour)

**User story:** As a user, I want the text rendering to look like an ISPF panel and
still degrade gracefully where Unicode or colour is unwanted.

#### Acceptance Criteria
5.1 THE text renderer SHALL draw panel framing using Unicode box-drawing
    characters by default. `[ARCH]`
5.2 WHERE an ASCII-only fallback is selected THE text renderer SHALL draw framing
    using ASCII characters (`+`, `-`, `|`). `[ARCH]`
5.3 THE ANSI renderer SHALL emit standard SGR colour codes such that colours
    remain visible when pasted into VS Code, Windows Terminal, or a Linux
    terminal, and the text remains readable when pasted into a plain editor.
    `[ARCH]`
5.4 THE renderers SHALL operate purely from the Screen_Model and SHALL NOT invoke
    egui. `[ARCH]`
5.5 THE set of Snapshot_Format values SHALL be PlainText, Ansi, Markdown, Html,
    Yaml. `[ARCH]`

---

## Requirement 6 -- Delivery of a Snapshot to the user (copy path)

**User story:** As a user, I want a snapshot to land somewhere I can immediately
use it.

#### Acceptance Criteria
6.1 WHEN a `SNAPSHOT` completes THE system SHALL copy the rendered text to the
    system clipboard. `[ARCH]` (Relates to clipboard-operations.)
6.2 THE system SHALL confirm to the user (status/notification) that the snapshot
    was produced and its format. `[ARCH]`
6.3 WHERE the active Context is not capturable (Req 2.3) THE system SHALL report
    that clearly and copy nothing. `[ARCH]`

---

## Requirement 7 -- Collection management

**User story:** As a user, I want to group multiple captures into a named,
persisted collection.

#### Acceptance Criteria
7.1 WHEN the user starts screen collection (`CAPTURE START`) THE system SHALL
    create a new Collection. `[SCRM-001]`
7.2 WHEN a Collection is created THE system SHALL assign it a unique identifier.
    `[SCRM-002]`
7.3 THE system SHALL allow a Collection to be named. `[SCRM-003]`
7.4 THE system SHALL allow descriptive notes to be attached to a Collection.
    `[SCRM-004]`
7.5 THE system SHALL persist Collections between application sessions.
    `[SCRM-005]`
7.6 WHEN the user issues `CAPTURE STOP` THE system SHALL end the active collection
    session (no further automatic captures) without deleting the Collection.
    `[SCRM-009 implied]`
7.7 WHEN the user issues `CAPTURE STATUS` THE system SHALL report whether
    collection is active, the active Collection, capture count, and capture mode.
    `[SCRM commands]`

---

## Requirement 8 -- Manual capture into a Collection

#### Acceptance Criteria
8.1 WHEN the user issues `CAPTURE SCREEN` THE system SHALL capture the current
    screen as a Snapshot. `[SCRM-006]`
8.2 WHEN a screen is captured THE system SHALL append it to the active Collection.
    `[SCRM-007]`
8.3 THE system SHALL assign sequential numbering to Captures within a Collection.
    `[SCRM-008]`
8.4 WHERE no Collection is active WHEN `CAPTURE SCREEN` (or the first automatic
    capture) is issued THE system SHALL auto-start a new default Collection,
    named by timestamp, and append the capture to it. (Owner-confirmed Q1.)
    `[SCRM-006/007 gap]`

---

## Requirement 9 -- Automatic and conditional capture

**User story:** As a user documenting a navigation flow, I want captures taken
automatically on screen transitions, optionally filtered by rules.

#### Acceptance Criteria
9.1 WHEN Automatic Capture Mode is enabled THE system SHALL capture each screen
    transition. `[SCRM-009]`
9.2 WHEN an ENTER key causes a screen change THE system SHALL create a Capture.
    `[SCRM-010]`
9.3 WHEN a PF key causes a screen change THE system SHALL create a Capture.
    `[SCRM-011]`
9.4 WHEN a PA key causes a screen change THE system SHALL create a Capture.
    `[SCRM-012]`
9.5 WHEN a menu selection causes navigation THE system SHALL create a Capture.
    `[SCRM-013]`
9.6 THE system SHALL support a configurable capture interval. `[SCRM-014]`
9.7 THE system SHALL support capture rules, and WHEN a rule evaluates TRUE THE
    system SHALL create a Capture. `[SCRM-015][SCRM-016]`
9.8 THE system SHALL support rule conditions on: Screen Name, Program Name,
    Message Class, Transaction ID, Dataset Name, User ID. `[SCRM-017]`
9.9 THE system SHALL support multiple active rules simultaneously. `[SCRM-018]`

---

## Requirement 10 -- Replay

**User story:** As a user, I want to step through a Collection's captures.

#### Acceptance Criteria
10.1 THE system SHALL replay collected screens (`CAPTURE REPLAY`). `[SCRM-030]`
10.2 THE system SHALL display Captures in collection sequence. `[SCRM-031]`
10.3 THE system SHALL support First, Previous, Next, Last navigation. `[SCRM-032]`
10.4 THE system SHALL support automatic playback. `[SCRM-033]`
10.5 THE system SHALL support adjustable playback speed. `[SCRM-034]`
10.6 THE system SHALL display capture timestamps during replay. `[SCRM-035]`
10.7 THE system SHALL display elapsed time between captures. `[SCRM-036]`
10.8 THE replayed screen content SHALL be shown as selectable text (consistent
     with Requirement 1). `[ARCH]`

---

## Requirement 11 -- Commands (all via the single dispatch path)

**User story:** As a user, I want every SCRM/snapshot action available as a typed
command, resolved the same way as every other FFWB command.

#### Acceptance Criteria
11.1 THE system SHALL register the commands: `CAPTURE START`, `CAPTURE STOP`,
     `CAPTURE SCREEN`, `CAPTURE STATUS`, `CAPTURE LIST`, `CAPTURE OPEN`,
     `CAPTURE SAVE`, `CAPTURE LOAD`, `CAPTURE REPLAY`, `CAPTURE EXPORT TEXT`,
     `CAPTURE EXPORT MD`, `CAPTURE EXPORT HTML`, `CAPTURE EXPORT PDF`,
     `CAPTURE EXPORT PDF PROTECTED`, `CAPTURE PURGE`. `[SCRM commands]`
     (`CAPTURE EXPORT PDF PROTECTED` delivers Requirement 20.)
11.2 THE system SHALL register the commands: `SNAPSHOT`, `SNAPSHOT TEXT`,
     `SNAPSHOT ANSI`, `SNAPSHOT MARKDOWN` (`MD`), `SNAPSHOT HTML`,
     `SNAPSHOT YAML` (`AI`). `[ARCH]`
11.3 EACH command SHALL resolve through `ff_command::resolve_target` to a
     `Function` target and dispatch through the shared dispatch path; NO bespoke
     command intercept SHALL be added. (framework-conformance.)
11.4 WHERE a menu item, toolbar button, or shortcut triggers an SCRM/snapshot
     action THE affordance SHALL invoke the SAME command as the typed form
     (command parity). (framework-conformance 1b.)

---

## Requirement 12 -- Export

**User story:** As a user, I want to export a Collection into portable formats.

#### Acceptance Criteria
12.1 THE system SHALL export a Collection as plain text. `[SCRM-037]`
12.2 THE system SHALL export a Collection as Markdown, with screen text in fenced
     code blocks. `[SCRM-038][SCRM-039]`
12.3 THE system SHALL export a Collection as HTML, rendering colours and
     attributes. `[SCRM-040][SCRM-041]`
12.4 THE system SHALL export a Collection as PDF with a title page, table of
     contents, screen index, and page numbering. `[SCRM-042..046]`
12.5 THE system SHALL export a Collection to a native zip-compatible archive whose
     layout contains `collection.yaml`, `screens/`, `images/`, `metadata/`.
     `[SCRM-047][SCRM-048][SCRM-049]`
12.6 WHERE a text-bearing export format is used THE screen content SHALL remain
     selectable text (Requirement 1.4). Note PDF: text SHALL be real text, not a
     rasterised page image. `[ARCH]`
12.7 WHEN exporting to PDF THE system SHALL support a PROTECTED PDF option that
     is copy-enabled but edit-locked (see Requirement 20). (owner: unalterable
     activity record.)

---

## Requirement 20 -- Protected / tamper-evident PDF (unalterable activity record)

**User story:** As an auditor or support engineer, I want to export a Collection
as a PDF that reviewers can still SELECT and COPY text from, but cannot EDIT, so
it can serve as an unalterable record of the activities carried out.

### Enforcement note (honesty about PDF security)

PDF has two distinct, complementary mechanisms, and this requirement uses both
deliberately -- do not conflate them:

- **Permission flags** (the "editing not allowed / copying allowed" bits). These
  are ADVISORY: compliant viewers honour them, and they are only meaningfully
  enforced when the PDF is encrypted with an OWNER password. They deter casual
  editing but are not proof against a determined party with PDF tooling.
- **Integrity (hash / digital signature).** A cryptographic hash or a digital
  signature makes any later modification DETECTABLE -- the record is not
  "impossible to alter" but "impossible to alter WITHOUT DETECTION", which is the
  real audit property. This is the strong guarantee.

Therefore the requirement is framed as "copy-enabled, edit-locked, and
tamper-EVIDENT", NOT "physically uneditable".

#### Acceptance Criteria
20.1 WHEN the user requests a protected PDF export THE system SHALL produce a PDF
     whose permission flags ALLOW text copying/extraction and DISALLOW content
     editing, annotation, and page assembly.
20.2 THE system SHALL enforce those permissions by encrypting the PDF with an
     owner password, so the edit-lock is honoured by compliant viewers (per the
     Enforcement note).
20.2a THE system SHALL support an OPTIONAL user (open/read) password: when
     supplied, opening the PDF SHALL require it (restricting who can read the
     record); when omitted, the PDF SHALL open freely while remaining edit-locked.
     (Owner-confirmed Q7.)
20.3 THE protected PDF SHALL keep its screen content as SELECTABLE, COPYABLE text
     (consistent with Requirement 1 and 12.6) -- copy is permitted, edit is not.
20.4 THE system SHALL embed a cryptographic HASH of the source Collection content
     in the PDF and the accompanying Evidence_Package metadata, so that any later
     alteration of the PDF is DETECTABLE. (Owner-confirmed Q6: hash in v1.)
20.4a THE system SHALL provide a configurable OPTIONAL digital signature (with a
     signing certificate/identity) as an add-on to the hash, deferred as a v1
     configurable option. (Owner-confirmed Q6: signature as add-on.)
20.5 THE system SHALL record, in the PDF metadata and the Evidence_Package, the
     generating user, timestamp, Collection id, and the integrity value, so the
     record is self-describing.
20.6 WHERE the user has NOT requested protection THE PDF export SHALL behave as
     Requirement 12.4 (unprotected, editable). Protection SHALL be opt-in.
20.7 THE protected-PDF export SHALL be reachable via a command (e.g.
     `CAPTURE EXPORT PDF PROTECTED`) resolved through the single dispatch path,
     with command parity for any UI affordance. (framework-conformance;
     extends Requirement 11.)

---

## Requirement 13 -- Sensitive information masking

#### Acceptance Criteria
13.1 A field SHALL be classified as sensitive when EITHER (a) the providing
     Context marks it sensitive in the Screen_Model -- notably password/hidden/
     masked-input fields -- OR (b) a configurable masking rule matches it.
     (Owner-confirmed Q5: "passwords, hidden", plus config rules.) `[SCRM-027]`
13.2 WHEN a field is classified as sensitive THE system SHALL support masking that
     field during export. `[SCRM-027]`
13.3 THE system SHALL support configurable masking rules. `[SCRM-028]`
13.4 THE system SHALL allow an export to be produced WITH or WITHOUT masking.
     `[SCRM-029]`
13.5 THE Screen_Model SHALL carry a per-field "sensitive" attribute so a Context
     (e.g. a password entry field) can mark content sensitive at capture time,
     independent of any export-time rule. (Extends Requirement 3.)

---

## Requirement 14 -- Evidence packages

#### Acceptance Criteria
14.1 THE system SHALL support creation of an Evidence_Package from a Collection.
     `[SCRM-050]`
14.2 THE Evidence_Package SHALL include user, date, session, collection, and
     screens. `[SCRM-051]`
14.3 THE system SHALL support inclusion of a test-case identifier. `[SCRM-052]`
14.4 THE system SHALL support pass/fail status indicators. `[SCRM-053]`
14.5 THE system SHALL support producing the Evidence_Package as a PROTECTED,
     tamper-evident PDF (Requirement 20), so the evidence is a copy-enabled,
     edit-locked, integrity-checkable record. (owner: unalterable activity
     record.)

---

## Requirement 15 -- DIDL / dialog-state integration

#### Acceptance Criteria
15.1 WHERE a dialog state is available THE system SHALL record the associated
     dialog state with the Capture. `[SCRM-054]`
15.2 THE system SHALL store the DIDL state identifier. `[SCRM-055]`
15.3 THE system SHALL permit replay filtering by dialog state. `[SCRM-056]`
15.4 THE system SHALL permit export of state-transition history. `[SCRM-057]`

---

## Requirement 16 -- SCRM viewer as a Workspace Context (framework conformance)

**User story:** As a user, I want the collection/replay UI to behave like every
other FFWB workspace (tab, focus, persistence).

#### Acceptance Criteria
16.1 THE SCRM collection/replay UI SHALL be a Workspace Context implementing
     `WorkspaceContext`, dispatched via `render_workspace_context`. (CR-NR-078.)
16.2 THE Context SHALL report its interior focus through `InteriorFocus` (its
     first interior control given a stable `egui::Id`), and SHALL ship the
     mandatory full-shell first-Tab egui_kittest test. (CR-CH-023,
     workspace-conformance.)
16.3 THE Context SHALL persist and restore as a `CustomWorkspace { kind, params }`
     `WorkspaceDescriptor`; transient replay position SHALL NOT be persisted.
     (CR-CH-012.)

---

## Requirement 17 -- Data model

#### Acceptance Criteria
17.1 THE system SHALL model a `ScreenCollection` with: CollectionId, Name,
     Description, CreatedBy, CreatedTimestamp, SessionId, Screens[]. `[SCRM-10]`
17.2 THE system SHALL model a `ScreenCapture` with: CaptureId, SequenceNumber,
     Timestamp, ScreenName, ProgramName, DialogState, CursorRow, CursorColumn,
     TextBuffer, AttributeBuffer, ImageBuffer (optional), Notes, Metadata.
     `[SCRM-10]`
17.3 THE persisted Collection format SHALL be the native zip-compatible archive of
     Requirement 12.5. `[SCRM-048]`

---

## Requirement 18 -- Non-functional

#### Acceptance Criteria
18.1 THE subsystem SHALL support Collections containing at least 10,000 Captures.
     `[SCRM-NFR-001]`
18.2 Capture operations SHALL NOT visibly interrupt user interaction. `[SCRM-NFR-002]`
18.3 THE subsystem SHALL support asynchronous export processing. `[SCRM-NFR-003]`
18.4 THE subsystem SHALL support crash recovery for an active Collection.
     `[SCRM-NFR-004]`
18.5 THE subsystem SHALL be structured to allow future integration with automated
     test frameworks and AI-assisted workflow analysis (no coupling that would
     preclude it). `[SCRM-NFR-005][SCRM-NFR-006]`

---

## Requirement 19 -- ff-desktop decomposition (delivered together with SCRM)

**User story:** As the maintainer, I want ff-desktop to compile and link faster by
being split into smaller crates, and I want SCRM to be the first vertical that
proves the new boundaries rather than more weight in the monolith.

#### Acceptance Criteria (behaviour-preserving refactor; see design.md for waves)
19.1 THE SCRM logical model + renderers SHALL ship in a NEW `ff-screen-model`
     crate (no egui, no SCRM logic), and the SCRM collection/replay/export/
     evidence logic in a NEW `ff-scrm` crate (no egui), so the bulk of the new
     capability compiles OUTSIDE the `ff-desktop` binary.
19.2 THE `ff-desktop` crate SHALL depend on `ff-screen-model` and `ff-scrm` only
     for the thin Context/command wiring (the egui viewer + command handlers).
19.3 THE decomposition SHALL be behaviour-preserving for all EXISTING
     functionality: each extraction wave SHALL keep `verify.ps1` (full) green with
     no observable behaviour change (a pure REFACTOR per workflow.md; no new
     acceptance criteria for moved code).
19.4 EACH extraction wave SHALL be independently shippable and SHALL reduce
     `ff-desktop` compiled size (fewer lines / fewer rebuild triggers), measured
     before/after (line count and, where practical, incremental rebuild time).
19.5 THE `ScreenProvider` seam (Requirement 2) SHALL be usable as the pattern for
     subsequent panel/dialog extractions from `ff-desktop` (documented in
     design.md as the extraction template), so the SCRM work and the ff-desktop
     optimisation are one program, not two.
19.6 NO extraction SHALL introduce a parallel command-dispatch, navigation,
     focus, or persistence mechanism (framework-conformance).

---

## Out of scope (v1) / future enhancements

Recorded from the SCRM source "Future Enhancements"; NOT gated here, listed so the
architecture does not preclude them: screen differencing, session recording, user
activity heat maps, process mining, AI-generated walkthroughs/training manuals,
voice-narrated replay, automated test evidence generation, PowerPoint export,
Git-managed collections. `[SCRM section 12]`

---

## Open questions -- owner review (RESOLVED except two deferred to design.md)

- Q1 (Req 8.4): RESOLVED -- auto-start a default Collection (named by timestamp)
  when `CAPTURE SCREEN`/first auto-capture occurs with no active Collection.
  Baked into Req 8.4.
- Q2 (Req 12.4/12.6 PDF): RESOLVED -- PDF text MUST be selectable; a rasterised
  PDF is excluded. Baked into Req 12.6.
- Q3 (Req 9): DEFERRED TO design.md -- automatic capture fires on the framework's
  real screen-change events. Proposed set (to be enumerated and confirmed against
  the code in design.md): a `navigate_to` Context transform/push (per-tab
  Navigation_Stack), a successful command completion that changes the screen, and
  a menu-option activation (`open_menu_by_name`). The source doc's ENTER/PF/PA
  triggers map onto these because keys dispatch through the same command path.
- Q4 (Req 2/19): RESOLVED -- crate names confirmed `ff-screen-model` and
  `ff-scrm`.
- Q5 (masking, Req 13): RESOLVED -- a field is sensitive if the Context marks it
  sensitive in the Screen_Model (passwords/hidden/masked input) OR a config rule
  matches. Baked into Req 13.1 and Req 13.5.
- Q6 (Req 20 protected PDF): RESOLVED -- v1 = owner-password permission lock +
  content hash (tamper-EVIDENT); digital signature is a configurable add-on. Baked
  into Req 20.4 / 20.4a. (Enforcement note retained: the edit-lock is advisory/
  deterrent-grade; the hash is what makes tampering detectable.)
- Q7 (Req 20.2): RESOLVED -- optional user (open/read) password supported; when
  omitted the PDF opens freely but stays edit-locked. Baked into Req 20.2a.
- Q8 (Req 20 PDF library): DEFERRED TO design.md -- select a Rust PDF crate that
  supports real selectable text (Q2) AND AES encryption/permission flags, noting
  which also support digital signatures (for the Q6 add-on). design.md will
  present a shortlist with the tradeoff for owner confirmation.

## Numbering note

Requirement 20 (Protected / tamper-evident PDF) was added after the initial draft
in response to the owner's "protected PDF" request; it is placed next to
Requirement 12 (Export) for readability but keeps the next free sequential number
(20) rather than renumbering existing requirements. Requirements 12.7 and 14.5
cross-reference it. A later editorial pass MAY renumber into strict sequence
before the gate closes.
