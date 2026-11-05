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
requirements. See Requirement 23 for the clipboard-flavor acceptance criteria,
and Requirement 28 (CR-CH-079) for the rule that EACH text flavor carries BOTH
the box-drawn layout AND the property/value tables.

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
9.14 REVISED (CR-NR-114, aligning with revised Req 1.3 / 23.2): THE per-transition
     PICTURE of Req 9.13 SHALL be a REAL WINDOW SCREENSHOT (the application
     window's rendered pixels), the SAME picture flavor the SNAPSHOT path produces
     (Req 23.2), NOT a rasterisation of the logical TEXT render. The ANSI/TEXT and
     Markdown renderings of the Capture remain Screen_Model-derived (Req 1, 2.2).
     This supersedes the earlier "rasterise the text render" wording so the
     recorder's picture matches the snapshot's picture. `[forward-plan]`

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
     (command `SCRM`, description "Screen Capture and Recording"; the description
     was shortened from "SCRM : ..." by CR-NR-117 D7a), in ADDITION to the
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
22.8 (REVISED by CR-CH-116 -- write-through model supersedes the CR-CH-021 clause.)
     WHEN the user issues RESET BARE THE system SHALL, after archiving, REBUILD on
     disk from the Recovery_Source `DEFAULT_SCRM_MENU_TOML` (via the shared
     Config_Rebuild_Helper, configuration-system Requirement 19.17): (a) the SCRM
     submenu menu screen `menus/scrm.toml` as a Complete_Config_File, and (b) the
     bare POM TOML `menus/pom.toml` that CONTAINS the SCRM option (Req 22.3). Under
     CR-CH-116 these files are also written on FIRST RUN (configuration-system
     Requirement 23.2), so the application runs from them; the compiled
     `DEFAULT_SCRM_MENU_TOML` is the Recovery_Source only. (The CR-CH-021 clause
     "never materialising a built-in default to disk" is RETIRED by CR-CH-116.)
     `[forward-plan]`
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
23.2a WHEN the captured FRONTMOST surface is a DIALOG / message box (Req 27.2),
     THE picture flavor SHALL be CROPPED to the dialog's on-screen RECTANGLE --
     just the dialog, NOT the whole window behind it. The system captures the full
     framebuffer egui screenshot and SHALL crop it to the dialog rect BEFORE
     encoding the PNG and BEFORE embedding the image in the CF_HTML composite. THE
     dialog rect SHALL be captured at RENDER time from the actual rendered bounds
     (the modal/Area response rect in points) and converted to physical pixels
     using the frame's pixels-per-point. WHERE egui's screenshot API cannot crop
     to a sub-rect THE system SHALL crop the RGBA buffer MANUALLY (copying the
     sub-rectangle into a new raster) and SHALL NOT silently fall back to the whole
     window. A full-window SNAPSHOT with NO dialog open SHALL remain UNCHANGED
     (the whole window is captured); the crop applies ONLY when a dialog is the
     frontmost captured surface. THE TEXT and CF_HTML text SHALL remain the
     dialog's logical model -- only the PICTURE (PNG + the image embedded in the
     CF_HTML) is cropped. (Owner finding CR-NR-114.) `[forward-plan]`
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
23.5f THE box-drawn TEXT inside the `CF_HTML` composite SHALL be rendered so that
     a rich target (Word / PowerPoint) shows the box-drawing in a FIXED-WIDTH
     font with its column alignment preserved, instead of adopting the target's
     default proportional (Normal) style. The fixed-width font SHALL be applied to
     the actual TEXT RUNS, not only to a block element: the box text SHALL be
     wrapped -- inside a `<pre>` that supplies `white-space:pre` for line breaks
     and alignment -- in BOTH a legacy `<font face="Consolas,'Courier New',monospace">`
     element AND a `<span style="font-family:'Consolas','Courier New',monospace">`,
     so the monospace family lands on the characters regardless of the paragraph
     style the target assigns to the `<pre>`. (Word remaps a bare `<pre>` to its
     "HTML Preformatted" style and OVERRIDES an inline font set on the `<pre>`
     itself, so a font on the `<pre>` alone was dropped on paste; a font on the
     text runs is honoured.) THE CF_HTML byte offsets
     (StartHTML/EndHTML/StartFragment/EndFragment) SHALL be recomputed correctly
     after the markup change, and the `<img>` (window screenshot) SHALL remain in
     the fragment alongside the `<pre>`. The decisive confirmation is a REAL paste
     into Word (the text renders monospaced and aligned). (Owner finding
     CR-NR-114: pasted text was adopting Word's normal non-monospace style; the
     first `<pre>`-only font fix did not survive Word paste.) `[forward-plan]`
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
25.2 THE SCRM POM option (Req 22.3) SHALL have the command `SCRM` and the
     description "Screen Capture and Recording" (shortened from "SCRM : ..." by
     CR-NR-117 D7a). `[forward-plan]`
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
25.5 (REVISED by CR-CH-116 -- supersedes the CR-CH-021 "never materialised"
     clause.) THE SCRM menu screen SHALL be WRITTEN to `menus/scrm.toml` on
     first run and REBUILT by RESET BARE from the Recovery_Source
     `DEFAULT_SCRM_MENU_TOML` (via the shared Config_Rebuild_Helper, Req 22.8);
     the application runs from the on-disk file. A user `menus/scrm.toml`
     remains authoritative (the standard write-through user-override-of-default
     precedence); the compiled default is the Recovery_Source only.
     `[forward-plan]`

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

## Requirement 28 -- Each clipboard text flavor carries BOTH the box-drawn layout AND the property/value tables (CR-CH-079)

**User story:** As a user who pastes a screen SNAPSHOT into Word AND into Notepad,
I want EACH text flavor to carry BOTH the box-drawn screen layout AND the
property/value pairs as tables, so neither paste target loses half the
representation.

### Context (owner-confirmed 2026-11-02, CR-CH-079)

Requirement 23 established the three-flavor clipboard (PICTURE / TEXT / COMPOSITE
RICH HTML). As SHIPPED, each TEXT-bearing flavor carries only ONE of the two text
representations a table-bearing screen can offer (owner-observed on the Config
snapshot):

- The COMPOSITE RICH (`CF_HTML`) flavor, consumed by Word, embeds the PNG picture
  AND the box-drawn layout (the `<pre>`/per-line monospace `<span>` body built
  from `body_lines`), but NOT the property/value tables.
- The PLAIN-TEXT (`CF_UNICODETEXT`) flavor, consumed by Notepad, is the Markdown
  rendering. For a Context that supplies `markdown_sections` (the Config panel),
  the Markdown renderer early-returns the per-group Property/Value TABLES ONLY and
  omits the box-drawn layout.

So Word gets image + box but no tables; Notepad gets tables but no box. This
requirement makes both flavors SYMMETRIC: each text flavor carries BOTH
representations. The PNG picture flavor (Req 23.2) is UNCHANGED.

This requirement BUILDS ON the existing SCRM clipboard flavors (Req 23) and the
`ff-screen-model` renderers; it introduces NO new clipboard flavor and NO new
framework mechanism. It modifies WHAT each existing flavor's text contains.

