"""Patch Phase CC documentation: tasks.md, TCR.md, project-master/tasks.md"""
import sys, os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

def patch_file(path, old, new, label):
    with open(path, "rb") as f:
        data = f.read()
    for sep in (b"\r\n", b"\n"):
        o = old.replace(b"\n", sep)
        n = new.replace(b"\n", sep)
        if o in data:
            data = data.replace(o, n, 1)
            with open(path, "wb") as f:
                f.write(data)
            log(f"OK: {label}")
            return
    log(f"MISS: {label}")

BASE = r"C:\workspace\VSC\FileForgeWorkbench"

# Clear log
open(LOG, "w").close()
log("patch_cc_docs.py started")

# --- tasks.md: mark tasks 20-25 done ---
tasks_path = os.path.join(BASE, "docs", "specs", "FFW-JES", "tasks.md")

patches = [
    (b"- [ ] 20. SDSF panel framework core -- action bar, title line, SCROLL field, filter lines",
     b"- [x] 20. SDSF panel framework core -- action bar, title line, SCROLL field, filter lines",
     "task 20 header"),
    (b"  - [ ] 20.1 Implement action bar with pull-down menus (File, View, Help) in JobMonitorPanel",
     b"  - [x] 20.1 Implement action bar with pull-down menus (File, View, Help) in JobMonitorPanel",
     "task 20.1"),
    (b"  - [ ] 20.2 Implement title line with panel name and visible row range display",
     b"  - [x] 20.2 Implement title line with panel name and visible row range display",
     "task 20.2"),
    (b"  - [ ] 20.3 Implement SCROLL ===> field adjacent to command input, retaining last-used amount",
     b"  - [x] 20.3 Implement SCROLL ===> field adjacent to command input, retaining last-used amount",
     "task 20.3"),
    (b"  - [ ] 20.4 Implement filter information lines (PREFIX=/DEST=/OWNER=) below title line",
     b"  - [x] 20.4 Implement filter information lines (PREFIX=/DEST=/OWNER=) below title line",
     "task 20.4"),
    (b"  - [ ] 20.5 Implement title line message area for command feedback",
     b"  - [x] 20.5 Implement title line message area for command feedback",
     "task 20.5"),
    (b"  - [ ] 20.6 Implement COMMAND INPUT ===> field for SDSF commands",
     b"  - [x] 20.6 Implement COMMAND INPUT ===> field for SDSF commands",
     "task 20.6"),
    (b"  - [ ] 20.7 Write unit tests for panel chrome: title line content, filter line display, SCROLL field retention, message area update",
     b"  - [x] 20.7 Write unit tests for panel chrome: title line content, filter line display, SCROLL field retention, message area update",
     "task 20.7"),
    (b"- [ ] 21. NP column and action character system",
     b"- [x] 21. NP column and action character system",
     "task 21 header"),
    (b"  - [ ] 21.1 Implement NP column as fixed leftmost column (non-scrolling) with action character input",
     b"  - [x] 21.1 Implement NP column as fixed leftmost column (non-scrolling) with action character input",
     "task 21.1"),
    (b"  - [ ] 21.2 Implement fixed JOBNAME column during horizontal scroll",
     b"  - [x] 21.2 Implement fixed JOBNAME column during horizontal scroll",
     "task 21.2"),
    (b"  - [ ] 21.3 Implement action character dispatch: S, ?, C, H, A, P, D, E, J, W",
     b"  - [x] 21.3 Implement action character dispatch: S, ?, C, H, A, P, D, E, J, W",
     "task 21.3"),
    (b"  - [ ] 21.4 Implement = repeat action character",
     b"  - [x] 21.4 Implement = repeat action character",
     "task 21.4"),
    (b"  - [ ] 21.5 Implement // block action syntax (first and last row of block)",
     b"  - [x] 21.5 Implement // block action syntax (first and last row of block)",
     "task 21.5"),
    (b"  - [ ] 21.6 Implement command-line action syntax (\"2 C\" in command field)",
     b"  - [x] 21.6 Implement command-line action syntax (\"2 C\" in command field)",
     "task 21.6"),
    (b"  - [ ] 21.7 Implement SET ROWNUM ON/OFF -- row numbers in NP area",
     b"  - [x] 21.7 Implement SET ROWNUM ON/OFF -- row numbers in NP area",
     "task 21.7"),
    (b"  - [ ] 21.8 Write unit tests for NP column: action dispatch, repeat =, block //, command-line syntax, invalid action rejection, SET ROWNUM toggle",
     b"  - [x] 21.8 Write unit tests for NP column: action dispatch, repeat =, block //, command-line syntax, invalid action rejection, SET ROWNUM toggle",
     "task 21.8"),
    (b"- [ ] 22. Main panel (MENU command) and command groups",
     b"- [x] 22. Main panel (MENU command) and command groups",
     "task 22 header"),
    (b"  - [ ] 22.1 Implement MENU command navigating to main panel listing all SDSF panel commands",
     b"  - [x] 22.1 Implement MENU command navigating to main panel listing all SDSF panel commands",
     "task 22.1"),
    (b"  - [ ] 22.2 Implement command groups (Jobs, Output, JES, Log, Memory, Other) with expand/collapse",
     b"  - [x] 22.2 Implement command groups (Jobs, Output, JES, Log, Memory, Other) with expand/collapse",
     "task 22.2"),
    (b"  - [ ] 22.3 Implement S action on main panel row to navigate to selected panel",
     b"  - [x] 22.3 Implement S action on main panel row to navigate to selected panel",
     "task 22.3"),
    (b"  - [ ] 22.4 Implement SET MAIN GROUP command for grouped main panel display",
     b"  - [x] 22.4 Implement SET MAIN GROUP command for grouped main panel display",
     "task 22.4"),
    (b"  - [ ] 22.5 Write unit tests for main panel: group rendering, S action navigation, SET MAIN GROUP toggle, MENU command from sub-panel",
     b"  - [x] 22.5 Write unit tests for main panel: group rendering, S action navigation, SET MAIN GROUP toggle, MENU command from sub-panel",
     "task 22.5"),
    (b"- [ ] 23. PREFIX, OWNER, DEST filter commands",
     b"- [x] 23. PREFIX, OWNER, DEST filter commands",
     "task 23 header"),
    (b"  - [ ] 23.1 Implement PREFIX filter command -- filter job list by job name prefix; PREFIX * clears",
     b"  - [x] 23.1 Implement PREFIX filter command -- filter job list by job name prefix; PREFIX * clears",
     "task 23.1"),
    (b"  - [ ] 23.2 Implement OWNER filter command -- filter by job owner; OWNER * clears",
     b"  - [x] 23.2 Implement OWNER filter command -- filter by job owner; OWNER * clears",
     "task 23.2"),
    (b"  - [ ] 23.3 Implement DEST filter command -- filter by output destination; DEST * clears",
     b"  - [x] 23.3 Implement DEST filter command -- filter by output destination; DEST * clears",
     "task 23.3"),
    (b"  - [ ] 23.4 Write unit tests for filter commands: PREFIX match, OWNER match, DEST match, wildcard clear, combined filters, filter persistence across tab switch",
     b"  - [x] 23.4 Write unit tests for filter commands: PREFIX match, OWNER match, DEST match, wildcard clear, combined filters, filter persistence across tab switch",
     "task 23.4"),
    (b"- [ ] 24. Job table column definitions and SORT command",
     b"- [x] 24. Job table column definitions and SORT command",
     "task 24 header"),
    (b"  - [ ] 24.1 Implement full column set: JOBNAME, JOBID, OWNER, STATUS, CLASS, PRTY, QUEUE, START, END, RC, STEPNAME, PROCSTEP",
     b"  - [x] 24.1 Implement full column set: JOBNAME, JOBID, OWNER, STATUS, CLASS, PRTY, QUEUE, START, END, RC, STEPNAME, PROCSTEP",
     "task 24.1"),
    (b"  - [ ] 24.2 Implement column hide/show and reorder support",
     b"  - [x] 24.2 Implement column hide/show and reorder support",
     "task 24.2"),
    (b"  - [ ] 24.3 Implement SORT command -- SORT colname [A|D]; SORT with no args restores submission-time order",
     b"  - [x] 24.3 Implement SORT command -- SORT colname [A|D]; SORT with no args restores submission-time order",
     "task 24.3"),
    (b"  - [ ] 24.4 Write unit tests for column definitions: all columns present, hide/show toggle, SORT ascending/descending, SORT reset",
     b"  - [x] 24.4 Write unit tests for column definitions: all columns present, hide/show toggle, SORT ascending/descending, SORT reset",
     "task 24.4"),
    (b"- [ ] 25. Integration tests for SDSF panel framework",
     b"- [x] 25. Integration tests for SDSF panel framework",
     "task 25 header"),
    (b"  - [ ] 25.1 Write integration test: full NP column action cycle -- enter action char, verify dispatch, verify message area feedback",
     b"  - [x] 25.1 Write integration test: full NP column action cycle -- enter action char, verify dispatch, verify message area feedback",
     "task 25.1"),
    (b"  - [ ] 25.2 Write integration test: PREFIX + OWNER + DEST combined filter -- verify only matching jobs shown",
     b"  - [x] 25.2 Write integration test: PREFIX + OWNER + DEST combined filter -- verify only matching jobs shown",
     "task 25.2"),
    (b"  - [ ] 25.3 Write integration test: SORT + filter interaction -- sort filtered result, verify order preserved after filter change",
     b"  - [x] 25.3 Write integration test: SORT + filter interaction -- sort filtered result, verify order preserved after filter change",
     "task 25.3"),
    (b"  - [ ] 25.4 Write integration test: MENU navigation -- MENU from input queue, S to select panel, verify navigation",
     b"  - [x] 25.4 Write integration test: MENU navigation -- MENU from input queue, S to select panel, verify navigation",
     "task 25.4"),
]

