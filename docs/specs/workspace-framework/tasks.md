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

## Later phase (WF.6 -- scheduled after the proof) -- DONE

- [x] 8. Migrate the remaining Contexts to `WorkspaceContext`
  - [x] 8.1 Plugin Manager, Event Log, Macro Library, Search Results, Command Configurator (each already reported a first interior post-B059)
    - Each now `impl WorkspaceContext` and is dispatched via `render_workspace_context`
      (owned-panel swap). Action-returning panels (Macro Library, Command
      Configurator, Search Results) stash their action/outcome on a
      `pending_*` field the shell drains after put-back; Event Log applies its
      Clear-Log request in place via `services.notifications`; Plugin Manager is
      action-free. `ShellServices` gained a read-only `command_store` for the
      Command Configurator. Behaviour-preserving: the existing
      `assert_first_tab_lands_on_reported_interior` tests remain green.
    - Covers: Requirement 1.4
  - [x] 8.2 Special cases: FilesPanel and FileEditor
    - Both are documented no-interior cases (workspace-conformance exception 2):
      the Editor Context renders the active `TabState` (shell-entangled inputs,
      native body focus) and the Files Panel has its own command field + bespoke
      `files_panel_cmd` -> tree Tab redirect (B024/Req 20.1). Each arm reports
      `InteriorFocus::none()` EXPLICITLY through the single `apply_interior_focus`
      latch path. New full-shell tests: `full_shell_file_editor_reports_no_interior_focus`,
      `full_shell_files_panel_reports_no_interior_focus`.
    - Covers: Requirement 3.4
  - [x] 8.3 Single latch path for every arm; steering rule updated
    - The `match kind` arms are NOT deleted: MenuWorkspace, FileEditor and
      FilesPanel legitimately cannot fit the `ShellServices`-only trait (extra
      inputs / entangled state), so they route their focus contract through the
      shared `apply_interior_focus` helper -- the SAME single latch path used by
      the trait dispatch. Every arm therefore either implements `WorkspaceContext`
      (dispatched via `render_workspace_context`) or calls `apply_interior_focus`
      with an explicit `InteriorFocus` (including the documented `none()` cases),
      so no arm can silently leave the anchors unset. Steering rule
      `workspace-conformance.md` updated to record the completed migration set.
    - Covers: Requirement 3.3

## Phase CF: The Common Workspace Frame (CR-NR-115, Requirement 7)

TDD-first. Every implementation task is preceded by a failing test. These tasks
deliver the COMMON frame + its additive shell state; the snapshot REPRODUCTION of
the frame is tasked in `screen-snapshot-scrm` tasks.md (Phase 2H), and the
per-Workspace-Kind content/status segments are FOLLOW-UP gates (not tasked here).
Agents compile-verify (`cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`,
`cargo fmt -p ff-desktop -- --check`) and run only the SCOPED new unit tests; the
owner runs the full gate (gate-policy.md).

- [ ] CF.1 Message_Line additive shell state (transient, multi-message, cleared on next action)
  - [ ] CF.1.1 Write failing unit tests for a `message_line` store on the shell: setting one or more messages makes them readable; dispatching an action CLEARS them before the action runs; with no message the store is empty (Req 7.11-7.14)
    - Validates: Requirement 7.11, 7.12, 7.13, 7.14
  - [ ] CF.1.2 Add the additive `message_line` shell state (e.g. `Vec<String>`) and the clear-on-dispatch hook at the shell's per-action dispatch seam; make CF.1.1 pass
    - Covers: Requirement 7.11, 7.12, 7.13, 7.14, 7.27
  - [ ] CF.1.3 Render the Message_Line as a new `TopBottomPanel::bottom` panel directly ABOVE the Status_Bar panel; it is absent when empty and may expand for multiple messages
    - Covers: Requirement 7.8, 7.13, 7.14

- [ ] CF.2 Remove the `RUNNING` phase segment from the Status_Bar (superseded by the Message_Line)
  - [ ] CF.2.1 Write a failing test asserting the composed Status_Bar does NOT contain the `RUNNING` phase segment
    - Validates: Requirement 7.15, 7.20
  - [ ] CF.2.2 Remove the phase-label segment from `render_status_bar`; make CF.2.1 pass
    - Covers: Requirement 7.15, 7.20

