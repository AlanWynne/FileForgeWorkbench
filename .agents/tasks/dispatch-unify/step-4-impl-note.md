# B080 Step 4 -- manager families: implementation note

Step 4 of the unified command dispatch migration (B080, Phase 3 task 7). Owner-
approved as a CONFORMANCE fix; Req 8.4 behaviour-preservation is mandatory. Work
is DIRECTLY in the main workspace, uncommitted (owner standing direction).

## Outcome: ALL FIVE manager families are ALREADY MIGRATED -- but into the
## FFEDIT Command Environment (CR-CH-053), NOT the B080 CommandTarget path.
## Zero new migration performed; zero behaviour change. Verdict is DEFER-and-
## report (brief option b-iii), and in fact the deferral target has shipped.

Step 4 scope (design-delta section 2.5) was the five manager-backed families:

| Family | Verbs | Backing manager |
|--------|-------|-----------------|
| nav | LOCATE, TOP, BOTTOM, UP, DOWN, LEFT, RIGHT, SORT | `nav_manager` |
| exclude/show | EXCLUDE [ALL], X [ALL], SHOW [ALL], INCLUDE [ALL], RESET [EXCLUDED|ALL] | `exclude_manager` |
| find | FIND, RFIND, CHANGE, RCHANGE | `find_manager` |
| profile | CAPS [ON/OFF], NULLS, STATS, LOCK, PROFILE [kw], HILITE | `edit_profile` |
| scroll | SCROLL <amt> | `scroll_amount` |

### The mandatory shape-check (brief's crux) -- answer: (b), and already resolved

These are EDITOR-ACTION verbs: they mutate the ACTIVE editor via the managers;
they do NOT open a workspace/Context. Several (FIND/CHANGE/LOCATE) carry
case-SENSITIVE, quoted Argument_Strings with the B062 case rule and
`parse_two_args` quoting. The existing `CommandTarget` variants are `Menu`,
`CustomWorkspace`, `Function`, `Macro`, `External`. None fits behaviour-
preservingly:

- `CustomWorkspace` is for workspace-OPENERS that call `nav_to_kind` (Step 2
  model). These verbs are not workspaces -- a conceptual + param mismatch.
- `Function` dispatch (`dispatch_command_target` -> `handle_command(command_id)`)
  DROPS params -- the exact Step-3 blocker that correctly deferred EDIT/BROWSE/
  VIEW. The quoted CHANGE/FIND argument would be lost.
- Routing them cleanly would require a NEW CommandTarget variant / new dispatch
  mechanism (a framework change needing express owner confirmation per
  framework-conformance.md) OR would duplicate logic the CR-CH-053 Command
  Environments model is slated to own.

That is brief option (b)(i)+(iii): the honest call is DEFER, do NOT force a
migration, do NOT expand the framework, do NOT pre-empt CR-CH-053. This mirrors
the B080 bug entry's own REFRAME, which already lists exactly these verbs
(LOCATE/FIND/CHANGE/EXCLUDE/SORT/CAPS/NULLS/PROFILE/scroll) as "NOT shell
targets ... they belong to a new Command Environments model (CR-CH-053, REXX
ADDRESS-inspired)".

### What actually happened in the tree: CR-CH-053 landed and claimed them

The deferral target is no longer merely "slated" -- the FFEDIT Command
Environment (CR-CH-053) has SHIPPED and already owns all five families. Evidence
(file:line, tree as read this pass):

- `crates/ff-desktop/src/shell/dispatch.rs` -- `ffedit_claim(&mut self, raw,
  upper) -> bool` is the FFEDIT claim step. It resolves the surface verb to a
  canonical verb via `AliasTable::ffedit_english()` (`environment.rs`), then
  matches:
  - E1 nav: LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT -> `self.nav_manager.*`
    (the former `try_commands_b2` arms, moved verbatim).
  - E2 exclude/show: EXCLUDE/X/SHOW/INCLUDE/RESET -> `self.ffedit_exclude/_show/
    _reset` (RESET BARE explicitly DECLINED so it falls through to FFCMD).
  - E3 find: FIND/RFIND/CHANGE/RCHANGE -> `self.find_manager.*`, CHANGE keeping
    `parse_two_args` quoting + B062 case-preserved term.
  - E4 profile: CAPS/NULLS/STATS/LOCK/PROFILE/HILITE -> `self.ffedit_caps/_nulls/
    _stats/_lock/_profile/_hilite`.
  - E5 scroll: SCROLL <amt> -> `ScrollAmount::parse` -> `self.scroll_amount`.
  - E9 SAVE (editor-buffer save) also claimed here.
- `crates/ff-desktop/src/shell/dispatch_ffedit.rs` -- holds the E2-E5 verb
  bodies (`ffedit_exclude/_show/_reset/_caps/_nulls/_stats/_lock/_profile/
  _hilite`, `find_status_to_error`, `ffedit_save`), each "the former
  `try_commands_b2` ladder arm body, moved VERBATIM so the observable result is
  identical (command-environments Req 6.3)".
