# Session State -- 2026-09-22 (end of day)

Durable handoff note. Everything below is committed and pushed to `origin/main`.

## Repo state
- Branch: `main`, in sync with `origin/main` (nothing to push).
- HEAD: `216718d`.
- Working tree clean except 5 KNOWN pre-existing untracked files that are
  intentionally left alone (do NOT commit them):
  - `TestData/ispf_editor_circular_change_tests.txt`
  - `tests/ispf_editor_circular_change_tests.txt`
  - `docs/working-notes/The-three-level-model.md`
  - `docs/source-documents/Integrating-ADVENT-as-a-Hidden-Easter-Egg-in-FileForgeWorkbench.md`
  - `"tools/powershell/Add-EguiKittest - Copy.ps1"`
- No open or reopened bugs (`docs/status/bugs.md` clean; last logged + fixed: B076).

## What shipped this session (in commit order)
- `6656284` CR-NR-087 -- no-arg UP/DOWN honour the active SCROLL amount.
- `5b5db01` CR-NR-080 Slice C -- per-Kind menu-bar assignment (traceability closure).
- `37b2f72` CR-NR-078 WF.6 -- migrated the remaining Contexts (Plugin Manager,
  Event Log, Macro Library, Command Configurator, Search Results) to the
  `WorkspaceContext` trait; FileEditor + FilesPanel documented no-interior cases.
  CR-NR-078 now fully complete.
- `7b4c40f` docs mojibake cleanup + new reusable tool `tools/python/fix_mojibake.py`.
- `a6067a3` B076 -- Linux/Unix build fix (`ff-rust-toolchain` used undeclared
  `dirs_next`; switched to the workspace `dirs` crate). Windows unaffected.
- `cd1a2d7` + `f24116d` CR-CH-045 -- custom-workspace Title_Line now conforms to
  the Menu Workspace title (centered themed heading, descriptive Title Case
  display name distinct from the `[XXX]` tab tag), redundant in-body panel
  titles removed. menu-and-statusbar Req 17.12-17.14.
- `f0d8c44` / `c1012e4` / `cfc664a` -- stale-status cleanup: CR-CH-044/045
  headers fixed; CR-NR-047 marked SUPERSEDED (Settings-as-Menu delivered by
  CR-CH-025/021; namespace-selector design intentionally replaced by the CONFIG
  command); CR-NR-048 marked DISCARDED (POM deliberately minimalist).
- `146c4b5` -- outstanding-CR re-assessment: flipped CR-NR-051/052/055 to DONE
  (built), CR-CH-014 to SUPERSEDED (modal dialog retired by CR-CH-029).
- `216718d` -- NEW steering rule `.kiro/steering/framework-conformance.md`
  (new requirements must build ON the framework; no framework change without
  express owner confirmation) + re-phrased the 6 remaining CRs to use the
  framework, not replace it.

## Verified build/test status
- Last full `verify.ps1` (FULL nextest) was CLEAN at the CR-CH-045 commit:
  9328 passed, 0 failed, `tools/logs/ai-review.log` empty. ffwb.exe rebuilt.
- Doc-only commits since then (no code change), so the build remains green.
- NOTE for tomorrow: B076 (dirs_next) was verified on Windows only (cargo check +
  clippy + verify all clean); the Unix cfg arm could not be cross-compiled here.
  Confirm the Linux laptop now builds clean to fully close the loop.

## Genuinely-open backlog (all re-phased to be framework-safe)
Ordered by suggested priority / smallest-first:

1. CR-CH-012 -- Menu-workspace RESTORE-ON-LAUNCH (startup-and-session Req 21.4).
   SMALL, unblocked. `restore_workspace_descriptors` (shell/commands.rs) still
   has a no-op `WorkspaceDescriptor::Menu` arm; the MENU command it waited on
   exists now (`open_menu_by_name`). Fix = populate that arm to open each
   restored menu by name (honour code-only files / load-error, never write a
   default). Invert the existing "must be skipped" test in shell/tests.rs
   (~line 4645) + add a round-trip test. Light gate (Req 21.4 already exists).
2. CR-NR-058 -- per-command instrumentation (command-framework Req 11): add
   start/params/completion+duration logging at the EXISTING
   `ff_command::Dispatch::execute_command` seam (already logs errors), scoped to
   `Function` targets. The compile-time dev/release gate half already shipped.
3. CR-NR-057 (+ merge CR-NR-054) -- `.` STOP vs `;` PUSH chaining: wire the
   separator to the EXISTING `navigate_to(desc, push)` flag; no second stack.
   Fold in CR-NR-054's residual arg-forwarding (existing `CommandParams`) +
   prefix-argument buffer. Recommend one combined gate.
4. CR-NR-049 -- FFTest context-inspection assertions + auto bug logging (query
   the existing `ShellAutomationRegistry`; bug reports to a SEPARATE artefact,
   never auto-writing `docs/status/bugs.md`).
5. CR-NR-056 -- systematic whole-project analysis (LARGE, proposals-only; the
   `docs/specs/project-analysis/` dir does not exist yet).

Correctly DEFERRED (leave as-is): CR-NR-053 (Regina REXX planning),
CR-NR-059 (responsive-calendar hide), CR-CH-013 (JES MENU rebinding -- waits
until `ff-jes` exists).

## Recommended next action (tomorrow)
Take CR-CH-012's menu-restore slice: run the light gate for Req 21.4 (confirm the
criterion, add the TCR row), then TDD -- write the failing round-trip test +
invert the skip test, implement the single restore arm via `open_menu_by_name`,
run `verify.ps1` FULL nextest, rebuild ffwb.exe, flip CR-CH-012 -> DONE, commit+push.

## Operating reminders (from this session)
- verify gate: `C:\tools\powershell7\pwsh.exe -NoProfile -ExecutionPolicy Bypass
  -File tools\powershell\verify.ps1`; CLEAN = `tools/logs/ai-review.log` empty +
  done-marker exit 0. Full run ~4.5 min; monitor `tools/logs/verify.progress.txt`.
  Do NOT poll with many background sleeps while it runs -- they starve the
  nextest CPU and stall it; wait in one longer interval, then read the marker.
- Terminal quirk: `execute_pwsh` echo is garbled and returns exit -1 spuriously;
  ALWAYS redirect to a file and read it back. Prefer background processes for
  long commands.
- Git commit messages: use SINGLE-quoted `-m` in pwsh (escaped double-quotes
  broke a commit this session).
- Docs must be ASCII (box-drawing + TCR emoji allowed); run
  `tools/python/fix_mojibake.py <file>` (dry-run) to check before committing.
- Steering now includes `framework-conformance.md`: new requirements build ON
  the framework; a framework change needs express owner confirmation.
