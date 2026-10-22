# English Identity_Base keys for the Event Log Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/event_log_panel.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): the log-level enum captions (All / Info /
# Success / Warning / Error) are logic/identity values driven by the
# NotificationLevel enum match; the per-entry "[{timestamp}] {level} {title}"
# row is data formatting; egui id_salt values; NotificationLevel::label().

ff-desktop-event-log-heading = Event Log
ff-desktop-event-log-search-hint = search...
ff-desktop-event-log-empty = No log entries.
# Added in the area-20 final sweep (missed in area 3): the Clear Log button.
ff-desktop-event-log-clear = Clear Log
