# CR-CH-052 Requirements-Gate Review

Reviewer: design-review subagent (fresh-eyes). Scope: DOCS-ONLY gate for
CR-CH-052 (uniform `=` reinitialise-to-POM + X / =X / END / RETURN, no POM
special-casing, plus the deferred B080 Step 7 ladder-arm deletion). No cargo /
gate run; judgement only, per the step brief.

Authoritative model: the owner's FINAL two-roots decision as recorded verbatim
in the original user request (messages 9-12) and in the CR-CH-052 change-log
entry. The gate-draft's open questions Q1-Q7 are OVERRIDDEN by that final model;
in particular the draft's per-tab-root interpretation of `=` is REJECTED.

Files inspected:
- docs/status/change-log.md (CR-CH-052 entry)
- docs/specs/command-framework/requirements.md (Req 10.2 revised, 10.14, glossary)
- docs/specs/command-framework/design.md (delta)
- docs/specs/menu-workspace/requirements.md (Req 1g, 14.4/14.5, 14.10, 14.13-14.16, glossary)
- docs/specs/menu-workspace/design.md (delta)
- docs/specs/menu-workspace/tasks.md (tasks 39-47)
- docs/specs/startup-and-session/requirements.md (Req 12, 46, 20.3)
- docs/specs/command-environments/requirements.md (FFEDIT carve-out cross-check)
- docs/project-management/project-master/tasks.md (Phase nav-ladder-semantics, NLS.1-6)
- docs/quality/TCR.md (NOT COVERED rows)
- .agents/tasks/crch052-gate/gate-draft.md (draft, for the overridden-questions baseline)

---

## Blocking owner-model checks

Every mandated check was verified against the written criteria. All PASS.

1. **`=` targets the POM (FFCMD_Root), addresses the FFCMD environment; per-tab-root
   REJECTED.** command-framework Req 10.2 (revised) reads: a `=`-prefixed command
   SHALL FIRST reinitialise the active tab's stack to the FFCMD_Root (the POM),
   THEN execute the remainder AGAINST THE FFCMD command environment (explicitly
   "not the Active_Environment such as FFEDIT/FFLINE"). The revision note states
   the ORIGIN meaning is UNCHANGED (`=` still means begin-from-the-POM) and that
   `=` is NOT the per-tab Tab_Visual_Root. The draft's per-tab-root reading is
   therefore rejected in the written criterion. `=1` resolving a POM option from
   any tab is covered by Req 10.14 (the `=<key>` POM-option fastpath seam) and the
   TCR row. PASS.

2. **`=X` = close-workspace / exit-when-last, not an always-exit literal.**
   menu-workspace Req 14.14 (new): `=X` reinitialises to the POM then runs `X`
   against FFCMD; being at the FFCMD_Root with an empty stack, `X` CLOSES the
   Workspace, app terminates only when it is the last; "SHALL NOT initiate an
   unconditional application exit". startup-and-session Req 12 and Req 20.3
   (revised) both remove `=X` from the unconditional-app-exit set. PASS.

3. **Bare `X` (FFCMD) collapses to the VISUAL root above it, else closes
   (exit-when-last); no POM special-casing; POM default command RETURN -> X.**
   menu-workspace Req 14.13 (new): non-empty stack -> collapse to Tab_Visual_Root;
   empty stack (already at root) -> CLOSE the Workspace, exit when last; "IDENTICAL
   for EVERY Workspace Context, including the Home Context (POM): no Context carries
   a bespoke `X` rule." Req 1g (revised) changes the default POM option command from
   `RETURN` to `X`, code-only (never written to disk, CR-CH-021 preserved). Bare `X`
   on the POM therefore closes it, not a return-to-Home no-op. PASS.

4. **RETURN folds into the uniform model and targets the VISUAL root (revising
   14.10 / CR-CH-038), not the POM.** menu-workspace Req 14.10 (revised) explicitly
   supersedes the CR-CH-038 POM-targeted rule: RETURN collapses to the active tab's
   Tab_Visual_Root (non-empty), closes at the root (exit when last), "RETURN is NO
   LONGER POM-targeted". PASS.

