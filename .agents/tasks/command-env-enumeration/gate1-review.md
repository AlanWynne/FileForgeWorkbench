# Gate 1 Review -- Command Environment root-enumeration API (CR-CH-067)

Branch: `cmdenv-root-enum` (worktree `.worktrees/cmdenv-root-enum`), single feature
commit `cdc35b7` on top of `aeff0d5`.
Reviewed: `git diff main...cmdenv-root-enum`; spec Req 20 + design delta
(command-environments); findings.md; TCR evidence block.

Verdict: APPROVED -- no blocking findings.

## Scope confirmed

The diff is confined to:
- `command_config/mod.rs` -- new `BUILTIN_FFCMD_LADDER_VERB_ROOTS` const + disjointness drift-guard test.
- `shell/alias_reader.rs` -- `builtin_ffcmd_roots()` now also unions the ladder roots.
- `shell/environment_registry.rs` -- new `root_commands` / `contains_root` + 7 tests.
- `shell/alias_reader/alias_user_overlay.rs` -- new load-time skip-with-WARN branch.
- `shell/alias_editor.rs` -- save-side `contains_root` hook + test adjustments.
- `shell/tests_alias_editor.rs` -- load-skip test + routing test re-baselined.
- docs (requirements/design/tasks/TCR). No new `.rs` files.

## Review against the blocking criteria

### 1. ADDITIVE read-only query, no reshape -- PASS
`root_commands` / `contains_root` are name-keyed associated functions ON
`EnvironmentRegistry` (`environment_registry.rs:303,334`). They are NOT a method on
the `CommandEnvironment` trait, NOT a second dispatcher, and do NOT move alias
lookup into `resolve_target`. Grep of all call sites
(`alias_editor.rs:210`, `alias_user_overlay.rs:83`) confirms the only consumers are
the alias load/save seams -- never the hot dispatch, navigation, or persistence
paths. No change to `CommandTarget`, `claim`, `EnvDispatchOutcome`, active-env
derivation, the front door, the Navigation_Stack, or the descriptor persistence
model. Framework-conformance: additive, owner-confirmed ("Proceed..."), consistent
with findings.md section 6 and the design delta.

