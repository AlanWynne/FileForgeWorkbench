# CR-NR-109 Phase 4 (Task 11) -- Review

Branch: `feature/crnr109-locale-alias` (worktree `.worktrees/crnr109-locale-alias`).
Commits reviewed (main..feature):

- `f93063a` feat: af POC alias data + verb keys (Task 11.7)
- `4ae52da` feat: locale-aware command-alias layer + menu-kind localised options (Task 11.1-11.7 + verification-pass fixes)
- `397f243` docs: flip CR-NR-109 Req 11.1-11.10 TCR rows to PASS + tick Task 11 subtasks
- `a3dbe67` docs: land CR-NR-109 spec deltas, plan, verification tool scripts

Verdict: APPROVED. No blocking findings.

Verification posture: read the four commit diffs and the recorded scoped
verification evidence (commit `4ae52da` body: `cargo check`/`clippy -D
warnings`/`fmt` CLEAN; scoped tests PASS incl. af POC, resolver, i18n_keys,
full-shell tests_locale_menu, first-Tab focus). Did NOT re-run suites. Ran only
narrow non-suite spot-checks (line counts, ASCII scan, TCR/tasks grep) to
corroborate the structural points below. No articulable doubt remained that
required a scoped `cargo test`/`check`.

## Point-by-point

### 1. Root-resolution seed coverage (the key correctness check) -- PASS
`alias_reader::builtin_ffcmd_roots()` seeds the FFCMD identity table from BOTH
the two built-in menus' option roots AND
`command_config::BUILTIN_WORKSPACE_VERB_ROOTS` (COMMANDS, LOG, PLUGINS, MACROS,
GSEARCH, SEARCH, KINDS, MENUS, CONFIG, KEYS, THEME, FILES, CATALOGS, =FILES, FILE
CATALOGS), deduplicated case-insensitively. The resolver
(`locale_resolver::resolve_locale_verb`) does a single `canonical_verb` lookup
against this table; the identity rows ARE the root-command table for L, so a
bare English root self-resolves to `Root` (Req 11.6 branch 1.2.1) rather than
`Unknown`. The scoped test `every_registered_ffcmd_root_resolves_to_itself`
asserts COMMANDS/LOG/PLUGINS/MACROS/SEARCH/GSEARCH, the two-word `FILE CATALOGS`,
the `=FILES` fastpath, and a menu-option root (CATALOGS) all resolve to Root.
`BUILTIN_WORKSPACE_VERB_ROOTS` is co-located with `builtin_workspace_target_for`
and guarded by `builtin_workspace_verb_roots_list_matches_the_classifier` so the
list and classifier cannot drift. No legitimate seeded root returns Unknown; an
unseeded verb falls through the UNCHANGED front door
(`rewrite_verb_to_root` leaves an Unknown string untouched), so dispatch is never
broken for an unseeded root.

### 2. Framework conformance -- PASS
`dispatch_command_string` calls `rewrite_verb_to_root(raw)` ONCE, after the `=`
strip and BEFORE the prelude/`resolve_target`/ladder; the rewrite only replaces
the verb token with its ROOT (argument + case preserved), then the unchanged
front door dispatches the ROOT. `activate_menu_option` routes the option's root
through the same `rewrite_verb_to_root` for the single-path property. The ROOT is
what dispatches, records (`command_line_history`), and persists. Built on the
existing per-environment `AliasTable` + `apply_locale_overlay_keyed` collision
machinery and the `ff_i18n::t()` seam. No second dispatcher, no parallel nav
stack, no new persistence format. The store lives in the grouped `I18nState`
sub-struct (`i18n.alias_store`), not a flat god-struct field, and the
`ui.locale` reload reuses the existing help-registry reload callback seam in
`main.rs` via a process-global Arc slot.

### 3. dead_code -- PASS
The blanket `#![allow(dead_code)]` is removed from both `alias_reader.rs` and
`alias_overlay.rs`. Remaining allows are narrow item-level `#[allow(dead_code)]`
with reason comments on the keyless-parallel API only (`AliasCatalogue::
with_environment`/`rows_for`/`load_into`, `AliasTable::apply_locale_overlay`
keyless shim). `apply_locale_overlay_keyed`'s allow was dropped (it now has a
production caller chain). Commit evidence records `clippy -D warnings` clean.

