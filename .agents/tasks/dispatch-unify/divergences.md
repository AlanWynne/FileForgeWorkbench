# Dispatch Divergence Enumeration (B080)

READ-ONLY investigation. No source changed. Supports Phase 3 task 7 (B080):
unifying command dispatch so `ff_command::resolve_target` is the single front
door for the typed command line, menu options, and keyboard seams. Classified by
the owner as a CONFORMANCE fix against EXISTING command-framework criteria
(Req 2.1, 2.7, 8.3, 8.4, 9.7), NOT a new requirement.

All file:line references are to the tree as read on this pass. Plain ASCII only
(`--`, `->`, straight quotes).

## Summary answer

There are THREE input seams that submit a command string:

- TYPED (Enter / arrow-history): `render_command_line.rs` ->
  `run_command_line` (`commands.rs:27`) -> `handle_command` (`commands.rs:81`).
- KEY (function key / Key_Label_Bar click): `update.rs:305`,
  `render_status.rs` (clicked slot), `update_keys.rs:83` (detached) ->
  `dispatch_key_command` (`target_dispatch.rs:116`) -> `dispatch_bound_command`
  (`target_dispatch.rs:89`) -> `resolve_and_dispatch_command`
  (`target_dispatch.rs:66`) -> `ff_command::resolve_target` -> on FallThrough,
  `handle_command`.
- MENU OPTION (click / Tab+Enter): `activate_menu_option`
  (`commands_menu.rs`) -> inline target dispatched directly, else
  `resolve_and_dispatch_command` -> on FallThrough, `handle_command`.

The crux of B080: the TYPED path jumps STRAIGHT to `handle_command` and NEVER
calls `resolve_target`, while the KEY and MENU paths go THROUGH `resolve_target`
first. Because `ShellTargetResolver::builtin_workspace_target` is a `None` stub
(`command_config/mod.rs:167`), every built-in verb FallThrough's on the
menu/key path and lands in the SAME `handle_command` ladder -- so today the two
paths MOSTLY converge by accident at `handle_command`. The genuine, observable
divergences live in the PRE-PROCESSING each path applies before/around
`handle_command`, enumerated below. The single largest is the KEY path's
field-merge (Req 9.8) which has no typed-path analogue, and the fact that the
typed path's `handle_command` is a ~60-branch if-ladder that does its own
ad-hoc per-verb arg parsing instead of one Req 9.7 split at the boundary.

Note on what is ALREADY correct (preserve it): the menu-option CLICK seam and
the typed menu-name path were deliberately unified onto ONE menu opener,
`open_menu_by_name` (`commands_menu.rs`), by CR-CH-043/B075; and the
current-menu Option_Key stage is ONE resolver, `try_current_menu_option`
(`commands_menu.rs`), shared by typed and click (CR-CH-043). The migration must
not regress those.

---

## D1 -- resolve_target is on the KEY/MENU path but NOT the TYPED path

- TYPED: `commands.rs:27` `run_command_line` calls `handle_command` directly;
  `handle_command` (`commands.rs:81`) never constructs a `ShellTargetResolver`
  and never calls `ff_command::resolve_target`.
- KEY/MENU: `target_dispatch.rs:66` `resolve_and_dispatch_command` builds a
  `ShellTargetResolver` and calls `ff_command::resolve_target(input, &resolver)`;
  only on `Err` (FallThrough) does it call `handle_command`.
- Difference: a user-defined Command_Definition id (stage 1,
  `user_command_target`) and a registered Command_ID (`is_registered_command`)
  resolve to a target ONLY on the key/menu path. Typing that same string in the
  command field skips user-definition and registry resolution entirely and runs
  the ladder, which may not recognise it -> different observable result for a
  user-defined id (dispatched by key, "unresolved" by typing).
- User-observable: YES (a user-defined command id works from a bound key/menu
  option but not from the command line).
- RECOMMENDATION: CONVERGE. Route the typed path through `resolve_target` too so
  all three seams share one front door (Req 2.1 single entry point; Req 8.3
  ordered chain; Req 8.4 same observable result for every string that resolves).

## D2 -- Argument merge: KEY path merges the field, TYPED path does not

