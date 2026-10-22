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
# Area 14 (Task 8) added the banner + LOG! indicator. TASK 9 (this pass) adds the
# remaining status-bar rows that were deferred: the four lifecycle phase labels,
# the "Ln/Col" and line-count segments, the CAPS mode indicator, and the
# argument-bearing degraded-logging hover. Line/col/count are TEXT placeables
# (pre-formatted in Rust, no Fluent NUMBER()) to stay byte-identical. The phase
# labels and CAPS are uppercase status-mnemonic PROSE (translatable, Req 4.1).
# Still NOT localised: key-label-bar slot text is DATA (key display names + bound
# command labels); session-start/profile/encoding/zoom segments are DATA owned
# elsewhere; the modified-dot glyph is a UI icon.

ff-desktop-shell-status-banner = FileForge Workbench v0.1.0
ff-desktop-shell-status-log-indicator = LOG!

# --- Task 9: lifecycle phase labels (LifecyclePhase match) ---
ff-desktop-shell-status-phase-running = RUNNING
ff-desktop-shell-status-phase-starting = STARTING
ff-desktop-shell-status-phase-shutting-down = SHUTTING DOWN
ff-desktop-shell-status-phase-terminated = TERMINATED

# --- Task 9: cursor position / line count / CAPS / degraded hover ---
# The degraded-logging reason is a DATA placeable (built by logging_degradation_reason).
ff-desktop-shell-status-line-col = Ln { $line }, Col { $col }
ff-desktop-shell-status-line-count = { $count } lines
ff-desktop-shell-status-caps = CAPS
ff-desktop-shell-status-log-degraded-hover = Logging degraded: { $reason }
