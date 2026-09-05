"""Fix Phase CN TCR row 14.8 and CN.1 master task."""
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
open(LOG, "w").close()
def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

RED  = b"\xf0\x9f\x94\xb4"
MANU = b"\xf0\x9f\x94\xb2"

# ── TCR: find the Phase CN Req 14.8 row (SCROLL field visibility) ────────────
TCR = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
data = open(TCR, "rb").read()
log(f"TCR before: {len(data)}")

# Find all occurrences of "Req 14.8"
start = 0
while True:
    idx = data.find(b"Req 14.8", start)
    if idx < 0:
        break
    log(f"Req 14.8 at {idx}: {repr(data[max(0,idx-60):idx+80])}")
    start = idx + 1

# The Phase CN one mentions SCROLL field
idx = data.find(b"Req 14.8: SCROLL")
if idx >= 0:
    row_start = data.rfind(b"| `ff-desktop`", 0, idx)
    row_end = data.find(b"\n", idx) + 1
    actual = data[row_start:row_end]
    log(f"Phase CN 14.8: {repr(actual)}")
    new = actual.replace(RED, MANU, 1)
    if new != actual:
        data = data.replace(actual, new, 1)
        log("Phase CN 14.8 -> MANUAL")
    else:
        log("Phase CN 14.8 already not RED")
else:
    log("Phase CN Req 14.8 SCROLL not found")

open(TCR, "wb").write(data)
log(f"TCR after: {len(data)}")

# ── project-master/tasks.md: mark CN.1 done ──────────────────────────────────
MASTER = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
mdata = open(MASTER, "rb").read()
for sep in (b"\r\n", b"\n"):
    old = b"- [ ] CN.1" + sep
    if old in mdata:
        mdata = mdata.replace(old, b"- [x] CN.1" + sep, 1)
        log(f"CN.1 marked done (sep={repr(sep)})")
        break
else:
    # Try without trailing sep
    if b"- [ ] CN.1" in mdata:
        mdata = mdata.replace(b"- [ ] CN.1", b"- [x] CN.1", 1)
        log("CN.1 marked done (no sep)")
    else:
        log("CN.1 not found")
open(MASTER, "wb").write(mdata)
log(f"master after: {len(mdata)}")
log("Done.")
