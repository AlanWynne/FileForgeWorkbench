#!/usr/bin/env python3
"""ASCII-fix tool for Rust source files (PA-STD remediation).

Replaces prohibited non-ASCII punctuation (both correctly-encoded UTF-8 and
common UTF-8-as-Latin1 MOJIBAKE byte-sequences) with ASCII substitutes, so
`.rs` files satisfy the strict-ASCII rule in documentation.md.

USAGE:
    python tools/python/ascii_fix.py --check  <dir>      # report only, no writes
    python tools/python/ascii_fix.py --apply  <dir>      # rewrite files in place

Operates on RAW BYTES (mixed CRLF/LF safe). Every change is logged to
tools/logs/ascii-fix.log. Idempotent: re-running on a clean file is a no-op.

Substitutions (byte-level, longest-first to avoid partial matches):
  UTF-8 mojibake (bytes that were UTF-8 then mis-decoded as Latin-1/CP1252):
    "\xc3\xa2\xe2\x82\xac\xe2\x80\x9d" (a-circumflex euro ...) family -> handled
  Correctly-encoded UTF-8 punctuation:
    em dash  U+2014  e2 80 94  -> --
    en dash  U+2013  e2 80 93  -> -
    r arrow  U+2192  e2 86 92  -> ->
    lr arrow U+2194  e2 86 94  -> <->
    l quote  U+2018  e2 80 98  -> '
    r quote  U+2019  e2 80 99  -> '
    l dquote U+201c  e2 80 9c  -> "
    r dquote U+201d  e2 80 9d  -> "
    ellipsis U+2026  e2 80 a6  -> ...
    multiply U+00d7  c3 97     -> x
    minus    U+2212  e2 88 92  -> -
    nbsp     U+00a0  c2 a0     -> (space)
"""
import sys
import os

LOG = os.path.join(os.path.dirname(__file__), "..", "logs", "ascii-fix.log")


def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


# Correctly-encoded UTF-8 byte sequences -> ASCII. Longest first.
UTF8_MAP = [
    (b"\xe2\x80\x94", b"--"),   # em dash
    (b"\xe2\x80\x93", b"-"),    # en dash
    (b"\xe2\x86\x92", b"->"),   # rightwards arrow
    (b"\xe2\x86\x90", b"<-"),   # leftwards arrow
    (b"\xe2\x86\x94", b"<->"),  # left-right arrow
    (b"\xe2\x94\x80", b"="),    # box-drawing light horizontal (comment separators)
    (b"\xe2\x80\x98", b"'"),    # left single quote
    (b"\xe2\x80\x99", b"'"),    # right single quote
    (b"\xe2\x80\x9c", b'"'),    # left double quote
    (b"\xe2\x80\x9d", b'"'),    # right double quote
    (b"\xe2\x80\xa6", b"..."),  # ellipsis
    (b"\xe2\x88\x92", b"-"),    # minus sign
    (b"\xe2\x89\xa5", b">="),   # >=
    (b"\xe2\x89\xa4", b"<="),   # <=
    (b"\xe2\x89\xa0", b"!="),   # !=
    (b"\xc3\x97", b"x"),        # multiplication sign
    (b"\xc2\xa0", b" "),        # non-breaking space
    (b"\xc2\xa7", b"Sec "),     # section sign (e.g. "Design Sec 9")
]

# Mojibake: the above UTF-8 sequences RE-ENCODED after a wrong Latin-1/CP1252
# decode. e.g. em dash e2 80 94, mis-decoded cp1252 -> "a-circ EURO dq-right"
# then re-saved as utf-8 -> c3 a2 e2 82 ac e2 80 9d. We map those back too.
MOJIBAKE_MAP = [
    (b"\xc3\xa2\xe2\x82\xac\xe2\x80\x9d", b"--"),   # em dash mojibake
    (b"\xc3\xa2\xe2\x82\xac\xe2\x80\x9c", b"-"),    # en dash mojibake
    (b"\xc3\xa2\xe2\x80\xa0\xe2\x80\x99", b"->"),   # arrow mojibake (approx)
    (b"\xc3\xa2\xe2\x82\xac\xe2\x84\xa2", b"'"),    # right single quote mojibake
    (b"\xc3\xa2\xe2\x82\xac\xc5\x93", b'"'),        # left double quote mojibake
]


def fix_bytes(data):
    changes = {}
    # Mojibake first (longer, more specific sequences).
    for seq, rep in MOJIBAKE_MAP:
        if seq in data:
            n = data.count(seq)
            data = data.replace(seq, rep)
            changes[repr(seq)] = (repr(rep), n)
    for seq, rep in UTF8_MAP:
        if seq in data:
            n = data.count(seq)
            data = data.replace(seq, rep)
            changes[repr(seq)] = (repr(rep), n)
    return data, changes


def has_non_ascii(data):
    return any(b > 0x7F for b in data)


def process(path, apply):
    with open(path, "rb") as f:
        data = f.read()
    if not has_non_ascii(data):
        return 0
    new_data, changes = fix_bytes(data)
    remaining = sum(1 for b in new_data if b > 0x7F)
    rel = path
    if changes:
        log(f"{rel}: {sum(c[1] for c in changes.values())} replacements")
        for seq, (rep, n) in changes.items():
            log(f"    {seq} -> {rep}  x{n}")
    if remaining:
        # Report any non-ASCII we did NOT map, so nothing is silently missed.
        leftover = sorted({b for b in new_data if b > 0x7F})
        log(f"    WARNING {rel}: {remaining} non-ASCII bytes UNMAPPED: "
            + ", ".join(f"0x{b:02X}" for b in leftover))
    if apply and changes and remaining == 0:
        with open(path, "wb") as f:
            f.write(new_data)
        log(f"    WROTE {rel}")
    elif apply and remaining:
        log(f"    SKIPPED WRITE {rel} (would leave non-ASCII)")
    return remaining


def main():
    if len(sys.argv) < 3 or sys.argv[1] not in ("--check", "--apply"):
        print(__doc__)
        sys.exit(2)
    apply = sys.argv[1] == "--apply"
    root = sys.argv[2]
    # Fresh log per run.
    with open(LOG, "w", encoding="utf-8") as f:
        f.write(f"ascii_fix {'APPLY' if apply else 'CHECK'} {root}\n")
    total_remaining = 0
    files = 0
    for dirpath, _dirs, names in os.walk(root):
        for name in names:
            if name.endswith(".rs"):
                files += 1
                total_remaining += process(os.path.join(dirpath, name), apply)
    log(f"DONE: scanned {files} .rs files; remaining non-ASCII bytes = {total_remaining}")
    # Exit nonzero if anything remains unmapped (so callers can gate).
    sys.exit(1 if total_remaining else 0)


if __name__ == "__main__":
    main()
