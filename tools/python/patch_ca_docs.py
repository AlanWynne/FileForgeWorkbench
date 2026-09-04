"""patch_ca_docs.py -- update docs for Phase CA (startup-and-session EARS integration).

Updates:
  - docs/specs/startup-and-session/tasks.md  -- mark tasks 28-32 done
  - docs/quality/TCR.md                      -- mark Req 20.1-20.6 rows PASS
  - docs/specs/project-master/tasks.md       -- add Phase CA entry
"""

import os
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\patch_ca_docs.txt"
ROOT = r"C:\workspace\VSC\FileForgeWorkbench"

os.makedirs(os.path.dirname(LOG), exist_ok=True)
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")


def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


def patch_file(path, replacements):
    with open(path, "rb") as f:
        data = f.read()
    log(f"  File: {path} ({len(data)} bytes)")
    count = 0
    for old_str, new_str in replacements:
        for sep in (b"\r\n", b"\n"):
            old_b = old_str.replace(b"\n", sep)
            new_b = new_str.replace(b"\n", sep)
            if old_b in data:
                data = data.replace(old_b, new_b, 1)
                log(f"    REPLACED (sep={repr(sep)}): {old_str[:60]!r}")
                count += 1
                break
        else:
            log(f"    NOT FOUND: {old_str[:60]!r}")
    with open(path, "wb") as f:
        f.write(data)
    log(f"  Done: {count}/{len(replacements)} replacements")
    return count


# ── 1. startup-and-session/tasks.md -- mark tasks 28-32 done ──────────────

tasks_path = os.path.join(ROOT, "docs", "specs", "startup-and-session", "tasks.md")
log("Patching startup-and-session/tasks.md ...")

tasks_replacements = [
    (
        b"- [ ] 28. Session start timestamp in status bar",
        b"- [x] 28. Session start timestamp in status bar",
    ),
    (
        b"  - [ ] 28.1 Record session start time in `WorkbenchShell` on startup",
        b"  - [x] 28.1 Record session start time in `WorkbenchShell` on startup",
    ),
    (
        b"  - [ ] 28.2 Add `Started: HH:MM` segment to status bar, populated from session start time",
        b"  - [x] 28.2 Add `Started: HH:MM` segment to status bar, populated from session start time",
    ),
    (
        b"  - [ ] 28.3 Write unit tests for timestamp formatting and status bar segment presence",
        b"  - [x] 28.3 Write unit tests for timestamp formatting and status bar segment presence",
    ),
    (
        b"- [ ] 29. Session end timestamp and logoff message",
        b"- [x] 29. Session end timestamp and logoff message",
    ),
    (
        b"  - [ ] 29.1 On exit sequence initiation, compute session duration and format logoff message",
        b"  - [x] 29.1 On exit sequence initiation, compute session duration and format logoff message",
    ),
    (
        b"  - [ ] 29.2 Display `Logoff at HH:MM -- session duration: Xm Ys` in status area before window closes",
        b"  - [x] 29.2 Display `Logoff at HH:MM -- session duration: Xm Ys` in status area before window closes",
    ),
    (
        b"  - [ ] 29.3 Write unit tests for duration calculation and message formatting",
        b"  - [x] 29.3 Write unit tests for duration calculation and message formatting",
    ),
    (
        b"- [ ] 30. LOGOFF command",
        b"- [x] 30. LOGOFF command",
    ),
    (
        b"  - [ ] 30.1 Register `LOGOFF` as a command alias for the exit sequence in `handle_command()`",
        b"  - [x] 30.1 Register `LOGOFF` as a command alias for the exit sequence in `handle_command()`",
    ),
    (
        b"  - [ ] 30.2 Write unit test verifying `LOGOFF` triggers the same exit path as `EXIT` and `=X`",
        b"  - [x] 30.2 Write unit test verifying `LOGOFF` triggers the same exit path as `EXIT` and `=X`",
    ),
    (
        b"- [ ] 31. TIME command",
        b"- [x] 31. TIME command",
    ),
    (
        b"  - [ ] 31.1 Implement `TIME` command handler: format current date/time as `Date: YYYY-MM-DD  Time: HH:MM:SS  Day: DDD` and display in status/response area",
        b"  - [x] 31.1 Implement `TIME` command handler: format current date/time as `Date: YYYY-MM-DD  Time: HH:MM:SS  Day: DDD` and display in status/response area",
    ),
    (
        b"  - [ ] 31.2 Write unit tests for TIME output format and day-of-year calculation",
        b"  - [x] 31.2 Write unit tests for TIME output format and day-of-year calculation",
    ),
    (
        b"- [ ] 32. STATUS command routing to FFW-JES",
        b"- [x] 32. STATUS command routing to FFW-JES",
    ),
    (
        b"  - [ ] 32.1 Implement `STATUS` command handler: route to FFW-JES job status panel (transform current tab or open new tab)",
        b"  - [x] 32.1 Implement `STATUS` command handler: route to FFW-JES job status panel (transform current tab or open new tab)",
    ),
    (
        b"  - [ ] 32.2 Implement `STATUS jobname` variant: route to FFW-JES panel with jobname filter pre-populated",
        b"  - [x] 32.2 Implement `STATUS jobname` variant: route to FFW-JES panel with jobname filter pre-populated",
    ),
    (
        b"  - [ ] 32.3 Write unit tests for STATUS routing and STATUS with jobname argument",
        b"  - [x] 32.3 Write unit tests for STATUS routing and STATUS with jobname argument",
    ),
    (
        b"- [ ] 33. TCR update for Requirement 20",
        b"- [x] 33. TCR update for Requirement 20",
    ),
    (
        b"  - [ ] 33.1 Update docs/quality/TCR.md -- mark all Req 20.1-20.6 rows as covered once tests pass",
        b"  - [x] 33.1 Update docs/quality/TCR.md -- mark all Req 20.1-20.6 rows as covered once tests pass",
    ),
]

