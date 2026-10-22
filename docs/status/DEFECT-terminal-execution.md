# DEFECT / ISSUE LOG -- Kiro Terminal & Tool-Call Execution Instability

Status: OPEN
First observed: on/before 2026-10-15; still active 2026-10-21.
Severity: HIGH -- it repeatedly blocks automated verification/commit steps and
has stalled four delegated workflow runs, forcing manual recovery each time.
Environment: Windows; integrated shell PowerShell 7.6.3 (`ConsoleHost`);
interpreters `C:\tools\powershell7\pwsh.exe` and `C:\tools\python\python.exe`.
Kiro Shell Integration: ENABLED.
Possibly relevant (open editor at time of writing): `~/.kiro/settings/sandbox.json`
-- the sandbox/execution configuration is a candidate area to inspect (NOT yet
investigated; flagged for the defect owner).

This document is a consolidated defect report compiled from the project steering
rules, `docs/PATTERNS-AND-LESSONS.md` (Part B), and direct first-hand incidents
accumulated across multiple sessions. It describes (1) the observed problems and
(2) every workaround put in place to cope with them.

---

## 1. Summary

Two distinct but related failure families have made terminal-driven and
tool-driven work unreliable on this machine:

- **FAMILY 1 -- Shell execution-layer quirks.** Commands run through Kiro's
  shell runner intermittently: report a spurious `Exit Code: -1` on success,
  have their stdout swallowed entirely, or do not execute at all for a stretch
  ("dead episodes"). Long/quiet commands (full test suites, full gates) make it
  worse and can wedge the channel.
- **FAMILY 2 -- Malformed tool-call emission.** Separately and more recently, the
  assistant's tool calls themselves intermittently fail to emit as structured
  calls and instead render as garbled plaintext (lines beginning `court ...`),
  so the call never runs. This affects ALL tools (file read, shell, workflow),
  comes and goes within a single turn, and was NOT cleared by a Kiro restart.

Both families cost round-trips, block automated verification, and have repeatedly
forced the human owner to run commands manually in an external terminal.

---

## 2. FAMILY 1 -- Shell execution-layer quirks (detailed)

### 2.1 Phantom `Exit Code: -1` (capture artifact)
- SYMPTOM: Kiro's shell runner appends `Exit Code: -1` even when the command
  SUCCEEDED -- real output printed, files written, prompt returned clean.
- CHARACTER: `-1` is never a real process exit code. Genuine program failures
  surface as the real non-zero code (1, 2, 101 for cargo). The `-1` is applied
  by the tool wrapper OUTSIDE the shell, so no choice of shell removes it.
- TRIGGER: most easily provoked by commands that go quiet near completion or
  redirect their final output away from the live stream.
- IMPACT: if treated as a real failure, provokes a reflexive retry loop -- which
  itself aggravates the instability.

### 2.2 Swallowed output
- SYMPTOM: the command runs (files on disk prove it) but its stdout -- and
  sometimes a completion sentinel -- never reaches the tool result; the `Output:`
  block is empty.
- IMPACT: success/failure cannot be read from the console; the result must be
  recovered from a file the command wrote.

### 2.3 Dead episodes
- SYMPTOM: a stretch (seconds up to ~1 minute, sometimes longer) where even a
  trivial command (`Write-Output`, `git status`) returns NOTHING and writes NO
  file -- i.e. it did not execute at all.
- CHARACTER: self-clearing; they pass on their own. Escalating (re-issuing more
  complex commands) prolongs them.
- IMPACT: makes any single command's result ambiguous (did it run or not?).

### 2.4 Wedged channel under load / long-running commands
- SYMPTOM: the integrated shell session stops responding for a long stretch,
  especially when a long/quiet command is running (a full `cargo test` /
  `cargo nextest` run, a whole-workspace `cargo gate` / `full-gate`, multi-minute
  suites) or when multiple compiles contend for the one channel.
