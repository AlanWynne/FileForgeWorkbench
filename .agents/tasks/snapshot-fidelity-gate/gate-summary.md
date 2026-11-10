# Gate Summary -- Full-window snapshot FIDELITY (CR-CH-115)

Requirements gate for FULL screen-snapshot/capture fidelity: the box-drawn
snapshot must faithfully mirror the WHOLE framed window for every screen, and the
same fidelity must extend to the capture path. Docs only -- no `.rs` changed, no
cargo gate run. Work done in the worktree `.worktrees/snapshot-fidelity-gate` on
branch `snapshot-fidelity-gate`.

## New CR id

**CR-CH-115** (next free id; the change-log max across CR-CH / CR-NR was 114 --
CR-NR-114 -- and the highest CR-CH was 079). Logged in
`docs/status/change-log.md`.

## New acceptance criteria

`docs/specs/screen-snapshot-scrm/requirements.md` NEW **Requirement 29**
(CR-CH-115), criteria **29.1-29.17**, grouped:

- Top chrome (always): 29.1 window title bar (`# | FileForge Workbench ... _ [] X`),
  29.2 the active `[XXX]` tab tag, 29.3 the workspace title (once, no duplicate),
  29.4 the `Command ===>` line NEAR THE TOP under the title (corrects the current
  bottom/outside placement) with its typed value + underscore input area.
- Body grid: 29.5 `|`-separated column header + rows, 29.6 fixed-width full-width
  cells, 29.7 always-present empty SCRATCH row, 29.8 combo cell `[value v]`
  (empty `[ v]`), 29.9 empty editable `____` / filled padded value -- IDENTICAL
  in plain-text and HTML flavors.
- Bottom chrome: 29.10 information/status bar, 29.11 F-key bar WHEN VISIBLE
  (omitted when hidden).
- General + shared: 29.12 GENERAL to all Contexts via a shared helper + shared
  renderer, 29.13 built purely from logical state (no egui), 29.14 the SAME
  fidelity on the CAPTURE path (shared seam, no divergent renderer), 29.15
  reconcile the POM/menu path (re-add title bar + `[POM]` tag), 29.16 PNG + table
  flavors unchanged, 29.17 feeds the workspace-architecture alignment.

Design: NEW `design.md` Section 12 (investigation + chosen design + framework
verdict + testability + workspace-architecture feed) and a traceability row.
Tasks: NEW `tasks.md` Phase 2G (TDD-first, tasks 2G.1-2G.6).

## Investigation verdicts (settled against the code, NOT assumed)

- **Render location + WHY thin**: the box is rendered by
  `ff-screen-model/src/render/text.rs::render_plain` from one `ScreenModel`. The
  per-Context builders in `crates/ff-desktop/src/shell/screen_providers.rs`
  (Alias, Theme, Menus, Keys, Kinds, CommandConfigurator, SearchResults, ...)
  populate ONLY `model.fields` (producing the `SS . . . : SNAPSHOT` dot-leader
  lines the owner saw) + optionally ONE `model.tables` entry +
  `model.command_line = command_text`. They set NO `header_lines` (no window
  title bar / no `[XXX]` tab tag) and NO `footer_lines` (no info bar / no F-key
  bar), and leave the command line in `model.command_line`, which `render_plain`
  emits OUTSIDE/below the frame (the owner-observed `Command ===> ss` under the
  box). The `Table` renderer uses `headers.join("  ")` / `row.join("  ")` -- no
  `|`, no fixed width, no full width, no scratch row, no `[value v]`/`____`
  convention. The POM/menu builder
  (`screen_snapshot.rs::menu_workspace_screen_model_full` +
  `screen_provider.rs::menu_screen_chrome`) is the one rich builder, but the
  2026-11-01 Bug-2 correction deliberately REMOVED its window title bar + `[POM]`
  tag.
- **Chrome / F-key sources**: window title = static `"FileForge Workbench"` set
  in `main.rs` (`ViewportBuilder::with_title` + `run_native` app name); tab tag
  `[XXX]` = `workspace_kind::BuiltinKind::default_title()` reached via
  `BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)` (Alias -> `[ALIASES]`);
  title line = `screen_providers::active_display_title`; command value =
  `self.command_text`; info/status bar = live status -> `model.status_bar.text`;
  F-key visibility = `self.key_bar_visible` + `self.key_bar_scope` (exactly the
  state `menu_screen_chrome` already reads to decide the PF-key footer, reusing
  `screen_snapshot::key_bar_line`).
- **Snapshot vs capture path**: ONE shared path, NOT divergent. `SNAPSHOT`
  (`commands_snapshot.rs`) and `CAPTURE SCREEN` / auto-capture
  (`commands_scrm.rs::auto_capture_active_context`) both obtain the model from
  the SAME seam `active_screen_capture()` -- capture via the thin `into_model()`
  wrapper `active_screen_model()` -- and render through the same `ff-screen-model`
  renderer. There is NO separate capture renderer. So fidelity added to the
  shared per-Context builders + the shared renderer flows to BOTH paths
  automatically (Req 29.14). Capture is "recording / multi-frame" only in that it
  appends successive models to a Collection; each model comes from the identical
  provider seam.
