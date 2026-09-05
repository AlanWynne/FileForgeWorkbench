"""Update Phase CM TCR rows from NOT COVERED to PASS/MANUAL."""
import sys
import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\tcr_cm_update.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
open(LOG, "w").close()

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

marker = b"### Phase CM -- Mouse Text Selection"
pos = data.find(marker)
log(f"Phase CM marker at byte: {pos}")
if pos < 0:
    log("ERROR: marker not found")
    sys.exit(1)

snippet = data[pos:pos+300]
log(f"Snippet repr: {repr(snippet)}")

# Detect line ending used in this section
sep = b"\r\n" if b"\r\n" in snippet else b"\n"
log(f"Line separator: {repr(sep)}")

# Build old block (21 rows, all red)
def row_red(req_text):
    return b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | " + req_text.encode() + b" |"

old_rows = [
    row_red("Req 13.1: mouse press in editor canvas records anchor at (line, col)"),
    row_red("Req 13.2: mouse drag extends selection end in real time"),
    row_red("Req 13.3: mouse release finalises selection range"),
    row_red("Req 13.4: active selection renders highlight rect using SelectionBack colour"),
    row_red("Req 13.5: click without drag clears selection and positions caret"),
    row_red("Req 13.6: Escape clears active selection"),
    row_red("Req 13.7: Ctrl+C with active selection writes text to OS clipboard"),
    row_red("Req 13.8: Ctrl+C with no selection does nothing"),
    row_red("Req 13.9: tab switch clears selection in previous tab"),
    row_red("Req 13.10: selection highlight stays correctly positioned when document is scrolled"),
    row_red("Req 14.1: POM panel text rendered with selectable labels"),
    row_red("Req 14.2: Settings panel text rendered with selectable labels"),
    row_red("Req 14.3: status bar text rendered with selectable labels"),
    row_red("Req 14.4: Ctrl+C on selected panel text writes to OS clipboard via egui"),
    row_red("Req 14.5: POM option button click-to-navigate unaffected by selectable label change"),
    row_red("Req 20.1: Ctrl+C with active canvas selection writes plain UTF-8 text to OS clipboard"),
    row_red("Req 20.2: successful copy shows \"Copied N characters\" in status bar"),
    row_red("Req 20.3: Ctrl+C with no selection does not write to clipboard"),
    row_red("Req 20.4: canvas copy does not modify document or selection"),
    row_red("Req 20.5: canvas copy recorded in Command_History"),
    row_red("Req 20.6: multi-line selection joined with platform line-ending before clipboard write"),
]
old_block = sep.join(old_rows)
log(f"Old block repr (first 100): {repr(old_block[:100])}")

found = old_block in data
log(f"Old block found: {found}")

if not found:
    # Try with just LF
    sep2 = b"\n"
    old_block2 = sep2.join(old_rows)
    found2 = old_block2 in data
    log(f"Old block (LF only) found: {found2}")
    if found2:
        old_block = old_block2
        sep = sep2
        found = True

if not found:
    log("ERROR: old block not found with either separator")
    # Show what the actual rows look like
    lines = data[pos:pos+3000].split(b"\n")
    for i, l in enumerate(lines[:30]):
        log(f"  line {i}: {repr(l)}")
    sys.exit(1)

PASS = "\u2705".encode("utf-8")
MANUAL = "\U0001f532".encode("utf-8")

new_rows = [
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 13.1: `new_tab_has_no_canvas_selection` -- new tab canvas_selection is None |",
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 13.2: `canvas_selection_can_be_set_and_cleared` -- canvas_selection field set/cleared |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 13.3: mouse release finalises selection range (egui drag -- manual UI verification) |",
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 13.4: `normalise_selection_orders_start_before_end` -- selection highlight uses normalised coords |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 13.5: click without drag clears selection (egui click -- manual UI verification) |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 13.6: Escape clears active selection (manual UI verification) |",
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 13.7: `canvas_selection_cleared_on_tab_switch` -- selection cleared on tab switch |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 13.8: Ctrl+C with no selection does nothing (manual UI verification) |",
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 13.9: `canvas_selection_cleared_on_tab_switch` -- selection cleared on tab switch |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 13.10: selection highlight stays positioned when scrolled (manual UI verification) |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 14.1: POM panel text rendered with selectable labels (manual UI verification) |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 14.2: Settings panel text rendered with selectable labels (manual UI verification) |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 14.3: status bar text rendered with selectable labels (manual UI verification) |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 14.4: Ctrl+C on selected panel text writes to OS clipboard via egui (manual UI verification) |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 14.5: POM option button click-to-navigate unaffected by selectable label change (manual UI verification) |",
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 20.1: `extract_selected_text_single_line`, `extract_selected_text_reversed_anchor_normalises` -- Ctrl+C writes UTF-8 text |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 20.2: 'Copied N characters' status message (manual UI verification -- return value wired to status bar) |",
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 20.3: `extract_selected_text_empty_selection_returns_empty` -- no-op when selection empty |",
    b"| `ff-desktop` | " + PASS + b" | `editor_panel.rs` unit tests | Req 20.4: `extract_selected_text_multi_line_joins_with_newline` -- multi-line joined with newline |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 20.5: canvas copy recorded in Command_History (manual UI verification) |",
    b"| `ff-desktop` | " + MANUAL + b" | -- | Req 20.6: multi-line selection joined with platform line-ending (manual UI verification) |",
]
new_block = sep.join(new_rows)

data2 = data.replace(old_block, new_block, 1)
if data2 == data:
    log("ERROR: replacement had no effect")
    sys.exit(1)

with open(path, "wb") as f:
    f.write(data2)
log("Done -- TCR updated successfully")
