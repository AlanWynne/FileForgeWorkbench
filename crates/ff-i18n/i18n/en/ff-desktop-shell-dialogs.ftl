# English Identity_Base keys for the shell dialog / overlay cluster
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/shell/update_dialogs.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() / t_args() (localization Req 4.1, 9.1, 10.1,
# 10.2).
#
# NOT localised (Req 4.2, 4.4): the swap-row "{n}: {label}" formatting (DATA --
# tab number + title); the external cmd_display (program + args, DATA); the
# "External execution cancelled." open_error (status message, Task 9); palette
# entry data; the swap_list_number_input / reset_bare_modal ids. "Cancel"
# appears in several dialogs with the SAME value, so one key serves them all.
# The reset-profile COUNT is passed as a TEXT placeable (pre-formatted in Rust)
# so the rendering stays byte-identical (no Fluent NUMBER() grouping). The 4
# leading spaces of the per-profile bullet are preserved with a quoted placeable.

ff-desktop-shell-dialogs-cancel = Cancel
ff-desktop-shell-dialogs-history-title = Command History
ff-desktop-shell-dialogs-history-empty = No command history.
ff-desktop-shell-dialogs-swap-title = Swap to Tab
ff-desktop-shell-dialogs-swap-number = Number:
ff-desktop-shell-dialogs-exec-title = Run external program?
ff-desktop-shell-dialogs-exec-body = An external program is about to run:
ff-desktop-shell-dialogs-exec-run = Run
ff-desktop-shell-dialogs-reset-heading = Reset to barebones?
ff-desktop-shell-dialogs-reset-intro = This archives the current configuration and reopens the workbench in a minimal barebones state.
ff-desktop-shell-dialogs-reset-one = Profile to be reset: { $name }
ff-desktop-shell-dialogs-reset-many = { $count } profiles will be reset:
ff-desktop-shell-dialogs-reset-bullet = { "    " }- { $name }
ff-desktop-shell-dialogs-reset-archive-note = Each profile's menus, themes, session, config and catalogs are MOVED (not deleted) to a timestamped folder under that profile's config-archive/ so you can recover them later.
ff-desktop-shell-dialogs-reset-cancel-hover = Close without changing any configuration
ff-desktop-shell-dialogs-reset-confirm = Confirm reset
ff-desktop-shell-dialogs-unsaved-title = Unsaved Workspace Changes
ff-desktop-shell-dialogs-unsaved-body = Workspace '{ $name }' has unsaved changes.
ff-desktop-shell-dialogs-unsaved-save = Save
ff-desktop-shell-dialogs-unsaved-discard = Discard