#### Acceptance Criteria
28.1 WHEN a SNAPSHOT targets the clipboard AND the active screen model carries
     BOTH a box-drawn layout (`body_lines`, or the generic framed body) AND
     tabular property/value content (`markdown_sections` and/or `tables`/`fields`),
     THE PLAIN-TEXT (`CF_UNICODETEXT`) flavor SHALL contain BOTH the box-drawn
     layout AND the property/value pairs rendered as Markdown-style tables, in a
     single plain-text block. `[CR-CH-079]`
28.2 WHEN a SNAPSHOT targets the clipboard AND the active screen model carries
     BOTH representations (as in 28.1), THE COMPOSITE RICH (`CF_HTML`) flavor
     SHALL contain the PNG window image AND the box-drawn layout AND the
     property/value pairs rendered as HTML tables (`<table>`), the HTML tables
     being the structural equivalent of the plain-text flavor's Markdown tables.
     `[CR-CH-079]`
28.3 THE PNG picture flavor (the `CF_DIB`/`CF_DIBV5` window screenshot, Req 23.2,
     23.4) SHALL be UNCHANGED by this requirement: nothing about the image
     content, cropping (Req 23.2a), or its placement changes. `[CR-CH-079]`
28.4 WITHIN each text flavor, the ORDER SHALL be the box-drawn layout FIRST, then
     the property/value tables. In the `CF_HTML` flavor the PNG image paragraph
     comes first (as today, Req 23.5f), then the box-drawn `<pre>`/monospace body,
     then the HTML tables. This mirrors the visual reading order (the framed
     screen, then its data broken out as tables) and preserves the existing
     image-then-box ordering of the composite. `[CR-CH-079]`
28.5 THE box-drawn layout inside the `CF_HTML` flavor SHALL retain the monospace
     per-line `<span>` treatment that makes it render fixed-width and aligned in
     Word (Req 23.5f); adding the HTML tables SHALL NOT regress that treatment,
     and the CF_HTML header byte offsets (StartHTML/EndHTML/StartFragment/
     EndFragment) SHALL remain correct after the markup grows. `[CR-CH-079]`
28.6 THE property/value tables in BOTH flavors SHALL be rendered from the SAME
     logical content the Markdown renderer already derives for the Context (for
     the Config panel, the per-group `markdown_sections`: a heading-less intro
     table plus one `[Group] (state)` heading + 2-column `Property | Value` table
     per namespace group), so the two flavors show the SAME data and group
     structure, differing only in table syntax (Markdown vs HTML). `[CR-CH-079]`
28.7 THIS rule SHALL be GENERAL to all table-bearing screen snapshots, NOT
     Config-specific: any Context whose screen model carries both a box-drawn
     layout and tabular content (via `markdown_sections`, `tables`, or `fields`)
     SHALL have both flavors carry both representations, implemented in the shared
     `ff-screen-model` renderers / clipboard composite path so every Context
     inherits it. A Context that supplies ONLY a box layout (no tables) SHALL be
     unaffected (both flavors carry just the box); a Context that supplies ONLY
     tables SHALL likewise be unaffected. `[CR-CH-079]`
28.8 THE change SHALL preserve existing single-format behaviour: `SNAPSHOT
     MARKDOWN`, `SNAPSHOT HTML`, `SNAPSHOT TEXT`, and the file/PDF export paths
     (Req 4.2-4.6, 4.8, 12, 23.3) retain their current output. Where a shared
     renderer is changed to satisfy 28.1/28.2, any single-format consumer that
     relied on the previous (one-representation) output SHALL be reconciled so its
     documented behaviour still holds. `[CR-CH-079]`

---

## Requirement 29 -- Full-window snapshot FIDELITY: a faithful top-to-bottom box-drawn mirror of EVERY framed screen, on both the snapshot AND capture paths (CR-CH-115)

**User story:** As a user who copies ANY screen SNAPSHOT (or records a CAPTURE),
I want the box-drawn text rendering to be a faithful mirror of the WHOLE framed
window -- the window title bar, the active workspace tab tag, the workspace
title, the `Command ===>` line near the top, the full-width data grid with
column separators and the always-present empty scratch row, the information
bar, and the function-key bar when it is visible -- so the capture reads like the
real screen instead of a thin, collapsed box that drops most of the chrome.

### Context (owner-confirmed 2026-11-03, CR-CH-115)

The owner pasted the live Alias Editor SNAPSHOT and the box collapsed to a
title, a `SS . . . : SNAPSHOT` dot-leader field line, and an un-separated column
list -- missing the window title bar, the `[ALIASES]` tab tag, the top
`Command ===>` line, the `|`-separated grid, the scratch row, the info bar, and
the F-key bar. This requirement makes the box-drawn text rendering a faithful
full-window mirror for EVERY Context, GENERAL (not Alias-specific), and extends
the SAME fidelity to the CAPTURE path (recording), which shares the snapshot's
`ScreenProvider` seam and `ff-screen-model` renderer.

This requirement BUILDS ON the existing SCRM `ScreenProvider` seam (Req 2), the
`ff-screen-model` `ScreenModel` + text renderer (Req 3, 5), and the per-Context
`*_screen_model()` builders. It extends Req 3.8-3.13 (which already defined the
full-screen model for the POM/menu Context) to EVERY table-bearing and
field-bearing Context. It introduces NO new clipboard flavor, NO new framework
mechanism, and NO reshape of a public `ff-screen-model` / `ff-scrm` type (the
`ScreenModel` struct already carries `header_lines`, `body_lines`,
`footer_lines`, `tables`, `command_line`, and `status_bar`; see design.md).

The text-field / combo cell CONVENTIONS below are owner-confirmed canonical and
are the SAME in the plain-text and HTML flavors (the owner's original `[---- ]v`
mock was a sketch; `[value v]` is the agreed convention -- see design.md for the
best-practice rationale). Unicode box-drawing in the RENDERED snapshot/fixture
output is deliberate runtime product content, exempt from the ASCII-only
documentation rule (consistent with Req 1.1, 5.1).

### Top-chrome acceptance criteria (always present)

29.1 WHEN any capturable Context's screen model is rendered to box-drawn text
     THE rendering SHALL begin with a WINDOW TITLE BAR line inside the frame, as
     the FIRST in-frame content line (above the workspace title row and the body):
     the application name followed by the window-control glyphs, in plain ASCII,
     e.g. `# | FileForge Workbench                      _ [] X`. The application
     name is the live window title; the `_ [] X` are the minimise / maximise /
     close control glyphs as plain ASCII. THE literal leading `# |` marker, the
     exact padding, and the `_ [] X` glyph set are SETTLED-BUT-TUNABLE at
     implementation (they follow the owner's 2026-11-03 mock; confirmable, not
     locked), as Req 25.3 treats its menu labels. `[CR-CH-115]`
