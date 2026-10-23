# CR-NR-109 Task 11 -- Resume Assessment (ASSESSMENT + SETUP ONLY)

Scope: assess the uncommitted Phase 4 (Task 11) working-tree code against
`docs/specs/localization/crnr109-impl-plan.md`, confirm STEP 0, and hand off the
ordered remaining work. NO subtasks were implemented, nothing committed, no prior
work deleted.

Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr109-locale-alias`
Branch: `feature/crnr109-locale-alias` (HEAD + main both at `1d10e13`; Phase 4
exists ONLY as uncommitted working-tree changes).

## Junk-file cleanup (done)

Deleted the six stray probe/ping files confirmed gone by `git status` (no longer
in the untracked list): `crates/ff-desktop/zz_probe.txt`, `zz_probe2.txt`,
`zz_probe3.txt`, `docs/zz_probe4.txt`, `ping.txt`, `ping2.txt`.

## Owner gate result (cargo gate FULL, nextest) + failing-test triage

The owner ran the full gate: fmt OK, clippy CLEAN, build OK, test run 2791/9913
scope with **2790 passed, 1 failed**. The single failure is NOT a CR-NR-109
test:

`ff-desktop shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme`
panics at `tests_focus.rs:1256` ("clicking Save on a built-in writes a USER theme
(Save As), resolving B081"). It is a Theme Editor Save-As full-shell egui_kittest
test. All CR-NR-109 Task 11 tests PASSED.

Triage performed (evidence-based):
- The test fails in ISOLATION on the Phase 4 tree (run alone, 1045 others
  skipped) -> not cross-test catalogue contamination.
  Log: `tools/logs/crnr109-themetest-alone.log`.
- The Phase 4 changes were STASHED (`git stash --include-untracked`) to reach the
  clean baseline `1d10e13`, and the SAME test run alone there ALSO FAILS with the
  identical panic at `tests_focus.rs:1256`.
  Log: `tools/logs/crnr109-themetest-baseline.log`.
- The stash was then POPPED and the full Phase 4 working tree verified restored
  intact (all 19 modified + 5 untracked `.rs` + docs present).
  Log: `tools/logs/restored-status.log`.

VERDICT: the failure is **PRE-EXISTING and independent of CR-NR-109**. It
reproduces on baseline `1d10e13` with the Phase 4 work entirely removed, so the
locale-alias work neither introduced nor worsened it. `tests_focus.rs` is
byte-identical to the committed baseline (Phase 4 never touched it); the most
recent commit touching it (`8de5b2d`) was itself an i18n-catalogue/timing fix for
this exact Save-click seam, consistent with a fragile full-shell label-query
(`get_by_label("Save").click()`) GUI test. It should be tracked/fixed as its own
(B081-area) item, OUTSIDE the CR-NR-109 scope. It is NOT a blocker for Task 11.

## Compile state: COMPILES CLEAN

`cargo check -p ff-desktop --manifest-path <worktree>\Cargo.toml` -> log
`tools/logs/crnr109-assess-check.log`:

```
Checking ff-desktop v0.1.0 (...crnr109-locale-alias\crates\ff-desktop)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.68s
```

No errors, no warnings in the check output. The uncommitted Task 11 code builds.
(Per the testing split rule, the OWNER runs the authoritative full gate / test
suite; this was a scoped compile check only. Clippy/fmt were NOT run in this
assessment step and remain for the implementer / owner.)

## STEP 0 (docs prerequisite): ALREADY SATISFIED -- nothing copied

All four approved gate docs are ALREADY present in the worktree as the prior
run's uncommitted edits, and are BYTE-IDENTICAL to the MAIN checkout (verified by
`git diff --no-index` main-vs-worktree for each -- all four diffs EMPTY):

| Doc | Present in worktree | Diff vs main |
|-----|---------------------|--------------|
| `docs/specs/localization/requirements.md` | Requirement 11 + criteria 11.1-11.10 + Req 4.2/7.3 amendment notes (ends line 433) | identical (empty) |
| `docs/specs/localization/design.md` | CR-NR-109 design section (line 234) + framework-conformance + testability | identical (empty) |
| `docs/specs/localization/tasks.md` | Phase 4 Task 11.1-11.7 (lines 182-226) | identical (empty) |
| `docs/quality/TCR.md` | 10 CR-NR-109 rows Req 11.1-11.10, all marked 🔴 NOT COVERED (lines 3840-3851) | identical (empty) |

No copy/sync was needed. (`docs/status/change-log.md` also carries the
CR-NR-109 entry, +15 lines.) STEP 0 is complete; code may proceed.

## Per-subtask status

| Subtask | Status | Reason (one line) |
|---------|--------|-------------------|
| 11.1 entry model (`AliasEntry` + localisation key) | DONE | `environment.rs`: `AliasEntry{surface,canonical,localisation_key}`, `display_surface()`, keyed overlay, `ffedit_english` builds `None` keys; 3 unit tests incl. `display_surface_resolves_localisation_key_else_raw_surface`. |
| 11.2 reader + store wiring | DONE | `alias_reader.rs`: TOML `[[alias]]` parse, `LocaleAliasStore` (grouped `i18n.alias_store: Arc<Mutex<..>>`), `publish_active_alias_store` + `reload_active_alias_store_for_locale`; seeded in `construct.rs:175`, reload on `ui.locale` in `main.rs:133`; collision-retains-base + 4 unit tests. |
| 11.3 identity synthesis | DONE | `AliasTable::identity_for(roots)` + `builtin_ffcmd_roots()` (from `recovery_pom_menu`/`recovery_settings_menu`); overlay-replaces-default handled in `apply_locale_overlay_keyed`; tests `synthesised_identity_row_exists_for_every_root`, `overlay_row_replaces_synthesised_default_without_changing_root`. |
| 11.4 unified resolver | DONE | `locale_resolver.rs`: `LocaleResolveOutcome`, `resolve_locale_verb(verb,env)`, `rewrite_verb_to_root(raw)`, `active_environment_name()`; call sites `dispatch.rs:87` (command line) and `commands_menu.rs:186` (`activate_menu_option`); 3 unit tests over the Req 11.6 order. |
| 11.5 menu-render refactor + split | DONE | PURE-REFACTOR split landed: `render_layout.rs` holds colours/measurement/`option_prefix_job`; `render.rs` production ~344 lines (`#[cfg(test)]` at line 345), under 400; `render_menu_workspace` takes `command_surfaces:&[String]`, fed by `render_body_arms.rs:174 self.menu_option_command_surfaces()`; `i18n_keys::localised_option_command` + unit test. |
| 11.6 full-shell egui_kittest test | DONE | `tests_locale_menu.rs` (registered `shell/mod.rs:138`): `full_shell_menu_option_shows_localised_surface_and_dispatches_root`, `full_shell_menu_option_en_shows_root_verb_unchanged`, `full_shell_localised_menu_first_tab_still_focuses_first_interior`; helpers `install_verb_find_soek_catalogue` / `overlay_alias_row` in `tests_common.rs`. |
| 11.7 af POC alias data + verb keys | NOT-STARTED | No `i18n/<locale>/aliases/` directory exists for ANY locale (af/en/de/fr); no `verb-find` key in ANY `.ftl` (empty `Select-String` result). The POC DATA and the minimal `verb-*` keys the plan's SEQUENCING CAVEAT requires have not been authored. |

