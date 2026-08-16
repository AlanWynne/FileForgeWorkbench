# FileForge Workbench — Manual Test Plan

**Binary:** `.\target\debug\ffwb.exe`  
**Build command:** `cargo build`  
**Scope:** All features implemented through Phase W (W.4 inclusive)

Mark each test `[P]` Pass · `[F]` Fail · `[S]` Skip (not applicable on this platform)

---

## How to run

```
cargo build
.\target\debug\ffwb.exe                          # empty launch
.\target\debug\ffwb.exe path\to\file.txt         # open one file
.\target\debug\ffwb.exe file1.txt file2.rs       # open multiple files
```

---

## Section 1 — Application Launch and Primary Option Menu

*Validates: startup-and-session Req 1, 7; menu-and-statusbar Req 14*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 1.1 | Launch `ffwb.exe` with no arguments | Window opens. Primary Option Menu (POM) is displayed in the central panel — NOT the editor. | |
| 1.2 | Verify POM title | Title line reads `FileForge Workbench — Primary Option Menu` with a version string below it. | |
| 1.3 | Verify POM option list | Seven numbered options visible: `0 Settings`, `1 Files`, `2 Utilities`, `3 Compilers`, `4 Lua Scripts`, `5 Terminals`, `6 Databases`. | |
| 1.4 | Verify live calendar | Calendar panel shows current month, year, day-of-week header, and today's date highlighted with `*`. | |
| 1.5 | Verify time and day-of-year | Calendar panel shows current `HH:MM` time and `Day of year: NNN`. Time updates each frame (watch for ~1 min). | |
| 1.6 | Verify menu bar | Menu bar contains: `Settings`, `Files`, `Utilities`, `Compilers`, `Lua`, `Terminals`, `Databases`, `Edit`, `Help`. | |
| 1.7 | Verify status bar | Status bar shows `RUNNING`, `Ln 1, Col 1`, encoding, line count, and `FileForge Workbench v0.1.0` on the right. | |
| 1.8 | Verify function key bar | Footer shows `F3 End  F7 Up  F8 Down  F12 Retrieve` labels. | |

---

## Section 2 — CLI File Arguments

*Validates: startup-and-session Req 6*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 2.1 | Launch with one file: `ffwb.exe README.md` | File opens in a tab. POM is NOT shown. Editor displays file content. | |
| 2.2 | Launch with two files: `ffwb.exe file1.txt file2.rs` | Two tabs open. Last file (`file2.rs`) is the active tab. | |
| 2.3 | Launch with a relative path: `ffwb.exe src\main.rs` (run from workspace root) | File resolves correctly and opens. | |
| 2.4 | Launch with a non-existent file: `ffwb.exe no_such_file.txt` | Error shown in status bar. Application still starts. No crash. | |
| 2.5 | Launch with a named flag: `ffwb.exe --no-session-restore README.md` | Flag is ignored (not treated as a file path). `README.md` opens normally. | |

---

## Section 3 — Session Save and Restore

*Validates: startup-and-session Req 4, 5*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 3.1 | Open two files, then close the application normally (File > Exit or `EXIT` command) | Application closes without error. | |
| 3.2 | Relaunch `ffwb.exe` with no arguments | The two previously open files are restored as tabs in the same order. | |
| 3.3 | Relaunch with a CLI file argument after a saved session | Only the CLI file opens. Session tabs are NOT restored (CLI takes precedence). | |
| 3.4 | Delete the session file (`%APPDATA%\ffworkbench\session.toml`) and relaunch | Application starts cleanly with POM. No error or crash. | |

---

## Section 4 — File Open

*Validates: file-operations Req 4; startup-and-session Req 14.6*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 4.1 | File > Open… menu item | Native file-open dialog appears. Selecting a file opens it in a new tab. | |
| 4.2 | Type `EDIT path\to\file.txt` in Command ===> field and press Enter | File opens in a new tab. Command field clears. | |
| 4.3 | Type `1` in Command ===> field and press Enter | Native file-open dialog appears (same as File > Open…). | |
| 4.4 | Type `FILES` in Command ===> field and press Enter | Native file-open dialog appears. | |
| 4.5 | Open a file that is already open | Existing tab is activated. No duplicate tab is created. | |
| 4.6 | Open a non-existent path via `EDIT` command | Error message appears in status bar. No crash. | |

---

## Section 5 — Multi-Tab Editor

