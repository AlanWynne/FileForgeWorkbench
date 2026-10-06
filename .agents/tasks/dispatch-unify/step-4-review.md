# B080 Step 4 -- manager families: dispatch-unification migration review

Review of the Step 4 work of the B080 dispatch-unification migration. Step 4 is a
CONFORMANCE, behaviour-preserving change against existing command-framework
criteria (Req 2.1 / 2.7 / 8.3 / 8.4 / 9.2 / 9.7); the requirements gate is CLOSED.
The coder was permitted to EITHER migrate the five manager families
(nav / exclude-show / find / profile / scroll) behaviour-preservingly OR to
defer-and-report like Step 3. A sound, documented deferral is a SUCCESSFUL
outcome.

**Watch for:** nothing blocking. The coder's shape decision is a defer-by-
supersession: the five families were already moved off the `handle_command`
ladder by CR-CH-053 (the FFEDIT Command Environment), not by a new B080
CommandTarget migration. I verified that claim against the live tree -- it holds
on every point I could check: the families are gone from the a/b/c ladders, they
now live in `ffedit_claim` / `dispatch_ffedit.rs` with their bodies moved
verbatim (B062 case preserved, `parse_two_args` quoting preserved), and they are
reached through the single front door. No new CommandTarget variant was added and
no param-carrying Function dispatch was slipped in.

**Verdict:** APPROVED

## High-level view

The decision to migrate was correctly declined. These are editor-action verbs
that mutate the active editor via managers; they are not workspace-openers. None
of the existing `CommandTarget` variants carries them behaviour-preservingly --
`CustomWorkspace` is a workspace-opener shape, and `Function` dispatch drops
params (the exact Step-3 blocker that correctly deferred EDIT/BROWSE/VIEW). The
only clean routing would require a new variant or param-carrying Function
dispatch, which is a framework change needing express owner confirmation. The
coder refused to force that. This is the same reasoning Step 3 used, applied to
the manager families.

The deferral is stronger than Step 3's because the alternative owner of these
verbs has already shipped. CR-CH-053's FFEDIT Command Environment claims all five
families: `ffedit_claim` in `shell/dispatch.rs` matches the canonical verbs and
delegates to the managers, with the E2-E5 verb bodies in `shell/dispatch_ffedit.rs`
moved verbatim from the old `try_commands_b2` arms. The old `commands_ladder_b2.rs`
file no longer exists.

Behaviour preservation checks out. The FFEDIT claim is gated in `run_command_ladder`
(`shell/commands.rs`) to fire only when an editor Context is the active
environment and the command is not `=`-prefixed, ordered ahead of the FFCMD exit
arms -- so active-wins (Req 5.1) and the `=` escape hatch (Req 3.2) are intact.
CHANGE keeps `parse_two_args` quoting and FIND/CHANGE keep the B062 case-preserved
term. RESET BARE is explicitly declined by FFEDIT so FFCMD still owns it (no new
shadowing). The single-front-door invariant (Req 2.1) holds because every path
still enters through `dispatch_command_string` -> `run_command_ladder`, whose
first act is the FFEDIT claim.

The scoped evidence is present and consistent: `cargo check -p ff-desktop`
finished clean (log at `tools/logs/b080-step4-check.txt`), no code was edited for
a migration (zero behaviour change), and the two FFEDIT files are ASCII-only and
under the 400 non-test-line rule. No `--workspace` / `cargo gate` / `ffwb-gate.ps1`
run was done in-agent, as required.

<details>
<summary>Issues (0)</summary>

No blocking or non-blocking issues. One informational note: Steps 5-6 direction
and Step 7 (retire the superseded Step-2 ladder arms) remain open follow-ups, as
the note records; they are out of Step 4 scope.

</details>

<details>
<summary>Details</summary>

### Shape decision: defer-by-supersession is the correct call

The judging crux is whether migrate-or-defer was decided soundly. The coder chose
defer, with the specific nuance that the deferral target has already shipped. The
reasoning in the impl-note is concrete and matches the tree:

