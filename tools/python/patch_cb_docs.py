"""patch_cb_docs.py -- update docs for Phase CB (command-semantics EARS integration Req 9)."""

import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\patch_cb_docs.txt"
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
                log(f"    REPLACED: {old_str[:60]!r}")
                count += 1
                break
        else:
            log(f"    NOT FOUND: {old_str[:60]!r}")
    with open(path, "wb") as f:
        f.write(data)
    log(f"  Done: {count}/{len(replacements)} replacements")
    return count


# ── 1. command-semantics/tasks.md -- mark tasks 19-24 done ──────────────────

tasks_path = os.path.join(ROOT, "docs", "specs", "command-semantics", "tasks.md")
log("Patching command-semantics/tasks.md ...")

task_pairs = [
    b"- [ ] 19. TSO dataset management commands",
    b"- [ ] 19.1 Register `ALLOCATE` command routing",
    b"- [ ] 19.2 Register `FREE` command routing",
    b"- [ ] 19.3 Register `DELETE` command routing",
    b"- [ ] 19.4 Register `RENAME oldname newname` command routing",
    b"- [ ] 19.5 Register `LISTCAT [pattern]` command routing",
    b"- [ ] 19.6 Register `LISTDS dsname [MEMBERS]` command routing",
    b"- [ ] 19.7 Register `LISTALC` command routing",
    b"- [ ] 19.8 Write unit tests for each command registration",
    b"- [ ] 20. TSO job commands (SUBMIT, STATUS) and EDIT routing extension",
    b"- [ ] 20.1 Register `SUBMIT dsname` command routing",
    b"- [ ] 20.2 Register `STATUS [jobname]` command routing",
    b"- [ ] 20.3 Extend `EDIT` command handler",
    b"- [ ] 20.4 Write unit tests for SUBMIT routing",
    b"- [ ] 21. TSO-style operand parsing and session prefix",
    b"- [ ] 21.1 Implement TSO-style operand parser",
    b"- [ ] 21.2 Implement `SET PREFIX dsn-prefix` command",
    b"- [ ] 21.3 Implement automatic prefix qualification",
    b"- [ ] 21.4 Write unit tests for positional operands",
    b"- [ ] 22. Command continuation, ds:// URI, and namespace conflict resolution",
    b"- [ ] 22.1 Implement trailing backslash continuation",
    b"- [ ] 22.2 Implement `ds://` URI scheme recognition",
    b"- [ ] 22.3 Implement namespace conflict resolution",
    b"- [ ] 22.4 Write unit tests for continuation accumulation",
    b"- [ ] 23. Capability model, secret operands, and audit events",
    b"- [ ] 23.1 Implement capability declaration on command registration",
    b"- [ ] 23.2 Implement capability verification on invocation",
    b"- [ ] 23.3 Implement secret operand declaration and redaction",
    b"- [ ] 23.4 Implement structured audit event emission",
    b"- [ ] 23.5 Write unit tests for capability check pass/fail",
    b"- [ ] 24. TCR update for Requirement 9",
    b"- [ ] 24.1 Update docs/quality/TCR.md",
]

replacements = [(old, old.replace(b"- [ ]", b"- [x]")) for old in task_pairs]
patch_file(tasks_path, replacements)

# ── 2. TCR.md -- mark Phase CB rows PASS ────────────────────────────────────

tcr_path = os.path.join(ROOT, "docs", "quality", "TCR.md")
log("Patching TCR.md ...")

RED = "\U0001f534".encode("utf-8")
GREEN = "\u2705".encode("utf-8")
TEST_FILE = b"`tso.rs` unit tests"

tcr_rows = [
    (b"Req 9.1: ALLOCATE command routes to dataset allocator with TSO keyword operands",),
    (b"Req 9.2: FREE command routes to dataset allocator",),
    (b"Req 9.3: DELETE command routes to VFS/catalog layer",),
    (b"Req 9.4: RENAME oldname newname routes to VFS/catalog layer",),
    (b"Req 9.5: LISTCAT [pattern] routes to catalog registry",),
    (b"Req 9.6: LISTDS dsname [MEMBERS] routes to VFS layer",),
    (b"Req 9.7: LISTALC routes to dataset allocator",),
    (b"Req 9.8: SUBMIT dsname routes to FFW-JES subsystem",),
    (b"Req 9.9: STATUS [jobname] routes to FFW-JES job status panel",),
    (b"Req 9.10: EDIT dsname routes to file-operations pipeline",),
    (b"Req 9.11: TSO-style positional and keyword operand parsing",),
    (b"Req 9.12: SET PREFIX and automatic dataset name qualification",),
    (b"Req 9.13: command continuation via trailing backslash",),
    (b"Req 9.14: ds:// URI scheme bypasses session prefix, routes to VFS",),
    (b"Req 9.15: namespace conflict resolution built-in > plugin > macro",),
    (b"Req 9.16: capability model -- commands declare and verify required capabilities",),
    (b"Req 9.17: secret operand redaction from history, logs, and status messages",),
    (b"Req 9.18: structured audit events on every command execution",),
]

tcr_replacements = []
for (desc,) in tcr_rows:
    old = RED + b" | -- | " + desc + b" |"
    new = GREEN + b" | " + TEST_FILE + b" | " + desc + b" |"
    tcr_replacements.append((old, new))

patch_file(tcr_path, tcr_replacements)

# ── 3. project-master/tasks.md -- add Phase CB entry ────────────────────────

master_path = os.path.join(ROOT, "docs", "specs", "project-master", "tasks.md")
log("Patching project-master/tasks.md ...")

with open(master_path, "rb") as f:
    master_data = f.read()

cb_marker = b"CB.1 command-semantics EARS integration"
if cb_marker not in master_data:
    ca_marker = b"CA.1 startup-and-session EARS integration"
    if ca_marker in master_data:
        idx = master_data.rfind(ca_marker)
        end = master_data.find(b"\n", idx)
        if end == -1:
            end = len(master_data)
        insert = b"\n- [x] CB.1 command-semantics EARS integration (Req 9: TSO commands, operand parsing, capabilities, audit)"
        master_data = master_data[:end + 1] + insert + master_data[end + 1:]
        with open(master_path, "wb") as f:
            f.write(master_data)
        log("  Appended CB.1 after CA entry")
    else:
        log("  WARNING: CA marker not found")
else:
    log("  CB entry already present")

log("All done.")
