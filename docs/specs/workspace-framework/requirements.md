# Requirements Document -- Workspace Context Framework (CR-NR-078)

## Introduction

This spec proposes a trait-based framework that makes the workspace-Context
contract COMPILER-ENFORCED. Today every workspace Context is rendered by a hand
-written arm of a `match self.tabs.active_tab().kind { ... }` in
`crates/ff-desktop/src/shell/render.rs`, and each arm must remember to report its
interior focus stops (`first_interior_id` / `last_interior_id`) and call
`honour_interior_focus_latch`. Forgetting that ritual produces a phantom
"invisible" Tab stop -- the SAME bug shipped four times (B056 POM, B057 Theme
Editor, B058 Config Panel, B059 the five remaining panels). The interim guardrail
is the `.kiro/steering/workspace-conformance.md` rule plus a full-shell test per
workspace, but the correct fix is to make the contract a value the compiler
demands, not a convention a human can skip.

This is a DESIGN PROPOSAL for owner review. No framework code is written until the
requirements gate is approved. The design lives in `design.md`.

### Goals

1. Every workspace Context implements ONE trait; the shell dispatches to it
   through a single code path (no per-kind `match` arm ritual).
2. The trait's render method RETURNS its interior focus contract, so an
   implementor CANNOT compile without providing it (the phantom-stop bug becomes
   unrepresentable).
3. No observable behaviour change: after migration, every Context behaves exactly
   as it does today (same rendering, same focus order, same actions), verified by
   the existing full-shell egui_kittest tests.
4. Migration is INCREMENTAL and behaviour-preserving: one Context at a time, each
   step green under `verify.ps1`, never a big-bang rewrite.

### Non-goals

- Not changing the visual appearance, layout, or command behaviour of any
  Context.
- Not replacing the `ff-layout` docking model (`DockablePanel`); this framework is
  the tab-Context-level contract and MUST reconcile with `DockablePanel` (see
  design.md), not compete with it.
- Not introducing class inheritance (Rust has none); the "parent/menu/custom
  class" idea from the prompt is realised as a trait + composition.

### Source References

