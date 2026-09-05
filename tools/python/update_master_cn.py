import sys, shutil

LOG = r"c:\workspace\VSC\FileForgeWorkbench\tools\logs\update_master_cn.txt"
TARGET = r"c:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(LOG, "w", encoding="utf-8") as f:
    f.write("=== update_master_cn.py ===\n")

with open(TARGET, "rb") as f:
    data = f.read()
log(f"File size: {len(data)} bytes")

# CM.1 and CM.2 are already [x] -- just append Phase CN after CM.2
OLD = None
for sep in (b"\r\n", b"\n"):
    candidate = (
        b"- [x] CM.2 Read-only panel selectable labels -- POM calendar, Settings key/desc/badge, status bar fields (Tasks 18.1-18.5 in caret-and-selection/tasks.md)"
    )
    if candidate in data:
        OLD = candidate
        log(f"Found CM.2 line with sep {repr(sep)}")
        break

if OLD is None:
    log("ERROR: CM.2 pattern not found")
    sys.exit(1)

NEW = (
    b"- [x] CM.2 Read-only panel selectable labels -- POM calendar, Settings key/desc/badge, status bar fields (Tasks 18.1-18.5 in caret-and-selection/tasks.md)\r\n"
    b"\r\n"
    b"### Phase CN -- Editor Scroll Amount Integration (CR-NR-035)\r\n"
    b"\r\n"
    b"> Wires the existing SCROLL ===> field value (PAGE/HALF/CSR/MAX/DATA/N) into the editor\r\n"
    b"> panel Page Up/Down handler so paging behaviour matches the ISPF convention.\r\n"
    b"> Requirement 14 in viewport-and-scrolling/requirements.md.\r\n"
    b"\r\n"
    b"- [ ] CN.1 Pass scroll_amount into editor_panel::render(); implement scroll_by_amount helper; wire all ScrollAmount variants; 5 unit tests (Task 16.1-16.6 in viewport-and-scrolling/tasks.md)"
)

data = data.replace(OLD, NEW, 1)
tmp = TARGET + ".tmp"
with open(tmp, "wb") as f:
    f.write(data)
shutil.move(tmp, TARGET)
log(f"Done. New size: {len(data)} bytes")
