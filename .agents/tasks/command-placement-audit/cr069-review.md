# CR-CH-069 Review -- Correct editor command-environment placement

Verdict: **APPROVED** (no blocking findings).

Branch `cmd-placement-fix` was merged into `main` and its worktree removed before
this review ran (the `.worktrees/cmd-placement-fix` path no longer exists and the
shell runner in this environment is non-functional -- every `pwsh7` invocation
returned the `Exit Code: -1` capture artifact and produced no log/file, the same
dead-terminal episode the implementation summary records). The CR-CH-069
implementation is present in the MAIN workspace source (confirmed by the
`CR-CH-069` markers across the changed files), so this review was performed
STATICALLY against the merged source in `c:\workspace\apps\FileForgeWorkbench`,
per the task brief's authorisation to verify statically when the terminal is
dead and to read the coder's recorded verification evidence rather than re-run
the suites.

## Scope checked -- exactly the three corrections

### (A) AUTONUM / NUM moved FFCMD -> FFEDIT -- CORRECT
- `shell/environment.rs::ffedit_english()` adds identity entries
  `("AUTONUM","AUTONUM")` and `("NUM","NUM")` (lines ~150-155), so
  `root_commands("FFEDIT")` contains them. The surface-token -> canonical
  resolution in `FfEditEnvironment::claim` therefore routes AUTONUM/NUM to their
  arms (Req 21.1).
- `command_config/mod.rs`: AUTONUM and NUM are removed from
  `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (explicit removal comment at ~244-247), so
  `root_commands("FFCMD")` no longer lists them (Req 21.1).
- `shell/commands_ladder_c.rs`: the former AUTONUM/NUM ladder arms are gone,
  replaced by a pointer comment (~132-136). They are no longer handled on the
  global FFCMD ladder.
- `shell/environment_ffedit.rs`: AUTONUM/NUM claim arms (~289-318) are moved
  VERBATIM -- `AUTONUM ON|OFF` slices `raw.trim()[7..]` and redirects to
  `NUMBER<rest>` via `shell.handle_command`; `NUM [rest]` redirects to
  `NUMBER [rest]`. Behaviour is preserved: only the claim LOCATION changed
  (editor-active FFEDIT vs the global ladder), satisfying active-wins (Req 5.1 /
  Req 21.2). The byte-offset slice is safe (ASCII "AUTONUM" = 7 bytes).
- NUMBER-engine scope creep: NONE. No NUMBER engine and no stub were added;
  NUMBER still falls through to the engine's "unrecognised" response exactly as
  on main. The deferral is explicitly flagged in the summary and in the Req 21.2
  text / the inline comments. This matches the required "no scope creep" posture.

### (B) af SOEK -> FIND relocated FFCMD -> FFEDIT catalogue -- CORRECT
- `crates/ff-i18n/i18n/af/aliases/FFEDIT.toml` (new) carries the single
  `SOEK -> FIND` row with `localisation_key = "verb-find"`. FIND is a valid FFEDIT
  root (`ffedit_english` owns FIND; command-semantics Req 12.1).
- `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` no longer declares any alias; the
  comment documents the move. (Localisation DATA, ASCII-exempt; both files are
  ASCII anyway.)
- `shell/alias_reader.rs::af_poc_alias_catalogue_loads_and_resolves` now asserts
  the FFEDIT catalogue ships, reads SOEK->FIND from it, loads it onto the
  `ffedit_english` base, resolves `canonical_verb("SOEK") == FIND` and FIND still
  self-resolves, AND asserts the FFCMD catalogue does NOT contain SOEK->FIND
  (Req 21.3). The test is strengthened, not weakened.

### (C) Stale EDIT doc-comment fixed -- CORRECT
- `shell/user_aliases.rs` `AliasEnvironment` doc-comment now states EDIT is an
  FFCMD file-open root and `ECOB -> EDIT` is an FFCMD alias, referencing
  `validate_accepts_root_valid_in_declared_environment` (Req 21.5). Doc-only.

## Tests, criteria, and coverage record
- New/updated tests carry `// Validates:` annotations to the real new criteria
  (Req 21.1-21.5):
  - `environment_registry::tests::autonum_and_num_are_ffedit_roots_not_ffcmd_roots`
    and `root_commands_ffedit_returns_canonical_editor_verbs` (expected set
    extended with AUTONUM/NUM -- strengthened) -- Req 21.1.
  - `environment::tests::ffedit_english_owns_autonum_and_num_roots` -- Req 21.1.
  - `environment_ffedit::tests::autonum_and_num_resolve_through_the_ffedit_alias_table`
    and `in_buffer_verbs_are_not_store_affecting` (extended) -- Req 21.1/21.2.
  - `tests_command::autonum_on_dispatches_to_number_on` and
    `num_command_dispatches_to_number` (updated to open an editor Context first,
    with the reason documented) plus new
    `autonum_and_num_not_claimed_on_non_editor_context` -- Req 21.2.
  - `alias_reader::tests::af_poc_alias_catalogue_loads_and_resolves` -- Req 21.3.
  - `environment_registry::tests::contains_root_reflects_autonum_num_find_moved_to_ffedit`
    (covers FIND + AUTONUM + NUM, both environments) -- Req 21.4.
