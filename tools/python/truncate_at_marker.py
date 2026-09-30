"""Truncate a file at the line containing a marker (marker line removed).

Usage: python truncate_at_marker.py <file> <marker>
Writes a log of what it did to tools/logs/truncate.txt and prints it.
Keeps original line endings by working in binary and splitting on b"\n".
"""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\truncate.txt"


def log(msg: str) -> None:
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as fh:
        fh.write(msg + "\n")


def main() -> int:
    if len(sys.argv) != 3:
        log("ERROR: need <file> <marker>")
        return 2
    path, marker = sys.argv[1], sys.argv[2]
    with open(path, "rb") as fh:
        data = fh.read()
    marker_b = marker.encode("utf-8")
    idx = data.find(marker_b)
    if idx == -1:
        log(f"ERROR: marker {marker!r} not found in {path}")
        return 1
    # Back up to the start of the marker's line.
    line_start = data.rfind(b"\n", 0, idx)
    cut = 0 if line_start == -1 else line_start + 1
    kept = data[:cut]
    # Ensure the file ends with exactly one trailing newline.
    kept = kept.rstrip(b"\r\n") + b"\n"
    with open(path, "wb") as fh:
        fh.write(kept)
    log(f"Truncated {path} at marker; kept {len(kept)} bytes (was {len(data)}).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
