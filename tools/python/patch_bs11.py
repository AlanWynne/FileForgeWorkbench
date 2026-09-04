"""Patch BS.11 entries in project-master/tasks.md to [x]."""
import sys

LOG = r"c:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"c:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

replacements = [
    # Line 534 variant (em-dash, en-dash)
    (b"- [ ] BS.11 Security hardening \xe2\x80\x94 param\xc3\xa9tris\xc3\xa9d SQL audit, log scrubbing, path-traversal property test (Tasks 26.1\xe2\x80\x9326.3)",
     b"- [x] BS.11 Security hardening \xe2\x80\x94 param\xc3\xa9tris\xc3\xa9d SQL audit, log scrubbing, path-traversal property test (Tasks 26.1\xe2\x80\x9326.3)"),
    # Line 778 variant (plain dashes)
    (b"- [ ] BS.11 Security hardening -- parameterised SQL audit, log scrubbing, path-traversal PBT (Tasks 26.1-26.3)",
     b"- [x] BS.11 Security hardening -- parameterised SQL audit, log scrubbing, path-traversal PBT (Tasks 26.1-26.3)"),
]

changed = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Replaced: {old[:60]}")
        changed += 1
    else:
        log(f"Not found: {old[:60]}")

if changed > 0:
    with open(path, "wb") as f:
        f.write(data)
    log(f"Written {changed} replacement(s)")
else:
    log("ERROR: no replacements made")
    sys.exit(1)

log("Done")