- OBSERVED CONSEQUENCE: delegated workflow coder steps reliably DID the code work
  and compiled clean, but then HUNG at the step's final verify/commit/signal
  shell step -- not advancing. This happened on at least four occasions
  (FEAT-001, FEAT-003, Slice 2 crate1, plus agent-restart-induced pauses).
- OBSERVED MITIGATION (partial): enabling Kiro Shell Integration (2026-10-15)
  removed the hard continuation-prompt lockups but did NOT eliminate the phantom
  `-1` or the swallow/dead episodes.

### 2.5 PSReadLine echo mangling (historical, largely mitigated)
- SYMPTOM: long/complex or `;`-chained one-liners in the interactive PSReadLine
  session came back echoed character-by-character, or with a second command
  glued to the front (a poisoned buffer bleeding a stale fragment -- e.g. a
  spurious `cd "..." ;` prefix observed glued onto later commands this session).
- MITIGATION: the non-interactive pwsh7 wrapper + a session-start buffer clear
  (see workarounds below).

---

## 3. FAMILY 2 -- Malformed tool-call emission (newer, 2026-10-21)

- SYMPTOM: the assistant's tool invocation does not render as a structured tool
  call. Instead it appears as garbled plaintext whose leading token is `court`,
  followed by the call's parameters splattered inline, e.g.:
  `court Check slice2 worktree ... false & 'C:\tools\powershell7\pwsh.exe' ... 90000`
  or `court c:\workspace\...\slice2-mtimes.txt`.
- EFFECT: the call NEVER executes -- it is a no-op that consumes a turn and emits
  visible garbage.
- SCOPE: affects ALL tool types observed (file read, shell exec, workflow
  inspect/launch). `read_file` was the most frequent victim in late sessions;
  `execute_pwsh` and `str_replace` sometimes emitted cleanly in the same turn.
- INTERMITTENCY: comes and goes WITHIN a single turn -- one call emits cleanly,
  the next garbles.
- RESTART BEHAVIOUR: a full Kiro restart did NOT clear it (unlike earlier in the
  day when a restart appeared to help). This points to an environment/service
  level fault rather than a per-session glitch.
- IMPACT: when active, the assistant cannot reliably drive ANY multi-step tool
  work (edits, verification, commits), because it cannot guarantee the next call
  lands. Work must be handed to the human's (reliable) external terminal.

---

## 4. Workarounds and mitigations put in place

These are the measures adopted (some now encoded as machine-wide steering rules)
to cope with the above. They reduce but do not eliminate the problems.

### 4.1 Non-interactive pwsh7 wrapper (steering: `terminal-discipline.md`)
Every shell command runs as:
`C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<command>"`
- `-NonInteractive` stops the PSReadLine echo/prediction that caused the
  character-by-character mangling; `-NoProfile` keeps the session clean/fast and
  avoids the 5.1 profile banner.
- Every command also carries explicit no-prompt switches (`-Force`,
  `-Confirm:$false`, `--yes`) so the first invocation cannot block on input.

### 4.2 One command, one job (steering + guard hook)
NEVER `;`-chain steps, pipe into `Format-Table`/`Select-Object`, or inline
`$( ... )`. Multi-step logic goes into a `tools/` script run via `-File`.

### 4.3 The `pwsh-clean-output-guard` PreToolUse hook
`.kiro/hooks/pwsh-clean-output-guard.json` inspects every shell command before it
runs and classifies it:
- ALLOW (silent): only the clean non-interactive pwsh7 wrapper.
- ASK: a bare toolchain/git/gh leader, a bare inspection cmdlet, a
  Format-Table/Select-Object pipe (reason string carries the wrapped form).
- BLOCK (exit 2): ANY line containing `;`, or a process-kill
  (`Stop-Process`/`taskkill`/`kill`) glued onto a command.
NOTE: the `;` block is a blunt substring check with NO exception -- so an inline
completion sentinel (`; $rc=$LASTEXITCODE; Write-Output ...`) is itself blocked
(see 4.6).

### 4.4 The `session-start-buffer-clear` SessionStart hook
Clears the PSReadLine buffer once per session so stale/poisoned output cannot
bleed into / visually mangle the first commands.

