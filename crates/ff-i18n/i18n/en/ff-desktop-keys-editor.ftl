# English Identity_Base keys for the Keys Editor Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/keys_editor_panel/render.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): row.key.display_name() (F1-F12 model identity
# labels); mk.toml_name() map keys; the egui ("keys_editor_cmd", ...) /
# keys_editor_kind / keys_editor_grid id_salt values.

ff-desktop-keys-editor-kind-label = Workspace kind:
ff-desktop-keys-editor-kind-select = (select)
ff-desktop-keys-editor-select-prompt = Select a workspace kind to edit its key list.
ff-desktop-keys-editor-header-base = Base command
ff-desktop-keys-editor-header-shift = Shift
ff-desktop-keys-editor-header-ctrl = Ctrl
ff-desktop-keys-editor-header-alt = Alt
ff-desktop-keys-editor-unassigned = unassigned
ff-desktop-keys-editor-save = Save
ff-desktop-keys-editor-reset = Reset to default
ff-desktop-keys-editor-saves-to = Saves to keymaps/<kind>.toml
