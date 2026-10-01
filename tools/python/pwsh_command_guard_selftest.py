#!/usr/bin/env python3
"""Self-test for pwsh_command_guard.classify. Logs results, no shell mangling.

Run:  C:\\tools\\python\\python.exe tools\\python\\pwsh_command_guard_selftest.py
Reads nothing from stdin; imports the guard module and asserts each case.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pwsh_command_guard as g  # noqa: E402

LOG = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "logs", "guard-selftest.txt"
)
LOG = os.path.abspath(LOG)
os.makedirs(os.path.dirname(LOG), exist_ok=True)


def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


# (command, expected_decision)
CASES = [
    # Clean enforced wrapper -> allow
    (
        r'C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo --version"',
        "allow",
    ),
    # Bare toolchain -> allow
    ("cargo test -p ff-desktop", "allow"),
    ("type tools\\logs\\out.txt", "allow"),
    # git/gh are NOT bare-allowed (they were the unprotected lane that swallowed
    # commands); they must route to the clean non-interactive wrapper -> ask.
    ("git status", "ask"),
    ("git commit -m \"a long message with several words\"", "ask"),
    ("gh pr create --title x --body y", "ask"),
    # git IS allowed when run through the clean non-interactive pwsh7 wrapper.
    (
        r'C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "git status"',
        "allow",
    ),
    # Inspection cmdlet on the shell -> ask (should have used a tool)
    ("Get-Content Cargo.toml", "ask"),
    ("Select-String -Path *.rs -Pattern foo", "ask"),
    ("gci crates", "ask"),
    # ';'-chain -> block
    ("cargo build ; cargo test", "block"),
    # kill glued on -> block
    ("Stop-Process -Name cargo -Force", "block"),
    ("cargo test ; Stop-Process -Name pwsh", "block"),
    # Format-Table pipe -> ask
    ("Get-Process | Format-Table Name", "ask"),
    # Unknown bare command, not clean wrapper -> ask
    ("somerandom.exe --flag", "ask"),
]


def main():
    # Fresh log each run.
    open(LOG, "w", encoding="utf-8").close()
    log("=== pwsh_command_guard self-test ===")
    failures = 0
    for cmd, expected in CASES:
        decision, reason = g.classify(cmd)
        ok = decision == expected
        if not ok:
            failures += 1
        log(
            f"[{'PASS' if ok else 'FAIL'}] expected={expected:5s} "
            f"got={decision:5s} :: {cmd}"
        )
        if not ok and reason:
            log(f"        reason: {reason}")
    log(f"=== {len(CASES) - failures}/{len(CASES)} passed, {failures} failed ===")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
