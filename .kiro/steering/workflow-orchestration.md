---
inclusion: always
---

# Workflow / Agent Liveness -- Evidence-Based Status Checks

When background work is delegated to a workflow or agent, the orchestrator must
be able to tell the difference between an agent that is STILL PRODUCING
deliverables and one that has HUNG. The workflow run-state alone cannot tell them
apart, so a status report that relies on it is unreliable. This rule exists so
that "is it still working?" is always answered with EVIDENCE, not a guess.

## The problem this prevents

`inspect_workflow` reports a step's LIFECYCLE state (running / completed /
failed), not the agent's actual activity. A step reads "running" from the moment
it starts until it emits a terminal signal -- a healthy agent mid-work and a hung
agent look IDENTICAL. Reporting "still busy" purely from that state is therefore
not a real status; it only means "not yet reported done."

## The rule -- MANDATORY on every status request about in-flight work

WHEN the user asks for a status or progress update AND a workflow/agent run is
in flight, the orchestrator MUST NOT answer from `inspect_workflow`'s step state
alone. It MUST corroborate with at least one piece of DELIVERABLE EVIDENCE from
the filesystem (only a working agent produces these), then report an explicit
verdict.

Evidence to check (prefer dedicated tools -- file info / search / git -- over the
interactive shell, per `tooling.md`):

1. **File mtimes of the run's target files.** Recently modified (within the last
   few minutes) = alive. Use the known target paths for the task (for a coding
   phase, the source files it was told to edit/create).
2. **Expected-artifact presence/absence.** Each phase/step writes known artifacts
   at known stages (e.g. a `phaseN-note.md` only at code-complete, a
   `phaseN-review.json` only at review-done). Their presence pins the REAL stage;
   their absence means that stage is not reached yet.
3. **git status churn.** A growing/working-tree diff is progress; a frozen diff
   is a signal to look closer.

Then report EXACTLY ONE verdict, with the evidence cited:

- **BUSY** -- cite the evidence ("`editable_surface.rs` modified 3 min ago" /
  "diff grew since last check"). Say what stage it is in.
- **POSSIBLE HANG** -- no deliverable activity (no target-file mtime change, no
  new artifact, no diff churn) for a sustained period (soft threshold ~15-20 min
  on a step that should be writing code). This is a SIGNAL TO FLAG, not an
  automatic "it is dead": some steps are legitimately quiet (a long `cargo test`
  run, a reviewer reading before it writes its verdict at the end). FLAG it to the
  user with the evidence and let them decide (wait, inspect the step's session,
  or abort/relaunch). Do NOT silently abort a run on suspicion.
- **DONE / FAILED** -- the terminal artifact exists (note/review/verdict) or a
  terminal notification arrived; report the outcome.

NEVER report "still working" / "still busy" without backing evidence. If the
dedicated tools and the shell both fail to yield evidence (e.g. terminal mangling
per `tooling.md`), say so plainly rather than inferring liveness.

## Scope -- when this fires, and when it does NOT

- Fires ONLY when a workflow/agent run is genuinely in flight at the time of the
  status request.
- Does NOT fire for a plain status question when nothing is running -- answer
  that directly; no filesystem audit.
- Does NOT fire for a read-only lookup or a question unrelated to delegated work.

## The heartbeat convention -- make the check trivial for future runs

The mtime/artifact check above is reliable but INDIRECT. To make busy-vs-hung a
one-line read, every delegated workflow BRIEF the orchestrator authors SHOULD
instruct each long-running step to append a timestamped progress line to
`tools/logs/<run-label>-progress.txt` as it works (mirroring how the canonical
gate `cargo gate` streams per-step progress to `.gate/gate.timing.log` /
`.gate/gate.history.csv`). Keep these logs ephemeral (`tools/logs/` is
git-ignored). When present, the liveness
check is just reading the last line's timestamp; when absent, fall back to the
mtime/artifact evidence above. Adopting the heartbeat is RECOMMENDED for every
new multi-step or long-running workflow; the evidence-based check (the rule
above) is MANDATORY regardless.

## Rationale

Delegation keeps the orchestrator responsive, but it also hides whether the
delegate is alive. A status report is only useful if it reflects reality; an
evidence-free "still running" has twice reported healthy on work the user
suspected was stalled. Grounding every status in a deliverable the agent either
did or did not produce makes the report trustworthy and turns a hang into
something caught in minutes rather than discovered much later.
