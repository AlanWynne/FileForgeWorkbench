import sys

LOG = r"tools\logs\fix-tab-state.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"crates\ff-desktop\src\tab_state.rs"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# Detect line ending
if b"\r\n" in data:
    sep = b"\r\n"
    log("Line ending: CRLF")
else:
    sep = b"\n"
    log("Line ending: LF")

# Count occurrences
ws_count = data.count(b"workspace_name: None,")
mw_count = data.count(b"menu_workspace: None,")
log(f"workspace_name: None count = {ws_count}")
log(f"menu_workspace: None count = {mw_count}")

# Pattern to replace: workspace_name: None,<sep>        }<sep>    }
# (end of a constructor body)
old = b"            workspace_name: None," + sep + b"        }" + sep + b"    }"
new = b"            workspace_name: None," + sep + b"            menu_workspace: None," + sep + b"        }" + sep + b"    }"

count = data.count(old)
log(f"Pattern found {count} times")

if count > 0:
    data = data.replace(old, new)
    with open(path, "wb") as f:
        f.write(data)
    log(f"Replaced {count} occurrences")
else:
    log("ERROR: pattern not found -- checking nearby bytes")
    # Try to find workspace_name: None, and show context
    idx = data.find(b"workspace_name: None,")
    if idx >= 0:
        log(f"Found at offset {idx}, context: {repr(data[idx:idx+60])}")
    else:
        log("workspace_name: None, not found at all")

log("Done")
