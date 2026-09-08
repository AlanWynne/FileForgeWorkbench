"""Update CR-NR-046 and CR-CH-010 status to IN PROGRESS in change-log.md."""
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

changes = [
    (
        b"- **Status**: PENDING GATE\r\n- **Linked spec**: `docs/specs/function-keys-and-history/requirements.md` (extension to Req 20)",
        b"- **Status**: IN PROGRESS\r\n- **Linked spec**: `docs/specs/function-keys-and-history/requirements.md` (extension to Req 20), `docs/specs/function-keys-and-history/cx-requirements.md` (new)"
    ),
    (
        b"- **Status**: PENDING GATE\r\n- **Affects**: `docs/specs/layout-and-docking/requirements.md` Req 3; `ff-desktop` command handler",
        b"- **Status**: IN PROGRESS\r\n- **Affects**: `docs/specs/layout-and-docking/requirements.md` Req 3; `ff-desktop` command handler; `docs/specs/function-keys-and-history/cx-requirements.md` Req 3"
    ),
]

for old, new in changes:
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Updated: {old[:60]!r}...")
    else:
        # Try LF variant
        old_lf = old.replace(b"\r\n", b"\n")
        new_lf = new.replace(b"\r\n", b"\n")
        if old_lf in data:
            data = data.replace(old_lf, new_lf, 1)
            log(f"Updated (LF): {old_lf[:60]!r}...")
        else:
            log(f"WARNING: pattern not found: {old[:60]!r}...")

with open(path, "wb") as f:
    f.write(data)

log("Done")
