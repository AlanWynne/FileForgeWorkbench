# B081 Theme-Editor Test-Isolation Fix -- Review

Verdict: APPROVED. Iteration 1 (no prior review). Test-infra / refactor-class fix;
no requirements gate required (no observable product behaviour change).

## Scope reviewed
- Branch diff `main...b081-theme-isolation` (one file changed:
  `crates/ff-desktop/src/shell/i18n_bootstrap.rs`, +32/-5).
- Implementer summary (`b081-fix-summary.md`) and findings (`findings.md`).
- `docs/PATTERNS-AND-LESSONS.md` A1 (variant 3, thread-scope) and C3 (test-only
  thread-local vs production process-global boundary).
- Unchanged production seam `crates/ff-i18n/src/seam.rs` (to establish the
  consequence of the change).
- Recorded verification logs under the worktree `tools/logs/`.

## Criteria assessment

1. A1-conformant (thread-scope via existing seam): PASS. The fix installs the
   test catalogue only through the existing `ff_i18n::set_test_catalogue`
   thread-local seam (seam.rs:48), which pre-dates this branch (CR-CH-071). No
   new ad-hoc isolation mechanism was introduced. This is A1 variant 3 applied to
   close the last two leaks into the same i18n global the A1 entry already
   describes.

2. C3 respected (production process-global unchanged): PASS. `seam.rs` is NOT in
   the diff. Production still resolves via the single process-global
   `RwLock<Option<Catalogue>>` set by `set_catalogue` (seam.rs:77); the
   thread-local `TEST_OVERRIDE` is read override-first but is set only by test
   code. No production caller sets the thread-local. The i18n production seam's
   process-global semantics are intact.

3. `install_catalogue_for_tests` no longer clobbers via the process-global: PASS.
   The prior `None => install_catalogue_for_locale(...)` global-fallback branch
   is removed (diff). It is now `if let Some(catalogue) = ... { set_test_catalogue
   (catalogue) }` -- always thread-local, and when the shipped root cannot be
   resolved the thread-local is simply left unset so lookups degrade to the key
   fallback on that thread exactly as production, without touching the global.

4. Offending locale test no longer leaves the global mutated: PASS.
   `locale_change_applies_afrikaans_end_to_end_through_reload_callback`
   legitimately drives the production global seam (it must, to test
   install/reload), so it is NOT converted to the thread-local. Instead it
   restores English via `install_catalogue_for_locale(ff_i18n::IDENTITY_BASE_TAG)`
   after its assertions and asserts the restore ("Settings"). It runs under
   `GLOBAL_CATALOGUE_SEAM_LOCK`, shared only with the other global-path test.

5. No other test still races the i18n process-global unrestored: PASS. Audit
   confirmed by grep of `set_catalogue` / `install_catalogue_for_locale` /
   `reload_active_alias_store_for_locale` across ff-desktop:
   - The only test writers of the i18n process-global are
     `install_catalogue_for_tests` (fallback now removed),
     `locale_change_...` (now restores), and
     `install_catalogue_for_locale_en_installs_the_seam` (installs and leaves
     English, under the lock -- harmless). All other test helpers use the
     per-thread `set_test_catalogue`.
   - `reload_active_alias_store_for_locale` mutates a SEPARATE process-global
     (`ACTIVE_ALIAS_STORE`), is called only from `main.rs` (production), and is
     not exercised by any test. It is not on the B081 racing path (the Save
     button label resolves through `ff_i18n::t`, not the alias store), so it is
     correctly out of scope.
   - `crates/ff-i18n/src/tests.rs` uses `set_catalogue` but is a separate test
     binary that cannot race the ff-desktop bin suite.

6. No product/dispatch/persistence/theme-editor change; test-infra only: PASS.
   Only `i18n_bootstrap.rs` changed. Both edits are inside a `#[cfg(test)]`
   function (`install_catalogue_for_tests`) and a `#[cfg(test)]` test. No
   framework seam, dispatch, persistence, or theme-editor code touched
   (framework-conformance.md satisfied).

7. Theme test green REPEATEDLY (recorded evidence): PASS. Four independent plain
   threaded `cargo test -p ff-desktop --bin ffwb` runs recorded in
   `tools/logs/b081-run1..4.log`:
   each shows `full_shell_theme_editor_type_name_and_save_creates_user_theme ...
   ok` and `test result: ok. 1162 passed; 0 failed`. The test's ordinal position
   shifts between runs (line 753 / 751 / 748 / 739), consistent with genuine
   re-executed threaded runs rather than a cached pass. The race is proven closed
   by repetition.

8. clippy -D warnings clean; fmt clean: PASS (exceeds the summary's stated
   owner-pending status). The summary said clippy/fmt were PENDING OWNER, but the
   logs are in fact present and clean:
   - `tools/logs/b081-clippy.log` ends `Finished dev profile ... in 2m 44s` with
     no `warning`/`error` lines -- clippy with `-D warnings` compiled to
     completion clean.
   - `tools/logs/b081-fmt.log` is 0 bytes, which is the expected output of
     `cargo fmt --check` on clean code (no diff printed, exit 0).
   Verified by effect (reading the logs), not by exit code.

9. Touched .rs under 400 non-test lines; plain ASCII: PASS. `mod tests` begins at
   line 246, so non-test code is ~245 lines. A non-ASCII scan of the file
   returned no matches (the Afrikaans "Stoor"/"Instellings" literals are plain
   ASCII).

## Non-blocking observations (not findings)
- The English restore in `locale_change_...` is a plain statement, not an RAII
  Drop guard, so an earlier-assertion panic would leave the global in Afrikaans.
  This is harmless here: Change 1 makes no full-shell harness test read the
  global, and the only co-writer recovers the poisoned `GLOBAL_CATALOGUE_SEAM_LOCK`
  via `into_inner()`. The implementer documents this as belt-and-braces. No
  change required.
- There is a pre-existing duplicated doc-comment block above
  `install_catalogue_for_tests` (two stacked `///` blocks). It predates this
  branch (outside the diff hunks) and is a cosmetic nit, out of scope.

## Conclusion
All review criteria pass. The fix is a faithful application of the documented A1
variant-3 pattern within the C3 boundary, closes both i18n-global leaks, is
test-infra only, and is backed by repeated green threaded runs plus clean
clippy/fmt logs. No blocking findings.
