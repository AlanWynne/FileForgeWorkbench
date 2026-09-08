"""Update CR-NR-047 status from PENDING GATE to IN PROGRESS in change-log.md."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

for sep in (b"\r\n", b"\n"):
    old = (
        b"### CR-NR-047 -- Settings Context as a Menu Workspace" + sep +
        b"- **Date/Phase**: Phase CW (pre-gate, depends on CR-NR-045)" + sep +
        b"- **Prompt**: \"The settings workspace should probably be a menu options workspace. A full list of available settings should be extracted from the requirements and a settings menu option created for it.\"" + sep +
        b"- **Description**: Restructure the Settings Context as a Menu Workspace whose options are loaded from `menus/settings.toml`. A default settings.toml is written on first launch with one option per major config namespace (Editor, Theme, Catalogs, VFS, Logging, Key Maps, Session, Plugins). Each option opens a sub-context showing only that namespace's keys using the existing flat-list widget. The flat-list view remains accessible as a sub-context." + sep +
        b"- **Status**: PENDING GATE" + sep +
        b"- **Linked spec**: `docs/specs/configuration-system/requirements.md` Req 15 (revision)"
    )
    if old in data:
        log(f"Found CR-NR-047 block with sep {repr(sep)}")
        new = (
            b"### CR-NR-047 -- Settings Context as a Menu Workspace" + sep +
            b"- **Date/Phase**: Phase CW (pre-gate, depends on CR-NR-045)" + sep +
            b"- **Prompt**: \"The settings workspace should probably be a menu options workspace. A full list of available settings should be extracted from the requirements and a settings menu option created for it.\"" + sep +
            b"- **Description**: Restructure the Settings Context as a Menu Workspace whose options are loaded from `menus/settings.toml`. A default settings.toml is written on first launch with one option per major config namespace (Editor, Theme, Catalogs, VFS, Logging, Key Maps, Session, Plugins). Each option opens a sub-context showing only that namespace's keys using the existing flat-list widget. The flat-list view remains accessible as a sub-context." + sep +
            b"- **Status**: IN PROGRESS" + sep +
            b"- **Linked spec**: `docs/specs/configuration-system/requirements.md` Req 15 (revision), `docs/specs/menu-workspace/cw-requirements.md` (new)"
        )
        data = data.replace(old, new, 1)
        with open(path, "wb") as f:
            f.write(data)
        log("CR-NR-047 status updated to IN PROGRESS")
        break
else:
    log("ERROR: CR-NR-047 block not found")
    idx = data.find(b"CR-NR-047")
    if idx >= 0:
        log(repr(data[idx:idx+400]))
    sys.exit(1)

log("Done")
