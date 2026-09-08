LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\ct4_plugin_notif_fix.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

files_replacements = {
    r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\plugin-manager-ui\requirements.md": [
        (b"Plugin Manager panel -- a dedicated UI for", b"Plugin Manager Context -- a dedicated Workspace for"),
        (b"| Plugin Manager panel | The egui panel rendered when POM option 8 is selected |",
         b"| Plugin Manager Context | The Workspace rendered when POM option 8 is selected |"),
        (b"## Requirement 1: Plugin Manager Panel", b"## Requirement 1: Plugin Manager Context"),
        (b"I want a Plugin Manager panel", b"I want a Plugin Manager Context"),
        (b"THE workbench SHALL open a `PluginManagerPanel` tab.", b"THE workbench SHALL open a `PluginManagerPanel` Workspace."),
        (b"2. THE Plugin Manager panel SHALL display", b"2. THE Plugin Manager Context SHALL display"),
        (b"3. FOR EACH plugin in the list, THE panel SHALL display:", b"3. FOR EACH plugin in the list, THE Plugin Manager Context SHALL display:"),
        (b"4. WHEN a plugin is in the Failed state, THE panel SHALL display the", b"4. WHEN a plugin is in the Failed state, THE Plugin Manager Context SHALL display the"),
        (b"6. THE panel SHALL include a filter text field", b"6. THE Plugin Manager Context SHALL include a filter text field"),
        (b"1. WHEN a plugin is Active, THE panel SHALL display a `Disable` button", b"1. WHEN a plugin is Active, THE Plugin Manager Context SHALL display a `Disable` button"),
        (b"3. WHEN a plugin is Inactive, THE panel SHALL display an `Enable` button", b"3. WHEN a plugin is Inactive, THE Plugin Manager Context SHALL display an `Enable` button"),
        (b"failure reason in the panel and set the plugin state to Failed --", b"failure reason in the Plugin Manager Context and set the plugin state to Failed --"),
        (b"1. WHEN the user selects a plugin in the list, THE panel SHALL display", b"1. WHEN the user selects a plugin in the list, THE Plugin Manager Context SHALL display"),
        (b"display each key with its current value and a link to the Settings\n   panel filtered to that plugin", b"display each key with its current value and a link to the Settings\n   Context filtered to that plugin"),
        (b"display each key with its current value and a link to the Settings\r\n   panel filtered to that plugin", b"display each key with its current value and a link to the Settings\r\n   Context filtered to that plugin"),
        (b"I want the Plugin Manager panel", b"I want the Plugin Manager Context"),
        (b"**Source:** Consistent with session persistence requirements across all panels.", b"**Source:** Consistent with session persistence requirements across all Workspaces."),
        (b"2. WHEN the workbench starts and restores a PluginManagerPanel tab,\n   THE panel SHALL reload", b"2. WHEN the workbench starts and restores a PluginManagerPanel Workspace,\n   THE Plugin Manager Context SHALL reload"),
        (b"2. WHEN the workbench starts and restores a PluginManagerPanel tab,\r\n   THE panel SHALL reload", b"2. WHEN the workbench starts and restores a PluginManagerPanel Workspace,\r\n   THE Plugin Manager Context SHALL reload"),
    ],
    r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\notification-system\requirements.md": [
        (b"| Event log | A persistent, scrollable panel showing all past notifications |",
         b"| Event log | A persistent, scrollable Event Log Context showing all past notifications |"),
        (b"Event Log panel.", b"Event Log Context."),
        (b"## Requirement 2: Event Log Panel", b"## Requirement 2: Event Log Context"),
        (b"an `EventLogPanel` tab.", b"an `EventLogPanel` Workspace."),
        (b"2. THE Event Log panel SHALL display", b"2. THE Event Log Context SHALL display"),
        (b"3. FOR EACH log entry, THE panel SHALL display:", b"3. FOR EACH log entry, THE Event Log Context SHALL display:"),
        (b"4. THE panel SHALL include a filter by level", b"4. THE Event Log Context SHALL include a filter by level"),
        (b"5. WHEN the user selects a log entry, THE panel SHALL display the full", b"5. WHEN the user selects a log entry, THE Event Log Context SHALL display the full"),
        (b"6. THE panel SHALL include a `Clear Log` button", b"6. THE Event Log Context SHALL include a `Clear Log` button"),
        (b"Event Log panel and mark all notifications as read.", b"Event Log Context and mark all notifications as read."),
    ],
}

for path, replacements in files_replacements.items():
    with open(path, "rb") as f:
        data = f.read()
    log(f"\n--- {path} ({len(data)} bytes) ---")
    count = 0
    for old, new in replacements:
        if old in data:
            data = data.replace(old, new)
            log(f"  Replaced: {old[:60]!r}")
            count += 1
        else:
            log(f"  NOT FOUND: {old[:60]!r}")
    with open(path, "wb") as f:
        f.write(data)
    log(f"  {count} replacements made.")

log("\nAll done.")