## Divergences between the uncommitted code and the plan

Overall the uncommitted code follows the plan's design faithfully. Notable
deviations, all MINOR and to be reconciled IN FAVOUR OF THE PLAN while keeping
the correct existing work:

1. Extra module split not named in the plan: `shell/environment_contract.rs` was
   extracted from `environment.rs` (the `CommandEnvironment` trait +
   `EnvDispatchOutcome`), re-exported from `environment.rs`, to keep
   `environment.rs` under the 400-line rule after `AliasEntry` grew it. This is a
   legitimate rust-standards split (no behaviour change) and is consistent with
   the plan's 400-line discipline; KEEP it. Confirm `environment.rs` and
   `environment_contract.rs` are each under 400 non-test lines (not yet measured
   here).
2. Resolver folds Req 11.6 steps 1 and 1.2.1 into ONE `canonical_verb` lookup
   against the store table (because the store's identity rows ARE the root table
   for L). The plan allowed a separate `is_root_command` registry check as step
   2; the code's single-lookup design is equivalent for seeded roots and is
   simpler. CAVEAT to verify during implementation: a root command that is NOT in
   the FFCMD identity seed (seed derives only from the two built-in menus'
   option roots) would return `Unknown` rather than `Root`. Confirm the seed
   covers every root a typed/selected verb can legitimately carry, or restore the
   registry fallback the plan described, so Req 11.6.1.2.1 holds for all roots
   (not just menu-option roots). This is the one substantive thing to re-check.
