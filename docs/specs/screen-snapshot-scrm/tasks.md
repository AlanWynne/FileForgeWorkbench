# Tasks -- Screen Snapshot Service + SCRM (CR-NR-098)

Ordered, independently completable tasks. `[ ]` = pending, `[x]` = complete.
Each task cross-references the requirements criteria it satisfies. NO task is
pre-marked complete. TDD per testing.md: write the failing test first.

Waves follow design.md Section 8. Waves 0-3 are the SCRM build (gated). The
later panel-extraction refactors (design.md Wave 4+) are behaviour-preserving and
tracked as separate CRs when they land; they are NOT in this list.

## Wave 0 -- foundation crates (no ff-desktop churn)

- [ ] 1. Create the `ff-screen-model` crate (no egui, no SCRM logic)
  - [ ] 1.1 Add the crate to the workspace members and define `ScreenModel` and
        its elements (Title, Field{label,value,attrs}, Table, Message, StatusBar,
        command_line, cursor, dimensions). Attrs carry colour/highlight/
        protection/sensitive. (Req 3.1-3.8, 13.5)
  - [ ] 1.2 Define the `ScreenProvider` trait (`screen_model(&self) -> ScreenModel`).
        (Req 2.1)
  - [ ] 1.3 Implement the PlainText renderer with Unicode box-drawing default and
        ASCII fallback. (Req 4.2, 5.1, 5.2, 5.4)
  - [ ] 1.4 Implement the Ansi renderer (SGR colour codes). (Req 4.3, 5.3)
  - [ ] 1.5 Implement the Markdown renderer (fenced code block; table element ->
        Markdown table). (Req 4.4)
  - [ ] 1.6 Implement the Html renderer (colours + layout). (Req 4.5)
  - [ ] 1.7 Implement the Yaml/AI renderer (title/fields/tables/buttons/cursor).
        (Req 4.6)
  - [ ] 1.8 Define `SnapshotFormat` enum (PlainText, Ansi, Markdown, Html, Yaml).
        (Req 5.5)
  - [ ] 1.9 Proptest: every renderer preserves all visible field text as
        selectable characters (the hard constraint). (Req 1.1, 1.2)

- [ ] 2. Create the `ff-scrm` crate (no egui; depends on ff-screen-model)
  - [ ] 2.1 Add the crate; define `ScreenCollection` and `ScreenCapture` data
        model incl. DIDL fields (DialogState id). (Req 17.1, 17.2, 15.1, 15.2)
  - [ ] 2.2 Implement native zip archive persistence (collection.yaml, screens/,
        images/, metadata/), zip-compatible. (Req 12.5, 17.3)
  - [ ] 2.3 Implement crash-recovery journal for an active Collection. (Req 18.4)
  - [ ] 2.4 Implement capture rules + rule evaluation (Screen Name, Program Name,
        Message Class, Transaction ID, Dataset Name, User ID; multiple active).
        (Req 9.7-9.9)
  - [ ] 2.5 Implement masking rules + sensitive-field masking on export. (Req 13)
  - [ ] 2.6 Implement the replay state machine (sequence, first/prev/next/last,
        autoplay, adjustable speed, timestamps, elapsed-between). (Req 10.1-10.7)
  - [ ] 2.7 Implement text/markdown/html exporters over a Collection. (Req 12.1-12.3)
  - [ ] 2.8 Proptest: a Collection round-trips through the zip archive unchanged;
        >=10,000 captures supported. (Req 12.5, 18.1)

## Wave 1 -- thin wiring: SNAPSHOT end to end

- [ ] 3. Register SNAPSHOT commands as Function-target Command_IDs
  - [ ] 3.1 Register SNAPSHOT + TEXT/ANSI/MARKDOWN(MD)/HTML/YAML(AI); resolve via
        resolve_target -> Function; NO new hard-coded intercept. (Req 4.1-4.6,
        11.2, 11.3)
  - [ ] 3.2 First `ScreenProvider` impl: the POM (Home Context). (Req 2.1, 2.2)
  - [ ] 3.3 Deliver the rendered snapshot to the clipboard + status confirmation;
        "not capturable" path for a Context with no provider. (Req 6.1-6.3, 2.3)
  - [ ] 3.4 Full-shell egui_kittest: SNAPSHOT on the POM copies selectable text of
        the expected fields. (Req 1.1, 6.1)

