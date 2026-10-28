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

The single tool for this is `tools/powershell/liveness-probe.ps1` (default
`-Task probe`). It gathers the signals below AND emits the verdict in one run,
writing the full report to `_ignore/logs/liveness-probe.txt` and echoing it back,
so the orchestrator does not compute the verdict by hand. The verdict is ALSO
the exit code (0 ALIVE / 3 SLOWING / 4 STALLED), so the probe can gate a script:

```
C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -File tools\powershell\liveness-probe.ps1 -Path <worktree-or-root> -StaleSeconds 300 -RunLabel <run-label>
```

If running the probe is not possible (terminal mangling per `tooling.md`),
gather the same evidence with the dedicated file-info / search / git tools --
prefer those over the interactive shell. The signals it combines:

1. **Bucketed file mtimes.** Recent writes are split into WORK (source / docs /
   tasks / configs), GATE (`.gate\` workflow logs), and BUILD (`target\`
   compiler output). The liveness signal is the FRESHER of newest-WORK and
   newest-GATE; BUILD churn is shown for visibility but deliberately EXCLUDED
   from the verdict, so a background `cargo build` cannot mask a stalled agent.
2. **Heartbeat.** With `-RunLabel`, the last-write time of
   `_ignore/logs/<run-label>-progress.txt` is folded into the liveness signal and
   its last line is printed (see the heartbeat convention below).
3. **Last git commit.** The latest commit time/subject is read; when the newest
   work write is NEWER than the last commit, the probe flags "uncommitted work
   in flight" -- the common "produced output but has not committed yet" stall
   shape.

The verdict (the probe prints it; this is what it means) compares the signal age
to `-StaleSeconds` (default 300):

- **ALIVE** (exit 0) -- newest work/gate/heartbeat write within 1x the threshold;
  actively writing. Cite the evidence ("`editable_surface.rs` modified 40s ago").
- **SLOWING** (exit 3) -- between 1x and 2x the threshold. Quiet; may be mid
  build/review (a long `cargo test`, a reviewer reading before it writes its
  verdict). RE-CHECK before calling it; do NOT abort yet. This is the SIGNAL TO
  FLAG, not an automatic "it is dead".
- **STALLED** (exit 4) -- older than 2x the threshold, or nothing written at all.
  Treat as hung or restart-paused; FLAG it with the evidence and let the user
  decide (wait, inspect the step's session, or abort/relaunch). Do NOT silently
  abort a run on suspicion.
- **DONE / FAILED** -- orthogonal to the probe: the terminal artifact exists
  (note/review/verdict) or a terminal notification arrived; report the outcome.

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

The mtime/artifact check above is reliable but INDIRECT. To make the ALIVE /
SLOWING / STALLED call a one-line read, every delegated workflow BRIEF the
orchestrator authors SHOULD instruct each long-running step to append a
timestamped progress line to `_ignore/logs/<run-label>-progress.txt` as it works
(mirroring how the canonical gate `cargo gate` streams per-step progress to
`.gate/gate.timing.log` / `.gate/gate.history.csv`). Keep these logs ephemeral
(`_ignore/logs/` is git-ignored -- the whole `_ignore/` tree). Pass the matching
`-RunLabel <run-label>` to
`tools/powershell/liveness-probe.ps1` and it folds the heartbeat's last-write
time into the liveness signal and prints its last line; when the heartbeat is
absent the probe falls back to the WORK/GATE mtime and git-commit evidence above.
Adopting the heartbeat is RECOMMENDED for every new multi-step or long-running
workflow; the evidence-based check (the rule above) is MANDATORY regardless.

## Rationale

Delegation keeps the orchestrator responsive, but it also hides whether the
delegate is alive. A status report is only useful if it reflects reality; an
evidence-free "still running" has twice reported healthy on work the user
suspected was stalled. Grounding every status in a deliverable the agent either
did or did not produce makes the report trustworthy and turns a hang into
something caught in minutes rather than discovered much later.
