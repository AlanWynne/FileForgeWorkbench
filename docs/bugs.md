# FileForge Workbench — Bug Register

**Format:** One row per bug. Update `Status` in-place — never delete a row.

## Status Values

| Status | Meaning |
|--------|---------|
| `OPEN` | Confirmed, not yet fixed |
| `IN PROGRESS` | Actively being worked |
| `FIXED` | Code change made — awaiting verification |
| `VERIFIED` | Fix confirmed by tester |
| `WONT FIX` | Accepted limitation or out of scope |

## Severity Values

| Severity | Meaning |
|----------|---------|
| `Critical` | Data loss or crash |
| `High` | Feature completely broken |
| `Medium` | Feature partially broken or awkward workaround exists |
| `Low` | Cosmetic or minor inconvenience |

---

## Bug Table

| ID   | Status | Severity | Component | Description | Linked Req | Manual Test | Notes |
|------|--------|----------|-----------|-------------|------------|-------------|-------|
| B001 | IN PROGRESS | High     | `ff-desktop` | On Opening the Appliction the ffwb.exe "ISPF" Like Disalog is not visible, it opens into a editor. | Req 14.1, 14.8–14.12 | 🔲 Manual: verify POM tab appears as first tab in tab bar on launch | Phase X (floating window) superseded. Phase Y: POM must be attached tab at index 0 in main tab bar. Tasks 19.1–19.11 in startup-and-session/tasks.md. |
| B002 | OPEN | High     | `ff-desktop` | The "X" on The Editor Window tabs does nto close the file.
| B003 | OPEN | High     | `ff-desktop` | There is no "Close" option on file options menu, There is no way to close files.
| B004 | OPEN | High     | `ff-desktop` | The Editor Window does not have a primary command and line command areas.
| B005 | FIXED | Medium | `ff-desktop` | Key Assignments dialog stops accepting keystrokes after Tab is pressed a few times — shell Tab-cycle consumed all Tab events globally, stealing focus from dialog TextEdit widgets. | Req 20 | — | Fixed Phase AS: added `modal_open` field; Tab-cycle and `command_field_focus_requested` suppressed when any modal is open. |
| B006 | FIXED | Medium | `ff-desktop` | Key Assignments dialog stops accepting keystrokes after Ctrl+C is pressed inside a dialog text field — shell function key dispatch and Ctrl+S handler ran unconditionally every frame, competing with dialog input on Windows clipboard stall. | Req 20 | — | Fixed Phase AS: fkey dispatch and Ctrl+S guard wrapped in `if self.modal_open` check. |

---

## How to file a new bug

1. Assign the next `B###` ID.
2. Set `Status` to `OPEN`.
3. Fill in `Severity`, `Component` (crate name or `ff-desktop`), and a clear one-line `Description`.
4. Link to the relevant `requirements.md` criterion in `Linked Req`.
5. Reference the manual test plan row in `Manual Test` if applicable.
6. If the bug reveals a **missing or incorrect acceptance criterion**, update `docs/specs/<sub-project>/requirements.md` and add a 🔴 row to `docs/TCR.md` before fixing the code (per the new-requirements gate).

---

## Changelog

| Date | Change |
|------|--------|
| Phase W | Initial register created — B001–B010 from known gaps documented in `docs/manual-test-plan.md` and `docs/TCR.md` |
| Phase AS | B005 added — Key Assignments dialog Tab-steal focus bug (FIXED) |
| Phase AS | B006 added — Key Assignments dialog Ctrl+C keystroke freeze bug (FIXED) |