3. `#![allow(dead_code)]` remains on `alias_reader.rs` and the overlay machinery.
   Several items (e.g. `rewrite_verb_to_root`, `menu_option_command_surfaces`) DO
   now have production callers, so the blanket allow may be masking genuinely
   unused items. The implementer should narrow/remove the allow once 11.7 lands
   and re-run clippy `-D warnings` to surface anything truly dead.
4. `overlay_row_for_test` hardcodes the `verb-find` localisation key for any
   test overlay row. Fine for the current tests; note it if a future test needs a
   different key.

No divergence contradicts an approved criterion; none requires a gate change.

## Confirmed remaining work (ordered by the plan's dependency chain)

The mechanism (11.1-11.6) is implemented and compiles. The remaining work is:

1. Task 11.7 -- author the af POC DATA + minimal verb keys (the only NOT-STARTED
   subtask):
   - `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` (and FFEDIT if an editor verb is
     in the POC) with reviewed rows, at minimum `surface="SOEK"`,
     `canonical="FIND"`, `localisation_key="verb-find"`.
   - The minimal `verb-*` keys the rows reference, in the EXISTING `.ftl` tree:
     `verb-find = FIND` under `i18n/en/` and `verb-find = SOEK` under `i18n/af/`
     (a focused file e.g. `ff-desktop-verbs.ftl`), registered with the catalogue
     loader if it enumerates files explicitly.
   - TDD: write `af_poc_alias_catalogue_loads_and_resolves` first
     (`// Validates: localization Requirement 11.1, 5.1, 5.5`).
2. Verification pass for the whole of Task 11:
   - Re-check divergence (2) above: ensure every legitimate root resolves to
     `Root` (seed coverage or registry fallback).
   - Narrow the `#![allow(dead_code)]` now that production callers exist; run
     scoped `cargo clippy -p ff-desktop -- -D warnings` and `cargo fmt -p
     ff-desktop -- --check` and read the logs.
   - Confirm `environment.rs` / `environment_contract.rs` / `render.rs` /
     `render_layout.rs` are each under 400 non-test lines.
   - Run the scoped new tests where quick (resolver, reader, i18n_keys, and the
     af POC test); leave the full `egui_kittest` suite + `cargo gate` to the OWNER.
3. TCR update: flip the 10 CR-NR-109 rows (Req 11.1-11.10) from 🔴 to PASS with
   the test name as evidence, per the plan's TCR row mapping, as each is proven.
   (Deferred until the owner's authoritative suite run per the testing split.)

## Evidence log files (under tools/logs/)

- `crnr109-assess-check.log` -- the scoped compile result (clean).
- `crnr109-gitstatus.log` -- post-cleanup git status (junk files gone).
- `crnr109-req-diff.log` / `crnr109-tcr-diff.log` / `crnr109-design-diff.log` /
  `crnr109-tasks-diff.log` -- all EMPTY (docs byte-identical to main, STEP 0).
- `crnr109-i18nfiles.log` -- i18n tree listing (no `aliases/`, confirming 11.7).
- `crnr109-verbfind.log` -- EMPTY (no `verb-find` key anywhere).
- `crnr109-threading.log` / `crnr109-wiring.log` / `crnr109-mods.log` /
  `crnr109-common.log` -- render threading, store/resolver wiring, module
  registration, and test-helper presence.
