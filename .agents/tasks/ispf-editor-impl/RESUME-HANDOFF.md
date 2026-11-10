# Resume handoff -- ISPF editor concerns (CR-CH-076/077/078)

Written 2026-11-01 end of session.

## main (all pushed to origin)
- 1532a5f  Group B Alias editor (shared ISPF table editor + Description col + I/D/C + SORT)
- b68f823  in-cell-Enter fix (line commands apply on Enter in the Act cell)
- 328b29c  requirements gate docs for CR-CH-076/077/078  <-- current main tip

## In progress -- NOT merged, NOT reviewed, NO full gate yet
Branch `ispf-editor-impl` (worktree .worktrees/ispf-editor-impl, off 328b29c):
- 387783b  FEAT-001 block commands RR/DD/CC/MM (scoped tests green)
- cc8948e  FEAT-002 arrow nav Up/Down rows + Left/Right cols (scoped tests green)
- 8d3c76f  FEAT-003 single-chrome (scoped tests green)

The implementation WORKFLOW FAILED at feat-003: the review + finalize steps
never ran. So: no code review, no full cargo gate.

## Why the running build (main 328b29c) still shows 2 command lines + 2 headers
The single-chrome fix is on branch ispf-editor-impl, which has NOT been merged to
main. Expected until the branch lands.

## THE ONE OPEN DECISION (needed before merge)
FEAT-003 resolved the duplicated Title + Command ===> line by making the SHELL's
app-level Title + command field the single ones, and routing the shell command
field into the Alias handler when the Alias Context is active (apply pending Act
line commands -> FILTER/RESET/SORT -> fall through to the FFCMD front door for
SAVE/END/RETURN/REFRESH). The surface no longer draws its own title/command line.
This is essentially OPTION A done at the shell level.
- Owner has NOT yet explicitly approved this approach.
- It DIVERGES from the literal approved spec wording (Req 10.1/10.2/7.12, which
  say the COMPONENT owns the command line) -- needs a small gated spec edit to
  reconcile code and criteria.
Alternatives were B (keep shell line, rework the line-command/save ENTRY seam --
contradicts Req 10.4, risky) and C (cosmetic: drop the surface label only -- not
true single-chrome). Orchestrator recommended A.

## NEXT STEPS when resuming
1. Owner decides: approve FEAT-003's option-A approach, or redirect to B/C.
2. Orchestrator launches a short workflow to REVIEW the three FEATs together and
   RECONCILE the spec wording (so code matches criteria), stopping before merge.
3. Owner runs a full `cargo gate` on .worktrees/ispf-editor-impl (must be CLEAN)
   + manual check: block commands (RR..RR / DD..DD / CC..CC with A/B), arrow nav
   (Up/Down rows, Left/Right cols), single-chrome live panel + snapshot agree.
4. Orchestrator squash-merges ispf-editor-impl to main, pushes, cleans up worktree.

## Also pending (owner report, not yet actioned)
- Config editor snapshot: owner started "when snapshotting the configuration
  editor we are getting the following" but did not paste the content. Re-ask next
  session -- likely another snapshot-fidelity concern for the Config editor.

## Housekeeping (owner action, non-blocking)
Leftover worktree dirs held by Kiro's indexer -- a Kiro window reload frees them
for deletion: .worktrees/group-a-capture-fidelity, group-b-alias, group-b-reqs
(and ispf-editor-impl once its work is merged).
