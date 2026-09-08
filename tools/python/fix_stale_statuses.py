"""Fix stale IN PROGRESS statuses in change-log.md and project-master/tasks.md."""
import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\fix_stale_statuses.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

log("=== fix_stale_statuses.py ===")

# --- change-log.md ---
cl_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"
with open(cl_path, "rb") as f:
    data = f.read()
log(f"change-log.md size: {len(data)} bytes")

sections = [
    b"CR-NR-040",
    b"CR-NR-041",
    b"CR-NR-042",
]

for marker in sections:
    idx = data.find(marker)
    if idx == -1:
        log(f"ERROR: {marker} not found")
        continue
    # Search within 1500 bytes of section start
    window_end = idx + 1500
    old = b"- **Status**: IN PROGRESS"
    pos = data.find(old, idx, window_end)
    if pos == -1:
        log(f"WARNING: IN PROGRESS not found near {marker.decode()}")
        continue
    data = data[:pos] + b"- **Status**: DONE" + data[pos + len(old):]
    log(f"Fixed {marker.decode()} status")

with open(cl_path, "wb") as f:
    f.write(data)
log("change-log.md written")

# --- project-master/tasks.md ---
pm_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
with open(pm_path, "rb") as f:
    pm = f.read()
log(f"project-master/tasks.md size: {len(pm)} bytes")

# Fix stale summary row: "Active work | Phase CQ -- Enterprise Features (requirements gate pending)"
for sep in (b"\r\n", b"\n"):
    old = b"| Active work | Phase CQ -- Enterprise Features (requirements gate pending) |" + sep
    if old in pm:
        new = b"| Active work | Phase CQ -- Enterprise Features COMPLETE. All 5 deliverables done. |" + sep
        pm = pm.replace(old, new, 1)
        log(f"Fixed project-master summary row (sep={repr(sep)})")
        break
else:
    log("WARNING: stale summary row not found in project-master/tasks.md (may already be correct)")

with open(pm_path, "wb") as f:
    f.write(pm)
log("project-master/tasks.md written")

log("Done.")