29.2 WHEN any capturable Context's screen model is rendered THE rendering SHALL
     place, as the SECOND in-frame content line (directly under the window title
     bar and still above the workspace title row and the body), the TAB-TAG line
     for the ACTIVE workspace -- the Context's `[XXX]` tab tag (e.g. `[ALIASES]`,
     `[CONFIG]`, `[THEME]`, `[POM]`), the terse bracketed Tab_Header tag distinct
     from the descriptive display title (CR-NR-090 / CR-CH-045). The correct tag
     per Context SHALL be emitted (see design.md for the tag source). `[CR-CH-115]`
     NOTE on renderer order: because the current `render/text.rs::render_plain`
     emits `model.title` (the border) first and `header_lines` BELOW the body,
     the title bar + tab tag SHALL be delivered as the FIRST TWO `body_lines`
     (which render directly under the title separator), NOT as `header_lines`;
     the design (Section 12) specifies this so "begin with the window title bar"
     holds against the real renderer. `[CR-CH-115]`
29.3 WHEN any capturable Context's screen model is rendered THE rendering SHALL
     include the workspace TITLE line (the descriptive display title, e.g.
     "Alias Editor"). THE title MAY be carried in the frame border (as today) OR
     as a chrome line, provided it appears exactly ONCE and is not duplicated.
     `[CR-CH-115]`
29.4 WHEN any capturable Context's screen model is rendered THE `Command ===>`
     line SHALL be positioned near the TOP, under the title (NOT at the bottom
     and NOT outside the frame). This CORRECTS the current Alias rendering, where
     the command line is emitted outside/below the frame. THE command line SHALL
     carry its current typed VALUE followed by an underscore input area (e.g.
     `Command ===> ss___...`), consistent with the POM/menu rendering (Req 3.11).
     THE underscore input-area width SHALL match the existing POM/menu
     command-line field width (the same run length the menu builder already
     emits), so the command line is consistent across every Context rather than
     a per-Context length. `[CR-CH-115]`

### Body grid acceptance criteria (faithful mirror of the live panel)

29.5 WHEN a Context presents tabular data THE box-drawn body SHALL render a
     COLUMN HEADER row and data rows using the `|` column separator between
     cells (NOT space-only joins), so the grid reads as a real ISPF table. THE
     ASCII fallback (Req 5.2) SHALL use `|` as today. `[CR-CH-115]`
29.6 THE grid cells SHALL be FIXED-WIDTH, padded to each column's width (each
     column width = max(header width, widest cell width in that column,
     including the scratch row)); WHERE a combo `[value v]` or a filled value
     exceeds the header width THE column SHALL WIDEN to that value and EVERY row
     -- header, data rows, and the scratch row -- SHALL pad to the widened width
     so the grid stays aligned (no truncation). `[CR-CH-115]`
29.6a THE box body SHALL FILL THE FULL FRAME WIDTH rather than collapse to the
     widest single content line: the frame inner width SHALL be derived from the
     screen model's column count (`model.dimensions.columns`, the conventional
     24x80 default, or the live panel column count where a Context threads it
     onto the model), and the grid SHALL be padded to that frame width. This is
     DISTINCT from the per-column max of 29.6: 29.6 sizes each column to its
     content; 29.6a pads the whole framed body out to the frame width so a narrow
     table still fills the window. `[CR-CH-115]`
29.7 THE box-drawn body SHALL include the always-present empty SCRATCH row at the
     bottom of the data rows (the insertable row the live table editor shows), so
     the capture mirrors the live panel's insert affordance. THE scratch row
     SHALL be a PROVIDER-OWNED row carried in `model.tables[*].rows` (a full row
     of empty cells, each rendered per 29.8/29.9), NOT a row appended only inside
     the plain-text renderer, so that the SAME scratch row appears in BOTH the
     plain-text grid AND the HTML `<table>` flavor (29.7a). `[CR-CH-115]`
29.7a THE scratch row SHALL appear IDENTICALLY in the plain-text and HTML text
     flavors: in the HTML flavor it is a trailing `<tr>` of empty `<td>` cells
     carrying the same `[ v]` / `____` cell strings as the plain-text row,
     because the cell text is provider-owned (see 29.8/29.9) and both the
     plain-text grid renderer and the HTML `<td>` emitter consume the same
     `model.tables` rows. `[CR-CH-115]`
29.8 A DROPDOWN / COMBO cell SHALL render as `[value v]` -- the selected value in
     square brackets with a trailing space and a `v` marker -- padded to the
     column width; an EMPTY combo SHALL render as `[ v]` padded to the column
     width. This convention SHALL be IDENTICAL in the plain-text and HTML text
     flavors: the cell string `[value v]` / `[ v]` is produced by the PROVIDER
     when it builds the row, so the plain-text grid and the HTML `<td>` emit the
     SAME verbatim string. WHERE a `[value v]` is wider than its column header
     THE column SHALL widen to it per 29.6 (no truncation). `[CR-CH-115]`
29.9 AN EDITABLE TEXT cell that is EMPTY SHALL render as a run of underscores
     `____` to the field width; an editable text cell that is FILLED SHALL render
     its value padded to the field width. This convention SHALL be IDENTICAL in
     the plain-text and HTML text flavors (same provider-owned cell string in the
     plain-text grid and the HTML `<td>`). WHERE a filled value is wider than its
     column header THE column SHALL widen to it per 29.6 (no truncation).
     `[CR-CH-115]`

### Bottom-chrome acceptance criteria

