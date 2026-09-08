"""Update configuration-system/requirements.md Req 15 with Phase CW note."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\configuration-system\requirements.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# Locate the Req 15 heading and insert the Phase CW note after the Source line
# We target the Source line and the blank line after it, inserting the note before "#### Acceptance Criteria"

for sep in (b"\r\n", b"\n"):
    old = (
        b"**Source:** [WB] Configuration as Data; [ISPF-POM] POM option 0." + sep +
        sep +
        b"#### Acceptance Criteria" + sep +
        sep +
        b"1. WHEN the user selects option `0` from the Home Context (Primary Option Menu), OR types `0` or `SETTINGS`" + sep +
        b"     or `=0` in any `Command ===>` field, THE shell SHALL open the Settings Context as a new Workspace" + sep +
        b"     with title `[SETTINGS]` and tab kind `SettingsPanel`." + sep +
        sep +
        b"2. THE Settings Context SHALL display all configuration keys registered in the `ff-config`" + sep +
        b"     schema, grouped by namespace (e.g., `Editor`, `Logging`, `Theme`, `Catalogs`, `VFS`)," + sep +
        b"     with each group rendered as a collapsible section."
    )
    if old in data:
        log(f"Found Req 15 block with sep {repr(sep)}")
        new = (
            b"**Source:** [WB] Configuration as Data; [ISPF-POM] POM option 0." + sep +
            sep +
            b"*(Phase CW restructures the Settings Context as a two-level Menu Workspace. The primary entry" + sep +
            b"point becomes a Settings_Menu (namespace selector) backed by `menus/settings.toml`. Each" + sep +
            b"namespace option opens a Settings_Namespace_View (filtered flat list). The migration from the" + sep +
            b"current flat-list implementation happens in Phase CW-impl. See" + sep +
            b"`docs/specs/menu-workspace/cw-requirements.md` for the full definition.)*" + sep +
            sep +
            b"#### Acceptance Criteria" + sep +
            sep +
            b"1. WHEN the user selects option `0` from the Home Context (Primary Option Menu), OR types `0` or `SETTINGS`" + sep +
            b"     or `=0` in any `Command ===>` field, THE shell SHALL open the Settings Context as a new Workspace." + sep +
            b"     After Phase CW-impl, this opens the Settings_Menu (namespace selector). Until then, it opens" + sep +
            b"     the flat-list Settings panel with title `[SETTINGS]` and tab kind `SettingsPanel`." + sep +
            sep +
            b"2. THE Settings_Namespace_View (opened from the Settings_Menu) SHALL display all configuration" + sep +
            b"     keys for the selected namespace, grouped and rendered as a collapsible section. The" + sep +
            b"     unfiltered flat-list view (option `A` in the Settings_Menu) SHALL display all namespaces."
        )
        data = data.replace(old, new, 1)
        with open(path, "wb") as f:
            f.write(data)
        log("Req 15 updated successfully")
        break
else:
    log("ERROR: Req 15 block not found with either separator")
    idx = data.find(b"Requirement 15")
    if idx >= 0:
        log(repr(data[idx:idx+500]))
    sys.exit(1)

# Also update criterion 9 (session persistence note) and criterion 10 (F3/END behaviour)
for sep in (b"\r\n", b"\n"):
    old9 = (
        b"9. THE `[SETTINGS]` Workspace SHALL persist in the session and be restored on next launch as a" + sep +
        b"     `SettingsPanel` tab kind." + sep +
        sep +
        b"10. WHEN the user presses `F3` or types `END` in the Settings Context command field," + sep +
        b"      THE shell SHALL return the Workspace to the Home Context (Primary Option Menu) view."
    )
    if old9 in data:
        log(f"Found criteria 9-10 block with sep {repr(sep)}")
        new9 = (
            b"9. THE `[SETTINGS]` Workspace SHALL persist in the session and be restored on next launch as a" + sep +
            b"     `SettingsPanel` tab kind. After Phase CW-impl, a Settings_Namespace_View tab SHALL persist" + sep +
            b"     with its namespace filter and be restored as a `SettingsPanel` tab kind with that filter." + sep +
            sep +
            b"10. WHEN the user presses `F3` or types `END` in a Settings_Namespace_View command field," + sep +
            b"      THE shell SHALL return the Workspace to the Settings_Menu. WHEN the user presses `F3` or" + sep +
            b"      types `END` in the Settings_Menu command field, THE shell SHALL return the Workspace to" + sep +
            b"      the Home Context (Primary Option Menu) view."
        )
        data = data.replace(old9, new9, 1)
        with open(path, "wb") as f:
            f.write(data)
        log("Criteria 9-10 updated successfully")
        break
else:
    log("WARNING: criteria 9-10 block not found -- may already be updated or pattern mismatch")

log("Done")
