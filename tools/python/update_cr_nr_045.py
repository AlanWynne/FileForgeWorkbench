import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# CR-NR-045 status update
for sep in (b"\r\n", b"\n"):
    old = b"### CR-NR-045 -- Menu Workspace Pattern (Configurable ISPF-style option menus)" + sep
    if old in data:
        log(f"Found CR-NR-045 header with sep {repr(sep)}")
        # Find the Status line within the CR-NR-045 block
        status_old = b"- **Status**: PENDING GATE" + sep + b"- **Linked spec**: `docs/specs/menu-workspace/requirements.md` (to be created -- Phase CU)"
        status_new = b"- **Status**: DONE -- Phase CU complete (CU.1-CU.6), all 6 spec tasks done" + sep + b"- **Linked spec**: `docs/specs/menu-workspace/requirements.md` (created Phase CU)"
        if status_old in data:
            data = data.replace(status_old, status_new, 1)
            with open(path, "wb") as f:
                f.write(data)
            log("CR-NR-045 status updated to DONE")
        else:
            log("WARNING: status pattern not found -- trying alternate")
            # Try just the PENDING GATE line near CR-NR-045
            idx = data.find(b"### CR-NR-045")
            if idx >= 0:
                chunk = data[idx:idx+600]
                log(f"Chunk around CR-NR-045: {chunk!r}")
            else:
                log("CR-NR-045 header not found in data")
        break
else:
    log("ERROR: CR-NR-045 header not found with either separator")

log("Done")
