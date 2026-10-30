# Tasks -- Master Screen Design Directive

Ordered tasks. `[ ]` = pending, `[x]` = complete. Each task cross-references the
requirement criteria it satisfies. NO task is pre-marked complete. TDD per
`.kiro/steering/testing.md`: write the failing test first.

This directive is a cross-cutting conformance contract; its implementation is
shared by the SAVE and SCRM streams (see the investigation plan). The tasks below
are the directive-owned work: the shared `ScreenProvider` dispatch, the
conformance harness, and the matrix bookkeeping. The per-Context provider impls
and the per-Context save-convention retrofits are owned by their own streams and
cross-referenced here.

## 1. Shared ScreenProvider dispatch (the Requirement 4 seam)

- [ ] 1. Replace the Menu_Workspace-only screen lookup with a per-Context ScreenProvider dispatch
  - [ ] 1.1 Change `active_screen_model()` in `shell/commands_scrm.rs` to ask the
        ACTIVE Context for its `ScreenModel` via the `ff-screen-model`
        `ScreenProvider` seam, defaulting to the existing menu provider for
        Menu_Workspaces. (Req 4.1, 4.2)
  - [ ] 1.2 Define the per-Context opt-out: a Context with no logical screen
        declares it explicitly (analogous to `InteriorFocus::none()`), and only
        such a Context yields the "not capturable" response. (Req 4.3, 4.4)
  - [ ] 1.3 Confirm snapshot/capture reaches a Context ONLY through the
        `ScreenProvider` seam and the single SNAPSHOT/CAPTURE front door; no
        bespoke path. (Req 4.5)

## 2. Conformance harness (the Requirement 5 checklist, automated)

- [ ] 2. Add the reusable full-shell egui_kittest conformance harness
  - [ ] 2.1 For each Context (`KindTag`), assert EITHER SNAPSHOT returns a
        `ScreenModel` OR the Context is on the documented opt-out list. (Req 4.2,
        4.3, 5.3)
  - [ ] 2.2 Fold in the existing per-Context first-Tab focus assertion so the
        harness proves focus + capture together. (Req 2.2)
  - [ ] 2.3 For each editable Context, assert SAVE/END/RETURN route through the
        CR-CH-071 seam (reuse the project-master Req 11 test pattern). (Req 3.2)

## 3. Matrix bookkeeping

- [ ] 3. Keep the Core Context retrofit matrix authoritative
  - [ ] 3.1 As each Context is retrofitted by its owning stream (SAVE / SCRM),
        flip its matrix row to COMPLIANT in `requirements.md` Requirement 5. (Req 5.1, 5.3)
  - [ ] 3.2 When a NEW Context is added (core or extension), add its matrix row
        and its harness assertions in the same change. (Req 1.4, 5.2)

## 4. Extension cross-reference confirmation

- [ ] 4. Confirm the extension-integration cross-reference
  - [ ] 4.1 Verify extension-integration Req 5 cross-references this directive so
        an extension Context inherits all three contracts. (Req 6.1, 6.2)

## Cross-references (owned elsewhere, tracked here)

- Per-Context `ScreenProvider` impls: owned by the SCRM stream
  (`docs/specs/screen-snapshot-scrm/tasks.md`).
- Per-Context save-convention retrofits (Command Configurator; Config/Settings on
  merge): owned by the SAVE stream and project-master Requirement 11.
- The focus-latch contract and its first-Tab tests: owned by
  `.kiro/steering/workspace-conformance.md` and `docs/specs/workspace-framework/`.
