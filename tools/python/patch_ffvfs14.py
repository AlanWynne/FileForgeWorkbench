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
    (b"- [ ] 14. POSIX files as native objects (Req 10)",
     b"- [x] 14. POSIX files as native objects (Req 10)"),
    (b"  - [ ] 14.1 Implement `PosixNativeProvider`",
     b"  - [x] 14.1 Implement `PosixNativeProvider`"),
    (b"  - [ ] 14.2 Implement read/write delegation to native OS file I/O",
     b"  - [x] 14.2 Implement read/write delegation to native OS file I/O"),
    (b"  - [ ] 14.3 Implement directory listing via `std::fs::read_dir`",
     b"  - [x] 14.3 Implement directory listing via `std::fs::read_dir`"),
    (b"  - [ ] 14.4 Implement stat returning native file metadata (size, timestamps, permissions)",
     b"  - [x] 14.4 Implement stat returning native file metadata (size, timestamps, permissions)"),
    (b"  - [ ] 14.5 Implement path-safety guard -- reject traversal outside catalog root",
     b"  - [x] 14.5 Implement path-safety guard -- reject traversal outside catalog root"),
    (b"  - [ ] 14.6 Write unit tests for all POSIX provider operations using `tempfile::TempDir`",
     b"  - [x] 14.6 Write unit tests for all POSIX provider operations using `tempfile::TempDir`"),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
        log(f"Replaced: {old[:70].decode()}")
    else:
        log(f"NOT FOUND: {old[:70].decode()}")

with open(path, "wb") as f:
    f.write(data)

log(f"Done. {count} replacements made.")
