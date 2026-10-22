# English Identity_Base keys for the Plugin Manager Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/plugin_manager_panel.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): state_label(...) returns the PluginState enum
# captions (Active / Initialized / Loaded / Discovered / Deactivating /
# Shutdown / Unknown) -- these are logic/identity values with unit tests
# asserting exact equality (active_plugin_state_label_is_active,
# shutdown_plugin_state_label_is_shutdown), so they stay unlocalised; egui
# filter_field_id() id value.

ff-desktop-plugin-manager-heading = Plugin Manager
ff-desktop-plugin-manager-filter = Filter:
ff-desktop-plugin-manager-filter-hint = plugin name...
ff-desktop-plugin-manager-clear = Clear
ff-desktop-plugin-manager-state = State:
ff-desktop-plugin-manager-empty = No plugins registered.
ff-desktop-plugin-manager-no-metadata = No additional metadata available.
ff-desktop-plugin-manager-select-prompt = Select a plugin to view details.
