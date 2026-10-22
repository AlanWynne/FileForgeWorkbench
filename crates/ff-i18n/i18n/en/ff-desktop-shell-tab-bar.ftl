# English Identity_Base keys for the shell tab bar + tab context menus
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/shell/render_tab_bar.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() / t_args() (localization Req 4.1, 9.1, 9.2,
# 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): the modified-marker glyph and the close "X"
# (U+00D7) glyph (UI icons); tab header labels / titles / paths (DATA); the two
# "Maximum ... floating/detached Workspaces" open_error status messages
# (deferred to the Task 9 status pass). The four "Open Containing Folder in X"
# items share ONE pattern with the tool name (Explorer / CMD / PowerShell /
# Terminal) passed as a NON-translated { $target } placeable so the prose
# localises while the tool name stays English (Req 9.2).

ff-desktop-shell-tab-bar-close-hover = Close tab
ff-desktop-shell-tab-bar-close = Close
ff-desktop-shell-tab-bar-close-all-but = Close All BUT This
ff-desktop-shell-tab-bar-close-left = Close All to the Left
ff-desktop-shell-tab-bar-close-right = Close All to the Right
ff-desktop-shell-tab-bar-close-unchanged = Close All Unchanged
ff-desktop-shell-tab-bar-clone = Clone to Other Tab
ff-desktop-shell-tab-bar-move-other-view = Move to Other View
ff-desktop-shell-tab-bar-pin = Pin Tab
ff-desktop-shell-tab-bar-exit = Exit
ff-desktop-shell-tab-bar-open-folder-in = Open Containing Folder in { $target }
ff-desktop-shell-tab-bar-copy-name = Copy Name to Clipboard
ff-desktop-shell-tab-bar-copy-path = Copy Path to Clipboard
ff-desktop-shell-tab-bar-save = Save
ff-desktop-shell-tab-bar-save-as = Save As
ff-desktop-shell-tab-bar-reload = Reload
ff-desktop-shell-tab-bar-new = New
ff-desktop-shell-tab-bar-new-file = New File
