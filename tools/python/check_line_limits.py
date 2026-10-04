"""Report ff-desktop non-test .rs source files whose non-test line count exceeds 400.

Usage: python tools/python/check_line_limits.py
Reads nothing but the source tree; writes a report to tools/logs/phase2-linecounts.log
and prints the same. "Non-test lines" = all lines before a line that is exactly
`#[cfg(test)]` introducing the trailing test module (approximation: counts lines up
to the first top-level `#[cfg(test)]`). Files named tests_*.rs are skipped entirely.
"""
import os

ROOT = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src"
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\phase2-linecounts.log"
LIMIT = 400


def non_test_line_count(path):
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    for i, line in enumerate(lines):
        if line.strip() == "#[cfg(test)]":
            return i
    return len(lines)


def main():
    offenders = []
    for dirpath, _dirs, files in os.walk(ROOT):
        for name in files:
            if not name.endswith(".rs"):
                continue
            if name.startswith("tests_") or name == "tests.rs":
                continue
            full = os.path.join(dirpath, name)
            n = non_test_line_count(full)
            if n > LIMIT:
                rel = os.path.relpath(full, ROOT)
                offenders.append((n, rel))
    offenders.sort(reverse=True)
    out = []
    if offenders:
        out.append(f"OVER-LIMIT non-test files (> {LIMIT} non-test lines):")
        for n, rel in offenders:
            out.append(f"  {n:5d}  {rel}")
    else:
        out.append(f"OK: no non-test .rs file exceeds {LIMIT} non-test lines.")
    text = "\n".join(out) + "\n"
    with open(LOG, "w", encoding="utf-8") as f:
        f.write(text)
    print(text)


if __name__ == "__main__":
    main()
