"""Patch project-master/tasks.md: mark BS.13 [x] and update summary table."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\patch_bs13.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

replacements = 0

# BS.13 line in Wave 4 section (around line 538 area)
for sep in (b"\r\n", b"\n"):
    old = b"- [ ] BS.13 Record-oriented editor integration" + sep
    new = b"- [x] BS.13 Record-oriented editor integration" + sep
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Replaced BS.13 Wave4 with sep {repr(sep)}")
        replacements += 1
        break

# BS.13 line in Stream 1 section
for sep in (b"\r\n", b"\n"):
    old = b"- [ ] BS.13 Record-oriented editor integration -- wire codecs into open/save path (Tasks 28.1" + sep
    new = b"- [x] BS.13 Record-oriented editor integration -- wire codecs into open/save path (Tasks 28.1" + sep
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Replaced BS.13 Stream1 with sep {repr(sep)}")
        replacements += 1
        break

# Also handle en-dash variant
for sep in (b"\r\n", b"\n"):
    old = b"- [ ] BS.13 Record-oriented editor integration \xe2\x80\x93 wire codecs into open/save path (Tasks 28.1" + sep
    new = b"- [x] BS.13 Record-oriented editor integration \xe2\x80\x93 wire codecs into open/save path (Tasks 28.1" + sep
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Replaced BS.13 Stream1 en-dash with sep {repr(sep)}")
        replacements += 1
        break

# Update summary table: BS.13-BS.15 (16 deliverables) -> BS.14-BS.15 (15 deliverables)
for sep in (b"\r\n", b"\n"):
    old = b"| `[ ]` Pending -- Stream 1 (dataset architecture) | BS.13-BS.15, ff-vfs.13-16, BU.2-BU.9 (16 deliverables) |" + sep
    new = b"| `[ ]` Pending -- Stream 1 (dataset architecture) | BS.14-BS.15, ff-vfs.13-16, BU.2-BU.9 (15 deliverables) |" + sep
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Updated summary table with sep {repr(sep)}")
        replacements += 1
        break

# Update active work line
for sep in (b"\r\n", b"\n"):
    old = b"| Active work | BS.13 (next in Stream 1) or BW (next in Stream 2) |" + sep
    new = b"| Active work | BS.14 (next in Stream 1) or BW (next in Stream 2) |" + sep
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Updated active work with sep {repr(sep)}")
        replacements += 1
        break

if replacements > 0:
    with open(path, "wb") as f:
        f.write(data)
    log(f"Written {replacements} replacement(s)")
else:
    log("ERROR: no replacements made")

log("Done")
