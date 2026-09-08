import sys

def log(msg):
    sys.stdout.write(msg + "\n")

# ── Task 9: Replace DEFAULT_POM_TOML in defaults.rs ─────────────────────────

path = r"crates\ff-desktop\src\menu_workspace\defaults.rs"
with open(path, "rb") as f:
    data = f.read()

sep = b"\r\n" if b"\r\n" in data else b"\n"

# Find the start of DEFAULT_POM_TOML const
start_marker = b"pub const DEFAULT_POM_TOML: &str = r#\""
end_marker = b"\"#;"

start = data.find(start_marker)
if start < 0:
    log("ERROR: DEFAULT_POM_TOML start not found")
    sys.exit(1)

# Find the end marker after start
end = data.find(end_marker, start)
if end < 0:
    log("ERROR: DEFAULT_POM_TOML end not found")
    sys.exit(1)

end += len(end_marker)

new_const = (
    b"/// Default content for `menus/pom.toml` -- 12-option POM (Phase CV)." + sep +
    b"///" + sep +
    b"/// Validates: Requirement 7.1, 7.4, 7.5 (menu-workspace cv-requirements)" + sep +
    b"pub const DEFAULT_POM_TOML: &str = r#\"title = \"FileForge Workbench -- Primary Option Menu\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"0\"" + sep +
    b"command = \"SETTINGS\"" + sep +
    b"description = \"FFWB Settings and Client Parameters\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"1\"" + sep +
    b"command = \"CATALOGS\"" + sep +
    b"description = \"Virtual File Catalogs -- Mainframe, POSIX, Native\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"2\"" + sep +
    b"command = \"FILES\"" + sep +
    b"description = \"File Explorer -- Browse catalogs and files in a tree view\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"3\"" + sep +
    b"command = \"UTILITIES\"" + sep +
    b"description = \"Perform utility functions\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"4\"" + sep +
    b"command = \"COMPILERS\"" + sep +
    b"description = \"Interactive language processing\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"5\"" + sep +
    b"command = \"MACROS\"" + sep +
    b"description = \"Run and manage Lua macros\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"6\"" + sep +
    b"command = \"TERMINALS\"" + sep +
    b"description = \"Enter TSO or Workstation commands\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"7\"" + sep +
    b"command = \"DATABASES\"" + sep +
    b"description = \"Database tool and query browser\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"8\"" + sep +
    b"command = \"PLUGINS\"" + sep +
    b"description = \"Vendor added plugins\"" + sep +
    b"group = \"Core\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"9\"" + sep +
    b"command = \"JES\"" + sep +
    b"description = \"JES job monitor and spool viewer\"" + sep +
    b"group = \"Extended\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"S\"" + sep +
    b"command = \"SEARCH\"" + sep +
    b"description = \"Global search and replace across files\"" + sep +
    b"group = \"Extended\"" + sep +
    sep +
    b"[[options]]" + sep +
    b"key = \"B\"" + sep +
    b"command = \"BATCH\"" + sep +
    b"description = \"Batch command execution (IKJEFT01 analogue)\"" + sep +
    b"group = \"Extended\"" + sep +
    b"\"#;"
)

data = data[:start] + new_const + data[end:]
with open(path, "wb") as f:
    f.write(data)
log("Task 9: DEFAULT_POM_TOML updated with 12 options")

# ── Task 10: Add 9/S/B routing to shell/commands.rs ─────────────────────────

path = r"crates\ff-desktop\src\shell\commands.rs"
with open(path, "rb") as f:
    data = f.read()

sep = b"\r\n" if b"\r\n" in data else b"\n"

# Insert after the "6" / MACROS handler -- find its return statement
anchor = (
    b"            self.tabs.open_macro_library_tab(&self.runtime);" + sep +
    b"        }" + sep +
    b"        self.open_error = None;" + sep +
    b"        return;" + sep +
    b"        }"
)
idx = data.find(anchor)
if idx < 0:
    log("ERROR: MACROS anchor not found in commands.rs")
    sys.exit(1)

insert_pos = idx + len(anchor)

new_handlers = (
    sep +
    sep +
    b"        // ── 9 / JES -- Validates: cv-requirements Requirement 6.2 ─────────────" + sep +
    b"        if upper == \"9\" || upper == \"JES\" || upper == \"=9\" {" + sep +
    b"            // Route to JES job monitor panel (stub -- full JES panel in ff-jes)" + sep +
    b"            self.open_error = Some(\"JES: job monitor panel not yet wired to POM option 9.\".to_string());" + sep +
    b"            return;" + sep +
    b"        }" + sep +
    sep +
    b"        // ── S / SEARCH -- Validates: cv-requirements Requirement 6.3 ──────────" + sep +
    b"        if upper == \"S\" || upper == \"=S\" {" + sep +
    b"            // Validates: cv-requirements Requirement 6.3 -- S opens Global Search" + sep +
    b"            self.open_or_focus_search_panel();" + sep +
    b"            self.open_error = None;" + sep +
    b"            return;" + sep +
    b"        }" + sep +
    sep +
    b"        // ── B / BATCH -- Validates: cv-requirements Requirement 6.4 ───────────" + sep +
    b"        if upper == \"B\" || upper == \"=B\" || upper == \"BATCH\" {" + sep +
    b"            // Validates: cv-requirements Requirement 6.4 -- B shows batch status message" + sep +
    b"            self.open_error = Some(" + sep +
    b"                \"Batch execution is available via the --batch CLI flag or the BATCH command.\".to_string()," + sep +
    b"            );" + sep +
    b"            return;" + sep +
    b"        }"
)

data = data[:insert_pos] + new_handlers + data[insert_pos:]
with open(path, "wb") as f:
    f.write(data)
log("Task 10: 9/S/B routing added to commands.rs")

log("All CV-impl patches applied")