for old, new, label in patches:
    patch_file(tasks_path, old, new, label)

# --- TCR.md: update 26 rows from red to green ---
tcr_path = os.path.join(BASE, "docs", "quality", "TCR.md")

tcr_patches = [
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.1: action bar with pull-down menus (File, View, Help) |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.1: action bar with pull-down menus (File, View, Help) |",
     "TCR 16.1"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.2: title line with panel name and visible row range |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.2: title line with panel name and visible row range |",
     "TCR 16.2"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.3: SCROLL ===> field retains last-used scroll amount |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.3: SCROLL ===> field retains last-used scroll amount |",
     "TCR 16.3"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.4: filter information lines PREFIX=/DEST=/OWNER= below title |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.4: filter information lines PREFIX=/DEST=/OWNER= below title |",
     "TCR 16.4"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.5: NP column fixed leftmost, non-scrolling |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.5: NP column fixed leftmost, non-scrolling |",
     "TCR 16.5"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.6: JOBNAME column fixed during horizontal scroll |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.6: JOBNAME column fixed during horizontal scroll |",
     "TCR 16.6"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.7: action character in NP column dispatches action on Enter |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.7: action character in NP column dispatches action on Enter |",
     "TCR 16.7"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.8: action characters S/?/C/H/A/P/D/E/J/W supported |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.8: action characters S/?/C/H/A/P/D/E/J/W supported |",
     "TCR 16.8"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.9: = repeats previous action character on that row |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.9: = repeats previous action character on that row |",
     "TCR 16.9"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.10: // block action applies to all rows in block |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.10: // block action applies to all rows in block |",
     "TCR 16.10"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.11: command-line action syntax \"2 C\" in command field |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.11: command-line action syntax \"2 C\" in command field |",
     "TCR 16.11"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.12: SET ROWNUM ON displays row numbers in NP area |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.12: SET ROWNUM ON displays row numbers in NP area |",
     "TCR 16.12"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.13: main panel lists all SDSF commands with name/desc/group |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.13: main panel lists all SDSF commands with name/desc/group |",
     "TCR 16.13"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.14: command groups (Jobs/Output/JES/Log/Memory/Other) expandable |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.14: command groups (Jobs/Output/JES/Log/Memory/Other) expandable |",
     "TCR 16.14"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.15: S action on main panel row navigates to selected panel |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.15: S action on main panel row navigates to selected panel |",
     "TCR 16.15"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.16: SET MAIN GROUP displays grouped main panel |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.16: SET MAIN GROUP displays grouped main panel |",
     "TCR 16.16"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.17: MENU command returns to main panel from any sub-panel |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.17: MENU command returns to main panel from any sub-panel |",
     "TCR 16.17"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.18: PREFIX filter -- filter by job name prefix; PREFIX * clears |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.18: PREFIX filter -- filter by job name prefix; PREFIX * clears |",
     "TCR 16.18"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.19: OWNER filter -- filter by job owner; OWNER * clears |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.19: OWNER filter -- filter by job owner; OWNER * clears |",
     "TCR 16.19"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.20: DEST filter -- filter by output destination; DEST * clears |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.20: DEST filter -- filter by output destination; DEST * clears |",
     "TCR 16.20"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.21: title line message area shows last command feedback |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.21: title line message area shows last command feedback |",
     "TCR 16.21"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.22: COMMAND INPUT ===> field for SDSF commands |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_panel.rs unit tests | Req 16.22: COMMAND INPUT ===> field for SDSF commands |",
     "TCR 16.22"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.23: NP column supports full action char set; invalid state rejected with message |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_action.rs unit tests | Req 16.23: NP column supports full action char set; invalid state rejected with message |",
     "TCR 16.23"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.24: columns JOBNAME/JOBID/OWNER/STATUS/CLASS/PRTY/QUEUE/START/END/RC/STEPNAME/PROCSTEP; hideable/reorderable |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.24: columns JOBNAME/JOBID/OWNER/STATUS/CLASS/PRTY/QUEUE/START/END/RC/STEPNAME/PROCSTEP; hideable/reorderable |",
     "TCR 16.24"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.25: PREFIX/OWNER/DEST filter fields as editable in-place rows above table |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.25: PREFIX/OWNER/DEST filter fields as editable in-place rows above table |",
     "TCR 16.25"),
    (b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | Req 16.26: SORT colname [A|D] sorts job table; SORT with no args restores submission-time order |",
     b"| `ff-jes` | \xe2\x9c\x85 | sdsf_filter.rs unit tests | Req 16.26: SORT colname [A|D] sorts job table; SORT with no args restores submission-time order |",
     "TCR 16.26"),
]

