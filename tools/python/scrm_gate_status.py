"""Report the current state of a running ffwb-gate run, PowerShell-profile-free.

Usage:
  python tools/python/scrm_gate_status.py            # one-shot status snapshot
  python tools/python/scrm_gate_status.py --wait N   # poll up to N seconds until
                                                      # ai-review.log appears, then report

Reads:
  tools/logs/verify.progress.txt   (phase/elapsed written by ffwb-gate.ps1)
  tools/logs/ai-review.log         (empty == clean gate; any lines == problems)
  tools/logs/scrm-verify-full.log  (full redirected run output; tail shown)

Writes a copy of the report to tools/logs/scrm-gate-status.txt.
"""

import os
import sys
import time

ROOT = r"C:\workspace\VSC\FileForgeWorkbench"
PROGRESS = os.path.join(ROOT, "tools", "logs", "verify.progress.txt")
AIREVIEW = os.path.join(ROOT, "tools", "logs", "ai-review.log")
FULLLOG = os.path.join(ROOT, "tools", "logs", "scrm-verify-full.log")
OUT = os.path.join(ROOT, "tools", "logs", "scrm-gate-status.txt")


def read(path, tail=None):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            data = f.read()
    except FileNotFoundError:
        return None
    if tail is not None:
        return "\n".join(data.splitlines()[-tail:])
    return data


def snapshot():
    lines = []
    lines.append("=== SCRM gate status @ " + time.strftime("%H:%M:%S") + " ===")
    prog = read(PROGRESS)
    lines.append("[progress.txt]")
    lines.append(prog.strip() if prog else "(missing)")
    lines.append("")
    if os.path.exists(AIREVIEW):
        size = os.path.getsize(AIREVIEW)
        lines.append("[ai-review.log] EXISTS size=%d bytes -> %s" % (
            size, "CLEAN" if size == 0 else "HAS PROBLEMS"))
        if size:
            lines.append(read(AIREVIEW, tail=60) or "")
        lines.append("GATE_FINISHED=1")
    else:
        lines.append("[ai-review.log] not present yet -> gate still running")
        lines.append("GATE_FINISHED=0")
    lines.append("")
    lines.append("[scrm-verify-full.log tail]")
    lines.append(read(FULLLOG, tail=25) or "(missing)")
    report = "\n".join(lines)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(report + "\n")
    print(report, flush=True)
    return os.path.exists(AIREVIEW)


def main():
    if len(sys.argv) >= 3 and sys.argv[1] == "--wait":
        deadline = time.time() + int(sys.argv[2])
        while time.time() < deadline:
            if os.path.exists(AIREVIEW):
                break
            time.sleep(10)
    snapshot()


if __name__ == "__main__":
    main()
