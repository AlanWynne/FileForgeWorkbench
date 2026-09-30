"""Block until a log file contains a marker line, then print its tail.

PowerShell-profile-free waiter (use C:\\tools\\python\\python.exe) for polling
cargo/test runs whose output is redirected to a log file.

Usage:
  python tools/python/wait_for_marker.py <logfile> [marker] [timeout_secs]

Defaults: marker="EXIT=", timeout=1200 seconds. Prints the last 60 lines of the
log once the marker appears (or on timeout).
"""

import os
import sys
import time


def main():
    if len(sys.argv) < 2:
        print("usage: wait_for_marker.py <logfile> [marker] [timeout_secs]")
        return
    logfile = sys.argv[1]
    marker = sys.argv[2] if len(sys.argv) > 2 else "EXIT="
    timeout = int(sys.argv[3]) if len(sys.argv) > 3 else 1200
    deadline = time.time() + timeout
    found = False
    while time.time() < deadline:
        try:
            with open(logfile, "r", encoding="utf-8", errors="replace") as f:
                data = f.read()
            if marker in data:
                found = True
                break
        except FileNotFoundError:
            pass
        time.sleep(5)
    try:
        with open(logfile, "r", encoding="utf-8", errors="replace") as f:
            lines = f.read().splitlines()
    except FileNotFoundError:
        lines = ["(log file not found)"]
    print("MARKER_FOUND=%s @ %s" % (found, time.strftime("%H:%M:%S")), flush=True)
    print("\n".join(lines[-60:]), flush=True)


if __name__ == "__main__":
    main()
