---
inclusion: always
---

# Project Tooling and Script Output

## Available interpreters
Both interpreters are installed; use these explicit paths so the correct engine
runs regardless of what is on PATH:

- PowerShell 7: `C:\tools\powershell7\pwsh.exe` (7.6.x). ALWAYS use this, never
  the default Windows PowerShell 5.1. The Postgres/credentials banner comes ONLY
  from the 5.1 machine profile; pwsh 7 has a separate profile and does NOT print
  it. So the fix for the banner is simply "use pwsh7" -- `-NoProfile` is not
  required to avoid the banner (though it is still preferred for a clean, fast,
  deterministic session). Use pwsh 7 for `.ps1` tools.
- Python 3: `C:\tools\python\python.exe` (3.13.x). Use for `.py` tools.

Write reusable scripts in either language and save them under
`C:\workspace\VSC\FileForgeWorkbench\tools\` (see Location and reuse below) so
they are not rebuilt every session.

## Terminal Invocation -- BE PRE-EMPTIVE, NOT RE-ACTIVE (mandatory)

The FIRST command MUST already be in a clean form. "Mangled output" (commands
echoed character-by-character, another command glued to the front,
`Exit Code: -1`) is caused by long/complex one-liners in an interactive
PSReadLine session and by chaining unrelated steps. Do NOT run a risky command,
watch it mangle, then recover with a log file -- that reactive path is the very
failure we are eliminating. Prevent the mangling at the FIRST command.

### The clean-form decision -- apply BEFORE issuing any shell command

```
Is it file inspection (read / find / count / grep)?
    -> YES: use a dedicated tool (read_file / grep_search / list_directory /
            file_search). Do NOT touch the shell at all.
    -> NO: continue.
Does the command run a program (toolchain, git, gh, a script, anything)?
    -> ALWAYS run it via the NON-INTERACTIVE pwsh7 wrapper (see below), OR put
       the logic in a tools/ script and run that script via the wrapper + read
       its log. There is NO bare exception any more: even cargo / rustc / rustup
       / python / py / type MUST go through the wrapper. A bare command still
       executes in the interactive PSReadLine session, so a poisoned buffer glues
       stale text onto it and yields `Exit Code: -1`; `-NonInteractive` is the
       only thing that stops that, and the guard now classifies every bare
       leader (toolchain, git, gh) as `ask` with the wrapped form in its reason.
```

### The rules (each is now MECHANICALLY ENFORCED -- see the guard hook below)

1. **Prefer the dedicated tools over the shell for inspection.** Use `read_file`,
   `grep_search`, `list_directory`, `file_search` instead of `Get-Content`,
   `Select-String`, `Get-ChildItem`, `Measure-Object`. Most "count lines / find
   files / read a file" needs have a tool and never touch the terminal.

2. **Use pwsh 7, non-interactively, for EVERY shell command.** Run it
   EXACTLY as:
   `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<command>"`
   pwsh7 removes the Postgres/credentials banner (the 5.1 machine profile is not
   loaded); `-NonInteractive` stops the PSReadLine echo/prediction that causes the
   character-by-character mangling; `-NoProfile` keeps the session clean and fast.
   There is NO bare-command exception: `cargo`/`rustc`/`rustup`/`python`/`py`/
   `type`, like `git` and `gh`, MUST all use the wrapper. The bare lane was the
   remaining way a command got swallowed by the terminal -- a bare command runs
   in the interactive session, so a poisoned buffer corrupts even a clean short
   command. The guard now classifies any bare leader as `ask` and hands back the
   exact wrapped form to run.

   `-NonInteractive` stops the PSReadLine echo/prediction mangling; it does NOT
   auto-answer a command's OWN confirmation prompt. A cmdlet or program that
   issues its own prompt (a `ShouldProcess` confirmation, an overwrite query, a
   credential/consent request) will STALL under `-NonInteractive` with no way to
   answer. Therefore EVERY command MUST also carry the explicit no-prompt switches
   that pre-empt its own prompts, for example:
   - `Remove-Item -Force` (add `-Recurse` for directories);
   - `-Confirm:$false` on any cmdlet that supports `-Confirm`
     (e.g. `Stop-Process -Confirm:$false`);
   - `-Force` on `New-Item` / `Copy-Item` / `Move-Item` where an overwrite could
     be queried;
   - `--yes` / `-y` on package managers and `gh` where a confirmation is possible;
   - NEVER an interactive flag (`-i` / `--interactive`) on `git` or any tool.
   The goal is that the FIRST invocation already cannot block on input -- do not
   run a prompting form and then react to the stall.

