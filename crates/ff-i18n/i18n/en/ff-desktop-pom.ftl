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

# Calendar month names (B084): the header month name now localises via the
# catalogue instead of the former hardcoded MONTH_NAMES array (localization
# Req 4.1). Keys are zero-padded two-digit, 01 = January .. 12 = December.
ff-desktop-pom-month-01 = January
ff-desktop-pom-month-02 = February
ff-desktop-pom-month-03 = March
ff-desktop-pom-month-04 = April
ff-desktop-pom-month-05 = May
ff-desktop-pom-month-06 = June
ff-desktop-pom-month-07 = July
ff-desktop-pom-month-08 = August
ff-desktop-pom-month-09 = September
ff-desktop-pom-month-10 = October
ff-desktop-pom-month-11 = November
ff-desktop-pom-month-12 = December
