# B081 Theme-Editor Test-Isolation Flake -- Fix Summary

Iteration 1 (no prior `b081-review.json` existed). Test-infra / refactor-class
fix: NO observable product behaviour change, so NO requirements gate (per
workflow.md "what does NOT need the gate"). Applies the already-documented A1
test-isolation pattern (variant 3, thread-scope) from
docs/PATTERNS-AND-LESSONS.md.

## Root cause (confirmed from source, matches findings.md)

`shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme`
does `harness.get_by_label("Save")`. The Save button's accessible name is
`ff_i18n::t("theme-editor-btn-save")` -> "Save" (en) / "Stoor" (af), resolved
through the PROCESS-GLOBAL i18n catalogue.

The suite already shields most tests with a per-thread catalogue override
(`ff_i18n::set_test_catalogue`, installed by `install_catalogue_for_tests`), but
there were TWO holes that let the process-global leak across threads under plain
threaded `cargo test`:

1. `install_catalogue_for_tests` had a FALLBACK branch: when the shipped root
   could not be resolved it called `install_catalogue_for_locale` -> the
   process-global `ff_i18n::set_catalogue`. That fallback was the one path by
   which a full-shell harness test depended on the shared global instead of its
   own thread-local, exposing it to another thread's global writes.
2. `locale_change_applies_afrikaans_end_to_end_through_reload_callback`
   legitimately drives the PRODUCTION global seam (`install_catalogue` with
   `ui.locale="af"` -> `set_catalogue(af)`) and never restored the global, so it
   left the process-global in Afrikaans permanently. A theme test on another
   thread reading the global then saw "Stoor" and `get_by_label("Save")` found
   zero nodes and panicked.

Pass-isolated / pass-under-nextest / flake-under-threaded-cargo-test is the exact
fingerprint of a shared-global race (A1 diagnostic tell).

## The A1 variant applied -- variant 3 (thread-scope), as documented

The global sits on a hot common-fixture path written by a large share of the
suite, and Rust's test runner is one-thread-per-test, so variant 3 (thread-local
override) is the correct choice (not a suite-wide lock). The existing
`set_test_catalogue` / `clear_test_catalogue` seam was already present
(CR-CH-071 follow-up); this fix only closes the two remaining leaks into the
global.

### Change 1 -- make the per-thread shield airtight
`crates/ff-desktop/src/shell/i18n_bootstrap.rs::install_catalogue_for_tests`:
dropped the process-global fallback. It now installs ONLY via
`set_test_catalogue` (thread-local). When the shipped root cannot be resolved it
leaves the thread-local UNSET, so lookups degrade to the key fallback on that
thread exactly as production does -- without writing the global and without
exposing the thread to another thread's global writes. (In the test binary the
dev-tree marker walk always resolves the root, so this is the dead-code hole
being closed, not a behaviour the suite relies on.)

### Change 2 -- stop the one global-mutating test leaving residue
`crates/ff-desktop/src/shell/i18n_bootstrap.rs::locale_change_applies_afrikaans_end_to_end_through_reload_callback`:
this test MUST write the global because it is specifically testing the production
process-global install/reload seam (CONFIRMED from source: it calls
`install_catalogue(&config)` and the real `on_locale_reload` callback, not the
thread-local helper). So it is NOT converted to the thread-local; instead it
RESTORES the global to the English Identity_Base
(`install_catalogue_for_locale(IDENTITY_BASE_TAG)`) after its assertions and
asserts the restore. With Change 1 in place no full-shell harness test reads the
global any more, so this is defence-in-depth, but it keeps the global clean for
the other global-path test that shares `GLOBAL_CATALOGUE_SEAM_LOCK`.

Brief `// A1/B081` why-notes were added at both changed sites.

## C3 boundary respected

The thread-local override is TEST-ONLY. No production code changed: production
still uses the single process-global `set_catalogue` (one active locale per
process, swapped atomically at startup / on locale reload). The i18n production
seam's process-global semantics are untouched, the theme editor is untouched, and
no dispatch / persistence / framework behaviour changed
(framework-conformance.md satisfied).

## Other process-global writers audited

Grep of `set_catalogue` / `install_catalogue_for_locale` /
`reload_active_alias_store_for_locale` across ff-desktop and ff-i18n:

- ff-desktop test binary writers of the process-global: exactly
  `install_catalogue_for_tests` (fallback -- now removed),
  `locale_change_applies_afrikaans_end_to_end_through_reload_callback` (now
  restores), and `install_catalogue_for_locale_en_installs_the_seam` (writes
  English under `GLOBAL_CATALOGUE_SEAM_LOCK`, leaves English -- harmless, no
  change needed). No other test writes the process-global.
