# Afrikaans (af) shell status bar (CR-NR-103 Task 10). REVIEWED POC locale.
# The version banner keeps the product name + version verbatim (Req 4.2). "LOG!"
# is a status-mnemonic indicator kept recognisable. Phase labels and CAPS are
# uppercase status prose -- translated. Ln/Col/count placeables are DATA.

ff-desktop-shell-status-banner = FileForge Workbench v0.1.0
ff-desktop-shell-status-log-indicator = LOG!

# --- lifecycle phase labels ---
ff-desktop-shell-status-phase-running = BESIG
ff-desktop-shell-status-phase-starting = BEGIN TANS
ff-desktop-shell-status-phase-shutting-down = SLUIT TANS AF
ff-desktop-shell-status-phase-terminated = BE{ "\u00cb" }INDIG

# --- cursor position / line count / CAPS / degraded hover ---
ff-desktop-shell-status-line-col = Re{ "\u00eb"}l { $line }, Kol { $col }
ff-desktop-shell-status-line-count = { $count } re{ "\u00eb" }ls
ff-desktop-shell-status-caps = CAPS
ff-desktop-shell-status-log-degraded-hover = Aantekening afgegradeer: { $reason }
