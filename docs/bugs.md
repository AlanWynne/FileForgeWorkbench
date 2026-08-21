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
| B007 | VERIFIED | Medium | `ff-desktop` | Typing `2` in the command field shows "Error in 2: command not yet implemented" — Phase AS shell routing for option 2 was compiled into the binary but the running executable was stale (not rebuilt after Phase AS). | Req 19.1, 19.4 | — | Root cause: stale debug binary. Fixed by `cargo build`. No code change required. |
| B008 | FIXED | High | `ff-desktop` | New Catalog dialog text fields cannot be typed into — clicking a field does not focus it, Tab does nothing. Root cause: `modal_open` flag did not include catalog/dataset dialogs, so the shell Tab-cycle and `command_field_focus_requested` stole focus every frame. | Req 3.1, 3.3 | — | Fixed: `modal_open` extended to include `FilesDialogState::None` check. |
| B009 | OPEN | High | `ff-desktop` | Allocated datasets are never displayed in the Files Panel content area. `AllocOutcome::Confirmed` discards the validated `AllocParams`; `ContentAreaState::entries` is never populated from any data source; clicking a catalog in the tree shows "No entries to display." | Req 5.1, 10.1 | — | Design gap: dataset store and content-area population not yet implemented. Requires new requirement gate before fix. |
| B010 | FIXED | High | `ff-desktop` | Catalogs created via the New Catalog dialog are lost on restart. The `CatalogRegistry` is never saved to disk on exit and never loaded from disk on startup. Task 10.2 and 10.3 in virtual-catalog-manager/tasks.md were marked `[x]` but the wiring code was never written. | Req 2.1, 2.2 | — | Fixed Phase AU: `SessionManager::save_catalog_registry()` and `load_catalog_registry()` added; wired into `on_exit()` and startup block in `shell.rs`. Catalogs persisted to `catalogs.toml` alongside `session.toml`. 382 tests pass. |
| B011 | FIXED | Medium | `ff-desktop` | Mainframe dataset names entered in the Allocate Dataset dialog are stored as-is (mixed case). Mainframe DSNs must be uppercase. | Req 5.8 | — | Fixed Phase AW: `validate()` now calls `.to_uppercase()` on the trimmed dataset name before storing in `AllocParams`. 399 tests pass. |
| B012 | FIXED | High | `ff-desktop` | Two datasets with the same name can be allocated within the same catalog. The `add_dataset()` function pushes without checking for duplicates. | Req 5.9 | — | Fixed Phase AW: `validate_for_catalog()` added; shell confirm handler uses it with existing DSN list; duplicate rejected with inline error. 399 tests pass. |

| B013 | FIXED | High | `ff-desktop` | Application opens without the Home catalog — File Explorer shows "No catalogs open" message. Root cause: `ffwb.exe` binary (built 2026/08/20 22:14) predates `update.rs` (modified 2026/08/20 22:46) which contains the Phase AX `ensure_default_home_catalog` wiring. Stale binary never wrote `catalogs.toml`. | Req 14.1, 14.2, 14.3 | — | Fixed by `cargo build`. No code change required. |
| B014 | FIXED | High | `ff-desktop` | Expanding a Native catalog node in the File Explorer shows "(no files)" — Native catalogs should list actual files from disk but the tree only reads from `files_panel.datasets` (allocated dataset store), never from the filesystem. | Req 19.6 | — | Fixed: `render_native_children()` added — reads `std::fs::read_dir` for Native catalogs; directories listed first then files alphabetically; double-click opens files. Mainframe/POSIX catalogs continue to use dataset store via `render_dataset_children()`. 404 tests pass. |

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
| Phase AS | B007 added — option 2 "command not yet implemented" error — stale binary, VERIFIED fixed by rebuild |
| Phase AS | B008 added — New Catalog dialog fields not focusable — modal_open missing catalog dialogs (FIXED) |
| Phase AS | B009 added — allocated datasets never appear in Files Panel — design gap, dataset store not implemented |
| Phase AU | B010 added — catalog registry never persisted; catalogs lost on restart — task 10.2/10.3 not implemented |
| Phase AU | B010 FIXED — `save_catalog_registry()` / `load_catalog_registry()` wired into `shell.rs` `on_exit()` and startup; 382 tests pass |
| Phase AW | B011 added — Mainframe dataset names not uppercased on allocation (FIXED) |
| Phase AW | B012 added — duplicate dataset names allowed within same catalog (FIXED) |
| Phase AX | B013 added — Home catalog missing on launch; stale binary predated Phase AX wiring; FIXED by `cargo build` |
| Phase AX | B014 added — Native catalog expansion showed "(no files)"; fixed by adding `render_native_children()` using `std::fs::read_dir` |
| Phase BB | B019 added — File Explorer Panel not resizable; fills entire central panel with no splitter |
| Phase BB | B019 FIXED — `SidePanel::left` with `resizable(true)`, min 120px, max 600px, default 260px |

| B015 | OPEN | High | `ff-desktop` | Tab close button ("×") is missing — tabs have no visible close button on their headers. | Req 3.8 multi-tab-editor | — | Reported via prompt: "Tabs don't have a [x] button." |
| B016 | OPEN | Low | `ff-desktop` | Tab header bracket style is inconsistent — some tabs render with `[]` delimiters, others have none. No documented rule governs which tabs get brackets. | Req 3.2, 3.3 multi-tab-editor | — | Reported via prompt: "Some Tabs have [] around them, some have nothing. What is the general rule?" |
| B017 | FIXED | Medium | `ff-desktop` | Windows junction points (e.g. `C:\Users\<user>\My Documents`) appear as directories in the File Explorer and produce "permission denied" errors when expanded. These are OS-level compatibility junctions that Windows intentionally restricts. | Req 15.1 file-tree-panel | — | Fixed Phase BB: `collect_native_entries()` calls `metadata().ok()?` — entries where metadata fails are silently skipped; junction points never appear in the listing. |
| B018 | FIXED | Medium | `ff-desktop` | Locked system files (e.g. `NTUSER.DAT`) produce an I/O error ("process cannot access the file because it is being used by another process") when the user attempts to open them in the editor. | Req 1.8 multi-tab-editor | — | Fixed Phase BB: `open_file_node()` attempts `File::open()` before handing to editor; OS error 32 stored in `last_error` and shown in status bar; no editor tab opened. |
| B019 | FIXED | Medium | `ff-desktop` | The File Explorer Panel (`=2` / `=FILES`) is not resizable — it fills the entire central panel as a tab with no splitter or drag handle. The user cannot adjust the width of the file listing area. | Req 1.3 file-tree-panel | — | Fixed: `render_central_panel` now uses `egui::SidePanel::left("file_explorer_side").resizable(true).min_width(120.0).max_width(600.0)` with a `CentralPanel` placeholder alongside it. Width persisted in `file_explorer_panel_width` field. |
