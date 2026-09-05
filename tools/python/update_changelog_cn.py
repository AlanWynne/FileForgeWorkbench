import sys, shutil, os, tempfile

LOG = r"c:\workspace\VSC\FileForgeWorkbench\tools\logs\update_changelog_cn.txt"
TARGET = r"c:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(LOG, "w", encoding="utf-8") as f:
    f.write("=== update_changelog_cn.py ===\n")

with open(TARGET, "rb") as f:
    data = f.read()
log(f"File size: {len(data)} bytes")

# Detect line ending
sep = b"\r\n" if b"\r\n" in data else b"\n"
log(f"Line ending: {repr(sep)}")

# The CR-NR-034 section was appended with LF-only -- try both separators
OLD_STATUS = None
for try_sep in (b"\r\n", b"\n"):
    candidate = b"- **Status**: IN PROGRESS" + try_sep + b"- **Linked spec**: `docs/specs/caret-and-selection/requirements.md` (new Requirement 13), `docs/specs/clipboard-operations/requirements.md` (new Requirement 20)"
    if candidate in data:
        OLD_STATUS = candidate
        log(f"Pattern found with sep {repr(try_sep)}")
        break
if OLD_STATUS is None:
    log("ERROR: pattern not found with either separator")
    sys.exit(1)

NEW_STATUS = (
    b"- **Status**: DONE -- Phase CM complete (CM.1 editor drag-select + Ctrl+C, CM.2 selectable labels in POM/Settings/status bar)" + sep +
    b"- **Linked spec**: `docs/specs/caret-and-selection/requirements.md` (new Requirement 13), `docs/specs/clipboard-operations/requirements.md` (new Requirement 20)" + sep +
    sep +
    b"### CR-NR-035 -- Editor SCROLL field wired to editor Page Up/Down behaviour" + sep +
    b"- **Date/Phase**: Phase CN (pre-gate)" + sep +
    b"- **Prompt**: \"The editor window in ispf looks something like: [diagram showing SCROLL ===> CSR right-aligned on the command line]. Scroll can be set to CSR or PAGE or a numeric value. This controls how paging in the editor works. FFWB does not have this? We need to add this\"" + sep +
    b"- **Description**: The SCROLL ===> field already exists in the shell command area (Phase BZ, Req 19.1-19.3) and persists a ScrollAmount value. However the editor panel Page Up/Down keys currently always scroll by a fixed visible_count (one full page). The editor must read the active ScrollAmount and apply it: PAGE = full visible_count, HALF = visible_count/2, CSR = scroll to cursor line, a numeric value N = scroll exactly N lines. The SCROLL field must also be visible and editable when an editor tab is active." + sep +
    b"- **Status**: PENDING GATE" + sep +
    b"- **Linked spec**: `docs/specs/viewport-and-scrolling/requirements.md` (new criterion), `docs/specs/menu-and-statusbar/requirements.md` Req 19.1-19.3 (field display already covered)"
)

if OLD_STATUS in data:
    log("Pattern found -- replacing")
    data = data.replace(OLD_STATUS, NEW_STATUS, 1)
    tmp = TARGET + ".tmp"
    with open(tmp, "wb") as f:
        f.write(data)
    shutil.move(tmp, TARGET)
    log(f"Done. New size: {len(data)} bytes")
else:
    log("ERROR: pattern not found -- no change made")
    # Show context around CR-NR-034
    idx = data.find(b"CR-NR-034")
    if idx >= 0:
        log("CR-NR-034 context: " + repr(data[idx:idx+400]))
