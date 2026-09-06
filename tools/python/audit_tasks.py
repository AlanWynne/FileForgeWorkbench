"""
audit_tasks.py -- Scan all docs/specs/<sub-project>/tasks.md files and report
pending vs complete task counts. Writes results to tools/logs/audit_tasks.txt.
"""
import os
import re

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\audit_tasks.txt"
SPECS_ROOT = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

log("=== Sub-Project Tasks Audit ===\n")

results = []
no_tasks = []

for entry in sorted(os.listdir(SPECS_ROOT)):
    folder = os.path.join(SPECS_ROOT, entry)
    if not os.path.isdir(folder):
        continue
    tasks_file = os.path.join(folder, "tasks.md")
    if not os.path.exists(tasks_file):
        no_tasks.append(entry)
        continue
    with open(tasks_file, encoding="utf-8", errors="replace") as f:
        content = f.read()
    done = len(re.findall(r"- \[x\]", content, re.IGNORECASE))
    pending = len(re.findall(r"- \[ \]", content))
    results.append((entry, done, pending))

log(f"{'Sub-project':<45} {'Done':>6} {'Pending':>8} {'Status'}")
log("-" * 75)

total_done = 0
total_pending = 0
has_pending = []
all_done = []
no_tasks_list = []

for name, done, pending in results:
    status = "ALL DONE" if pending == 0 else f"PENDING ({pending})"
    log(f"{name:<45} {done:>6} {pending:>8}   {status}")
    total_done += done
    total_pending += pending
    if pending > 0:
        has_pending.append((name, pending))
    else:
        all_done.append(name)

log("-" * 75)
log(f"{'TOTAL':<45} {total_done:>6} {total_pending:>8}")
log("")

log(f"=== Sub-projects WITH pending tasks ({len(has_pending)}) ===")
for name, count in has_pending:
    log(f"  {name} -- {count} pending")

log("")
log(f"=== Sub-projects with NO tasks.md ({len(no_tasks)}) ===")
for name in sorted(no_tasks):
    log(f"  {name}")

log("")
log(f"=== Summary ===")
log(f"  Sub-projects with tasks.md: {len(results)}")
log(f"  All done: {len(all_done)}")
log(f"  Has pending: {len(has_pending)}")
log(f"  No tasks.md: {len(no_tasks)}")
log(f"  Total [x] tasks: {total_done}")
log(f"  Total [ ] tasks: {total_pending}")
log("Done.")
