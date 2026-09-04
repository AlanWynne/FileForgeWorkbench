# Script Output -- MANDATORY STDOUT CAPTURE

## Problem

`executeBash` frequently swallows stdout from Python and PowerShell scripts.
When this happens there is no feedback, the fix cannot be verified, and the
same work is repeated in the next attempt. This wastes time and risks
introducing duplicate or partial changes.

## Rule

**Every script invocation MUST redirect stdout and stderr to a log file.**
Never rely on inline stdout capture alone.

---

## Python Scripts

Always write output to a log file AND print to stdout:

```python
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Use log() instead of print() throughout the script
log("Script started")
log(f"Result: {result}")
log("Done")
```

Or redirect at the shell level:

```bat
python tools\python\my_script.py >> tools\logs\script-out.txt 2>&1
type tools\logs\script-out.txt
```

Always follow the script invocation with `type <log-file>` to read the output
back in the same `executeBash` call or the next one.

---

## PowerShell Scripts

```powershell
.\tools\powershell\my_script.ps1 | Tee-Object -FilePath tools\logs\script-out.txt
type tools\logs\script-out.txt
```

---

## Log File Location

- All script logs go to `tools\logs\` (create the directory if it does not exist).
- Log files are ephemeral -- do not commit them. Add `tools/logs/` to `.gitignore`.
- Clear or overwrite the log at the start of each script run so stale output
  does not mislead a subsequent read.

---

## Verification Step

After every script invocation, read the log file with `type` before declaring
the script succeeded or failed:

```bat
type tools\logs\script-out.txt
```

If the log file is empty or missing, the script did not run correctly -- do not
assume success.

---

## Binary-Mode Python File Patches

When patching files with mixed or unknown line endings (CRLF vs LF), use this
pattern and log every step:

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

# Try both line ending variants
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

This pattern:
- Logs file size so you know the file was opened
- Tries both `\r\n` and `\n` so line-ending ambiguity is resolved automatically
- Logs which separator matched
- Logs explicitly if nothing matched (never silent failure)
