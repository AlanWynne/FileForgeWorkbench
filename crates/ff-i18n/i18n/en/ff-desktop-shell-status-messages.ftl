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

# --- Workspace open/save (workspace_io.rs) ---
# "WORKSPACE SAVE AS" is a command verb phrase, baked into the English value (D1).
# The root path and error text are DATA placeables (non-translated).
ff-desktop-shell-msg-ws-open-failed = Cannot open workspace: { $error }
ff-desktop-shell-msg-ws-root-not-found = Workspace warning: root '{ $root }' not found on disk
ff-desktop-shell-msg-ws-none-to-save = No active workspace to save
ff-desktop-shell-msg-ws-no-path = No workspace file path set -- use WORKSPACE SAVE AS <path>
ff-desktop-shell-msg-ws-save-failed = Cannot save workspace: { $error }

# --- Session restore (commands_session.rs) ---
ff-desktop-shell-msg-restore-failed = Could not restore: { $error }

# --- Theme set/activate (render_theme.rs) ---
# mode section name and theme name are DATA placeables; error is pre-formatted.
ff-desktop-shell-msg-theme-mode-save-failed = Theme changed to { $mode } for this session, but could not be saved: { $error }
ff-desktop-shell-msg-theme-name-save-failed = Theme '{ $name }' applied for this session, but could not be saved: { $error }
ff-desktop-shell-msg-theme-load-failed = Theme '{ $name }' could not be loaded

# --- Macro library / search apply (actions.rs) ---
# "Delete failed: { $error }" reuses ff-desktop-shell-msg-delete-failed (identical
# text, shared with render_nav_ops). Replace counts are TEXT placeables.
ff-desktop-shell-msg-lua-not-available = Lua execution not yet available
ff-desktop-shell-msg-replace-summary = Replaced { $count } occurrence(s) in { $files } file(s)
ff-desktop-shell-msg-replace-failed = Replace failed: { $error }

# --- External program execution (external_adapter.rs) ---
# program name + exit code are DATA placeables. The ShellDisabled display-boundary
# key and the open-ended spawn e.to_string() errors are handled in step 11 / left
# per D2 (not here).
ff-desktop-shell-msg-external-started-detached = Started '{ $program }' (detached).
ff-desktop-shell-msg-external-finished = '{ $program }' finished (exit { $code }). See the Output panel.

# --- Startup workspace restore (update_startup.rs) ---
ff-desktop-shell-msg-workspace-not-found = Workspace '{ $path }' not found -- starting without workspace

# --- SCRM capture/snapshot command status (commands_scrm.rs) ---
# Only the clearly-UF command-status own-literals here. The SCRM ENGINE report
# returns (scrm.start/stop/status/list/purge and the handle_capture_export/
# evidence/save/load helper report strings, e.g. "Collection exported to ...",
# "CAPTURE LOAD failed: ...") are DEFERRED per decision D3 (SCRM report text) and
# left as source strings this gate. CAPTURE / CAPTURE SCREEN verbs baked in (D1);
# seq, subcommand, and format label are DATA placeables.
ff-desktop-shell-msg-scrm-captured-screen = Captured screen { $seq }.
ff-desktop-shell-msg-scrm-screen-not-capturable = CAPTURE SCREEN: the active workspace cannot be captured yet.
ff-desktop-shell-msg-scrm-unknown-subcommand = CAPTURE: unknown subcommand '{ $subcommand }'.
ff-desktop-shell-msg-scrm-no-collection-replay = No screen collection to replay.
ff-desktop-shell-msg-scrm-snapshot-copied = Snapshot copied to clipboard ({ $format }).
ff-desktop-shell-msg-scrm-snapshot-no-clipboard = Snapshot rendered ({ $format }) but the clipboard was unavailable.
