#!/usr/bin/env python3
"""Measure per-file and total line counts for a crate's src tree.

Usage:
    python measure_crate_lines.py <crate_src_dir> [out_log]

Writes a stable, sorted report (largest first) to stdout AND to the log file so
terminal capture is never the single source of truth (see tooling.md). Splits
test lines (inside a `#[cfg(test)]` module heuristically counted) from non-test
lines is NOT attempted here -- this is a raw line census for the Req 19.4
before/after decomposition measurement.
"""
import os
import sys


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: measure_crate_lines.py <crate_src_dir> [out_log]")
        return 2
    root = sys.argv[1]
    out_log = sys.argv[2] if len(sys.argv) > 2 else None

    rows = []
    total = 0
    for dirpath, _dirs, files in os.walk(root):
        for name in files:
            if not name.endswith(".rs"):
                continue
            path = os.path.join(dirpath, name)
            try:
                with open(path, "r", encoding="utf-8", errors="replace") as f:
                    n = sum(1 for _ in f)
            except OSError as e:
                print(f"ERROR reading {path}: {e}")
                continue
            rel = os.path.relpath(path, root).replace("\\", "/")
            rows.append((n, rel))
            total += n

    rows.sort(reverse=True)
    lines = []
    lines.append(f"crate src root : {root}")
    lines.append(f"total .rs files: {len(rows)}")
    lines.append(f"total lines    : {total}")
    lines.append("")
    lines.append("per-file (largest first):")
    for n, rel in rows:
        lines.append(f"{n:8d}  {rel}")
    report = "\n".join(lines) + "\n"

    sys.stdout.write(report)
    if out_log:
        os.makedirs(os.path.dirname(out_log), exist_ok=True)
        with open(out_log, "w", encoding="utf-8") as f:
            f.write(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
