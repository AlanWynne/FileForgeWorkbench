import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\find_pending.txt"

with open(LOG, "w", encoding="utf-8") as lf:
    lf.write("=== Pending task audit ===\n\n")

def log(msg):
    with open(LOG, "a", encoding="utf-8") as lf:
        lf.write(msg + "\n")

files = [
    r"docs\specs\compiler-toolchain-integration\tasks.md",
    r"docs\specs\viewport-and-scrolling\tasks.md",
    r"docs\specs\virtual-catalog-manager\tasks.md",
    r"docs\specs\project-master\tasks.md",
]

for f in files:
    log("=== " + f + " ===")
    try:
        with open(f, encoding="utf-8", errors="replace") as fh:
            lines = fh.readlines()
        log("  total lines: " + str(len(lines)))
        found = 0
        for i, line in enumerate(lines, 1):
            if "- [ ]" in line:
                log("  L" + str(i) + ": " + line.rstrip())
                found += 1
        if found == 0:
            log("  (none found)")
    except Exception as e:
        log("  ERROR: " + str(e))
    log("")

log("Done.")