*Validates: multi-tab-editor Req 1, 2, 3*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 5.1 | Open three files | Three tab headers appear in the tab bar. | |
| 5.2 | Click a non-active tab header | That tab becomes active. Editor content switches to that file. | |
| 5.3 | Switch between tabs | Each tab restores its own scroll position and cursor position independently. | |
| 5.4 | Verify modified indicator | Edit a file (type a character). Tab header shows `●` prefix. | |
| 5.5 | Verify active tab highlight | Active tab header has a visually distinct background from inactive tabs. | |
| 5.6 | Click the `×` close button on a tab (when multiple tabs open) | Tab closes. Remaining tabs are unaffected. | |

---

## Section 6 — Editor Navigation

*Validates: viewport-and-scrolling Req; navigation-commands Req*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 6.1 | Open a multi-line file. Press Down Arrow | Cursor moves down one line. Status bar `Ln` counter increments. | |
| 6.2 | Press Up Arrow | Cursor moves up one line. | |
| 6.3 | Press Right Arrow | Cursor moves right one column. Status bar `Col` counter increments. | |
| 6.4 | Press Left Arrow | Cursor moves left one column. | |
| 6.5 | Press Page Down | Viewport scrolls down by approximately one screen of lines. | |
| 6.6 | Press Page Up | Viewport scrolls up by approximately one screen of lines. | |
| 6.7 | Scroll to the last line with Down Arrow | Cursor stops at the last line. No crash or wrap-around. | |
| 6.8 | Scroll to line 1 with Up Arrow | Cursor stops at line 1. No crash. | |
| 6.9 | Mouse wheel scroll | Viewport scrolls up/down. | |
| 6.10 | Click the mouse in the editor text area | Cursor moves to the clicked line and column. Status bar updates. | |

---

## Section 7 — ISPF Command Field

*Validates: command-semantics Req; startup-and-session Req 14.6*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 7.1 | Type `EXIT` and press Enter | Application closes. | |
| 7.2 | Type `QUIT` and press Enter | Application closes. | |
| 7.3 | Type `=X` and press Enter | Application closes. | |
| 7.4 | Type `TOP` and press Enter (with a file open) | Viewport scrolls to line 1. | |
| 7.5 | Type `BOTTOM` and press Enter | Viewport scrolls to the last line. | |
| 7.6 | Type `UP 5` and press Enter | Viewport scrolls up 5 lines. | |
| 7.7 | Type `DOWN 5` and press Enter | Viewport scrolls down 5 lines. | |
| 7.8 | Type `LOCATE 10` and press Enter (file has ≥10 lines) | Viewport scrolls so line 10 is visible. | |
| 7.9 | Type `LOCATE 999999` (beyond file end) and press Enter | Error message in status bar. No crash. | |
| 7.10 | Type `FIND hello` and press Enter (file contains "hello") | Viewport scrolls to first occurrence. | |
| 7.11 | Type `FIND ZZZNOMATCH` and press Enter | "NOT FOUND" message in status bar. | |
| 7.12 | Type `RFIND` after a successful FIND | Finds the previous occurrence. | |
| 7.13 | Type an unrecognised command e.g. `BLAH` | Error message in status bar. No crash. | |

---

## Section 8 — Function Keys

*Validates: function-keys-and-history Req 3, 4*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 8.1 | Press F3 | Application exits (END command). | |
| 8.2 | Open a long file. Press F7 | Viewport scrolls up (UP command). | |
| 8.3 | Press F8 | Viewport scrolls down (DOWN command). | |
| 8.4 | Type a command and press Enter. Then press F12 | Command field is populated with the last command (RETRIEVE). | |
| 8.5 | Press F12 again | Cycles to the previous command in history. | |

---

## Section 9 — Text Editing

*Validates: edit-operations Req 1, 2, 4, 12; startup-and-session Req 13*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 9.1 | Open a file. Click in the editor. Type a character | Character is inserted at the cursor position. Document content updates. | |
| 9.2 | Type several characters | All characters appear in sequence. Cursor advances with each character. | |
| 9.3 | Press Backspace | Character immediately before the cursor is deleted. | |
| 9.4 | Press Enter | Line splits at the cursor position. New line is created below. | |
| 9.5 | Press Ctrl+Z | Last edit is undone. Document reverts to previous state. | |
| 9.6 | Press Ctrl+Z multiple times | Each press undoes one more edit. | |
| 9.7 | Verify modified indicator appears | After typing, tab header shows `●`. Status bar shows `●` modified indicator. | |
| 9.8 | Verify current-line highlight | The line containing the cursor has a visible background highlight. | |
| 9.9 | Verify caret rendering | A visible vertical bar (caret) is rendered at the cursor column. | |