- All other test helpers (tests_common.rs `install_verb_find_soek_catalogue` /
  `install_find_menu_localised_catalogue`, environment.rs, alias_reader.rs) use
  the per-thread `set_test_catalogue`. Safe.
- `crates/ff-i18n/src/tests.rs` uses `set_catalogue`, but that is a SEPARATE test
  binary (ff-i18n lib tests) that cannot race the ff-desktop bin suite. Out of
  scope.
- `main.rs` / `alias_reader.rs` production callers are unchanged (production
  model).

So the whole class is closed, not just the one named test.

## Files changed + exact by-name staging list

Only one source file changed:

```
git -C c:\workspace\apps\FileForgeWorkbench\.worktrees\b081-theme-isolation add crates/ff-desktop/src/shell/i18n_bootstrap.rs
```

(Do NOT stage tools/logs or TERMINAL_DEFECTS.md.)

Commit message (no ';'):
```
fix(test): close B081 i18n-catalogue test-isolation leak (A1 thread-scope)
```

## Change-log / TCR note

Not required. This is a test-infra-only fix with no observable product behaviour
change and no new/changed acceptance criteria (theme-and-appearance Req
20.5/20.13 already exist). Precedent: the earlier i18n per-thread fix rode along
in CR-CH-071 without its own CR id, so a standalone test-isolation fix is a plain
commit. No standalone CR id invented.

## Verification status

The terminal entered repeated dead episodes this session (recorded in
TERMINAL_DEFECTS.md); it recovered intermittently, which was enough to capture
the critical repeated-race evidence but NOT the clippy/fmt result lines.

### Repeated threaded-run race verification -- DONE, PROVEN GREEN (A1 repetition)

Four independent plain-threaded `cargo test -p ff-desktop --bin ffwb` runs, each
read back by effect from its log:

| Run | Log | Result | Theme test |
|-----|-----|--------|------------|
| 1 | b081-run1.log | `test result: ok. 1162 passed; 0 failed` | `... ok` |
| 2 | b081-run2.log | `test result: ok. 1162 passed; 0 failed` | `... ok` |
| 3 | b081-run3.log | `test result: ok. 1162 passed; 0 failed` | `... ok` |
| 4 | b081-run4.log | `test result: ok. 1162 passed; 0 failed` | `... ok` |

`full_shell_theme_editor_type_name_and_save_creates_user_theme` passed in EVERY
run. The race is proven closed by repetition (not a single pass). `cargo test`
also compiled ff-desktop + its test harness clean, so the code COMPILES.

### clippy / fmt -- PENDING OWNER (terminal re-wedged on these commands)

The scoped `cargo clippy -p ff-desktop -- -D warnings` and
`cargo fmt -p ff-desktop -- --check` could not be captured: each invocation hit a
dead episode and wrote no log. The change is low-risk for lint/format (a doc
comment, a `match` -> `if let Some`, and a restore call + assertion, all
idiomatic), and the crate compiles clean under the test runs. Owner, please run:
```
C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo clippy -p ff-desktop --manifest-path c:\workspace\apps\FileForgeWorkbench\.worktrees\b081-theme-isolation\Cargo.toml -- -D warnings *> c:\workspace\apps\FileForgeWorkbench\.worktrees\b081-theme-isolation\tools\logs\b081-clippy.log"
```
```
C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo fmt -p ff-desktop --manifest-path c:\workspace\apps\FileForgeWorkbench\.worktrees\b081-theme-isolation\Cargo.toml -- --check *> c:\workspace\apps\FileForgeWorkbench\.worktrees\b081-theme-isolation\tools\logs\b081-fmt.log"
```
(ff-i18n was NOT touched, so its scoped trio is not needed. Do NOT run the full
cargo gate -- owner does that before merge.)

## Commit status -- COMMITTED

Committed on branch `b081-theme-isolation` in the worktree:
`f0ea95e fix(test): close B081 i18n-catalogue test-isolation leak (A1 thread-scope)`
-- 1 file changed, 32 insertions(+), 5 deletions(-). Staged BY NAME
(`crates/ff-desktop/src/shell/i18n_bootstrap.rs` only); tools/logs and
TERMINAL_DEFECTS.md were NOT staged. No merge / push / PR.

## Lessons-file update warranted?

No new lesson. This fix is a direct application of the EXISTING A1 variant-3
pattern and the C3 boundary already documented in docs/PATTERNS-AND-LESSONS.md;
it closes the last two leaks into the same i18n global the A1 entry already
describes. If anything, it reinforces the existing entry rather than adding a new
one, so no PATTERNS-AND-LESSONS.md edit is needed.
