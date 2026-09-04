LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\bw_group1_patch.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\edit-operations\tasks.md"

with open(path, "rb") as f:
    raw = f.read()

log(f"File size: {len(raw)} bytes")

# Work on decoded text; preserve original bytes for write-back
text = raw.decode("utf-8")

# Target task numbers to mark complete (top-level and all sub-items)
# Strategy: scan line by line, track whether we are inside a target block
lines = text.splitlines(keepends=True)
new_lines = []
in_target = False
replacements = 0

TARGET_TASKS = {"28.", "29.", "30.", "31.", "32.", "33.", "35."}
END_TASKS    = {"34.", "36.", "37.", "38.", "39."}

for line in lines:
    stripped = line.lstrip()

    # Check for top-level task line (starts with "- [ ] N." or "- [x] N.")
    matched_start = False
    for num in TARGET_TASKS:
        if stripped.startswith(f"- [ ] {num}") or stripped.startswith(f"- [x] {num}"):
            in_target = True
            matched_start = True
            break

    if not matched_start:
        for num in END_TASKS:
            if stripped.startswith(f"- [ ] {num}") or stripped.startswith(f"- [x] {num}"):
                in_target = False
                break

    if in_target and "- [ ]" in line:
        new_line = line.replace("- [ ]", "- [x]", 1)
        replacements += 1
        new_lines.append(new_line)
    else:
        new_lines.append(line)

new_text = "".join(new_lines)
new_raw = new_text.encode("utf-8")

with open(path, "wb") as f:
    f.write(new_raw)

log(f"Replacements made: {replacements}")
log("Done")
