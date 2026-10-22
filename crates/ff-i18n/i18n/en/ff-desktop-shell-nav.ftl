# English Identity_Base keys for the shell File Explorer dialogs + nav headings
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/shell/render_nav_ops.rs, render_nav.rs, and render.rs,
# byte-for-byte, ASCII only. Resolved through ff_i18n::t() / t_args()
# (localization Req 4.1, 9.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): every self.open_error = format!(...) message
# (New/Create/Paste/Delete/Rename failures -- status messages deferred to the
# Task 9 status pass); node data; the close glyph on the Toolchain Panel heading
# (a UI icon, left as the raw glyph). "Delete"/"Rename"/"Cancel" each appear as
# both a window title and a button with the SAME value, so one key serves both.

ff-desktop-shell-nav-new-folder-title = New Folder
ff-desktop-shell-nav-new-file-title = New File
ff-desktop-shell-nav-name-label = Name:
ff-desktop-shell-nav-create = Create
ff-desktop-shell-nav-cancel = Cancel
ff-desktop-shell-nav-delete-title = Delete
ff-desktop-shell-nav-delete-confirm = Delete '{ $label }'? This cannot be undone.
ff-desktop-shell-nav-rename-title = Rename
ff-desktop-shell-nav-new-name-label = New name:
ff-desktop-shell-nav-file-explorer-heading = File Explorer
ff-desktop-shell-nav-toolchain-heading = Toolchain Panel
