LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\ct_changelog_fix.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "w", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

old = b"- **Status**: IN PROGRESS\r\n\r\n### CR-NR-044"
new = b"- **Status**: DONE -- Phase CT complete, all 7 tasks done, ~200 terminology replacements across 69 sub-project specs\r\n\r\n### CR-NR-044"
if old in data:
    data = data.replace(old, new)
    log("Replaced CR-CH-009 status (CRLF)")
else:
    old2 = b"- **Status**: IN PROGRESS\n\n### CR-NR-044"
    new2 = b"- **Status**: DONE -- Phase CT complete, all 7 tasks done, ~200 terminology replacements across 69 sub-project specs\n\n### CR-NR-044"
    if old2 in data:
        data = data.replace(old2, new2)
        log("Replaced CR-CH-009 status (LF)")
    else:
        log("ERROR: pattern not found")

with open(path, "wb") as f:
    f.write(data)

log("Done.")
