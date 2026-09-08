import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\ct4_config_fix2.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "w", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\configuration-system\requirements.md"

with open(path, "rb") as f:
    data = f.read()

# Fix the em dash in heading (U+2014 = \xe2\x80\x94 in UTF-8)
old = b"Settings Context \xe2\x80\x94 Interactive"
new = b"Settings Context -- Interactive"
if old in data:
    data = data.replace(old, new)
    log("Fixed em dash in heading")
else:
    log("em dash not found in heading")

# Fix remaining "Settings panel" on line 343
old2 = b"6. THE Settings panel SHALL display a lock indicator"
new2 = b"6. THE Settings Context SHALL display a lock indicator"
if old2 in data:
    data = data.replace(old2, new2)
    log("Fixed lock indicator line")
else:
    log("lock indicator line not found")

with open(path, "wb") as f:
    f.write(data)

log("Done.")
