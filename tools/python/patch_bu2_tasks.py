import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "w", encoding="utf-8") as f:
        f.write(msg + "\n")

def log_append(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\virtual-catalog-manager\tasks.md"
with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# Detect line ending
sep = b"\r\n" if b"\r\n" in data else b"\n"
log_append(f"Line ending: {repr(sep)}")

# Show lines around task 18 to confirm exact content
lines = data.split(sep)
for i, line in enumerate(lines):
    if b"18.1" in line or b"18.2" in line or b"19.1" in line or b"20.1" in line:
        log_append(f"Line {i}: {repr(line[:80])}")

replacements = [
    (b"  - [ ] 18.1", b"  - [x] 18.1"),
    (b"  - [ ] 18.2", b"  - [x] 18.2"),
    (b"  - [ ] 18.3", b"  - [x] 18.3"),
    (b"  - [ ] 18.4", b"  - [x] 18.4"),
    (b"  - [ ] 18.5 Run `cargo test -p ff-desktop` -- confirm new tests FAIL (red).",
     b"  - [x] 18.5 Run `cargo test -p ff-desktop` -- confirm new tests FAIL (red)."),
    (b"  - [ ] 19.1", b"  - [x] 19.1"),
    (b"  - [ ] 19.2", b"  - [x] 19.2"),
    (b"  - [ ] 19.3", b"  - [x] 19.3"),
    (b"  - [ ] 19.4 Run `cargo test -p ff-desktop` -- confirm new tests FAIL (red).",
     b"  - [x] 19.4 Run `cargo test -p ff-desktop` -- confirm new tests FAIL (red)."),
    (b"  - [ ] 20.1", b"  - [x] 20.1"),
    (b"  - [ ] 20.3", b"  - [x] 20.3"),
    (b"  - [ ] 20.4 Run `cargo test -p ff-desktop` -- confirm new tests FAIL (red).",
     b"  - [x] 20.4 Run `cargo test -p ff-desktop` -- confirm new tests FAIL (red)."),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
        log_append(f"OK: {old[:50]}")
    else:
        log_append(f"MISS: {old[:50]}")

with open(path, "wb") as f:
    f.write(data)

log_append(f"Done. {count}/{len(replacements)} replacements made.")
