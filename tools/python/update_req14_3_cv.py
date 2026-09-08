"""Update startup-and-session/requirements.md Req 14.3 with the Phase CV revised option list."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\startup-and-session\requirements.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# The old Req 14.3 block (try both CRLF and LF)
old_block_crlf = (
    b"3. THE Primary Option Menu SHALL display a numbered list of menu options, each with a short label and a one-line description. The built-in options SHALL be, at minimum:\r\n"
    b"   - `0 Settings` -- FFWB Settings and Client Parameters\r\n"
    b"   - `1 File Catalogs` -- Virtual File Catalogs -- Mainframe, POSIX, Native\r\n"
    b"   - `2 Files` -- File Explorer -- Browse catalogs and files in a tree view\r\n"
    b"   - `3 Utilities` -- Perform utility functions\r\n"
    b"   - `4 Compilers` -- Interactive language processing\r\n"
    b"   - `5 Lua Scripts` -- Run and manage Lua macros\r\n"
    b"   - `6 Terminals` -- Enter TSO or Workstation commands\r\n"
    b"   - `7 Databases` -- Database tool and query browser\r\n"
    b"   - `8 Plugins` -- Vendor added plugins\r\n"
    b"   [ISPF-POM]\r\n"
    b"   *(Note: Phase CV will revise this option list and migrate the POM to the Menu\r\n"
    b"   Workspace pattern backed by `menus/pom.toml`. See\r\n"
    b"   `docs/specs/menu-workspace/requirements.md` for the pattern definition.)*\r\n"
)

new_block_crlf = (
    b"3. THE Primary Option Menu SHALL display a numbered list of menu options, each with a short label and a one-line description. The built-in options SHALL be:\r\n"
    b"\r\n"
    b"   **Core options (0-8 -- unchanged from Phase AC):**\r\n"
    b"   - `0 Settings` -- FFWB Settings and Client Parameters\r\n"
    b"   - `1 File Catalogs` -- Virtual File Catalogs -- Mainframe, POSIX, Native\r\n"
    b"   - `2 Files` -- File Explorer -- Browse catalogs and files in a tree view\r\n"
    b"   - `3 Utilities` -- Perform utility functions\r\n"
    b"   - `4 Compilers` -- Interactive language processing\r\n"
    b"   - `5 Lua Scripts` -- Run and manage Lua macros\r\n"
    b"   - `6 Terminals` -- Enter TSO or Workstation commands\r\n"
    b"   - `7 Databases` -- Database tool and query browser\r\n"
    b"   - `8 Plugins` -- Vendor added plugins\r\n"
    b"\r\n"
    b"   **Extended options (added Phase CV):**\r\n"
    b"   - `9 Jobs` -- JES job monitor and spool viewer\r\n"
    b"   - `S Search` -- Global search and replace across files\r\n"
    b"   - `B Batch` -- Batch command execution (IKJEFT01 analogue)\r\n"
    b"\r\n"
    b"   [ISPF-POM]\r\n"
    b"   *(Phase CV revised this option list from 9 to 12 options, adding Jobs (9),\r\n"
    b"   Search (S), and Batch (B). The POM will be migrated to the Menu Workspace\r\n"
    b"   pattern backed by `menus/pom.toml` in Phase CU-impl. See\r\n"
    b"   `docs/specs/menu-workspace/requirements.md` and\r\n"
    b"   `docs/specs/menu-workspace/cv-requirements.md` for the pattern and\r\n"
    b"   content definitions.)*\r\n"
)

if old_block_crlf in data:
    log("Found old Req 14.3 block (CRLF)")
    data = data.replace(old_block_crlf, new_block_crlf, 1)
    with open(path, "wb") as f:
        f.write(data)
    log("Replacement written successfully")
else:
    log("ERROR: old Req 14.3 block not found with CRLF -- trying LF variant")
    old_block_lf = old_block_crlf.replace(b"\r\n", b"\n")
    new_block_lf = new_block_crlf.replace(b"\r\n", b"\n")
    if old_block_lf in data:
        log("Found old Req 14.3 block (LF)")
        data = data.replace(old_block_lf, new_block_lf, 1)
        with open(path, "wb") as f:
            f.write(data)
        log("Replacement written successfully (LF)")
    else:
        log("ERROR: pattern not found with either separator -- no change made")
        log("Dumping first 200 bytes around 'Phase CV' for diagnosis:")
        idx = data.find(b"Phase CV")
        if idx >= 0:
            log(repr(data[max(0,idx-100):idx+200]))
        sys.exit(1)

log("Done")
