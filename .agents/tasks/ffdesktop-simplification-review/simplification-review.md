# ff-desktop safe-simplification refactor (Change A delete-dead-code + Change C tests.rs split)

This working-tree change executes two of the three "safe, behaviour-preserving"
simplifications from the ff-desktop review: Change A removes provably dead code
(the `panel_layout` module plus three uncalled `TabManager` tab-openers), and
Change C splits the 10,999-line `shell/tests.rs` monolith into nine
feature-area sibling test modules. Change B (notification mpsc-channel removal)
was deliberately NOT performed this run -- the owner deferred it to the
requirements gate because dropping the channel would invalidate documented
notification-system Requirement 1.1, so it is not purely behaviour-preserving.
The implementer's note records the scoped-check evidence (fmt/check/clippy
clean, 454 shell tests intact under nextest), which I relied on per the review
instructions rather than re-running the suites.

Watch for: nothing blocking. One cosmetic residue -- two stale comments in
`tab_state.rs` still name the now-deleted `open_theme_editor_tab` /
`open_menus_editor_tab` openers (confirmed; comment text only, no code
reference, no compile impact). The task brief listed Change B and a fourth
deletion (`transform_active_pom_tab`) that the implementer correctly did NOT
make; both omissions are right, not defects (confirmed).

**Verdict**: APPROVED

## High-level view

Change A is a clean dead-code deletion. `panel_layout.rs` and its `mod`
declaration are gone, and the three `TabManager` openers removed
(`open_theme_editor_tab`, `open_menus_editor_tab`, `open_menu_workspace_here`)
each carried `#[allow(dead_code)]` and have no caller anywhere, including tests.
The one opener the brief also listed for deletion, `transform_active_pom_tab`,
was correctly kept: it has live callers in `tests_focus.rs` and `tests_nav.rs`,
so deleting it would have broken the build. That STOP was the right call.

Change B is absent from the diff, which matches the implementer's stated owner
decision to defer it. The shared `Arc<Mutex<NotificationQueue>>` sink, the mpsc
channel (`notification_rx`/`notification_tx`), `notification_sender()`, the
per-frame drain in `shell/update.rs`, and both notification tests are all still
present and unmodified. Nothing that posts notifications is broken, because
nothing in the notification path was touched. The brief asked me to verify B's
removal; since B was intentionally not done, there is nothing to reject -- it is
a documented scope change, out of this run's blocking scope.

Change C preserves the test suite exactly. I independently reconstructed the
pre-split test-name set from `git HEAD:shell/tests.rs` and compared it to the
union across the nine new files: 454 before, 454 after, zero missing, zero
added, zero duplicate names. Per-file counts (83+87+96+71+44+16+25+32 = 454,
with `tests_common.rs` holding 0 tests and only shared helpers) match the
implementer's recorded figures. No test was renamed, deleted, weakened, or
`#[ignore]`d (no `#[ignore]` exists before or after). The modules are direct
children of `shell`, so the heavy `super::` usage inside moved tests still
resolves unchanged, and the shared helpers (`make_shell`, `make_dispatch`,
`harness_shell`, `is_shell_command`, and the rest) were migrated to
`tests_common` as `pub(crate)`.

Character and size rules hold for what this diff touched. The nine new files
are pure ASCII (the implementer converted the old box-drawing separators to
`-`), and the lines added to `shell/mod.rs` are ASCII. Pre-existing non-ASCII
comments elsewhere in `mod.rs` and `tab_manager.rs` are untouched by this diff
and therefore out of scope. The 400-line source rule explicitly excludes
`#[cfg(test)]` modules, so the test files do not trip it.

<details>
<summary>Issues (1)</summary>

1. **Stale opener names in tab_state.rs comments** (non-blocking, confirmed) --
   `tab_state.rs` doc comments on the `theme_editor` / `menus_editor`
   constructors still say "only used by the retained open_theme_editor_tab /
   open_menus_editor_tab", but those openers were deleted in this change. Comment
   text only, no code reference, no compile or behaviour impact. Optional
   tidy-up: reword to note the constructors are now retained under
   `#[allow(dead_code)]` with no caller.

</details>

<details>
<summary>Details</summary>

## Change A -- dead-code deletion with the right STOP

The diff removes `mod panel_layout;` from `main.rs`, deletes `panel_layout.rs`
(157 lines), and removes three `TabManager` methods. A workspace grep for
`open_theme_editor_tab|open_menus_editor_tab|open_menu_workspace_here|panel_layout`
returns only two hits, both inside `tab_state.rs` comments -- no live code path
references any deleted item (confirmed). Each deleted method previously carried
`#[allow(dead_code)]` and a CR-CH-022 note explaining it was superseded by the
in-place `navigate_to` path, so the deletions remove genuinely orphaned code
rather than anything on the live command/navigation flow.

The brief's deletion list also named `transform_active_pom_tab`, but the
implementer kept it, and that is correct: a grep shows live callers in
`shell/tests_focus.rs` (three sites) and `shell/tests_nav.rs`, exercising the
POM-tab in-place transform primitive. Deleting it would have broken those
tests. The STOP is sound and matches the note.

