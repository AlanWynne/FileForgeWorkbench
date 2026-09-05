"""Update TCR.md: mark Req 3.3 workspace sidebar row from NOT COVERED to PASS."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

old_row = b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.3: workspace roots displayed as top-level nodes in File Explorer |"
new_row = b"| `ff-desktop` | \xe2\x9c\x85 | `file_explorer_panel::tests::workspace_roots_collected_for_sidebar_display` | Req 3.3: workspace roots displayed as top-level nodes in File Explorer |"

replaced = False
if old_row in data:
    data = data.replace(old_row, new_row, 1)
    replaced = True
    log("Replaced row")

if not replaced:
    log("Pattern not found -- searching for partial match")
    idx = data.find(b"Req 3.3: workspace roots")
    if idx >= 0:
        log(f"Found 'Req 3.3: workspace roots' at byte {idx}")
        context = data[max(0, idx-80):idx+120]
        log(f"Context: {context!r}")
    else:
        log("ERROR: 'Req 3.3: workspace roots' not found in file")
    sys.exit(1)

with open(path, "wb") as f:
    f.write(data)
log("Done")
