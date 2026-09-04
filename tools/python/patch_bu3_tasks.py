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

replacements = [
    (b"- [ ] 18. CatalogRegistry API extension tests",
     b"- [x] 18. CatalogRegistry API extension tests"),
    (b"- [ ] 19. resolve_and_open_dataset tests",
     b"- [x] 19. resolve_and_open_dataset tests"),
    (b"- [ ] 20. Content area population tests",
     b"- [x] 20. Content area population tests"),
    (b"- [ ] 21. Add CatalogRegistry::allocate() and list_datasets() methods",
     b"- [x] 21. Add CatalogRegistry::allocate() and list_datasets() methods"),
    (b"  - [ ] 21.1 In `catalog_registry.rs`: add `allocate(catalog_name: &str, params: AllocParams)",
     b"  - [x] 21.1 In `catalog_registry.rs`: add `allocate(catalog_name: &str, params: AllocParams)"),
    (b"  - [ ] 21.2 In `catalog_registry.rs`: add `list_datasets(catalog_name: &str)",
     b"  - [x] 21.2 In `catalog_registry.rs`: add `list_datasets(catalog_name: &str)"),
    (b"  - [ ] 21.3 Run `cargo test -p ff-desktop` -- confirm tasks 18.1-18.4 now PASS (green).",
     b"  - [x] 21.3 Run `cargo test -p ff-desktop` -- confirm tasks 18.1-18.4 now PASS (green)."),
    (b"  - [ ] 21.4 Run `cargo clippy -p ff-desktop -- -D warnings` -- clean.",
     b"  - [x] 21.4 Run `cargo clippy -p ff-desktop -- -D warnings` -- clean."),
    (b"  - [ ] 23.1 In `files_panel.rs`: replace `load_entries_from_datasets(catalog_name)` with",
     b"  - [x] 23.1 In `files_panel.rs`: replace `load_entries_from_datasets(catalog_name)` with"),
    (b"  - [ ] 23.2 Run `cargo test -p ff-desktop` -- tasks 20.1 and 20.3 now PASS.",
     b"  - [x] 23.2 Run `cargo test -p ff-desktop` -- tasks 20.1 and 20.3 now PASS."),
    (b"  - [ ] 25.1 In `files_panel.rs`: add `resolve_and_open_dataset(registry, dsn)` function",
     b"  - [x] 25.1 In `files_panel.rs`: add `resolve_and_open_dataset(registry, dsn)` function"),
    (b"  - [ ] 25.4 Run `cargo test -p ff-desktop` -- tasks 19.1-19.3 now PASS.",
     b"  - [x] 25.4 Run `cargo test -p ff-desktop` -- tasks 19.1-19.3 now PASS."),
    (b"  - [ ] 25.5 Run `cargo clippy -p ff-desktop -- -D warnings` -- clean.",
     b"  - [x] 25.5 Run `cargo clippy -p ff-desktop -- -D warnings` -- clean."),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
        log_append(f"OK: {old[:60]}")
    else:
        log_append(f"MISS: {old[:60]}")

with open(path, "wb") as f:
    f.write(data)

log_append(f"Done. {count}/{len(replacements)} replacements.")
