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
