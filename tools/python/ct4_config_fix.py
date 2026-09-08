import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\ct4_config_fix.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\configuration-system\requirements.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

replacements = [
    (
        b"### Requirement 15: Settings Panel",
        b"### Requirement 15: Settings Context"
    ),
    (
        b"I want a graphical Settings panel that lets me view and",
        b"I want a graphical Settings Context that lets me view and"
    ),
    (
        b"from the Primary Option Menu, OR types `0` or `SETTINGS`\n     or `=0` in any `Command ===>` field, THE shell SHALL open the Settings panel as a new tab",
        b"from the Home Context (Primary Option Menu), OR types `0` or `SETTINGS`\n     or `=0` in any `Command ===>` field, THE shell SHALL open the Settings Context as a new Workspace"
    ),
    (
        b"from the Primary Option Menu, OR types `0` or `SETTINGS`\r\n     or `=0` in any `Command ===>` field, THE shell SHALL open the Settings panel as a new tab",
        b"from the Home Context (Primary Option Menu), OR types `0` or `SETTINGS`\r\n     or `=0` in any `Command ===>` field, THE shell SHALL open the Settings Context as a new Workspace"
    ),
    (
        b"2. THE Settings panel SHALL display all configuration keys",
        b"2. THE Settings Context SHALL display all configuration keys"
    ),
    (
        b"3. FOR each configuration key, THE Settings panel SHALL display:",
        b"3. FOR each configuration key, THE Settings Context SHALL display:"
    ),
    (
        b"4. WHEN the user changes a value in the Settings panel and confirms",
        b"4. WHEN the user changes a value in the Settings Context and confirms"
    ),
    (
        b"5. WHEN a value fails schema validation (out of range, not in allowed set, fails regex),\n     THE Settings panel SHALL display",
        b"5. WHEN a value fails schema validation (out of range, not in allowed set, fails regex),\n     THE Settings Context SHALL display"
    ),
    (
        b"5. WHEN a value fails schema validation (out of range, not in allowed set, fails regex),\r\n     THE Settings panel SHALL display",
        b"5. WHEN a value fails schema validation (out of range, not in allowed set, fails regex),\r\n     THE Settings Context SHALL display"
    ),
    (
        b"6. THE Settings panel SHALL display a `Reset to Default` button",
        b"6. THE Settings Context SHALL display a `Reset to Default` button"
    ),
    (
        b"7. THE Settings panel SHALL include a search/filter input",
        b"7. THE Settings Context SHALL include a search/filter input"
    ),
    (
        b"in the filter, THE panel SHALL show only keys",
        b"in the filter, THE Settings Context SHALL show only keys"
    ),
    (
        b"8. THE Settings panel SHALL display a read-only `Source File`",
        b"8. THE Settings Context SHALL display a read-only `Source File`"
    ),
    (
        b"9. THE `[SETTINGS]` tab SHALL persist",
        b"9. THE `[SETTINGS]` Workspace SHALL persist"
    ),
    (
        b"10. WHEN the user presses `F3` or types `END` in the Settings panel command field,\n      THE shell SHALL return the tab to the Primary Option Menu view.",
        b"10. WHEN the user presses `F3` or types `END` in the Settings Context command field,\n      THE shell SHALL return the Workspace to the Home Context (Primary Option Menu) view."
    ),
    (
        b"10. WHEN the user presses `F3` or types `END` in the Settings panel command field,\r\n      THE shell SHALL return the tab to the Primary Option Menu view.",
        b"10. WHEN the user presses `F3` or types `END` in the Settings Context command field,\r\n      THE shell SHALL return the Workspace to the Home Context (Primary Option Menu) view."
    ),
    (
        b"11. WHEN the user clicks `Settings` in the POM option list (option 0 button), THE shell\n      SHALL navigate to the Settings panel using",
        b"11. WHEN the user clicks `Settings` in the POM option list (option 0 button), THE shell\n      SHALL navigate to the Settings Context using"
    ),
    (
        b"11. WHEN the user clicks `Settings` in the POM option list (option 0 button), THE shell\r\n      SHALL navigate to the Settings panel using",
        b"11. WHEN the user clicks `Settings` in the POM option list (option 0 button), THE shell\r\n      SHALL navigate to the Settings Context using"
    ),
    (
        b"ConfigHandle so that the Settings panel and other consumers",
        b"ConfigHandle so that the Settings Context and other consumers"
    ),
    (
        b"5. THE Configuration_System SHALL expose an `is_locked(key: &str) -> bool` method on\n   ConfigHandle so that the Settings panel and other consumers can check lock status before\n   attempting writes.\n6. THE Settings panel SHALL display a lock indicator",
        b"5. THE Configuration_System SHALL expose an `is_locked(key: &str) -> bool` method on\n   ConfigHandle so that the Settings Context and other consumers can check lock status before\n   attempting writes.\n6. THE Settings Context SHALL display a lock indicator"
    ),
    (
        b"5. THE Configuration_System SHALL expose an `is_locked(key: &str) -> bool` method on\r\n   ConfigHandle so that the Settings panel and other consumers can check lock status before\r\n   attempting writes.\r\n6. THE Settings panel SHALL display a lock indicator",
        b"5. THE Configuration_System SHALL expose an `is_locked(key: &str) -> bool` method on\r\n   ConfigHandle so that the Settings Context and other consumers can check lock status before\r\n   attempting writes.\r\n6. THE Settings Context SHALL display a lock indicator"
    ),
    # Fix arrow characters
    (b"Boolean \xe2\x86\x92 checkbox", b"Boolean -> checkbox"),
    (b"Integer / Float with min/max \xe2\x86\x92 slider; without constraints \xe2\x86\x92 numeric text field", b"Integer / Float with min/max -> slider; without constraints -> numeric text field"),
    (b"String with `allowed_values` \xe2\x86\x92 drop-down selector", b"String with `allowed_values` -> drop-down selector"),
    (b"String without constraints \xe2\x86\x92 single-line text field", b"String without constraints -> single-line text field"),
    # Fix em dash in heading
    (b"Settings Panel \xe2\x80\x94 Interactive", b"Settings Context -- Interactive"),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new)
        log(f"Replaced: {old[:60]!r}")
        count += 1
    else:
        log(f"NOT FOUND: {old[:60]!r}")

with open(path, "wb") as f:
    f.write(data)

log(f"Done. {count} replacements made.")