### 2. root_commands returns the complete canonical root set, deterministic, canonical-only -- PASS
- FFCMD delegates to `builtin_ffcmd_roots()`, now a union of the two code-only
  built-in menus' option roots + `BUILTIN_WORKSPACE_VERB_ROOTS` +
  `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (`alias_reader.rs`). The ladder-roots union is
  the correct completeness fix: EDIT/VIEW/BROWSE are FFCMD file-open verbs handled
  by the `handle_command` ladder (not the classifier), so without the union an
  `ECOB -> EDIT` FFCMD alias would be wrongly rejected. A new drift-guard test
  (`builtin_ffcmd_ladder_verb_roots_are_disjoint_from_classifier_and_carry_file_open`)
  pins the ladder list DISJOINT from the classifier and requires EDIT/VIEW/BROWSE,
  so the two lists cannot silently drift or double-count.
- FFEDIT delegates to `AliasTable::ffedit_english().canonical_verbs_owned()`.
  Verified `canonical_verbs_owned` (`environment.rs:235`) collects `e.canonical`
  (not `e.surface`), de-duplicated via `eq_ignore_ascii_case`. The FFEDIT seed maps
  surface `X -> EXCLUDE` and `INCLUDE -> SHOW`, so the enumerated set carries the
  canonicals EXCLUDE/SHOW and never the surfaces X/INCLUDE.
- Determinism: `root_commands` applies `roots.sort()` before returning, and both
  source collectors already de-dup case-insensitively. Test
  `root_commands_is_deterministic_and_deduplicated` asserts stable sorted order and
  no case-only duplicates.
- Canonical-only: test `root_commands_ffedit_excludes_alias_surfaces` asserts X and
  INCLUDE are absent while EXCLUDE/SHOW are present -- directly validates Req 20.3.
- Unknown env -> empty: `root_commands_unknown_environment_is_empty` covers FFNAV /
  HOSTFS / MAINFRAME / FFLINE / bogus / empty-string, no panic. Validates Req 20.5.

### 3. contains_root case-insensitive, matching existing comparison -- PASS
`contains_root` trims `root` then matches via `eq_ignore_ascii_case`
(`environment_registry.rs:334-338`) -- the same comparison used by
`builtin_ffcmd_roots`'s dedup, `canonical_verbs_owned`'s dedup, and the no-shadow
guard. Tests `contains_root_is_case_insensitive_and_trimmed` (CONFIG/config/"  Config  ",
find/FIND) and `contains_root_false_for_bogus_or_unknown` (bogus root, right-root
wrong-env both directions, surface X rejected, unknown env false) cover Req 20.4/20.5.
`env_name()` returns the registry name constants (`user_aliases.rs`), so the
name-keyed match is consistent.

### 4. REJECT-WITH-WARNING at load; existing guards intact -- PASS
The new branch in `apply_user_rows` (`alias_user_overlay.rs:75-94`) runs AFTER the
malformed-row and no-shadow guards and BEFORE the row is applied. When
`contains_root(env_name, &canonical)` is false it emits a WARN naming surface, root
(canonical), and environment, then `continue` -- skipping that single row only.
`canonical` is trimmed+uppercased consistent with `surface`. The no-shadow guard
and the downstream cross-layer collision WARN are unchanged. Load never aborts; a
single invalid row cannot block startup nor drop another row. Test
`load_skips_user_row_with_root_invalid_in_environment` writes an invalid row (FIND
under FFCMD) directly to disk plus a valid FFCMD GSEARCH row, loads via
`refresh_alias_store_in_place`, and asserts the invalid row is skipped while the
valid row is still applied -- exactly Req 20.7.

### 5. CR-NR-111/112/113, CR-CH-066 untouched; rust-standards; tests -- PASS
- No Alias Editor UI change: `alias_editor_panel/*` is not in the diff (Gate 2
  scope, correctly deferred). The save-side change is a pure validation hook in
  `validate_user_alias_rows`, not editor UX.
- CR-NR-112 template expansion and CR-NR-113 persistence schema are untouched
  (`UserAliasRowToml` fields unchanged; the `localisation_key` field already
  existed). CR-CH-066 menu placement untouched.
- File size: `environment_registry.rs` non-test code ends before the `#[cfg(test)]`
  module at line 347; well under 400. No new non-test files.
- No `unwrap`/`expect` in non-test code -- the `expect`/`unwrap` in the diff are all
  inside `#[cfg(test)]` modules (allowed, and `expect("why")` is used).
- `Debug`: no new public types introduced (only functions and a `const`), so the
  derive requirement does not apply.
- ASCII: new source and doc content is plain ASCII.
- Tests: present, each annotated `// Validates: command-environments Requirement 20.x`,
  deterministic (no clock/random/order reliance), and cover every criterion 20.1-20.8.

## Notes (non-blocking, for the owner / Gate 2)

1. Design-doc signature drift (doc-only): `design.md` shows the API as instance
   methods `root_commands(&self, env_name)` / `contains_root(&self, ...)`, but the
   implementation is associated functions `root_commands(env_name)` /
   `contains_root(env_name, root)` with no `&self`. The code comment documents the
   deliberate choice (so the `apply_user_rows` load path, which holds no registry
   handle, can reach it, and because the FFCMD base is not a trait object). Req 20.1
   only mandates the query be keyed by name ON the `EnvironmentRegistry` type, which
   the associated-fn form satisfies. The design snippet is slightly stale on the
   `&self` detail; worth a one-line doc correction but not a code defect.

2. The FFCMD no-shadow guard in `apply_user_rows` still uses `builtin_ffcmd_roots()`
   unconditionally regardless of the row's environment (findings.md section 5.1).
   This is an acknowledged pre-existing looseness whose per-environment
   reconciliation is explicitly deferred to Gate 2. Req 20.7 only requires the
   existing guards remain intact, which they do. Not a Gate 1 blocker.

## Verification evidence (per testing.md -- not re-run)

The coder's TCR block records the owner's authoritative `cargo gate` (full, nextest,
workspace): fmt OK / clippy CLEAN / build OK / 6911 passed, with the only failure the
unrelated pre-existing flaky `ff-logging::thread_safety_tests::log_call_returns_within_1ms`
wall-clock test (a crate this change never touches), which passed on an isolated
re-run. All ff-desktop Req 20.x tests passed. No terminal dead-episode was reported.
I did a narrow static spot-check of the two source collectors
(`canonical_verbs_owned`, `builtin_ffcmd_roots`) and the new branch to confirm the
canonical-only and skip-with-WARN claims; no additional doubt remained, so the suites
were not re-run. The owner's authoritative gate is the final word on the green run.

## Conclusion

All eight criteria (Req 20.1-20.8) are implemented as additive, read-only behaviour
that builds on existing enumerable data and the existing skip-with-WARN discipline,
with no framework reshape. Tests are present, annotated, deterministic, and cover
each criterion. No blocking findings. APPROVED.