## Wave 2 -- collection lifecycle + auto-capture + viewer

- [ ] 4. CAPTURE collection lifecycle commands
  - [ ] 4.1 CAPTURE START/STOP/STATUS/LIST/OPEN/SAVE/LOAD/PURGE as Function
        commands. (Req 7.1-7.7, 11.1)
  - [ ] 4.2 CAPTURE SCREEN appends to the active Collection with sequential
        numbering; auto-start a timestamp-named default when none active.
        (Req 8.1-8.4)
- [ ] 5. Automatic capture hook
  - [ ] 5.1 Hook `nav_stack::reconstruct_context` to emit a capture on Context
        transition when Auto Mode is on. (Req 9.1, 9.5, design S4)
  - [ ] 5.2 Post-command model-diff guard (content hash) to catch command-driven
        screen changes without double-capturing a transition. (Req 9.2-9.4, design S4)
  - [ ] 5.3 Configurable capture interval. (Req 9.6)
  - [ ] 5.4 Off-frame async capture write so capture never visibly interrupts
        interaction. (Req 18.2, 18.3)
- [ ] 6. SCRM viewer Context
  - [ ] 6.1 Add WorkspaceKind::ScrmViewer + TabKind + open_scrm_viewer_tab +
        reconstruct_context arm + descriptor mapping. (Req 16.1, 16.3)
  - [ ] 6.2 Implement ScrmViewerContext: WorkspaceContext with replay controls,
        selectable-text screen render, stable first-control egui::Id, returns
        InteriorFocus. (Req 16.1, 16.2, 10.8)
  - [ ] 6.3 CAPTURE REPLAY opens/focuses the viewer for a Collection; replay
        filtering by dialog state. (Req 10.1, 15.3)
  - [ ] 6.4 Mandatory full-shell first-Tab egui_kittest focus test for the viewer.
        (Req 16.2)

## Wave 3 -- exporters, evidence, protected PDF

- [ ] 7. Export commands
  - [ ] 7.1 CAPTURE EXPORT TEXT/MD/HTML wired to ff-scrm exporters; async export.
        (Req 12.1-12.3, 18.3)
  - [ ] 7.2 PDF export with title page, TOC, screen index, page numbering; real
        selectable text (printpdf). (Req 12.4, 12.6)
  - [ ] 7.3 State-transition history export. (Req 15.4)
- [ ] 8. Evidence packages
  - [ ] 8.1 Build evidence package (user/date/session/collection/screens). (Req 14.1, 14.2)
  - [ ] 8.2 Test-case id + pass/fail indicators. (Req 14.3, 14.4)
- [ ] 9. Protected / tamper-evident PDF
  - [ ] 9.1 PRE-SLICE CHECKPOINT: confirm the pinned PDF crate versions can apply
        owner-password encryption + permission flags (edit-lock, copy-allow).
        Surface as a blocker if not; do not work around. (design S5, Req 20.2)
  - [ ] 9.2 CAPTURE EXPORT PDF PROTECTED: copy-enabled, edit-locked via owner
        password; selectable text preserved. (Req 20.1, 20.2, 20.3, 20.7)
  - [ ] 9.3 Optional user (open/read) password. (Req 20.2a, Q7)
  - [ ] 9.4 Embed content hash in PDF + evidence metadata (tamper-evidence) +
        generating user/timestamp/collection id. (Req 20.4, 20.5)
  - [ ] 9.5 Evidence package as a protected PDF. (Req 14.5)
  - [ ] 9.6 Digital signature add-on: design-confirm the signer path before build;
        deferred configurable option, surfaced to owner. (Req 20.4a, Q6)

## Cross-cutting

- [ ] 10. TCR + verification
  - [ ] 10.1 Add/flip TCR rows for every Req 1-20 criterion as it is covered.
  - [ ] 10.2 Each wave ends with a clean full verify.ps1 (nextest) run; ai-review.log empty.
  - [ ] 10.3 Confirm ff-desktop line count / rebuild-trigger reduction is
        measured before/after (Req 19.4).
