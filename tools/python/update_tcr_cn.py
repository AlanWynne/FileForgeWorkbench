import sys, shutil

LOG = r"c:\workspace\VSC\FileForgeWorkbench\tools\logs\update_tcr_cn.txt"
TARGET = r"c:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(LOG, "w", encoding="utf-8") as f:
    f.write("=== update_tcr_cn.py ===\n")

with open(TARGET, "rb") as f:
    data = f.read()
log(f"File size: {len(data)} bytes")

APPEND = (
    b"\r\n"
    b"### Phase CN -- Editor Scroll Amount Integration (CR-NR-035)\r\n"
    b"\r\n"
    b"| Crate | Status | Test files | Notes |\r\n"
    b"|-------|--------|-----------|-------|\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.1: PAGE scroll amount scrolls full visible_count lines |\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.2: HALF scroll amount scrolls max(1, visible_count/2) lines |\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.3: CSR Page Down scrolls so cursor is first visible line |\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.4: CSR Page Up scrolls so cursor is last visible line |\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.5: numeric N scroll amount scrolls exactly N lines |\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.6: MAX Page Down scrolls to last page; MAX Page Up scrolls to first line |\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.7: DATA scroll amount scrolls visible_count - 1 lines |\r\n"
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 14.8: SCROLL ===> field visible and editable when editor tab is active |\r\n"
)

data = data + APPEND
tmp = TARGET + ".tmp"
with open(tmp, "wb") as f:
    f.write(data)
shutil.move(tmp, TARGET)
log(f"Done. New size: {len(data)} bytes")
