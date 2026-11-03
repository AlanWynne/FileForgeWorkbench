# Design Document -- Workspace Context Framework (CR-NR-078)

DESIGN PROPOSAL for owner review. No framework code is written until the
requirements gate is approved. This document sketches the trait, the borrow
-checker approach, the migration plan, and the reconciliation with
`ff-layout::DockablePanel`.

## 1. The problem this removes

`crates/ff-desktop/src/shell/render.rs` renders the active Workspace_Context via:

```rust
self.first_interior_id = None;   // reset each frame
self.last_interior_id  = None;
match self.tabs.active_tab().kind {
    TabKind::ConfigPanel => {
        config_panel::render(ui, &mut self.config_panel, &self.config_handle);
        // EACH arm must remember this ritual, or a phantom Tab stop appears:
        let id = config_panel::filter_field_id();
        self.first_interior_id = Some(id);
        self.last_interior_id  = Some(id);
        self.honour_interior_focus_latch(ctx, Some(id), Some(id));
    }
    // ... one arm per kind, each repeating the ritual ...
}
```

The ritual is opt-in per arm. B056/B057/B058/B059 were all "an arm forgot the
ritual." The fix: make the ritual a VALUE the render MUST return, dispatched once.

## 2. The trait (proposed shape)

```rust
/// The interior focus contract a Context reports to the shell Boundary_Policy.
#[derive(Debug, Default, Clone, Copy)]
pub struct InteriorFocus {
    /// First interior Tab stop (command-field -> here). Stable, same-frame id.
    pub first: Option<egui::Id>,
    /// Last interior Tab stop (Shift+Tab reverse boundary anchor).
    pub last: Option<egui::Id>,
}

impl InteriorFocus {
    /// Explicit "this Context has no interior focus stops" (Requirement 1.3).
    pub fn none() -> Self { Self { first: None, last: None } }
    /// A single control that is both first and last (the common case).
    pub fn single(id: egui::Id) -> Self { Self { first: Some(id), last: Some(id) } }
    pub fn new(first: egui::Id, last: egui::Id) -> Self {
        Self { first: Some(first), last: Some(last) }
    }
}

/// Every Workspace_Context implements this. The render MUST return an
/// InteriorFocus -- the compiler will not let an implementor omit it, so the
/// phantom-Tab-stop bug (B056-B059) becomes unrepresentable.
pub trait WorkspaceContext {
    /// Render the Context into `ui`, using `services` to reach shell state, and
    /// RETURN the interior focus contract for this frame.
    fn render(&mut self, ui: &mut egui::Ui, services: &mut ShellServices<'_>)
        -> InteriorFocus;
}
```

The shell dispatch collapses to ONE path (Requirement 1.2):

```rust
self.first_interior_id = None;
self.last_interior_id  = None;
let focus = self.active_context_mut().render(ui, &mut services);  // trait call
self.first_interior_id = focus.first;
self.last_interior_id  = focus.last;
self.honour_interior_focus_latch(ctx, focus.first, focus.last);
```

No per-kind arm can skip the latch, because there is no per-kind arm anymore.

## 3. The borrow-checker problem and ShellServices

Today each arm calls `panel::render(ui, &mut self.some_panel, &self.some_field)`
-- the panel state and the shell fields it reads are DISJOINT borrows, which the
compiler allows. A naive trait `render(&mut self, ui)` on the shell would need
`&mut WorkbenchShell` AND `&mut Ui` (the ui is itself borrowed from the shell's
egui frame), plus the Context is a FIELD of the shell -- so `context.render(ui,
&mut self)` is a double `&mut self` borrow and will not compile.

`ShellServices` solves this by SPLITTING the shell's borrows: the shell hands the
Context (a) `&mut` to ITS OWN state (the specific panel field) and (b) a
`ShellServices` that borrows the OTHER shell fields the Context legitimately
needs -- never `&mut self` wholesale. Sketch:

```rust
/// A per-frame view of the shell services a Context may use, constructed by the
/// shell from its own fields. Does NOT expose &mut WorkbenchShell (Req 2.1).
pub struct ShellServices<'a> {
    pub config: &'a ff_config::ConfigHandle,
    pub runtime: &'a tokio::runtime::Runtime,
    pub notifications: &'a Arc<Mutex<NotificationQueue>>,
    pub themes_dir: PathBuf,
    pub menus_dir: PathBuf,
    /// Deferred requests the Context enqueues instead of mutating the shell
    /// directly (Req 2.2): open a command, dispatch a CommandTarget, open a file.
    pub requests: &'a mut Vec<ShellRequest>,
}

pub enum ShellRequest {
    Command(String),                 // handle_command(cmd)
    Target(ff_command::CommandTarget),
    OpenFile(String),
    // ... the small, closed set of things a Context can ask the shell to do.
}
```

After the trait call returns, the shell drains `requests` and applies them
through the existing pipelines (`handle_command`, `dispatch_command_target`,
`file.open`). This GENERALISES the "pure render -> action" pattern already used
by Theme Editor / Menus Editor / Command Configurator (which return an `*Action`
enum today) -- those become `ShellRequest`s or keep their own action enums drained
by the shell. The Context never holds `ShellServices` across frames (Req 2.3).

Two implementation options for the dispatch borrow (to be settled at build time,
both viable):

- **A. Owned-panel swap:** `std::mem::take` the active panel out of the shell,
  call `render` on the local value with a `ShellServices` borrowing the rest, then
  put it back. Simple, no unsafe; one move per frame (cheap for these panels).
- **B. Field-split accessor:** a method returning `(&mut dyn WorkspaceContext,
  ShellServices<'_>)` via disjoint field borrows (the panels live in distinct
  fields, so the borrow checker can split them). No move, but more plumbing.

Option A is the recommended default (simplest, provably correct). The choice does
not affect the trait's public shape.

## 4. How the two existing patterns satisfy the trait

- **MenuWorkspaceState** (POM / Settings / user menus): already produces a
  `render_menu_workspace(...) -> result { first_interior_id, last_interior_id,
  selected, calendar_nav }`. Its `WorkspaceContext::render` wraps that: return
  `InteriorFocus { first, last }` from the result and enqueue `selected` /
  `calendar_nav` as requests. Near-mechanical.
- **Custom panels** (Config, Theme Editor, Menus Editor, Plugin Manager, Event
  Log, Macro Library, Search Results, Command Configurator): each already has a
  free `render` fn and (post-B057/B058/B059) a known first/last interior id. Its
  `WorkspaceContext::render` calls that fn and returns the `InteriorFocus`. The
  `*Action` enums they already return become `ShellRequest`s (or are drained
  as-is).

This is the Rust answer to the prompt's "menu class vs custom class": they are
not subclasses; they are two independent implementors of one trait (composition).

## 5. The two special cases (Requirement 3.4)

