import sys

LOG = r"tools\logs\patch-update-rs.txt"

def log(msg):
    sys.stdout.write(msg + "\n")
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"crates\ff-desktop\src\shell\update.rs"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# Detect line ending
sep = b"\r\n" if b"\r\n" in data else b"\n"
log(f"Line ending: {'CRLF' if sep == b'\r\n' else 'LF'}")

# --- Patch 1: add ensure_default_menu_files after the startup block ---
# Find the anchor: the last line of the startup block before "Startup focus"
anchor = (
    b"                self.tabs.close_welcome_tab();" + sep +
    b"                self.tabs.insert_pom_tab(&self.runtime);" + sep +
    b"            }" + sep
)
idx = data.find(anchor)
if idx < 0:
    log("ERROR: startup anchor not found")
else:
    insert_pos = idx + len(anchor)
    new_block = (
        sep +
        b"            // Validates: menu-workspace Requirement 4.1, 4.2, 4.6 -- create default menu files." + sep +
        b"            if let Some(session) = &self.session {" + sep +
        b"                if let Ok(mut udd) = ff_session::UserDataDir::resolve(None) {" + sep +
        b"                    let _ = udd.initialise();" + sep +
        b"                    crate::menu_workspace::defaults::ensure_default_menu_files(udd.data_dir());" + sep +
        b"                }" + sep +
        b"            }" + sep
    )
    data = data[:insert_pos] + new_block + data[insert_pos:]
    log("Patch 1 applied: ensure_default_menu_files wired into startup")

with open(path, "wb") as f:
    f.write(data)

log("Done")
