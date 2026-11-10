# Gate summary -- The Common Workspace Frame (CR-NR-115)

Requirements + design gate, DOCS ONLY. No `.rs` changed; no cargo gate run. This
gate is the explicit FIRST step of the workspace-architecture alignment the owner
asked for before building the next major extensions: get the frame COMMON to
every workspace right first, then one follow-up gate per workspace kind.

## CR id

CR-NR-115 (next free id; CR-CH-115 and CR-NR-114 were the previous highest).
Logged in `docs/status/change-log.md` under New Requirements.

## Spec files created/edited (all under the worktree)

- `docs/specs/workspace-framework/requirements.md` -- NEW Requirement 7 (the
  common-frame contract + the common-vs-unique boundary; the BACKBONE), criteria
  7.1-7.27.
- `docs/specs/workspace-framework/design.md` -- NEW Section 11 (investigation
  verdict, the boundary table, element data sources, the Message_Line mechanism,
  last-action time, framework verdict, follow-up gates).
- `docs/specs/workspace-framework/tasks.md` -- NEW Phase CF (TDD-first tasks for
  the common frame + its additive shell state).
- `docs/specs/screen-snapshot-scrm/requirements.md` -- NEW Requirement 30 (how the
  snapshot/capture REPRODUCES the common frame), criteria 30.1-30.17 + numbering
  note.
- `docs/specs/screen-snapshot-scrm/design.md` -- NEW Section 13 (reproduction
  design + a rendered-frame fixture).
- `docs/specs/screen-snapshot-scrm/tasks.md` -- NEW Phase 2H (reproduction tasks).
- `docs/specs/menu-and-statusbar/requirements.md` -- common-frame cross-reference
  note under Requirement 5 (status bar / message line / PF-key mechanics it owns).
- `docs/status/change-log.md` -- CR-NR-115 entry.

## The common-frame criteria ids

- Backbone (workspace-framework Req 7): 7.1-7.10 (the eight elements + single
  chrome), 7.11-7.15 (Message_Line transient/multi-message/cleared-on-next-action,
  replaces RUNNING), 7.16-7.21 (Status_Bar common segments + per-Kind segment,
  `Last:` + `Prof:`, no RUNNING, per-Kind content deferred), 7.22 (pfshow-driven
  Function_Key_Bar), 7.23-7.25 (the common-vs-unique boundary, single
  representation, per-Kind follow-ups), 7.26-7.27 (logical-state-only; no
  framework change; Message_Line additive state flagged).
- Reproduction (screen-snapshot-scrm Req 30): 30.1-30.7 (eight elements in the
  snapshot, incl. menu bar added + full tab bar revising 29.2), 30.8-30.9 (message
  line captured when present, omitted when empty), 30.10 (common+per-Kind status,
  no RUNNING), 30.11 (pfshow key bar), 30.12-30.14 (box-drawing content grid,
  cell conventions, fixed-width/scratch/overflow), 30.15-30.17 (general, capture
  parity, no framework change).

## The common-vs-unique boundary (central deliverable)

SHELL-OWNED (Common_Frame, rendered exactly once around every workspace):
1. Window_Title_Bar  2. Menu_Bar  3. Tab_Bar (ALL tabs + tags + close)
4. Workspace_Title (centered)  5. Primary_Command_Field (`Command ===>`)
6. Message_Line (transient, above the status bar)  7. Status_Bar (common segments)
8. Function_Key_Bar (when `pfshow` on).

WORKSPACE / COMMAND-ENVIRONMENT-OWNED:
- The Content_Area between the command field and the message line (opaque to the
  frame; the workspace fills it).
- ONE Workspace_Status_Segment the active Kind contributes to the status bar.

Nothing else of the frame is workspace-owned. Re-drawing a frame element inside
the content area (duplicate title; a dot-leader fields block repeating grid data)
violates the single-representation principle.

## Investigation verdicts

- ELEMENT SOURCES (settled against code): window title = constant in `main.rs`;
  menu bar = data-driven panel / menu-TOML echo; tab bar = `TabManager` +
  `BuiltinKind::default_title()`; workspace title = `mod_helpers::title_line_text`
  -> `active_display_title`; command field = `self.command_text`; status bar =
  `shell/render_status.rs::render_status_bar` (phase/started/profile/Ln-Col/
  encoding/zoom/line-count); function-key bar = `render_status.rs::render_key_
  label_bar`.
- PFSHOW: `self.key_bar_visible` + `self.key_bar_scope`, driven by the `PFSHOW`
  command (`shell/commands_pfshow.rs`); the key bar early-returns when hidden.
- MESSAGE-LINE MECHANISM: NO transient message-line area today. `open_error` is a
  single-slot error inside the status bar; `NotificationQueue`
  (`notification/mod.rs`) is the persistent Event Log store. CHOSEN: ADDITIVE
  shell state (a `message_line` store + a new bottom panel above the status bar),
  cleared at the per-action dispatch seam. ADDITIVE shell state, NOT a
  framework-type reshape.
- TAB-BAR AVAILABILITY: the full tab bar (all open tabs + tags + close) IS
  available to the snapshot model from `TabManager` + `BuiltinKind`; reproducing
  it is a composition change in the shared chrome helper, no model reshape.
- FRAMEWORK VERDICT: BUILDS ON existing seams. No new command-dispatch path, no
  new navigation stack, no new persistence format, no reshape of a public type
  (`WorkspaceContext` / `InteriorFocus` / `WorkspaceDescriptor` / `ScreenModel` /
  `Table`). Status-bar segment changes are content within the existing
  `render_status_bar`. The Message_Line and `last_action_at` are ADDITIVE shell
  state.

## Explicit owner-decision point

The ADDITIVE Message_Line + `last_action_at` shell state (new fields + one new
bottom panel above the status bar). This is NEW shell state and a NEW user-visible
line -- additive, NOT a framework-type reshape, so it does not need owner
framework confirmation as a type change, but its BEHAVIOUR (transient,
multi-message, cleared on next action, captured in a snapshot when present) and
the removal of the `RUNNING` segment are flagged here for explicit owner
confirmation.

## Follow-up per-Workspace-Kind gates implied (deferred, one per Kind)

Each builds ON the CR-NR-115 common-frame contract and defines that Kind's
Content_Area content + its Workspace_Status_Segment:
- Alias Kind (incl. the D2 fix: provider stops emitting the dot-leader fields
  block; D5 fix: grid shows the Parameters value).
- Menu (POM/Settings/user menus) Kind.
- Workspace Kinds (Kinds Editor) Kind.
- Config Kind.
- Theme Editor Kind.
- Every other Workspace Kind (Keys, Command Configurator, Macro Library, Event
  Log, Plugin/Extension Manager, Search Results, Hex, Database, ...).

GATE_READY_FOR_APPROVAL
