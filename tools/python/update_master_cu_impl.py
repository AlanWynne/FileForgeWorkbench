import sys

path = r"docs\specs\project-master\tasks.md"

with open(path, "rb") as f:
    data = f.read()

# Find the CU-impl section and add a completion entry
anchor = b"| `[x]` Phase CU complete | Menu Workspace Pattern -- spec only (CU.1-CU.6) |"
idx = data.find(anchor)
if idx < 0:
    sys.stdout.write("ERROR: CU anchor not found\n")
    sys.exit(1)

sep = b"\r\n" if b"\r\n" in data else b"\n"
insert = sep + b"| `[x]` Phase CU-impl complete | Menu Workspace Implementation -- TabKind, loader, hot-reload, render, dispatch, defaults (Tasks 1-8) |"

# Insert after the anchor line
end_of_line = data.find(sep, idx + len(anchor))
if end_of_line < 0:
    end_of_line = len(data)

data = data[:end_of_line] + insert + data[end_of_line:]

with open(path, "wb") as f:
    f.write(data)

sys.stdout.write("project-master updated with CU-impl complete\n")
