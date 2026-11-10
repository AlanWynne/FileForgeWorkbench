# Morning Handoff -- SCRM program (updated 2026-10-31 late)

Read this first. The whole SCRM program is nearly done; one branch (scrm-2e) is
in flight with a fix running.

## State of main (pushed to origin)

main tip before scrm-2e merges includes everything through Phase 2D:
- Phase 0 (requirements gate), Phase 1A (ScreenProvider seam), Stream SAVE,
  Phase 2A (full-screen model + per-Context providers), Phase 2B (three-flavor
  clipboard: real window screenshot + Unicode-box Markdown + CF_HTML composite),
  Phase 2C (capture lifecycle: triggering-command, Save-As, replay picker),
  Phase 2D (menu bar = runtime echo of POM TOML, SCRM menu family, Alt+ mnemonics).
All merged, pushed, gated CLEAN, worktrees/branches cleaned up.

## In flight: Phase 2E (dialog/message-box capture) -- branch scrm-2e, NOT merged

2E delivered: dialog/message ScreenModels, frontmost-surface SNAPSHOT, a Copy
affordance on every modal (button + Ctrl+C), and credential-secret redaction.
Gate was CLEAN (4706). BUT owner manual testing found issues, so it is NOT merged.

A FIX WORKFLOW is RUNNING (launched late this session) covering three things the
owner found + decided (spec amended: SCRM Req 27.3 revised, new Req 27.8, master
Req 4.4a):
1. Dialog Copy button was TEXT-ONLY -> changed to the FULL COMPOSITE (picture +
   text + CF_HTML) like SNAPSHOT, so pasting a dialog into Word shows the PICTURE
   too (owner decision B).
2. FAITHFUL FIELD CAPTURE: dialog + Config ScreenModels were thin/generic ->
   must capture the REAL current field values (the full filename in Save-As, the
   folder, dropdown, the Config settings tree + values). This also fixes Config
   Bug 2.
3. Config SNAPSHOT SIDE-EFFECT: typing `snapshot` in the Config workspace
   collapses the focused Accessibility tree item (Enter double-handled) -> fix so
   SNAPSHOT never mutates the screen it captures. This is Config Bug 1.

## What to do in the morning

1. Check the fix workflow result (notifications / inspect_workflow). If it landed
   and the review APPROVED:
2. Run the full `cargo gate` in .worktrees/scrm-2e-dialog-capture -- expect CLEAN.
3. MANUAL re-test (the decisive checks -- the gate cannot prove these):
   - Dialog Copy -> paste into WORD: should now show the window SCREENSHOT PICTURE
     AND the text (not text-only).
   - Open SCRM Save-As (CAPTURE STOP with a non-empty collection), click Copy,
     paste into Notepad: the text must contain the FULL filename / folder / type,
     not a truncated/generic model.
   - Config workspace: type SNAPSHOT -> the Accessibility tree item must NOT
     collapse, and the pasted text must show the REAL settings tree + values (not
     just Filter + Unsaved-edits).
4. If all pass: tell me "merge" and I squash-merge scrm-2e to main, push, clean up
   the worktree/branch. That COMPLETES the entire SCRM program.
   If anything is still off, tell me what and I send a targeted fix.

## Known / deferred (not blockers)

- Per-transition real-window PNG during CAPTURE is deferred (egui 0.33 screenshot
  is async-only); capture records logical model + text + command metadata.
- ODT/ODP snapshot/export deferred (PNG + PDF + HTML/MD/TEXT + .ffscrm supported).
- Save-As dialog heading/labels still hardcoded English (localization follow-up,
  non-blocking).
- A ladder-only verb transition records no triggering command (only resolve_target
  verbs do) -- future enhancement, no second dispatcher.

## Worktree cleanup lesson

When a .worktrees/<name> folder won't delete ("being used by another process"),
the holder is usually KIRO itself indexing the worktree -- a Kiro window RELOAD
releases the lock, then it deletes. (tools/powershell/find-path-holder.ps1 helps
diagnose a path lock without admin.)

## After 2E merges -- the SCRM program is DONE

Remaining program-level follow-ups, all recorded, none urgent:
- The portable-installation reorg (docs/specs/portable-installation/) -- a future
  framework change; the SCRM captures dir (screen-collections) is already
  cross-linked to it (Req 24.7 / portable Req 3.4).
- Extensions spec D1-D4 owner decisions (docs/specs/extension-integration/) --
  a separate strategic conversation, not started.
