# English Identity_Base keys for the Menus Editor Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/menus_editor_panel/render.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): the "^"/"v" move-row glyph buttons (directional
# icons); the terse "on"/"bar" toggle mnemonics (enabled / show_in_menu_bar,
# same class as the Aa/W/.* mnemonic captions); all egui id_salt / Id::new
# values (menus_editor_title, menus_opt_*, menus_editor_save_as_name, etc.);
# user menu.title buffer content.

ff-desktop-menus-editor-menu-label = Menu:
ff-desktop-menus-editor-menu-none = (none)
ff-desktop-menus-editor-select-prompt = Select a menu to edit.
ff-desktop-menus-editor-title-label = Title:
ff-desktop-menus-editor-show-calendar = Show calendar
ff-desktop-menus-editor-group-headers = Group headers
ff-desktop-menus-editor-group-separator = Group separator:
ff-desktop-menus-editor-sep-space = Space
ff-desktop-menus-editor-sep-line = Line
ff-desktop-menus-editor-sep-none = None
ff-desktop-menus-editor-options-header = Options (key | command | description | group)
ff-desktop-menus-editor-hint-key = key
ff-desktop-menus-editor-hint-command = command
ff-desktop-menus-editor-hint-description = description
ff-desktop-menus-editor-hint-group = group
ff-desktop-menus-editor-delete = Delete
ff-desktop-menus-editor-add-option = Add option
ff-desktop-menus-editor-save = Save
ff-desktop-menus-editor-save-as-label = Save As:
ff-desktop-menus-editor-save-as-hint = new name
ff-desktop-menus-editor-save-as = Save As