- **[CR-NR-078]** = this change request (owner: "a custom workspace template ...
  so we would not be hitting the same problems from one workspace to the next").
- **[CR-CH-023]** = unified tab-order model (menu-and-statusbar Req 16), the
  contract this framework enforces.
- **B056 / B057 / B058 / B059** = the four instances of the phantom-Tab-stop bug
  this framework prevents structurally.
- **[layout-and-docking]** = existing `DockablePanel` trait to reconcile with.

### Cross-References

- **`menu-and-statusbar`** Requirement 16 (Boundary_Policy / interior focus).
- **`layout-and-docking`** (`DockablePanel::render`).
- **`.kiro/steering/workspace-conformance.md`** (the interim rule this supersedes
  once the framework lands).

## Glossary

| Term | Definition |
|------|-----------|
| **Workspace_Context** | The content inside a Workspace tab (Home/POM, Editor, Config, Theme Editor, Menus Editor, Plugin Manager, Event Log, Macro Library, Search Results, Command Configurator, Menu_Workspace). |
| **WorkspaceContext** (trait) | The proposed trait every Workspace_Context implements; its `render` returns an `InteriorFocus`. |
| **InteriorFocus** | The value a Context returns from `render`: the FIRST and LAST interior focus-stop `egui::Id`s (both optional) the shell Boundary_Policy uses. |
| **ShellServices** | A context object passed to `render` that mediates a Context's access to shell services (command dispatch, config, runtime, notifications, themes/menus dirs) so the borrow checker is satisfied. |
| **Boundary_Policy** | The shell-level Tab handling (CR-CH-023): command-field -> first interior; last interior -> menu bar; wrap; Shift+Tab reverse. |

## Requirements

### Requirement 1: The WorkspaceContext trait

**User Story:** As a workbench developer, I want every workspace Context to
implement one trait whose render method returns its interior focus contract, so
that the phantom-Tab-stop bug cannot be written.

#### Acceptance Criteria

1. THE framework SHALL define a `WorkspaceContext` trait with a render method
   whose RETURN TYPE is `InteriorFocus` (carrying the first and last interior
   focus-stop ids, each `Option<egui::Id>`). An implementor cannot compile
   without returning it.
2. THE shell SHALL dispatch the active Workspace_Context's render through the
   trait on a SINGLE code path, and SHALL feed the returned `InteriorFocus` to
   `honour_interior_focus_latch` in that one path (Requirement 3 of CR-CH-023 /
   menu-and-statusbar Req 16.3/16.8). No per-kind `match` arm SHALL re-implement
   the focus-latch ritual.
3. WHERE a Context genuinely has no interior focus stops, its render SHALL return
   `InteriorFocus::none()` (both `None`) EXPLICITLY -- a documented, deliberate
   value, not an accidental omission.
4. THE trait SHALL NOT require inheritance; each Context implements it by
   composition. `MenuWorkspaceState` (the POM / Settings / user menus) and every
   custom panel (Config, Theme Editor, Menus Editor, Plugin Manager, Event Log,
   Macro Library, Search Results, Command Configurator) SHALL each be an
   implementor.
5. THE observable behaviour of every migrated Context SHALL be IDENTICAL to its
   pre-migration behaviour (rendering, focus order, actions), verified by the
   existing full-shell egui_kittest first-Tab tests and the per-Context tests.

### Requirement 2: ShellServices access mediation

**User Story:** As a workbench developer, I want a Context's render to reach the
shell services it needs without the shell handing it `&mut self`, so the borrow
checker permits trait dispatch while `&mut Ui` is also borrowed.

#### Acceptance Criteria

1. THE framework SHALL define a `ShellServices` context object passed to
   `render`, exposing the services Contexts need (command dispatch / a way to
   enqueue a command or `CommandTarget`, config handle read access, the Tokio
   runtime handle, the notification queue, and the resolved themes/menus
   directories), WITHOUT exposing `&mut WorkbenchShell` wholesale.
2. A Context's render SHALL communicate state-changing intent to the shell by
   RETURNING an action / enqueuing a request through `ShellServices`, not by
   mutating shell fields directly, so the existing "pure render -> action"
   pattern (Theme Editor / Menus Editor / Command Configurator) is preserved and
   generalised.
3. THE `ShellServices` object SHALL be constructed by the shell each frame from
   its own fields; a Context SHALL NOT retain it across frames.

### Requirement 3: Incremental, behaviour-preserving migration

**User Story:** As a maintainer, I want the framework adopted one Context at a
time so each step is safe and reviewable.

#### Acceptance Criteria

1. THE migration SHALL convert Contexts to the trait ONE at a time; after each
   conversion the full test suite and `verify.ps1` SHALL be clean.
2. EACH converted Context SHALL keep (or gain) its full-shell first-Tab
   egui_kittest test (workspace-conformance rule); the test SHALL pass unchanged
   in meaning after conversion.
3. WHEN all Contexts are converted, the per-kind focus-latch ritual SHALL be
   removed from `render.rs`, leaving the single trait-dispatch path (Requirement
   1.2). The `workspace-conformance.md` steering rule SHALL be updated to point at
   the trait as the enforcement mechanism.
4. THE two special cases -- FilesPanel (bespoke B024 catalog-focus handling) and
   FileEditor (natively-focusable text body) -- SHALL be accommodated by the trait
   (each returns the appropriate `InteriorFocus`), NOT exempted from it. WHERE the
   editor body's focusable id cannot be reported as a stable interior id, the
   design SHALL document the chosen approach (e.g. reporting the editor widget id
   or an explicit `InteriorFocus::none()` with the reason).

### Requirement 4: Reconciliation with DockablePanel

**User Story:** As an architect, I want one panel/Context abstraction, not two
competing ones.

#### Acceptance Criteria

1. THE design SHALL reconcile `WorkspaceContext` with the existing
   `ff-layout::DockablePanel` trait (which has `render(&mut self, ui)` and no
   focus return): EITHER by extending `DockablePanel` to carry the `InteriorFocus`
   return, OR by defining `WorkspaceContext` as the tab-Context contract layered
   above `DockablePanel`, with the relationship stated explicitly in design.md.
2. THE chosen reconciliation SHALL preserve `ff-layout`'s GUI-independence
   constraint (ff-layout depends only on egui for the render signature; it never
   depends on ff-desktop).

### Requirement 5: Naming

#### Acceptance Criteria

1. THE trait SHALL be named `WorkspaceContext`, the focus return type
   `InteriorFocus`, and the access object `ShellServices` (owner-approved). These
   names are consistent with the canonical UI terminology (Workbench / Workspace /
   Context).

### Requirement 6: Host-agnostic render (detach-ready)

**User Story:** As the owner, I want to detach a Workspace into its own OS-level
window (so the OS window manager can tile and Alt-Tab between Workspaces) while
running a SINGLE FFWB process, and I do not want that to require rewriting each
Context.

#### Acceptance Criteria

1. THE `WorkspaceContext::render` method SHALL be HOST-AGNOSTIC: it takes a
   `&mut egui::Ui` and a `ShellServices`, returns an `InteriorFocus`, and SHALL
   NOT assume it is being drawn in the main window's central panel. The SHELL
   chooses the draw target.
2. THE framework SHALL make a Context renderable in ANY host without per-Context
   change: (a) a tab in the main window (now), (b) a dock zone (future
   `ff-layout`), and (c) a DETACHED OS-level egui viewport of the SAME FFWB
   process (future "Detached Workspace", layout-and-docking). A detached
   Workspace is a second VIEWPORT, not a second process, and shares all shell
   state.
3. WHERE a Context is drawn in a detached viewport, THAT viewport's own
   command-line / Boundary_Policy SHALL reuse the SAME `InteriorFocus` the Context
   reports, so Tab-order focus is correct in the detached window identically to
   the tab (no separate focus logic per host).
4. This requirement constrains the framework SHAPE only; the detach FEATURE
   itself is delivered separately (layout-and-docking, "Detached Workspace") and
   is out of scope for the phase-1 migration. The framework MUST NOT preclude it.

---

### Requirement 7: The Common Workspace Frame -- common-vs-unique boundary (CR-NR-115)

**User Story:** As the architecture owner, before building the next major
extensions I want ONE model that says exactly which parts of every workspace are
the SHELL's common frame and which parts each workspace / command environment
owns, so that every workspace -- current and future -- has the same chrome drawn
exactly once, and the per-workspace work that follows only has to fill the
content area and contribute its own status content.

#### Context (owner-confirmed 2026-11-03, CR-NR-115)

The owner pasted a full-window sketch of the ISPF-style frame that is COMMON to
every workspace and asked, as the explicit first step of the
workspace-architecture alignment, to "separate out two parts: what is common to
all workspaces and what is unique to each workspace ... lets try getting the
common parts right first then we can focus on each workspace at a time" and
confirmed "lets first get the common right then we can have a gate per workspace
kind". This requirement is the BACKBONE of that alignment: it defines the common
frame NORMATIVELY. The snapshot/capture reproduction of this frame is specified
in `screen-snapshot-scrm` Requirement 30 (which revises Requirement 29); the
status-bar and function-key-bar rendering detail is owned by `menu-and-statusbar`
(Requirement 5, 7, 8, 12, 16). This requirement is the single place the
common-vs-unique boundary is stated; the other specs build on it.

This BUILDS ON existing framework seams (framework-conformance.md): the shell
chrome already renders the window title bar (`main.rs`), the menu bar, the tab
bar, the Title_Line, the `Command ===>` field, the Status_Bar
(`shell/render_status.rs`), and the key-label bar
(`shell/render_status.rs::render_key_label_bar`, driven by `pfshow`); the
`WorkspaceContext` trait + `render_workspace_context` dispatch already draws each
Context's content into the central panel. This requirement names the boundary
between the two; it introduces NO new command-dispatch path, NO new navigation
stack, and NO reshape of a public framework type. The one piece of NEW shell
state it implies -- a transient Message_Line -- is ADDITIVE shell state (a new
field/area), NOT a framework-type reshape; it is called out for owner
confirmation in the design.

#### Glossary additions

| Term | Definition |
|------|-----------|
| **Common_Frame** | The set of chrome elements the SHELL owns and renders exactly once around every Workspace, regardless of Kind: the Window_Title_Bar, the Menu_Bar, the Tab_Bar, the Workspace_Title, the Primary_Command_Field, the Message_Line, the Status_Bar, and the Function_Key_Bar. |
| **Content_Area** | The region between the Primary_Command_Field and the Message_Line that each Workspace / Command Environment owns and fills; the shell frame renders no content here. |
| **Message_Line** | A transient informational line rendered directly ABOVE the Status_Bar, showing message(s) raised by the last action; cleared when the next action is taken. |
| **Common_Status_Segment** | A Status_Bar segment the SHELL owns for every Workspace: the last-action time and the active profile ONLY (revised 2026-11-04; Ln/Col, encoding and line count moved to the Workspace_Status_Segment below). |
| **Workspace_Status_Segment** | A Status_Bar segment whose content a specific Workspace Kind owns and contributes (e.g. the Alias Kind's `Aliases: nnn | Rows 1-nnn visible`). |

#### Acceptance Criteria -- the eight common-frame elements (shell-owned, each exactly once)

1. THE Common_Frame SHALL consist of EXACTLY these eight shell-owned elements,
   rendered EXACTLY ONCE around every Workspace regardless of Kind, in this
   top-to-bottom visual order: (1) Window_Title_Bar, (2) Menu_Bar, (3) Tab_Bar,
   (4) Workspace_Title, (5) Primary_Command_Field, [Content_Area], (6)
   Message_Line, (7) Status_Bar, (8) Function_Key_Bar. No Workspace SHALL draw
   its own copy of any of these elements (single-chrome).
2. THE Window_Title_Bar SHALL show the application brand token `FileForge
   Workbench` (the brand token is NOT translated) followed by the window-control
   glyphs (`_ [] X`, plain ASCII). Descriptive / chrome text other than the brand
   token SHALL be translatable via the existing locale mechanism. The brand text
   SHALL be the one live window title, not a second hardcoded copy.
3. THE Menu_Bar SHALL be the live menu bar (e.g. `Settings | Help | SCRM`),
   rendered once by the shell above the Tab_Bar (menu-and-statusbar Requirement
   2.1, Requirement 22).
4. THE Tab_Bar SHALL show ALL open Workspace tabs with their `[XXX]` Tab_Header
   tags (e.g. `[ALIASES]  [CONFIG]`) and each tab's close affordance -- the full
   tab bar, not only the active tab's tag. The `[XXX]` tag is the terse bracketed
   Tab_Header tag, distinct from the descriptive Workspace_Title (CR-NR-090 /
   CR-CH-045).
5. THE Workspace_Title SHALL be the descriptive display title of the active
   Workspace, rendered CENTERED on its own line, exactly once.
6. THE Primary_Command_Field SHALL be the `Command ===>` line carrying the
   current typed value followed by an underscore input area (`Command ===>
   <value>____...`), rendered once near the top under the Workspace_Title.
7. THE Content_Area between the Primary_Command_Field and the Message_Line SHALL
   be owned and filled by the active Workspace / Command Environment; the shell
   Common_Frame SHALL NOT render any content into it. For the purpose of this
   backbone the Content_Area is an OPAQUE region the Workspace fills.
8. THE Message_Line SHALL be rendered by the shell directly ABOVE the Status_Bar
   (see Requirement 7 message-line criteria below).
9. THE Status_Bar SHALL be a single shell-owned line composed of Common_Status_
   Segments PLUS a Workspace_Status_Segment (see Requirement 7 status-bar
   criteria below).
10. THE Function_Key_Bar SHALL be rendered or hidden per the `pfshow` setting
    (see Requirement 7 PF-key criteria below).

#### Acceptance Criteria -- the Message_Line (transient, multi-message, cleared on next action)

11. WHEN an action raises one or more informational messages THE Message_Line
    SHALL display them in the line directly above the Status_Bar.
12. THE Message_Line SHALL be CLEARED as soon as the NEXT action is taken, so a
    message lives only until the following action; between actions it persists.
13. WHERE multiple messages are present THE Message_Line MAY EXPAND to show more
    than one message line.
14. WHERE no message is present THE Message_Line SHALL be absent / empty (it is
    not a permanently reserved blank line like the Status_Bar).
15. THE Message_Line SHALL REPLACE the former `RUNNING` Status_Bar phase segment:
    the `RUNNING` segment is redundant and SHALL be removed from the Status_Bar in
    favour of the Message_Line (owner decision).

#### Acceptance Criteria -- the Status_Bar common + per-workspace segment model
(REVISED 2026-11-04, owner decision: common vs workspace-owned split)

16. THE Status_Bar SHALL contain ONLY these Common_Status_Segments, shell-owned
    for EVERY Workspace: a LAST-ACTION time `Last: HH:MM` (updated on each
    action, NOT a continuously-ticking clock -- owner picked last-action over
    started/current) and `Prof: <ActiveProfileName>` (the active profile name,
    abbreviated from the former `Profile: default`). `[REVISED 2026-11-04]`
16a. THE cursor `Ln <n>, Col <n>`, the encoding (e.g. `UTF-8`), and the line
     count are WORKSPACE-OWNED segments, NOT common: they describe the DATA a
     Workspace shows (the shell's own UI is always Unicode internally; encoding
     is only meaningful for the content a document-backed Workspace displays --
     a mainframe-dataset editor may be EBCDIC, a local file Latin-1/UTF-16 --
     and Ln/Col/line count describe a document cursor, which a settings-type
     Workspace such as Config or Alias does not have). They are emitted as part
     of the Workspace_Status_Segment (criterion 17) by any Kind that has a
     document cursor. `[NEW 2026-11-04]`
17. THE Status_Bar SHALL ALSO contain a Workspace_Status_Segment (one or more
    cells) whose content is decided by the ACTIVE Workspace Kind. THE editor
    Context (file/untitled buffer) SHALL contribute `Ln <n>, Col <n>`, the
    encoding, and the line count AS ITS Workspace_Status_Segment (no regression
    from the pre-split behaviour for the editor). EVERY OTHER Kind contributes
    an EMPTY Workspace_Status_Segment until its own per-Kind follow-up gate
    decides its content (e.g. the Alias Kind MAY later contribute
    `Aliases: nnn | Rows 1-nnn visible`). The common segments are shell-owned;
    the per-Kind segment is workspace-owned. So the Status_Bar =
    Common_Status_Segments (Last, Prof) + the active Kind's
    Workspace_Status_Segment cells (possibly none). `[REVISED 2026-11-04]`
18. THE former `Started HH:MM` segment SHALL be REPLACED by the `Last: HH:MM`
    last-action time (owner decision Q2: last-action time, not started/current).
19. THE former `Profile: default` / `Profile Default` segment SHALL be shortened
    to `Prof: <ActiveProfileName>` (owner decision Q3).
20. THE `RUNNING` phase segment SHALL NOT appear in the Status_Bar (superseded by
    the Message_Line, criterion 15).
21. THE DEFINITION of each Workspace Kind's Workspace_Status_Segment CONTENT
    (beyond the editor Context's Ln/Col/encoding/line-count default, criterion
    17) is OUT OF SCOPE for this common-frame requirement and SHALL be decided in
    the per-Workspace-Kind follow-up gates; this requirement defines only that
    the segment EXISTS, is Kind-owned, and that Config/Alias/other non-document
    Kinds show NOTHING there until their own gate.

#### Acceptance Criteria -- the Function_Key_Bar (pfshow-driven)

22. WHERE the `pfshow` setting is ON (the live `key_bar_visible` state) THE
    Function_Key_Bar SHALL be rendered at the bottom of the frame; WHERE `pfshow`
    is OFF THE Function_Key_Bar SHALL be omitted entirely. Its visibility is
    driven by `pfshow`, not merely "shown when visible".

#### Acceptance Criteria -- the common-vs-unique boundary (stated normatively)

23. THE boundary SHALL be: the SHELL owns and renders the eight Common_Frame
    elements (criteria 1-10) and the Common_Status_Segments (criterion 16); each
    WORKSPACE / Command Environment owns the Content_Area (criterion 7) and its
    Workspace_Status_Segment (criterion 17), and nothing else of the frame. A
    Workspace SHALL NOT render any Common_Frame element itself (single
    representation of each chrome element -- the single-representation principle,
    which also forbids a Workspace re-rendering its own title or command line
    inside the Content_Area).
24. THE single-representation principle SHALL hold: each piece of screen content
    SHALL be rendered exactly ONCE. A Workspace SHALL NOT duplicate data that is
    already shown in the frame or in its own primary view (e.g. a fields block
    that repeats grid data is a single-representation violation). The common-frame
    rule here is the PRINCIPLE; the per-Workspace fix for a specific duplication
    is a follow-up Kind gate.
25. THE per-Workspace-Kind CONTENT of the Content_Area and the
    Workspace_Status_Segment SHALL be specified in FOLLOW-UP gates, one per
    Workspace Kind (Alias, Menu, Kinds, Config, Theme, ...), each building on this
    common-frame contract. This requirement is deliberately the COMMON part only.

#### Acceptance Criteria -- framework conformance

26. THE Common_Frame SHALL be composed PURELY from live logical shell state (the
    window title, the active Kind's tab tag, the command text, the live status
    sources, `key_bar_visible`/`key_bar_scope`, the message store), never by
    reading another Context's egui widgets, consistent with the
    logical-model-first rule (screen-snapshot-scrm Requirement 2.2).
27. THIS requirement SHALL introduce NO new command-dispatch path, NO new
    navigation stack, NO new session-persistence format, and NO reshape of a
    public framework type (`WorkspaceContext`, `InteriorFocus`,
    `WorkspaceDescriptor`, `ScreenModel`). The Message_Line is ADDITIVE shell
    state (a new shell field/area), which is NOT a framework-type reshape; it is
    flagged for explicit owner confirmation in design.md.
