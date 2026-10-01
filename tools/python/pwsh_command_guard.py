#!/usr/bin/env python3
"""PreToolUse guard for execute_pwsh -- pre-emptive mangled-output prevention.

This script is invoked by the Kiro PreToolUse hook registered in
`.kiro/hooks/pwsh-clean-output-guard.json`. It reads the hook payload as JSON on
stdin, inspects the shell command that is ABOUT to run, and decides whether that
command follows the clean-output process mandated by
`.kiro/steering/tooling.md` (the "Terminal Invocation" section).

Goal: be PRE-EMPTIVE. Stop a command that is likely to produce mangled output
(character-by-character echo, a second command glued to the front,
`Exit Code: -1`) BEFORE it runs, instead of recovering after the fact.

Decision protocol (Kiro PreToolUse):
  * exit 0 with NO decision JSON  -> allow silently (command is clean).
  * exit 0 with hookSpecificOutput.permissionDecision == "ask"
                                  -> Kiro prompts the owner to confirm first.
  * exit 2                        -> hard block; stderr is forwarded as the reason.

We use "ask" (not a hard block) for the risky-but-sometimes-legitimate cases so
the owner stays in control; we reserve exit 2 for the unambiguously bad forms.

Inputs: the exact JSON shape varies by Kiro version, so we search the payload
for the command string defensively rather than assuming one key.

Usage (manual test):
    echo '{"tool_input":{"command":"Get-Content x"}}' | \
        C:\\tools\\python\\python.exe tools\\python\\pwsh_command_guard.py
"""

import json
import re
import sys

CLEAN_PREFIX = r"C:\tools\powershell7\pwsh.exe"
NONINTERACTIVE_FLAGS = ("-noprofile", "-noninteractive")

# Cheap file-inspection intents that should never touch the shell at all --
# a dedicated Kiro tool exists for each. Presence of one of these cmdlets in a
# bare command is a strong signal the wrong path was taken.
INSPECTION_CMDLETS = (
    "get-content",
    "select-string",
    "get-childitem",
    "measure-object",
)

# Interactive-only aliases for the above (short forms people reach for).
INSPECTION_ALIASES = (
    r"\bgci\b",
    r"\bls\b",
    r"\bcat\b",
    r"\btype\b",
    r"\bsls\b",
)

# Commands that are fine to run directly WITHOUT the pwsh7 wrapper: the project
# build/test toolchain. These are short, non-interactive, single-purpose commands
# that do not trip the PSReadLine echo/prediction mangling the way long or
# interactive one-liners do, and wrapping them adds no value.
#
# NOTE: `git` and `gh` are DELIBERATELY NOT here. They were the one lane left
# unprotected by the -NonInteractive wrapper, and that is exactly where a git
# command got "swallowed by the terminal": git commands are frequently long
# (commit messages, many flags, paths) and can be interactive (pager, editor,
# prompts), which is precisely what triggers the PSReadLine mangling. Removing
# them from the allow-list routes them through case 3c -> "ask", steering them
# to the clean non-interactive pwsh7 wrapper form like every other command.
ALLOWED_BARE_LEADERS = (
    "cargo",
    "rustc",
    "rustup",
    "python",
    "py",
    "type",  # reading back a log file, explicitly endorsed by tooling.md
)


def find_command(payload):
    """Locate the command string inside an arbitrary hook payload."""
    if isinstance(payload, dict):
        for key in ("command", "cmd", "commandLine", "command_line"):
            val = payload.get(key)
            if isinstance(val, str) and val.strip():
                return val
        for value in payload.values():
            found = find_command(value)
            if found:
                return found
    elif isinstance(payload, list):
        for item in payload:
            found = find_command(item)
            if found:
                return found
    return None


def uses_clean_wrapper(cmd_lower):
    """True if the command explicitly invokes the non-interactive pwsh7 form."""
    if CLEAN_PREFIX.lower() not in cmd_lower:
        return False
    return all(flag in cmd_lower for flag in NONINTERACTIVE_FLAGS)


def leading_token(cmd):
    stripped = cmd.strip().strip("&").strip()
    if not stripped:
        return ""
    # Strip a leading call operator or drive-qualified path down to a bare name.
    first = stripped.split()[0]
    return first.lower().lstrip(".\\/")


def classify(cmd):
    """Return (decision, reason). decision in {"allow","ask","block"}.

    Order matters:
      1. Hard-block the unambiguously bad forms (';'-chain, process-kill) FIRST --
         these are bad even for an otherwise-allowed leader like `cargo`.
      2. Then allow the clean wrapper and known-safe bare toolchain leaders.
      3. Then apply the softer "ask" heuristics (inspection cmdlets, format pipes,
         and the catch-all "not the clean form" case).
    """
    cmd_lower = cmd.lower()

    # --- 1. Hard blocks (checked before any allow, so a bad chain on a good
    #        leader like "cargo build ; Stop-Process" is still blocked). ---
    if ";" in cmd:
        return (
            "block",
            "';'-chained line (tooling.md 3/4: one command, one job).",
        )
    if re.search(r"stop-process|taskkill|\bkill\b|\bspps\b", cmd_lower):
        return (
            "block",
            "process-kill glued onto a command (tooling.md 4).",
        )

    # --- 2. Allowed clean forms. ---
    # 2a. Already the clean, enforced non-interactive pwsh7 form.
    if uses_clean_wrapper(cmd_lower):
        return "allow", ""
    # 2b. Known-safe bare toolchain leader (cargo/git/python/type/...).
    if leading_token(cmd) in ALLOWED_BARE_LEADERS:
        return "allow", ""

    # --- 3. Softer "ask" heuristics for everything else. ---
    # 3a. Cheap inspection that should have used a dedicated tool, not the shell.
    if any(c in cmd_lower for c in INSPECTION_CMDLETS) or any(
        re.search(p, cmd_lower) for p in INSPECTION_ALIASES
    ):
        return (
            "ask",
            "inspection cmdlet (tooling.md 1: use read_file/grep_search/"
            "list_directory/file_search instead).",
        )

    # 3b. Multi-stage formatting pipe -- long interactive one-liner mangling.
    if re.search(
        r"\|\s*(format-table|ft|format-list|fl|select-object|select)\b", cmd_lower
    ):
        return (
            "ask",
            "Format-Table/Select-Object pipe (tooling.md 3: script it + read "
            "the log).",
        )

    # 3c. Any other bare shell command that is not the clean wrapper and not a
    #     known-safe toolchain leader: ask, and steer toward the clean form.
    return (
        "ask",
        "not the pwsh7 wrapper or a bare toolchain leader (tooling.md 2: wrap "
        "with pwsh7 -NoProfile -NonInteractive, or script it).",
    )


def main():
    raw = sys.stdin.read()
    try:
        payload = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError:
        # Cannot parse -> do not obstruct; allow silently.
        return 0

    cmd = find_command(payload)
    if not cmd:
        return 0

    decision, reason = classify(cmd)

    if decision == "allow":
        return 0

    if decision == "block":
        sys.stderr.write("Guard BLOCK: " + reason + "\n")
        return 2

    # decision == "ask"
    out = {
        "hookSpecificOutput": {
            "permissionDecision": "ask",
            "permissionDecisionReason": "Guard: " + reason,
        }
    }
    sys.stdout.write(json.dumps(out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
