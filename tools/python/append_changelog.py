import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"

new_entries = r"""
### CR-NR-045 -- Menu Workspace Pattern (Configurable ISPF-style option menus)
- **Date/Phase**: Phase CU (pre-gate)
- **Prompt**: "One of the features of ISPF is the ability to Customize it... we need a Solution where we can create menu workspaces that are a list of options. Each option is a number or set of characters (up to 4) Followed by a Command, and a description."
- **Description**: Introduce a Menu Workspace as a first-class pattern: a Workspace whose Context is a list of options loaded from a TOML config file (`menus/<name>.toml`). Each option has a key (1-4 chars), a command string, and a description. The POM becomes an instance of this pattern. Chained option paths (e.g. `=0.Themes`) are supported. A default `menus/pom.toml` is written on first launch. Hot-reload on file change. New sub-project `menu-workspace`.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/menu-workspace/requirements.md` (to be created -- Phase CU)

### CR-NR-046 -- Named Workspaces and Per-Workspace KEYS Command
- **Date/Phase**: Phase CX (pre-gate)
- **Prompt**: "Each Workspace should have its own name to allow menu option mapping as well as Function key mapping. A workspaces Function keys can be mapped at any time by invoking the KEYS command in the workspace."
- **Description**: Each Workspace gains a user-visible name string. The KEYS command is extended to accept an optional name argument (`KEYS <name>`) so the user can open the Key Configuration Dialog pre-loaded with any named key map, not just the current Workspace's map. The Key Configuration Dialog gains a `Map Name` field the user can change mid-session.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/function-keys-and-history/requirements.md` (extension to Req 20)

### CR-CH-010 -- SPLIT Command Alias for Workspace Detach
- **Date/Phase**: Phase CX (pre-gate)
- **Prompt**: "A Workspace should be detachable into its own Window... This serves as a replacement to the ISPF split command."
- **Description**: Add a `SPLIT` primary command as an ISPF-heritage alias for the existing Workspace detach operation (Ctrl+Shift+T / Move to Other View). Registers as Command_ID `layout.split`. Adds an explicit note in `layout-and-docking` that this replaces ISPF split-screen with modern OS window management.
- **Affects**: `docs/specs/layout-and-docking/requirements.md` Req 3; `ff-desktop` command handler
- **Status**: PENDING GATE

### CR-NR-047 -- Settings Context as a Menu Workspace
- **Date/Phase**: Phase CW (pre-gate, depends on CR-NR-045)
- **Prompt**: "The settings workspace should probably be a menu options workspace. A full list of available settings should be extracted from the requirements and a settings menu option created for it."
- **Description**: Restructure the Settings Context as a Menu Workspace whose options are loaded from `menus/settings.toml`. A default settings.toml is written on first launch with one option per major config namespace (Editor, Theme, Catalogs, VFS, Logging, Key Maps, Session, Plugins). Each option opens a sub-context showing only that namespace's keys using the existing flat-list widget. The flat-list view remains accessible as a sub-context.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/configuration-system/requirements.md` Req 15 (revision)

### CR-NR-048 -- POM Options Review and Logical Grouping
- **Date/Phase**: Phase CV (pre-gate, depends on CR-NR-045)
- **Prompt**: "We need a full review of the current options in the POM... perhaps we need to create menu options for those that can be grouped."
- **Description**: Review the current 9 POM options (0-8) against all implemented functionality. Revise the option list with logical grouping, adding entries for JES (job monitor), Search (global search), and Batch (batch execution) which currently have no POM entry. Define the default `menus/pom.toml` content. Update `startup-and-session` Req 14.3 accordingly.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Req 14.3 (revision)

### CR-NR-049 -- FFTest Context Inspection and Automatic Bug Logging
- **Date/Phase**: Phase CZ (pre-gate)
- **Prompt**: "The Automated Dialog testing capability would have to be able to examine the context of a workspace to verify that what expected to happen happened and also to write the finding to a Log file / Bug report. Bugs should be logged for repair."
- **Description**: Extend the FFTest framework with: (1) Workspace context inspection assertions (`ASSERT CONTEXT IS`, `ASSERT WORKSPACE COUNT IS`, `ASSERT OPTION EXISTS`); (2) automatic bug report generation -- on assertion failure the runner appends a structured entry to `reports/bugs-from-tests.md` in a format compatible with `docs/status/bugs.md`; (3) a full FFTest script suite covering all major functional areas (POM navigation, file ops, editor, catalog management, settings, key config, compiler, plugin manager, notification system, batch, global search, command palette). New requirements Reqs 11-13 in `automated-dialog-testing/requirements.md`.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/automated-dialog-testing/requirements.md` (new Reqs 11-13)
"""

with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

log(f"Reading: {path}")
with open(path, "rb") as f:
    data = f.read()
log(f"File size: {len(data)} bytes")

# Detect line ending used in file
if b"\r\n" in data:
    sep = b"\r\n"
    log("Line ending: CRLF")
else:
    sep = b"\n"
    log("Line ending: LF")

# Encode new entries using same line ending
encoded = new_entries.replace("\r\n", "\n").replace("\n", sep.decode()).encode("utf-8")

# Append (ensure file ends with newline before appending)
if not data.endswith(sep):
    data = data + sep

data = data + encoded

with open(path, "wb") as f:
    f.write(data)

log(f"New file size: {len(data)} bytes")
log("Done -- 6 new entries appended to change-log.md")
