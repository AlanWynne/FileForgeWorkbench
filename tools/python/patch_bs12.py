path = r"docs\specs\project-master\tasks.md"
with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()

changed = 0
for i, line in enumerate(lines):
    if "BS.12" in line and "[ ]" in line and "Master" in line:
        lines[i] = line.replace("[ ]", "[x]", 1)
        print(f"Line {i+1} patched: {lines[i].rstrip()}")
        changed += 1
    elif "BS.12-BS.15" in line and "17 deliverables" in line:
        lines[i] = line.replace("BS.12-BS.15", "BS.13-BS.15").replace("17 deliverables", "16 deliverables")
        print(f"Line {i+1} patched: {lines[i].rstrip()}")
        changed += 1
    elif "BS.12 (next in Stream 1)" in line:
        lines[i] = line.replace("BS.12 (next in Stream 1)", "BS.13 (next in Stream 1)")
        print(f"Line {i+1} patched: {lines[i].rstrip()}")
        changed += 1

if changed:
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(lines)
    print(f"Done: {changed} line(s) patched")
else:
    print("ERROR: no lines matched")
