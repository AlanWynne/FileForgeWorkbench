"""Patch stale PENDING GATE statuses in change-log.md."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\patch_changelog_status.txt"
PATH = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

with open(PATH, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

replacements = [
    (
        b"- **Status**: PENDING GATE\r\n- **Linked spec**: `docs/specs/ears-integration/workflow.md` Phase EI-0",
        b"- **Status**: DONE -- EI-0 through EI-6 all complete; all 16 EI-5 batches executed as Phases BW-CI; FTSO resolved as extension to shell-command (no new sub-project); MiniX confirmed as internal architecture label only\r\n- **Linked spec**: `docs/specs/ears-integration/workflow.md` (all phases [x])"
    ),
    (
        b"- **Status**: PENDING GATE -- gate complete, awaiting implementation approval\r\n- **Linked spec**: `docs/specs/workspace-model/requirements.md` (new sub-project, Req 1-6)",
        b"- **Status**: DONE -- Phase BS-A complete\r\n- **Linked spec**: `docs/specs/workspace-model/requirements.md` (new sub-project, Req 1-6)"
    ),
    (
        b"- **Status**: PENDING GATE -- gate complete, awaiting implementation approval\r\n- **Linked spec**: `docs/specs/command-palette/requirements.md` (new sub-project, Req 1-5)",
        b"- **Status**: DONE -- Phase BS-B complete\r\n- **Linked spec**: `docs/specs/command-palette/requirements.md` (new sub-project, Req 1-5)"
    ),
    (
        b"- **Status**: PENDING GATE -- gate complete, awaiting implementation approval\r\n- **Linked spec**: `docs/specs/global-search/requirements.md` (new sub-project, Req 1-6)",
        b"- **Status**: DONE -- Phase BS-C complete\r\n- **Linked spec**: `docs/specs/global-search/requirements.md` (new sub-project, Req 1-6)"
    ),
]

for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        log(f"Replaced: {old[:60]!r}")
    else:
        log(f"NOT FOUND (trying LF): {old[:60]!r}")
        old_lf = old.replace(b"\r\n", b"\n")
        new_lf = new.replace(b"\r\n", b"\n")
        if old_lf in data:
            data = data.replace(old_lf, new_lf, 1)
            log(f"  Replaced with LF variant")
        else:
            log(f"  ERROR: pattern not found with either separator")

with open(PATH, "wb") as f:
    f.write(data)

log(f"File written. New size: {len(data)} bytes")
log("Done.")
