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

# Toolchain leaders. Historically these ran directly WITHOUT the pwsh7 wrapper
# on the theory that short, single-purpose commands do not trip PSReadLine
# mangling. That theory failed in practice: a bare command still executes in the
# INTERACTIVE PSReadLine session, so when the session buffer is already poisoned
# (a previous ';'-chain or echoed fragment left behind), even a clean bare
# `cargo --version` comes back glued to stale text with `Exit Code: -1`. The
# bare lane was therefore the remaining poisoning channel.
#
# Fix (CR-CH guard hardening): NO command runs bare anymore. Every command --
# toolchain leaders included -- must go through the non-interactive pwsh7
# wrapper, whose `-NonInteractive` flag is the only thing that actually stops the
# PSReadLine echo/prediction that causes the mangling. A bare toolchain leader is
# now classified "ask" (not blocked -- it is legitimate work), and the reason
# string carries the ready-to-run wrapped form so the owner can approve/paste it.
#
# `git` and `gh` were never bare-allowed, for the same reason; they continue to
# route through the wrapper.
TOOLCHAIN_LEADERS = (
    "cargo",
    "rustc",
    "rustup",
    "python",
    "py",
    "type",  # reading back a log file, explicitly endorsed by tooling.md
)


def wrapped_form(cmd):
    """The clean non-interactive pwsh7 form of an arbitrary command string."""
    inner = cmd.strip().replace('"', '`"')
    return (
        r'C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command '
        f'"{inner}"'
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
         these are bad even inside an otherwise-fine command like `cargo build`.
      2. Then allow ONLY the clean non-interactive pwsh7 wrapper (the one form a
         poisoned interactive buffer cannot corrupt).
      3. Then apply the softer "ask" heuristics: inspection cmdlets, format
         pipes, bare toolchain leaders (with the wrapped form in the reason), and
         the catch-all "not the clean form" case.
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

    # --- 2. Allowed clean form. ---
    # The clean, enforced non-interactive pwsh7 form is the ONLY form that runs
    # without a prompt, because it is the only form that cannot be poisoned by a
    # dirty interactive PSReadLine buffer.
    if uses_clean_wrapper(cmd_lower):
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

    # 3c. A bare toolchain leader (cargo/rustc/python/type/...). Legitimate work,
    #     but it must not run bare in the interactive shell (poisoning lane).
    #     Ask, and hand back the exact wrapped command to run instead.
    if leading_token(cmd) in TOOLCHAIN_LEADERS:
        return (
            "ask",
            "bare toolchain command runs in the interactive shell (poisoning "
            "lane). Run the wrapped form instead: " + wrapped_form(cmd),
        )

    # 3d. Any other bare shell command that is not the clean wrapper: ask, and
    #     steer toward the clean form.
    return (
        "ask",
        "not the pwsh7 wrapper (tooling.md 2: wrap with pwsh7 -NoProfile "
        "-NonInteractive, or script it). Suggested: " + wrapped_form(cmd),
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