- These verbs mutate the active editor via managers (`nav_manager`,
  `exclude_manager`, `find_manager`, `edit_profile`, `scroll_amount`); they do not
  open a workspace/Context. `CustomWorkspace` is the Step-2 workspace-opener model
  (`nav_to_kind`), a conceptual and param mismatch.
- `Function` dispatch routes `CommandTarget::Function { command_id, .. }` through
  `handle_command(command_id)`, which carries no params -- the exact Step-3
  blocker. The quoted CHANGE/FIND argument would be lost, a Req 8.4 regression.
- Routing them cleanly would need a new CommandTarget variant or param-carrying
  Function dispatch -- a framework change requiring express owner confirmation
  (framework-conformance.md) -- or would duplicate the logic CR-CH-053 already
  owns.

This is the same class of reasoning Step 3 used to defer EDIT/BROWSE/VIEW, and it
is cited with file:line evidence rather than asserted. The B080 bug REFRAME
itself lists exactly these verbs as belonging to the Command Environments model,
so the deferral is consistent with the recorded direction, not an improvisation.

### Verification that CR-CH-053 actually owns the families (the deferral target shipped)

I did not take the "already migrated" claim on faith; I confirmed it against the
tree this pass:

- `shell/dispatch.rs::ffedit_claim(&mut self, raw, upper) -> bool` resolves the
  surface verb via `AliasTable::ffedit_english()` then matches the canonical verb:
  - E1 nav: LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT -> `self.nav_manager.*`,
    the former `try_commands_b2` arms moved verbatim (incl. the CR-NR-087
    numeric `UP n` / `DOWN n` vs scroll-amount behaviour).
  - E2 exclude/show: EXCLUDE/X/SHOW/INCLUDE/RESET -> `ffedit_exclude/_show/_reset`.
  - E3 find: FIND/RFIND/CHANGE/RCHANGE -> `self.find_manager.*`.
  - E4 profile: CAPS/NULLS/STATS/LOCK/PROFILE/HILITE -> `ffedit_caps/_nulls/
    _stats/_lock/_profile/_hilite`.
  - E5 scroll: SCROLL -> `ScrollAmount::parse` -> `self.scroll_amount`.
- `shell/dispatch_ffedit.rs` holds the E2-E5 verb bodies, each the former ladder
  arm moved verbatim (observable result identical, command-environments Req 6.3).
- `commands_ladder_b2.rs` NO LONGER EXISTS (file_search: no match).
- A grep of `commands_ladder_a/b/c.rs` for `nav_manager`, `exclude_manager`,
  `find_manager`, `edit_profile`, `scroll_amount` returns ZERO matches -- no
  manager-family arm remains on the ladder.

So the families are neither on the ladder nor in `builtin_workspace_target_for`
(which remains the Step-2 CustomWorkspace/nav family only -- no over-claim). They
are in FFEDIT. This is a materially stronger deferral than Step 3's: the model
that should own the verbs is built and does own them, so a B080 migration now
would re-implement shipped logic.

### Behaviour preservation (Req 8.4) under the FFEDIT claim

The FFEDIT claim is gated in `run_command_ladder` (`shell/commands.rs`):

```
if !cmd.trim_start().starts_with('=') {
    ... active_environment(active_kind, active_is_home) == EnvironmentKind::FfEdit
        && self.ffedit_claim(cmd, upper) { return; }
}
```

- Active-wins (Req 5.1): the claim fires only when the active Context's
  environment is FFEDIT (an editor), ordered ahead of `try_commands_a` (which
  owns the FFCMD exit verbs). On a non-editor Context the gate is false and the
  same verb falls through to FFCMD, preserving pre-CR-CH-053 behaviour.
- `=` escape (Req 3.2): a `=`-prefixed command is not offered to the environment,
  so `=X` still reaches FFCMD's exit rather than FFEDIT EXCLUDE.
- B062 case rule: FIND/CHANGE use `verb_arg` for the case-preserved term; the
  exclude/show bodies slice the surface string (`cmd.trim()[8..]` etc.) to
  preserve argument case; the toggles key on `upper` only for keyword matching.
