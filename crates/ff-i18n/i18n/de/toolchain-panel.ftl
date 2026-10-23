# German (de) Compiler Toolchain panel (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/toolchain-panel.ftl; only prose translated. Plugin names (Rust / GCC),
# rustup, severity codes E/W/N, and build_status_text() stay English (Req 4.2,
# 4.4). { $reason } / { $version } are placeables.

# --- Section headings -------------------------------------------------------
toolchain-panel-heading-toolchains = Compiler-Toolchains
toolchain-panel-heading-build-output = Build-Ausgabe
toolchain-panel-heading-diagnostics = Diagnosen

# --- Per-toolchain status row -----------------------------------------------
toolchain-panel-status-not-found = Nicht gefunden
toolchain-panel-btn-install-rustup = { "\u00dc" }ber rustup installieren
toolchain-panel-btn-install-gcc = GCC installieren
toolchain-panel-status-installing = Installiert...
toolchain-panel-status-install-failed = Installation fehlgeschlagen: { $reason }
toolchain-panel-btn-retry = Wiederholen
toolchain-panel-status-ready = Bereit -- { $version }
toolchain-panel-status-detected = Erkannt -- { $version }
toolchain-panel-btn-build = Erstellen
toolchain-panel-status-building = Wird erstellt...
toolchain-panel-status-unknown = Unbekannter Zustand
