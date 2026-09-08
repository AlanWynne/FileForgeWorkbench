#!/usr/bin/env python3
"""Recursively convert em-dash characters to ASCII dashes in matching files.

Purpose
-------
Walks a directory tree starting at a root (default: current working directory),
finds files whose names match one or more glob patterns, and replaces every
em-dash (U+2014, UTF-8 bytes E2 80 94) with the ASCII substitute.

The em-dash substitute is `--` by default (per the project's documentation
character rules). Use --single to substitute a single `-` instead.

Inputs
------
--root PATH        Directory to start the recursive scan (default: cwd).
--pattern GLOB     Filename glob to match; repeatable (default: *.md).
--single           Replace em-dash with a single '-' instead of '--'.
--apply            Actually write changes. Without this flag the script runs
                   in DRY-RUN mode and only reports what would change.
--log PATH         Log file path (default: tools/logs/convert-em-dash.txt).

Outputs / overwrite behaviour
-----------------------------
- DRY-RUN (default): no files are modified; the log lists candidate files and
  the number of em-dashes found in each.
- --apply: matching files are overwritten IN PLACE. Only files that actually
  contain an em-dash are rewritten. Files are read and written in binary mode
  so existing CRLF/LF line endings are preserved exactly.

All progress is printed to stdout AND appended to the log file so output is
never silently swallowed by the terminal.

Examples
--------
    python tools/python/convert_em_dash.py                 # dry-run, *.md, cwd
    python tools/python/convert_em_dash.py --apply
    python tools/python/convert_em_dash.py --pattern *.md --pattern *.txt --apply
    python tools/python/convert_em_dash.py --root docs --single --apply
"""

import argparse
import fnmatch
import os
import sys

EM_DASH = b"\xe2\x80\x94"  # U+2014 in UTF-8


def build_log(log_path):
    """Return a log() function that prints and appends to log_path.

    Overwrites (truncates) the log at the start of each run so stale output
    does not mislead a later reader.
    """
    os.makedirs(os.path.dirname(log_path), exist_ok=True)
    # Truncate at start of run.
    with open(log_path, "w", encoding="utf-8") as f:
        f.write("")

    def log(msg):
        print(msg, flush=True)
        with open(log_path, "a", encoding="utf-8") as f:
            f.write(msg + "\n")

    return log


def iter_matching_files(root, patterns):
    """Yield file paths under root whose basename matches any glob pattern."""
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in filenames:
            if any(fnmatch.fnmatch(name, pat) for pat in patterns):
                yield os.path.join(dirpath, name)


def process_file(path, replacement, apply_changes, log):
    """Report/convert em-dashes in one file. Returns count of em-dashes found."""
    try:
        with open(path, "rb") as f:
            data = f.read()
    except OSError as exc:
        log("ERROR reading {}: {}".format(path, exc))
        return 0

    count = data.count(EM_DASH)
    if count == 0:
        return 0

    if apply_changes:
        new_data = data.replace(EM_DASH, replacement)
        try:
            with open(path, "wb") as f:
                f.write(new_data)
        except OSError as exc:
            log("ERROR writing {}: {}".format(path, exc))
            return count
        log("FIXED  {}  ({} em-dash replaced)".format(path, count))
    else:
        log("WOULD FIX  {}  ({} em-dash found)".format(path, count))

    return count


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Recursively convert em-dash to ASCII dash in matching files."
    )
    parser.add_argument("--root", default=os.getcwd(),
                        help="Directory to scan recursively (default: cwd).")
    parser.add_argument("--pattern", action="append", default=None,
                        help="Filename glob to match; repeatable (default: *.md).")
    parser.add_argument("--single", action="store_true",
                        help="Replace em-dash with '-' instead of '--'.")
    parser.add_argument("--apply", action="store_true",
                        help="Write changes. Without it the run is a dry-run.")
    parser.add_argument(
        "--log",
        default=os.path.join("tools", "logs", "convert-em-dash.txt"),
        help="Log file path (default: tools/logs/convert-em-dash.txt).",
    )
    args = parser.parse_args(argv)

    patterns = args.pattern if args.pattern else ["*.md"]
    replacement = b"-" if args.single else b"--"
    log = build_log(args.log)

    mode = "APPLY" if args.apply else "DRY-RUN"
    log("=== convert_em_dash ({}) ===".format(mode))
    log("Root:        {}".format(os.path.abspath(args.root)))
    log("Patterns:    {}".format(", ".join(patterns)))
    log("Replacement: {!r}".format(replacement.decode("ascii")))
    log("")

    if not os.path.isdir(args.root):
        log("ERROR: root is not a directory: {}".format(args.root))
        return 1

    files_scanned = 0
    files_with_dash = 0
    total_dashes = 0

    for path in iter_matching_files(args.root, patterns):
        files_scanned += 1
        count = process_file(path, replacement, args.apply, log)
        if count:
            files_with_dash += 1
            total_dashes += count

    log("")
    log("Scanned {} matching file(s).".format(files_scanned))
    log("{} file(s) contained em-dashes; {} em-dash total.".format(
        files_with_dash, total_dashes))
    if not args.apply and total_dashes:
        log("Dry-run only. Re-run with --apply to write the changes.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