### 4. 400-line rule -- PASS
Measured non-test production lines: environment.rs 356, environment_contract.rs
56, locale_resolver.rs 170, render_layout.rs 85. render.rs: the linecount tool's
"NonTest=17" is an artifact (the first `#[cfg(test)]` is a test-only `use` at
line 18); the REAL `mod tests` is at line 345/346 with the `// === Tests`
separator at 343, so the production body is ~342 lines -- under 400. All under
the limit. render.rs was split into render_layout.rs as a pure refactor (helpers
+ their tests moved; re-exported so existing paths are unchanged).

### 5. Task 11.7 -- PASS
`crates/ff-i18n/i18n/af/aliases/FFCMD.toml` ships the single reviewed row
`surface=SOEK`, `canonical=FIND`, `localisation_key=verb-find`. `verb-find = FIND`
in `en/ff-desktop-verbs.ftl` and `verb-find = SOEK` in `af/ff-desktop-verbs.ftl`;
lowercase-hyphen Fluent key syntax; ASCII. The directory-merge catalogue loader
auto-picks up the `.ftl` file (no explicit loader registration needed; the
`aliases/` subdir is read separately). fr/de alias data NOT added. The reader
test `af_poc_alias_catalogue_loads_and_resolves` exists in
`shell/alias_reader.rs` with `// Validates: localization Requirement 11.1, 5.1,
5.5` and asserts SOEK->FIND, FIND->FIND (Req 5.5 Identity_Base retained), and
`display_surface_for_canonical("FIND") == Some("SOEK")` (t(verb-find) under the
installed af catalogue).

### 6. TCR -- PASS
All 10 rows Req 11.1-11.10 (TCR.md lines 3842-3851) are flipped to PASS. Each
cited test was confirmed present in the tree:
environment::tests::{display_surface_resolves_localisation_key_else_raw_surface,
synthesised_identity_row_exists_for_every_root,
overlay_row_replaces_synthesised_default_without_changing_root,
overlay_row_matching_identity_surface_updates_without_collision};
alias_reader::tests::{reader_parses_toml_rows_into_catalogue,
missing_environment_file_leaves_that_environment_identity_only,
af_poc_alias_catalogue_loads_and_resolves,
reader_rejects_within_environment_collision_and_retains_base};
locale_resolver::tests::{resolve_locale_verb_follows_the_req_11_6_order,
resolution_is_scoped_to_the_active_environment,
every_registered_ffcmd_root_resolves_to_itself,
root_command_is_returned_regardless_of_surface};
i18n_keys::tests::localised_option_command_resolves_alias_surface_else_root;
tests_locale_menu::{full_shell_menu_option_shows_localised_surface_and_dispatches_root,
full_shell_menu_option_en_shows_root_verb_unchanged}. No PASS row cites a
non-existent test. The ✅ marker is the file's established allowed emoji
convention. Owner's full gate run remains the authoritative suite PASS per the
verification split, as stated in the commit body.

### 7. tasks.md -- PASS
Parent `11.` and subtasks 11.1-11.7 all `[x]`; only `[ ]`/`[x]` markers;
descriptive titles kept verbatim; "Satisfies ..." notes on indented content
lines, not inside brackets.

### 8. Workspace conformance -- PASS
The new `full_shell_localised_menu_first_tab_still_focuses_first_interior` guards
that the localised command column does not disturb the first-Tab focus latch, and
references the pre-existing `tests_focus::full_shell_first_tab_focuses_reported_
first_interior` as unchanged/green. The change reads `focus.first_interior_id`
only; the InteriorFocus contract and `render_menu_workspace` focus-anchor
reporting are not altered (the renderer gained a `command_surfaces` DISPLAY
argument only, with an empty-slice fallback).

### 9. House rules -- PASS
ASCII scan of the new source/data/docs returned no matches. No unwrap/expect in
non-test code (resolver uses `if let Ok`/`let else`; reader matches on read
errors; `expect` only under `#[cfg(test)]`). thiserror used for
`AliasOverlayError` in the lib-style overlay module. Files staged by name per the
finish-plan commit sequence; no push/PR/merge; no secrets.

## Conclusion
All nine review points pass with no blocking findings. The implementation builds
on the single dispatch front door, the existing alias/overlay machinery, and the
ff_i18n seam; the critical seed-coverage concern is satisfied and proven by a
scoped test. APPROVED.