- **FilesPanel:** has bespoke state-driven catalog focus (`focused_catalog`, from
  B024) rather than egui-native widget focus. Its `render` returns an
  `InteriorFocus` whose `first` is the id it wants the command-field Tab to land
  on (e.g. the first catalog row's stable id, or the tree container), reconciling
  the B024 mechanism with the Boundary_Policy. The design must verify the B024
  Tab-transfer still works; a dedicated full-shell test is required.
- **FileEditor / Untitled:** the editor body is one large natively-focusable
  widget. Its `render` returns `InteriorFocus::single(<editor widget id>)` so Tab
  from the command field lands in the editor, OR `InteriorFocus::none()` with a
  documented reason if the editor manages its own caret focus differently. To be
  decided during migration with a test either way.

## 6. Reconciliation with ff-layout::DockablePanel (Requirement 4)

`ff-layout` already defines `DockablePanel { fn render(&mut self, ui: &mut
egui::Ui); ... }` for the docking model. Two options:

- **Option X -- extend DockablePanel:** change its `render` to return
  `InteriorFocus`. Pro: one trait. Con: touches the docking model and every
  DockablePanel impl; `InteriorFocus` (an egui id pair) must live in ff-layout,
  which is meant to stay GUI-independent beyond the render signature.
- **Option Y -- layer WorkspaceContext above DockablePanel (recommended):**
  `WorkspaceContext` is the ff-desktop tab-Context contract; a Context MAY also be
  a `DockablePanel` when docked, but the tab-level focus contract lives in
  ff-desktop where the shell Boundary_Policy also lives. Keeps ff-layout's
  GUI-independence intact (Req 4.2) and scopes the change to ff-desktop.

Recommendation: Option Y. The design will state the relationship explicitly.

## 7. Migration plan (incremental, behaviour-preserving -- Requirement 3)

Order chosen so the simplest, already-conformant Contexts go first (lowest risk),
special cases last:

1. Define `WorkspaceContext`, `InteriorFocus`, `ShellServices`, `ShellRequest`
   and the single dispatch path -- WITHOUT removing the `match` yet (both coexist;
   nothing migrated). Add unit coverage for the dispatch + request draining.
2. Migrate the five B059 panels (Plugin Manager, Event Log, Macro Library, Search
   Results, Command Configurator) -- each already returns a known first interior;
   convert one per commit, keep its full-shell first-Tab test green.
3. Migrate Config, Theme Editor, Menus Editor (they already have first/last ids).
4. Migrate MenuWorkspace (POM / Settings / user menus) -- wrap the existing
   result.
5. Migrate the two special cases (FilesPanel, FileEditor) with dedicated tests.
6. Remove the per-kind focus-latch ritual and (if fully trait-dispatched) the
   `match kind` render arms; update `workspace-conformance.md` to cite the trait
   as the enforcement mechanism.

Each step: `verify.ps1` CLEAN, full-shell tests green, no observable change.

## 8. What this does NOT change

- No visual, layout, or command-behaviour change to any Context (Req 1.5).
- No change to the shell chrome (command line, menu bar, tab bar, status bar) or
  the Boundary_Policy itself -- only HOW Contexts report into it.
- No change to `ff-layout`'s model under Option Y.

## 9. Testing strategy

- Reuse the existing `full_shell_*_first_tab_focuses_interior` tests unchanged in
  meaning (they assert first Tab lands on the reported first interior) -- they
  become the migration's safety net.
- Add a unit test that a Context implementing `WorkspaceContext` and returning
  `InteriorFocus::none()` does so explicitly (compile-level: the method must
  return a value).
- Add dispatch/`ShellRequest`-draining unit tests.
- Per steering `testing.md` + `workspace-conformance.md`: every migrated Context
  keeps its full-shell first-Tab egui_kittest test.

## 10. Decisions (APPROVED by owner)

1. **Naming:** `WorkspaceContext` (trait), `InteriorFocus` (focus return),
   `ShellServices` (access object). APPROVED.
2. **DockablePanel reconciliation: Option Y** -- `WorkspaceContext` lives in
   ff-desktop, layered ABOVE `ff-layout::DockablePanel`; they are orthogonal (one
   = content + focus contract, the other = placement/geometry). APPROVED, with the
   owner's detach rationale making it decisive: a detached Workspace is an OS-level
   egui VIEWPORT of the SAME single FFWB process (not a second process), so the
   content-and-focus contract must be independent of placement -- a detached window
   is not docked yet still needs `InteriorFocus`. Option X (folding focus into
   DockablePanel) would break for detached windows and leak GUI-focus concepts into
   the GUI-independent `ff-layout` crate. Rejected.
3. **Borrow approach: Option A** (owned-panel swap: `mem::take` the active panel,
   render it as a local with `ShellServices` borrowing the rest, put it back).
   APPROVED (simplest, provably correct; does not affect the trait's public shape).
4. **Phase-1 scope (prove the pattern):** migrate the Contexts behind these five
   screens -- the POM and the Settings menu (BOTH the shared `MenuWorkspace`
   implementor), the Theme Editor, the Menus Editor, and the Config panel. i.e.
   FOUR implementors (MenuWorkspace + ThemeEditor + MenusEditor + ConfigPanel). The
   remaining panels (Plugin Manager, Event Log, Macro Library, Search Results,
   Command Configurator) and the two special cases (FilesPanel, FileEditor) migrate
   in a LATER phase. APPROVED.
5. **Sequencing:** build the framework FIRST; the Key Assignment Editor is authored
   ON the framework afterwards. APPROVED.

### Host-agnostic constraint (from the detach rationale)

`WorkspaceContext::render(&mut self, ui: &mut egui::Ui, services: &mut
ShellServices) -> InteriorFocus` MUST be HOST-AGNOSTIC: it takes a `&mut Ui` and
returns its focus contract, and MUST NOT assume it is drawn in the main window's
central panel. The shell chooses the draw target each frame -- central panel (tab),
a dock zone (future `ff-layout`), or a detached OS viewport (future "Detached
Workspace", already sketched in `layout-and-docking`). Because a Context's render
is written once and reused in all three hosts, detach becomes "call the same
`render` into a viewport's `Ui`" with NO per-Context change -- and every detached
window's own command-line/Boundary_Policy reuses the same `InteriorFocus`. This
constraint is a hard requirement of the framework so detach is a clean future
extension, not a rework (see requirements.md Requirement 6).

---

## 11. The Common Workspace Frame -- common-vs-unique boundary (Requirement 7, CR-NR-115)

### Overview

This section defines, NORMATIVELY and grounded in the live code, the boundary
between what the SHELL owns (the Common_Frame) and what each Workspace / Command
Environment owns (the Content_Area and its own status segment). It is the
backbone of the workspace-architecture alignment the owner asked for before
building the next major extensions. The snapshot/capture REPRODUCTION of this
frame is designed in `screen-snapshot-scrm` design.md Section 13 (which revises
Section 12); the Status_Bar and Function_Key_Bar RENDERING detail is owned by
`menu-and-statusbar`. This section is the authoritative statement of the boundary
itself. The central deliverable of this gate is the boundary table below.

### Investigation verdict -- where each frame element is rendered TODAY (settled against code, NOT assumed)

Read: `crates/ff-desktop/src/main.rs`,
`crates/ff-desktop/src/shell/render_status.rs`,
`crates/ff-desktop/src/shell/render_command_line.rs`,
`crates/ff-desktop/src/shell/render_chrome.rs`,
`crates/ff-desktop/src/shell/render_body.rs`,
`crates/ff-desktop/src/shell/mod_helpers.rs`,
`crates/ff-desktop/src/shell/actions.rs`,
`crates/ff-desktop/src/shell/commands_pfshow.rs`,
`crates/ff-desktop/src/notification/mod.rs`,
`crates/ff-desktop/src/screen_snapshot.rs`,
`crates/ff-screen-model/src/model.rs`.

| Frame element | Rendered today by | Live source of its data |
|---------------|-------------------|-------------------------|
| Window_Title_Bar | The OS window frame; egui `ViewportBuilder::with_title("FileForge Workbench")` + `run_native("FileForge Workbench", ...)` in `main.rs`. The in-frame title-bar LINE (for a snapshot) is composed from this constant + the `_ [] X` glyphs. | Constant app name in `main.rs`; brand token untranslated. |
| Menu_Bar | The data-driven menu bar panel (`shell/render_chrome.rs` for detached; the Primary_Window menu bar panel), content from the POM/SCRM menu TOML echo (CR-NR-114). | Menu TOML (runtime echo); e.g. `Settings | Help | SCRM`. |
| Tab_Bar | The tab bar panel over `TabManager`; each tab carries its `[XXX]` Tab_Header tag and a close affordance. | `TabManager` tabs; `BuiltinKind::default_title()` / `from_tab_kind(...)` gives the `[XXX]` tag per Kind. |
| Workspace_Title | `shell/render_command_line.rs` (the centered, strong monospace Title_Line, CR-CH-045). | `shell/mod_helpers.rs::title_line_text(tab)` -> `screen_providers::active_display_title`. |
| Primary_Command_Field | `shell/render_command_line.rs` (the `Command ===>` field; detached variant too). | `self.command_text` (the live command field buffer). |
| Content_Area | The central panel body, drawn by `render_body.rs::render_active_tab_body` -> `render_workspace_context` (the active `WorkspaceContext`). | Owned by each Workspace / Command Environment. |
| Message_Line | NOT rendered today as a dedicated line. The closest existing mechanism is `self.open_error` (a one-shot error shown inside the Status_Bar) and the `NotificationQueue` (the Event Log store). | NEW shell state (see below). |
| Status_Bar | `shell/render_status.rs::render_status_bar`. Today's segments: phase (`RUNNING`), `format_session_start()` (`Started: HH:MM`), `active_profile_label()` (`Profile: default`), `Ln n, Col n`, encoding, zoom-when-nonzero, line count, modified dot, CAPS, `open_error`. | `self.app.phase()`, `self.session_start`, `ff_session::active_profile()`, `tab.cursor`, `tab.encoding_label()`, `tab.line_count`. |
| Function_Key_Bar | `shell/render_status.rs::render_key_label_bar`; returns early WHEN `!self.key_bar_visible`. | `self.key_bar_visible` + `self.key_bar_scope` (driven by the `PFSHOW` command, `commands_pfshow.rs`). |

Conclusion: SEVEN of the eight common elements are ALREADY shell-owned and
rendered once today. The ONE genuinely new element is the Message_Line.

### The common-vs-unique boundary (THE central deliverable)

SHELL-OWNED (the Common_Frame, rendered exactly once around every Workspace):

1. Window_Title_Bar 2. Menu_Bar 3. Tab_Bar 4. Workspace_Title
5. Primary_Command_Field 6. Message_Line 7. Status_Bar (common segments)
8. Function_Key_Bar (when `pfshow` on).

WORKSPACE / COMMAND-ENVIRONMENT-OWNED:

- The Content_Area between the Primary_Command_Field and the Message_Line.
- ONE Workspace_Status_Segment the active Kind contributes to the Status_Bar.

Nothing else of the frame is workspace-owned. A Workspace that re-draws its own
title or command line inside the Content_Area violates the single-representation
principle (Requirement 7.23, 7.24) -- this is exactly the D1/D2 class from the
owner's snapshot review (duplicate title; a dot-leader fields block that repeats
grid data). The PRINCIPLE is common here; the per-Workspace fix (e.g. the Alias
provider dropping its `model.fields` block) is a follow-up Kind gate.

### Element data sources (where each element's content comes from)

- Window_Title_Bar text: constant `"FileForge Workbench"` from `main.rs` +
  ASCII glyphs `_ [] X`. Brand token NOT translated; other chrome text is
  translatable via `ff_i18n`.
- Tab `[XXX]` tags: `BuiltinKind::default_title()` per open tab.
- Workspace_Title: `title_line_text(tab)` -> `active_display_title`.
- Primary_Command_Field value: `self.command_text`.
- Status_Bar common segments: last-action time (NEW -- a `last_action_at`
  timestamp, see below) formatted `Last: HH:MM`; `Prof: <name>` from
  `ff_session::active_profile()` (shortened label); `Ln/Col` from `tab.cursor`;
  encoding from `tab.encoding_label()`; line count from `tab.line_count`.
- Workspace_Status_Segment: contributed by the active Kind (content defined in
  the per-Kind follow-up gate, not here).
- Function_Key_Bar: `self.key_bar_visible` / `self.key_bar_scope`.

### The Message_Line mechanism (build-on vs additive shell state) -- OWNER-CONFIRMATION POINT

There is NO existing dedicated transient message-line area today. Two existing
mechanisms are near but not a fit:

- `self.open_error` is a single-slot error string shown INSIDE the Status_Bar; it
  is not a multi-message transient line above the bar, and it is error-only.
- The `NotificationQueue` (`notification/mod.rs`) is a persistent 1000-entry store
  feeding the Event Log; it is NOT a transient "lives until the next action" line.

CHOSEN design: add ADDITIVE shell state -- a small `message_line` store on the
shell (e.g. `Vec<String>` plus a flag), rendered by a new shell panel directly
ABOVE the Status_Bar panel. On each dispatched action the shell CLEARS the
message line first, then the action may populate it; so a message lives only
until the next action (Requirement 7.11-7.14). The `RUNNING` phase segment is
removed from the Status_Bar (Requirement 7.15, 7.20). This REUSES the shell's
existing per-action dispatch seam (the point where a command is dispatched is the
natural "next action" boundary) and the existing bottom-panel rendering pattern
(`TopBottomPanel::bottom(...)`, as the Status_Bar and key bar already use).

This is ADDITIVE shell state -- a new field and a new bottom panel -- NOT a
reshape of any public framework type. Per framework-conformance.md it does not
need owner framework confirmation as a TYPE change, but because it is NEW shell
state and a NEW user-visible line it is explicitly FLAGGED here for owner
confirmation of the behaviour (transient, multi-message, cleared on next action,
captured in a snapshot if present). The alternative -- reusing `open_error` and
widening it to a multi-message transient line -- was REJECTED because `open_error`
is error-only and lives in the Status_Bar, not above it; a dedicated
`message_line` keeps the Status_Bar and the Message_Line as distinct lines per the
owner's sketch.

### Last-action time (Status_Bar) -- additive shell state

Today the Status_Bar shows `Started: HH:MM` from `session_start`. The owner picked
LAST-ACTION time (updated on each action, not a ticking clock). CHOSEN design: add
a `last_action_at` timestamp on the shell, set to `chrono::Local::now()` on each
dispatched action, formatted `Last: HH:MM`. This replaces the `Started` segment
(Requirement 7.18). It is additive shell state, no type reshape. `active_profile_
label()` is shortened from `Profile: <name>` to `Prof: <name>` (Requirement 7.19).

### Framework-change verdict (framework-conformance.md)

This BUILDS ON existing seams and needs NO framework change:

- Dispatch (`resolve_target` / `target_dispatch`): UNTOUCHED.
- Navigation (`navigate_to`): UNTOUCHED.
- `WorkspaceContext` / `InteriorFocus` / `WorkspaceDescriptor` / `ScreenModel`
  public shapes: UNCHANGED.
- The Status_Bar segment set changes (remove `RUNNING`, `Started`->`Last:`,
  `Profile:`->`Prof:`, add a Kind-owned segment) -- this is content within the
  existing `render_status_bar`, not a new mechanism.
- The Message_Line and `last_action_at` are ADDITIVE shell STATE (new fields + one
  new bottom panel), explicitly NOT a public-type reshape. FLAGGED for owner
  confirmation as NEW state/behaviour, not as a framework change.

If, during implementation, the Workspace_Status_Segment turns out to need a new
public trait on the Kind registry (e.g. a `status_segment()` method), that would
be an ADDITIVE trait method -- surface it to the owner at that point; it is NOT
assumed by this gate, which defines only that the segment exists and is Kind-owned
(Requirement 7.21).

### Testability

- The eight common-frame elements and the boundary are a documentation contract;
  their RENDERING is tested where each element lives (menu-and-statusbar tests for
  the Status_Bar / key bar; the snapshot reproduction in screen-snapshot-scrm).
- The Message_Line transient behaviour (populated on an action, cleared on the
  next) is unit-testable headlessly on the shell state (set message -> dispatch an
  action -> assert cleared), with NO egui. See tasks.md.
- The `last_action_at` -> `Last: HH:MM` formatting and the `Prof:` label are pure
  functions, unit-testable like the existing `format_session_start` /
  `active_profile_label` tests.
- The Status_Bar no longer showing `RUNNING` and showing the Kind segment is
  assertable via the existing automation registration seam
  (`automation::ids::STATUSBAR_MESSAGE`) and a headless status-composition test.

### Follow-up per-Workspace-Kind gates (recorded)

This gate deliberately covers the COMMON frame only. The per-Kind content of the
Content_Area and the Workspace_Status_Segment is a FOLLOW-UP gate per Kind:
Alias, Menu, Kinds, Config, Theme Editor, and every other Workspace Kind. Each
builds ON this contract. The owner-confirmed sequencing is: common frame first
(this gate), then one gate per Workspace Kind.
