"""Remove duplicate '- **Status**: IN PROGRESS' line from CR-NR-019 entry."""
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

idx = data.find(b"CR-NR-019")
if idx == -1:
    log("ERROR: CR-NR-019 not found")
    sys.exit(1)

log(f"CR-NR-019 at offset {idx}")
log("Chunk: " + repr(data[idx:idx+500]))

fixed = False
for sep in (b"\r\n", b"\n"):
    pattern = b"- **Status**: IN PROGRESS" + sep + b"- **Status**: DONE"
    if pattern in data:
        log(f"Pattern found with sep={repr(sep)}")
        data = data.replace(pattern, b"- **Status**: DONE", 1)
        with open(path, "wb") as f:
            f.write(data)
        log("Done -- duplicate line removed")
        fixed = True
        break

if not fixed:
    log("Pattern not found with either separator -- inspect chunk above")
