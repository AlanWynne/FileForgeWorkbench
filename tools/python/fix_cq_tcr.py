"""Fix CQ TCR rows: ff-config Req 16.x, 17.x, 18.x from ? to checkmark."""
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\fix_cq_tcr.txt"
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

log("=== fix_cq_tcr.py ===")

tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
data = open(tcr_path, encoding="utf-8").read()
log(f"TCR size: {len(data)}")

# Find the Phase CQ section and fix ? rows within it
# The CQ rows are in the ff-config section and contain audit.rs, export_import.rs, merger.rs
fixed = 0
lines = data.split("\n")
new_lines = []
in_cq = False
for line in lines:
    # Detect start of CQ section
    if "Phase CQ" in line and "Enterprise" in line:
        in_cq = True
    # Detect start of next phase section (CR)
    if "Phase CR" in line and in_cq:
        in_cq = False
    # Fix ? rows in CQ section
    if in_cq and "| ? |" in line and (
        "audit.rs" in line or "export_import.rs" in line or
        "merger.rs" in line or "config_handle.rs" in line or
        "error.rs" in line
    ):
        line = line.replace("| ? |", "| \u2705 |", 1)
        fixed += 1
    new_lines.append(line)

data = "\n".join(new_lines)
log(f"Fixed {fixed} CQ rows")

with open(tcr_path, "w", encoding="utf-8") as f:
    f.write(data)
log("TCR.md written")
log("Done.")
