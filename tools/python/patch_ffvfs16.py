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
    (b"- [ ] 16. workspace.backup / restore / reconcile / diagnose (Req 12)",
     b"- [x] 16. workspace.backup / restore / reconcile / diagnose (Req 12)"),
    (b"  - [ ] 16.1 Implement `workspace.backup` VFS command",
     b"  - [x] 16.1 Implement `workspace.backup` VFS command"),
    (b"  - [ ] 16.2 Implement `workspace.restore` VFS command",
     b"  - [x] 16.2 Implement `workspace.restore` VFS command"),
    (b"  - [ ] 16.3 Implement `workspace.diagnose` VFS command",
     b"  - [x] 16.3 Implement `workspace.diagnose` VFS command"),
    (b"  - [ ] 16.4 Implement `workspace.reconcile` VFS command",
     b"  - [x] 16.4 Implement `workspace.reconcile` VFS command"),
    (b"  - [ ] 16.5 Write unit and integration tests for backup/restore round-trip",
     b"  - [x] 16.5 Write unit and integration tests for backup/restore round-trip"),
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
