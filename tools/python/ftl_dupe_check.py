#!/usr/bin/env python3
"""Check for duplicate Fluent message ids across every .ftl in a locale dir.

A duplicate id across files in the same locale directory breaks ff-i18n's bundle
build (fluent add_resource rejects an override), which takes down the whole
locale bundle. This guards against that at authoring time.

Usage: python ftl_dupe_check.py <locale_dir>
Writes its report to tools/logs/ftl-dupe-check.txt and prints it.
"""
import sys
import re
import os

LOG = os.path.join(os.path.dirname(__file__), "..", "logs", "ftl-dupe-check.txt")
KEY_RE = re.compile(r"^([a-zA-Z][a-zA-Z0-9_-]*)\s*=")


def log(msg, fh):
    print(msg, flush=True)
    fh.write(msg + "\n")


def main():
    if len(sys.argv) != 2:
        print("usage: ftl_dupe_check.py <locale_dir>")
        return 2
    locale_dir = sys.argv[1]
    seen = {}  # key -> first file
    dupes = []
    os.makedirs(os.path.dirname(LOG), exist_ok=True)
    with open(LOG, "w", encoding="utf-8") as fh:
        files = sorted(
            f for f in os.listdir(locale_dir) if f.endswith(".ftl")
        )
        total_keys = 0
        for fname in files:
            path = os.path.join(locale_dir, fname)
            with open(path, "r", encoding="utf-8") as f:
                for i, line in enumerate(f, 1):
                    m = KEY_RE.match(line)
                    if not m:
                        continue
                    key = m.group(1)
                    total_keys += 1
                    if key in seen:
                        dupes.append((key, seen[key], f"{fname}:{i}"))
                    else:
                        seen[key] = f"{fname}:{i}"
        log(f"scanned {len(files)} file(s), {total_keys} key(s)", fh)
        if dupes:
            log(f"DUPLICATE KEYS: {len(dupes)}", fh)
            for key, first, second in dupes:
                log(f"  '{key}' first at {first}, again at {second}", fh)
            return 1
        log("OK: no duplicate message ids across the locale", fh)
        return 0


if __name__ == "__main__":
    sys.exit(main())
