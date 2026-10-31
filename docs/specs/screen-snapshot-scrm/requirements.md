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

### The owner constraint (first-class requirement) -- REVISED (CR-NR-114, multi-flavor clipboard)

The capture text MUST be derived from a LOGICAL screen model, NOT grabbed from
egui pixels. That part of the constraint is unchanged and still drives
Requirement 3 (Screen Model) and Requirement 5 (Text-first renderers).

What is REVISED (owner-confirmed this gate): the earlier wording required the
copyable TEXT to be the single authoritative artefact, with a bitmap only as an
OPTIONAL SECONDARY artefact. The owner has relaxed this to a MULTI-FLAVOR
CLIPBOARD model in which the picture and the text are CO-EQUAL clipboard flavors,
neither subordinate:

- A single SNAPSHOT places BOTH representations on the clipboard at once: the PNG
  picture of the screen AND the box-drawn ANSI/TEXT rendering (both derived from
  the same logical screen model).
- The DESTINATION application chooses which flavor it can consume: a rich target
  (Word, PowerPoint) takes BOTH (the image and the monospaced text block); a
  plain-text target (Notepad, a code editor) takes ONLY the text and drops the
  picture -- the familiar behaviour of pasting mixed image+text content from a
  word processor into a text editor.

The text is therefore no longer the sole authoritative artefact for the
CLIPBOARD; it is one of two co-equal flavors offered simultaneously. The
logical-model-first rule (text and picture both rendered FROM the logical model,
never a raw egui screengrab) remains in force and still constrains all export
requirements. See Requirement 23 for the clipboard-flavor acceptance criteria.

NOTE (implementation feasibility, to confirm during design): offering image and
text clipboard flavors simultaneously is well-supported by the OS clipboard
(e.g. Windows CF_DIB/PNG + CF_UNICODETEXT), but the app's current clipboard
backend may expose text only. This revised constraint is subject to the clipboard
layer supporting an image flavor; if it does not today, adding that capability is
part of the SCRM stream, not a reason to weaken the requirement.

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
1.1 WHEN a screen is captured THE system SHALL produce a TEXT representation
    derived from the Screen_Model whose visible characters can be selected and
    copied as text. THE text representation SHALL use proper Unicode box-drawing
    characters (`+U250x` family: `|_ - | _|` i.e. the box glyphs) for frames and
    separators, NOT the ASCII `+`/`-`/`|` fallback, so it reads as a real screen.
    (Unicode box-drawing in captured OUTPUT is deliberate runtime product content,
    exempt from the ASCII-only documentation rule.) `[ARCH]`
1.2 THE TEXT representation SHALL remain the authoritative SELECTABLE-TEXT capture
    (not a raster image of text). A raster PICTURE flavor (Requirement 23) is
    produced IN ADDITION, for pasting a visual of the window; it does not replace
    the text. `[ARCH]` `[SCRM-026]`
1.3 REVISED (CR-NR-114): the PICTURE flavor SHALL be a REAL SCREENSHOT of the
    FileForge application WINDOW (the window's actual rendered pixels -- fonts,
    colours, layout, box-drawing as drawn on screen), NOT a text-painted raster of
    the Screen_Model. This DELIBERATELY REVERSES the former "logical-model-first,
    never a screengrab" constraint FOR THE PICTURE FLAVOR ONLY (the earlier
    text-rasterised PNG looked identical to the text and added no value). The TEXT
    and HTML flavors remain Screen_Model-derived; only the picture becomes a true
    window screengrab. The screengrab captures the WINDOW, not the whole desktop.
    (Owner-confirmed framework/constraint change.) `[ARCH]` `[SCRM-026]`
1.4 WHEN a Capture is exported in any text-bearing format THE exported screen
    content SHALL remain selectable text (not an embedded image of text); a
    picture MAY be embedded ALONGSIDE the selectable text (e.g. the HTML composite
    of Requirement 23), never as a replacement for it. `[ARCH]`

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
3.9 WHEN the POM / menu `ScreenProvider` yields its Screen_Model THE model SHALL
    include the FULL on-screen chrome a user sees, not a trimmed menu-option list:
    the window chrome / title bar, the menu bar, the tab header (the `[XXX]`
    Workspace Kind tag), the title line (the descriptive display title), the
    Command line WITH its current value, the POM option rows WITH their command
    values, the calendar, and the PF-key footer. (Extends Req 3.8; closes the
    "captures a trimmed model" gap.) `[master-directive]`