- KEY: `target_dispatch.rs:116-128` `dispatch_key_command` reads
  `self.command_text` (`original`), and when non-empty synthesises
  `format!("{} {}", command.trim(), original)` before dispatch (Req 9.8:
  type `8`, press DOWN -> `DOWN 8`).
- TYPED: `commands.rs:27` `run_command_line` passes the field text verbatim; the
  verb and argument are already one string the user typed.
- Difference: this is the specified, intended behaviour difference between the
  two input SOURCES, not a divergence in how a given string is processed. It is
  the Req 9.8 field-as-argument rule and has no typed analogue by design.
- User-observable: YES, but intended.
- RECOMMENDATION: PRESERVE. Req 9.8 mandates the key-path merge; the typed path
  correctly has no merge. HOWEVER the merge must remain the ONLY place a split is
  synthesised; see D3/D7 -- after the merge, both paths must perform the SAME
  single verb/arg split at the dispatch boundary (Req 9.7). Locked by
  `tests_command.rs` SWAP-key tests (e.g. `1` + SWAP key -> `SWAP 1`,
  field cleared, around `tests_command.rs:628-694`).

## D3 -- The Req 9.7 single verb/arg split does not exist; the ladder re-parses per verb

- TYPED and (after merge) KEY both end in `handle_command`, which does NOT split
  once into (verb, arg). Instead EACH ladder arm re-derives its own argument:
  - `verb_arg(cmd, "DOWN")` etc. (`helpers.rs:242`) -- case-insensitive verb,
    case-preserved remainder.
  - hand-counted byte slices: `cmd.trim()[8..]` for EXCLUDE
    (`commands_ladder_b2.rs`), `cmd.trim().get(4..)` for SWAP
    (`commands_ladder_c.rs`), `cmd.trim().get("SNAPSHOT".len()..)`
    (`commands_ladder_b.rs`), `cmd.trim().get(9..)` for WORKSPACE
    (`commands_ladder_c.rs`), etc.
  - `upper.strip_prefix("COMMAND ")` (`commands_ladder_b.rs`).
- `ff_command`'s `execute_command` (`dispatch.rs:80`) takes an ALREADY-parsed
  `id` + `params`; it does NOT insert the Req 9.2 `arg` param. So the spec'd
  "parse once, place Argument_String into `params.arg`" (Req 9.2, 9.7) is not
  implemented anywhere -- each arm improvises.
- Difference: not typed-vs-key, but the ladder's per-arm parsing is the reason
  the two paths can only "converge by accident" and is the structural blocker to
  one boundary split. A merged `SWAP 1` from the key path and a typed `SWAP 1`
  reach the same arm, but any arm that keys on exact-string `upper ==` (not
  `verb_arg`) can treat a trailing space / argument differently from the merged
  form.
- User-observable: POTENTIALLY (exact-match arms vs prefix arms treat a stray
  argument differently -- see D8).
- RECOMMENDATION: CONVERGE. Perform ONE verb/arg split at the dispatch boundary
  (Req 9.7) and feed `params.arg` (Req 9.2); table handlers read `arg` instead of
  re-slicing. Behaviour-preserving per verb (Req 8.4).

## D4 -- Command_Line_Outcome wrap: applied on BOTH submit paths, NOT on raw handle_command

- TYPED: `run_command_line` (`commands.rs:27`) wraps `begin_command_line()` ...
  `handle_command` ... `finish_command_line(&original)`.
- KEY: `dispatch_key_command` (`target_dispatch.rs:116`) ALSO wraps
  `begin_command_line()` ... dispatch ... `finish_command_line(&original)`.
- Difference: NONE between the two SUBMIT paths -- both apply the outcome pass
  (CR-CH-033, Req 13.1). The asymmetry is only that a RAW `handle_command` call
  (recursion inside the ladder, e.g. AUTONUM -> NUMBER redirect in
  `commands_ladder_c.rs`, or chained-fastpath segment re-dispatch in
  `commands_fastpath.rs`) is NOT wrapped -- correctly, because the OUTER submit
  already wrapped the whole invocation.
- Important subtlety for KEY: `dispatch_key_command` passes `&original` (the
  field text the user typed) to `finish_command_line`, NOT the merged
  `<command> <field>` string -- so a Restore brings back what the user typed, not
  the synthesised merge. The typed path passes its `original` (the typed line).
  These are consistent in intent (Restore == "what the user typed").
