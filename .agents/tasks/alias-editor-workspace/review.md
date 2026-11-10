# Review -- CR-NR-113 Alias Editor Context

Branch: `crnr113-alias-editor`
Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr113-alias-editor`
Diff reviewed: `git diff c7b9c26..HEAD` (5 commits, 28 files, +1397/-43)
Requirements: `docs/specs/command-configurator/requirements.md` Requirement 5 (5.1-5.12)
Steering: framework-conformance.md, wiring-standard.md, workspace-conformance.md, rust-standards.md

## Verdict: APPROVED (no blocking findings)

## Evidence summary

Verification evidence read (not re-run):
- Owner full gate (original request message #12): `cargo gate` CLEAN -- fmt ok, clippy clean,
  build ok, test 9959/9959 passed, 0 failed. Authoritative green suite.
- Coder commits: `422a19a feat` (Context + user store), `b30f984 test` (resolver/save/focus/
  end-to-end), `247c5b2 docs(tcr)` (12 rows flipped), `3fe1fed docs(tasks)`, `daf38a3 refactor`
  (alias_reader 400-line split).
- Spot-checks performed (narrow, read-only): line counts, unwrap/expect scan, non-ASCII scan,
  bespoke-intercept grep, ff-session diff (empty). All via logged tools/ scripts, read back.

## Checks against the task's specific points

### Single dispatch FRONT DOOR -- PASS
`command_config/mod.rs`: `BUILTIN_WORKSPACE_VERB_ROOTS` gains `"ALIASES"`;
`builtin_workspace_target_for` maps `ALIASES -> custom("aliases")`. `target_dispatch.rs`
`dispatch_command_target` CustomWorkspace arm routes `"aliases" => self.open_alias_editor()`.
`open_alias_editor` (shell/alias_editor.rs) calls `navigate_to(CommandConfigurator { editor:
"aliases" }, push=true)` -- transforms the current tab IN PLACE and pushes the opener onto the
per-tab Navigation_Stack. `nav_reconstruct.rs reconstruct_custom` maps the `editor="aliases"`
marker to `TabKind::AliasEditor` titled `[ALIASES]`, exactly mirroring Theme/Menus/Keys/Kinds.
No bespoke `if upper == "ALIASES"` intercept found in any `commands*.rs` / `dispatch*.rs`
(grep clean); no parallel dispatcher. Front door is the only path.

### ONE WorkspaceContext focus-latch path -- PASS
`alias_editor_panel/render.rs` implements `WorkspaceContext for AliasEditorState`, returning
`InteriorFocus { first, last }`. `render_body.rs TabKind::AliasEditor` arm uses the owned-panel
swap `mem::take -> render_workspace_context -> put back -> apply_alias_editor_action`, i.e. the
single shared latch path -- no hand-wired per-arm latch. The first interior control has a stable
`egui::Id`: the first row's surface field (`surface_id(0)`) when rows exist, else the "Add alias"
button via `push_id(add_button_id())`. LAST is the stable Save button. The frame-reset of
`first_interior_id`/`last_interior_id` in `render_active_tab_body` is honoured (the arm
re-populates through the trait).

### NO new WorkspaceKind; nothing built-in written to disk -- PASS
`git diff` of `crates/ff-session` is EMPTY -- no new `WorkspaceKind` enum variant; aliases reuse
`WorkspaceKind::CommandConfigurator` with the `editor="aliases"` param. The new `TabKind`/
`KindTag::AliasEditor` variants and `BuiltinKind::Aliases` registry entry are the per-editor
pattern every sibling editor already follows (CR-NR-090 Workspace Kinds registry), NOT the
ff_session framework enum. Only `user.toml` (user data) is written, created on first save
(`save_user_aliases` create_dir_all + write). No built-in menu/theme/catalogue is written to disk.

### User-store overlay precedence -- PASS
`alias_reader.rs seed_and_overlay_for_locale`: seed (Identity_Base) -> per-locale
`i18n/<locale>/aliases/` overlay -> User_Alias_Store overlay, in that order (THIRD layer).
`alias_user_overlay.rs apply_user_rows` applies ONE row at a time so the overlay seam's own
within-environment discipline yields SAME-root = no-op and DIFFERENT-root = row rejected with a
WARN (per-locale/base surface wins, Req 5.7). NO-SHADOW on `builtin_ffcmd_roots()` (which unions
the built-in menu roots AND `BUILTIN_WORKSPACE_VERB_ROOTS`) is enforced at SAVE (reject,
`validate_user_alias_rows` branch (d)) and at LOAD (skip-with-WARN, `apply_user_rows`). Proven by
`validate_rejects_builtin_root_shadow` and `load_skips_user_row_that_shadows_builtin_root`.

### Req 5.6 live-refresh re-seeds BOTH overlays -- PASS
`refresh_alias_store_in_place` reads the existing FFCMD roots back from the live store, then one
re-seed re-applies seed + per-locale overlay + user overlay (honouring the shell's resolved,
test-overridable user path). Test `refresh_keeps_base_layer_and_adds_user_layer` proves CONFIG
(a built-in root) still resolves AND the new user alias GG resolves after save -- neither layer
dropped. `save_writes_file_and_live_resolver_resolves_new_alias` proves immediate resolution with
no restart.

### Req 5.10 full-shell first-Tab egui_kittest -- PASS
`tests_focus.rs::full_shell_alias_editor_first_tab_focuses_first_interior`: opens ALIASES via
`dispatch_command_string`, asserts the Context is AliasEditor, that a first interior id is
reported, that entry focus is the command field, and that the first Tab lands EXACTLY on the
reported first interior (no phantom stop).

### CR-NR-112 reuse, no double-expansion -- PASS
`parameterised_alias_authored_in_editor_expands_end_to_end` authors `ECOB -> EDIT` with template
`FFWB.COBOL.MASTER(&1)`, saves, and asserts `rewrite_verb_to_root("ecob helloW")` expands to
`EDIT FFWB.COBOL.MASTER(helloW)` through the EXISTING single pre-pass. The editor authors data
only; no new expansion/dispatch behaviour is added (alias_template.rs is unchanged by this CR).

### TCR rows -- PASS
`docs/quality/TCR.md`: the CR-NR-113 section carries 12 rows (Req 5.1-5.12), all flipped to
covered with real proving test fn names, each confirmed to exist in source (e.g.
`aliases_verb_opens_the_alias_editor_context`, `validate_rejects_builtin_root_shadow`,
`user_rows_route_into_their_target_environment`, `full_shell_alias_editor_first_tab_focuses_first_interior`,
`non_ascii_field_value_round_trips`). The adjacent CR-NR-112 table is untouched (still its own
112 test names and criteria).

### Standards -- PASS
- 400-line rule: non-test line counts -- render.rs 181, state.rs 77, mod.rs 12, alias_editor.rs
  ~231 (tests from 232), user_aliases.rs ~143 (tests from 144), alias_reader.rs ~372 (tests from
  373; the overlay methods were split into alias_user_overlay.rs 97 lines precisely to stay under
  400), alias_user_overlay.rs 97. All under 400.
- No unwrap/expect in production: scan shows unwrap/expect only inside `#[cfg(test)]` regions.
- Plain ASCII: non-ASCII scan of all new source files is clean.

## Notes (non-blocking, no action required)
- TCR uses the green-circle marker for these rows, matching the adjacent CR-NR-112 table's
  established convention; the file-level Status Key table lists different legacy symbols. This is
  pre-existing file convention, not introduced by this change.