3.10 WHERE the PF-key footer is SHOWN (pfshow on, per the active
    `key_bar_visible` / `key_bar_scope`) THE Screen_Model SHALL include the PF-key
    footer line(s); WHERE it is HIDDEN (pfshow off) THE model SHALL omit the
    PF-key footer, so the capture faithfully reflects the pfshow state. (Extends
    Req 3.8.) `[master-directive]`
3.11 THE command-line element SHALL carry the command line's current VALUE (the
    text the user has typed), not merely the command-line label, so a capture
    records what was being entered. (Confirms existing `model.command_line`
    behaviour; extends Req 3.8.) `[master-directive]`
3.12 EACH POM option row SHALL carry both its displayed text (mnemonic +
    description) AND its command value, so the capture records what each option
    invokes. (Confirms existing behaviour; extends Req 3.8.) `[master-directive]`
3.13 THE enriched POM / menu Screen_Model SHALL be built purely from the logical
    model (menu-workspace state, key-bar state, calendar state), NOT by reading
    egui widgets (consistent with Req 2.2, Req 5.4). `[master-directive]`

---

## Requirement 4 -- Snapshot command (single-screen, no collection required)

**User story:** As a user, I want a one-shot `SNAPSHOT` of the current screen in
the format I choose, without having to start a collection.

#### Acceptance Criteria
4.1 WHEN the user issues `SNAPSHOT` with NO format argument THE system SHALL
    place the COMPOSITE clipboard payload (the PNG picture AND the Markdown-fenced
    text, Requirement 23) on the clipboard directly, WITHOUT any format picker.
    (Owner-confirmed CR-NR-114 clipboard revision: the composite carries both
    flavors, so asking the user to choose a format is redundant.) `[ARCH]`
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
4.7 RETIRED (CR-NR-114). The former no-argument FORMAT PICKER is removed: bare
    `SNAPSHOT` now always places the composite clipboard payload (Req 4.1, Req 23)
    and there is nothing to choose, so a picker is redundant. No format-picker
    dialog SHALL be presented. (Supersedes the earlier owner decision that bare
    SNAPSHOT pops a picker.) `[forward-plan]`
4.8 WHEN the user issues `SNAPSHOT <format>` with an EXPLICIT format
    (TEXT/ANSI/MARKDOWN/MD/HTML/YAML/AI/PNG/PDF) THE system SHALL render/produce
    that single format directly -- these explicit verbs are RETAINED for
    single-format output (e.g. writing one format to a file). Only the no-argument
    picker is removed. `[forward-plan]`

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
9.10 WHEN an automatic Capture is created because a command caused a screen
     transition THE system SHALL record, on that Capture, the triggering command
     as resolved through the single dispatch path (the canonical command string
     and its resolved `CommandTarget` kind), so the Collection is a command + screen
     timeline and not a screen-only timeline. (Owner-confirmed recorder scope;
     amends Req 9, extends Req 17.2.) `[recorder]`
9.11 THE triggering-command record SHALL be obtained from the existing
     command-dispatch seam (`ff_command::resolve_target` / `shell/target_dispatch.rs`)
     and SHALL NOT be gathered via a bespoke command intercept. (framework-conformance;
     extends Req 11.3.) `[recorder]`
9.12 WHERE an automatic Capture is created by a non-command cause (a timed
     interval per Req 9.6, or a rule per Req 9.7) THE triggering-command record
     SHALL be absent for that Capture, and absence SHALL be a valid, explicitly
     representable state (not an empty-string sentinel). `[recorder]`
9.13 WHEN capture START records a per-transition Capture THE Capture SHALL store,
     alongside the triggering-command metadata (Req 9.10), the transition's
     screen in BOTH text form (the ANSI/TEXT rendering) AND, where the capture
     configuration enables it, a picture rendering (the bitmap of Requirement 23)
     and a Markdown rendering, so each Capture is a bitmap + ANSI + Markdown +
     command record. This CONFIRMS and extends the Wave 2R recorder (Req 9.10-9.12,
     17.2a) to carry the picture + Markdown variants per Capture. `[forward-plan]`
9.14 THE per-transition bitmap of Req 9.13 SHALL be produced by rasterising the
     logical TEXT render (Requirement 23), NOT by screen-grabbing egui
     (consistent with Req 1 and Req 2.2). `[forward-plan]`

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
10.9 WHERE a Capture carries a triggering command (Req 9.10) THE replay view
     SHALL display that command as the cause of the transition INTO that Capture
     (e.g. a "caused by: <command>" line shown with the screen), as selectable
     text. `[recorder]`
10.10 WHERE a Capture has no triggering command (Req 9.12) THE replay view SHALL
     omit the cause line for that Capture rather than showing an empty one.
     `[recorder]`