5. **END wording unchanged.** menu-workspace Req 14.4/14.5 still read "pop exactly
   one level per press" and "empty stack -> close the Workspace, exit when last".
   The 14.10 prose restates "END is unchanged: it pops ONE Navigation_Stack level".
   PASS.

6. **TWO-ROOTS distinction explicit in the glossaries; bottom/root terminology.**
   command-framework glossary defines FFCMD_Root (= the POM, a property of the
   FFCMD environment, GLOBAL and always available, targeted by `=`). menu-workspace
   glossary defines Tab_Visual_Root (the Context a tab was STARTed at, the BOTTOM of
   the Navigation_Stack, targeted by bare `X`/END/RETURN) and marks it DISTINCT from
   the FFCMD_Root. Navigation_Stack glossary states "the BOTTOM of the stack is the
   Tab_Visual_Root". Terminology uses bottom/root consistently; no ambiguous "top"
   for home. PASS.

7. **FFEDIT bare-`X`=EXCLUDE carve-out in BOTH menu-workspace Req 14 and
   command-environments; `=X` the uniform editor escape.** menu-workspace Req 14.13
   states the EXCEPTION: where the Active_Environment owns bare `X` (FFEDIT EXCLUDE,
   command-environments Req 5.1 / CR-CH-053 E8) the uniform FFCMD `X` is not reached;
   `=X` remains the uniform escape. command-environments carries the matching rules
   (Req 3.2a: `=X` in the editor is the base verb not EXCLUDE; Req 5.1: bare `X` =
   EXCLUDE; Req 5.2b / 6a context). The carve-out is framed as a deliberate
   environment-ownership exception, NOT a POM-style special-case. PASS.

8. **EXIT / QUIT / LOGOFF remain unconditional application exit; startup-and-session
   Req 12/20.3 revised to remove `=X` and state key clauses in terms of the bound
   command.** Req 12 (revised): EXIT/QUIT/LOGOFF = UNCONDITIONAL exit; `=X` removed;
   "Criteria are stated in terms of the BOUND COMMAND, never a hardwired Ctrl+X".
   Req 20.3 (revised): LOGOFF identical to EXIT/QUIT; `=X` no longer equivalent.
   PASS.

9. **B080 arm-deletion reroute captured in design + tasks.** command-framework
   design.md delta: reroute the three nav callers (POM option-key recursion
   commands.rs ~L163; chained-segment loop commands_fastpath.rs ~L128; START
   reconstruction nav_stack.rs ~L231) from `handle_command` to
   `dispatch_command_string`; then delete the 11 superseded arms (KEYS, KINDS;
   CONFIG, FILES/=FILES, GSEARCH/SEARCH, COMMANDS, MENUS, LOG, CATALOGS/FILE
   CATALOGS, PLUGINS, MACROS) plus the bare-THEME branch, "keeping `THEME <name>`".
   Recursion-safety note preserved. tasks.md 44 (reroute) and 45 (delete) + NLS.5
   mirror this. PASS.

10. **Detached-workspace close via the same path.** menu-workspace Req 14.16 (new):
    `X`-at-Tab_Visual_Root / `=X` close a Detached_Workspace via the SAME
    close-workspace-or-exit path as docked (menu-and-statusbar Req 18.3/18.11), exit
    when last. startup-and-session Req 46 restates "docked == detached close path".
    design.md Detached_Workspace-parity section present. Task 47 includes a detached
    close test. PASS.

---

## Gate-mechanics checks

- **EARS format + numbering.** New/revised criteria are EARS: 10.2, 10.14, 14.13,
  14.14, 14.16, 14.46/46 use "WHEN ... THE shell SHALL ..."; 14.15 is a ubiquitous
  "THE ... model SHALL be identical ..." (valid EARS). Numbering continues each
  sub-project's existing sequence (command-framework 10.14; menu-workspace
  14.13-14.16; startup-and-session new criterion within Req 14 and Req 20.3). PASS.
