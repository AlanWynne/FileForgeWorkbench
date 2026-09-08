"""Mark Task 32 subtasks complete in configuration-system/tasks.md and project-master/tasks.md."""
import sys

LOG = None  # print only

def patch(path, replacements):
    with open(path, "rb") as f:
        data = f.read()
    print(f"{path}: {len(data)} bytes", flush=True)
    count = 0
    for old, new in replacements:
        if old in data:
            data = data.replace(old, new, 1)
            count += 1
        else:
            print(f"  WARNING not found: {old[:60]}", flush=True)
    with open(path, "wb") as f:
        f.write(data)
    print(f"  {count} replacements made", flush=True)

TASKS = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\configuration-system\tasks.md"
MASTER = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

# Mark all Task 32 subtasks [x] in configuration-system/tasks.md
task_reps = []
for i in range(1, 13):
    old = f"  - [ ] 32.{i}".encode()
    new = f"  - [x] 32.{i}".encode()
    task_reps.append((old, new))
task_reps.append((b"- [ ] 32. Locked Configuration Keys (Requirement 18)", b"- [x] 32. Locked Configuration Keys (Requirement 18)"))

patch(TASKS, task_reps)

# Mark CQ.4 and CQ.5 complete in project-master/tasks.md
master_reps = [
    (b"- [ ] CQ.4 Locked config keys -- KeyLocked error, locked_keys enforcement in merger, is_locked API, Settings panel lock indicator (Task 32)",
     b"- [x] CQ.4 Locked config keys -- KeyLocked error, locked_keys enforcement in merger, is_locked API, Settings panel lock indicator (Task 32)"),
    (b"- [ ] CQ.5 TCR update + cargo test --workspace green (Task 32.11-32.12)",
     b"- [x] CQ.5 TCR update + cargo test --workspace green (Task 32.11-32.12)"),
]
patch(MASTER, master_reps)

print("Done.", flush=True)