### 4.5 Verify by EFFECT, never by the reported exit code (steering rule 5)
Do not treat a bare `Exit Code: -1` as failure and do not reflexively re-run.
Establish the real result from a durable side effect:
- redirect the command's full output to a log and READ the log back;
- confirm a produced artifact with `Test-Path` / read the file / `git status`;
- the program's REAL non-zero code appears in the log, not on the `Exit Code`
  line.

### 4.6 Mandatory log capture / "redirect inside the script" (steering: `tooling.md`)
Every script writes its own output to a log file (internal redirect) which is
then read back with the read tool; never rely on inline stdout capture. Owner
refinement (2026-10-21): do the redirect INSIDE the script, not on the bare
invocation, so the command line stays clean and the capture is owned by the
script. The log write survives even when the shell-capture layer swallows stdout.

### 4.7 Child-process fallback for a wedged session
When output is swallowed or the session wedges, retry the SAME command as a
discrete child process (`& 'C:\tools\powershell7\pwsh.exe' ... -File <script>`),
which has a clean start/end and is observed more reliably than a command tracked
inside the long-lived integrated session. (Does not remove the phantom `-1`.)

### 4.8 Execution split -- agents COMPILE-VERIFY, owner runs the heavy suite
(steering: `testing.md` "Execution split", machine-wide; owner-directed 2026-10-21)
The single biggest mitigation for the hang family: long/quiet commands are the
trigger, so they are removed from the agent's remit entirely.
- An agent (and every workflow step) proves the change COMPILES CLEAN + is
  lint/format clean, SCOPED to touched crates: `cargo check -p <crate>`,
  `cargo clippy -p <crate> -- -D warnings`, `cargo fmt -- --check`, reading logs.
- An agent MUST NOT run `cargo test` / `cargo nextest` / `cargo gate` /
  `cargo full-gate`. The OWNER runs the full gate and the test suite MANUALLY in
  an external terminal and is the authority on a green run.
- RESULT: with this in force, workflow coder steps completed their WORK without
  the mid-work hang -- but still sometimes hung at the final verify/commit/SIGNAL
  boundary (see 4.9).

### 4.9 Per-step commit + manual recovery for the signal-boundary hang
Because steps still hang at their final commit/signal, delegated workflows are
briefed to COMMIT AFTER EACH feature/crate so progress is durable. When a step
hangs at that boundary, the proven recovery (used four times) is:
1. verify the edits compile clean (owner runs `cargo check`/`clippy`/`build`, or
   the authoritative `cargo gate --build`, in their terminal);
2. commit the agent's staged work by hand;
3. abort the stuck workflow (frees the wedged session / build lock);
4. fast-forward-merge the branch; continue the next unit.

### 4.10 Liveness probing by filesystem evidence (steering: `workflow-orchestration.md`)
Because a workflow step shows "running" whether it is healthy or hung, status is
judged by DELIVERABLE EVIDENCE, not the step state: newest source-file mtimes
(build `target/` churn EXCLUDED), new commits, and expected artifacts. Tools:
`tools/powershell/liveness-probe.ps1` and the ad-hoc `last-writes.ps1`
(newest writes, `target/` excluded, verdict ALIVE/SLOWING/STALLED by age).

### 4.11 Owner-runs-in-external-terminal (the ultimate fallback)
When the assistant's channel is degraded (dead episodes, or the Family-2 garble),
the human runs the command in their own terminal -- which has been reliable
throughout -- and pastes the result back. This is how every full `cargo gate`
has been run (Slice 1: 9885/9885; Slice 2 crate1: 9888/9888).

### 4.12 Delegate-all-work split and workflow structure
Heavy work is delegated to workflows in dedicated git worktrees, one
feature/crate per step, with an implement->review loop. This isolates the
blast radius so a hang loses at most one unit of work, all preserved on disk.

---

## 5. Net effect / residual risk