---

## Section 10 — File Save

*Validates: file-operations Req 1; edit-operations Req 12*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 10.1 | Edit a file. Press Ctrl+S | File is saved. Modified indicator `●` disappears from tab header and status bar. | |
| 10.2 | Edit a file. Use File > Save menu item | File is saved. Modified indicator clears. | |
| 10.3 | Open the saved file in a text editor (Notepad etc.) | Confirms the edited content was actually written to disk. | |
| 10.4 | Save an unmodified file (Ctrl+S) | No error. No crash. (May be a no-op.) | |

---

## Section 11 — Theme Switching

*Validates: theme-and-appearance Req*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 11.1 | Settings > Dark Theme | Editor and UI switch to dark colour scheme. | |
| 11.2 | Settings > Light Theme | Editor and UI switch to light colour scheme. | |
| 11.3 | Settings > High Contrast | Editor and UI switch to high-contrast colour scheme. | |
| 11.4 | Switch theme and then switch tabs | Theme persists across tab switches. | |

---

## Section 12 — EXCLUDE / SHOW / RESET

*Validates: exclude-show-filter Req*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 12.1 | Open a file with repeated content. Type `EXCLUDE ALL` and press Enter | All lines are hidden. A placeholder row shows the count of excluded lines. | |
| 12.2 | Type `SHOW ALL` and press Enter | All lines are restored. Placeholder disappears. | |
| 12.3 | Type `EXCLUDE 'fn'` (or a word present in the file) and press Enter | Lines containing that word are hidden. | |
| 12.4 | Type `RESET` and press Enter | All exclusions are cleared. All lines visible. | |

---

## Section 13 — Navigation Commands (SORT / LOCATE)

*Validates: navigation-commands Req*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 13.1 | Open a multi-line file. Type `LOCATE 1` | Viewport scrolls to line 1. | |
| 13.2 | Type `LOCATE 50` (file has ≥50 lines) | Viewport scrolls to line 50. | |
| 13.3 | Type `LEFT` and press Enter | Viewport scrolls left (horizontal scroll). | |
| 13.4 | Type `RIGHT` and press Enter | Viewport scrolls right. | |

---

## Section 14 — Line Commands (Prefix Area / Gutter)

*Validates: line-commands Req*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 14.1 | Click in the gutter (prefix area) of a line and type `D` then press Enter | That line is deleted. | |
| 14.2 | Type `I` in the gutter of a line and press Enter | A blank line is inserted below. | |
| 14.3 | Type `R` in the gutter of a line and press Enter | That line is repeated (duplicated) below. | |
| 14.4 | Type an invalid prefix command (e.g. `Z`) and press Enter | Error message shown. Line is unchanged. | |

---

## Section 15 — Compilers Menu and Toolchain Panel

*Validates: compiler-toolchain-integration Req 14.6, 15.1–15.3, 17.1–17.3*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 15.1 | Click Compilers menu > Toolchain Panel | Toolchain Panel appears docked at the bottom of the window. | |
| 15.2 | Verify GCC row | Panel shows a `GCC` status row. If GCC is not installed: shows "Not found" and an `[Install GCC]` button. If installed: shows "Ready — <version>". | |
| 15.3 | Verify Rust row | Panel shows a `Rust` status row. If Rust is not installed: shows "Not found" and an `[Install via rustup]` button. If installed: shows "Ready — <version>". | |
| 15.4 | Type `3` in Command ===> field and press Enter | Toolchain Panel opens (same as menu item). | |
| 15.5 | Click the `×` close button on the Toolchain Panel | Panel closes. Editor area expands to fill the space. | |
| 15.6 | Reopen the panel. Verify Build Output section | "Build Output" heading is visible with a scrollable area below it. | |
| 15.7 | Verify Diagnostics section | "Diagnostics" heading is visible with a scrollable area below it. | |
| 15.8 | If Rust is installed: click `[Build]` button on the Rust row (with a Cargo project open) | Build output lines appear in the Build Output area. | |

---

## Section 16 — Exit Behaviour