- **General vs specific decision**: GENERAL (strongly preferred, per the prompt
  and `framework-conformance.md` reusability). Implemented in the shared
  `ff-screen-model` grid renderer + a shared `ff-desktop` chrome helper called
  from every per-Context `*_screen_model()` builder. Extends the former POM-only
  Req 3.8-3.13 to every Context.
- **Framework verdict**: NO framework change. Builds ON the existing
  `ScreenProvider` seam, `ScreenModel` text renderer, and per-Context builders.
  The `ScreenModel` already carries `header_lines`/`body_lines`/`footer_lines`/
  `tables`/`command_line`/`status_bar`, so no public type is reshaped. The
  combo/text cell distinction is carried as provider-owned CELL TEXT
  (`[value v]` / `____`), keeping the `Table` public shape unchanged (a `Table`
  `CellKind` enum was considered and REJECTED precisely to avoid a public
  reshape). No new dispatcher, navigation stack, persistence format, or clipboard
  flavor. The additive changes: a shared chrome helper, a grid-aware
  `render/text.rs` table rendering, and each builder calling the helper.

## Combo-cell recommendation (owner question)

The owner's `[---- ]v` was a sketch; the recommended canonical convention is
`[value v]` -- square brackets signal a bounded selection control, the trailing
ASCII `v` is the dropdown affordance (ASCII so it survives a plain-text paste and
the ASCII-fallback frame), and an empty combo is `[ v]` so an un-selected
dropdown still reads as a dropdown (distinct from an empty editable field `____`).
This distinguishes the three cell kinds -- dropdown / empty text / filled text --
in monospace, identically in the plain-text and HTML flavors (Req 29.8, 29.9).

## Points needing explicit owner decision (SETTLED-BUT-TUNABLE where noted)

These are confirmable at implementation, not blockers; the requirements now mark
items 1 and 4 SETTLED-BUT-TUNABLE in the EARS text (as Req 25.3 does), per
design-review Finding 7.

1. **Title-bar glyphs / exact format (SETTLED-BUT-TUNABLE, Req 29.1).** Req 29.1
   fixes the form `# | FileForge Workbench                      _ [] X` (app name
   + ASCII `_ [] X` control glyphs), inside the frame border. The leading `# |`,
   exact spacing/width, and glyph set follow the owner's mock and are marked
   settled-but-tunable in 29.1 -- confirm the literal set.
2. **Bug-2 supersession (Req 29.15).** Req 29.15 RE-ADDS the window title bar +
   `[POM]` tab tag to the POM/menu snapshot, deliberately superseding the
   2026-11-01 Bug-2 "no header block" correction FOR THOSE TWO LINES. Confirm this
   reversal (the menu's two-column body + top command line + PF footer are kept).
3. **Combo convention (Req 29.8/29.9).** Confirm `[value v]` / `[ v]` / `____` as
   the canonical cell conventions (owner invited a recommendation; these are the
   agreed values pending explicit sign-off).
4. **Info-bar content (SETTLED-BUT-TUNABLE, Req 29.10).** Req 29.10 places the
   information/status bar at the bottom from the live status text
   (`model.status_bar.text`); the exact status source to mirror is marked
   settled-but-tunable in 29.10 -- confirm it is the shell's current status line.

All four are recorded in the requirements/design; none blocks authoring. No
framework change is implied by any of them.

## Design-review revision 1 (CHANGES_REQUESTED -> addressed)

The design-review (`.agents/tasks/snapshot-fidelity-gate/design-review.json`,
0 HIGH / 4 MEDIUM / 3 NIT) was addressed in full; see the "Design-review
responses" subsection at the end of `design.md` Section 12. Summary:

- F1 (HTML-flavor identity + scratch row): scratch row is now a PROVIDER-OWNED
  `model.tables` row so the HTML `<td>` emitter carries it and the `[value v]` /
  `____` strings identically; added ff-screen-model HTML test (task 2G.1.6) and
  full-shell HTML-flavor assertion (task 2G.4.3).
- F2 (begins-with-title-bar vs renderer order): title bar + tab tag are now the
  FIRST TWO `body_lines` (verified `render_plain` order); 29.1/29.2 reworded;
  design "Renderer-order reconciliation" added; task 2G.2.1 asserts first/second
  body row.
- F3 (full frame width undefined): NEW Req 29.6a sources the frame width from
  `model.dimensions.columns` (verified existing field); task 2G.1.1a asserts a
  narrow grid pads to the frame width.
- F4 (stale Wave F tasks): tasks 16.1/16.3/17.2 updated to real-window-screengrab
  + composite-no-picker; Req 9.14 revised to a real window screenshot to align
  with Req 1.3 / 23.2.
- F5 (command-line underscore width): Req 29.4 = POM/menu field width.
- F6 (overflow): Req 29.6/29.8/29.9 widen-on-overflow, no truncation.
- F7 (settled vs open): Req 29.1 and 29.10 marked SETTLED-BUT-TUNABLE.

GATE_READY_FOR_APPROVAL