10.11 A captured session SHALL be SAVED TO A FILE (the native zip-compatible
     Collection archive, Req 12.5 / 17.3) so that it can be REOPENED and replayed
     later, independent of the live session that captured it. Replay SHALL NOT be
     limited to the active in-memory collection. (Owner-confirmed D(replay/save):
     the real requirement is save-to-file + reopen, not whether the viewer screen
     self-captures.) `[forward-plan]`
10.12 THE system SHALL LIST the saved sessions and let the user OPEN / SELECT one
     from that list for replay (a saved-session picker), reachable through the
     single command front door; selecting a session SHALL open it in the replay
     Context (Requirement 16). (Owner-confirmed D(replay/save).) `[forward-plan]`
10.13 A saved session SHALL be EXPORTABLE to PDF (now, reusing the existing
     `ff-scrm/pdf.rs` path, Req 12.4) and to ODT / ODP (later, deferred), via the
     existing `CAPTURE EXPORT ...` commands (command parity, Req 11.4); the export
     SHALL keep screen content as selectable text where the format supports it
     (Req 1.4, 12.6). (Owner-confirmed D(replay/save) + D(formats).) `[forward-plan]`
10.14 THE replay story SHALL therefore be: capture STOP -> Save-As (Requirement
     24) names the session and picks the output type -> the saved session is
     listed and can be opened/selected for replay (Req 10.12) -> the saved session
     can be exported to PDF now / ODT / ODP later (Req 10.13). The requirement is
     centred on save-to-file + reopen/select + export, NOT on the viewer screen
     self-capturing. `[forward-plan]`

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
     TextBuffer, AttributeBuffer, ImageBuffer (optional), Notes, Metadata,
     TriggeringCommand (optional). `[SCRM-10]` `[recorder]`
17.2a THE optional `TriggeringCommand` SHALL carry the canonical command string
     AND the resolved `CommandTarget` kind (Req 9.10), and SHALL be representable
     as absent (Req 9.12). It SHALL round-trip through the persisted Collection
     format (Req 17.3) unchanged. `[recorder]`
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

## Requirement 21 -- Export a recording as a macro (DEFERRED, NOT v1)

**User story:** As a user, I want to turn a command + screen recording into a
re-runnable macro, so a captured navigation flow can be replayed as ACTIONS, not
just reviewed as screens.

### Status

DEFERRED by owner decision. This requirement is recorded so the v1 data model is
designed to make it possible, but the macro EXPORTER itself is NOT built in v1.
v1 delivers only the captured triggering-command data (Req 9.10, 17.2a) in a
shape a later exporter can consume. Tracked as NOT-COVERED in the coverage record.

#### Acceptance Criteria (deferred)
21.1 THE system SHALL (in a later phase) export the ordered sequence of
     `TriggeringCommand` entries from a Collection as a macro artefact compatible
     with the FFWB macro facility (Lua macro engine / Macro Library). `[recorder-deferred]`
21.2 THE v1 `TriggeringCommand` schema (Req 17.2a) SHALL be sufficient to drive
     21.1 without a further data-model change -- i.e. it SHALL capture enough to
     reconstruct each command as issued. THIS is the only part of Requirement 21
     that v1 MUST satisfy; 21.1 is the deferred exporter. `[recorder]`
21.3 WHERE a Capture has no triggering command (Req 9.12) THE macro export SHALL
     skip it rather than emit an empty action. `[recorder-deferred]`

---

## Requirement 22 -- SCRM surfaced on the menu bar (core discoverability)

**User story:** As a user, I want the already-built SCRM snapshot/capture family
to be discoverable from the Workbench menu bar, not only by typing, so I can reach
it without memorising the verbs -- while the bare-core POM stays minimal.

### Context (FORWARD-PLAN decision 1, RE-PHRASED under owner-confirmed menu-bar-from-POM model)

SCRM is BUILT and core-wired (Requirements 1-20): the `SNAPSHOT` and
`CAPTURE START/STOP/SCREEN/STATUS/LIST/PURGE/REPLAY` verbs already dispatch
through the single front door and the `[REPLAY]` Context is wired. This
requirement is pure SURFACING: it places those EXISTING verbs on the menu bar.
It builds ON the framework (menu-workspace Requirement 17 menu-bar support,
CR-NR-080; code-only built-in menu content, CR-CH-021) and introduces NO new
verb, NO bespoke intercept, NO second dispatcher, and NO second navigation stack
(framework-conformance). It is the menu-bar analogue of the existing dynamic
`THEME LIST` theme-picker dropdown.

### OWNER-CONFIRMED FRAMEWORK CHANGE -- the menu bar is a RUNTIME ECHO of the POM TOML