- The wrapper + guard + buffer-clear largely fixed the PSReadLine MANGLING (2.5).
- The execution-split (4.8) largely fixed the MID-WORK hangs on long commands.
- The phantom `-1` (2.1), swallowed output (2.2), and dead episodes (2.3) are
  COPED WITH (verify-by-effect, log-and-read-back) but NOT eliminated.
- The signal-boundary hang (2.4) still recurs; it is survived via per-step
  commits + manual recovery (4.9), not prevented.
- FAMILY 2 (malformed tool-call emission) has NO effective workaround from inside
  the session: it affects the assistant's ability to emit calls at all, survived
  a restart, and currently forces reliance on the owner's external terminal.

---

## 6. Suggested investigation for the defect owner

1. **`~/.kiro/settings/sandbox.json`** -- inspect the sandbox/execution config;
   the execution-layer wrapper that applies the phantom `-1` and loses completion
   likely lives at this layer. (Open in the editor at report time; not yet
   inspected.)
2. **Kiro shell runner / tool-wrapper** -- the phantom `-1` and swallowed output
   are applied OUTSIDE the shell (confirmed: present even on a clean one-line
   command whose log shows success), so the fault is in Kiro's command-execution
   wrapper, not PowerShell.
3. **Tool-call emission path (FAMILY 2)** -- the `court ...` garble is a
   serialization fault in how the assistant's tool calls are emitted/parsed;
   intermittent, tool-agnostic, survived a restart. This is the highest-value
   thing to capture while reproducible, since no in-session workaround exists.
4. **Correlate with load** -- the hangs/wedges correlate with long-running,
   quiet, or parallel compile commands; capture timing around a `cargo gate` run.

---

## 7. Incident 2026-10-22 -- L10N Task 8 workflow code step hung at the signal/commit boundary

Another clean instance of the FAMILY-1 signal-boundary hang (section 2.4 / 4.9),
this time in the delegated `l10n-task8-ffdesktop` workflow (wf_7a5d44144739f873,
convert ff-desktop render literals to the ff_i18n seam).

Timeline (from the liveness probe + git, all 2026-10-22):
- setup-worktree + plan steps: completed cleanly (worktree + 20-area plan.md).
- code step (build-loop iter 0): DID the work for area 1 (command_config):
  edited command_config/edit.rs (13:44:24), render.rs (13:44:02), authored
  crates/ff-i18n/i18n/en/ff-desktop-command-config.ftl (13:43:32), and ran its own
  verification scaffolding -- ftl-dupe-check, ls-en, pyver logs (13:48:03-05).
- Then WENT SILENT. No further filesystem writes, NO commit (HEAD stayed at the
  pre-run main commit 1d18130), step still reported `running` by inspect_workflow.
- liveness-probe at 13:49 said ALIVE (61s), at 14:00 said STALLED (670s, >2x the
  300s threshold) -- i.e. it hung in the ~13:48-13:49 window, right after finishing
  area 1's work and at the point it would verify/commit/signal. ~15 min dead.

Recovery (the proven 4.9 playbook, now used a 5th+ time):
1. Confirmed the area-1 edits were complete and correct by READING them (the
   .ftl keys matched the converted t() sites; dupe-check clean at 9 files/226 keys).
2. Committed the agent's uncommitted area-1 work BY HAND: 75f4c9b
   "feat(l10n): task8 command-config localise render literals".
3. Gave the step a short resume window (committed, waited ~2 min, re-probed) --
   no change, still STALLED.
4. Aborted the workflow (frees the wedged session/build lock).
5. Continued the remaining 19 areas INLINE (file tools are reliable; the owner
   runs the authoritative gate), rather than relaunch a workflow that would very
   likely hang the same way.

Orchestration note that nearly caused a WRONG call: early in diagnosis the
assistant briefly judged the step ALIVE on the strength of fresh tools/logs/
writes -- but several of those were the ASSISTANT'S OWN probe outputs, not the
agent's. Lesson reinforced (workflow-orchestration.md): when judging liveness,
attribute writes to the AGENT, exclude the orchestrator's own probe/log files,
and prefer the liveness-probe's bucketed verdict over an eyeballed mtime.

