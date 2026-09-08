import sys

path = r"docs\status\current-work.md"

with open(path, "rb") as f:
    data = f.read()

sep = b"\r\n" if b"\r\n" in data else b"\n"

old = b"| Phase CU -- Menu Workspace Pattern | NEXT | Spec only: menu-workspace sub-project requirements + design | [project-master tasks](../specs/project-master/tasks.md) |"
new = b"| Phase CU -- Menu Workspace Pattern | DONE | All 8 implementation tasks complete -- TabKind, loader, hot-reload, render, dispatch, defaults | [project-master tasks](../specs/project-master/tasks.md) |"

if old in data:
    data = data.replace(old, new)
    with open(path, "wb") as f:
        f.write(data)
    sys.stdout.write("current-work.md updated\n")
else:
    sys.stdout.write("Pattern not found\n")
