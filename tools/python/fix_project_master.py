import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def fix_file(path, replacements):
    data = open(path, "rb").read()
    count = 0
    for old, new in replacements:
        if old in data:
            data = data.replace(old, new, 1)
            count += 1
        else:
            with open(LOG, "a", encoding="utf-8") as f:
                f.write(f"NOT FOUND in {path}: {old[:80]!r}\n")
    with open(path, "wb") as f:
        f.write(data)
    return count

with open(LOG, "w", encoding="utf-8") as f:
    f.write("Starting CE.impl doc fixes\n")

# --- undo-redo-transactions/tasks.md: mark Tasks 19-20 as [x] ---
undo_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\undo-redo-transactions\tasks.md"
undo_replacements = [
    (b"- [ ] 19. SETUNDO command", b"- [x] 19. SETUNDO command"),
    (b"  - [ ] 19.1 Register SETUNDO primary command with ON/OFF/n operand parsing",
     b"  - [x] 19.1 Register SETUNDO primary command with ON/OFF/n operand parsing"),
    (b"  - [ ] 19.2 Implement SETUNDO ON",
     b"  - [x] 19.2 Implement SETUNDO ON"),
    (b"  - [ ] 19.3 Implement SETUNDO OFF",
     b"  - [x] 19.3 Implement SETUNDO OFF"),
    (b"  - [ ] 19.4 Implement SETUNDO n",
     b"  - [x] 19.4 Implement SETUNDO n"),
    (b"  - [ ] 19.5 Write unit tests for SETUNDO",
     b"  - [x] 19.5 Write unit tests for SETUNDO"),
    (b"- [ ] 20. RECOVERY command", b"- [x] 20. RECOVERY command"),
    (b"  - [ ] 20.1 Register RECOVERY primary command with ON/OFF/n operand parsing",
     b"  - [x] 20.1 Register RECOVERY primary command with ON/OFF/n operand parsing"),
    (b"  - [ ] 20.2 Implement RECOVERY ON",
     b"  - [x] 20.2 Implement RECOVERY ON"),
    (b"  - [ ] 20.3 Implement RECOVERY OFF",
     b"  - [x] 20.3 Implement RECOVERY OFF"),
    (b"  - [ ] 20.4 Implement RECOVERY n",
     b"  - [x] 20.4 Implement RECOVERY n"),
    (b"  - [ ] 20.5 Write unit tests for RECOVERY",
     b"  - [x] 20.5 Write unit tests for RECOVERY"),
]

n = fix_file(undo_path, undo_replacements)
with open(LOG, "a", encoding="utf-8") as f:
    f.write(f"undo-redo tasks.md: {n}/{len(undo_replacements)} replacements\n")

# --- project-master: mark CE.impl as [x] ---
master_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
master_replacements = [
    (b"- [ ] CE.impl undo-redo-transactions: SETUNDO command, RECOVERY ON/OFF (Tasks 19-20 in undo-redo-transactions/tasks.md)",
     b"- [x] CE.impl undo-redo-transactions: SETUNDO command, RECOVERY ON/OFF (Tasks 19-20 in undo-redo-transactions/tasks.md)"),
    (b"- [ ] CE.1 SETUNDO command -- ON/OFF/n operands, immediate effect on max_levels (Tasks 19.1-19.5)",
     b"- [x] CE.1 SETUNDO command -- ON/OFF/n operands, immediate effect on max_levels (Tasks 19.1-19.5)"),
    (b"- [ ] CE.2 RECOVERY command -- ON/OFF/n operands, immediate effect on recovery interval (Tasks 20.1-20.5)",
     b"- [x] CE.2 RECOVERY command -- ON/OFF/n operands, immediate effect on recovery interval (Tasks 20.1-20.5)"),
]

n = fix_file(master_path, master_replacements)
with open(LOG, "a", encoding="utf-8") as f:
    f.write(f"project-master tasks.md: {n}/{len(master_replacements)} replacements\n")

sys.stderr.write("done\n")
