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

# === TASK 9.3 -- verb-embedding command-ladder messages ===
# Verb + keyword tokens (DETACH/UNSPLIT/FOCUS/SWAP/SUBMIT/STATUS/CREATE/REPLACE/
# COMPARE/WORKSPACE + OPEN/SAVE/LIST/...) are English, BAKED into the value (D1).
# Interpolated DATA (tab number, range, arg, dsn, filter, date/time/day) are
# placeables. Counts/numbers are TEXT placeables (pre-formatted) for byte-identical
# en rendering.

# --- detach / split / swap (commands_ladder_c.rs) ---
ff-desktop-shell-msg-detach-max-reached = Maximum number of detached Workspaces (16) reached.
ff-desktop-shell-msg-unsplit-not-split = UNSPLIT: the Workspace is not split.
ff-desktop-shell-msg-focus-not-split = FOCUS: the Workspace is not split.
ff-desktop-shell-msg-swap-out-of-range = SWAP: tab number { $n } out of range (valid: 1 to { $count })
ff-desktop-shell-msg-swap-bad-arg = SWAP: invalid argument '{ $arg }'. Usage: SWAP <n>, SWAP LIST

# --- JES / dataset stubs (commands_ladder_c.rs) ---
ff-desktop-shell-msg-submit-not-available = SUBMIT: JES subsystem not yet available
ff-desktop-shell-msg-time-status = Date: { $date }  Time: { $time }  Day: { $day }
ff-desktop-shell-msg-status-routing-filter = STATUS: routing to JES panel (filter: { $filter })
ff-desktop-shell-msg-status-routing = STATUS: routing to JES job status panel
ff-desktop-shell-msg-create-not-available = CREATE { $dsn }: dataset creation not yet available
ff-desktop-shell-msg-replace-dsn-not-available = REPLACE { $dsn }: dataset replace not yet available
ff-desktop-shell-msg-compare-not-available = COMPARE { $dsn }: compare view not yet available

# --- WORKSPACE subcommands (commands_ladder_c.rs) ---
ff-desktop-shell-msg-workspace-open-requires-path = WORKSPACE OPEN requires a path
ff-desktop-shell-msg-workspace-saveas-requires-path = WORKSPACE SAVE AS requires a path
ff-desktop-shell-msg-workspace-addroot-requires-path = WORKSPACE ADD ROOT requires a path
ff-desktop-shell-msg-workspace-no-active-open-first = No active workspace -- use WORKSPACE OPEN first
ff-desktop-shell-msg-workspace-removeroot-requires-path = WORKSPACE REMOVE ROOT requires a path
ff-desktop-shell-msg-workspace-no-active = No active workspace
ff-desktop-shell-msg-workspace-unknown-subcommand = WORKSPACE: unknown subcommand. Use OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT

# --- verb-prefixed cluster B (ladder_a/b, commands, fastpath, target_dispatch, dialogs) ---
# EDIT/DOCK/COMMAND/THEME verbs + TOP/BOTTOM keywords baked (D1). arg/name are
# DATA placeables (theme name, position token echoed back). command_id and
# workspace_kind strings are stable identifiers -> non-translated DATA placeables.
ff-desktop-shell-msg-edit-requires-path = EDIT requires a file path
ff-desktop-shell-msg-dock-not-detached = DOCK: the current workspace is not detached
ff-desktop-shell-msg-command-bad-position = COMMAND: '{ $arg }' is not a valid position (TOP or BOTTOM)
ff-desktop-shell-msg-theme-not-exist = THEME: '{ $name }' does not exist
ff-desktop-shell-msg-chain-depth-exceeded = Chained path exceeds maximum depth of 4 segments.
ff-desktop-shell-msg-command-not-defined = Command '{ $id }' is not defined.
ff-desktop-shell-msg-custom-ws-not-runnable = Custom workspace target '{ $target }' is not yet runnable via a command binding.
ff-desktop-shell-msg-macro-not-runnable = Macro targets are not yet runnable via a command binding.
ff-desktop-shell-msg-external-cancelled = External execution cancelled.

# --- mid-sentence verb (Req 9.2 canonical) + HILITE/PFSHOW ---
# profile-locked is THE Req 9.2 example: the verb "LOCK OFF" sits MID-SENTENCE
# and would move under translation, so it is a NON-translated { $verb } placeable
# (NOT baked), while the surrounding prose localises. HILITE / PFSHOW verbs +
# their keyword lists stay baked English (D1); keyword/arg echoed back are DATA.
ff-desktop-shell-msg-profile-locked = Profile is locked -- use { $verb } to unlock
ff-desktop-shell-msg-hilite-unknown-mode = HILITE: unknown mode '{ $keyword }'
ff-desktop-shell-msg-pfshow-bad-arg = Unknown PFSHOW argument '{ $arg }' -- expected ON, OFF, BASE, SHIFT, CTRL, or ALT.

# === TASK 9.3 -- thiserror DISPLAY-BOUNDARY keys (D2) ===
# Req 9.3: localise WHERE SHOWN, not at the error source. The ProfileError /
# ShellError enums are UNCHANGED (ff_i18n is NOT a dependency of the library
# error crates). Only the fixed-shape variants are keyed here; open-ended
# runtime/OS e.to_string() errors stay pass-through. The clean user-facing keys
# DROP the enum's "[profile]" / "[shell]" dev prefixes. key/value/name are DATA.
ff-desktop-shell-msg-profile-unknown-key = Unknown profile key: { $key }
ff-desktop-shell-msg-profile-unknown-value = Unknown value for { $key }: { $value }
ff-desktop-shell-msg-shell-disabled = Shell access is disabled by configuration