This requirement is RE-PHRASED from the earlier "additive SNAPSHOT bar entry, bare
POM stays 0/3/X" wording under two owner-confirmed framework changes (recorded in
`docs/status/change-log.md` CR-NR-114 and per framework-conformance.md as
confirmed framework changes, NOT workarounds):

- **(a) Menu-bar-from-POM.** The menu bar is NOT a separate hardwired menu. It is
  a RUNTIME ECHO of the POM TOML: every option in the POM TOML appears as a
  top-level menu-bar item, built at runtime, and each bar item's DROPDOWN is built
  at runtime from that item's SUBMENU TOML. Nothing on the menu bar is hardwired.
  This REPLACES the current hardwired `DEFAULT_MENU_BAR_TOML` + dynamic-`SNAPSHOT`
  arrangement (Wave S task 14). SCRM is a NEW menu SCREEN with its own submenu
  TOML, which serves as the dropdown for the SCRM bar item.
- **(b) CR-CH-073 bare-POM invariant change.** The bare-core POM GAINS an SCRM
  option. This DELIBERATELY changes the CR-CH-073 bare-POM invariant (previously
  EXACTLY `0` Settings / `3` Help / `X`): the bare POM now CONTAINS SCRM by
  design. Because SCRM is then in the POM, it appears on the bar automatically,
  and the old hardwired Snapshot bar entry is gone, replaced by SCRM-from-POM.

Built-in defaults REMAIN code-only `DEFAULT_*_TOML`, never written to disk
(CR-CH-021). Everything resolves through the SINGLE dispatch seam
(`resolve_target` / `open_menu_by_name`): no second dispatcher, no hardwired bar,
no new navigation stack.

#### Acceptance Criteria
22.1 THE menu bar SHALL be built at RUNTIME as an echo of the POM TOML: for each
     option in the active POM menu, THE system SHALL render a top-level menu-bar
     item; NO menu-bar item SHALL be hardwired. (Owner-confirmed framework change
     (a); REPLACES the hardwired `DEFAULT_MENU_BAR_TOML`.) `[forward-plan]`
22.2 EACH menu-bar item's DROPDOWN SHALL be built at RUNTIME from that item's
     SUBMENU TOML (the submenu menu screen the option opens), so the dropdown
     content is the submenu's options. `[forward-plan]`
22.3 THE bare-core POM (`DEFAULT_POM_TOML`) SHALL CONTAIN an SCRM option
     (label/description "SCRM : Screen Capture and Recording"), in ADDITION to the
     existing `0` Settings / `3` Help / `X` terminate options. This DELIBERATELY
     changes the CR-CH-073 bare-POM invariant (owner-confirmed framework change
     (b)); the bare POM is NO LONGER exactly 0/3/X. `[forward-plan]`
22.4 BECAUSE SCRM is a POM option (Req 22.3) THE SCRM bar item SHALL appear on the
     menu bar automatically via the POM-echo (Req 22.1), and the old hardwired
     Snapshot bar entry SHALL be GONE, replaced by SCRM-from-POM. `[forward-plan]`
22.5 THE SCRM bar item's dropdown SHALL be built from the SCRM submenu TOML (the
     new SCRM menu screen, Requirement 25), whose options are the SCRM verbs
     (Snapshot / Capture start / Capture stop / Capture replay / SCRM Help /
     Exit). `[forward-plan]`
22.6 EACH SCRM submenu option's `command` SHALL be an EXISTING SCRM verb
     (`SNAPSHOT`[/`TEXT`/`ANSI`/`MARKDOWN`/`MD`/`HTML`/`YAML`/`AI`] or
     `CAPTURE START/STOP/SCREEN/STATUS/LIST/PURGE/REPLAY`), so activating it
     routes through `ff_command::resolve_target` and the shared dispatch path
     identically to typing the verb. NO new verb, intercept, dispatcher, or
     navigation stack SHALL be introduced. (framework-conformance; extends
     Requirement 11.3, 11.4.) `[forward-plan]`
22.7 WHEN the user activates any menu-bar item or submenu option THE system SHALL
     dispatch its command through the single front door (`dispatch_command_string`
     -> `resolve_target`, or `open_menu_by_name` for a submenu-open), with the same
     observable effect as the typed command (command parity). (extends
     Requirement 11.4.) `[forward-plan]`
22.8 WHEN the user issues RESET BARE THE system SHALL (a) write/generate the
     default SCRM submenu menu screen (from the code-only `DEFAULT_SCRM_MENU_TOML`,
     never materialising a built-in default to disk, CR-CH-021), and (b) produce a
     bare POM TOML that CONTAINS the SCRM option (Req 22.3). `[forward-plan]`