- `parse_two_args` quoting: CHANGE delegates to
  `super::helpers::parse_two_args(rest)` and surfaces the same "requires two
  arguments" error on a parse miss.
- RESET BARE guard: FFEDIT's RESET declines `RESET BARE ...` (returns false) so
  the FFCMD `RESET BARE` ladder arm still handles it -- preventing a new
  shadowing regression introduced by running the env claim ahead of the FFCMD
  arms.

The single-front-door invariant (Req 2.1) is preserved: all seams still enter
`dispatch_command_string` -> prelude -> `resolve_target` -> `run_command_ladder`,
whose first act is the FFEDIT claim. No second dispatcher, no parallel path.

### No framework change slipped in

`builtin_workspace_target_for` is unchanged (still the Step-2 family). No new
`CommandTarget` variant exists. No param-carrying Function dispatch was added --
the Step-3 blocker is untouched, which is exactly why the migration was declined
rather than forced. The CR-CH-053 claim mechanism (`ffedit_claim`) is an existing,
owner-confirmed framework element (command-environments Req 6.x), not a B080
addition. This is conformant with framework-conformance.md.

### Scoped evidence, ASCII, file-size, and no full-suite runs

- Scoped check: `tools/logs/b080-step4-check.txt` records `Finished dev profile`
  for `cargo check -p ff-desktop` -- clean. Because no code was changed for the
  migration, no new tests were added; the behaviour is covered by the existing
  shell suite plus the `environment.rs` unit tests
  (`ffedit_claims_owned_verb_only_when_active_env_is_ffedit`,
  `ffedit_shadows_ffcmd_x_but_not_the_other_exit_verbs`). The note does not claim
  a full `cargo test -p ff-desktop` count for Step 4, which is honest given zero
  code change; the scoped-check evidence is present, so I did not re-run suites.
- ASCII-only: grep for non-ASCII in `shell/dispatch.rs` and `shell/dispatch_ffedit.rs`
  returns no matches.
- File size: `shell/dispatch.rs` test module starts at line 341, so non-test code
  is ~339 lines (< 400). `shell/dispatch_ffedit.rs` is ~226 lines with no test
  module (< 400).
- No `--workspace` / `cargo gate` / `ffwb-gate.ps1` was run in-agent; only the
  scoped `cargo check -p ff-desktop` appears in the log, per the rule.

### Documentation and process

The impl-note mirrors Step 3's deferral rigor: it states the shape decision,
cites file:line evidence for every claim, explains why migrating would be wrong
(duplication + framework change + regression risk), and records the owner-
confirmed disposition (DONE-BY-SUPERSESSION) plus the deferred Steps 5-6 and the
still-valid Step 7 cleanup. The ladder-site annotations in `commands.rs` are
comment-only and explain the retirement of `try_commands_b2`. This is a correct,
well-documented deferral.

</details>

<details>
<summary>Files referenced</summary>

- `crates/ff-desktop/src/shell/dispatch.rs` -- `ffedit_claim` router; nav (E1),
  find (E3), scroll (E5), and the RESET-BARE decline; test module from line 341.
- `crates/ff-desktop/src/shell/dispatch_ffedit.rs` -- E2 (exclude/show/reset),
  E4 (profile family), SAVE, `find_status_to_error`; verb bodies moved verbatim.
- `crates/ff-desktop/src/shell/commands.rs` -- `run_command_ladder` FFEDIT gate
  (active-wins, `=`-escape) ahead of `try_commands_a`; retirement comment at the
  old `try_commands_b2` site.
- `crates/ff-desktop/src/shell/commands_ladder_a/b/c.rs` -- confirmed no
  manager-family arm remains.
- `crates/ff-desktop/src/command_config/mod.rs` -- `builtin_workspace_target_for`
  unchanged (Step-2 family only; no over-claim).
- `tools/logs/b080-step4-check.txt` -- scoped `cargo check -p ff-desktop` clean.
- No full diff beyond comment-only annotations: Step 4 made zero code change.

</details>