3. **One command, one job.** NEVER `;`-chain steps, NEVER pipe into
   `Format-Table`/`Select-Object`, NEVER put inline `$( ... )` subexpressions on
   the same line. If logic is needed, put it in a `tools/` script and run the
   script (see Script Output). A `;`-chained line is a hard violation.

4. **NEVER chain a process-cleanup / kill step onto a real command.** Killing
   stale `pwsh`/`cargo` processes on the same line is a primary source of the
   glued-command echo. If cleanup is ever needed, run it as its OWN separate
   invocation. A kill glued onto another command is a hard violation.

5. **For any script or multi-step logic, redirect to a log and read the log**
   (see "Script Output -- MANDATORY STDOUT CAPTURE" below). Do not try to parse
   rich stdout inline.

6. **If a command is ever refused by the guard hook, do NOT re-issue the same
   form.** Switch to a dedicated tool, the pwsh7 wrapper, or a `tools/` script +
   log. Re-issuing the same interactive one-liner reproduces the same mangling.

### The guard hook -- enforcement, not just convention

`.kiro/hooks/pwsh-clean-output-guard.json` is a `PreToolUse` hook on the shell
tool. Before ANY shell command runs, it invokes
`tools/python/pwsh_command_guard.py`, which classifies the command:

- **allow (silent)** -- ONLY the clean, non-interactive pwsh7 wrapper form. It is
  the only form that cannot be corrupted by a poisoned interactive buffer, so it
  is the only form allowed to run without a prompt.
- **ask (owner confirms first)** -- any bare command, including a bare toolchain
  leader (`cargo`/`rustc`/`rustup`/`python`/`py`/`type`), a bare `git`/`gh`
  command, a bare inspection cmdlet
  (`Get-Content`/`Select-String`/`Get-ChildItem`/`Measure-Object` or an alias), a
  `Format-Table`/`Select-Object` pipe, or any other non-wrapper command. For a
  bare toolchain/other leader the reason string carries the ready-to-run wrapped
  form; for an inspection cmdlet it points to the dedicated tool to use instead.
- **block (exit 2, command does not run)** -- a `;`-chained line, or a
  process-kill (`Stop-Process`/`taskkill`/`kill`) glued onto a command.

The guard makes the rules above a guardrail rather than a hope. When it fires,
read its reason, fix the command to the clean form, and proceed -- do NOT argue
with it or retry the rejected form. The guard is self-tested by
`tools/python/pwsh_command_guard_selftest.py` (run it after editing the guard).

### The session-start clear -- buffer hygiene, once per session

`.kiro/hooks/session-start-buffer-clear.json` is a `SessionStart` hook that runs
`Clear-Host` (via the clean non-interactive pwsh7 form) ONCE at the start of each
session. Its job is buffer hygiene: it wipes any stale or previously poisoned
PSReadLine output so it cannot bleed into or visually mangle the first commands
of the new session.

This is deliberately NOT a per-command clear. A `clear` prepended to a real
command would be a `;`-chained two-step line -- the guard hook hard-blocks that,
and rightly so, because chaining is itself a mangling trigger. A screen clear
also only wipes what is already displayed; it does not stop PSReadLine from
re-rendering the NEXT command. Prevention of per-command mangling is the job of
`-NonInteractive` (rule 2), not of clearing the screen. So: clear ONCE at
session start for a clean slate, and rely on the non-interactive wrapper for
every command after that. Never glue a clear onto another command.

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