The only residue is cosmetic: `tab_state.rs` still has two comments naming the
deleted openers as the "only user" of the `theme_editor` / `menus_editor`
constructors. Those constructors remain under `#[allow(dead_code)]`
(`theme_editor` is still used by a test; `menus_editor` is now caller-less),
so no new `dead_code` warning results, consistent with the clean clippy run the
implementer recorded. Pure documentation drift, non-blocking.

## Change B -- not performed, and correctly so

There is no notification-related change in the diff. A grep for
`notification_rx|notification_tx|notification_sender` finds the channel still
constructed in `shell/mod.rs` (`sync_channel::<Notification>(64)`), the fields
still stored, `notification_sender()` still returning a `NotificationSender`,
the per-frame `try_recv` drain still in `shell/update.rs`, and both tests
(`notification_sender_is_clone_and_send`, `notifications_drained_from_channel_each_frame`)
still present in `tests_misc.rs`. The shared `Arc<Mutex<NotificationQueue>>`
remains the single working sink and nothing feeding it changed.

The review brief instructed me to verify B's removal (that only the dead mpsc
channel was cut and no real cross-thread producer was lost). Because B was not
executed, that verification resolves trivially: nothing was cut, so nothing was
broken. The implementer's note gives the rationale -- removing the channel
touches documented notification-system Requirement 1.1 and so must go through
the gate rather than ride in a behaviour-preserving refactor. This is a
deliberate, documented scope reduction, not a defect, and does not block
approval of A and C.

## Change C -- the split is faithful

The test-name invariant is the load-bearing property for this split, so I did
not rely on the recorded count alone: I extracted every `#[test]` fn name from
`git HEAD:crates/ff-desktop/src/shell/tests.rs` and from each new
`shell/tests_*.rs`, then compared the sets.

```
BEFORE (git HEAD tests.rs):  454 tests, 454 unique names
AFTER  (union tests_*.rs):   454 tests, 454 unique names
  tests_command.rs       83
  tests_focus.rs         87
  tests_menu_workspace.rs 96
  tests_misc.rs          71
  tests_nav.rs           44
  tests_scrm.rs          16
  tests_session.rs       25
  tests_split_detach.rs  32
  tests_common.rs         0  (shared helpers only)
MISSING: 0   ADDED: 0   DUPLICATES: 0
```

This matches the implementer's recorded figures exactly. No `#[ignore]` appears
in the new files, and none existed in the pre-split file either, so no test was
quietly disabled. Assertions move verbatim with their functions (the split is a
pure relocation; the diff adds no edits to test bodies). The helpers that the
moved tests depend on are present in `tests_common.rs` as `pub(crate)`
(`make_shell`, `make_dispatch`, `is_shell_command`, `harness_shell`,
`make_shell_with_history_path`, `make_shell_with_menus_editor`,
`make_shell_with_scrm_dir`, and the rest), glob-imported by each area module.
Keeping the modules as direct children of `shell` preserves every `super::`
path the tests use, which is why `cargo check`/`cargo test --no-run` compiled
in the recorded run.

The three failures the implementer saw under plain multithreaded `cargo test`
(`startup_loads_persisted_command_history`, `exit_saves_command_history_and_reloads`,
`close_workspace_removes_settings_from_config`) are the known B048
shared-env-var race (`FFWB_HISTORY_PATH` / `FFWB_USER_CONFIG_PATH` set via
`std::env::set_var`), not a split artefact -- they pass under `--test-threads=1`
and under the canonical `cargo nextest` process isolation. The split only
changed thread interleaving, not the tests; this is consistent and not a
regression introduced here.

## Character set and file size

The nine new files scan clean for non-ASCII (`[^\x00-\x7F]` -> no matches); the
old box-drawing separators were converted to ASCII `-`. The lines added to
`shell/mod.rs` (the nine `#[cfg(test)] mod tests_*;` declarations plus a
comment) are ASCII. The non-ASCII hits that remain in `mod.rs` and
`tab_manager.rs` (em dashes, box-drawing, arrows) sit on comment lines this diff
did not modify -- they are pre-existing and out of scope for a change that only
deletes methods and removes one `mod` line. The implementer fixed the single em
dash on the one `tab_manager.rs` comment line it did touch (`open_file`
duplicate-detection comment, now `--`). The 400-line non-test source rule
excludes `#[cfg(test)]` modules, so the test files are exempt by rule; the
non-test files touched only shrank.

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/main.rs` -- removed `mod panel_layout;` (Change A).
- `crates/ff-desktop/src/panel_layout.rs` -- deleted entire dead module (Change A).
- `crates/ff-desktop/src/tab_manager.rs` -- deleted three uncalled openers; one
  em-dash comment fixed to ASCII (Change A).
- `crates/ff-desktop/src/shell/mod.rs` -- replaced `mod tests;` with nine
  `#[cfg(test)]` area-module declarations (Change C).
- `crates/ff-desktop/src/shell/tests.rs` -- deleted monolith (Change C).
- `crates/ff-desktop/src/shell/tests_common.rs` + eight `tests_*.rs` -- new
  split test modules, 454 tests total, ASCII-clean (Change C).

Other working-tree entries (tooling.md, pwsh_command_guard*.py,
wiring-standard.md, cargo.tecst.txt) are from prior workflow steps, not this
change, per the implementer's note. Full diff: `git diff` plus the untracked
`shell/tests_*.rs` files.

</details>
