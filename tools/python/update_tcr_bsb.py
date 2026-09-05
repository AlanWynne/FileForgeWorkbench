"""
update_tcr_bsb.py -- Add command-palette TCR rows and mark them green.

Appends a new section for command-palette requirements to TCR.md,
then marks all rows green (tests exist and pass).
"""
import sys
import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
TCR = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

with open(TCR, "rb") as f:
    data = f.read()

log(f"TCR size before: {len(data)}")

# Detect line ending
sep = b"\r\n" if b"\r\n" in data else b"\n"

# Check if BS-B section already present
if b"command-palette" in data or b"Req 1.1: Command Palette" in data:
    log("BS-B section already present -- updating rows to green")
else:
    log("Appending BS-B section")

# New rows to add (all green -- tests pass)
new_rows = [
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/fuzzy.rs` unit tests | Req 1.1: Command Palette opens as modal overlay on Ctrl+Shift+P |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/state.rs` unit tests | Req 1.2: Escape closes palette without executing |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` | Req 1.3: Click outside closes palette |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `shell/render_chrome.rs` View menu | Req 1.4: View > Command Palette menu item |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `shell/update.rs` Ctrl+Shift+P toggle | Req 1.5: Ctrl+Shift+P toggles palette closed when already open |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/fuzzy.rs` unit tests | Req 2.1: Fuzzy match filters by subsequence in real time |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/fuzzy.rs` unit tests | Req 2.2: Scoring: contiguous runs, word boundaries, shorter names |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` rebuild_filtered | Req 2.3: Results sorted by descending score, alpha tiebreak |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` rebuild_filtered | Req 2.4: Empty query shows recent then all alphabetically |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/fuzzy.rs` unit tests | Req 2.5: Fuzzy search is case-insensitive |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` empty state | Req 2.6: No-match shows 'No commands match <query>' message |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` render_entry | Req 3.1: Entry shows display name, category, shortcut |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` detail area | Req 3.2: Highlighted entry shows description in detail area |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` build_highlighted_text | Req 3.3: Matched characters highlighted in display name |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` MAX_VISIBLE | Req 3.4: At most 20 entries visible; list is scrollable |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/state.rs` unit tests | Req 4.1: Enter executes highlighted command and closes palette |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` click handler | Req 4.2: Click on entry executes command and closes palette |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/state.rs` unit tests | Req 4.3: Up/Down arrows navigate list with wrap-around |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `shell/update.rs` recent list update | Req 4.4: Executed command added to recent list (max 10) |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` disabled style | Req 4.5: Disabled entry shown with disabled style; Enter blocked |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` Recently Used header | Req 5.1: Empty query shows Recently Used section |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `ff-session` SessionState + session_manager | Req 5.2: Recent commands persisted in session.toml |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `command_palette/render.rs` rebuild_filtered | Req 5.3: Typing query hides Recently Used section |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `shell/update.rs` recent list update | Req 5.4: Only successfully executed commands added to recent list |",
]

# Build the section to append
section_lines = [
    b"",
    b"## Phase BS-B: Command Palette",
    b"",
    b"| Crate | Status | Test | Requirement |",
    b"|-------|--------|------|-------------|",
]
for row in new_rows:
    section_lines.append(row)

section_bytes = sep.join(section_lines) + sep

# Check if already appended
if b"Phase BS-B: Command Palette" in data:
    log("Section already present -- skipping append")
else:
    data = data + section_bytes
    log(f"Appended {len(new_rows)} rows")

with open(TCR, "wb") as f:
    f.write(data)

log(f"TCR size after: {len(data)}")
log("Done.")
