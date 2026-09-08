import sys

LOG = r"c:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
open(LOG, "w").close()

def log(m):
    print(m, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(m + "\n")

path = r"c:\workspace\VSC\FileForgeWorkbench\docs\specs\configuration-system\tasks.md"
with open(path, "rb") as f:
    data = f.read()
log(f"File size before: {len(data)}")

# Detect line ending
sep = b"\r\n" if b"\r\n" in data else b"\n"
log(f"Line ending: {repr(sep)}")

count = 0
# Replace all "- [ ] 30." and "- [ ] 31." subtask lines with "- [x]"
# We do a simple global replace of the task markers
lines = data.split(sep)
new_lines = []
in_task_30_31 = False
for line in lines:
    decoded = line.decode("utf-8", errors="replace")
    # Detect start of task 30 or 31
    if decoded.strip().startswith("- [ ] 30.") or decoded.strip().startswith("- [ ] 31."):
        in_task_30_31 = True
    # Detect end: next top-level task (32) or end of section
    if in_task_30_31 and decoded.strip().startswith("- [") and ("32." in decoded or "33." in decoded):
        in_task_30_31 = False
    if in_task_30_31 and "- [ ]" in decoded:
        new_line = line.replace(b"- [ ]", b"- [x]")
        if new_line != line:
            count += 1
        new_lines.append(new_line)
    else:
        new_lines.append(line)

new_data = sep.join(new_lines)
with open(path, "wb") as f:
    f.write(new_data)
log(f"File size after: {len(new_data)}")
log(f"Replaced {count} checkboxes")
log("Done")