- User-observable: outcome handling is identical where it matters.
- RECOMMENDATION: PRESERVE / CONVERGE-by-construction. When the typed path is
  routed through the shared front door (D1), keep exactly ONE outcome wrap at the
  outermost submit boundary for all three seams (Req 13.1 "single decision
  point"). Do not wrap inner re-dispatches. Locked by `tests_session.rs`
  (Restore-on-error, e.g. `run_command_line("ZXQWV nonsense")` keeps the text,
  around `tests_session.rs:160-173`).

## D5 -- Uppercase handling: verb case-insensitive, args case-preserving (B062)

- `handle_command` computes `upper = cmd.trim().to_uppercase()`
  (`commands.rs:82`) and most arms branch on `upper == "..."` or
  `upper.starts_with("... ")`, then re-slice the ORIGINAL `cmd` for the argument
  to preserve its case (e.g. RESET BARE reads `cmd.trim().get("RESET BARE".len()..)`
  for case-preserved profile names, `commands_ladder_b.rs`). `verb_arg`
  (`helpers.rs:242`) implements the same rule: `eq_ignore_ascii_case` on the
  head, case-preserved trimmed remainder.
- Both the TYPED and KEY paths funnel into this same `handle_command`, so B062 is
  honoured identically on both -- the merge in `dispatch_key_command` preserves
  the field's case in `original`, and the ladder preserves it from there.
- `resolve_target` itself trims but does case-insensitive matching
  (`command_target.rs:267`); `ShellTargetResolver::menu_name_target` lowercases
  the resolved name (`command_config/mod.rs`). Menu names are case-insensitive
  by spec (Req 8.3 "matching is case-insensitive").
- User-observable: NO divergence between paths today.
- RECOMMENDATION: PRESERVE. The B062 rule (case-insensitive verb, case-preserving
  argument) must survive the migration. The one boundary split (D3) must produce
  a verb token for case-insensitive table lookup and a case-PRESERVED
  Argument_String for `params.arg`. Locked by
  `verb_arg_matches_case_insensitively_and_preserves_argument`
  (`tests_command.rs:322`).

## D6 -- Current-menu Option_Key stage (Req 8.3 stage 1): only on handle_command, applied correctly

- TYPED and (post-FallThrough) KEY/MENU: stage 1 runs as the FIRST thing in
  `handle_command` via `try_current_menu_option(cmd)` (`commands.rs`, calling
  `commands_menu.rs`). It activates a matching Option_Key of the ACTIVE menu and
  returns.
- `resolve_target` does NOT implement stage 1 (its doc explicitly says "The
  current-menu Option_Key stage (Requirement 8.3 stage 1) is applied by the shell
  BEFORE this function is consulted" -- `command_target.rs:258`). But the KEY/MENU
  path calls `resolve_target` FIRST and only reaches `handle_command` (hence
  stage 1) on FallThrough.
- Difference / HAZARD: for the KEY/MENU path, `resolve_target` runs stages 2-4
  (user def, registry, menu-name) BEFORE stage-1 current-menu Option_Key is ever
  consulted (stage 1 only runs if resolve_target FallThrough's). For the TYPED
  path, stage 1 runs FIRST (top of handle_command) and resolve_target is never
  called. So the ORDER of stage 1 vs stages 2-4 is INVERTED between the paths.
  In practice an Option_Key is a short token unlikely to equal a registered
  Command_ID, so the inversion rarely bites -- but it is a latent Req 8.3
  ordering violation (stage 1 must precede stage 2).
- User-observable: EDGE (only when a menu's Option_Key string also equals a
  user-defined id / registered Command_ID / menu name).
- RECOMMENDATION: CONVERGE. The unified front door must apply stage 1
  (current-menu Option_Key) BEFORE delegating to `resolve_target` stages 2-5, for
  ALL seams, matching the single ordered chain of Req 8.3. Today the typed path
  has the order right; the fix is to give the key/menu path the same ordering
  (stage 1 first) rather than resolve_target-first.

## D7 -- POM fastpath / config-driven Option_Key (`resolve_pom_option_key`): only in handle_command

- `handle_command` (`commands.rs`) runs `resolve_pom_option_key(&upper)`
  (`commands_fastpath.rs`) after stage 1, resolving a bare key (`1`, `S`) or
  `=1` against pom.toml and recursing into `handle_command` with the resolved
  Option_Command.
- Only reached by the typed path directly and by the key/menu path on
  FallThrough. `resolve_target` has no POM-fastpath notion.
- Difference: same shape as D6 -- order relative to resolve_target stages is
  inverted between paths, but converges at handle_command in the common case.
- User-observable: EDGE.
- RECOMMENDATION: CONVERGE (as part of the single ordered chain -- POM fastpath
  is a shell-local extension of stage 1/2 and must run at the same point for all
  seams). Behaviour per key is config-driven and must not change (Req 8.4).
  Locked by POM option tests in `tests_nav.rs` (typed key vs clicked row give
  identical result, around `tests_nav.rs:625-662`).

## D8 -- Chained fastpath (`=0.K`, `3.1`): only in handle_command; re-dispatches per segment

- `try_chained_fastpath(&upper)` (`commands_fastpath.rs`, called from
  `commands.rs`) splits on `.`/`;`, optionally pops to POM for a leading `=`, and
  re-dispatches each segment through `handle_command`.
- Reached by the typed path directly; by the key/menu path only on FallThrough.
  A chained path is unlikely to be a user-def id or registered Command_ID, so
  resolve_target FallThrough's and the typed and key paths converge.
- SUBTLE HAZARD: the KEY path merges the field first (D2). If the field holds
  `=0.K` and a key bound to some verb is pressed, the merge yields
  `<verb> =0.K`, which is NOT a chained path (first token is `<verb>`), so the
  chained handler would not fire. This is the intended Req 9.8 semantics (the key
  supplies the verb, the field the argument), not a bug -- but it is a place
  where "type then Enter" and "type then press key" legitimately differ.
- User-observable: YES for the merge case (intended, Req 9.8).
- RECOMMENDATION: PRESERVE the per-segment re-dispatch behaviour and the
  `=`-origin rule; CONVERGE only insofar as every segment re-dispatch must go
  through the SAME unified front door so a segment that is itself a user-def /
  menu name resolves consistently. Locked by chained-fastpath tests
  (`=0.M` in detached context, `tests_split_detach.rs:1036`; `=0.K` family in
  `tests_nav.rs`).

## D9 -- Branch-ORDER dependencies in the ladder (ordering hazards)

The if-ladder's correctness depends on arm ORDER in several places. These are
not typed-vs-key divergences (both paths share the ladder) but they are the
hazards the verb-table removes, and a divergence risk if any arm is ever
duplicated onto only one path:

- `COMMAND` (singular) is checked BEFORE `COMMANDS` (plural) in
  `commands_ladder_b.rs`; the code comments it is "disjoint" but relies on the
  `upper == "COMMAND"` / `starts_with("COMMAND ")` arms sitting before the
  `upper == "COMMANDS"` arm.
- `SPLIT DETACH` is matched in `commands_ladder_c.rs` (DETACH arm) BEFORE the
  bare `SPLIT` / `SPLIT RIGHT` / `SPLIT DOWN` arms, so the deprecated alias is
  claimed first.
- `EXCLUDE ALL` / `X ALL` are checked before the general `EXCLUDE ` / `X `
  prefix arms (`commands_ladder_b2.rs`).
- `RESET BARE` is checked (in `commands_ladder_b1`) before the generic
  `RESET` / `RESET EXCLUDED` / `RESET ALL` arm (`commands_ladder_b2.rs`) -- across
  TWO submodules, so the ordering dependency spans files.
- `RETRIEVE` is matched by prefix (`starts_with("RETRIEVE ")`) to catch the
  B066/B067 merged `RETRIEVE <field>` form (`commands_ladder_b.rs`).
- Stage ordering: `try_commands_a` -> `resolve_pom_option_key` ->
  `try_commands_b1` -> `try_commands_b2` -> `try_chained_fastpath` -> NAME ->
  `try_commands_c` -> `try_menu_name_dispatch` -> engine. A verb added to the
  wrong submodule changes precedence.
- User-observable: a reordering bug would be; today it is correct.
- RECOMMENDATION: CONVERGE. A verb-token dispatch table keyed on the parsed verb
  removes all intra-ladder ordering hazards: `COMMAND` and `COMMANDS` are
  distinct keys; `SPLIT DETACH` vs `SPLIT` becomes a sub-parse inside the SPLIT
  entry, not a cross-arm order; multi-word verbs (`RESET BARE`, `EXCLUDE ALL`)
  become explicit sub-dispatch within one entry. See design-delta.md section 3.

## D10 -- Inline menu-option target bypasses resolve_target (menu path only)

- `activate_menu_option` (`commands_menu.rs`) dispatches an inline
  `[options.target]` DIRECTLY via `dispatch_command_target` (Req 10.6), WITHOUT
  calling `resolve_target`. Only when there is no inline target does it call
  `resolve_and_dispatch_command`.
- The typed and key paths have no inline-target notion.
- Difference: intended -- an inline target is already a resolved CommandTarget,
  so re-resolving a string would be wrong.
- User-observable: NO (correct by design).
- RECOMMENDATION: PRESERVE. An already-resolved CommandTarget dispatches straight
  to `dispatch_command_target`; only STRING inputs go through `resolve_target`.

## D11 -- History recording point differs

- TYPED/KEY (via handle_command): `self.command_line_history.record(cmd)` runs at
  the TOP of `handle_command` (`commands.rs`), once per submitted line, for the
  string that reaches handle_command.
- For the KEY path the string reaching handle_command (on FallThrough) is the
  MERGED `<command> <field>`; for the typed path it is the typed line. So the KEY
  path records the merged form (e.g. `SWAP 1`), the typed path records `SWAP 1`
  only if that is what was typed. The RETRIEVE-verb exclusion in `record` handles
  the merged `RETRIEVE <field>` form.
- Difference: both record the effective executed line; consistent in intent.
- User-observable: EDGE (history shows the merged line for key invocations,
  which is the executed command -- arguably correct).
- RECOMMENDATION: PRESERVE, but ensure the unified front door records ONCE at the
  single boundary (not per inner re-dispatch), matching today's top-of-
  handle_command single record.

---

## Divergence decision table (fed into design-delta.md section 6)

| ID | What differs | Observable | Decision | Conforms to |
|----|--------------|------------|----------|-------------|
| D1 | typed skips resolve_target; key/menu use it | YES | CONVERGE | Req 2.1, 8.3, 8.4 |
| D2 | key merges Command field as arg; typed does not | YES (intended) | PRESERVE | Req 9.8 |
| D3 | no single verb/arg split; ladder re-parses per arm | POTENTIAL | CONVERGE | Req 9.2, 9.7 |
| D4 | Command_Line_Outcome wrap | identical on submit paths | PRESERVE | Req 13.1 |
| D5 | verb case-insensitive, arg case-preserving (B062) | NO divergence | PRESERVE | Req 8.3 (case-insensitive), B062 |
| D6 | stage-1 current-menu Option_Key order vs resolve_target | EDGE | CONVERGE | Req 8.3 stage order |
| D7 | POM fastpath / config Option_Key order | EDGE | CONVERGE | Req 8.3, menu-ws 2.1e/2.1i |
| D8 | chained fastpath + merge interaction | YES (intended) | PRESERVE (route segments through 1 door) | Req 5, 9.8 |
| D9 | ladder branch-ORDER dependencies | latent | CONVERGE (verb table) | Req 8.10 |
| D10 | inline menu target bypasses resolve_target | NO (correct) | PRESERVE | Req 10.6 |
| D11 | history record point / merged form | EDGE | PRESERVE (record once) | Req 7, 9.8 |

## Behaviour ambiguities for the owner to confirm

- A1 (D6/D7 ordering): today the TYPED path runs stage-1 current-menu Option_Key
  and the POM fastpath BEFORE any user-def/registry/menu-name resolution, while
  the KEY/MENU path runs resolve_target (stages 2-5) first and only reaches
  stage 1 on FallThrough. Req 8.3 says stage 1 is first. Confirm the unified
  front door must apply stage 1 (and the POM fastpath) BEFORE resolve_target for
  ALL seams (this makes the key/menu path match the typed path's ordering).
- A2 (D11): confirm history should record the EXECUTED (post-merge) line for
  key invocations, matching today's behaviour, rather than the bare bound verb.
- A3 (D2/D8): confirm the field-merge (Req 9.8) remains the only synthesis of a
  command string on the key path, and that a chained path typed into the field
  combined with a bound key is intended to merge (not chain) -- i.e. preserve
  current behaviour.
