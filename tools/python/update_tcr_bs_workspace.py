"""
Update TCR.md Phase BS Workspace Model rows from NOT COVERED to PASS/MANUAL.
Binary-safe: handles BOM + CRLF.
"""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
TCR = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(TCR, "rb") as f:
    data = f.read()

log(f"TCR size before: {len(data)}")

# Detect line ending
sep = b"\r\n" if b"\r\n" in data else b"\n"

# Map of (old_bytes, new_bytes) replacements -- one per criterion row
replacements = [
    # Req 1.1
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 1.1: workspace saved as valid TOML with name, roots, settings, recent_files |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 1.1: workspace saved as valid TOML with name, roots, settings, recent_files |"),
    # Req 1.2
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 1.2: Workspace_File is valid TOML v1.0 |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 1.2: Workspace_File is valid TOML v1.0 |"),
    # Req 1.3
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 1.3: missing required field produces error, workspace not loaded |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 1.3: missing required field produces error, workspace not loaded |"),
    # Req 1.4
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 1.4: relative root paths resolved relative to Workspace_File directory |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 1.4: relative root paths resolved relative to Workspace_File directory |"),
    # Req 1.5
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 1.5: [settings] table applied as Workspace config layer |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 1.5: [settings] table applied as Workspace config layer |"),
    # Req 2.1
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 2.1: WORKSPACE OPEN loads file, registers roots, applies settings, restores MRU |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 2.1: WORKSPACE OPEN loads file, registers roots, applies settings, restores MRU |"),
    # Req 2.2
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 2.2: WORKSPACE SAVE writes current state to Workspace_File |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/commands.rs` | Req 2.2: WORKSPACE SAVE writes current state to Workspace_File |"),
    # Req 2.3
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 2.3: WORKSPACE SAVE AS writes to specified path |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/commands.rs` | Req 2.3: WORKSPACE SAVE AS writes to specified path |"),
    # Req 2.4
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 2.4: WORKSPACE CLOSE unloads roots, settings layer, MRU list |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 2.4: WORKSPACE CLOSE unloads roots, settings layer, MRU list |"),
    # Req 2.5
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 2.5: opening workspace when one is active closes current first; prompts if unsaved |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 2.5: opening workspace when one is active closes current first; prompts if unsaved |"),
    # Req 2.6
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 2.6: at most one workspace active at any time |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/mod.rs` | Req 2.6: at most one workspace active at any time |"),
    # Req 3.1
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.1: WORKSPACE ADD ROOT registers new catalog mount point |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/commands.rs` | Req 3.1: WORKSPACE ADD ROOT registers new catalog mount point |"),
    # Req 3.2
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.2: WORKSPACE REMOVE ROOT unregisters catalog; open tabs show warning |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/commands.rs` | Req 3.2: WORKSPACE REMOVE ROOT unregisters catalog; open tabs show warning |"),
    # Req 3.4
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.4: workspace load auto-registers all roots as Native catalogs |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 3.4: workspace load auto-registers all roots as Native catalogs |"),
    # Req 3.5
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.5: missing root path at load shows status bar warning; remaining roots loaded |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 3.5: missing root path at load shows status bar warning; remaining roots loaded |"),
    # Req 4.1
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 4.1: workspace [settings] applied as highest-priority config layer |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 4.1: workspace [settings] applied as highest-priority config layer |"),
    # Req 4.3
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 4.3: workspace close removes Workspace layer; hot-reload callbacks invoked |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 4.3: workspace close removes Workspace layer; hot-reload callbacks invoked |"),
    # Req 5.1
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 5.1: active_workspace_path persisted in session.toml on exit |",
     b"| `ff-session` | \xe2\x9c\x85 | `session_state.rs` unit tests | Req 5.1: active_workspace_path persisted in session.toml on exit |"),
    # Req 5.2
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 5.2: workspace auto-loaded at startup from persisted path |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/update.rs` startup_tests | Req 5.2: workspace auto-loaded at startup from persisted path |"),
    # Req 5.3
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 5.3: missing persisted path starts without workspace; stale path cleared |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/update.rs` startup_tests | Req 5.3: missing persisted path starts without workspace; stale path cleared |"),
    # Req 6.1
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 6.1: workspace MRU list accumulates files opened while workspace active |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 6.1: workspace MRU list accumulates files opened while workspace active |"),
    # Req 6.2
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 6.2: MRU list persisted in Workspace_File [[recent_files]] |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 6.2: MRU list persisted in Workspace_File [[recent_files]] |"),
    # Req 6.3
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 6.3: workspace close reverts to global recent-files list |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 6.3: workspace close reverts to global recent-files list |"),
    # Req 6.4
    (b"| `ff-session` | \xf0\x9f\x94\xb4 | -- | Req 6.4: workspace MRU depth configurable via workspace.recent_files_depth |",
     b"| `ff-session` | \xe2\x9c\x85 | `workspace.rs` unit tests | Req 6.4: workspace MRU depth configurable via workspace.recent_files_depth |"),
]

changed = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        changed += 1
        log(f"  replaced: {old[30:70]!r}")
    else:
        log(f"  NOT FOUND: {old[30:70]!r}")

with open(TCR, "wb") as f:
    f.write(data)

log(f"TCR size after: {len(data)}")
log(f"Rows updated: {changed}/24")
log("Done.")
