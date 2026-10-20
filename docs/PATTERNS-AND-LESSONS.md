# FileForgeWorkbench Patterns and Lessons

Durable, curated knowledge captured while building FFWB (Rust / egui workspace).
Read this when you hit a recurring class of problem; it records the root causes
and the proven fix patterns so a lesson is learned ONCE.

## How to use this file (and what does NOT go here)

This file is the layer BELOW steering. The division:

- STEERING (`.kiro/steering/`, `~/.kiro/steering/`) states the RULE -- the
  imperative "always do X", loaded into every session. Terse by design.
- THIS FILE records the WAR STORY -- the dated incident, the root cause, the
  exact fix, and a pointer up to the governing rule. Read on demand, not loaded
  every turn.

When a lesson hardens into a must-always rule, GRADUATE it up into the relevant
steering file (edited in `c:\workspace\_steering`, then deployed down) and leave
only a one-line pointer here. Do not duplicate a steering rule verbatim here --
that causes the drift the tiered-steering model exists to prevent.

Plain ASCII only (see `documentation-charset.md`). Add a dated line to the change
log at the bottom when you add a lesson.

---

## PART A -- Recurring bug classes and the pattern that closes each

### A1. Test-isolation races on a process-global (B048 class)
- SYMPTOM: a test passes under `cargo nextest` or `--test-threads=1` but FAILS
  intermittently under plain multithreaded `cargo test`. Typical panic shape: a
  value set by THIS test reads back as a value set by ANOTHER test (e.g. set
  `ispf`, read `rust`).
- ROOT CAUSE: two or more tests mutate the SAME process-global and run on
  separate threads with no serialization. Comments claiming "serialized" are not
  a mechanism.
- FIX PATTERN (matches Walrus `wal-cli-common/src/input_glob.rs` `CWD_LOCK`): a
  test-only unit mutex plus an RAII guard that both serialises the tests and
  restores the global on drop (so restoration survives a panic), with poison
  recovery:
  ```rust
  static PROFILE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

  struct ProfileGuard { _lock: std::sync::MutexGuard<'static, ()> }
  impl ProfileGuard {
      fn enter() -> ProfileGuard {
          let lock = PROFILE_TEST_LOCK.lock()
              .unwrap_or_else(|poisoned| poisoned.into_inner());
          set_active_profile(None);
          ProfileGuard { _lock: lock }
      }
  }
  impl Drop for ProfileGuard {
      fn drop(&mut self) { set_active_profile(None); }
  }
  // every test that mutates the global: `let _guard = ProfileGuard::enter();`
  ```
- WHERE HIT: `ff-session/src/user_data_dir.rs` (ACTIVE_PROFILE, 2026-10-20);
  earlier B048 in three `ff-desktop` shell tests (there solved by removing the
  global dependency via constructor-seam injection -- the OTHER valid fix when
  the test does not need to exercise the global itself).
- VERIFY: run the failing crate under plain `cargo test` SEVERAL times (a race
  needs repetition, not one pass) -- see `.agents/tasks/b048-test-isolation` for
  the injection variant.
- GOVERNING RULE: `testing.md` (deterministic tests; no reliance on thread order).

### A2. Phantom Tab stop in a Workspace Context (B056-B059 class)
- SYMPTOM: pressing Tab from the command line lands on an "invisible" stop before
  the first real control of a panel.
- ROOT CAUSE: a central-panel render arm drew its panel but did not report its
  interior focus anchors / honour the focus latch, so the focus model had a hole.
- FIX PATTERN: every Workspace Context reports focus through the ONE latch path
  -- implement `WorkspaceContext::render -> InteriorFocus` and dispatch via
  `render_workspace_context` (or `apply_interior_focus` for the documented
  variances). Give the first interactive control a STABLE `egui::Id`. A render
  arm that renders interactive widgets but leaves both ids `None` without the
  documented no-interior exception is this bug.
- VERIFY: a full-shell `egui_kittest` first-Tab test asserting the first Tab from
  the command field lands EXACTLY on the reported first interior control.
- GOVERNING RULE: `workspace-conformance.md` (the tab-order contract).

### A3. Divergent command dispatch (B080 class)
- SYMPTOM: a verb behaves differently depending on whether it was typed, clicked
  in a menu, or triggered by a key.
- ROOT CAUSE: a bespoke `if upper == "..."` intercept or a parallel dispatcher
  outside the single resolve/dispatch path.
- FIX PATTERN: every action resolves through `resolve_target` into a
  `CommandTarget` and dispatches through `shell/target_dispatch.rs`; the typed
  line, menu clicks, and keys all route through that ONE path. Add a new command
  as a registered Command_Id / `Function` target or a `menus/*.toml` entry, not a
  new intercept.
- GOVERNING RULE: `framework-conformance.md` (single command dispatch);
  `wiring-standard.md` (the wiring recipe).

