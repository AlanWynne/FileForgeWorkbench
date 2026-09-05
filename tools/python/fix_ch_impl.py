import re

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

open(LOG, "w").close()
log("fix_ch_impl.py started")

# --- Task markers -----------------------------------------------------------
task_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\FFW-JES\tasks.md"
with open(task_path, "rb") as f:
    data = f.read()
log(f"Tasks file: {len(data)} bytes")

task_replacements = [
    (b"- [ ] 30. Overtype fields", b"- [x] 30. Overtype fields"),
    (b"  - [ ] 30.1 Implement visual distinction", b"  - [x] 30.1 Implement visual distinction"),
    (b"  - [ ] 30.2 Implement direct overtype:", b"  - [x] 30.2 Implement direct overtype:"),
    (b"  - [ ] 30.3 Implement command-line overtype syntax:", b"  - [x] 30.3 Implement command-line overtype syntax:"),
    (b"  - [ ] 30.4 Implement Overtype Extension pop-up", b"  - [x] 30.4 Implement Overtype Extension pop-up"),
    (b"  - [ ] 30.5 Write unit tests for overtype", b"  - [x] 30.5 Write unit tests for overtype"),
    (b"- [ ] 31. Help system (HELP, ACTH, COLH, CMDH, SEARCH)", b"- [x] 31. Help system (HELP, ACTH, COLH, CMDH, SEARCH)"),
    (b"  - [ ] 31.1 Implement context-sensitive HELP", b"  - [x] 31.1 Implement context-sensitive HELP"),
    (b"  - [ ] 31.2 Implement ACTH command:", b"  - [x] 31.2 Implement ACTH command:"),
    (b"  - [ ] 31.3 Implement COLH command:", b"  - [x] 31.3 Implement COLH command:"),
    (b"  - [ ] 31.4 Implement CMDH command:", b"  - [x] 31.4 Implement CMDH command:"),
    (b"  - [ ] 31.5 Implement SEARCH <text>", b"  - [x] 31.5 Implement SEARCH <text>"),
    (b"  - [ ] 31.6 Write unit tests for HELP panel", b"  - [x] 31.6 Write unit tests for HELP panel"),
    (b"- [ ] 32. Log panels (LOG, ULOG, NEXT, PREV, SNAPSHOT)", b"- [x] 32. Log panels (LOG, ULOG, NEXT, PREV, SNAPSHOT)"),
    (b"  - [ ] 32.1 Implement LOG command:", b"  - [x] 32.1 Implement LOG command:"),
    (b"  - [ ] 32.2 Implement ULOG command:", b"  - [x] 32.2 Implement ULOG command:"),
    (b"  - [ ] 32.3 Implement NEXT/PREV commands", b"  - [x] 32.3 Implement NEXT/PREV commands"),
    (b"  - [ ] 32.4 Implement SNAPSHOT command:", b"  - [x] 32.4 Implement SNAPSHOT command:"),
    (b"  - [ ] 32.5 Implement SYS panel:", b"  - [x] 32.5 Implement SYS panel:"),
    (b"  - [ ] 32.6 Implement DASH panel:", b"  - [x] 32.6 Implement DASH panel:"),
    (b"  - [ ] 32.7 Implement INIT panel:", b"  - [x] 32.7 Implement INIT panel:"),
    (b"  - [ ] 32.8 Implement JC panel:", b"  - [x] 32.8 Implement JC panel:"),
    (b"  - [ ] 32.9 Implement SP panel:", b"  - [x] 32.9 Implement SP panel:"),
    (b"  - [ ] 32.10 Write unit tests for LOG/ULOG", b"  - [x] 32.10 Write unit tests for LOG/ULOG"),
    (b"- [ ] 33. Browse and print", b"- [x] 33. Browse and print"),
    (b"  - [ ] 33.1 Implement browse settings:", b"  - [x] 33.1 Implement browse settings:"),
    (b"  - [ ] 33.2 Implement PRINT action character:", b"  - [x] 33.2 Implement PRINT action character:"),
    (b"  - [ ] 33.3 Implement COLS command", b"  - [x] 33.3 Implement COLS command"),
    (b"  - [ ] 33.4 Write unit tests for browse settings", b"  - [x] 33.4 Write unit tests for browse settings"),
    (b"- [ ] 34. SET P2 commands and persistence", b"- [x] 34. SET P2 commands and persistence"),
    (b"  - [ ] 34.1 Implement SET BCOLOR", b"  - [x] 34.1 Implement SET BCOLOR"),
    (b"  - [ ] 34.2 Implement SET CONFIRM ON/OFF:", b"  - [x] 34.2 Implement SET CONFIRM ON/OFF:"),
    (b"  - [ ] 34.3 Implement SET CURSOR <field>:", b"  - [x] 34.3 Implement SET CURSOR <field>:"),
    (b"  - [ ] 34.4 Implement SET DATE <format>:", b"  - [x] 34.4 Implement SET DATE <format>:"),
    (b"  - [ ] 34.5 Implement SET DELAY <seconds>:", b"  - [x] 34.5 Implement SET DELAY <seconds>:"),
    (b"  - [ ] 34.6 Implement SET HEX ON/OFF:", b"  - [x] 34.6 Implement SET HEX ON/OFF:"),
    (b"  - [ ] 34.7 Implement SET SCHARS <chars>:", b"  - [x] 34.7 Implement SET SCHARS <chars>:"),
    (b"  - [ ] 34.8 Implement SET SCREEN <rows> <cols>:", b"  - [x] 34.8 Implement SET SCREEN <rows> <cols>:"),
    (b"  - [ ] 34.9 Implement SET P2 persistence:", b"  - [x] 34.9 Implement SET P2 persistence:"),
    (b"  - [ ] 34.10 Write unit tests for each SET P2", b"  - [x] 34.10 Write unit tests for each SET P2"),
]

count = 0
for old, new in task_replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
    else:
        log(f"  WARNING: not found: {old[:60]}")

with open(task_path, "wb") as f:
    f.write(data)
log(f"Tasks: {count}/{len(task_replacements)} replacements")

# --- project-master ---------------------------------------------------------
master_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
with open(master_path, "rb") as f:
    mdata = f.read()
log(f"Master file: {len(mdata)} bytes")

mdata = mdata.replace(
    b"- [ ] CH.impl FFW-JES P2:",
    b"- [x] CH.impl FFW-JES P2:",
    1
)
with open(master_path, "wb") as f:
    f.write(mdata)
log("Master: CH.impl marked [x]")

# --- TCR --------------------------------------------------------------------
tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(tcr_path, "rb") as f:
    tdata = f.read()
log(f"TCR file: {len(tdata)} bytes")

tcr_count = 0
for i in range(1, 31):
    req = f"Req 18.{i}:".encode()
    pattern = b"| `ff-jes` | \xf0\x9f\x94\xb4 | -- | " + req
    replacement = b"| `ff-jes` | \xe2\x9c\x85 | `ff-jes` unit tests | " + req
    if pattern in tdata:
        tdata = tdata.replace(pattern, replacement, 1)
        tcr_count += 1
        log(f"  TCR Req 18.{i} updated")
    else:
        log(f"  TCR Req 18.{i} not found (may already be updated)")

with open(tcr_path, "wb") as f:
    f.write(tdata)
log(f"TCR: {tcr_count} rows updated")
log("Done")