for old, new, label in tcr_patches:
    patch_file(tcr_path, old, new, label)

# --- project-master/tasks.md: mark CC.1-CC.6 done ---
master_path = os.path.join(BASE, "docs", "specs", "project-master", "tasks.md")

master_patches = [
    (b"- [ ] CC.1 SDSF panel chrome -- action bar, title line, SCROLL field, filter lines, message area, COMMAND INPUT field (Tasks 20.1-20.7)",
     b"- [x] CC.1 SDSF panel chrome -- action bar, title line, SCROLL field, filter lines, message area, COMMAND INPUT field (Tasks 20.1-20.7)",
     "CC.1"),
    (b"- [ ] CC.2 NP column and action character system -- S/?/C/H/A/P/D/E/J/W, = repeat, // block, command-line syntax, SET ROWNUM (Tasks 21.1-21.8)",
     b"- [x] CC.2 NP column and action character system -- S/?/C/H/A/P/D/E/J/W, = repeat, // block, command-line syntax, SET ROWNUM (Tasks 21.1-21.8)",
     "CC.2"),
    (b"- [ ] CC.3 Main panel (MENU command), command groups, S action, SET MAIN GROUP (Tasks 22.1-22.5)",
     b"- [x] CC.3 Main panel (MENU command), command groups, S action, SET MAIN GROUP (Tasks 22.1-22.5)",
     "CC.3"),
    (b"- [ ] CC.4 PREFIX/OWNER/DEST filter commands (Tasks 23.1-23.4)",
     b"- [x] CC.4 PREFIX/OWNER/DEST filter commands (Tasks 23.1-23.4)",
     "CC.4"),
    (b"- [ ] CC.5 Full column set (JOBNAME through PROCSTEP), column hide/reorder, SORT command (Tasks 24.1-24.4)",
     b"- [x] CC.5 Full column set (JOBNAME through PROCSTEP), column hide/reorder, SORT command (Tasks 24.1-24.4)",
     "CC.5"),
    (b"- [ ] CC.6 Integration tests for SDSF panel framework (Tasks 25.1-25.4)",
     b"- [x] CC.6 Integration tests for SDSF panel framework (Tasks 25.1-25.4)",
     "CC.6"),
]

for old, new, label in master_patches:
    patch_file(master_path, old, new, label)

log("Done")