patch_file(tasks_path, tasks_replacements)

# ── 2. TCR.md -- mark Phase CA rows PASS ──────────────────────────────────

tcr_path = os.path.join(ROOT, "docs", "quality", "TCR.md")
log("Patching TCR.md ...")

# emoji bytes
RED = "\U0001f534".encode("utf-8")   # red circle
GREEN = "\u2705".encode("utf-8")     # green check

tcr_replacements = [
    (
        RED + b" | -- | Req 20.1: session start timestamp displayed in status bar as Started: HH:MM |",
        GREEN + b" | `shell/mod.rs`, `shell/tests.rs` | Req 20.1: session start timestamp displayed in status bar as Started: HH:MM |",
    ),
    (
        RED + b" | -- | Req 20.2: session end timestamp and duration shown in status area on exit |",
        GREEN + b" | `shell/mod.rs`, `shell/tests.rs` | Req 20.2: session end timestamp and duration shown in status area on exit |",
    ),
    (
        RED + b" | -- | Req 20.3: LOGOFF command initiates exit sequence identical to EXIT/=X |",
        GREEN + b" | `shell/commands.rs`, `shell/tests.rs` | Req 20.3: LOGOFF command initiates exit sequence identical to EXIT/=X |",
    ),
    (
        RED + b" | -- | Req 20.4: TIME command displays current date/time/day-of-year in response area |",
        GREEN + b" | `shell/commands.rs`, `shell/tests.rs` | Req 20.4: TIME command displays current date/time/day-of-year in response area |",
    ),
    (
        RED + b" | -- | Req 20.5: STATUS command routes to FFW-JES job status panel |",
        GREEN + b" | `shell/commands.rs`, `shell/tests.rs` | Req 20.5: STATUS command routes to FFW-JES job status panel |",
    ),
    (
        RED + b" | -- | Req 20.6: STATUS jobname routes to FFW-JES panel filtered by jobname |",
        GREEN + b" | `shell/commands.rs`, `shell/tests.rs` | Req 20.6: STATUS jobname routes to FFW-JES panel filtered by jobname |",
    ),
]

patch_file(tcr_path, tcr_replacements)

# ── 3. project-master/tasks.md -- add Phase CA entry ──────────────────────

master_path = os.path.join(ROOT, "docs", "specs", "project-master", "tasks.md")
log("Patching project-master/tasks.md ...")

master_replacements = [
    (
        b"- [ ] CA.1 startup-and-session EARS integration (Req 20: LOGOFF, TIME, STATUS, session timestamps)",
        b"- [x] CA.1 startup-and-session EARS integration (Req 20: LOGOFF, TIME, STATUS, session timestamps)",
    ),
]

# If the entry doesn't exist yet, append it
with open(master_path, "rb") as f:
    master_data = f.read()

ca_marker = b"CA.1 startup-and-session EARS integration"
if ca_marker not in master_data:
    log("  CA entry not found -- appending to project-master/tasks.md")
    # Find the BZ entry to insert after it
    bz_marker = b"- [x] BZ."
    if bz_marker in master_data:
        # Find end of BZ line
        idx = master_data.rfind(bz_marker)
        end = master_data.find(b"\n", idx)
        if end == -1:
            end = len(master_data)
        insert = b"\n- [x] CA.1 startup-and-session EARS integration (Req 20: LOGOFF, TIME, STATUS, session timestamps)"
        master_data = master_data[:end + 1] + insert + master_data[end + 1:]
        with open(master_path, "wb") as f:
            f.write(master_data)
        log("  Appended CA.1 after BZ entry")
    else:
        log("  WARNING: BZ marker not found -- appending at end")
        with open(master_path, "ab") as f:
            f.write(b"\n- [x] CA.1 startup-and-session EARS integration (Req 20: LOGOFF, TIME, STATUS, session timestamps)\n")
else:
    patch_file(master_path, master_replacements)

log("All done.")
