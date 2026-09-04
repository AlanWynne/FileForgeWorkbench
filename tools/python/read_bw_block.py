"""Read Phase BW block from project-master/tasks.md."""
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
open(LOG, "w").close()

with open(r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md", "rb") as f:
    data = f.read()

idx = data.find(b"Phase BW")
if idx == -1:
    with open(LOG, "ab") as f:
        f.write(b"Phase BW not found\n")
else:
    with open(LOG, "ab") as f:
        f.write(b"=== Phase BW block ===\n")
        f.write(data[idx:idx+800])
        f.write(b"\n=== end ===\n")