22.9 Typed access to `SNAPSHOT` / `CAPTURE ...` SHALL continue to work unchanged
     (the menu/bar surfacing is additive to typed access). `[forward-plan]`

---

## Requirement 23 -- Picture snapshot formats (PNG first, then PDF-reuse)

**User story:** As a user, I want a screen SNAPSHOT to place a real PICTURE of the
window AND the box-drawn text on the clipboard so that pasting into Word or
PowerPoint brings in BOTH the screenshot image and the text together, while
pasting into a plain-text editor brings in just the text.

### Context (owner-confirmed CR-NR-114, three-flavor clipboard -- REVISED)

The clipboard SNAPSHOT now carries THREE flavors from one copy:

- PICTURE (PNG) = a REAL SCREENSHOT of the FileForge application WINDOW (its
  actual rendered pixels), per the revised Requirement 1.3 -- NOT a text-painted
  raster. The earlier text-rasterised PNG was pointless because it looked
  identical to the text flavor; the picture must be a true print-screen of the
  window.
- TEXT = the Screen_Model text rendering using proper Unicode box-drawing
  characters (Req 1.1), as a Markdown-fenced block so a plain-text target pastes
  readable, aligned text.
- COMPOSITE RICH (HTML, `CF_HTML`) = an HTML fragment that EMBEDS the window
  screenshot image AND the box-drawn text together, so a rich target (Word /
  PowerPoint) that prefers HTML pastes BOTH the picture and the text in one go.

This is why "paste into Word shows both": Word consumes the HTML composite flavor
(image + text), a plain-text editor consumes the text flavor, and an
image-only target can consume the DIB/picture flavor. A single clipboard paste
always yields ONE flavor chosen by the target -- the composite HTML flavor is how
we get image+text appearing together.

PDF stays a FILE/single-format output (`SNAPSHOT PDF`) reusing `ff-scrm/pdf.rs`.
ODT/ODP deferred.

#### Acceptance Criteria
23.1 THE set of SNAPSHOT picture formats SHALL be PNG (first) and PDF (reusing
     `ff-scrm/pdf.rs`); ODT and ODP SHALL be DEFERRED and recorded as NOT-COVERED,
     not built in this phase. (Owner-confirmed D(formats).) `[forward-plan]`
23.2 WHEN the user issues `SNAPSHOT PNG` (or a clipboard SNAPSHOT) THE system
     SHALL produce a PNG that is a REAL SCREENSHOT of the FileForge application
     WINDOW (its actual rendered pixels), per revised Req 1.3 -- NOT a
     text-rasterised image. The screenshot captures the WINDOW, not the whole
     desktop. (Reverses the former text-raster approach for the picture flavor.)
     `[forward-plan]`
23.3 WHEN the user issues `SNAPSHOT PDF` THE system SHALL produce a single-screen
     PDF by REUSING the existing `ff-scrm/pdf.rs` path, keeping the screen content
     as selectable text (Req 12.6), not a rasterised page image. `[forward-plan]`
23.4 WHEN the user issues a SNAPSHOT that targets the CLIPBOARD, THE system SHALL
     place THREE flavors on the clipboard SIMULTANEOUSLY in one transaction:
     (a) the PICTURE -- a real window screenshot (Req 23.2) in a DIB flavor
     (`CF_DIB`/`CF_DIBV5`) a rich/image target accepts; (b) the TEXT -- the
     Markdown-fenced Unicode-box-drawing rendering as `CF_UNICODETEXT`; and
     (c) the COMPOSITE HTML (`CF_HTML`) -- an HTML fragment embedding the window
     screenshot image AND the box-drawn text together. All three derive from the
     SAME capture. (Owner-confirmed CR-NR-114: three-flavor clipboard.)
     `[forward-plan]`
23.5c WHEN the clipboard content is pasted into a RICH target that prefers HTML
     (Word / PowerPoint), THE target SHALL consume the COMPOSITE HTML flavor and
     render BOTH the window screenshot image AND the box-drawn text together in
     one paste; WHEN pasted into a PLAIN-TEXT target (Notepad / a code editor),
     THE target SHALL receive ONLY the Markdown-fenced text; WHEN pasted into an
     image-only target (e.g. an image editor), THE target SHALL receive the
     screenshot picture. The destination application selects ONE flavor; the
     composite HTML is what makes image+text appear together in Word.
     `[forward-plan]`
