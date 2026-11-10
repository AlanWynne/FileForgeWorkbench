# CR-CH-071 Review -- ISPF Save/End/Cancel/Return/Refresh for editable Contexts

Branch: `crch071-save-cancel-refresh`
Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\crch071-save-cancel-refresh`
Range reviewed: `42d6db8..HEAD` (single commit `9f3676d`).
Verdict: APPROVED (no blocking findings).

## Scope and evidence read

- Branch log + full diff captured to `tools/logs/crch071-review-log.log` and
  `tools/logs/crch071-review-diff.log` and read back.
- Source read directly in the worktree: `shell/commands_ladder_a.rs`,
  `shell/editable_context.rs`, `shell/alias_editor.rs`,
  `shell/alias_editor_line_commands.rs`, `command_config/mod.rs`,
  `shell/environment_registry.rs` (test), `shell/tests_crch071.rs`,
  `shell/tests_alias_editor.rs`.
- Requirements: command-environments Requirement 20a.1-20a.14; command-configurator
  Req 5.9/5.14 (amended); plan `crch071-impl-plan.md`; summary
  `crch071-impl-summary.md`; framework-conformance.md.
- Verification evidence read from the impl summary / commit (NOT re-run):
  `cargo check` clean, `cargo clippy -- -D warnings` clean, `cargo fmt --check`
  clean, `cargo test tests_crch071` 13/13 pass, nextest 4/4 + 13/13 green.

## Acceptance checklist

1. SAVE/CANCEL/REFRESH are FFCMD-base verbs delegating to the focused-editable
   seam, base owns no working copy -- MET. `try_commands_a` adds SAVE/REFRESH/
   CANCEL arms that call `editable_context_kind()` + `editable_persist/discard/
   reload` on the new `shell/editable_context.rs` seam, which routes by active
   `KindTag` to the owning Context. The base itself holds no working copy.

2. FFEDIT buffer SAVE byte-identical + active-wins intact; existing FFEDIT SAVE
   tests not weakened -- MET. No change to `environment_ffedit` production logic;
   `is_store_affecting_verb("SAVE")` still true; editor buffers are FFEDIT-active
   so FFEDIT claims SAVE before the FFCMD ladder. Regression test
   `ffedit_buffer_save_active_wins_boundary_is_unchanged` asserts this. No
   existing FFEDIT SAVE assertion relaxed or removed.

3. Semantics matrix -- MET and matches Req 20a.1-20a.4 verbatim:
   SAVE=persist+stay; END (and F3)=persist then `nav_end` on ok, stay+inline
   error + store-unchanged on validation failure; CANCEL=discard+`nav_end`;
   RETURN=discard+`nav_return`; REFRESH=discard+reload+stay (no write). Each arm
   has a proving test in `tests_crch071.rs`.

4. Alias Editor private SAVE/CANCEL/END interpreter retired -- one dispatch path
   -- MET. `apply_alias_command_line` removes the private `"SAVE"` and
   `"CANCEL" | "END"` arms; the fall-through hands the original command text to
   `dispatch_command_string`, so the typed `Command ===>` line and F3/global END
   reach the SAME handler. `A::CommandLine(text) => apply_alias_command_line`
   confirms the panel action routes here. Test
   `typed_command_line_end_matches_global_end` asserts identical results.

5. No-shadow REJECTS a user alias named SAVE/CANCEL/REFRESH (validate + load-time
   skip-with-WARN) -- MET. Automatic via the three verbs joining
   `BUILTIN_FFCMD_LADDER_VERB_ROOTS` -> `builtin_ffcmd_roots()`, which both the
   save-side `validate_user_alias_rows` and the load-side overlay already read.
   Locked by `validate_rejects_save_cancel_refresh_shadow` (save) and
   `load_skips_user_rows_shadowing_save_cancel_refresh` (load).

6. EXIT still triggers app-exit; app-exit not broken -- MET. `try_exit_family`
   is untouched; END (not EXIT) is the save-then-leave verb. Test
   `exit_family_remains_app_exit_not_context_save_then_leave`.

7. `root_commands("FFCMD")` enumeration updated with SAVE/CANCEL/REFRESH -- MET.
   Appended to `BUILTIN_FFCMD_LADDER_VERB_ROOTS`; test
   `ffcmd_roots_contain_save_cancel_refresh` asserts `root_commands` + case
   -insensitive `contains_root`. The disjointness drift guard still holds (these
   are not classifier verbs).

8. No collateral damage / framework conformance -- MET. No second dispatcher,
   nav stack, or persistence format. The seam is a dispatch helper on
   `WorkbenchShell` keyed by `KindTag`, mirroring the existing
   `apply_<editor>_action` pattern; `EditableKind` is an internal `pub(super)`
   enum, not a public framework type. END/RETURN continue through `nav_end`/
   `nav_return`. CR-CH-066/067/068/069/070 paths untouched (the only shared edit
   is appending to the FFCMD root list, which is additive). Sibling editors
   explicitly return `None` (guarded by
   `menus_editor_is_not_wired_into_the_seam_in_v1`).

9. Standards -- MET. All changed source files are under 400 non-test lines
   (`editable_context.rs` ~103; `commands_ladder_a.rs` ~330, no test module;
   `alias_editor.rs` non-test ends ~287; `alias_editor_line_commands.rs` non-test
   well under limit). No `unwrap`/`expect` in non-test production code
   (`let _ = ...`, `match`, `if let` only); the `.unwrap()`/`.expect()` calls are
   all inside `#[cfg(test)]` modules, which is allowed. New public items derive
   `Debug`.

