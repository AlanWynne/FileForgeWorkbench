# English Identity_Base keys for the Compiler Toolchain panel
# (CR-NR-103, Localisation Phase 2 Slice 2, task 7.6, crate ff-toolchain-panel).
#
# These values are the EXACT current English literals from the ff-toolchain-panel
# render function (lib.rs), byte-for-byte, ASCII only. Render code resolves each
# displayed string through the ff_i18n::t()/t_args() seam (localization Req 4.1,
# 10.1).
#
# NOT localised and left as source literals (Req 4.2, 4.4): egui id_salt values;
# the toolchain plugin names (Rust / GCC -- proper nouns, shown via {name:<6} and
# compared in logic); the diagnostic severity prefixes E/W/N (severity_label --
# terse codes, with English-asserting unit tests); and ToolchainEntry::
# build_status_text() (a model-layer status method with its own unit tests
# asserting English "succeeded"/"failed"/"error" substrings -- localising it
# would break those tests, so it stays English in this slice).

# --- Section headings -------------------------------------------------------
toolchain-panel-heading-toolchains = Compiler Toolchains
toolchain-panel-heading-build-output = Build Output
toolchain-panel-heading-diagnostics = Diagnostics

# --- Per-toolchain status row -----------------------------------------------
toolchain-panel-status-not-found = Not found
toolchain-panel-btn-install-rustup = Install via rustup
toolchain-panel-btn-install-gcc = Install GCC
toolchain-panel-status-installing = Installing...
toolchain-panel-status-install-failed = Install failed: { $reason }
toolchain-panel-btn-retry = Retry
toolchain-panel-status-ready = Ready -- { $version }
toolchain-panel-status-detected = Detected -- { $version }
toolchain-panel-btn-build = Build
toolchain-panel-status-building = Building...
toolchain-panel-status-unknown = Unknown state
