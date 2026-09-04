path = r"docs\specs\project-master\tasks.md"
with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()

target = "Tasks 27.127.4"
replacement = "Tasks 27.1-27.4"
for i, line in enumerate(lines):
    if target in line:
        lines[i] = line.replace(target, replacement)
        print(f"Line {i+1} fixed: {lines[i].rstrip()}")

with open(path, "w", encoding="utf-8") as f:
    f.writelines(lines)
print("Done")
