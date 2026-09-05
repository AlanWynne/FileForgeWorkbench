import sys
path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
log_path = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\search-out.txt"

data = open(path, "rb").read()

# Find the Stream 2 block
idx = data.find(b"Stream 2 -- EARS P1")
chunk = data[idx:idx+800]

with open(log_path, "w", encoding="utf-8") as f:
    f.write(repr(chunk))

sys.stderr.write("done\n")
