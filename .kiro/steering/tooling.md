---
inclusion: always
---

# Project Tooling and Script Output

## When to use a project tool
Use this whenever a task needs a script, data transformation, report generator,
migration, or repository-maintenance helper.

### Location and reuse
- Reuse an existing project tool before writing a new script.
- Reusable project-specific tools live under `tools/python/`, `tools/powershell/`,
  or another appropriate `tools/` subfolder.
- Shared tools under `C:\tools\scripts` may be invoked when appropriate; do not
  copy them into the project without a reason and recorded provenance.
- Do not create scripts in the repository root.

### Temporary vs reusable
- One-off experiments, generated scripts, and failed approaches belong in the
  session workspace, not the repository.
- Do not delete a project tool merely because the current task is done.
- Before deleting or replacing a tool, check references and ask for confirmation
  if its purpose or ownership is unclear.
- Promote a temporary script into `tools/` only when it is safe to rerun,
  documented, and likely to be reused.

### Safety and documentation
- Inspect a script before running it.
- Prefer read-only or dry-run modes for discovery and migration work.
- Never use broad recursive deletion or unresolved wildcard paths.
- Tools that modify files must document inputs, outputs, and overwrite behaviour
  in a usage message or adjacent README.
- Keep generated output outside the repository unless it is an intentional artefact.
- After adding or changing a tool, run its documented help or smallest safe
  validation command.

---

## Script Output -- MANDATORY STDOUT CAPTURE

Terminal execution frequently swallows stdout from Python and PowerShell scripts.
When that happens there is no feedback, the fix cannot be verified, and work is
repeated. **Every script invocation MUST redirect stdout and stderr to a log
file.** Never rely on inline stdout capture alone.

### Python
Write to a log file and print:
```python
import sys
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
def log(msg):
    print(msg)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")
log("Script started")
```
Or redirect at the shell level, then read it back:
```bat
python tools\python\my_script.py >> tools\logs\script-out.txt 2>&1
type tools\logs\script-out.txt
```

### PowerShell
```powershell
.\tools\powershell\my_script.ps1 | Tee-Object -FilePath tools\logs\script-out.txt
type tools\logs\script-out.txt
```

### Log file location
- All script logs go to `tools\logs\` (create it if missing).
- Logs are ephemeral -- do not commit them; keep `tools/logs/` in `.gitignore`.
- Clear or overwrite the log at the start of each run so stale output does not mislead.

### Verification step
After every invocation, read the log with `type` before declaring success or
failure. An empty or missing log means the script did not run correctly -- do not
assume success.

### Binary-mode file patches (mixed line endings)
When patching files with unknown CRLF/LF endings, try both variants and log each
step so failures are never silent:
```python
import sys
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")
path = r"C:\path\to\file.md"
with open(path, "rb") as f:
    data = f.read()
log(f"File size: {len(data)} bytes")
for sep in (b"\r\n", b"\n"):
    old = b"- **Status**: IN PROGRESS" + sep + b"- **Status**: DONE"
    if old in data:
        log(f"Pattern found with separator {repr(sep)}")
        data = data.replace(old, b"- **Status**: DONE", 1)
        with open(path, "wb") as f:
            f.write(data)
        log("Replacement written")
        break
else:
    log("ERROR: pattern not found with either separator -- no change made")
```