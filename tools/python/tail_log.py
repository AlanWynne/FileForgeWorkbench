"""Print the last N lines of a file. Usage: python tail_log.py <file> [n]"""
import sys

path = sys.argv[1]
n = int(sys.argv[2]) if len(sys.argv) > 2 else 40
with open(path, "rb") as fh:
    data = fh.read()
lines = data.split(b"\n")
out = b"\n".join(lines[-n:])
with open(path + ".tail", "wb") as fh:
    fh.write(out)
