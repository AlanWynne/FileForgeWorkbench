# English Identity_Base keys for the shell status bar
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/shell/render_status.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# SCOPE: area 14 localises the version banner and the "LOG!" degradation
# indicator (per the plan). The banner value keeps the product name + version
# verbatim (product name stays English, Req 4.2); "LOG!" is prose (a status
# mnemonic indicator).
#
# NOT localised here: the phase labels (RUNNING/STARTING/SHUTTING DOWN/
# TERMINATED -- uppercase status mnemonics), the "Ln {} Col {}" / "{} lines" /
# "CAPS" data+status segments, and the open_error text are swept/decided in
# area 20 / the Task 9 status pass, not area 14; key-label-bar slot text is DATA
# (key display names + bound command labels); the degraded-logging tooltip is an
# argument-bearing status message (Task 9).

ff-desktop-shell-status-banner = FileForge Workbench v0.1.0
ff-desktop-shell-status-log-indicator = LOG!
