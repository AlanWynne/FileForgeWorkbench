import os, re

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\tcr-update.txt"
TCR = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

with open(LOG, "w", encoding="utf-8") as lf:
    lf.write("start\n")

def log(msg):
    with open(LOG, "a", encoding="utf-8") as lf:
        lf.write(msg + "\n")

log(f"TCR exists: {os.path.exists(TCR)}")

with open(TCR, "rb") as f:
    data = f.read()

log(f"TCR size: {len(data)}")

# Find the Phase CR section
idx = data.find(b"Phase CR")
log(f"Phase CR idx: {idx}")
if idx >= 0:
    log(repr(data[idx:idx+500]))

# Find specific rows
for needle in [b"theme.follow_os config key", b"Macro Library panel via POM"]:
    i = data.find(needle)
    log(f"'{needle[:40]}' at {i}")
    if i >= 0:
        log(repr(data[max(0,i-100):i+100]))
