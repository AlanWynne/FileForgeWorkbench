# RESUME NOTE -- B054 Menus editor typing + Tab accessibility (paused)

Last updated: paused for a break. Pick up from here.

## One-line status

B054 (Menus editor fields un-typable + command line blocked) is FIXED. Tab
keyboard navigation through the editor was then rebuilt for full accessibility.
Committed to a safe point (see below). AWAITING owner's final in-app Tab test,
then a small close-out remains.

## Git state

- Last commit before this note: `de74fdc`.
- THIS session's work committed on top (see the commit made alongside this note):
  the B054 focus fix + the Tab focus-ring accessibility rework. verify.ps1 was
  CLEAN (FULL, nextest) at commit time; ffwb.exe rebuilt.
- UNTRACKED (owner-added, NOT committed by me -- leave alone / owner decides):
  - `docs/source-documents/FileForgeWorkbench-Screen-Snapshot-Architecture.MD`
  - `docs/source-documents/FileForgeWorkbench_Easter_Egg_Commands.docx`

## What was fixed (root causes, now understood)

1. B054 typing bug -- ROOT CAUSE was NOT layout (3 layout attempts wasted). The
   shell's Tab-cycle / focus_stop machinery (built for the POM option ring +
   panels) ran every frame for the Menus editor too: it CONSUMED the Tab key
   (`i.events.retain` deleting Tab events) and force-requested focus onto the
   command field / POM stops, so text fields AND the command line could not hold
   focus. Fix: `shell/update.rs` -- the Tab block no longer force-cycles for a
   MenusEditor tab; instead the shell drives an explicit Menus-editor focus ring.
2. Tab traversal -- egui native Tab followed widget CREATION order (menu bar is
   built before the central panel), so Tab drifted up to the menu bar. Fix: the
   panel render (`menus_editor_panel/render.rs`) now CAPTURES each interactive
   control's real `Response.id` into `MenusEditorState.focus_ids` in visual
   order; the shell (`shell/menus_editor.rs::menus_editor_focus_ring`) prefixes
   the command line and the Tab handler (`shell/update.rs`) walks that list.
   Every control is keyboard-reachable (combo, checkboxes, separator, per-option
   key/command/description/group/on/^/v/Delete, footer Add/Save/SaveAs) --
   accessibility fix at the owner's (correct) insistence. Shift+Tab reverses.
3. Layout: reverted to the compact single-row-per-option layout (owner preferred
   it); footer pinned via `TopBottomPanel::bottom(...).show_inside` so Add/Save
   are never hidden behind the status bar; options in a `CentralPanel` + both-axis
   ScrollArea so the list scrolls and the button stays visible.
4. Removed the temporary B054 diagnostic from `shell/render.rs` earlier.

## AWAITING owner test (before final close-out)

Owner was about to run `.\target\debug\ffwb.exe`, open the Menus editor, and Tab
through to confirm order: command line -> menu selector -> title -> show_calendar
-> group_headers -> separator (Space/Line/None) -> per option (Key/Command/
Description/Group/on/^/v/Delete) -> footer (Add/Save/SaveAs name/SaveAs) -> wraps.
Also check each reachable control is OPERABLE by keyboard (Space/Enter toggles the
combo/checkboxes/selectables). Report any control that is reachable but not
operable.

## CLOSE-OUT still to do (after owner confirms Tab works)

- Mark B054 FIXED in `docs/status/bugs.md` (currently OPEN/REOPENED).
- Delete this resume note.
- Update TCR / tasks for the B054 + accessibility work if needed.
- LOG A NEW BUG: the per-frame `theme.active` validation-warning STORM. The logs
  (`%LOCALAPPDATA%\FileForgeWorkbench\logs\*.log`) flood every frame with
  `WARN [ff_config::access] validation: key "theme.active": value is not in
  allowed set -- applying default`. Something wrote an invalid value (likely a
  theme NAME like "Default Legacy") into the `theme.active` MODE key (allowed:
  dark/light/high_contrast/legacy). Per-frame read is in `shell/update.rs`
  (~L360-410). Log it as its own B###, fix (stop persisting a name into the mode
  key; validate once, not every frame). Noisy but not the typing blocker.
- Deferred from CR-CH-022 (unrelated): task 25.5 multi-segment chained
  separators `=a.b`/`=a;b`; task 25.7 nav_stack session persistence.

## Test status

verify.ps1 CLEAN (FULL, nextest) at commit time. 9 menus-editor unit tests pass
including `menus_editor_focus_ring_starts_with_command_line_and_appends_captured_ids`.