Also observed this session: the manual-recovery path ITSELF then hit the Family-1
quirks -- a cold `cargo check` in the fresh worktree (build cache gone after the
slice2 worktree delete) ran many minutes/near-stalled on the single shared
channel, and several `git add`/status commands returned no echo (dead episodes)
while that build held the channel. Consistent with "correlates with long/quiet/
parallel compiles" (section 6.4).

### 7.1 Open design question (owner, 2026-10-22): remove the verify/commit/signal
### boundary from workflow steps?

The owner asked whether the verify/commit/signal step at the END of each workflow
coder step -- which is where the hang keeps landing -- should be removed from the
workflows, since it appears to be the recurring trigger.

Assessment (for the defect owner to decide):
- The boundary is WHERE it hangs, but the ROOT CAUSE is the Kiro shell
  runner/tool-wrapper losing completion on a long/quiet command (sections 2.4,
  6.2), not the existence of a commit step. Removing the commit does not fix the
  wrapper; it just moves/relabels the failure.
- A workflow step MUST do SOMETHING terminal to hand control back (emit a success
  signal / mark the step complete). If that terminal action goes through the same
  wedge-prone shell channel, moving it earlier or renaming it will not help.
- BUT there is a real, targeted mitigation in the same spirit as the owner's idea:
  make the step's terminal action NOT depend on a long/quiet shell command.
  Specifically:
  (a) Keep per-area COMMITS (they are short git commands, not the long pole) but
      ensure the step NEVER runs a long/quiet compile/test as its last act --
      compile-verify must be SCOPED + logged (already policy 4.8), and the
      heavy gate stays with the owner (4.8/4.11).
  (b) Prefer the agent completing via a NON-shell signal (the send_message /
      update_workflow status path) rather than a final shell command, so the
      hand-off does not ride the wedge-prone channel.
  (c) Strongest option given the evidence this session: for THIS machine, run the
      mechanical per-area extraction INLINE in the orchestrator (file tools +
      short scoped commands) instead of via a multi-step workflow at all -- the
      workflow's value (isolation, review loop) is outweighed here by its
      reliable hang at the step boundary. The inline path has completed every
      unit this session without a terminal-triggered loss.
- RECOMMENDATION: do NOT simply delete the commit step (loses progress durability
  and still hangs elsewhere). INSTEAD: (1) keep per-area commits, (2) make step
  completion a non-shell signal where the harness allows, (3) for terminal-heavy
  mechanical tasks on this machine, prefer the inline orchestrator path over a
  delegated workflow until the underlying shell-wrapper fault (section 6.1/6.2) is
  fixed. Escalate the wrapper fault as the actual defect to resolve.

### 7.2 Owner clarification (2026-10-22): the inline path is a WORKAROUND, not a
### preference -- the GOAL is to make delegated workflows end cleanly

The owner clarified they PREFER the delegated workflow (isolation + review loop);
the inline path is used ONLY because the workflow keeps failing to END properly
(wedges at the step boundary). So the objective is to make a workflow step's final
commit+signal reliable, not to abandon workflows. The owner proposed putting the
git commits into a script whose result is sent to a log file.

Precise failure-mode analysis (what actually hangs):
- A workflow coder step does two terminal things: (A) the COMMIT (a short, quick
  git command) and (B) the COMPLETION SIGNAL (telling the harness the step is done
  so control returns and the next step starts).
- The repeated evidence (5+ incidents, incl. section 7) is that the step DOES the
  work and even runs its own verification, then goes silent WITHOUT the step ever
  transitioning to completed. That is the SIGNAL (B) not landing -- NOT the commit
  (A). The commit is not the long pole; it is quick and quiet-but-brief.

Does the owner's "commit via a script -> log" idea help? PARTIALLY:
- YES for OBSERVABILITY and for keeping the commit SHORT/BOUNDED: a one-shot
  `tools/` script that does `git add <specific files>` + `git commit -m "..."`
  and redirects to a log, which the step then READS BACK (verify by effect, not
  exit code), is a clean bounded action and a good practice.
