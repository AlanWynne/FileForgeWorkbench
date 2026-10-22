# English Identity_Base keys for the Primary Option Menu calendar chrome
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/primary_option_menu.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() / t_args() (localization Req 4.1, 9.1, 10.1,
# 10.2).
#
# NOT localised (Req 4.2, 4.4): the "<" / ">" month-nav glyph buttons
# (directional icons); the blank / single-space calendar cells (layout
# whitespace); the "{:>2} " day-number cells and the "{month} {year}" header
# (DATA -- MONTH_NAMES is a model array, a Task 10 concern); POM option
# descriptions (sourced from the menu model, already localised in Task 5). The
# weekday header is one byte-identical value (fixed-width spacing preserved).
# The Time / Day-of-year values are pre-formatted in Rust and passed as TEXT
# placeables so the ":02" / spacing render byte-identically; the dotted-leader
# label prose is what localises.

ff-desktop-pom-weekday-header = Su Mo Tu We Th Fr Sa
ff-desktop-pom-time = Time . . . . : { $value }
ff-desktop-pom-day-of-year = Day of year. :   { $value }