23.5d THE multi-flavor offer SHALL use the OS multi-flavor clipboard in ONE
     clipboard-open transaction, publishing `CF_HTML` (the composite, the flavor
     Word prefers), `CF_DIB`/`CF_DIBV5` (the window screenshot), and
     `CF_UNICODETEXT` (the Markdown text). The `CF_HTML` payload SHALL carry the
     correct CF_HTML header (Version/StartHTML/EndHTML/StartFragment/EndFragment
     byte offsets) and embed the screenshot as a base64 `data:` image inside the
     fragment. The implementation SHALL be VERIFIED by a real paste into Word
     (both the picture and the text must appear), not only by a layer-level test.
     `[forward-plan]`
23.5e ANSI output (text with colour escape sequences) SHALL be available ONLY as
     an EXPLICIT `SNAPSHOT ANSI` single-format output (Req 4.3), intended for a
     terminal / ANSI-aware viewer or a file; ANSI SHALL NOT be a clipboard flavor
     (raw escape codes paste as garbage in ordinary editors). Colour fidelity on
     the clipboard is carried by the PNG picture, not by ANSI. `[forward-plan]`
23.4a WHERE a FILE/document output format (as opposed to the clipboard) supports
     carrying BOTH a picture and the ANSI/TEXT rendering, THE SNAPSHOT SHALL embed
     BOTH; WHERE it supports only text, the text SHALL be written. (Owner-confirmed
     D(formats): produce both where the format supports it.) `[forward-plan]`
23.5 THE picture bitmap SHALL be an OPTIONAL secondary artefact (Req 1.3): it
     SHALL NOT replace the selectable-text capture. `[forward-plan]`
23.6 EACH picture-format SNAPSHOT verb (`SNAPSHOT PNG` / `SNAPSHOT PDF`) SHALL
     resolve through `ff_command::resolve_target` to a `Function` target and
     dispatch through the shared path, command parity with the typed verb (Req
     11.3, 11.4); NO bespoke intercept. `[forward-plan]`
23.7 ADDING a picture encoder dependency (e.g. the `image` / `png` crate) SHALL
     be pinned to an exact version (`.kiro/steering` dependency discipline); the
     logical-model-first constraint SHALL NOT be weakened to add it. `[forward-plan]`

---

## Requirement 24 -- Capture STOP Save-As dialog

**User story:** As a user, when I stop a capture I want to name the session and
choose how it is saved, so the captured session becomes a reopenable, exportable
record rather than being lost when the live collection ends.

### Context (owner-confirmed D(replay/save))

Today `CAPTURE STOP` ends the active collection with no dialog. The owner-
confirmed requirement is a Save-As dialog on STOP that names the session and
picks the output type, embedding the bitmap alongside the monospaced text where
the type supports it. The dialog INVOKES the EXISTING `CAPTURE SAVE` / `CAPTURE
EXPORT ...` commands (command parity), not a new persistence path.

#### Acceptance Criteria
24.1 WHEN the user issues `CAPTURE STOP` THE system SHALL present a Save-As dialog
     to NAME the session and CHOOSE the output type. `[forward-plan]`
24.2 THE Save-As output-type choices SHALL be ANSI/TEXT, HTML, Markdown, PDF
     (now), and ODT / ODP (later, deferred); the dialog SHALL offer the IMPLEMENTED
     subset at any time and record ODT/ODP as deferred. `[forward-plan]`
24.3 WHERE the chosen output type supports it, THE saved artefact SHALL embed the
     per-Capture bitmap (Req 9.13, 23) ALONGSIDE the monospaced selectable text,
     never replacing the text (Req 1.3). `[forward-plan]`
24.4 THE Save-As dialog SHALL persist the session as the native zip-compatible
     Collection archive (Req 12.5 / 17.3) so it can be reopened and replayed
     (Req 10.11), and SHALL invoke the EXISTING `CAPTURE SAVE` / `CAPTURE EXPORT`
     commands through the single front door (command parity, Req 11.4); it SHALL
     NOT introduce a new save path. `[forward-plan]`
24.5 WHERE the user CANCELS the Save-As dialog THE capture SHALL still STOP (no
     further automatic captures) but the session SHALL NOT be written to a file;
     the in-memory collection SHALL remain available until the session ends
     (consistent with Req 7.6). `[forward-plan]`
24.6 IF the Save-As dialog is a Workspace Context (rather than a modal overlay)
     THE Context SHALL satisfy the master screen design directive
     (`docs/specs/master-screen-directive/`): the focus latch, and a
     `ScreenProvider` or documented opt-out. `[master-directive]`
