# Alias Editor SAVE -- core-001 strip regression investigation

Read-only investigation of the stripped worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members`
(HEAD = `414ba84`, the core-001 strip commit; parent/main = `4e3db77`).

## Summary answer -- VERDICT: WORKING (production), with LOST TEST COVERAGE

Alias Editor SAVE is **CURRENTLY WORKING** in the stripped core. The strip did
**not** sever the SAVE dispatch path the Alias Context relies on. The full
production chain -- front door -> `resolve_target` (declines) -> ladder active-env
gate (now always FFCMD base) -> `try_commands_a` SAVE arm -> editable-Context
seam -> `alias_editor_persist` -> `save_user_aliases` -> per-locale override
write + live-resolver refresh -- is intact in the worktree, file-for-file.

The regression is **only in test coverage, not in behaviour**: the strip DELETED
the entire CR-CH-071 seam test file `tests_crch071.rs` (470 lines), which held
the test that drove SAVE through the *globally-typed* front door
(`dispatch_command_string("SAVE")`) on the Alias Editor and asserted the override
file was written. The production code those tests validated still exists and is
still reachable. Equivalent coverage of the dispatched SAVE path **survives** in
`tests_alias_editor.rs` (its `save()` helper routes "SAVE" through the same front
door), so Alias SAVE is not entirely untested -- but the dedicated seam tests
(END save-then-leave, CANCEL/RETURN discard-then-leave, REFRESH reload, the
FFEDIT-buffer active-wins boundary, non-editable no-op) were removed.

No production change is required to make Alias SAVE work. The only remedial
action worth considering is restoring the Alias-relevant subset of the deleted
seam tests so the retained core behaviour is guarded against future drift.

---

## Evidence

### 1. How the Alias Context persists its content

Answer: **(a) a dispatched SAVE verb**, claimed by the FFCMD base and routed
through the CR-CH-071 "editable-Context seam" to an alias-store write. It is NOT
a panel-only direct write.

Production chain (all paths in the worktree):

- Front door: `shell/dispatch.rs::dispatch_command_string`
  (`crates/ff-desktop/src/shell/dispatch.rs:46`). Runs `=`-strip, verb->root
  rewrite, prelude, then `ff_command::resolve_target`, then
  `run_command_ladder`.
- `resolve_target` DECLINES SAVE: `command_config/mod.rs::builtin_workspace_target_for`
  does not match "SAVE" as a classifier verb. SAVE appears in the FFCMD root
  list (`command_config/mod.rs:266`) only for root-enumeration / no-shadow
  validation, with an explicit comment that it is "NOT a classifier verb" and is
  "handled by the ladder's editable-Context seam (`try_commands_a`)"
  (`crates/ff-desktop/src/command_config/mod.rs:257-268`). So `resolve_target`
  returns `Err` and SAVE falls through to the ladder.
- Ladder active-env gate: `shell/commands.rs::run_command_ladder`
  (`crates/ff-desktop/src/shell/commands.rs:131`). Before `try_commands_a` it
  asks `self.environments.is_ffedit_active(..)`
  (`crates/ff-desktop/src/shell/commands.rs:~165`). In the strip this is
  hard-wired `false` (see section 3), so no environment claims SAVE first and the
  ladder proceeds to `try_commands_a`.
- SAVE arm: `shell/commands_ladder_a.rs::try_commands_a`
  (`crates/ff-desktop/src/shell/commands_ladder_a.rs:225`):
  ```
  if upper == "SAVE" {
      if self.editable_context_kind().is_some() {
          let _ = self.editable_persist();   // persist + STAY
      }
      self.open_error = None;
      return true;
  }
  ```
- Seam: `shell/editable_context.rs`
  (`crates/ff-desktop/src/shell/editable_context.rs`).
  `editable_context_kind()` maps `KindTag::AliasEditor -> Some(EditableKind::AliasEditor)`
  (`:51-56`); `editable_persist()` routes `AliasEditor -> self.alias_editor_persist()`
  (`:76-81`).
- Alias write: `shell/alias_editor.rs`
  (`crates/ff-desktop/src/shell/alias_editor.rs`).
  `alias_editor_persist()` (`:153-156`) calls `save_user_aliases()` (`:121-145`),
  which validates the working rows (`validate_user_alias_rows`), writes the
  per-locale override files via `write_override_rows`, and refreshes the live
  resolver via `refresh_alias_store_in_place()` (`:180-205`). The shipped seed is
  never written; only `<base>/<locale>/<ENV>.toml`.

Note on the Alias panel command line: a "SAVE" typed inside the panel's own
`Command ===>` line is NOT a private interpreter. `apply_alias_command_line`
(`shell/alias_editor_line_commands.rs:83`) matches only FILTER/RESET and in its
`_ =>` arm forwards the original text to `self.dispatch_command_string(..)`
(`:103-109`) -- i.e. the exact same front door, so the panel line and a global
SAVE converge on one path (CR-CH-071 Req 20a.10).

### 2. Does a user typing SAVE on the Alias Context persist content now?

**Yes.** Following `resolve_target` -> (declines) -> ladder -> active-env gate
(FFCMD base, never claims) -> `try_commands_a` SAVE arm ->
`editable_context_kind()` = `Some(AliasEditor)` -> `editable_persist()` ->
`alias_editor_persist()` -> `save_user_aliases()`: the working rows are validated,
written to `<base>/<locale>/<ENV>.toml`, and the live `LocaleAliasStore` is
re-seeded in place so the alias resolves without restart. The Context STAYS
(SAVE), and END additionally leaves after a successful save. Exact
functions/files are cited in section 1.

### 3. Did the strip (414ba84) or test-fixes REMOVE/sever any SAVE binding?

`git show --stat 414ba84` scoped to the Alias SAVE production files
(`dispatch.rs`, `commands_ladder_a.rs`, `editable_context.rs`, `alias_editor.rs`,
`command_config/mod.rs`) returns **empty** -- the strip did not modify any of
them. The production SAVE path was left untouched.

What the strip DID change that is SAVE-adjacent (from the commit stat):

- Deleted `shell/dispatch_ffedit.rs` (332 lines), `shell/environment_ffedit.rs`
  (386 lines), `shell/mainframe_backend.rs` (89 lines) -- the FFEDIT editor
  environment, its verb bodies, and the physical backend SAVE seam. These are the
  *editor-buffer* SAVE path (FFEDIT owned editor SAVE-into-document), correctly
  parked with the editor domain. This is NOT the Alias SAVE path.
- Heavily rewrote `shell/environment_registry.rs` (239 lines changed):
  `RegisteredEnv` now has only the `FfCmdBase` variant; `with_builtins()` registers
  only FFCMD; `is_ffedit_active()` is hard-wired to return `false`
  (`environment_registry.rs`, `is_ffedit_active` body). Effect on Alias SAVE:
  *beneficial / neutral* -- because FFEDIT no longer claims, SAVE now reaches
  `try_commands_a` unconditionally on the Alias Context (the active-wins
  pre-emption by FFEDIT, which only ever applied to editor *buffers*, is gone).
  Crucially, SAVE/CANCEL/REFRESH remain FFCMD roots: `environment_registry.rs`
  test `ffcmd_roots_contain_save_cancel_refresh` is retained and still asserts
  `contains_root(FFCMD, "SAVE")`.
- Deleted `shell/tests_crch071.rs` (470 lines) -- the CR-CH-071 editable-Context
  seam behaviour tests. This is the lost coverage.

Before/after of the SAVE binding the strip touched:

- BEFORE (main 4e3db77): two SAVE owners -- FFEDIT claimed SAVE on an editor
  buffer (active-wins); the FFCMD base claimed SAVE on a non-buffer editable
  Context (the Alias Editor) via the seam.
- AFTER (strip 414ba84): FFEDIT and the buffer SAVE are parked; the FFCMD-base
  seam SAVE (the Alias path) is UNCHANGED and now the sole SAVE owner. The Alias
  binding was neither removed nor severed.

Lost Alias SAVE coverage in the deleted `tests_crch071.rs` (names recovered from
`git show 4e3db77:.../tests_crch071.rs`):

- `save_persists_and_stays_on_alias_editor` -- drove `dispatch_command_string("SAVE")`
  on the Alias Editor and asserted the override file was written + resolver
  refreshed + Context stayed. **This is the only test that exercised the
  globally-typed bare-SAVE front door on the Alias Context.**
- `end_persists_then_leaves_on_alias_editor`,
  `end_on_validation_failure_stays_and_leaves_store_unchanged`,
  `cancel_discards_and_leaves_without_saving`,
  `return_discards_and_leaves_without_saving`,
  `refresh_discards_edits_reloads_last_saved_and_stays`,
  `save_and_refresh_are_noops_on_non_editable_context`,
  `typed_command_line_end_matches_global_end`,
  `typed_command_line_cancel_and_refresh_route_through_front_door`,
  `load_skips_user_rows_shadowing_save_cancel_refresh`,
  `exit_family_remains_app_exit_not_context_save_then_leave`.
- `ffedit_buffer_save_active_wins_boundary_is_unchanged` -- correctly removed
  (it validated the now-parked FFEDIT buffer SAVE).

The 25 tests in the owner's "parked FFEDIT" removal named around SAVE
(`editor_save_on_dirty_untitled_errors_and_stays`,
`ffedit_save_routes_dirty_host_file_write_through_owning_env`,
`host_bound_save_is_behaviour_preserving`,
`save_routes_through_dispatch_to_environment_...`) are all
**editor-buffer / FFEDIT** SAVE tests (they reference the owning environment,
host-file write-through, and FFEDIT dispatch) -- correctly parked. None of them
is the Alias SAVE path.

Surviving Alias SAVE coverage (NOT lost), in `tests_alias_editor.rs`:

- Local `save()` helper (`tests_alias_editor.rs:~57`) does
  `apply_alias_editor_action(AliasEditorAction::CommandLine("SAVE"))`, which
  forwards through `apply_alias_command_line` `_ =>` to
  `dispatch_command_string("SAVE")` -- the SAME front-door seam path. Tests using
  it: `save_writes_file_and_live_resolver_resolves_new_alias`,
  `refresh_keeps_base_layer_and_adds_user_layer`,
  `user_rows_route_into_their_target_environment`,
  `save_validation_failure_leaves_file_unchanged`,
  `save_makes_menu_render_surface_reflect_the_edited_alias`,
  `save_writes_only_user_override_not_shipped_seed`,
  `parameterised_alias_authored_in_editor_expands_end_to_end`.
- Row-level validation unit tests in `alias_editor.rs` (`validate_user_alias_rows`
  suite) are retained.

So the dispatched Alias SAVE *write* behaviour is still tested via the panel
command-line entry; what is no longer tested is the *bare globally-typed* SAVE /
END / CANCEL / RETURN / REFRESH verbs landing on the Alias Context through
`dispatch_command_string` directly, and the active-wins boundary.

### 4. Conclusion

- **Alias save: WORKING.** Proven by the intact, unmodified chain in section 1
  (none of the five production files was touched by `414ba84`), the retained
  FFCMD SAVE root (`environment_registry.rs::ffcmd_roots_contain_save_cancel_refresh`),
  and the surviving write tests in `tests_alias_editor.rs`.
- **Settings/Config and Theme save do NOT share this seam yet.**
  `editable_context.rs::editable_context_kind()` wires ONLY
  `KindTag::AliasEditor`; the sibling config editors (Settings/ConfigPanel,
  Menus, Theme, Keys, Kinds) are explicitly the fast-follow set and return `None`
  (fall through to plain navigation, Req 20a.14). Each of those Contexts persists
  through its own panel action (e.g. Theme via `apply_theme_editor_action`,
  ConfigPanel via its commit path), independent of the FFCMD SAVE verb. The strip
  did not change that; they were never on the FFCMD SAVE seam. So the owner's
  statement that "SAVE was in FFCMD specifically for the Alias Context" matches
  the code: Alias is the sole v1 seam consumer.
- **No production regression; no production fix needed.** This is consistent with
  the owner's guiding principle -- the FFCMD SAVE root still has a real binding/
  function in the core (the Alias Context), so it correctly stays in FFCMD. It is
  not an orphaned verb.

## Recommendation (test-only; nothing to implement in production)

Restore the Alias-relevant subset of the deleted `tests_crch071.rs` so the
retained core SAVE behaviour is guarded. Minimal set, re-pointed at the current
per-locale override model used by `tests_alias_editor.rs`:

- `save_persists_and_stays_on_alias_editor` -- the globally-typed
  `dispatch_command_string("SAVE")` proof (the one genuine coverage gap vs the
  surviving panel-line `save()` helper).
- `end_persists_then_leaves_on_alias_editor` and
  `end_on_validation_failure_stays_and_leaves_store_unchanged`.
- `cancel_discards_and_leaves_without_saving`,
  `return_discards_and_leaves_without_saving`,
  `refresh_discards_edits_reloads_last_saved_and_stays`.
- `save_and_refresh_are_noops_on_non_editable_context` (POM).

DROP from the restore (correctly parked with FFEDIT, no core binding):
`ffedit_buffer_save_active_wins_boundary_is_unchanged` and anything asserting the
FFEDIT buffer SAVE / `is_store_affecting_verb` path.

Do NOT add any FFCMD verb or production code: the SAVE binding already exists and
functions. This is purely re-establishing regression guards for an existing core
Context.
