LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\update_changelog.txt"
with open(LOG, "w", encoding="utf-8") as f:
    f.write("=== update_changelog.py ===\n\n")

def log(m):
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(m + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"
with open(path, "rb") as f:
    data = f.read()
log("File size: " + str(len(data)))

old = b"- **Status**: PENDING GATE\r\n- **Linked spec**: `docs/specs/accessibility/requirements.md`"
new = b"- **Status**: IN PROGRESS\r\n- **Linked spec**: `docs/specs/accessibility/requirements.md`"
if old in data:
    data = data.replace(old, new, 1)
    log("REPLACED: CR-NR-040 status PENDING GATE -> IN PROGRESS (CRLF)")
    with open(path, "wb") as f:
        f.write(data)
else:
    old2 = b"- **Status**: PENDING GATE\n- **Linked spec**: `docs/specs/accessibility/requirements.md`"
    new2 = b"- **Status**: IN PROGRESS\n- **Linked spec**: `docs/specs/accessibility/requirements.md`"
    if old2 in data:
        data = data.replace(old2, new2, 1)
        log("REPLACED: CR-NR-040 status PENDING GATE -> IN PROGRESS (LF)")
        with open(path, "wb") as f:
            f.write(data)
    else:
        log("NOT FOUND: pattern not matched -- check change-log.md manually")

log("Done.")
