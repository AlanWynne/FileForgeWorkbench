# English Identity_Base keys for the Command Palette overlay
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/command_palette/render.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() / t_args() (localization Req 4.1, 9.1, 10.1).
#
# NOT localised (Req 4.2, 4.4): the Window::new("Command Palette") title (the
# window has title_bar(false) so it is never displayed); per-entry command
# display_name / category / command_id / shortcut (command-store DATA, not a
# render literal here); the palette_search_input / command_palette_window ids.
# The embedded user query in the empty-state message is passed as a TEXT
# placeable (data), not translated.

ff-desktop-command-palette-search-hint = Type to search commands...
ff-desktop-command-palette-recently-used = Recently Used
ff-desktop-command-palette-no-match = No commands match '{ $query }'
