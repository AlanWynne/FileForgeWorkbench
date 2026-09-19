"""Check that touched docs use only allowed characters (documentation.md).

Allowed: ASCII (0x00-0x7F), box-drawing (U+2500-U+257F), and the TCR status
emoji (checkmark U+2705, cross U+274C, white square U+2B1C, red circle U+1F534).
Prints each offending file with line/char, or 'clean'.
"""
import sys

FILES = [
    r"docs\status\change-log.md",
    r"docs\specs\menu-workspace\tasks.md",
    r"docs\specs\menu-workspace\requirements.md",
    r"docs\specs\menu-workspace\design.md",
    r"docs\quality\TCR.md",
    r"docs\project-management\project-master\tasks.md",
]
ALLOWED = set(range(0x00, 0x80)) | set(range(0x2500, 0x2580)) | {0x2705, 0x274C, 0x2B1C, 0x1F534, 0x1F532}
BASE = r"C:\workspace\VSC\FileForgeWorkbench"


def main():
    any_bad = False
    for rel in FILES:
        path = BASE + "\\" + rel
        try:
            with open(path, "r", encoding="utf-8") as f:
                lines = f.readlines()
        except FileNotFoundError:
            print(f"{rel}: MISSING")
            continue
        bad = []
        for ln, line in enumerate(lines, 1):
            for col, ch in enumerate(line, 1):
                if ord(ch) not in ALLOWED:
                    bad.append((ln, col, hex(ord(ch)), ch))
        if bad:
            any_bad = True
            print(f"{rel}: {len(bad)} offending chars")
            for ln, col, code, ch in bad[:10]:
                print(f"   line {ln} col {col}: {code} {ch!r}")
        else:
            print(f"{rel}: clean")
    sys.exit(1 if any_bad else 0)


if __name__ == "__main__":
    main()