29.10 WHEN any capturable Context's screen model is rendered THE rendering SHALL
     include the INFORMATION / STATUS bar at the BOTTOM, carrying the live status
     text the screen shows (`model.status_bar.text`). THE exact live status
     SOURCE the info bar mirrors (the shell's current status line) is
     SETTLED-BUT-TUNABLE at implementation -- confirmable, not locked -- as Req
     25.3 treats its labels. WHERE the status text is EMPTY the bottom info bar
     SHALL still be a defined (possibly blank) bottom line, NOT dropped (distinct
     from the F-key bar of 29.11, which IS omitted when hidden). `[CR-CH-115]`
29.11 WHERE the FUNCTION-KEY (F-key) bar is currently VISIBLE (the live
     `key_bar_visible` state is on) THE rendering SHALL include the F-key display
     line(s) at the bottom; WHERE it is HIDDEN THE rendering SHALL OMIT the F-key
     bar entirely, so the capture reflects the live F-key-bar visibility state.
     (Mirrors Req 3.10's pfshow rule, generalised to every Context.) `[CR-CH-115]`

### General-scope and shared-renderer acceptance criteria

29.12 THE full-window fidelity of 29.1-29.11 SHALL be GENERAL to ALL capturable
     Contexts -- not Alias-specific. It SHALL be implemented in the SHARED place
     (the `ff-screen-model` text renderer plus a shared shell helper that builds
     the common chrome -- title bar, tab tag, title, command line, info bar,
     F-key bar -- for every per-Context `*_screen_model()` builder), so every
     Context inherits the faithful box without per-Context bespoke chrome code.
     `[CR-CH-115]`
29.13 THE chrome lines (title bar, tab tag, title, command line, info bar, F-key
     bar) and the grid SHALL be built PURELY from the live LOGICAL state (the
     window title, the active Kind's tab tag, `command_text`, the panel's rows,
     the status text, `key_bar_visible` / `key_bar_scope`), NEVER by reading egui
     widgets (consistent with Req 2.2, 5.4). `[CR-CH-115]`
29.14 THE SAME full-window fidelity SHALL apply to the CAPTURE path (recording),
     not only to the single point-in-time SNAPSHOT. BECAUSE capture and snapshot
     share the one `ScreenProvider` seam and the one `ff-screen-model` renderer
     (see design.md), the fidelity SHALL be delivered in that shared path so a
     CAPTURE SCREEN / auto-capture records the IDENTICAL faithful box a SNAPSHOT
     produces for the same screen. No second, divergent capture renderer SHALL be
     introduced. `[CR-CH-115]`
29.15 THE existing POM / menu rendering (Req 3.8-3.13) SHALL be reconciled with
     the generalised chrome so the POM capture GAINS the window title bar and the
     `[POM]` tab tag (which the 2026-11-01 Bug-2 correction had removed), while
     keeping its two-column options+calendar body and its top `Command ===>`
     line. The owner's 2026-11-03 direction RE-ADDS the title bar + tab tag to
     ALL snapshots, superseding the Bug-2 "no header block" decision FOR THE
     TITLE BAR + TAB TAG specifically. `[CR-CH-115]`
29.16 THE picture (PNG) flavor (Req 23.2) and the property/value table flavors
     (Req 28) SHALL be UNCHANGED by this requirement: it governs the BOX-DRAWN
     TEXT layout and its chrome only. A Context with no tabular content SHALL
     still gain the top/bottom chrome (title bar, tab tag, title, command line,
     info bar, F-key-when-visible) around whatever body it renders. `[CR-CH-115]`

### Feeds the workspace-architecture alignment (recorded intent)

29.17 THE full-window fidelity model (the common chrome + grid conventions) SHALL
     be recorded as a first-class INPUT to the upcoming workspace-architecture
     alignment: a Context that renders a faithful full-window box is, by
     construction, one whose logical screen model is complete, which is the same
     completeness the workspace-architecture model needs. (Owner direction;
     design.md Section 12 records this.) `[CR-CH-115]`

---

## Requirement 30 -- Snapshot/capture reproduction of the COMMON WORKSPACE FRAME (CR-NR-115)

**User story:** As a user who copies ANY screen SNAPSHOT (or records a CAPTURE), I
want the box-drawn rendering to reproduce the SHELL's common workspace frame --
the window title bar, the live menu bar, the FULL tab bar (all open tabs with
their tags and close affordance), the centered workspace title, the top
`Command ===>` line, the content area the workspace owns, the message line when a
message is present, the status bar (common segments + the active Kind's segment),
and the function-key bar when `pfshow` is on -- so the snapshot mirrors the whole
live frame, not a thin box.

### Context (owner-confirmed 2026-11-03, CR-NR-115)

This requirement is the SNAPSHOT-REPRODUCTION half of the common-workspace-frame
gate. The common frame itself -- the eight shell-owned elements, the message line
behaviour, the status-bar common+per-Kind model, the `pfshow`-driven key bar, and
the normative common-vs-unique boundary -- is defined in `workspace-framework`
Requirement 7 (the backbone). THIS requirement says how the snapshot/capture
reproduces that frame, EXTENDING and in two places REVISING Requirement 29
(CR-CH-115):

- REVISES 29.2: the snapshot SHALL reproduce the FULL tab bar (ALL open tabs with
  their `[XXX]` tags and the close affordance), NOT only the active Context's tag.
  This supersedes 29.2's active-tag-only rule (owner decision D3).
- EXTENDS 29.1-29.16 by ADDING the live MENU BAR line (owner decision D4) and the
  MESSAGE LINE (when a message is present) to the reproduced frame.
- CONFIRMS the box-drawing grid conventions of 29.5-29.9 as the canonical
  content-table rendering (owner decision: use box-drawing cell borders /
  vertical separators for content tables).

All reproduction is from the live LOGICAL state via the shared `ScreenProvider`
seam and `ff-screen-model` renderer (no egui read), consistent with Req 2.2, 5.4,
29.13. Unicode box-drawing in the RENDERED output is deliberate runtime product
content, exempt from the ASCII-only documentation rule (Req 1.1, 5.1).

#### Acceptance Criteria -- the eight common-frame elements in the snapshot

30.1 WHEN any capturable Context's screen model is rendered to box-drawn text THE
     rendering SHALL reproduce the common workspace frame in this top-to-bottom
     order: Window_Title_Bar, Menu_Bar, Tab_Bar, Workspace_Title, Primary_Command_
     Field, [Content_Area], Message_Line (when present), Status_Bar, Function_Key_
     Bar (when `pfshow` on). This is the `workspace-framework` Requirement 7
     Common_Frame reproduced faithfully. `[CR-NR-115]`
30.2 THE Window_Title_Bar line SHALL carry the brand token `FileForge Workbench`
     (untranslated) followed by the ASCII control glyphs `_ [] X`, as the first
     in-frame content line (consistent with Req 29.1). `[CR-NR-115]`
30.3 THE rendering SHALL ADD the live MENU_BAR line (e.g. `Settings | Help |
     SCRM`) directly under the Window_Title_Bar (owner decision D4 -- previously
     omitted). The menu text comes from the live menu echo (CR-NR-114), not a
     hardcoded copy. `[CR-NR-115]`
30.4 THE rendering SHALL reproduce the FULL TAB_BAR: ALL open Workspace tabs with
     their `[XXX]` Tab_Header tags (e.g. `[ALIASES]  [CONFIG]`) and each tab's
     close affordance, NOT only the active Context's tag. This REVISES Req 29.2
     (active-tag-only) per owner decision D3. `[CR-NR-115]`
30.5 THE rendering SHALL reproduce the Workspace_Title (the descriptive display
     title) CENTERED, exactly once (consistent with Req 29.3). `[CR-NR-115]`
30.6 THE rendering SHALL reproduce the Primary_Command_Field as a TOP line
     `Command ===> <value>____...` (consistent with Req 29.4). `[CR-NR-115]`
30.7 THE Content_Area between the Primary_Command_Field and the Message_Line SHALL
     be reproduced as the active Workspace's own body (its grid / fields /
     messages); the shared frame reproduction SHALL NOT invent content there
     (consistent with Req 29.16: a Context with no tabular content still gains the
     surrounding chrome around whatever body it renders). `[CR-NR-115]`

#### Acceptance Criteria -- the message line in the snapshot

30.8 WHERE a Message_Line message is present AT THE MOMENT a snapshot/capture is
     taken THE rendering SHALL include the message line(s) directly above the
     Status_Bar, so a transient message that is on screen is captured. `[CR-NR-115]`
30.9 WHERE no message is present THE message line SHALL be OMITTED from the
     rendering (it is not a permanently reserved blank line, distinct from the
     Status_Bar info line of Req 29.10). `[CR-NR-115]`

#### Acceptance Criteria -- the status bar in the snapshot

30.10 THE reproduced Status_Bar SHALL carry ONLY the Common_Status_Segments --
      `Last: HH:MM` (time of the last action or screen transition, initialised
      to the startup time, never `--:--`), `Prof: <ActiveProfileName>` and
      `Zoom: <signed offset>` (always shown, e.g. `Zoom: 0`, `Zoom: +14`,
      `Zoom: -3`, formatted by the SAME formatter as the live bar; revised
      2026-11-04, owner-requested) -- PLUS
      the active Kind's Workspace_Status_Segment cells where the Kind
      contributes any. `Ln <n>, Col <n>`, encoding and line count are NOT
      common: they are workspace-owned (the editor Context contributes them as
      its Workspace_Status_Segment; a settings-type Workspace such as Config or
      Alias shows neither until its own per-Kind gate). The `RUNNING` segment
      SHALL NOT appear (superseded by the Message_Line). This REVISES Req
      29.10's "mirror the live status text" to the common+per-Kind model of
      `workspace-framework` Requirement 7.16-7.21 (revised 2026-11-04).
      `[CR-NR-115]`

#### Acceptance Criteria -- the function-key bar in the snapshot

30.11 WHERE `pfshow` is ON THE rendering SHALL include the Function_Key_Bar
      line(s) at the bottom; WHERE `pfshow` is OFF THE rendering SHALL omit it
      (consistent with Req 29.11 / 3.10). `[CR-NR-115]`

#### Acceptance Criteria -- box-drawing content grid (owner decision, confirms Req 29.5-29.9)

30.12 THE content-table grid SHALL be rendered with BOX-DRAWING characters for
      cell borders and vertical separators (a header rule, a header/data separator
      rule, `|`-style vertical separators between cells using box-drawing glyphs,
      and a bottom rule), rather than ASCII dash/pipe rules, as the canonical
      faithful rendering (owner decision). The ASCII-only fallback (Req 5.2)
      remains available where selected. `[CR-NR-115]`
30.13 THE combo cell convention SHALL remain `[value v]` (empty `[ v]`), an empty
      editable cell `____`, and a filled cell its value, IDENTICAL in the
      plain-text and HTML flavors (confirms Req 29.8, 29.9). `[CR-NR-115]`
30.14 THE grid SHALL keep the fixed-width, full-frame-width, scratch-row, and
      widen-on-overflow behaviour of Req 29.6, 29.6a, 29.7, 29.7a (unchanged by
      this requirement -- only the border glyphs become box-drawing per 30.12).
      `[CR-NR-115]`

#### Acceptance Criteria -- general scope, shared path, and framework conformance

30.15 THE common-frame reproduction SHALL be GENERAL to ALL capturable Contexts,
      implemented in the SHARED place (the shared chrome helper + the
      `ff-screen-model` renderer), consistent with Req 29.12 -- no per-Context
      bespoke chrome. `[CR-NR-115]`
30.16 THE SAME reproduction SHALL apply to the CAPTURE path as to the SNAPSHOT
      path, via the one shared `ScreenProvider` seam and renderer (consistent with
      Req 29.14). `[CR-NR-115]`
30.17 THIS requirement SHALL introduce NO new clipboard flavor, NO new framework
      mechanism, and NO reshape of a public `ff-screen-model` / `ff-scrm` type.
      The menu-bar line and the full tab bar are composed from live logical state
      into the EXISTING `ScreenModel` slots (`header_lines` / `body_lines` /
      `status_bar` / `footer_lines`); the message line is reproduced from the
      shell's additive Message_Line state (defined in `workspace-framework`
      Requirement 7). `[CR-NR-115]`

#### Acceptance Criteria -- ruled bands and cells (owner layout, CR-NR-115 refinement)

The snapshot is ONE box made of horizontal BANDS separated by rules. This
refines 30.1-30.6 and 30.8-30.11 (it supersedes any reading of them that leaves a
blank row, a missing rule, or an undivided menu/tab/status line). Snapshot
rendering only: the live egui window keeps its own visuals.

30.18 THE rendering SHALL draw a horizontal RULE between bands: after the title
      bar, after the menu bar, after the tab bar, after the centered title,
      DIRECTLY after the last content row, after the message band (when
      present), and after the status bar when a key line follows. THERE SHALL BE
      NO rule between the last band and the bottom border. THE rule BETWEEN the
      Command line and the first content row IS REQUIRED (the Content_Area
      top-boundary rule, 30.29), replacing the earlier "no rule" wording.
      `[CR-NR-115, revised 2026-11-04, owner Option 1]`
30.19 THE title bar SHALL be one band: a dedicated leading `#` cell, a divider,
      the app title (the single `WINDOW_TITLE` constant), and the window controls
      `_ [] X` RIGHT-ALIGNED against the right border. The only divider in the
      title bar is the one after the `#` cell. `[CR-NR-115]`
30.20 THE menu bar items, the tab bar entries and the status bar segments SHALL
      each be rendered as separate CELLS divided by the vertical glyph (e.g.
      `Settings | Help | SCRM`). WHERE a cell divider meets a rule above or below,
      THE rule SHALL carry the matching junction glyph: a tee-down where the
      divider continues only below, a tee-up where it continues only above, and a
      cross where it continues on both sides; the rule at the top border takes
      tee-down and at the bottom border tee-up. The Unicode glyphs are the ones
      already used by the content grid. `[CR-NR-115]`
30.21 THE `Command ===>` line SHALL show the typed field text followed by
      underscores that fill to the right edge of the content area (no trailing
      gap beyond the common one-space cell padding); an empty field SHALL show
      underscores only. `[CR-NR-115]`
30.22 THERE SHALL be NO blank row before the status bar: the content's closing
      rule is directly followed by the message band (when present) or the status
      band. The message band, when present, has its own rules above and below.
      The key line (only when `pfshow` is on) SHALL be the LAST band inside the
      box, after the status bar's closing rule, followed by the bottom border.
      `[CR-NR-115]`
30.23 THE ASCII fallback (Req 5.2) SHALL produce the equivalent layout using only
      `+ - |`, with `+` at every corner, tee and junction. `[CR-NR-115]`
30.24 OWNER DEFAULTS (each a single named place in code, owner may flip before
      merge): (a) the ACTIVE tab keeps a marker, `>[TAG] x<`
      (`common_frame.rs::ACTIVE_TAB_MARKERS`); (b) the Command line shows the
      field as it was BEFORE the triggering snapshot/capture command, so the
      triggering command (e.g. `ss`) is never echoed and an empty field shows
      underscores only (`common_frame.rs::common_frame_command_field_text`); (c)
      the status profile segment stays `Prof: <name>`
      (`actions.rs::active_profile_label`). `[CR-NR-115]`
30.25 WHERE no message is present THE message band AND its rules SHALL be
      OMITTED ENTIRELY: the content's closing rule leads directly into the
      status band (consistent with 30.9, 30.22). WHERE one or more messages are
      present THE message band SHALL render between the content and the status
      band as its OWN band with a rule above and a rule below; the message band
      is a SINGLE CELL (no internal dividers), so its own rules carry no
      junctions from it (only the status band below may contribute junctions to
      the rule between them). Multiple messages render as multiple rows inside
      that one band. `[CR-NR-115, revised 2026-11-04]`
30.26 THE function-key band SHALL be rendered as ONE CELL: its items stay
      joined by the plain text separator ` | ` inside that single cell, so
      neither the rule above (the status band's closing rule) nor the bottom
      border carries a key-line junction -- only the status band's own cell
      dividers may appear on the rule between the status band and the key
      line. `[CR-NR-115, revised 2026-11-04]`
30.27 THE frame rules above (30.6, 30.11, 30.21-30.26) SHALL hold for EVERY
      Context INCLUDING every Menu Kind (POM, Settings, any `menus/<name>.toml`),
      with no name-specific branch: the menu builder returns the CONTENT AREA
      only (option rows plus calendar), and the shared common frame supplies the
      `Command ===>` line (empty field, underscores filling to the right border,
      never the triggering command), the message band above the status band, the
      status band `Last | Prof | Zoom`, and the key line as the LAST band, one
      cell, with no blank spacer rows. `[CR-NR-115 follow-up, 2026-11-04]`
30.28 A SNAPSHOT / CAPTURE SHALL be taken from the PRE-ACTION state: the message
      band contents and `Last:` time present before the triggering command ran,
      and the command field as it was before that command (see
      `workspace-framework` Requirement 7.29). The PNG picture is the live
      window grab and is unchanged. `[CR-NR-115 follow-up, 2026-11-04]`
30.29 THE rendering SHALL draw a frame-owned horizontal RULE directly BETWEEN the
      `Command ===>` line and the first Content_Area row (a box-drawing junction
      row: left tee, horizontals, right tee; `+ - +` in the ASCII fallback). THE
      rule is composed by the shared framed renderer / common frame, is
      IDENTICAL for EVERY Workspace Kind with no name-specific branch, and marks
      the boundary between the shell area and the workspace-owned Content_Area.
      It is a rule, NOT a blank row. WHEN the Content_Area begins with its own
      top rule (e.g. a table grid), THE frame rule and the grid's top rule SHALL
      MERGE into ONE rule line carrying the grid's column junctions (top-tee) on
      that single line, and THERE SHALL NEVER be two consecutive rules. The merge
      is specified generically in the framed renderer (not per workspace). The
      same rule appears on the LIVE screen (workspace-framework Requirement
      7.30). `[CR-NR-115, NEW 2026-11-04, owner Option 1]`
30.30 WHEN the Content_Area ends with a grid bottom rule, THAT rule and the
      frame rule that precedes the message band (or the status band when no
      message is present) SHALL MERGE into ONE rule line carrying the grid's
      column junctions (bottom-tee), and THERE SHALL NEVER be two consecutive
      rules at the content bottom. WHEN the Content_Area ends with no rule of
      its own, THE single frame rule is drawn as in 30.18. The merge is
      generic in the framed renderer. `[CR-NR-115, NEW 2026-11-04, owner Option 1]`
30.31 PER-KIND CONTENT is owned by each Kind's spec, not by the frame: the
      Menu Kind content area (option rows, calendar block, group rules, the
      Option | Command | Description clipboard table and its empty status
      cells) is specified in `menu-workspace` Requirement 23 (CR-NR-117). A
      per-Kind clipboard table SHALL be supplied through `markdown_sections`
      with that Kind's own column headers (it is NOT limited to Property |
      Value), and SHALL show the same displayed texts as the box rows (a
      snapshot never adds text that is not on the live screen). No change to
      the frame criteria 30.1-30.30 or to the `ScreenModel` / `Table` types is
      implied. `[CR-NR-117]`

#### Acceptance Criteria -- content grid drawn in the frame's rules (Alias per-Kind gate, CR-NR-116, building on CR-NR-115, 2026-11-04)

Added by the first per-Workspace-Kind gate (Alias, `command-configurator`
Requirement 7.14-7.28). SHARED renderer behaviour: every Context whose model
carries `tables` inherits it. Renderer-internal; no public `ScreenModel` /
`Table` shape changes (criterion 30.17 stands).

> NOTE ON TAGGING: criteria 30.32-30.40 below carry the `[CR-NR-115]` tag
> because they EXTEND the common-frame family; the owning change-log id for this
> gate is CR-NR-116. The tag marks the framework family these criteria build on,
> not their ownership.

30.32 WHEN a framed model (one carrying `header_lines`) contains `tables`, THE
      framed renderer SHALL draw each table INSIDE the frame's own ruled bands
      instead of as a nested box: a rule, one header band, a header/data rule,
      then one band per data row; the Content_Area's closing rule (30.18)
      follows the last data band. It SHALL NOT emit a blank row before the
      table, a caption row, or an inner top/bottom border. The table's
      `caption` remains available to the Markdown / HTML flavors (their
      heading) and is simply not drawn in the framed box. `[CR-NR-115]`
30.33 THE table's column widths SHALL be, per column, the widest of its header
      and all its cells (so dividers align across the header and every data
      row), with each cell padded to its column width and dividers drawn as
      band-cell dividers (` | ` rendered as the vertical glyph). The last
      column's right edge is the frame's right border. A cell longer than its
      provider-declared field width widens the column (and, if needed, the
      frame); NO cell is truncated (consistent with 29.6, 30.14). `[CR-NR-115]`
30.34 THE rules around the grid SHALL carry the junction glyphs of 30.20: the
      grid's top rule a tee-down at every column divider, the header/data rule a
      cross, and the closing rule a tee-up at every column divider (a cross
      where the status band below has a divider at the same column, a tee-down
      where only the status band has one). The existing band/junction machinery
      SHALL be reused; no second junction mechanism is introduced. `[CR-NR-115]`
30.35 RECONCILED (rebase onto fd35620): the grid's OWN top rule MERGES with the
      frame-owned rule the shell draws between the `Command ===>` line and the
      first Content_Area row (main's 30.29 frame rule + 30.30 grid-merge). When
      the first content row is a grid, the single rule under the Command line is
      the MERGED rule, carrying the grid header's tee-down junctions; it is NOT a
      frame-suppressed rule. A Content_Area with no grid renders the plain frame
      rule (30.29). (SUPERSEDES the gate's earlier wording "the frame adds NO
      rule between the Command line and the first content row", which predated the
      CR-NR-115 frame rule that landed on main.) `[CR-NR-115]` `[OWNER CONFIRM c]`
30.36 EACH cell string SHALL be carried as a DISCRETE value and SHALL NOT be
      re-split on ` | `; a cell containing ` | ` stays in its cell. Control
      characters (tab, CR, LF) in a cell SHALL render as a space and SHALL NOT
      be interpreted as the right-align marker. `[CR-NR-115]`
30.37 WHEN a model carries more than one table, or a table follows body lines /
      fields, THE tables SHALL be laid out one after another with adjacent rules
      COALESCED into one (never two consecutive rules); no blank spacer row is
      inserted between them. A table with NO rows renders its header band and
      rules only; a table with NO columns is omitted. `[CR-NR-115]`
30.38 THE ASCII fallback (Req 5.2) SHALL produce the equivalent layout with
      `+ - |` (30.23). `[CR-NR-115]`
30.39 THE legacy renderer path (`render_plain` for a model WITHOUT
      `header_lines`) SHALL be UNCHANGED, including its below-the-frame
      `render_table_grid`; only the framed path gains 30.32-30.38. Tables of
      other Contexts (Theme, Menu, Keys, Kinds) adopt the new form automatically;
      their goldens/tests are regenerated in the same change and reviewed in
      their own per-Kind gates. `[CR-NR-115]`
30.40 A Context MAY supply `markdown_sections` (Req 28.6) to present RAW values
      in the Markdown / HTML-table flavors while `tables` carries the
      box-convention strings (`[value v]`, `____`, padding). WHEN
      `markdown_sections` is non-empty the structured flavors use it and ignore
      `fields`/`tables` (existing behaviour), so 29.7a / 30.13 govern the BOX
      and HTML `<pre>` text and the structured flavors may carry raw data. This
      is the SAME `markdown_sections` mechanism the Menu Kind uses for its
      Option|Command|Description clipboard table (30.31, CR-NR-117).
      `[CR-NR-115]` `[OWNER CONFIRM g]`

Rendered fixture (the box-drawing glyphs are runtime product output, exempt from
the ASCII-only documentation rule):

```
┌───┬─────────────────────────────────────────────────────────┐
│ # │ FileForge Workbench                              _ [] X │
├───┴──────┬──────┬───────────────────────────────────────────┤
│ Settings │ Help │ SCRM                                      │
├──────────┴───┬──┴───────────────────────────────────────────┤
│ >[CONFIG] x< │ [POM] x                                      │
├──────────────┴──────────────────────────────────────────────┤
│                        Configuration                        │
├─────────────────────────────────────────────────────────────┤
│ Command ===> ______________________________________________ │
├─────────────────────────────────────────────────────────────┤
│ (content rows)                                              │
├─────────────────────────────────────────────────────────────┤
│ info one                                                    │
├─────────────┬───────────────────────────────────┤
│ Last: 11:02 │ Prof: default                     │
└─────────────┴───────────────────────────────────┘
```

Verified by the GOLDEN tests (`tests_common_frame_golden.rs` with the
`shell/golden/*.txt` fixtures, and `render/framed_tests.rs`), which compare the
full rendered text line for line. `[CR-NR-115]`

#### Investigation note (settled, see design.md Section 13)

- The FULL tab bar (all open tabs + tags + close affordance) is available to the
  snapshot model from `TabManager` + `BuiltinKind::default_title()`; today's
  menu/POM chrome builder emitted no tab line at all, and the editor/panel
  builders emitted at most the active tag. Reproducing the full tab bar is a
  composition change in the shared chrome helper, not a model reshape.
- The menu bar text is the live menu echo (CR-NR-114); composing its line is
  logical-state composition, no egui read.
- The message line is reproduced from the shell's additive Message_Line state
  (workspace-framework Requirement 7); when empty it is omitted.

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

Requirement 28 (CR-CH-079) is appended after Requirement 27 with the next free
sequential number (28); it extends Requirement 23 (the three-flavor clipboard) so
that EACH text flavor carries BOTH the box-drawn layout AND the property/value
tables.

Requirement 29 (CR-CH-115) is appended after Requirement 28 with the next free
sequential number (29); it extends Requirement 3.8-3.13 (the full-screen logical
model, previously defined for the POM/menu Context) to EVERY capturable Context
as a faithful full-window box-drawn mirror, and extends the SAME fidelity to the
capture path. It supersedes the 2026-11-01 Bug-2 "no header block" correction FOR
THE WINDOW TITLE BAR + TAB TAG specifically (29.15).

Design-review revision 1 (addressing
`.agents/tasks/snapshot-fidelity-gate/design-review.json`) added two sub-criteria
to Requirement 29 -- 29.6a (the frame-width source, `model.dimensions.columns`,
distinct from the per-column max of 29.6) and 29.7a (the scratch row is a
provider-owned row so it appears identically in the plain-text and HTML flavors)
-- and tightened 29.1/29.2 (title bar + tab tag are the first two in-frame body
lines, reconciled with the real `render_plain` order), 29.4 (underscore width =
POM/menu field width), 29.6/29.8/29.9 (widen-on-overflow, no truncation), and
29.10 (settled-but-tunable status source; defined-blank info bar when status is
empty). Req 9.14 was revised to a real window screenshot to align with Req 1.3 /
23.2.

Requirement 30 (CR-NR-115) is appended after Requirement 29 with the next free
sequential number (30); it specifies how the snapshot/capture reproduces the
COMMON WORKSPACE FRAME defined normatively in `workspace-framework` Requirement 7.
It REVISES Requirement 29.2 (the snapshot reproduces the FULL tab bar -- all open
tabs + tags + close affordance -- not only the active tag; owner decision D3),
EXTENDS the reproduced frame with the live menu-bar line (owner decision D4) and
the message line (when present), REVISES Requirement 29.10's status-bar text to
the common+per-Kind segment model (last-action time, `Prof:`, no `RUNNING`), and
CONFIRMS the box-drawing content-grid rendering (owner decision). It introduces no
new clipboard flavor, no new framework mechanism, and no public-type reshape; the
menu bar / full tab bar / message line are composed from live logical state into
the existing `ScreenModel` slots. The owner's discrepancy review D1 (single
title) is covered by the single-chrome rule; D2 (data rendered twice) by the
single-representation principle (workspace-framework Req 7.24, with the Alias-
specific provider fix deferred to the Alias follow-up Kind gate); D5 (Parameters
value lost) and D6 (status content) are workspace-content / status fidelity
handled by the per-Kind follow-up gates and the common status model respectively.


---

## Requirement 31 -- SVG snapshot format + configurable default snapshot format (CR-NR-118)

**User story:** As a user who pastes screen snapshots into documents, I want an
SVG snapshot format as an alternative to the raster PNG, and a configuration item
to choose which format is my default, so I can get a smaller, scalable, selectable
image where my document tool supports it while keeping PNG where I need maximum
paste compatibility.

### Status

PENDING GATE / QUEUED AFTER PORTABILITY. This requirement is RECORDED now at the
owner's direction but is NOT scheduled for implementation until the
portable-installation work (`docs/specs/portable-installation/`) is complete. It
is logged as CR-NR-118 in `docs/status/change-log.md`. The gate CONFIRM step and
implementation follow the portability phase.

### Context (owner-confirmed 2026-11-05)

The current picture snapshot flavor is a REAL WINDOW SCREENSHOT raster (PNG),
established by CR-NR-114 (Req 1.3, 23.2). The owner observed that, because the
FileForge screen is a text/colour/box-drawing grid rather than a photograph, an
SVG rendered from the LOGICAL screen model would typically be SMALLER than the
PNG and remain a SCALABLE, selectable-text image. This requirement ADDS SVG as a
snapshot output format, ADDS a config item selecting the default snapshot format
(PNG or SVG), and sets the RESET-BARE / Recovery_Source default to SVG.

This BUILDS ON the existing framework: the `ScreenProvider` seam (Req 2), the
`ff-screen-model` renderers (Req 5), the single command dispatch path (Req 11),
the write-through configuration model (CR-CH-116, configuration-system Req 23),
and the multi-flavor clipboard model (Req 23). It introduces NO new command
dispatcher, NO new navigation stack, and NO new persistence format. The SVG
renderer is a NEW renderer over the SAME logical model the PNG/TEXT/HTML flavors
already use -- a sibling of the existing renderers, not a new capture mechanism.

### Relationship to the existing PNG picture flavor (honesty about scope)

- The PNG picture flavor (Req 23.2) is a true WINDOW SCREENGRAB (actual rendered
  pixels). The SVG flavor is DIFFERENT in kind: it is rendered FROM the logical
  Screen_Model (text runs, colours, rectangles, box-drawing), the same source as
  the TEXT/HTML flavors -- NOT a trace of the raster. The two therefore differ by
  design: PNG is pixel-faithful; SVG is logical-model-faithful and selectable.
- The clipboard caveat is explicit (see Req 31.5): Word's clipboard support for
  SVG is inconsistent across versions, so the default clipboard behaviour and the
  file-output behaviour are specified separately. The gate design step MUST
  settle the clipboard-vs-file behaviour for SVG rather than assume it.

### Acceptance Criteria -- SVG as a snapshot format

31.1 THE set of SNAPSHOT picture-output formats SHALL be EXTENDED to include SVG,
     alongside the existing PNG (Req 23.1, 23.2) and PDF (Req 23.3). `[CR-NR-118]`
31.2 WHEN the user issues `SNAPSHOT SVG` THE system SHALL produce an SVG rendering
     of the captured screen, derived from the Screen_Model (text runs, colours,
     rectangles, box-drawing), NOT a rasterised or traced image. The screen text
     SHALL be emitted as real SVG `<text>` so it remains SELECTABLE and SEARCHABLE
     (consistent with the selectable-text constraint, Req 1.1, 1.4). `[CR-NR-118]`
31.3 THE `SNAPSHOT SVG` verb SHALL resolve through `ff_command::resolve_target`
     to a `Function` target and dispatch through the shared path, with command
     parity for any menu/toolbar affordance (Req 11.3, 11.4); NO bespoke
     intercept SHALL be added. `[CR-NR-118]`
31.4 WHERE the snapshot targets a FILE, the SVG output SHALL be a standalone,
     self-contained `.svg` (fonts referenced by family, no external asset
     dependencies). The system MAY additionally offer a gzip-compressed `.svgz`
     file variant for size; whether `.svgz` is offered SHALL be settled at the
     gate design step. `[CR-NR-118]`

### Acceptance Criteria -- clipboard behaviour for SVG (to be settled at design)

31.5 THE design step SHALL settle how an SVG snapshot that targets the CLIPBOARD
     behaves, given that Word's clipboard SVG support is inconsistent across
     versions. The settled behaviour SHALL be ONE of: (a) place SVG on the
     clipboard in an SVG-capable flavor WHILE ALSO offering a PNG/DIB fallback
     flavor for targets that cannot consume SVG (the multi-flavor approach of
     Req 23.4, extended with an SVG flavor); or (b) keep the clipboard picture
     flavor as PNG for paste compatibility and treat SVG as a FILE/save output
     only. The decision SHALL be recorded in design.md with its rationale; this
     criterion is satisfied by the recorded decision, not by a specific choice.
     `[CR-NR-118]`
31.6 WHERE SVG is offered as a clipboard flavor (31.5 option (a)) THE
     selectable-text and the PNG-fallback flavors SHALL be published in ONE
     clipboard-open transaction (consistent with the single-transaction rule of
     Req 23.4, 23.5d). `[CR-NR-118]`

### Acceptance Criteria -- configurable default snapshot format

31.7 THE configuration SHALL carry a snapshot DEFAULT-FORMAT item whose value
     selects the default picture format a bare `SNAPSHOT` (no explicit format
     argument) and the default UI snapshot affordance produce. The item's allowed
     values SHALL include at least `PNG` and `SVG`. (The exact config key name and
     its placement in the config schema SHALL be settled at the gate design step,
     consistent with configuration-system.) `[CR-NR-118]`
31.8 WHEN the user issues a bare `SNAPSHOT` (no explicit format) THE system SHALL
     produce the picture flavor named by the default-format config item (31.7),
     composited with the selectable TEXT / rich-HTML flavors exactly as the
     existing composite clipboard payload does (Req 4.1, 23) -- i.e. the config
     item selects WHICH picture format fills the picture slot of the composite,
     without otherwise changing the composite model. `[CR-NR-118]`
31.9 WHEN the user issues `SNAPSHOT <format>` with an EXPLICIT format
     (`PNG`/`SVG`/`PDF`/`TEXT`/`ANSI`/`MARKDOWN`/`MD`/`HTML`/`YAML`/`AI`) THE
     explicit format SHALL OVERRIDE the default-format config item for that
     invocation (consistent with Req 4.8). The explicit verb list SHALL now
     include `SVG`. `[CR-NR-118]`
31.10 THE RESET-BARE / Recovery_Source DEFAULT value of the snapshot
     default-format config item SHALL be `SVG`. WHEN the user issues RESET BARE
     (or on first-run write, per the write-through model CR-CH-116) THE rebuilt
     config SHALL set the snapshot default format to SVG. A user who prefers PNG
     SHALL be able to change the config item to `PNG`. `[CR-NR-118]`
31.11 THE default-format config item SHALL be written as part of the
     Complete_Config_File on first run and rebuilt by RESET BARE from the
     Recovery_Source via the shared Config_Rebuild_Helper (configuration-system
     Req 23 / 19.17), consistent with every other write-through config value; the
     compiled default (SVG, 31.10) is the Recovery_Source only. `[CR-NR-118]`

### Acceptance Criteria -- conformance and non-regression

31.12 ADDING an SVG renderer / dependency SHALL be pinned to an exact version
     (`.kiro/steering` dependency discipline), consistent with Req 23.7. The
     logical-model-first constraint SHALL NOT be weakened to add it. `[CR-NR-118]`
31.13 THE existing PNG picture flavor (Req 23.2, 23.2a crop), the selectable-text
     flavors (Req 28), and the capture/recorder path (Req 9.13-9.14) SHALL be
     UNCHANGED by this requirement EXCEPT where the default-format config item
     selects SVG for the composite picture slot (31.8). A user whose default
     remains PNG SHALL see no behaviour change. `[CR-NR-118]`
31.14 WHERE the capture/recorder path stores a per-Capture picture (Req 9.13) THE
     format of that stored picture MAY follow the same default-format config item;
     whether the recorder honours the SVG default or always stores PNG SHALL be
     settled at the gate design step and recorded. `[CR-NR-118]`
