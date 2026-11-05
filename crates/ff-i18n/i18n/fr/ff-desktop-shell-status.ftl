# French (fr) shell status bar (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/ff-desktop-shell-status.ftl; only prose translated. The banner keeps the
# product name + version verbatim (Req 4.2). Line/col/count are text placeables.
# Phase labels and CAPS are uppercase status mnemonics (translatable).

ff-desktop-shell-status-banner = FileForge Workbench v0.1.0
ff-desktop-shell-status-log-indicator = LOG!
ff-desktop-shell-status-phase-running = EN COURS
ff-desktop-shell-status-phase-starting = D{ "\u00c9" }MARRAGE
ff-desktop-shell-status-phase-shutting-down = ARR{ "\u00ca" }T EN COURS
ff-desktop-shell-status-phase-terminated = TERMIN{ "\u00c9" }
ff-desktop-shell-status-line-col = Lgn { $line }, Col { $col }
ff-desktop-shell-status-line-count = { $count } lignes
ff-desktop-shell-status-caps = MAJ
ff-desktop-shell-status-log-degraded-hover = Journalisation d{ "\u00e9" }grad{ "\u00e9" }e : { $reason }

# --- Alias Editor status segment (CR-NR-116, Req 7.25) ---
ff-desktop-shell-status-alias-count = Alias : { $count }
ff-desktop-shell-status-alias-rows = Lignes 1-{ $max } visibles
ff-desktop-shell-status-alias-rows-none = Lignes 0 visibles
ff-desktop-shell-status-alias-filter = Filtre : { $ce }/{ $root }
