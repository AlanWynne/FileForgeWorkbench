"""
fix_stale_pending.py -- Fix stale [ ] markers in two tasks.md files.

viewport-and-scrolling: task 16 header is [ ] but all sub-tasks are [x] -- mark header [x].
virtual-catalog-manager: task 17 is explicitly SUPERSEDED by BU.7 -- mark all [ ] as [x].
"""

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\fix_stale_pending.txt"

with open(LOG, "w", encoding="utf-8") as f:
    f.write("=== fix_stale_pending.py ===\n\n")

def log(msg):
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

def patch_file(path, description, replacements):
    log("--- " + description + " ---")
    with open(path, "rb") as f:
        data = f.read()
    log("File size: " + str(len(data)) + " bytes")
    changed = 0
    for old_bytes, new_bytes, note in replacements:
        if old_bytes in data:
            data = data.replace(old_bytes, new_bytes, 1)
            log("  REPLACED: " + note)
            changed += 1
        else:
            log("  NOT FOUND: " + note)
    if changed > 0:
        with open(path, "wb") as f:
            f.write(data)
        log("  Written " + str(changed) + " replacement(s).")
    else:
        log("  No changes made.")
    log("")

# --- viewport-and-scrolling: fix task 16 header ---
# The header line is "- [ ] 16. Editor scroll amount integration"
# All sub-tasks are [x] so the header should be [x] too.
# Try both CRLF and LF variants.

vp_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\viewport-and-scrolling\tasks.md"

patch_file(vp_path, "viewport-and-scrolling task 16 header", [
    (
        b"- [ ] 16. Editor scroll amount integration (Requirement 14, CR-NR-035)\r\n",
        b"- [x] 16. Editor scroll amount integration (Requirement 14, CR-NR-035)\r\n",
        "task 16 header CRLF"
    ),
    (
        b"- [ ] 16. Editor scroll amount integration (Requirement 14, CR-NR-035)\n",
        b"- [x] 16. Editor scroll amount integration (Requirement 14, CR-NR-035)\n",
        "task 16 header LF"
    ),
])

# --- virtual-catalog-manager: fix task 17 (SUPERSEDED by BU.7) ---
# Replace the task 17 header and all 8 sub-task lines from [ ] to [x].
# The task note already says SUPERSEDED -- we just update the checkbox state.

vcm_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\virtual-catalog-manager\tasks.md"

# Build list of lines to fix -- each sub-task line, try CRLF then LF
lines_to_fix = [
    b"- [ ] 17. Dataset file creation on first open (Req 16.3, 16.6)",
    b"  - [ ] 17.1 Write failing test `opening_missing_dataset_creates_file_and_parent_dirs`",
    b"  - [ ] 17.2 Write failing test `opening_missing_dataset_creates_parent_dirs`",
    b"  - [ ] 17.3 Add `create_dataset_file(path: &Path) -> Result<(), std::io::Error>`",
    b"  - [ ] 17.4 In `render.rs` `FilesPanelAction::OpenFile` Mainframe handler:",
    b"  - [ ] 17.5 In `file_explorer_panel.rs` `render_dataset_children()` double-click handler:",
    b"  - [ ] 17.6 Run `cargo test -p ff-desktop`",
    b"  - [ ] 17.7 Run `cargo clippy -p ff-desktop -- -D warnings`",
    b"  - [ ] 17.8 Update `docs/quality/TCR.md`",
]

with open(vcm_path, "rb") as f:
    data = f.read()
log("--- virtual-catalog-manager task 17 (SUPERSEDED) ---")
log("File size: " + str(len(data)) + " bytes")

changed = 0
for prefix in lines_to_fix:
    old = prefix.replace(b"- [ ]", b"- [ ]")  # identity -- just to be explicit
    new = prefix.replace(b"- [ ]", b"- [x]")
    # find the line in data (prefix match, ignore rest of line)
    idx = data.find(old)
    if idx != -1:
        data = data[:idx] + new + data[idx + len(old):]
        log("  REPLACED: " + prefix[:60].decode("utf-8", errors="replace"))
        changed += 1
    else:
        log("  NOT FOUND: " + prefix[:60].decode("utf-8", errors="replace"))

if changed > 0:
    with open(vcm_path, "wb") as f:
        f.write(data)
    log("  Written " + str(changed) + " replacement(s).")
else:
    log("  No changes made.")

log("")
log("Done.")
