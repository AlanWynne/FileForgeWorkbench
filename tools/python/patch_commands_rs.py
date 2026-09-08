import sys

path = r"crates\ff-desktop\src\shell\commands.rs"

with open(path, "rb") as f:
    data = f.read()

sep = b"\r\n" if b"\r\n" in data else b"\n"
sys.stdout.write(f"Line ending: {'CRLF' if sep == b'\r\n' else 'LF'}\n")

# Find the anchor: the Route through CommandEngine comment
anchor = b"        // \xe2\x94\x80\xe2\x94\x80 Route through CommandEngine"
idx = data.find(anchor)
if idx < 0:
    # Try ASCII version
    anchor = b"        // Route through CommandEngine"
    idx = data.find(anchor)

if idx < 0:
    sys.stdout.write("ERROR: anchor not found\n")
    # Show last 200 bytes before end
    sys.stdout.write(repr(data[-500:]) + "\n")
else:
    sys.stdout.write(f"Anchor found at offset {idx}\n")
    insert = (
        b"        // Menu_Workspace option key lookup -- Validates: menu-workspace Requirement 3.1, 3.6, 3.7" + sep +
        b"        if self.tabs.active_tab().kind == crate::tab_state::TabKind::MenuWorkspace {" + sep +
        b"            if let Some(mw) = self.tabs.active_tab().menu_workspace.as_ref() {" + sep +
        b"                if let Some(menu) = mw.menu.as_ref() {" + sep +
        b"                    match crate::menu_workspace::commands::lookup_option(cmd.trim(), menu) {" + sep +
        b"                        Ok(option_cmd) => {" + sep +
        b"                            self.handle_command(&option_cmd);" + sep +
        b"                            return;" + sep +
        b"                        }" + sep +
        b"                        Err(msg) => {" + sep +
        b"                            self.open_error = Some(msg);" + sep +
        b"                            return;" + sep +
        b"                        }" + sep +
        b"                    }" + sep +
        b"                }" + sep +
        b"            }" + sep +
        b"        }" + sep +
        sep
    )
    data = data[:idx] + insert + data[idx:]
    with open(path, "wb") as f:
        f.write(data)
    sys.stdout.write("Patch applied: MenuWorkspace option key lookup added\n")