- `crates/ff-desktop/src/shell/commands.rs` -- `run_command_ladder` gates the
  FFEDIT claim FIRST (CR-CH-053 E8, active-wins) when an editor Context is
  focused and the command is not `=`-prefixed:
  `active_environment(..) == EnvironmentKind::FfEdit && self.ffedit_claim(cmd, upper)`.
- `crates/ff-desktop/src/shell/commands.rs` -- comment at the former
  `try_commands_b2` site: "the `commands_ladder_b2.rs` file + its call were
  retired ... The nav / exclude / find / profile / scroll families it used to
  sit beside are now claimed by `ffedit_claim` above when an editor Context is
  active." (CR-CH-053 E7.)
- `crates/ff-desktop/src/shell/environment.rs` -- `active_environment` /
  `environment_for_kind` (editor -> FFEDIT), and `AliasTable::ffedit_english()`
  seeding exactly these verbs (incl. the X -> EXCLUDE / INCLUDE -> SHOW aliases).
- `commands_ladder_b2.rs` -- NO LONGER EXISTS (file_search: no match).
- grep of `commands_ladder_a/b/c.rs` for the family verbs/managers: NO manager-
  family arm remains (only the unrelated `COMMAND TOP/BOTTOM` command-line-
  position verb and `SPLIT DOWN`).

So the five families are NOT on the ladder at all, and they are NOT in
`builtin_workspace_target_for` (unchanged -- still only the Step-2
CustomWorkspace/nav family). They live in FFEDIT. They are still reached through
the ONE front door (`dispatch_command_string` -> prelude -> `resolve_target` ->
`run_command_ladder`, whose first act is the FFEDIT claim), so the single-front-
door invariant (Req 2.1) holds.

### Why this is the correct Step-4 result (not a failure, not zero-work)

The brief states explicitly: a correctly-reasoned, documented deferral is a
SUCCESSFUL Step 4 outcome, exactly as Step 3's full-family deferral was correct;
migrated-zero-verbs is NOT a failure when the evidence supports deferral. Here
the evidence is stronger than Step 3's: not only do the families fail to fit the
B080 CommandTarget shape, the alternative model that SHOULD own them (CR-CH-053)
has already been built and does own them. Performing a B080 CustomWorkspace/
Function migration now would:
- re-implement logic CR-CH-053 already owns (duplication, brief b-iii);
- require a framework change (new variant / param-carrying Function) that needs
  express owner confirmation (framework-conformance.md); and
- risk regressing the active-wins ordering, B062 case rule, and `parse_two_args`
  quoting that FFEDIT now carries verbatim.

### Verification (scoped, Kiro-run)

- `cargo check -p ff-desktop`: Finished dev profile, clean (log
  `tools/logs/b080-step4-check.txt`). The tree is already consistent and
  compiling -- NOTHING was edited for a migration (zero behaviour change). No
  new tests were added because no code changed; the CR-CH-053 behaviour-
  preservation is covered by the existing shell suite + the `environment.rs`
  unit tests (`ffedit_claims_owned_verb_only_when_active_env_is_ffedit`,
  `ffedit_shadows_ffcmd_x_but_not_the_other_exit_verbs`, etc.).
- NOT run (owner's manual gate): `--workspace`, `cargo gate`, `ffwb-gate.ps1`.

## Recommendation to the owner

OWNER CONFIRMED (this session): record B080 Step 4 as DONE-BY-SUPERSESSION -- the
manager families were migrated off the `handle_command` ladder by CR-CH-053
(FFEDIT Command Environment, E1-E5/E7), not by a B080 CommandTarget migration,
because they are editor-action verbs that do not fit an existing CommandTarget
variant behaviour-preservingly. The zero-code-change, defer-and-report outcome is
the correct Step 4 result (like Step 3). The ORCHESTRATOR updates
`RESUME-ffdesktop-simplification.md` and the B080 bug note directly; this agent
does NOT edit those. No code change is required or appropriate for Step 4.

Open questions, dispositions (owner, this session):
- Steps 5 (split/detach/swap + workspace) and 6 (standalone + scrm) direction
  given CR-CH-053's trajectory: DEFERRED to the owner (recorded as an open
  question for the morning). Do NOT act on it.
- Step 7 (retire the Step-2 superseded CustomWorkspace/nav ladder arms) remains a
  valid B080 cleanup regardless of CR-CH-053.

## Status

Step 4 outcome: DONE-BY-SUPERSESSION (CR-CH-053), owner-confirmed. Zero code
change, zero behaviour change. Not committed; no worktree. RESUME doc + B080 note
updated by the orchestrator (not this agent). Steps 5-6 direction is an open
owner question deferred to the morning; Step 7 remains valid B080 cleanup.
