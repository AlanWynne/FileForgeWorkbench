"""Patch BS.11 line 534 in project-master/tasks.md."""
path = r"c:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()

changed = 0
for i, line in enumerate(lines):
    if "BS.11" in line and "[ ]" in line and "Security hardening" in line and "param" in line.lower():
        lines[i] = line.replace("[ ]", "[x]", 1)
        print(f"Line {i+1} patched: {lines[i].rstrip()}")
        changed += 1

if changed:
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(lines)
    print(f"Done: {changed} line(s) patched")
else:
    print("ERROR: no lines matched")
