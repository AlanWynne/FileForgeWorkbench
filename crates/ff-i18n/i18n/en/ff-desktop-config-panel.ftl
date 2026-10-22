# English Identity_Base keys for the Configuration Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/config_panel/{render.rs,widgets.rs}, byte-for-byte,
# ASCII only. Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): the "X" clear-filter button (a UI glyph, not
# prose); egui id_salt / filter_field_id values; config key names and values.

ff-desktop-config-panel-filter = Filter:
ff-desktop-config-panel-source-file = Source File:
ff-desktop-config-panel-complex-value = [complex value -- edit TOML file directly]
