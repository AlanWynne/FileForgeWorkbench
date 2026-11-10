# Phase 0 Requirements-Gate Design Review -- scrm-saveconv-extensions investigation

Reviewer: design-review subagent (fresh, no authoring context).
Scope: DOCS-ONLY requirements gate (CR-NR-114). No source change expected; none found.
Verdict: APPROVED (0 HIGH, 0 MEDIUM, 1 NIT).

## What was reviewed

- `docs/specs/master-screen-directive/requirements.md` (new)
- `docs/specs/master-screen-directive/design.md` (new)
- `docs/specs/master-screen-directive/tasks.md` (new)
- `docs/specs/extension-integration/requirements.md` (Req 5 + new AC 6)
- `docs/specs/screen-snapshot-scrm/requirements.md` (Req 3.9-3.13, 4.7-4.8, 9.13-9.14, 10.11-10.14, 22 re-phrased, 23, 24, 25, 26)
- `docs/specs/screen-snapshot-scrm/tasks.md` (Wave F, tasks 15-21; Wave S task 14 marked superseded)
- `docs/project-management/project-master/requirements.md` (Req 11 conformance table + forward directive)
- `docs/status/change-log.md` (CR-NR-114 entry)
- `.kiro/steering/specs.md` (spec index registration -- the actual spec index; there is no `docs/specs/specs.md`)

## Acceptance checks (all PASS)

1. **EARS phrasing** -- PASS. New/edited criteria use WHEN/WHERE ... THE ... SHALL form consistently (e.g. master-directive 1.1-6.2; SCRM 3.9, 4.7, 9.13, 10.11, 22.1-22.9, 23.1-23.7, 24.1-24.6, 25.1-25.5, 26.1-26.5; extension-integration Req 5 AC 6; project-master Req 11.1-11.10). A small number of reference clauses are declarative ("THIS directive does NOT restate ...") but each sits beside a normative SHALL; this is appropriate for a conformance directive and is not a phrasing defect.

2. **ASCII-only** -- PASS. Charset scan (`[^\x00-\x7F\u2500-\u257F]`) on the three master-directive files returned no matches; same for `screen-snapshot-scrm/requirements.md`, `screen-snapshot-scrm/tasks.md`, and `extension-integration/requirements.md`. `project-master/requirements.md` has pre-existing non-ASCII (lines 184-187 connector table, 365 BOM artefact, 423-503, 580) but ALL fall OUTSIDE the edited Requirement 11 region (587+); the author's edited rows and forward directive are clean. `.kiro/steering/specs.md` new row is clean.

3. **Framework-conformance** -- PASS. Every capability is expressed on an existing seam: SNAPSHOT/CAPTURE verbs resolve through `resolve_target` -> `Function` target (SCRM Req 11.3, 22.6, 23.6); menu opens via `open_menu_by_name` (22.7); the SCRM viewer and any Save-As Context are `WorkspaceContext`/`InteriorFocus` (16.1-16.3, 24.6); session restore is `WorkspaceDescriptor` (16.3); the per-Context `ScreenProvider` reuses the existing `ff-screen-model` trait (master-directive Req 4, design "No framework change"). No second dispatcher and no hardwired bar are introduced -- the bar becomes a runtime POM-TOML echo.

4. **Two owner-confirmed framework changes explicitly recorded as framework changes (not workarounds)** -- PASS. (a) Menu-bar-from-POM replacing hardwired `DEFAULT_MENU_BAR_TOML` + dynamic-SNAPSHOT, and (b) the CR-CH-073 bare-POM invariant change (bare POM now CONTAINS SCRM) are both stated as "OWNER-CONFIRMED FRAMEWORK CHANGE" in SCRM Req 22 and as a dedicated "FRAMEWORK CHANGES (owner-confirmed this gate ...)" bullet in the CR-NR-114 change-log entry, each explicitly labelled "NOT workarounds".

5. **No contradiction with existing confirmed decisions** -- PASS. CR-CH-021 (built-in defaults code-only, never written to disk) is reaffirmed in Req 22.3/22.8 and Req 25.5. Save-convention central Req 11 is referenced, not duplicated (master-directive Req 3.1 explicitly declines to restate the ten criteria; extension-integration AC 6 cross-references). Existing specs are extended/cross-referenced, not rewritten (extension-integration D1-D4 left untouched; SCRM existing requirements extended with new sub-criteria and a re-phrased Req 22 that preserves the numbering note).