24.7 THE Save-As dialog SHALL DEFAULT its save location to a DISCOVERABLE
     captures directory within the ffwb domain (so saved captures are easy to
     find), and THE user SHALL be able to NAVIGATE AWAY to any other directory.
     (Owner directive.) THIS default directory IS the `screen-collections`
     location owned by the portable-installation spec
     (`docs/specs/portable-installation/` Req 1.4, 3.4): once that portability
     reorg lands, the default SHALL resolve through the `ff-paths` authority to
     the `data/screen-collections/` root (preserve-on-upgrade); until then it
     resolves against the current ffwb domain convention. The saved-session
     picker (Req 10.12) SHALL list sessions from this SAME directory.
     `[forward-plan]`

---

## Requirement 25 -- SCRM menu screen (keys, labels, title)

**User story:** As a user, I want a dedicated SCRM menu screen whose options are
the SCRM actions, so the SCRM bar dropdown and the SCRM POM option both open a
consistent, named menu.

### Context (owner-confirmed D(SCRM menu keys/labels), STILL TUNABLE)

The SCRM submenu is a new code-only menu screen (`DEFAULT_SCRM_MENU_TOML`, never
written to disk, CR-CH-021) that serves as the dropdown for the SCRM bar item
(Req 22.5) and the target the SCRM POM option opens. The keys/labels below are
the owner's refined draft and are RECORDED AS SETTLED-BUT-TUNABLE (the owner may
adjust them at implementation).

#### Acceptance Criteria
25.1 THE SCRM menu screen SHALL have the title "FileForge Workbench -- Screen
     Capture and Recording". `[forward-plan]`
25.2 THE SCRM POM option (Req 22.3) SHALL have the label/description
     "SCRM : Screen Capture and Recording". `[forward-plan]`
25.3 THE SCRM menu screen SHALL list these options with these mnemonic keys
     (owner-confirmed, tunable): Snapshot = `S`; Capture start = `CS`; Capture
     stop = `CT`; Capture replay = `CR`; SCRM Help = `H`; Exit = `X`.
     `[forward-plan]`
25.4 EACH SCRM menu-screen option's `command` SHALL be an EXISTING SCRM verb
     resolved through `resolve_target` / `open_menu_by_name` (Req 22.6): Snapshot
     -> `SNAPSHOT` (placing the composite clipboard payload, Req 4.1); Capture start ->
     `CAPTURE START`; Capture stop -> `CAPTURE STOP` (invoking the Save-As dialog,
     Req 24); Capture replay -> `CAPTURE REPLAY` (opening the saved-session picker,
     Req 10.12); SCRM Help -> the SCRM help topic; Exit -> the menu END/exit verb.
     NO new verb SHALL be introduced. `[forward-plan]`
25.5 THE SCRM menu screen SHALL be generated by RESET BARE (Req 22.8) from the
     code-only default; it SHALL NOT be materialised to disk as a built-in
     (CR-CH-021). A user `menus/<scrm>.toml` MAY override it (the standard
     user-override-of-default rule). `[forward-plan]`

---

## Requirement 26 -- Mnemonic-letter menu-bar accelerators

**User story:** As a keyboard user, I want each menu-bar item reachable by a
mnemonic-letter accelerator (Alt+<letter>), so I can open a bar menu without the
mouse, without overloading the POM numeric option keys.

### Context (owner-confirmed D(hotkeys))

Accelerators are MNEMONIC LETTERS (Alt+S Settings, Alt+H Help, Alt+C SCRM, etc.),
resolved through the EXISTING key map framework (menu-and-statusbar Requirement
17, function-keys-and-history, menu-workspace Requirement 17), NOT a new
accelerator mechanism. Mnemonics must avoid collisions with existing Alt-bindings
in the active key profile.

#### Acceptance Criteria
26.1 EACH menu-bar item SHALL have a MNEMONIC-LETTER accelerator (Alt+<letter>)
     that opens that bar item's menu, reusing the option's existing `key` where it
     is a letter and assigning a distinct mnemonic where the `key` is a digit.
     (Owner-confirmed D(hotkeys).) `[forward-plan]`
26.2 THE accelerators SHALL be resolved through the EXISTING key map framework
     (menu-and-statusbar Req 17, function-keys-and-history, menu-workspace Req 17)
     as modified-key bindings; NO new accelerator mechanism SHALL be introduced.
     `[forward-plan]`
26.3 THE chosen mnemonics SHALL NOT collide with existing Alt-bindings in the
     active key profile; a collision SHALL be resolved by choosing a distinct
     mnemonic, not by overriding an existing binding. `[forward-plan]`
26.4 THE accelerators SHALL NOT overload the POM numeric option keys (digits
     already select POM options); mnemonic LETTERS are used precisely to avoid
     that overload. `[forward-plan]`
