"""List all bug rows in docs/status/bugs.md whose status is not closed."""
import re

CLOSED = {"FIXED", "VERIFIED", "SUPERSEDED", "DUPLICATE", "INVALID", "CLOSED"}
PATH = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\bugs.md"


def main():
    rows = []
    with open(PATH, encoding="utf-8") as f:
        for line in f:
            if not line.startswith("| B"):
                continue
            cells = [c.strip() for c in line.split("|")]
            # cells[0] is '', cells[1]=id, [2]=status, [3]=severity, [4]=component, [5]=description
            if len(cells) < 6:
                continue
            bid, status, sev, comp = cells[1], cells[2], cells[3], cells[4]
            if not re.match(r"^B\d+$", bid):
                continue
            if status.upper() in CLOSED:
                continue
            desc = cells[5][:150]
            print(f"{bid} | {status} | {sev} | {comp} | {desc}")


if __name__ == "__main__":
    main()