10. CR-CH-071 TCR rows / tasks -- MET with a justified deviation. The impl summary
    records that `docs/quality/TCR.md` is a gate-REGENERATED crate-organised
    report with NO CR-CH-071 rows to flip (plan Task 10 assumed gate-seeded rows
    that do not exist; the owner has confirmed TCR is regenerated every gate). The
    diff touches ONLY `docs/specs/command-environments/tasks.md` (flipping 32.x,
    33.x, 35.1-35.7 to `[x]`, leaving 34.x fast-follow and 35.8 owner-gate `[ ]`).
    No other CR's table is touched. The per-criterion coverage record is the
    command-environments tasks coverage table, which is the repo's authoritative
    non-gate record. Acceptable.

## Observations (non-blocking, not filed as findings)

- The new section separators in `commands_ladder_a.rs` use U+2500 box-drawing
  (`-- SAVE / CANCEL / REFRESH --` style) matching the file's pre-existing END/
  RETURN separators. documentation-charset.md asks source separators to be plain
  ASCII `// === ... ===`. This is a pre-existing file convention the coder
  followed, cleanup-tier only, and has no correctness/security/data impact. Not
  a blocker; mentioned for a future ASCII sweep of this file.

## Verification

Per the step's discipline the recorded coder evidence was read, not re-run:
check/clippy/fmt clean, `tests_crch071` 13/13 pass, and the four i18n-race tests
plus the module green under the canonical nextest harness. The summary's root
-cause analysis of the four threaded-`cargo test` failures (a pre-existing
process-global `ff_i18n::set_catalogue` isolation race, not a CR-CH-071
regression; green under nextest, the owner's authoritative gate runner) is
consistent with the diff -- CR-CH-071 adds no code touching `set_catalogue` or
localised-surface derivation. Option A (a catalogue test-lock) is correctly
deferred as a separate CR. No specific doubt justified re-running suites; narrow
spot-checks (line counts, command-line routing) corroborated the summary.

## Conclusion

All acceptance items are met; the one deviation (TCR) is justified and does not
touch other CRs; the single observation is cleanup-tier. No blocking findings.
Verdict: APPROVED. The owner's full nextest `cargo gate` remains the pre-merge
authority per gate-policy.md.
