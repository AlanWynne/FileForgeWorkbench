# SCRM Recorder -- open discussion (save point)

Status: GATE DOCS DRAFTED, AWAITING OWNER CONFIRMATION. Scope agreed. No code
yet. Spec files under docs/specs/screen-snapshot-scrm/ have been AMENDED (docs
only) with the recorder delta; owner must confirm before any source change.

## What was written (gate step 6 pending -- CONFIRM)

Amendments (documentation only), all marked `[recorder]` / `[recorder-deferred]`:
- requirements.md:
  - Req 9.10-9.12 (NEW): record the triggering command/key on a command-caused
    auto-Capture, via the dispatch seam, absent for interval/rule captures.
  - Req 10.9-10.10 (NEW): show "caused by: <command>" in replay where present.
  - Req 17.2 amended + 17.2a (NEW): optional TriggeringCommand on ScreenCapture,
    round-trips in persistence.
  - Req 21 (NEW, DEFERRED): export a recording as a macro -- v1 only guarantees
    the schema is macro-sufficient (21.2); the exporter (21.1) is deferred.
  - Out-of-scope list: "session recording" annotated as now partially gated.
- tasks.md: new Wave 2R (tasks 11-13): TriggeringCommand model + round-trip
  proptest; populate from dispatch seam; show cause in replay + egui_kittest.
- design.md: S3 data-model note, new S4 "Recording the triggering command"
  subsection, Wave 2R in the plan, requirements-mapping line updated.
Verified ASCII-clean (documentation-charset).

## Resume / next action

AWAIT owner confirmation of the amended docs. On approval: delegate implementation
via a workflow (coder + semantic reviewer). Touches: the dispatch seam read point
(ff-desktop auto-capture hook), ff-scrm ScreenCapture model + zip persistence, and
the ScrmViewerContext replay render + its egui_kittest. TDD per testing.md;
scoped cargo check/clippy/fmt + quick scoped tests (owner runs full gate).

## Agreed understanding (owner-confirmed this session)

The "recorder" is NOT a new parallel subsystem. Most of what was first imagined
is ALREADY specified in the merged SCRM vision:

- Snapshot of current screen as text == Requirement 1 + Requirement 4.
- "Start/stop recording a series of snapshots to a file" == `CAPTURE START` /
  `CAPTURE STOP` (Requirement 7) + Automatic Capture Mode capturing each screen
  transition on ENTER/PF/PA/menu navigation (Requirement 9).
- Step through later == Replay (Requirement 10).

So there is NO new `RECORD` verb and NO new recorder artefact. Reuse
`CAPTURE START` / `CAPTURE STOP` / `CAPTURE REPLAY` and the existing
Collection/Capture format AS-IS.

## THE actual new requirement (the only genuine gap) -- v1

Record, as a FIRST-CLASS part of the Collection data, the COMMAND / KEY that
caused each transition between snapshots. Today Requirement 9 stores the
resulting SCREEN but not the command that moved from screen N to N+1. The delta:

1. Add a "triggering command/key" field to the Capture model (the verb / menu
   option / PF-PA key resolved through the single dispatch front door that caused
   this transition). Populated from the command-dispatch seam
   (resolve_target / target_dispatch), not a bespoke intercept
   (framework-conformance).
2. Persist that field in the Collection format.
3. Surface it in Replay -- show "caused by: <command>" between successive screens.

This turns a screen-only timeline into a command + screen timeline. That is the
whole v1 scope.

## DEFERRED requirement (owner-confirmed, NOT v1)

Save/export the recording AS A MACRO: a series of commands extractable from the
command-annotated Collection into a macro file (FFWB has a Lua macro engine +
Macro Library). v1 only needs the command field captured + persisted in a way
that COULD later feed a macro exporter; the exporter itself is deferred. Record
this as a deferred/NOT-COVERED requirement during the gate so the schema is
designed macro-friendly now without building the exporter.

## Next action on resume

1. Run the requirements gate (workflow.md) as an UPDATE to the existing spec
   `docs/specs/screen-snapshot-scrm/` -- this is an extension of CR-NR-098,
   primarily amending Requirement 9 + the Capture data model + Replay
   (Requirement 10), plus a new DEFERRED macro-export requirement.
2. Keep every criterion in EARS (WHEN ... THE ... SHALL ...); add a
   NOT-COVERED/DEFERRED row for the macro exporter in the coverage record.
3. Owner confirm the amended docs, THEN delegate implementation via a workflow
   (coder + reviewer; touches the dispatch seam, ff-scrm Capture model +
   persistence, and the Replay Context).

## Relevant existing facts (confirmed from repo)

- SCRM spec: `docs/specs/screen-snapshot-scrm/` (requirements/design/tasks +
  decomposition-tasks).
- Crates `ff-screen-model` + `ff-scrm` built and wired; SCRM.0-SCRM.6 done and
  merged; residual SCRM.7 = TCR bookkeeping, plus menu-bar surfacing pending
  (ROADMAP / FORWARD-PLAN).
- Existing SCRM commands: SNAPSHOT [TEXT|ANSI|MARKDOWN|MD|HTML|YAML|AI],
  CAPTURE START|STOP|SCREEN|STATUS|REPLAY. Collections are named, persisted,
  sequentially-numbered Captures. Replay = First/Prev/Next/Last + auto playback +
  timestamps + elapsed-time-between-captures.
- Glossary terms: Screen_Model, ScreenProvider, Snapshot, Capture, Collection,
  Replay_Session, Evidence_Package, Snapshot_Format (PlainText/Ansi/Markdown/
  Html/Yaml).
- Requirement 9 (Automatic/conditional capture) is the primary amend target:
  9.1 captures each transition; 9.2-9.5 are the ENTER/PF/PA/menu triggers whose
  CAUSE must now be recorded on the Capture.
