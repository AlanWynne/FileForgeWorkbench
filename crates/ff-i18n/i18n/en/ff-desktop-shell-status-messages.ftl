# English Identity_Base keys for shell STATUS / ERROR messages
# (CR-NR-103, Localisation Phase 2, TASK 9, crate ff-desktop).
#
# These are the argument-bearing / verb-embedding open_error status messages
# deferred from Task 8 (which localised the plain render literals). Values are
# the EXACT current English text, byte-for-byte, ASCII only, resolved through
# ff_i18n::t() / t_args() (localization Req 4.1, 9.1, 9.2, 10.1, 10.2).
#
# Conventions (Task 8, confirmed in the shipped .ftl headers):
# - Interpolated runtime values are named placeables supplied via t_args.
# - Error text from a `?`/Display source is pre-formatted in Rust (e.to_string())
#   and passed as a TEXT placeable { $error }; it is NOT translated here (Req 9.3
#   display-boundary: the error enum stays source-language).
# - Counts are passed as TEXT placeables (pre-formatted in Rust), NOT Fluent
#   NUMBER(), so rendering stays byte-identical for counts >= 1000 (no grouping).
# - Embedded command VERBS/keywords stay English: for single verb-prefix
#   messages the verb is baked into the English value (D1); the one mid-sentence
#   case uses a non-translated { $verb } placeable.
# - File/dataset names, command ids, and workspace-kind strings are DATA and are
#   passed as non-translated placeables (Req 4.2).

# --- File Explorer node operations (render_nav_ops.rs) ---
ff-desktop-shell-msg-new-file-failed = New file/folder: { $error }
ff-desktop-shell-msg-create-failed = Create failed: { $error }
ff-desktop-shell-msg-paste-failed = Paste: { $error }
ff-desktop-shell-msg-paste-count-failed = Paste: { $count } file(s) could not be copied
ff-desktop-shell-msg-delete-provider-failed = Delete: { $error }
ff-desktop-shell-msg-delete-failed = Delete failed: { $error }
ff-desktop-shell-msg-rename-provider-failed = Rename: { $error }
ff-desktop-shell-msg-rename-failed = Rename failed: { $error }
