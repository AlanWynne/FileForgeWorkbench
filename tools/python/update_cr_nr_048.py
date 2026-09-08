"""Update CR-NR-048 status from PENDING GATE to IN PROGRESS in change-log.md."""
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
    old = b"### CR-NR-048 -- POM Options Review and Logical Grouping" + sep + b"- **Date/Phase**: Phase CV (pre-gate, depends on CR-NR-045)" + sep + b"- **Prompt**: \"We need a full review of the current options in the POM... perhaps we need to create menu options for those that can be grouped.\"" + sep + b"- **Description**: Review the current 9 POM options (0-8) against all implemented functionality. Revise the option list with logical grouping, adding entries for JES (job monitor), Search (global search), and Batch (batch execution) which currently have no POM entry. Define the default `menus/pom.toml` content. Update `startup-and-session` Req 14.3 accordingly." + sep + b"- **Status**: PENDING GATE" + sep + b"- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Req 14.3 (revision)"
    if old in data:
        log(f"Found CR-NR-048 block with sep {repr(sep)}")
        new = b"### CR-NR-048 -- POM Options Review and Logical Grouping" + sep + b"- **Date/Phase**: Phase CV (pre-gate, depends on CR-NR-045)" + sep + b"- **Prompt**: \"We need a full review of the current options in the POM... perhaps we need to create menu options for those that can be grouped.\"" + sep + b"- **Description**: Review the current 9 POM options (0-8) against all implemented functionality. Revise the option list with logical grouping, adding entries for JES (job monitor), Search (global search), and Batch (batch execution) which currently have no POM entry. Define the default `menus/pom.toml` content. Update `startup-and-session` Req 14.3 accordingly." + sep + b"- **Status**: IN PROGRESS" + sep + b"- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Req 14.3 (revision), `docs/specs/menu-workspace/cv-requirements.md` (new)"
        data = data.replace(old, new, 1)
        with open(path, "wb") as f:
            f.write(data)
        log("CR-NR-048 status updated to IN PROGRESS")
        break
else:
    log("ERROR: CR-NR-048 block not found with either separator")
    idx = data.find(b"CR-NR-048")
    if idx >= 0:
        log(repr(data[idx:idx+400]))
    sys.exit(1)

log("Done")