6. **Deliverable coverage** -- PASS.
   - (1) New `master-screen-directive/` with requirements.md/design.md/tasks.md mandating the three cross-cutting contracts; Core Context retrofit matrix present (Req 5).
   - (2) `extension-integration` Req 5 AC 6 cross-references the master directive without duplicating criteria; D1-D4 untouched (AC 1-5 preserved, AC 6 added).
   - (3) SCRM requirements + tasks updated for full-screen model (3.9-3.13), no-arg format picker (4.7-4.8), PNG-then-PDF with combined picture + ANSI/TEXT rasterised from the text render (23), capture-START bitmap+ANSI+Markdown+command (9.13-9.14), capture-STOP Save-As (24), replay save-to-file/list/open/export (10.11-10.14), POM-TOML-driven bar + RESET BARE (22, 25), mnemonic-letter accelerators (26).
   - (4) project-master Req 11 conformance table refreshed to current reality + forward directive added, cross-linked to the master directive.
   - (5) `.kiro/steering/specs.md` registers `master-screen-directive` alphabetically after `markdown-rendering` in the established entry format.

7. **task-format.md** -- PASS (with one NIT). All task lines in both tasks.md files use only `[ ]`/`[x]` markers and every line carries a descriptive title; status notes are not placed inside brackets.

## Verified assumptions (checked against live source)

- `active_screen_model()` exists in `crates/ff-desktop/src/shell/commands_scrm.rs` and is Menu_Workspace-only, exactly as the design states is the root cause of "cannot be captured yet". The directive's Req 4 change (dispatch to a per-Context `ScreenProvider`) is grounded in real code.
- The CR-CH-071 editable-Context seam (`editable_context_kind` / `editable_persist` / `editable_discard` / `editable_reload`) exists in `crates/ff-desktop/src/shell/editable_context.rs`, with END/RETURN/REFRESH wired in `commands_ladder_a.rs`. Req 3.2's claim that editable Contexts reach SAVE/END/RETURN through this single seam is accurate.
- The retrofit matrix (master-directive Req 5) and the project-master Req 11 conformance table match live source: `editable_context_kind()` maps AliasEditor/ThemeEditor/MenusEditor/KeysEditor/KindsEditor (all "COMPLIANT"); Config/Settings falls through as non-editable with a TODO (matrix "PENDING branch merge"); CommandConfigurator has no arm (matrix "NON-COMPLIANT -> retrofit"). No matrix row contradicts the code.
- The `ScreenProvider` trait and the `ff-screen-model` / `ff-scrm` crate names are consistent with SCRM Req 2/19 and Q4.

## Unverified / wrong assumptions

- None material. The PNG/PDF encoder choice (SCRM Req 23) and the exact SCRM mnemonic keys (Req 25.3) are owner decisions recorded as "settled-but-tunable" and are not design claims requiring source verification at this gate.

## Findings

1. NIT -- `docs/specs/screen-snapshot-scrm/tasks.md` task numbering is out of sequence across waves: the `## Cross-cutting` block uses `- [ ] 10. TCR + verification` which appears AFTER task 9 (Wave 3) while Wave 2R continues at 11-13 and Wave F adds 15-21 (14 is the superseded Wave S). The numbers are unique and every line is titled, so task-format.md is satisfied and the UI will render them correctly; this is purely a readability nit and the affected ordering is largely pre-existing, not introduced by this gate. Concrete fix (optional, non-blocking): move the `## Cross-cutting` TCR block to the end and renumber it after the highest wave number (e.g. `22. TCR + verification`), or leave as-is since numbers remain unique and titled.

## Verdict

0 HIGH + 0 MEDIUM = APPROVED. The one NIT does not affect the verdict. The requirements gate docs are internally consistent, EARS-phrased, ASCII-clean in all authored/edited regions, build on existing framework seams, explicitly record both owner-confirmed framework changes, and cover all five deliverables. Spot-checked source citations hold.
