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

- DIAGNOSTIC TELL (how to KNOW it is this class, not a logic regression): the
  FAILING SET SHIFTS between runs of UNCHANGED code -- different test names, and
  even a different COUNT (e.g. 4 failed, then a different 4, then 6). A real
  regression fails the SAME tests every run. A shifting set == a shared mutable
  global raced under the threaded runner. Also: "green under nextest, red under
  `cargo test`" is the same tell (nextest is process-per-test, so it MASKS the
  race; a test that only passes in its own process is not truly isolated).
  Corollary: do NOT chase the individual named tests -- find the shared global.

- THREE valid fixes -- pick by WHO writes the global, not by which is nearest:
  1. LOCK every writer (the `CWD_LOCK` pattern above). Correct ONLY if you lock
     EVERY writer; a partial lock that guards some tests but not the common
     fixture (`make_shell` / a shared setup) does NOT work -- the unlocked
     fixture still clobbers mid-assertion. Cost: serialises those tests.
  2. INJECT the dependency through a constructor seam so the test does not touch
     the global at all (the B048 `ff-desktop` variant; the same philosophy FFWB
     uses for user-config / history paths). Best when the test does not need to
     exercise the global itself.
  3. THREAD-SCOPE the global with a test-only PER-THREAD OVERRIDE. Use this when
     the global sits on a HOT path written by a large share of the suite (so a
     lock would serialise most of `cargo test` and kill parallelism) AND the
     runner is one-thread-per-test. The reader checks the thread-local first,
     else the process global; test installers set the thread-local; PRODUCTION
     never sets it and is unchanged. No lock, full parallelism.
- DECISION RULE: few writers, rare global -> (1) lock. Test does not need the
  global -> (2) inject. Global on a hot common-fixture path -> (3) thread-scope.
  A THREAD-LOCAL OVERRIDE IS A TEST-ISOLATION TOOL, NOT A PRODUCTION MODEL -- see
  the boundary note below; do not "fix" a production global by making it
  thread-local.
- WHERE HIT (variant 3): the `ff-i18n` active catalogue is a process-global
  `RwLock<Option<Catalogue>>` read by `t()`/`t_args()`; the whole `ff-desktop`
  test fixture (`make_shell`/`harness_shell` -> `install_catalogue_for_tests`)
  installs the English base into it, so hundreds of unlocked writers raced the
  dozen localisation tests. Fixed by a thread-local override in
  `ff-i18n/src/seam.rs` (`set_test_catalogue`/`clear_test_catalogue`); the two
  genuine PRODUCTION-global-path tests kept a small shared lock. Full
  `ff-desktop` suite went from a shifting 4-6 failures to 1162/1162 under
  threaded `cargo test` (CR-CH-071 follow-up, branch crch071-save-cancel-refresh).

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

### C3. Test-isolation fixes vs production shared-state -- know which problem you have
This is the BOUNDARY note A1 (variant 3) points to. The in-memory test-isolation
fix and multi-instance (two app copies at once) are COUSINS, not the same fix.

- THREAD-LOCAL OVERRIDE IS TEST-ONLY, NEVER A PRODUCTION MODEL. The `ff-i18n`
  thread-local catalogue override (A1 variant 3) is correct ONLY because Rust's
  test runner is one-thread-per-test. In PRODUCTION there is exactly ONE active
  locale per process, and the process-global `RwLock<Option<Catalogue>>` is the
  RIGHT model (set once at startup, swapped atomically on a locale change). A
  thread-local in production would be a BUG: the egui render thread, Tokio worker
  threads, and background-I/O threads would each see a different (or empty)
  catalogue, so labels would render inconsistently off the main thread. Do not
  "fix" a production global by making it thread-local.
- WHAT DOES transfer to base code is the PRINCIPLE, not the mechanism: prefer an
  INJECTED per-instance seam over a process-global (the B048 variant-2 fix, and
  what FFWB already does for user-config / history paths via constructor seams --
  which is exactly why those do NOT race). The i18n catalogue is the one
  subsystem that stayed global; that is an architectural observation, not a
  defect to rush-fix.
- MULTI-INSTANCE (running two copies of the app at once) is a DIFFERENT axis:
  separate processes share NO memory, so a thread-local / in-memory lock does
  nothing for it. The real contention is on FILES and OS resources on disk:
  config / history / session files, the user alias store (`user.toml`),
  `menus/*.toml` / themes, and log files -- two instances writing the same path
  is last-writer-wins / lost-update / partial-write corruption. The patterns that
  help THERE are: (a) ATOMIC writes (write-temp-then-rename -- Walrus has
  `atomic.rs`; FFWB `archive.rs` does transactional writes -- extend to the small
  config/alias/menu writers); (b) an advisory SINGLE-INSTANCE lockfile to
  detect/own a second instance; (c) per-instance scratch/config paths where
  instances must be independent (the test fixture already proves the code SUPPORTS
  injected paths -- two normal instances just default to the same shared files).
- STATUS: multi-instance safety is NOT yet audited or implemented; the above is
  the map for a future change request, not a claim that FFWB is multi-instance
  safe today. Treat it as its own gated CR (requirements + a read-only audit of
  every production file writer first).

---

## Change log
- 2026-10-20 -- Initial version. A1 (test-isolation race, ff-session
  ACTIVE_PROFILE + B048), A2 (phantom Tab stop B056-B059), A3 (divergent dispatch
  B080); B1-B5 terminal forensics (moved out of steering during the terminal-rule
  rationalisation); C1 (path portability after the relocation), C2 (tiered
  steering + the shell-exit-code drift finding).
- 2026-10 -- Extended A1 with the diagnostic tell (shifting failing set ==
  shared-global race; green-under-nextest-red-under-cargo-test), the THREE-fix
  taxonomy (lock all writers / inject seam / thread-scope), the decision rule,
  and the ff-i18n catalogue thread-local override incident (CR-CH-071 follow-up,
  1162/1162 green under threaded cargo test). Added C3 (test-isolation vs
  production shared-state boundary: thread-local is test-only; the transferable
  principle is injected per-instance seams; multi-instance is a distinct on-disk
  file-contention axis needing atomic writes / single-instance lock / per-instance
  paths -- not yet audited, flagged as its own future CR).
