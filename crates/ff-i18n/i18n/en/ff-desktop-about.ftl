# English Identity_Base keys for the About dialog (PARTIAL -- area 18)
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/about_dialog.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() / t_args() (localization Req 4.1, 9.1, 10.1).
#
# SCOPE (area 18, PARTIAL): only the window title, the app-name heading, the
# version line, and the Close button are localised. The window title + heading
# keep the product name verbatim (product name stays English, Req 4.2). VERSION
# is passed as a TEXT placeable (a &str). The Close button preserves its two
# leading/trailing spaces with a quoted placeable.
#
# DELIBERATELY NOT localised (Req 4.4): the prose constants CREATOR, AI_CREDIT,
# COPYRIGHT, DESCRIPTION and their ui.label(CONST) sites -- about_dialog.rs has
# UNIT TESTS asserting their English substrings (about_dialog_contains_creator_
# credit, about_dialog_contains_aws_credit, about_dialog_copyright_contains_
# creator_name, about_dialog_description_is_nonempty). They fall under the
# "model-layer string with an asserting test -- LEAVE it" rule; localising them
# is deferred to a later task that moves the assertions to the .ftl value.

ff-desktop-about-window-title = About FileForge Workbench
ff-desktop-about-heading = FileForge Workbench
ff-desktop-about-version = Version { $version }
ff-desktop-about-close = { "  " }Close{ "  " }
