"""Read tails of TCR.md and change-log.md for BX gate preparation."""
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

open(LOG, "w").close()

for path, label, nbytes in [
    (r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md", "TCR tail", 3000),
    (r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md", "change-log tail", 2000),
]:
    with open(path, "rb") as f:
        f.seek(0, 2)
        size = f.tell()
        f.seek(max(0, size - nbytes))
        tail = f.read().decode("utf-8", errors="replace")
    log(f"=== {label} (last {nbytes} bytes of {size}) ===")
    log(tail)
    log("")
