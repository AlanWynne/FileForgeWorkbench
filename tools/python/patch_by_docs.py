"""Phase BY documentation patch script.

Updates:
1. docs/specs/sequence-numbers/tasks.md  -- mark Tasks 20-22 [x]
2. docs/quality/TCR.md                   -- mark BY rows as covered
3. docs/specs/project-master/tasks.md   -- mark BY entries [x]
"""

import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

os.makedirs(os.path.dirname(LOG), exist_ok=True)
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")


def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


def patch_binary(path, replacements):
    """Apply list of (old_bytes, new_bytes) replacements to a file."""
    with open(path, "rb") as f:
        data = f.read()
    log(f"  File size: {len(data)} bytes")
    for old, new in replacements:
        for sep in (b"\r\n", b"\n"):
            old_s = old.replace(b"\n", sep)
            new_s = new.replace(b"\n", sep)
            if old_s in data:
                data = data.replace(old_s, new_s, 1)
                log(f"  Patched (sep={repr(sep)}): {old[:40]!r}...")
                break
        else:
            log(f"  WARNING: pattern not found: {old[:60]!r}")
    with open(path, "wb") as f:
        f.write(data)


# ── 1. sequence-numbers/tasks.md ────────────────────────────────────────────
log("=== Patching sequence-numbers/tasks.md ===")
tasks_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\sequence-numbers\tasks.md"

patch_binary(tasks_path, [
    (
        b"- [ ] 20. AUTONUM alias for NUMBER ON/OFF\n"
        b"  - [ ] 20.1 Extend `NumberCommand` argument parser to recognise `AUTONUM ON` and `AUTONUM OFF` as aliases for `NUMBER ON` and `NUMBER OFF`\n"
        b"  - [ ] 20.2 Write unit tests verifying `AUTONUM ON` produces identical state change to `NUMBER ON`, and `AUTONUM OFF` to `NUMBER OFF`",
        b"- [x] 20. AUTONUM alias for NUMBER ON/OFF\n"
        b"  - [x] 20.1 Extend `NumberCommand` argument parser to recognise `AUTONUM ON` and `AUTONUM OFF` as aliases for `NUMBER ON` and `NUMBER OFF`\n"
        b"  - [x] 20.2 Write unit tests verifying `AUTONUM ON` produces identical state change to `NUMBER ON`, and `AUTONUM OFF` to `NUMBER OFF`",
    ),
    (
        b"- [ ] 21. NUM alias for NUMBER command\n"
        b"  - [ ] 21.1 Register `NUM` as a command alias for `NUMBER` in the command framework (Command_ID: `sequence.number`, alias: `NUM`)\n"
        b"  - [ ] 21.2 Verify all NUMBER sub-commands (ON, OFF, SHOW, COLS, STD) are reachable via `NUM`\n"
        b"  - [ ] 21.3 Write unit tests verifying `NUM ON`, `NUM OFF`, `NUM SHOW`, `NUM COLS`, `NUM STD` each dispatch to the same handler as the equivalent `NUMBER` form",
        b"- [x] 21. NUM alias for NUMBER command\n"
        b"  - [x] 21.1 Register `NUM` as a command alias for `NUMBER` in the command framework (Command_ID: `sequence.number`, alias: `NUM`)\n"
        b"  - [x] 21.2 Verify all NUMBER sub-commands (ON, OFF, SHOW, COLS, STD) are reachable via `NUM`\n"
        b"  - [x] 21.3 Write unit tests verifying `NUM ON`, `NUM OFF`, `NUM SHOW`, `NUM COLS`, `NUM STD` each dispatch to the same handler as the equivalent `NUMBER` form",
    ),
    (
        b"- [ ] 22. TCR update for BY alias criteria\n"
        b"  - [ ] 22.1 Update docs/quality/TCR.md -- mark AUTONUM and NUM alias rows as covered once tests pass",
        b"- [x] 22. TCR update for BY alias criteria\n"
        b"  - [x] 22.1 Update docs/quality/TCR.md -- mark AUTONUM and NUM alias rows as covered once tests pass",
    ),
])

# ── 2. TCR.md ────────────────────────────────────────────────────────────────
log("=== Patching TCR.md ===")
tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

patch_binary(tcr_path, [
    (
        b"| `ff-sequence-numbers` | \xf0\x9f\x94\xb4 | -- | Req 6.7a: AUTONUM ON/OFF treated as alias for NUMBER ON/OFF |",
        b"| `ff-sequence-numbers` | \xe2\x9c\x85 | `number_cmd.rs` unit tests | Req 6.7a: AUTONUM ON/OFF treated as alias for NUMBER ON/OFF |",
    ),
    (
        b"| `ff-sequence-numbers` | \xf0\x9f\x94\xb4 | -- | Req 8 alias: NUM accepted as alias for NUMBER command with all sub-commands |",
        b"| `ff-sequence-numbers` | \xe2\x9c\x85 | `number_cmd.rs` + `commands.rs` unit tests | Req 8 alias: NUM accepted as alias for NUMBER command with all sub-commands |",
    ),
])

# ── 3. project-master/tasks.md ───────────────────────────────────────────────
log("=== Patching project-master/tasks.md ===")
master_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(master_path, "rb") as f:
    master_data = f.read()
log(f"  File size: {len(master_data)} bytes")

# Find and replace BY section entries
by_patterns = [
    (b"- [ ] BY.1", b"- [x] BY.1"),
    (b"- [ ] BY.2", b"- [x] BY.2"),
    (b"- [ ] BY.3", b"- [x] BY.3"),
    (b"- [ ] BY.impl", b"- [x] BY.impl"),
]
for old, new in by_patterns:
    if old in master_data:
        master_data = master_data.replace(old, new, 1)
        log(f"  Patched: {old!r}")
    else:
        log(f"  Not found (may already be done or not present): {old!r}")

with open(master_path, "wb") as f:
    f.write(master_data)

log("=== Done ===")
