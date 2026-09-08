import sys

LOG = r"c:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
open(LOG, "w").close()

def log(m):
    print(m, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(m + "\n")

path = r"c:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(path, "rb") as f:
    data = f.read()
log(f"TCR size before: {len(data)}")

replacements = [
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 16.1: WHEN any config key effective value changes, THE system SHALL append AuditEntry with timestamp, key, old/new value, layer, actor |",
        b"| `ff-config` | \xe2\x9c\x85 | `audit.rs` unit tests | Req 16.1: WHEN any config key effective value changes, THE system SHALL append AuditEntry with timestamp, key, old/new value, layer, actor |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 16.2: audit log persisted to rolling file at <user-config-dir>/audit.log; max 10,000 entries |",
        b"| `ff-config` | \xe2\x9c\x85 | `audit.rs` unit tests | Req 16.2: audit log persisted to rolling file at <user-config-dir>/audit.log; max 10,000 entries |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 16.3: query_audit_log(filter) API supports filtering by key prefix, layer, time range, actor |",
        b"| `ff-config` | \xe2\x9c\x85 | `audit.rs` unit tests | Req 16.3: query_audit_log(filter) API supports filtering by key prefix, layer, time range, actor |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 16.4: audit log write failure emits WARN and does not prevent config change |",
        b"| `ff-config` | \xe2\x9c\x85 | `audit.rs` unit tests | Req 16.4: audit log write failure emits WARN and does not prevent config change |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 16.5: AuditEntry public type with timestamp, key, old_value, new_value, layer, actor fields |",
        b"| `ff-config` | \xe2\x9c\x85 | `audit.rs` unit tests | Req 16.5: AuditEntry public type with timestamp, key, old_value, new_value, layer, actor fields |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 16.6: clear_audit_log() truncates in-memory and on-disk audit log |",
        b"| `ff-config` | \xe2\x9c\x85 | `audit.rs` unit tests | Req 16.6: clear_audit_log() truncates in-memory and on-disk audit log |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.1: export_settings(scope, path) writes TOML file for specified ExportScope |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.1: export_settings(scope, path) writes TOML file for specified ExportScope |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.2: ExportScope enum with AllLayers, UserLayer, ProjectLayer variants |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.2: ExportScope enum with AllLayers, UserLayer, ProjectLayer variants |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.3: exported TOML includes [_export_meta] header with timestamp, version, scope |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.3: exported TOML includes [_export_meta] header with timestamp, version, scope |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.4: import_settings(path, target) merges exported values into target layer |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.4: import_settings(path, target) merges exported values into target layer |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.5: ImportTarget enum with UserLayer, ProjectLayer variants |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.5: ImportTarget enum with UserLayer, ProjectLayer variants |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.6: invalid values skipped and reported in ImportSummary; import does not fail entirely |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.6: invalid values skipped and reported in ImportSummary; import does not fail entirely |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.7: ImportSummary struct with imported_count, skipped_count, skipped_keys fields |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.7: ImportSummary struct with imported_count, skipped_count, skipped_keys fields |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.8: unreadable or invalid TOML import file returns ConfigError; no changes made |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.8: unreadable or invalid TOML import file returns ConfigError; no changes made |",
    ),
    (
        b"| `ff-config` | \xf0\x9f\x94\xb4 | -- | Req 17.9: successful import triggers hot-reload cycle; callbacks notified of changed keys |",
        b"| `ff-config` | \xe2\x9c\x85 | `export_import.rs` unit tests | Req 17.9: successful import triggers hot-reload cycle; callbacks notified of changed keys |",
    ),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
        log(f"Replaced: {old[30:80]}")
    else:
        log(f"NOT FOUND: {old[30:80]}")

with open(path, "wb") as f:
    f.write(data)
log(f"TCR size after: {len(data)}")
log(f"Updated {count} of {len(replacements)} rows")
log("Done")
