# English Identity_Base keys for the Kinds Editor Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/kinds_editor_panel/render.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): stable_name() / display_title() Kind identity
# strings (asserted in workspace_kind/tests.rs); the "ext:{n}" base-label format;
# the edit-profile MODE mnemonic checkbox captions (CAPS, NULLS, STATS, LOCK,
# HILITE, Unicode) and the Top/Bottom command-line-position enum display values
# (logic/identity toggles, same class as the Aa/W/.* mnemonic captions); all
# egui id_salt / Id::new values.

ff-desktop-kinds-editor-kind-label = Kind:
ff-desktop-kinds-editor-kind-select = (select)
ff-desktop-kinds-editor-new-label = New Kind:
ff-desktop-kinds-editor-new-name-hint = name
ff-desktop-kinds-editor-modelled-on-inline = modelled on
ff-desktop-kinds-editor-create = Create
ff-desktop-kinds-editor-select-prompt = Select a Kind to edit, or create a new one.
ff-desktop-kinds-editor-modelled-on-label = Modelled on:
ff-desktop-kinds-editor-change-base-hint = (create a new Kind to change the base)
ff-desktop-kinds-editor-title-label = Title:
ff-desktop-kinds-editor-menu-bar-label = Menu bar:
ff-desktop-kinds-editor-key-list-label = Key list:
ff-desktop-kinds-editor-profile = Profile
ff-desktop-kinds-editor-tab-size = Tab size:
ff-desktop-kinds-editor-line-endings = Line endings:
ff-desktop-kinds-editor-command-line = Command line:
ff-desktop-kinds-editor-save = Save
ff-desktop-kinds-editor-saves-to = Saves to workspace-kinds/<name>.toml
ff-desktop-kinds-editor-optional-hint = (use base default)
