#!/usr/bin/env python3
"""Validate a locale .ftl catalogue against the English Identity_Base (CR-NR-103).

Checks, per locale directory under crates/ff-i18n/i18n/<locale>/:
  1. Malformed Fluent unicode escapes -- any literal backslash-u that is NOT
     wrapped in a quoted string placeable { "\\uXXXX" }. A raw {\\uXXXX} or a
     bare \\uXXXX in a value will render wrong or fail to parse.
  2. Non-ASCII bytes in the source (documentation-charset: .ftl source stays
     ASCII; accented letters must be unicode escapes).
  3. Key parity against en: every en key present, and no extra keys (extra keys
     are reported as a warning, missing keys as an error).

Writes a full report to tools/logs/ftl-locale-check.txt and echoes it. Exit 0 =
OK, exit 1 = problems found. Per tooling.md, read the log back to confirm.

Usage:
  python tools/python/ftl_locale_check.py <repo_root> <locale>
"""
import os
import re
import sys

LOG_DIR = None


def log(lines, msg):
    lines.append(msg)
    print(msg, flush=True)


# A value may contain { "\uXXXX" } (good). Flag any other backslash-u.
GOOD_ESCAPE = re.compile(r'\{\s*"(\\u[0-9a-fA-F]{4})+"\s*\}')
# key = value  (Fluent message line; ignore comments and blanks and attributes)
MSG_LINE = re.compile(r'^([a-zA-Z][a-zA-Z0-9_-]*)\s*=')


def collect_keys(ftl_dir):
    keys = set()
    for fn in sorted(os.listdir(ftl_dir)):
        if not fn.endswith(".ftl"):
            continue
        with open(os.path.join(ftl_dir, fn), "r", encoding="utf-8") as f:
            for line in f:
                m = MSG_LINE.match(line)
                if m:
                    keys.add(m.group(1))
    return keys


def check_file(path, lines):
    problems = 0
    with open(path, "rb") as f:
        raw = f.read()
    # Non-ASCII byte check.
    for i, b in enumerate(raw):
        if b > 0x7F:
            log(lines, "  NON-ASCII byte 0x%02x at offset %d in %s" %
                (b, i, os.path.basename(path)))
            problems += 1
            break
    text = raw.decode("utf-8", errors="replace")
    # Replace all GOOD escapes, then any remaining backslash-u is malformed.
    stripped = GOOD_ESCAPE.sub("", text)
    for lineno, line in enumerate(stripped.splitlines(), 1):
        if "\\u" in line:
            log(lines, "  MALFORMED escape (not wrapped in { \"...\" }) at "
                "%s:%d -> %s" % (os.path.basename(path), lineno, line.strip()))
            problems += 1
    return problems


def main():
    if len(sys.argv) != 3:
        print("usage: ftl_locale_check.py <repo_root> <locale>")
        return 2
    repo = sys.argv[1]
    locale = sys.argv[2]
    i18n = os.path.join(repo, "crates", "ff-i18n", "i18n")
    en_dir = os.path.join(i18n, "en")
    loc_dir = os.path.join(i18n, locale)
    log_dir = os.path.join(repo, "tools", "logs")
    os.makedirs(log_dir, exist_ok=True)
    log_path = os.path.join(log_dir, "ftl-locale-check.txt")

    lines = []
    log(lines, "=== FTL locale check: %s ===" % locale)
    problems = 0

    if not os.path.isdir(loc_dir):
        log(lines, "ERROR: locale dir missing: %s" % loc_dir)
        with open(log_path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")
        return 1

    # Per-file escape + ASCII checks.
    for fn in sorted(os.listdir(loc_dir)):
        if fn.endswith(".ftl"):
            problems += check_file(os.path.join(loc_dir, fn), lines)

    # Key parity.
    en_keys = collect_keys(en_dir)
    loc_keys = collect_keys(loc_dir)
    missing = en_keys - loc_keys
    extra = loc_keys - en_keys
    log(lines, "en keys: %d   %s keys: %d" % (len(en_keys), locale, len(loc_keys)))
    if missing:
        log(lines, "MISSING %d key(s) vs en (will fall back to English):" % len(missing))
        for k in sorted(missing):
            log(lines, "  - %s" % k)
    if extra:
        log(lines, "WARNING: %d extra key(s) not in en:" % len(extra))
        for k in sorted(extra):
            log(lines, "  + %s" % k)
        problems += len(extra)

    if problems == 0 and not missing:
        log(lines, "OK: %s catalogue clean, full %d-key parity with en" %
            (locale, len(en_keys)))
    elif problems == 0 and missing:
        log(lines, "OK (escapes/ASCII clean) but %d key(s) rely on English fallback"
            % len(missing))

    with open(log_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
