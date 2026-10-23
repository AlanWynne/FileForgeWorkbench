# French (fr) Compiler Toolchain panel (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/toolchain-panel.ftl; only prose translated. Plugin names (Rust / GCC),
# rustup, severity codes E/W/N, and build_status_text() stay English (Req 4.2,
# 4.4). { $reason } / { $version } are placeables.

# --- Section headings -------------------------------------------------------
toolchain-panel-heading-toolchains = Cha{ "\u00ee" }nes d'outils du compilateur
toolchain-panel-heading-build-output = Sortie de compilation
toolchain-panel-heading-diagnostics = Diagnostics

# --- Per-toolchain status row -----------------------------------------------
toolchain-panel-status-not-found = Introuvable
toolchain-panel-btn-install-rustup = Installer via rustup
toolchain-panel-btn-install-gcc = Installer GCC
toolchain-panel-status-installing = Installation...
toolchain-panel-status-install-failed = { "\u00c9" }chec de l'installation : { $reason }
toolchain-panel-btn-retry = R{ "\u00e9" }essayer
toolchain-panel-status-ready = Pr{ "\u00ea" }t -- { $version }
toolchain-panel-status-detected = D{ "\u00e9" }tect{ "\u00e9" } -- { $version }
toolchain-panel-btn-build = Compiler
toolchain-panel-status-building = Compilation...
toolchain-panel-status-unknown = { "\u00c9" }tat inconnu
