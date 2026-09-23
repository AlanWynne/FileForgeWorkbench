#!/usr/bin/env python3
"""Detect and (optionally) repair mojibake / non-ASCII sequences in a Markdown
or text file, replacing them with the ASCII substitutes mandated by
`.kiro/steering/documentation.md`.

Mojibake here is UTF-8 bytes that were at some point decoded as Windows-1252 /
Latin-1 and re-encoded, so a real em-dash (U+2014, UTF-8 `E2 80 94`) shows up as
the three characters `a-circumflex, Euro, "` (bytes `C3 A2 E2 82 AC E2 80 9D`),
etc. We operate on the decoded text and map each known bad sequence to its ASCII
substitute.

Usage:
    python tools/python/fix_mojibake.py <file>            # dry-run: report only
    python tools/python/fix_mojibake.py <file> --apply    # rewrite the file

All actions are logged to tools/logs/fix-mojibake.log AND printed. The file is
read/written as UTF-8; box-drawing (U+2500-U+257F) and the TCR status emoji are
preserved (they are allowed by the documentation rule).
"""

import sys
from pathlib import Path

LOG = Path(r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\fix-mojibake.log")


def log(msg: str) -> None:
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


# Mojibake sequences (UTF-8-as-cp1252 double-encoding) and true typographic
# characters, mapped to their ASCII substitutes per documentation.md.
# Order matters: longer / more specific sequences first.
REPLACEMENTS = [
    # -- double-encoded (mojibake) forms --
    ("\u00e2\u20ac\u201c", "-"),    # en dash  U+2013 -> -
    ("\u00e2\u20ac\u201d", "--"),   # em dash  U+2014 -> --
    ("\u00e2\u20ac\u2122", "'"),    # right single quote U+2019
    ("\u00e2\u20ac\u02dc", "'"),    # left single quote  U+2018
    ("\u00e2\u20ac\u0153", '"'),    # left double quote  U+201C
    ("\u00e2\u20ac\u009d", '"'),    # right double quote U+201D (variant)
    ("\u00e2\u20ac\u201e", '"'),    # double low-9 quote
    ("\u00e2\u20ac\u00a6", "..."),  # ellipsis U+2026
    ("\u00e2\u20ac", '"'),          # stray dangling curly-quote lead-in
    # -- true typographic characters (single code point) --
    ("\u2014", "--"),
    ("\u2013", "-"),
    ("\u2019", "'"),
    ("\u2018", "'"),
    ("\u201c", '"'),
    ("\u201d", '"'),
    ("\u2026", "..."),
    ("\u2192", "->"),
    ("\u2264", "<="),
    ("\u2265", ">="),
    ("\ufeff", ""),                 # BOM
]

# Allowed non-ASCII ranges to leave untouched.
ALLOWED_EMOJI = {"\u2705", "\U0001f534", "\U0001f532", "\u274c", "\u2b1c"}


def is_box_drawing(ch: str) -> bool:
    return "\u2500" <= ch <= "\u257f"


def main() -> int:
    if len(sys.argv) < 2:
        log("ERROR: usage: fix_mojibake.py <file> [--apply]")
        return 2
    path = Path(sys.argv[1])
    apply = "--apply" in sys.argv[1:]
    if not path.is_file():
        log(f"ERROR: not a file: {path}")
        return 2

    text = path.read_text(encoding="utf-8")
    log(f"Loaded {path} ({len(text)} chars)")

    total = 0
    for bad, good in REPLACEMENTS:
        n = text.count(bad)
        if n:
            disp = repr(bad)
            log(f"  {disp} -> {good!r}: {n} occurrence(s)")
            text = text.replace(bad, good)
            total += n
    log(f"Total sequences replaced: {total}")

    # Residual non-ASCII scan (excluding allowed ranges).
    residual = {}
    for ch in text:
        if ord(ch) < 0x80 or is_box_drawing(ch) or ch in ALLOWED_EMOJI:
            continue
        residual[ch] = residual.get(ch, 0) + 1
    if residual:
        log("Residual non-ASCII (NOT auto-substituted -- review):")
        for ch, n in sorted(residual.items(), key=lambda kv: -kv[1]):
            log(f"  U+{ord(ch):04X} {ch!r}: {n}")
    else:
        log("No residual disallowed non-ASCII remains.")

    if apply and total:
        path.write_text(text, encoding="utf-8", newline="")
        log(f"APPLIED: rewrote {path}")
    elif apply:
        log("APPLY requested but nothing to change.")
    else:
        log("DRY-RUN: no file written (pass --apply to rewrite).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
