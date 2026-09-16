# RESUME NOTE -- B054 Menus editor typing bug (paused, bedtime)

Last updated: session paused mid-investigation. Pick this up cold from here.

## One-line status

The Menus editor's Command and Description fields cannot be typed into, and (in
the latest uncommitted build) the main command line is ALSO blocked while the
Menus editor is open. Three layout-based fix attempts have FAILED. The bug is a
GLOBAL input/focus problem, not layout. A diagnostic is in place but has not yet
produced data.

## Git state

- Last COMMITTED + PUSHED: `58bf50f` "CR-CH-022: per-tab Navigation_Stack + B054
  (Menus editor typable)". In that commit the editor could type in key/group but
  NOT command/description; command line worked.
- UNCOMMITTED working-tree changes (do NOT discard -- they hold the diagnostic):
  - `crates/ff-desktop/src/menus_editor_panel/render.rs` -- speculative layout
    churn: changed to per-option vertical `ui.group` blocks with full-width
    fields. Owner reports this layout is UGLY and OVERFLOWS the screen with no
    scrollbar (cannot reach the "Add option" button). This layout should likely
    be REVERTED to a compact single-row-per-option once the real cause is known.
  - `crates/ff-desktop/src/shell/render.rs` -- TEMPORARY DIAGNOSTIC in the
    `TabKind::MenusEditor` render arm: logs `[menus-editor-diag] focused=... 
    has_text_event=...` every frame via `ff_logging::log_debug!`. REMOVE this
    before committing the real fix.
- `ffwb.exe` rebuilt 02:18 WITH the diagnostic.

## What is CONFIRMED

1. Layout theories are WRONG (width exhaustion, id collision, Grid): all three
   attempts failed and the last made it worse (command line blocked too, ugly
   overflow).
2. The command line being blocked while the Menus editor is open => GLOBAL
   focus/input steal, not a per-field layout issue.
3. `render_command_field` (shell/render.rs ~L58) is UNCONDITIONAL and not
   suppressed for MenusEditor -- so the command line *should* be typable; if it
   is not, focus is being held elsewhere or text events are consumed upstream.
4. The MenusEditor render arm has the SAME shape as the ThemeEditor arm (which
   works), so the difference is inside `menus_editor_panel::render` OR in some
   global per-frame state that only the Menus editor triggers.
5. SEPARATE REAL BUG found in the logs (not the blocker, fix regardless):
   `%LOCALAPPDATA%\FileForgeWorkbench\logs\*.log` is FLOODED every frame with
   `WARN [ff_config::access] [config] validation: key "theme.active": value is
   not in allowed set -- applying default`. Something wrote an INVALID value
   (probably a theme NAME like "Default Legacy") into the `theme.active` MODE
   key (allowed set: dark/light/high_contrast/legacy). The per-frame theme block
   in `shell/update.rs` (~L360-410) reads `theme.active` / `theme.active_name`
   every frame. This is a B039-class regression -- log it as its own B### and
   fix (stop persisting a name into the mode key; validate before the per-frame
   read). It is noisy but is NOT the typing blocker.

## NEXT STEP (do this first -- get the data, stop guessing)

The diagnostic build (ffwb.exe 02:18) was never run by the owner. Get ONE clean
run:

1. Clear logs, run `.\target\debug\ffwb.exe`.
2. Open Menus editor (type `MENUS` Enter, or POM option `M`).
3. Click the option Command field, type `abc`.
4. Click the main command line, type `xyz`.
5. Close.
6. Read `%LOCALAPPDATA%\FileForgeWorkbench\logs\` newest *.log, grep
   `menus-editor-diag`.

Interpretation:
- `focused=Some(<id>)` + `has_text_event=false` => text events consumed upstream
  (a global `ctx.input_mut` event drain, or the shell Tab/fkey handling). Look
  for any `i.events.retain(...)` / event consumption that runs unconditionally.
- `focused=None` => focus not landing (a per-frame `request_focus` steal, or the
  editor widgets have no interactive area).
- `focused=Some(...)` + `has_text_event=true` but field unchanged => binding bug.

IMPORTANT: in the last owner run the diag lines were ABSENT from the log, which
means either the MenusEditor arm did not execute (the active tab was not
`TabKind::MenusEditor` -- check what `MENUS` actually opened) OR the owner ran an
OLDER exe (the exe was `ffwb.exe` @02:08, before the diag). Confirm the running
exe is the 02:18 build first. If diag lines are STILL absent with the fresh
build, the real bug is that `MENUS` is not landing on `TabKind::MenusEditor` at
all (investigate `open_menus_editor` + `navigate_to`/`reconstruct_context` for
the "editor" marker CustomWorkspace(CommandConfigurator{editor:menus}) round-trip
-- reconstruct_custom maps it to MenusEditor, but the FIRST open goes through
`open_menus_editor` -> `navigate_to(descriptor, push)` -> `reconstruct_context`
which for a CustomWorkspace(CommandConfigurator, editor=menus) sets
TabKind::MenusEditor. Verify that path actually sets MenusEditor and not
CommandConfigurator).

## STRONG HYPOTHESIS to check after the data

The transient-editor descriptor encoding is suspect. In `shell/nav_stack.rs`,
Theme/Menus editors are encoded as `CustomWorkspace { workspace_kind:
CommandConfigurator, params: {editor: "theme"|"menus"} }`. `reconstruct_custom`
routes `editor=menus` -> `set_active_tab_context(TabKind::MenusEditor, "[MENUS]")`.
BUT `open_menus_editor` calls `navigate_to(descriptor, push=true)` which pushes
the CURRENT context then reconstructs. If the CURRENT context descriptor also
resolves oddly, or if reconstruct routes to CommandConfigurator instead of
MenusEditor, the tab kind would be wrong -> the MenusEditor render arm never runs
-> the fields the owner sees belong to a different panel (e.g. CommandConfigurator
render) which may not be typable, and focus is wedged. THE ABSENT DIAG LINES
POINT HERE. Verify with the fresh-build run: does `MENUS` produce
`TabKind::MenusEditor`? (Add a temp log of the active tab kind right after
`open_menus_editor`.)

## Deferred (still open from CR-CH-022, unrelated to this bug)

- Task 25.5: multi-segment chained separators `=a.b` / `=a;b` (never implemented).
- Task 25.7: nav_stack session persistence.

## When fixed

- Remove the temp diagnostic from shell/render.rs.
- Revert/redo the editor layout to a compact, scrollable single-row-per-option
  (owner: current vertical layout is ugly and overflows).
- Log the theme.active storm as its own B### and fix it.
- Update B054 note in docs/status/bugs.md (currently marked FIXED -- it is NOT;
  reopen it).
- verify.ps1 CLEAN, rebuild, commit, push.
