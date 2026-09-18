# Tasks -- Workspace Context Framework (CR-NR-078)

Decisions (owner-approved, design.md section 10): trait name `WorkspaceContext`;
Option Y (separate from `ff-layout::DockablePanel`, in ff-desktop, host-agnostic);
Option A (owned-panel swap for the borrow); phase-1 scope = MenuWorkspace (POM +
Settings), Theme Editor, Menus Editor, Config panel; Key Assignment Editor built
on the framework afterwards.

Each migration step MUST keep the existing full-shell first-Tab egui_kittest test
green and leave `verify.ps1` clean (behaviour-preserving, Requirement 1.5 / 3.1).

## Phase 1: Framework + proof migration (owner-approved scope)

- [x] 1. Define the framework types (no migration yet; coexists with the `match`)
  - [x] 1.1 Add `InteriorFocus { first: Option<egui::Id>, last: Option<egui::Id> }` with `none()`, `single(id)`, `new(first, last)` constructors in a new ff-desktop module (e.g. `shell/workspace_context.rs`)
    - Covers: Requirement 1.1, 1.3
  - [x] 1.2 Define the `WorkspaceContext` trait: `fn render(&mut self, ui: &mut egui::Ui, services: &mut ShellServices<'_>) -> InteriorFocus` (host-agnostic -- no assumption of the central panel)
    - Covers: Requirement 1.1, 6.1
  - [x] 1.3 Define `ShellServices<'a>` (config read, runtime handle, notification queue, themes/menus dirs) and `ShellRequest` (Command(String) / Target(CommandTarget) / OpenFile(String) / ...); `ShellServices` carries `&mut Vec<ShellRequest>` the Context enqueues into
    - Covers: Requirement 2.1, 2.2, 2.3
  - [x] 1.4 Add the single trait-dispatch helper on the shell using Option A (owned-panel swap): take the active Context out, build `ShellServices` from the remaining fields, call `render`, drain `ShellRequest`s through the existing pipelines (`handle_command` / `dispatch_command_target` / `file.open`), put the Context back, then feed the returned `InteriorFocus` to `honour_interior_focus_latch`
    - Covers: Requirement 1.2, 2.2
  - [x] 1.5 Unit tests: `InteriorFocus::none/single/new`; the dispatch drains a queued `ShellRequest` through the right pipeline; a Context returning `InteriorFocus::none()` compiles and is honoured
    - Validates: Requirement 1.1, 1.3, 2.2

- [x] 2. Migrate the Config panel (simplest custom panel first)
  - [x] 2.1 Implement `WorkspaceContext` for the Config panel; its `render` calls the existing `config_panel::render` and returns `InteriorFocus::single(filter_field_id())`; route its effects via `ShellRequest`
    - Covers: Requirement 1.4, 1.5
  - [x] 2.2 Switch the `TabKind::ConfigPanel` arm in `render.rs` to the trait-dispatch helper; remove that arm's inline focus-latch ritual
    - Covers: Requirement 3.1, 3.3
  - [x] 2.3 Confirm `full_shell_config_first_tab_focuses_filter_field` passes unchanged; verify.ps1 clean
    - Validates: Requirement 1.5, 3.2

- [x] 3. Migrate the Theme Editor
  - [x] 3.1 Implement `WorkspaceContext` for the Theme Editor; `render` wraps `theme_editor_panel::render`, returns `InteriorFocus { first = selector combo id, last = last hex field id }`, routes `ThemeEditorAction` via `ShellRequest`/existing apply path
    - Covers: Requirement 1.4, 1.5
  - [x] 3.2 Switch the `TabKind::ThemeEditor` arm to trait dispatch; remove its inline ritual
    - Covers: Requirement 3.1, 3.3
  - [x] 3.3 Confirm `full_shell_theme_editor_first_tab_focuses_theme_selector` passes unchanged; verify.ps1 clean
    - Validates: Requirement 1.5, 3.2

