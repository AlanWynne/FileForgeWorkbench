"""Patch the Files menu in render_chrome.rs to add workspace menu items."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\patch_files_menu_out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

PATH = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render_chrome.rs"

with open(PATH, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# The old Files menu block -- use the exact bytes from the file (CRLF, UTF-8 with BOM)
# We match from the comment line through the closing }); of the Files menu
# Use a unique anchor: the Exit button inside the Files menu followed by close_menu
OLD = (
    b"                    if ui.button(\"Exit\").clicked() {\r\n"
    b"                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);\r\n"
    b"                    }\r\n"
    b"                });\r\n"
    b"                // \xe2\x94\x80\xe2\x94\x80 Utilities"
)

NEW = (
    b"                    ui.separator();\r\n"
    b"                    // -- Workspace items -- Validates: workspace-model Requirement 2.1-2.4\r\n"
    b"                    if ui.button(\"Open Workspace...\").clicked() {\r\n"
    b"                        self.open_error = Some(\r\n"
    b"                            \"Use: WORKSPACE OPEN <path>\".to_string(),\r\n"
    b"                        );\r\n"
    b"                        ui.close_menu();\r\n"
    b"                    }\r\n"
    b"                    let ws_name = self\r\n"
    b"                        .active_workspace\r\n"
    b"                        .as_ref()\r\n"
    b"                        .map(|ws| ws.name.clone());\r\n"
    b"                    ui.add_enabled_ui(ws_name.is_some(), |ui| {\r\n"
    b"                        let label = ws_name\r\n"
    b"                            .as_deref()\r\n"
    b"                            .map(|n| format!(\"Save Workspace ({})\", n))\r\n"
    b"                            .unwrap_or_else(|| \"Save Workspace\".to_string());\r\n"
    b"                        if ui.button(label).clicked() {\r\n"
    b"                            self.save_workspace_to(None);\r\n"
    b"                            ui.close_menu();\r\n"
    b"                        }\r\n"
    b"                    });\r\n"
    b"                    ui.add_enabled_ui(self.active_workspace.is_some(), |ui| {\r\n"
    b"                        if ui.button(\"Save Workspace As...\").clicked() {\r\n"
    b"                            self.open_error = Some(\r\n"
    b"                                \"Use: WORKSPACE SAVE AS <path>\".to_string(),\r\n"
    b"                            );\r\n"
    b"                            ui.close_menu();\r\n"
    b"                        }\r\n"
    b"                    });\r\n"
    b"                    ui.add_enabled_ui(self.active_workspace.is_some(), |ui| {\r\n"
    b"                        if ui.button(\"Close Workspace\").clicked() {\r\n"
    b"                            self.close_workspace();\r\n"
    b"                            ui.close_menu();\r\n"
    b"                        }\r\n"
    b"                    });\r\n"
    b"                    ui.separator();\r\n"
    b"                    if ui.button(\"Exit\").clicked() {\r\n"
    b"                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);\r\n"
    b"                    }\r\n"
    b"                });\r\n"
    b"                // \xe2\x94\x80\xe2\x94\x80 Utilities"
)

if OLD in data:
    log("Pattern found -- applying patch")
    data = data.replace(OLD, NEW, 1)
    with open(PATH, "wb") as f:
        f.write(data)
    log("Patch written successfully")
else:
    log("ERROR: pattern not found -- no change made")
    # Dump context around 'Exit' in Files menu for diagnosis
    idx = data.find(b'if ui.button("Exit").clicked()')
    if idx >= 0:
        log(f"'Exit' button found at byte {idx}")
        log(repr(data[idx-20:idx+120]))
    sys.exit(1)
