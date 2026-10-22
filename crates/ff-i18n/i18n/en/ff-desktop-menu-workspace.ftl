# English Identity_Base keys for the Menu Workspace render placeholder
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/menu_workspace/render.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 2.6, 4.1, 10.1).
#
# SCOPE: area 12 localises ONLY the empty-options placeholder (Req 2.6).
# NOT localised here (Req 4.2, 4.4): menu TITLE / option DESCRIPTIONS -- already
# localised via menu_workspace/i18n_keys.rs (Task 5), do NOT double-localise;
# the load-error fallback ("Menu file error: unknown error") and the advisory
# line are status/error messages deferred to the Task 9 / area-17 status pass;
# group header labels and option columns are DATA from the menu model.

ff-desktop-menu-workspace-no-options = No options defined in this menu.