---

## PART B -- Local execution and verification gotchas

The imperative rules are in `terminal-discipline.md` and `tooling.md`. This
section is the FORENSIC record of the terminal quirks on this machine -- the
"why", kept out of always-loaded steering.

### B1. Phantom `Exit Code: -1` is a capture artifact, not a failure
- Kiro's shell runner intermittently appends `Exit Code: -1` even when the
  command SUCCEEDED (output printed, files written). It is Kiro failing to
  observe completion, applied by the tool wrapper OUTSIDE the shell -- so no
  choice of shell removes it. `-1` is never a real process exit code; genuine
  failures surface as the program's real non-zero code (1, 2, 101 for cargo).
- A `-1` on a command whose log shows success needs NO follow-up. Do NOT
  reflexively re-run; that retry loop is what the quirk provokes.
- RULE: verify by effect / read the log (see `terminal-discipline.md` rule 5).

### B2. Two worse shapes: swallowed output and dead episodes
- SWALLOWED OUTPUT: the command runs (files on disk prove it) but its stdout
  never reaches the tool result; the `Output:` block is empty. RECOVER by reading
  the log the command redirected to (`*> tools\logs\<name>.log`); the file write
  is independent of the shell-capture layer, so it survives.
- DEAD EPISODE: a stretch (seconds to ~1 min) where even a trivial command
  returns nothing AND writes no file -- it did not execute at all. These clear on
  their own; wait and retry, do NOT escalate or build a more complex command.

### B3. The sentinel vs the guard -- use the log, not `; Write-Output`
- Older notes said to append `; $rc=$LASTEXITCODE; Write-Output "KIRO_..."`. The
  `pwsh-clean-output-guard` hook HARD-BLOCKS any line containing `;` (a blunt
  substring check, no sentinel exception). So the inline sentinel is NOT usable
  here.
- The guard-safe way to learn a command's real result: run the wrapped command
  with its output redirected to a log, then READ the log; or confirm by effect
  (`Test-Path`, read the produced artifact, `git status`). The program's real
  exit code appears in the log; the `Exit Code` line does not.

### B4. Child-process fallback for a wedged session
- When output is swallowed or the session wedges, retrying the SAME command as a
  discrete child process is observed more reliably than inside the long-lived
  integrated session:
  ```
  & 'C:\tools\powershell7\pwsh.exe' -NoProfile -NonInteractive -File <script.ps1>
  ```
  Put the logic in a `tools/` script and run it via `-File` (no inline `;`), then
  read the script's log. This does NOT remove the phantom `-1` (wrapper-layer),
  so the log/effect check still governs.

### B5. Observed on this machine (2026-10-15)
- Integrated shell is PowerShell 7.6.3, `ConsoleHost`. Enabling Kiro Shell
  Integration removed the hard continuation-prompt lockups but did NOT eliminate
  the phantom `-1` or the intermittent swallow/dead episodes.
- The phantom `-1` appears even on a clean one-line command whose log shows
  success, confirming it is a wrapper-layer capture artifact.

---

## PART C -- Workspace / repository lessons

### C1. Absolute paths rot on relocation; keep tools portable
- The repo was relocated (`c:\workspace\VSC\...` -> `c:\workspace\apps\...`).
  Anything with a hardcoded absolute project path broke: most damagingly the
  `pwsh-clean-output-guard` hook, which pointed its guard script at the dead path
  and blocked EVERY shell command until fixed.
- LESSON: reusable tools derive the repo root from their own location
  (`Split-Path -Parent $PSScriptRoot` up to the root), as `ffwb-gate.ps1` /
  `liveness-probe.ps1` do. Rule examples use RELATIVE paths (`tools/logs/...`),
  never an absolute project path. One-off scripts are deleted after use, not left
  in the repo to rot.

### C2. Steering is tiered and copied DOWN; never edit the deployed copy
- Universal rules live in `c:\workspace\_steering\user-space\steering\` (Tier 1,
  deployed to `~/.kiro/steering/`); project rules stay in `.kiro/steering/`
  (Tier 3). Edit Tier 1 in `_steering` and redeploy (`deploy-steering.ps1` /
  `sync-steering.ps1`); a change made directly in a deployed copy is overwritten
  on the next sync and causes drift.
- WATCH FOR drift: a rule active at `~/.kiro/steering/` that is absent from the
  `_steering` source (seen with `shell-exit-code.md`) means someone edited the
  deployed copy directly -- fold it back into the source.

---

## Change log
- 2026-10-20 -- Initial version. A1 (test-isolation race, ff-session
  ACTIVE_PROFILE + B048), A2 (phantom Tab stop B056-B059), A3 (divergent dispatch
  B080); B1-B5 terminal forensics (moved out of steering during the terminal-rule
  rationalisation); C1 (path portability after the relocation), C2 (tiered
  steering + the shell-exit-code drift finding).
