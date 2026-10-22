# English Identity_Base keys for the shell command-line / scroll-field prompts
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/shell/render_command_line.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): the command_field_input / scroll_field_input /
# detached_command_field_input ids and the typed command buffer. The
# "Command ===>" prompt is used at BOTH the primary field and the shared
# command-field body (same value, one key).
#
# TASK 9 added the SCROLL invalid-amount error (previously deferred). The SCROLL
# verb and the PAGE/HALF/CSR/MAX/DATA/n keyword list stay English (baked into the
# value, D1); only the offending argument is a DATA placeable. ONE shared key is
# used at both render_command_line.rs and environment_ffedit.rs (identical text).

ff-desktop-shell-commandline-command-prompt = Command ===>
ff-desktop-shell-commandline-scroll-prompt = SCROLL ===>
ff-desktop-shell-commandline-scroll-invalid = SCROLL: '{ $arg }' is not a valid scroll amount (PAGE/HALF/CSR/MAX/DATA/n)
