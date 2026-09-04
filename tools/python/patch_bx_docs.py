import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\bx_docs.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

log("=== Phase BX doc update ===")

# --- 1. Update TCR.md: mark all 12 BX rows as covered ---
tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(tcr_path, "r", encoding="utf-8") as f:
    tcr = f.read()

replacements = [
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.1: O overlays target line(s) with source content, non-blank chars only |",
     "| `ff-line-commands` | \u2705 | `execution/overlay.rs` unit tests | Req 15.1: O overlays target line(s) with source content, non-blank chars only |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.2: On overlays n consecutive lines with source content |",
     "| `ff-line-commands` | \u2705 | `execution/overlay.rs` unit tests | Req 15.2: On overlays n consecutive lines with source content |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.3: W copies single line content to system clipboard |",
     "| `ff-line-commands` | \u2705 | `execution/clipboard_copy.rs` unit tests | Req 15.3: W copies single line content to system clipboard |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.4: WW copies block of lines to system clipboard |",
     "| `ff-line-commands` | \u2705 | `execution/clipboard_copy.rs` unit tests | Req 15.4: WW copies block of lines to system clipboard |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.5: F shows (un-excludes) only the first line of an excluded block |",
     "| `ff-line-commands` | \u2705 | `execution/show_excluded.rs` unit tests | Req 15.5: F shows (un-excludes) only the first line of an excluded block |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.6: L shows (un-excludes) only the last line of an excluded block |",
     "| `ff-line-commands` | \u2705 | `execution/show_excluded.rs` unit tests | Req 15.6: L shows (un-excludes) only the last line of an excluded block |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.7: ] shifts single line right by exactly one column |",
     "| `ff-line-commands` | \u2705 | `parser.rs` + `resolution.rs` unit tests | Req 15.7: ] shifts single line right by exactly one column |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.8: ]] shifts block of lines right by exactly one column |",
     "| `ff-line-commands` | \u2705 | `parser.rs` + `resolution.rs` unit tests | Req 15.8: ]] shifts block of lines right by exactly one column |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.9: S shows (un-excludes) first line of excluded block at that position |",
     "| `ff-line-commands` | \u2705 | `execution/show_excluded.rs` unit tests | Req 15.9: S shows (un-excludes) first line of excluded block at that position |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.10: overlay operation (O/On) produces a single undoable Transaction |",
     "| `ff-line-commands` | \u2705 | `execution/overlay.rs` unit tests | Req 15.10: overlay operation (O/On) produces a single undoable Transaction |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.11: clipboard copy (W/WW) produces no Transaction |",
     "| `ff-line-commands` | \u2705 | `execution/clipboard_copy.rs` unit tests | Req 15.11: clipboard copy (W/WW) produces no Transaction |"),
    ("| `ff-line-commands` | \U0001f534 | -- | Req 15.12: F, L, S produce no Transaction (session state only) |",
     "| `ff-line-commands` | \u2705 | `execution/show_excluded.rs` unit tests | Req 15.12: F, L, S produce no Transaction (session state only) |"),
]

count = 0
for old, new in replacements:
    if old in tcr:
        tcr = tcr.replace(old, new, 1)
        count += 1
        log(f"  TCR updated row {count}")
    else:
        log(f"  WARNING: row {count+1} not found")

with open(tcr_path, "w", encoding="utf-8") as f:
    f.write(tcr)
log(f"TCR: {count}/12 rows updated")

# --- 2. Update project-master/tasks.md: mark BX tasks done ---
pm_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
with open(pm_path, "r", encoding="utf-8") as f:
    pm = pm_path and open(pm_path, "r", encoding="utf-8").read()

pm_replacements = [
    ("- [ ] BX.1 Overlay line command (O, On) -- LineCommandKind variants, parser, execution (Tasks 22.1-22.6)",
     "- [x] BX.1 Overlay line command (O, On) -- LineCommandKind variants, parser, execution (Tasks 22.1-22.6)"),
    ("- [ ] BX.2 Clipboard copy line command (W, WW) -- variants, parser, ff-clipboard integration (Tasks 23.1-23.6)",
     "- [x] BX.2 Clipboard copy line command (W, WW) -- variants, parser, ff-clipboard integration (Tasks 23.1-23.6)"),
    ("- [ ] BX.3 First-of-excluded (F) -- ShowFirst variant, parser, execution (Tasks 24.1-24.6)",
     "- [x] BX.3 First-of-excluded (F) -- ShowFirst variant, parser, execution (Tasks 24.1-24.6)"),
    ("- [ ] BX.4 Last-of-excluded (L) -- ShowLast variant, parser, execution (Tasks 25.1-25.6)",
     "- [x] BX.4 Last-of-excluded (L) -- ShowLast variant, parser, execution (Tasks 25.1-25.6)"),
    ("- [ ] BX.5 Single-column shift right (]) -- ShiftRightOne variant, parser, delegate to shift_right(1) (Tasks 26.1-26.6)",
     "- [x] BX.5 Single-column shift right (]) -- ShiftRightOne variant, parser, delegate to shift_right(1) (Tasks 26.1-26.6)"),
    ("- [ ] BX.6 Show-excluded (S) -- ShowLine variant, parser, execution (Tasks 27.1-27.6)",
     "- [x] BX.6 Show-excluded (S) -- ShowLine variant, parser, execution (Tasks 27.1-27.6)"),
    ("- [ ] BX.7 TCR update for Requirement 15 (Task 28.1)",
     "- [x] BX.7 TCR update for Requirement 15 (Task 28.1)"),
    ("- [ ] BX.impl line-commands: O/W/F/L/]/S line commands (Tasks 22-28 in line-commands/tasks.md)",
     "- [x] BX.impl line-commands: O/W/F/L/]/S line commands (Tasks 22-28 in line-commands/tasks.md)"),
]

with open(pm_path, "r", encoding="utf-8") as f:
    pm = f.read()

count2 = 0
for old, new in pm_replacements:
    if old in pm:
        pm = pm.replace(old, new, 1)
        count2 += 1
        log(f"  PM updated: {old[:60]}...")
    else:
        log(f"  WARNING PM not found: {old[:60]}...")

with open(pm_path, "w", encoding="utf-8") as f:
    f.write(pm)
log(f"Project-master: {count2}/8 entries updated")

# --- 3. Update line-commands/tasks.md: mark Tasks 22-28 done ---
lc_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\line-commands\tasks.md"
with open(lc_path, "r", encoding="utf-8") as f:
    lc = f.read()

# Mark all sub-tasks under tasks 22-28 as done
import re
# Replace all "- [ ]" within the Phase BX section
bx_start = lc.find("## Phase BX")
if bx_start == -1:
    log("WARNING: Phase BX section not found in tasks.md")
else:
    bx_section = lc[bx_start:]
    bx_updated = bx_section.replace("- [ ]", "- [x]")
    lc = lc[:bx_start] + bx_updated
    with open(lc_path, "w", encoding="utf-8") as f:
        f.write(lc)
    log("line-commands/tasks.md: Phase BX tasks marked [x]")

log("=== Done ===")