26.5 WHEN an accelerator fires THE system SHALL open the bar item's menu through
     the SAME dispatch seam as a click (Req 22.7); NO bespoke path. `[forward-plan]`

---

## Requirement 27 -- Message boxes and dialog boxes are capturable (copy/paste-able)

**User story:** As a user, I want to copy the text of ANY message box or dialog
box -- not just Workspace Contexts -- so I never hit the Windows frustration of an
error or confirmation message whose text cannot be selected or copied.

### Context (owner-confirmed CR-NR-114)

A message box or modal dialog is a TRANSIENT OVERLAY on top of the active Context,
NOT a Workspace Context. The every-Context snapshot/capture obligation
(Requirement 2, master screen directive Requirement 4) therefore does not
automatically reach it; this requirement extends the snapshot/capture contract to
these transient surfaces. Owner intent: EVERYTHING must be copy/paste-able. The
ONLY permitted exception is a CREDENTIAL SECRET (password, token, API key, secret
key) field, which SHALL be redacted from the capture -- never emitted in clear.

Both kinds are in scope:
- (a) Message / notification / error / confirmation popups (mostly a text
  message plus buttons).
- (b) Modal dialogs with structure (fields, lists, buttons) -- e.g. the dataset
  allocator, catalog manager, the SCRM Save-As dialog (Req 24).

#### Acceptance Criteria
27.1 EVERY message box and dialog box SHALL be capturable: its content SHALL be
     available as a logical screen model (title, message/body text, field
     labels+values, list rows, button labels) through the same `ScreenProvider`
     seam the master directive mandates (master-screen-directive Req 4;
     screen-snapshot-scrm Req 2). NO dialog or message box SHALL present text that
     cannot be copied. `[master-directive]`
27.2 WHEN a message box or dialog box is showing AND the user issues SNAPSHOT, THE
     system SHALL capture the FRONTMOST surface -- the dialog/message box itself
     (its logical model), not the Context hidden behind it -- and place it on the
     clipboard per the multi-flavor model (Req 23.4, 23.5c): both the PNG picture
     and the box-drawn ANSI/TEXT. `[forward-plan]`
27.3 IN ADDITION to SNAPSHOT, EVERY message box and dialog box SHALL offer a
     direct COPY affordance (e.g. a copy action / right-click copy / a copy
     button) that places its text on the clipboard, so copying a message does NOT
     require invoking the full SNAPSHOT machinery. This directly addresses the
     uncopyable-message-box frustration. `[forward-plan]`
27.4 THE ONLY content a message box or dialog box MAY withhold from a capture or
     copy SHALL be a CREDENTIAL SECRET (password / token / API key / secret-key
     field): such a field SHALL be REDACTED (e.g. replaced with a masked
     placeholder) in the captured model and on the clipboard, never emitted in
     clear. All other content SHALL be copy/paste-able. `[forward-plan]`
27.5 A dialog's secret-field redaction SHALL be a DELIBERATE, declared property of
     that field (the dialog marks which fields are secret), analogous to the
     documented capture opt-out (master-screen-directive Req 4.3); a field SHALL
     NOT be silently uncopyable for any other reason. `[forward-plan]`
27.6 WHERE a dialog box is itself a Workspace Context (rather than a transient
     overlay), it SHALL ALSO satisfy the master screen directive in full
     (Req 24.6); this requirement additionally binds the transient (non-Context)
     message boxes and modal overlays that the master directive's every-Context
     wording does not reach. `[master-directive]`
27.7 THE capture of a dialog/message box SHALL render from its logical model, NOT
     from an egui pixel grab (consistent with Req 1, Req 2.2). `[forward-plan]`

---

## Out of scope (v1) / future enhancements

Recorded from the SCRM source "Future Enhancements"; NOT gated here, listed so the
architecture does not preclude them: screen differencing, user activity heat
maps, process mining, AI-generated walkthroughs/training manuals, voice-narrated
replay, automated test evidence generation, PowerPoint export, Git-managed
collections. `[SCRM section 12]`

NOTE: "session recording" is no longer purely future -- its core (command +
screen timeline) is gated by Req 9.10-9.12, 10.9-10.10, and 17.2a. Exporting that
recording as a re-runnable MACRO remains deferred (Req 21).

DEFERRED picture/document formats: ODT and ODP snapshot/export (Req 23.1, 24.2,
10.13) are DEFERRED by owner decision D(formats); PNG (first) and PDF-reuse
(second) are in scope. ODT/ODP are recorded as NOT-COVERED, to be revisited after
PNG + PDF land.

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
