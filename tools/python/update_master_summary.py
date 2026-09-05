import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\r\n' if b'\r\n' in data else b'\n'

# 1. Update the "Active work" row in the second summary (line ~861)
old_active = b'| Active work | All EARS P2 streams complete -- review open bugs or plan next phase |'
new_active = b'| Active work | All EARS P2 streams complete; Phase BS (Productivity Core) complete |'

if old_active in data:
    data = data.replace(old_active, new_active, 1)
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('active row updated\n')
else:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('active row not found -- already updated or different text\n')

# 2. Update the summary heading to reflect current state
old_heading = b'## Summary (updated after Phase CG -- current state)'
new_heading = b'## Summary (updated after Phase BS -- current state)'

if old_heading in data:
    data = data.replace(old_heading, new_heading, 1)
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('summary heading updated\n')
else:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('summary heading not found\n')

# 3. Append a new final summary section
new_section = (
    sep +
    b'---' + sep +
    sep +
    b'## Summary (updated after Phase BS Productivity Core -- current state)' + sep +
    sep +
    b'| Status | Count |' + sep +
    b'|--------|-------|' + sep +
    b'| `[x]` Complete with real tests | 62 library crates (incl. ff-global-search) + ff-desktop binary |' + sep +
    b'| `[x]` Stream 1 complete | BV, BS.1-BS.15, ff-vfs.13-16, BU.1-BU.9 |' + sep +
    b'| `[x]` Stream 2 complete | BW, BX, BY, BZ, CA, CB, CC, CD (all EARS P1) |' + sep +
    b'| `[x]` Stream 3 complete | CE, CF, CG, CH, CI (all EARS P2) |' + sep +
    b'| `[x]` Phase CJ complete | Bootstrap scripts (CJ.1-CJ.6) |' + sep +
    b'| `[x]` Phase CK complete | FFTest framework (CK.1-CK.4) |' + sep +
    b'| `[x]` Phase BS-A complete | Workspace Model (BS-A.1-BS-A.6) |' + sep +
    b'| `[x]` Phase BS-B complete | Command Palette (BS-B.1-BS-B.4) |' + sep +
    b'| `[x]` Phase BS-C complete | Global Search (BS-C.1-BS-C.5) |' + sep +
    b'| Test count | 655 passing (644 ff-desktop + 11 ff-global-search), 0 failures |' + sep +
    b'| Active work | Phase BS Productivity Core complete -- review open bugs or plan next phase |' + sep
)

if b'Phase BS Productivity Core -- current state' not in data:
    data = data + new_section
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('new summary appended\n')
else:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('new summary already present\n')

with open(path, 'wb') as f:
    f.write(data)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('DONE\n')
