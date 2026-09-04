import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\virtual-file-system\tasks.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

replacements = [
    (b"- [ ] 15. VFS staged transaction protocol (Req 11)", b"- [x] 15. VFS staged transaction protocol (Req 11)"),
    (b"  - [ ] 15.1 Define `VfsTransaction` struct", b"  - [x] 15.1 Define `VfsTransaction` struct"),
    (b"  - [ ] 15.2 Implement two-phase commit", b"  - [x] 15.2 Implement two-phase commit"),
    (b"  - [ ] 15.3 Implement rollback", b"  - [x] 15.3 Implement rollback"),
    (b"  - [ ] 15.4 Implement transaction journal", b"  - [x] 15.4 Implement transaction journal"),
    (b"  - [ ] 15.5 Write unit and integration tests", b"  - [x] 15.5 Write unit and integration tests"),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
        log(f"Replaced: {old[:60]}")
    else:
        log(f"NOT FOUND: {old[:60]}")

with open(path, "wb") as f:
    f.write(data)

log(f"Done. {count} replacements made.")
