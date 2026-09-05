"""Update task markers and TCR rows for CI.impl (Requirement 10)."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\fix_ci_impl.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

log("fix_ci_impl.py started")

# ── 1. command-semantics/tasks.md ─────────────────────────────────────────────
tasks_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\command-semantics\tasks.md"
with open(tasks_path, "rb") as f:
    data = f.read()
log(f"tasks.md size: {len(data)} bytes")

replacements = [
    (b"- [ ] 25. TSO P2 output and job management commands (OUTPUT, CANCEL)",
     b"- [x] 25. TSO P2 output and job management commands (OUTPUT, CANCEL)"),
    (b"  - [ ] 25.1 Register `OUTPUT jobname [options]`",
     b"  - [x] 25.1 Register `OUTPUT jobname [options]`"),
    (b"  - [ ] 25.2 Register `CANCEL jobname [PURGE]`",
     b"  - [x] 25.2 Register `CANCEL jobname [PURGE]`"),
    (b"  - [ ] 25.3 Write unit tests for OUTPUT routing, CANCEL with/without PURGE",
     b"  - [x] 25.3 Write unit tests for OUTPUT routing, CANCEL with/without PURGE"),
    (b"- [ ] 26. TSO P2 communication and profile commands (SEND, PROFILE, PRINTDS)",
     b"- [x] 26. TSO P2 communication and profile commands (SEND, PROFILE, PRINTDS)"),
    (b"  - [ ] 26.1 Register `SEND 'message'",
     b"  - [x] 26.1 Register `SEND 'message'"),
    (b"  - [ ] 26.2 Register `PROFILE [operands]`",
     b"  - [x] 26.2 Register `PROFILE [operands]`"),
    (b"  - [ ] 26.3 Register `PRINTDS DATASET(dsname)",
     b"  - [x] 26.3 Register `PRINTDS DATASET(dsname)"),
    (b"  - [ ] 26.4 Write unit tests for SEND routing variants",
     b"  - [x] 26.4 Write unit tests for SEND routing variants"),
    (b"- [ ] 27. TCR update for Requirement 10",
     b"- [x] 27. TCR update for Requirement 10"),
    (b"  - [ ] 27.1 Update docs/quality/TCR.md",
     b"  - [x] 27.1 Update docs/quality/TCR.md"),
]

count = 0
for old, new in replacements:
    for sep in (b"\r\n", b"\n"):
        old_s = old.replace(b"\n", sep)
        new_s = new.replace(b"\n", sep)
        if old_s in data:
            data = data.replace(old_s, new_s, 1)
            count += 1
            break
        # also try without sep (single-line match)
    else:
        if old in data:
            data = data.replace(old, new, 1)
            count += 1
        else:
            log(f"  WARNING: pattern not found: {old[:60]}")

with open(tasks_path, "wb") as f:
    f.write(data)
log(f"tasks.md: {count}/{len(replacements)} replacements")

# ── 2. project-master/tasks.md ────────────────────────────────────────────────
master_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
with open(master_path, "rb") as f:
    mdata = f.read()
log(f"project-master/tasks.md size: {len(mdata)} bytes")

old_ci = b"- [ ] CI.impl"
new_ci = b"- [x] CI.impl"
if old_ci in mdata:
    mdata = mdata.replace(old_ci, new_ci, 1)
    log("project-master: CI.impl marked [x]")
else:
    log("WARNING: CI.impl not found in project-master/tasks.md")

with open(master_path, "wb") as f:
    f.write(mdata)

# ── 3. TCR.md ─────────────────────────────────────────────────────────────────
tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(tcr_path, "rb") as f:
    tdata = f.read()
log(f"TCR.md size: {len(tdata)} bytes")

tcr_replacements = [
    (b"| `ff-command-semantics` | \xf0\x9f\x94\xb4 | -- | Req 10.1: OUTPUT jobname routes to FFW-JES for job output display/retrieval |",
     b"| `ff-command-semantics` | \xe2\x9c\x85 | `tso.rs` unit tests | Req 10.1: OUTPUT jobname routes to FFW-JES for job output display/retrieval |"),
    (b"| `ff-command-semantics` | \xf0\x9f\x94\xb4 | -- | Req 10.2: CANCEL jobname [PURGE] routes to FFW-JES; PURGE requests output purge |",
     b"| `ff-command-semantics` | \xe2\x9c\x85 | `tso.rs` unit tests | Req 10.2: CANCEL jobname [PURGE] routes to FFW-JES; PURGE requests output purge |"),
    (b"| `ff-command-semantics` | \xf0\x9f\x94\xb4 | -- | Req 10.3: SEND 'message' [USER/LOGON/BROADCAST] routes to messaging subsystem |",
     b"| `ff-command-semantics` | \xe2\x9c\x85 | `tso.rs` unit tests | Req 10.3: SEND 'message' [USER/LOGON/BROADCAST] routes to messaging subsystem |"),
    (b"| `ff-command-semantics` | \xf0\x9f\x94\xb4 | -- | Req 10.4: PROFILE [operands] routes to session profile subsystem |",
     b"| `ff-command-semantics` | \xe2\x9c\x85 | `tso.rs` unit tests | Req 10.4: PROFILE [operands] routes to session profile subsystem |"),
    (b"| `ff-command-semantics` | \xf0\x9f\x94\xb4 | -- | Req 10.5: PRINTDS DATASET(dsname) routes to file-operations pipeline |",
     b"| `ff-command-semantics` | \xe2\x9c\x85 | `tso.rs` unit tests | Req 10.5: PRINTDS DATASET(dsname) routes to file-operations pipeline |"),
]

tcr_count = 0
for old, new in tcr_replacements:
    if old in tdata:
        tdata = tdata.replace(old, new, 1)
        tcr_count += 1
    else:
        log(f"  TCR WARNING: pattern not found (trying ASCII fallback): {old[:60]}")

with open(tcr_path, "wb") as f:
    f.write(tdata)
log(f"TCR.md: {tcr_count}/{len(tcr_replacements)} replacements")

log("Done.")