- **Design deltas.** Both command-framework/design.md and menu-workspace/design.md
  carry a CR-CH-052 delta; the menu-side delta correctly cross-references the
  command-framework delta as the dispatch-side source. PASS.
- **tasks.md.** menu-workspace tasks 39-47 are `[ ]` only, each cross-references the
  criteria it validates, independently completable, with the "do not start until
  approved" gate note. PASS.
- **project-master.** Phase (nav-ladder-semantics) with NLS.1-6 (`[ ]` only) and an
  updated Summary row for the phase present. PASS.
- **TCR.** One NOT COVERED (red-circle) row per new/revised criterion (10.2, 10.14,
  1g, 14.13, 14.14, 14.15, 14.16, 14.10, s&s 12/40/46/20.3) plus a B080-deletion
  row, all `ff-desktop`, with a justification that every row is headless-drivable
  via full-shell egui_kittest (so none is MANUAL -- consistent with testing.md).
  PASS.
- **change-log.** CR-CH-052 entry present, Status IN PROGRESS, the owner's final
  two-roots model recorded verbatim including the explicit rejection of the
  per-tab-root draft, the FFEDIT exception, the B080 fold-in, and "source under
  crates/ NOT touched". PASS.
- **ASCII cleanliness.** Spot-checked docs/specs/menu-workspace/requirements.md
  (prose-heavy, highest em-dash risk): zero non-ASCII matches. Corroborates the
  coder's full scan. PASS.
- **No source touched.** The CR entry and task notes assert crates/ untouched; this
  is a docs-only gate and nothing in the inspected docs implies otherwise. Per the
  brief, not re-verified by tooling. PASS (accepted).

---

## Findings

### NIT-1 (non-blocking): startup-and-session criterion numbering style
The change-log, TCR, and project-master refer to the revised startup-and-session
criteria as "Req 14.12 / 14.40 / 14.46", but the requirements.md file numbers the
same criteria as flat items "12." and "46." under their enclosing Requirement, and
"20.3" under Requirement 20. The cross-references interpret these as Requirement
14's criteria (14.12, 14.46), which matches the sub-project's own criterion-under-
Requirement convention and introduces no ambiguity about which criterion is meant.
No action required; recorded only so a future reader is not confused by the
"14.NN" label vs the bare "NN." in the file. Severity: NIT (does not affect the
verdict).

---

## Verified assumptions
- `=` keeps its original POM-origin meaning and is NOT generalised to per-tab-root
  (command-framework Req 10.2 revised text + revision note). Verified by reading
  the criterion, not by trusting the draft.
- `=` addresses FFCMD, so the Active_Environment never receives a `=`-prefixed
  string -- cross-checked against command-environments Req 3.2a, which independently
  states the same, so the two sub-projects agree.
- FFEDIT bare-`X`=EXCLUDE and `=X`-as-escape are specified in command-environments
  (Req 3.2a / 5.1 / 5.2b), satisfying "present in BOTH" without the gate needing to
  edit that file. Verified by grep + read.
- The 11 dead arms + bare-THEME-branch (keeping `THEME <name>`) and the three
  rerouted callers are enumerated identically in the design delta and in tasks 44/45.
- menu-workspace requirements.md is ASCII-clean (direct grep for [^\x00-\x7F]).

## Unverified / wrong assumptions
- None. No criterion was found to contradict the owner's final model, and no claim
  in the docs was found to be unsupported by the sibling specs it cites. The draft's
  per-tab-root interpretation (its Q3) and its tentative "RETURN stays POM" (Q5) were
  both correctly discarded in the written criteria, which was the single highest
  risk the brief flagged.

---

## Verdict

No HIGH or MEDIUM findings. One NIT (numbering-label style), which by the mechanical
rule does not block. The written gate matches the owner's final two-roots model
exactly on every mandated check; EARS, numbering, design deltas, tasks, project-
master, TCR, and change-log are all present and consistent; the docs are ASCII-clean
and no source was touched.

APPROVED.
