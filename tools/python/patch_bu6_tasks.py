LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

old = b"- [ ] BU.6 File Explorer Panel Mainframe content reads from SQLite (Task 24)"
new = b"- [x] BU.6 File Explorer Panel Mainframe content reads from SQLite (Task 24)"

count = data.count(old)
log(f"Occurrences: {count}")
data = data.replace(old, new)

with open(path, "wb") as f:
    f.write(data)

log(f"Replaced {count} occurrence(s). Done.")
