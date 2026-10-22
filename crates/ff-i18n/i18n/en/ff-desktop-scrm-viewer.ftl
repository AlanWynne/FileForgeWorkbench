# English Identity_Base keys for the SCRM (screen-snapshot) viewer panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop -- area 20 sweep).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/scrm_viewer_panel.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() / t_args() (localization Req 4.1, 9.1, 10.1).
#
# The four navigation buttons embed directional glyphs (|<, <, >, >|) as part of
# their label text; they are kept verbatim in the value (the glyph is inseparable
# from the button's meaning). The capture position counts are passed as TEXT
# placeables (pre-formatted) to keep byte-identical rendering (no Fluent NUMBER()
# grouping). The screen TextEdit content is DATA (the captured screen text).

ff-desktop-scrm-viewer-first = |< First
ff-desktop-scrm-viewer-prev = < Prev
ff-desktop-scrm-viewer-next = Next >
ff-desktop-scrm-viewer-last = Last >|
ff-desktop-scrm-viewer-position = Capture { $position } of { $count }
ff-desktop-scrm-viewer-no-captures = No captures
