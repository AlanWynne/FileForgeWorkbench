import sys
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\read_lines.txt"
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\update.rs"
lines = open(path, encoding="utf-8").readlines()
total = len(lines)

def show(start, end, label):
    print(f"\n--- {label} (lines {start}-{end}) ---")
    for i, l in enumerate(lines[start-1:end], start):
        print(f"{i}: {l}", end="")

show(190, 215, "session restore block")
show(870, 900, "session save block")
print(f"\nTotal lines: {total}")