- [ ] CF.3 Last-action time Status_Bar segment (`Last: HH:MM`, replacing `Started`)
  - [ ] CF.3.1 Write a failing unit test for a `last_action_at` timestamp updated on each action and formatted `Last: HH:MM` (mirroring the existing `format_session_start` test)
    - Validates: Requirement 7.16, 7.18
  - [ ] CF.3.2 Add the additive `last_action_at` shell state, update it at the per-action dispatch seam, and replace the `Started: HH:MM` segment with `Last: HH:MM`; make CF.3.1 pass
    - Covers: Requirement 7.16, 7.18, 7.27

- [ ] CF.4 Shorten the profile segment to `Prof: <ActiveProfileName>`
  - [ ] CF.4.1 Write a failing test asserting `active_profile_label()` (or its successor) yields `Prof: <name>` / `Prof: default`
    - Validates: Requirement 7.19
  - [ ] CF.4.2 Shorten the label from `Profile: <name>` to `Prof: <name>`; make CF.4.1 pass
    - Covers: Requirement 7.19

- [ ] CF.5 Workspace_Status_Segment hook (existence only; content per-Kind follow-up)
  - [ ] CF.5.1 Write a failing test proving the Status_Bar composition includes a slot for a Kind-contributed segment alongside the common segments (a stub/empty segment is acceptable; content is per-Kind follow-up)
    - Validates: Requirement 7.17, 7.21
  - [ ] CF.5.2 Add the composition point for a Kind-owned Workspace_Status_Segment in `render_status_bar` (empty by default; the per-Kind gate fills it). If this needs an additive Kind-registry method, STOP and surface it to the owner per design.md
    - Covers: Requirement 7.17, 7.21, 7.27

- [ ] CF.6 Common-frame single-chrome regression guard
  - [ ] CF.6.1 Write a full-shell egui_kittest test asserting the eight common-frame elements render once (window title/menu/tab/title/command line present; Status_Bar + Message_Line distinct lines; key bar per `pfshow`) and no active Workspace re-draws a frame element in its Content_Area
    - Validates: Requirement 7.1, 7.5, 7.6, 7.22, 7.23, 7.24
  - [ ] CF.6.2 Confirm the existing full-shell first-Tab tests remain green (behaviour-preserving around the new Message_Line panel)
    - Validates: Requirement 7.1, 7.27

- [x] CF.7 Common-frame follow-up: Zoom segment, message routing, capture timing (2026-11-04, branch common-frame-fixes)
  - [x] CF.7.1 Zoom common segment: one `format_zoom_label()` for the live bar and the snapshot band, always shown after Prof, hover tooltip only (tests_zoom_segment.rs)
    - Validates: Requirement 7.16, 7.16b
    - Test status: automated PASS pending owner full gate. The hover tooltip TEXT is MANUAL (kittest cannot read hover popups headlessly); the not-a-Tab-stop assertion is automated.
  - [x] CF.7.2 Route informational messages to the Message_Line via `post_info`, errors stay on `open_error` (tests_message_routing.rs)
    - Validates: Requirement 7.28
  - [x] CF.7.3 Capture the pre-action Message_Line and `Last:` through the one capture seam (tests_capture_timing.rs)
    - Validates: Requirement 7.29

- [x] CF.8 Content_Area top-boundary rule on the live screen (owner Option 1, 2026-11-04)
  - [x] CF.8.1 Write a failing full-shell egui_kittest test asserting a frame-owned rule is drawn between the `Command ===>` line and the Content_Area for at least two Kinds (a menu Kind and a grid Kind) and that the first-Tab focus order is unchanged
    - Validates: Requirement 7.30, 7.31
  - [x] CF.8.2 Draw the rule in the shell central-panel composition (not in any Workspace) and keep the snapshot rule in step (see screen-snapshot-scrm 2H.8)
    - Covers: Requirement 7.30, 7.31, 7.32
