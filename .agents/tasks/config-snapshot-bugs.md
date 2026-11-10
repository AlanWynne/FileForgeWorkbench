# Config-panel SNAPSHOT bugs (owner-reported 2026-10-31)

Two distinct defects found while snapshotting the CONFIGURATION workspace. Both
are Phase-2A ScreenProvider defects for the Config panel, NOT Phase 2E (dialogs).
Fix as a focused follow-up AFTER scrm-2e merges. Do NOT fold into the 2E branch.

## Bug 1 -- SNAPSHOT collapses the focused config-tree item

Repro: open the Configuration workspace. Focus defaults to the "Accessibility"
tree item. Type `snapshot` in the command line and press Enter. The Enter (or the
SNAPSHOT dispatch) is ALSO consumed by the focused config-tree item as an
activate/collapse, so "Accessibility" COLLAPSES. The window screenshot (PNG) then
captures the COLLAPSED state -- not what the user was looking at when they typed
the command.

Likely cause: the Enter that submits the command line is not fully consumed /
is also reaching the focused tree widget's activate handler, OR the config-tree
key handling collapses the focused node on an Enter that should have been owned by
the command line. Diagnose where the Enter is double-handled (command-line submit
vs config-tree activate) -- analogous to the earlier menu-bar arrow double-consume
class. The SNAPSHOT itself should not change the screen it is capturing.

Expected: typing SNAPSHOT + Enter captures the screen AS IT IS (Accessibility
expanded/focused as the user left it), not a mutated/collapsed state.

## Bug 2 -- Config-panel ScreenModel is far too thin

Repro: SNAPSHOT the Configuration workspace, paste into a text editor. The text
flavor shows only:
  - Filter . . . :
  - Unsaved edits . . . : no
plus a Property/Value table of just those two rows. That is NOT a faithful
representation of the actual Config screen (which shows the settings
tree/categories like Accessibility and their values).

Cause: the Phase-2A per-Context ScreenProvider builder for the Config panel
(crates/ff-desktop/src/shell/screen_providers.rs, the config_panel_screen_model
builder) captures only the filter string + dirty flag, not the visible settings
tree/content. The logical ScreenModel must represent what is actually on screen:
the settings categories/keys and their effective values (respecting the current
filter and expand/collapse state), so the TEXT/Markdown/HTML flavors are a
faithful capture.

Expected: the Config SNAPSHOT text reflects the real screen -- the settings
tree/categories and values the user sees -- not just Filter + Unsaved edits.
This is also where ANSI colour could later matter (changed/error fields), noted
earlier in the investigation.

## Scope note
Both are Config-panel provider issues (Phase 2A). The master-directive retrofit
matrix row for ConfigPanel should be revisited: it has a provider, but the
provider is inadequate (bug 2) and the capture is not side-effect-free (bug 1).
Fix as one small follow-up worktree after 2E, with the owner's manual re-check
(SNAPSHOT the real Config screen, confirm the text matches and nothing collapses).
