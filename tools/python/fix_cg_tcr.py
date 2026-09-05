import re

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

open(LOG, "w").close()
log("fix_cg_tcr.py started")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(path, "rb") as f:
    data = f.read()
log(f"File size: {len(data)} bytes")

count = 0
for i in range(1, 31):
    req = f"Req 11.{i}:".encode()
    # Replace the red marker line for this requirement
    for sep in (b"\r\n", b"\n"):
        pattern = b"| `ff-macro` | \xf0\x9f\x94\xb4 | -- | " + req
        replacement = b"| `ff-macro` | \xe2\x9c\x85 | `ff-lua` unit tests | " + req
        if pattern in data:
            data = data.replace(pattern, replacement, 1)
            count += 1
            log(f"  Updated Req 11.{i}")
            break
    else:
        log(f"  WARNING: Req 11.{i} pattern not found")

with open(path, "wb") as f:
    f.write(data)
log(f"Total updated: {count}/30")
log("Done")
