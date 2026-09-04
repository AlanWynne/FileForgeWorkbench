"""Patch BS.15 lines in project-master/tasks.md from [ ] to [x]."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# Both lines use plain ASCII (no em-dash) -- just two occurrences of the same text
old = b"- [ ] BS.15 Update `dataset-catalog/design.md` for CR-NR-016 (Task 30.1)"
new = b"- [x] BS.15 Update `dataset-catalog/design.md` for CR-NR-016 (Task 30.1)"

count = data.count(old)
log(f"Occurrences found: {count}")

if count == 0:
    log("ERROR: pattern not found")
    sys.exit(1)

data = data.replace(old, new)
with open(path, "wb") as f:
    f.write(data)
log(f"Replaced {count} occurrence(s).")