- But NOT SUFFICIENT ALONE: the hang is at (B), after the work/commit, so scripting
  (A) would not by itself have saved the Task-8 run.

The combination that SHOULD let a workflow step end cleanly on this machine:
  1. SCRIPTED COMMIT -> LOG (owner's idea): a bounded `tools/` git-commit script,
     result read back from its log. Short, observable, never a long/quiet command.
  2. COMPLETION VIA A NON-SHELL SIGNAL: the step signals success through the
     agent/tool layer (send_message severity=success / update_workflow status),
     NOT a trailing shell command -- so the hand-off does NOT ride the wedge-prone
     pwsh runner that is the actual fault.
  3. NO LONG/QUIET COMMAND AS THE STEP'S LAST ACT: compile-verify stays scoped +
     logged; the heavy gate stays with the owner. Finish on the short scripted
     commit + the tool-level signal.

Caveat: this targets the OBSERVED failure (signal not landing) but does not fix
the underlying Kiro shell-wrapper fault (6.1/6.2). If even a SHORT git command
occasionally wedges (a `git add` echo was swallowed during cold-build channel
contention in the section-7 recovery), item (2) is what carries the hand-off.
Needs a TRIAL delegated run using this pattern to confirm. If it ends cleanly,
switch back to the preferred delegated-workflow path for these tasks.

NEXT ACTION for the defect owner / orchestrator: brief the NEXT delegated workflow
with this 3-part step-completion pattern (scripted-commit-to-log + non-shell
success signal + no long final command) and observe whether the step boundary
stops hanging. Keep escalating the shell-wrapper fault (6.1/6.2) as the root fix.

### 7.3 Owner idea (2026-10-22): run the shell command ASYNC / fire-and-forget so
### the step does not sit blocked waiting for a response

Good instinct, but it targets a DIFFERENT "wait" than the one that hangs, so apply
it precisely. Two separate waits exist:
  (W1) the tool call / step blocked waiting for a shell command to RETURN;
  (W2) the workflow step waiting to be told it is COMPLETE (the signal that does
       not land -- the actual Task-8 hang).
Fire-and-forget addresses W1. The hang was W2. So async helps, but not where the
hang is.

Mechanisms available:
  - `run_in_background=true` on the shell tool, and the dedicated
    control_pwsh_process start/stop tools: launch and return immediately; read
    output later from a log / get_process_output. The SUPPORTED fire-and-forget.
  - OS-level detach inside pwsh: Start-Process / Start-Job / `&` background.

WHERE ASYNC HELPS -- the COMPILE/VERIFY commands. These are the long/quiet commands
that correlate with the wedge (6.4). Background the `cargo check`/`clippy` to a
log and READ the log; the step never sits blocked on a long/quiet command. This
removes the long command from the "last act" position (one of the 3 fix parts).

WHERE ASYNC MUST NOT BE USED -- the COMMIT + COMPLETION SIGNAL. You cannot
fire-and-forget the thing that confirms the commit and tells the harness the step
is done. Backgrounding the commit then immediately signalling success risks
signalling BEFORE the commit lands (a real race on this machine, where a `git add`
echo was swallowed under contention in the section-7 recovery). Keep the commit
SYNCHRONOUS-BUT-BOUNDED (short, scripted, verified by reading its log); keep
completion a NON-SHELL tool-level signal.

Refined per-step pattern (supersedes the 7.2 list):
  1. ASYNC/BACKGROUNDED compile-verify -> log; step reads the log (not blocked on
     the long command). [owner's async idea, correctly placed]
  2. SHORT SCRIPTED COMMIT -> log; verified by reading the log (sync, quick).
  3. NON-SHELL COMPLETION SIGNAL (send_message success / status), never a trailing
     shell command.

Honest caveat: the wedge lives in Kiro's command-execution WRAPPER (it loses
completion even on commands that succeeded -- phantom -1, swallowed output). Async
changes WHEN/WHETHER we block, but the same wrapper still mediates a background
process's start + output capture, so async REDUCES but may not fully eliminate the
exposure. This is why a TRIAL run is required, not an assumed fix.

## Change log
- 2026-10-21 -- Initial defect report compiled from steering rules,
  `PATTERNS-AND-LESSONS.md` Part B, and first-hand session incidents (Family 1
  shell quirks + Family 2 tool-call emission garble). Compiled at owner request
  for logging a defect/issue.
- 2026-10-22 -- Added section 7: L10N Task 8 workflow code step hung at the
  signal/commit boundary (another 2.4/4.9 instance); recorded the full timeline,
  the by-hand recovery (commit 75f4c9b + abort + continue inline), the liveness
  mis-attribution lesson, and the cold-build channel contention during recovery.
  Added 7.1 answering the owner's question about removing the verify/commit/signal
  boundary from workflows (recommend: do not delete the commit; make step
  completion a non-shell signal + prefer the inline path on this machine until the
  shell-wrapper fault is fixed).
  Added 7.2: owner clarified the inline path is a WORKAROUND, not a preference --
  the goal is to make delegated workflows END cleanly. Pinpointed the hang at the
  COMPLETION SIGNAL (not the commit); proposed the 3-part step-completion pattern
  (owner's scripted-commit-to-log + non-shell success signal + no long final
  command) and a trial run to confirm.
  Added 7.3: owner idea to run shell commands ASYNC / fire-and-forget. Clarified
  it addresses the "blocked waiting for a command to return" wait (W1), not the
  "waiting to be told complete" wait (W2) that actually hangs. Correct placement:
  async/background the long COMPILE-VERIFY (via run_in_background / Start-Process),
  but keep the commit synchronous-but-bounded and completion a non-shell signal.
  Refined the per-step pattern accordingly; still needs a trial run.

## 8. Incident 2026-10-22 -- L10N Task 8 TRIAL of the 3-part step-completion pattern: hang RECURRED on step 1

This is the TRIAL run the owner asked for in sections 7.2/7.3 -- a delegated
workflow briefed with the full 3-part step-completion pattern, to test whether a
workflow step can reach its completion signal WITHOUT hanging at the boundary.
RESULT: NEGATIVE. The hang recurred on the VERY FIRST coder step, despite the
pattern. This confirms the 7.3 caveat ("async changes WHEN/WHETHER we block, but
the same wrapper still mediates a background process's start + output capture, so
async REDUCES but may not fully eliminate the exposure").

### Setup
- Orphaned prior trial run wf_df38cebc46b21acb (paused by an agent restart mid
  area-04-macro-library; its area-04 work was finished on disk and committed by
  hand as ff0fcbe). Because the restart severed its parent, this session could
  neither message its steps nor drive its status (both refused: "not in the
  caller's own workflow" / "only the owning parent can call") -- it remains
  paused server-side, harmless. LESSON: an agent restart ORPHANS a running
  workflow -- the new session is not its owning parent, so it cannot resume,
  advance, or abort it. Relaunch a fresh run owned by the live session instead.
- Launched a FRESH trial wf_7fe1c73a504ecbfe (label l10n-task8-areas5-20-trial2),
  owned by the live session, for areas 5-20. Brief carried the full 3-part
  pattern: (1) ASYNC/backgrounded compile-verify -> log, read back; (2) SHORT
  scripted/explicit commit -> log, verify by effect; (3) NON-SHELL completion
  signal. It also carried the explicit fix for the wf-commit.ps1 array-splitting
  gotcha (call git directly through the clean wrapper, staging each file).
- create-workflow completed; plan accepted (16 area steps 5-20 + a final
  semantic review).

### Timeline (all 2026-10-22)
- area-05-plugin-manager (wf-coder) DID the work: authored
  crates/ff-i18n/i18n/en/ff-desktop-plugin-manager.ftl (8 keys, byte-exact,
  state_label exclusion documented) and converted the render literals in
  plugin_manager_panel.rs to ff_i18n::t() (heading / filter / filter-hint /
  clear; state_label() left as &'static str per its asserting unit tests;
  filter_field_id() untouched).
- 15:33 -- clippy.log finished CLEAN ("Finished dev profile ... in 21.18s", whole
  workspace, no warnings). So the code + compile-verify SUCCEEDED.
- 15:37 -- area05-add.log created but ZERO BYTES. The `git add` actually SUCCEEDED
  (git status later showed both files staged, index column "M "/"A ") but its
  output was SWALLOWED (Family-1 symptom 2.2) -- the empty log is NOT evidence the
  add failed.
- Then SILENCE. No further agent writes; later mtimes were target/flycheck
  (rust-analyzer, not the agent) and the orchestrator's OWN probe logs. Step
  stayed `running`; branch tip stayed ff0fcbe (NO area-05 commit landed).
  ~20 min dead at the COMMIT/SIGNAL boundary -- the exact 2.4 hang.

### What this proves about the pattern
- The hang is NOT the commit command being long/quiet (it is a quick git commit).
- The hang is NOT prevented by scripting the commit, by backgrounding the verify,
  or by intending a non-shell signal: the step never GOT to the signal. It went
  silent after a SUCCESSFUL `git add` whose output was swallowed, i.e. the wrapper
  lost completion on a SHORT git command under post-compile channel state.
- So the fault is squarely in Kiro's command-execution WRAPPER losing completion
  (sections 6.1/6.2), BELOW where the 3-part pattern operates. The pattern cannot
  fix it. The owner's preferred "make workflows end cleanly" goal is BLOCKED by
  the wrapper fault, not by step design.

### Recovery (proven 4.9 playbook, ~7th use)
1. Confirmed area-05 complete + correct by READING the .ftl and the converted
   render fn (keys matched; exclusions honoured); clippy.log already clean.
2. git status showed both files already STAGED -> committed by hand:
   51dc89c "feat(l10n): task8 plugin-manager localise render literals" (verified
   by reading the commit log: [feature/l10n-phase2-task8 51dc89c], 2 files).
3. The stuck step was `running` (not `paused`), so update_workflow could not end
   it by nodeId directly. Sequence that worked: update_workflow status=paused
   (no node) -> wait for it to SETTLE paused -> inspect confirmed area-05 now
   `paused` -> update_workflow nodeId=area-05 status=completed -> step ended.
   LESSON: to free a HUNG (running) step you must pause the RUN first (requests
   cancellation of the in-flight turn), let it settle, THEN end the now-paused
   step by nodeId; you cannot end a `running` node directly.
4. ABORTED the workflow (status=aborted with a statusReason recording the trial
   result). Continuing areas 6-20 INLINE per 7.3(c).

### Decision / recommendation (reinforced)
- The trial answers the 7.2/7.3 question: on THIS machine the 3-part pattern does
  NOT make a delegated step end cleanly -- the hang recurred on step 1. Until the
  shell-wrapper completion fault (6.1/6.2) is fixed at the Kiro level, the INLINE
  orchestrator path (7.3(c)) remains the only reliable route for terminal-heavy
  mechanical tasks on this machine, and is now the chosen path for L10N Task 8
  areas 6-20.
- ESCALATE the wrapper completion fault as the actual defect to resolve
  (candidate area: ~/.kiro/settings/sandbox.json / the Kiro shell runner, per
  section 6). The step-completion pattern is a sound practice but is not a fix.

## Change log (appended)
- 2026-10-22 -- Added section 8: the delegated TRIAL of the 3-part step-completion
  pattern (wf_7fe1c73a504ecbfe) hung at the commit/signal boundary on its FIRST
  coder step (area-05-plugin-manager) despite the pattern. Recorded the timeline
  (clippy clean 15:33, empty-but-successful git add 15:37, ~20 min silent at the
  boundary, no commit landed), the proof that the fault is the wrapper losing
  completion on a short git command (below where the pattern operates), the
  recovery (hand-commit 51dc89c + pause-run/settle/end-node/abort), the orphaned
  prior run wf_df38cebc46b21acb (agent restart orphans a running workflow from its
  new session), and the decision to finish areas 6-20 inline. Pattern tried,
  result negative; escalate the wrapper fault as the root defect.