*Validates: startup-and-session Req 9*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 16.1 | With no unsaved changes, click the window close button (×) | Application closes immediately. No prompt. | |
| 16.2 | With no unsaved changes, type `EXIT` in command field | Application closes. | |
| 16.3 | Edit a file (make it dirty). Close the window | Application closes. (Note: unsaved-changes prompt is a known gap — verify no crash.) | |
| 16.4 | After exit, relaunch | Session is restored (previously open tabs reappear). | |

---

## Section 17 — Status Bar

*Validates: menu-and-statusbar Req; startup-and-session Req 7*

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 17.1 | Open a file. Move cursor to line 5, column 3 | Status bar shows `Ln 5, Col 3`. | |
| 17.2 | Verify encoding label | Status bar shows `UTF-8` (or detected encoding) for the open file. | |
| 17.3 | Verify line count | Status bar shows the correct number of lines in the file. | |
| 17.4 | Edit the file | Status bar shows `●` modified indicator. | |
| 17.5 | Save the file | `●` indicator disappears from status bar. | |
| 17.6 | With no file open (POM visible) | Status bar shows `RUNNING` and version. No crash. | |

---

## Section 18 — Keyboard Shortcuts Summary

| # | Shortcut | Expected action | Result |
|---|----------|----------------|--------|
| 18.1 | Ctrl+S | Save active file | |
| 18.2 | Ctrl+Z | Undo last edit | |
| 18.3 | F3 | Exit / END | |
| 18.4 | F7 | Scroll up | |
| 18.5 | F8 | Scroll down | |
| 18.6 | F12 | RETRIEVE last command | |
| 18.7 | Arrow keys | Move cursor | |
| 18.8 | Page Up / Page Down | Scroll viewport | |

---

## Section 19 — Robustness / Edge Cases

| # | Test | Expected result | Result |
|---|------|----------------|--------|
| 19.1 | Open a very large file (>10 MB) | File opens without crash. Scrolling is responsive. | |
| 19.2 | Open a binary file | File opens (may show garbled content). No crash. | |
| 19.3 | Open a file with Windows line endings (CRLF) | File displays correctly. Encoding label shows UTF-8 or detected encoding. | |
| 19.4 | Open a read-only file (set file to read-only via OS) | File opens. Typing should either be blocked or show an error on save. | |
| 19.5 | Rapidly switch between tabs 10+ times | No crash. Each tab shows correct content and cursor position. | |
| 19.6 | Type in the command field without a file open (POM visible) | No crash. Command is processed normally. | |
| 19.7 | Press all function keys F1–F12 | No crash. Unbound keys do nothing. Bound keys (F3/F7/F8/F12) work as expected. | |
| 19.8 | Resize the window | UI reflows correctly. No rendering artefacts. | |

---

## Known Gaps (do not fail these)

The following are documented as not yet implemented and should be marked `[S]`:

| Feature | Status |
|---------|--------|
| File > Save As… dialog | Not yet wired to a file-picker |
| Unsaved-changes prompt on close/exit | Not yet implemented |
| Tab right-click context menu | Not yet implemented |
| Ctrl+Tab MRU tab switching | Not yet implemented |
| Drag-and-drop tab reordering | Not yet implemented |
| Install confirmation dialogs (GCC / Rust) | UI trigger present; full dialog not implemented |
| Background install progress (GCC / Rust) | Not yet wired to ff-bgio |

---

## Test Run Summary

| Section | Total | Pass | Fail | Skip |
|---------|-------|------|------|------|
| 1 — Launch & POM | 8 | | | |
| 2 — CLI Arguments | 5 | | | |
| 3 — Session Restore | 4 | | | |
| 4 — File Open | 6 | | | |
| 5 — Multi-Tab | 6 | | | |
| 6 — Navigation | 10 | | | |
| 7 — Command Field | 13 | | | |
| 8 — Function Keys | 5 | | | |
| 9 — Text Editing | 9 | | | |
| 10 — File Save | 4 | | | |
| 11 — Themes | 4 | | | |
| 12 — Exclude/Show | 4 | | | |
| 13 — Nav Commands | 4 | | | |
| 14 — Line Commands | 4 | | | |
| 15 — Toolchain Panel | 8 | | | |
| 16 — Exit | 4 | | | |
| 17 — Status Bar | 6 | | | |
| 18 — Shortcuts | 8 | | | |
| 19 — Robustness | 8 | | | |
| **Total** | **120** | | | |

---

*Generated after Phase W (W.4) completion. All 64 library crates pass `cargo test --workspace`.*
