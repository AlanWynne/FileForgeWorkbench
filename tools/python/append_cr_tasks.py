"""Append Phase CR tasks to spec task files and project-master."""
import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\append_cr_tasks.txt"
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

log("=== append_cr_tasks.py ===")

# --- lua-macro-engine/tasks.md ---
lua_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\lua-macro-engine\tasks.md"
data = open(lua_path, "rb").read()
log(f"lua tasks size: {len(data)}")

anchor = b"24.8 Write unit tests for DISKR/DISKW/FINIS/SKIP operations, return code conventions, FFCMD sequential execution, and transaction atomicity\n  - Covers: Requirement 11 (AC 11.24, 11.25, 11.26, 11.27, 11.28, 11.29, 11.30)"
if anchor not in data:
    log("ERROR: anchor not found in lua tasks.md")
else:
    append_block = b"""

---

## Phase CR Tasks

- [ ] 25. Macro Library panel (Phase CR)
  - [ ] 25.1 Add `TabKind::MacroLibrary` variant to ff-desktop; route POM option 6, `MACROS` command, and `=6` fastpath to open/switch to the Macro Library tab
    - Covers: Requirement 12.1
  - [ ] 25.2 Create `macro_library_panel.rs` in ff-desktop with `MacroLibraryPanelState` (entries list, filter string, selected index)
    - Covers: Requirement 12.2, 12.6, 12.7
  - [ ] 25.3 Populate panel entries from macro inventory; display name, source directory, full path per row
    - Covers: Requirement 12.2
  - [ ] 25.4 Implement Run action (Enter / Run button): dispatch `MACRO <name>` via command field and show result in status bar
    - Covers: Requirement 12.3
  - [ ] 25.5 Implement Edit action (F2 / Edit button): open the `.lua` file path in a new editor tab
    - Covers: Requirement 12.4
  - [ ] 25.6 Implement Delete action (Delete key / Delete button): confirmation prompt then `std::fs::remove_file` + remove from inventory
    - Covers: Requirement 12.5
  - [ ] 25.7 Implement filter input: case-insensitive substring match on macro name; re-filter on each keystroke
    - Covers: Requirement 12.6
  - [ ] 25.8 Refresh panel list when macro inventory changes (new discovery, deletion, auto-reload)
    - Covers: Requirement 12.8
  - [ ] 25.9 Wire session persistence: MacroLibrary tab kind persists/restores across sessions (tab open state only; no selection/filter)
    - Covers: Requirement 12.7
  - [ ] 25.10 Write unit tests: `macro_library_filter_case_insensitive`, `macro_library_run_dispatches_command`, `macro_library_edit_opens_tab`, `macro_library_delete_removes_entry`
    - Covers: Requirement 12.1-12.8
"""
    data = data + append_block
    with open(lua_path, "wb") as f:
        f.write(data)
    log("lua tasks.md appended OK")

# --- project-master/tasks.md (CRLF) ---
pm_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
pm = open(pm_path, "rb").read()
log(f"project-master size: {len(pm)}")

# Find the CQ section end to insert after it
cq_anchor = b"- [x] CQ.5 TCR update + cargo test --workspace green (Task 32.11-32.12)\r\n"
if cq_anchor not in pm:
    log("ERROR: CQ anchor not found")
else:
    cr_block = b"""\r\n---\r\n\r\n### Phase CR -- OS Theme Follow + Macro Library Management (CR-NR-043)\r\n\r\n> Adds OS dark/light mode follow to ff-theme/ff-desktop and a Macro Library\r\n> management panel (POM option 6) to ff-desktop.\r\n> Extends theme-and-appearance/requirements.md Req 16 and\r\n> lua-macro-engine/requirements.md Req 12.\r\n\r\n- [ ] CR.1 Requirements gate -- theme-and-appearance Req 16, lua-macro-engine Req 12, tasks, TCR rows\r\n- [ ] CR.2 OS dark/light mode follow -- theme.follow_os config key, frame-level OS detection, Settings panel checkbox\r\n- [ ] CR.3 Macro Library panel -- TabKind::MacroLibrary, MACROS command, =6 fastpath, list/run/edit/delete/filter\r\n- [ ] CR.4 TCR update + cargo test --workspace green\r\n\r\n"""
    pm = pm.replace(cq_anchor, cq_anchor + cr_block, 1)
    with open(pm_path, "wb") as f:
        f.write(pm)
    log("project-master tasks.md updated OK")

log("Done.")
