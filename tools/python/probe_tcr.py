import sys
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
open(LOG, "w").close()
def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

data = open(r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md", "rb").read()
log(f"TCR size: {len(data)}")

# Find Phase CN section
idx = data.find(b"Phase CN")
log(f"Phase CN at: {idx}")
if idx >= 0:
    log(repr(data[idx:idx+800]))
log("Done.")
