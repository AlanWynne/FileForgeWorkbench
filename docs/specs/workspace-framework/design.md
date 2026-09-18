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