- Drift-guard `builtin_ffcmd_ladder_verb_roots_are_disjoint_from_classifier_and_carry_file_open`
  is intact and unweakened; it still passes with AUTONUM/NUM removed (they were
  never classifier verbs).
- `docs/quality/TCR.md` CR-CH-069 rows (Req 21.1-21.5) are flipped to covered
  (checkmark) with the proving test names, consistent with the source.
- EARS criteria Req 21.1-21.5 in `docs/specs/command-environments/requirements.md`
  match the three corrections precisely and mark the NUMBER engine deferred.
- Change-log entry and ROADMAP row for CR-CH-069 exist.

## Framework conformance
- No new dispatcher: the AUTONUM/NUM redirect re-enters the single front door via
  `shell.handle_command`. No second navigation stack, no persistence-format
  change, no reshape of a public framework type. The change is a classification
  edit to the existing root sources plus the existing FFEDIT active-wins claim
  path -- builds ON the framework.

## Other mechanical checks
- Source `.rs`: changed files are under 400 non-test lines (environment.rs
  non-test ends ~391 before the `#[cfg(test)]` region; environment_ffedit.rs
  ~342).
- No `unwrap`/`expect` introduced in non-test code (the claim arms use
  `unwrap_or("")` and `handle_command`, no panicking calls).
- ASCII-clean source comments (`--`, `->`); TOML files are localisation DATA
  (exempt) and contain only ASCII.
- No collateral damage observed to CR-NR-110/111/112/113 or CR-CH-066/067/068:
  only the two AUTONUM/NUM placement tests were updated (with documented reasons)
  and the af POC test relocated; the CR-CH-067 `contains_root` interaction is the
  intended consequence (an FFCMD-anchored AUTONUM/NUM/FIND row is now correctly
  invalid-in-environment), not a regression.

## Verification evidence (read, not re-run)
Per the implementation summary: the agent proved compile/clippy/fmt clean
(scoped `-p ff-desktop`), and the OWNER ran the authoritative full `cargo gate`
in the worktree CLEAN at 10008/10008, which executes the Req 21.2 proving tests
(`autonum_and_num_not_claimed_on_non_editor_context`,
`num_command_dispatches_to_number`) green. This cleared the prior review's single
blocking finding F1, whose recorded failing `tests-autonum.log` was a stale TDD
red-phase artifact (the committed test setup/message differ from the logged
panic, confirming the source was finalised after that log). The committed source
chain (`set_active_tab_home` -> POM Home -> FFCMD base -> `is_ffedit_active`
false) makes the test pass, consistent with the owner's green gate. The required
verification evidence is therefore present; no suite was re-run for this review.

## Non-blocking observation (not a finding)
- The `docs/status/change-log.md` CR-CH-069 entry and the `docs/quality/TCR.md`
  phase header still read "GATE AUTHORED / PENDING OWNER APPROVAL" (the docs-first
  wording) even though the code has landed and the per-criterion TCR rows are
  flipped to covered. This is a stale narrative status line, not a correctness or
  coverage problem (the authoritative per-row coverage is already ✅). Optional
  cleanup only; it does not block approval.

## Conclusion
All three scoped corrections are implemented correctly against the authored
criteria, tests are annotated and strengthened rather than weakened, the TCR is
covered with proving test names, framework conformance holds, and the required
verification evidence (owner's clean full gate) is recorded. No blocking
findings. APPROVED.
