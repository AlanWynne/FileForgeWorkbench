"""Patch BS.14 lines in project-master/tasks.md from [ ] to [x]."""
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

replacements = [
    # Wave 4 line with em-dash (U+2014 = \xe2\x80\x94) and en-dash (U+2013 = \xe2\x80\x93)
    (
        b"- [ ] BS.14 Non-functional validation \xe2\x80\x94 cross-platform, performance, Git-compat, data-fidelity tests (Tasks 29.1\xe2\x80\x9329.4)",
        b"- [x] BS.14 Non-functional validation \xe2\x80\x94 cross-platform, performance, Git-compat, data-fidelity tests (Tasks 29.1\xe2\x80\x9329.4)",
    ),
    # Stream 1 line with plain hyphens
    (
        b"- [ ] BS.14 Non-functional validation -- cross-platform, performance, Git-compat, data-fidelity (Tasks 29.1-29.4)",
        b"- [x] BS.14 Non-functional validation -- cross-platform, performance, Git-compat, data-fidelity (Tasks 29.1-29.4)",
    ),
]

changed = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Replaced: {repr(old[:60])}")
        changed += 1
    else:
        log(f"NOT FOUND: {repr(old[:60])}")

if changed > 0:
    with open(path, "wb") as f:
        f.write(data)
    log(f"Written {changed} replacement(s).")
else:
    log("ERROR: no replacements made.")
    sys.exit(1)