- [x] 4. Migrate the Menus Editor
  - [x] 4.1 Implement `WorkspaceContext` for the Menus Editor; `render` wraps `menus_editor_panel::render`, returns `InteriorFocus { first = menu selector combo id, last = save-as name field id }`, routes `MenusEditorAction`
    - Covers: Requirement 1.4, 1.5
  - [x] 4.2 Switch the `TabKind::MenusEditor` arm to trait dispatch; remove its inline ritual
    - Covers: Requirement 3.1, 3.3
  - [x] 4.3 Confirm the Menus Editor first-Tab test passes unchanged; verify.ps1 clean
    - Validates: Requirement 1.5, 3.2

- [x] 5. Migrate the MenuWorkspace (POM + Settings + user menus -- one implementor)
  - [x] 5.1 Implement `WorkspaceContext` for the MenuWorkspace Context; `render` wraps `render_menu_workspace`, returns `InteriorFocus { first, last }` from its result, enqueues `selected` / `calendar_nav` as requests. Preserve POM tab identity, `[POM]` / `[SETTINGS]` titles, and Legacy chrome
    - Note: the MenuWorkspace routes its `InteriorFocus` through the shared `apply_interior_focus` helper (not the `ShellServices`-only trait) because its render carries menu-specific calendar inputs/outputs -- accepted, documented variance; same `InteriorFocus` type + same single latch path.
    - Covers: Requirement 1.4, 1.5
  - [x] 5.2 Switch the `TabKind::PrimaryOptionMenu` AND `TabKind::MenuWorkspace` arms to trait dispatch; remove their inline rituals
    - Covers: Requirement 3.1, 3.3
  - [x] 5.3 Confirm the POM/menu-workspace first-Tab + Settings tests pass unchanged; verify.ps1 clean
    - Validates: Requirement 1.5, 3.2

- [x] 6. Reconcile with DockablePanel + host-agnostic proof
  - [x] 6.1 Add a doc note (design.md + code) stating `WorkspaceContext` is layered above `ff-layout::DockablePanel` (Option Y); a Context MAY impl both; `ff-layout` stays GUI-independent
    - Covers: Requirement 4.1, 4.2
  - [x] 6.2 Add a unit/harness test rendering a migrated Context into a NON-central-panel `Ui` (a plain egui area) to prove `render` is host-agnostic (detach-ready), without building the detach feature
    - Test: `shell::tests::workspace_context_render_is_host_agnostic`
    - Validates: Requirement 6.1, 6.2

- [x] 7. Close out phase 1
  - [x] 7.1 Update `.kiro/steering/workspace-conformance.md`: for MIGRATED Contexts the trait's `InteriorFocus` return is the enforcement mechanism; unmigrated arms still follow the manual rule until their later phase
    - Covers: Requirement 3.3
  - [x] 7.2 Update `docs/quality/TCR.md`: set the CR-NR-078 phase-1 rows to their status
    - Covers: Requirement 1-6 (phase-1 criteria)
  - [x] 7.3 verify.ps1 CLEAN (FULL, nextest); rebuild ffwb.exe; change-log CR-NR-078 phase-1 -> DONE

## Later phase (NOT phase 1 -- scheduled after the proof)

- [ ] 8. Migrate the remaining Contexts to `WorkspaceContext`
  - [ ] 8.1 Plugin Manager, Event Log, Macro Library, Search Results, Command Configurator (each already reports a first interior post-B059)
    - Covers: Requirement 1.4
  - [ ] 8.2 Special cases: FilesPanel (reconcile B024 catalog-focus with `InteriorFocus`) and FileEditor (editor body id or documented `none()`), each with a dedicated full-shell test
    - Covers: Requirement 3.4
  - [ ] 8.3 Remove the `match kind` render arms entirely (full trait dispatch); update the steering rule to cite the trait as the sole enforcement mechanism
    - Covers: Requirement 3.3
