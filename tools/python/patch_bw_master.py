LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\bw_master_patch.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(path, "rb") as f:
    raw = f.read()

log(f"File size: {len(raw)} bytes")
text = raw.decode("utf-8")

# Mark BW.1 through BW.6 and BW.8 as [x]
targets = ["BW.1 ", "BW.2 ", "BW.3 ", "BW.4 ", "BW.5 ", "BW.6 ", "BW.8 "]
replacements = 0

lines = text.splitlines(keepends=True)
new_lines = []
for line in lines:
    new_line = line
    for t in targets:
        if f"- [ ] {t}" in line:
            new_line = line.replace(f"- [ ] {t}", f"- [x] {t}", 1)
            replacements += 1
            break
    new_lines.append(new_line)

new_text = "".join(new_lines)
with open(path, "wb") as f:
    f.write(new_text.encode("utf-8"))

log(f"Replacements made: {replacements}")
log("Done")
