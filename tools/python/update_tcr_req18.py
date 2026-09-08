"""Update TCR Req 18 rows from NOT COVERED to PASS (Task 32.11)."""
import sys

TARGET = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

with open(TARGET, "rb") as f:
    data = f.read()

print(f"File size: {len(data)} bytes", flush=True)

replacements = [
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 18.1: system-layer [_locked].locked_keys list parsed into locked key set |",
        b"| `ff-config` | \xe2\x9c\x85 | `merger.rs`, `config_handle.rs` unit tests | Req 18.1: system-layer [_locked].locked_keys list parsed into locked key set |"
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 18.2: locked key uses system-layer value regardless of higher-priority layer definitions |",
        b"| `ff-config` | \xe2\x9c\x85 | `merger.rs` unit tests | Req 18.2: locked key uses system-layer value regardless of higher-priority layer definitions |"
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 18.3: set_user_value() on locked key returns ConfigError::KeyLocked |",
        b"| `ff-config` | \xe2\x9c\x85 | `config_handle.rs` unit tests | Req 18.3: set_user_value() on locked key returns ConfigError::KeyLocked |"
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 18.4: higher-priority layer value for locked key silently ignored; DEBUG log emitted |",
        b"| `ff-config` | \xe2\x9c\x85 | `merger.rs` unit tests | Req 18.4: higher-priority layer value for locked key silently ignored; DEBUG log emitted |"
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 18.5: is_locked(key) -> bool method on ConfigHandle |",
        b"| `ff-config` | \xe2\x9c\x85 | `config_handle.rs` unit tests | Req 18.5: is_locked(key) -> bool method on ConfigHandle |"
    ),
    (
        b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 18.6: Settings panel shows LOCKED badge and disables widget + Reset button for locked keys |",
        b"| `ff-desktop` | \xf0\x9f\x94\xb2 | -- | Req 18.6: Settings panel shows LOCKED badge and disables widget + Reset button for locked keys |"
    ),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
        print(f"Replaced row {count}", flush=True)
    else:
        print(f"WARNING: row not found: {old[:80]}", flush=True)

with open(TARGET, "wb") as f:
    f.write(data)

print(f"Done. {count} rows updated.", flush=True)
