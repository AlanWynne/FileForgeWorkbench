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
    (b"- [ ] 13. StorageProvider trait in ff-vfs (Req 9)",
     b"- [x] 13. StorageProvider trait in ff-vfs (Req 9)"),
    (b"  - [ ] 13.1 Define `StorageProvider` trait in `src/storage_provider.rs`",
     b"  - [x] 13.1 Define `StorageProvider` trait in `src/storage_provider.rs`"),
    (b"  - [ ] 13.2 Implement capability advertisement: each provider declares which operations it supports",
     b"  - [x] 13.2 Implement capability advertisement: each provider declares which operations it supports"),
    (b"  - [ ] 13.3 Implement `UnsupportedOperation` default returns for optional methods",
     b"  - [x] 13.3 Implement `UnsupportedOperation` default returns for optional methods"),
    (b"  - [ ] 13.4 Register `StorageProvider` with `ProviderRegistry` alongside `VfsProvider`",
     b"  - [x] 13.4 Register `StorageProvider` with `ProviderRegistry` alongside `VfsProvider`"),
    (b"  - [ ] 13.5 Write unit tests for trait object construction, capability advertisement, and unsupported-operation defaults",
     b"  - [x] 13.5 Write unit tests for trait object construction, capability advertisement, and unsupported-operation defaults"),
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
