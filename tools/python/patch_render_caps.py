LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\patch_render_caps.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render.rs"

with open(path, "rb") as f:
    raw = f.read()

log(f"File size: {len(raw)} bytes")
text = raw.decode("utf-8")

old = (
    "                // Requirement 6.5: modified indicator\n"
    "                if tab.is_modified {\n"
    "                    ui.colored_label(to_egui_color(self.palette.editor.accent), \"\u25cf\");\n"
    "                    ui.separator();\n"
    "                }\n"
    "\n"
    "                if let Some(err) = &self.open_error {"
)

new = (
    "                // Requirement 6.5: modified indicator\n"
    "                if tab.is_modified {\n"
    "                    ui.colored_label(to_egui_color(self.palette.editor.accent), \"\u25cf\");\n"
    "                    ui.separator();\n"
    "                }\n"
    "                // Req 16.3: CAPS mode indicator\n"
    "                if tab.edit_profile.caps.is_on() {\n"
    "                    ui.colored_label(to_egui_color(self.palette.editor.accent), \"CAPS\");\n"
    "                    ui.separator();\n"
    "                }\n"
    "\n"
    "                if let Some(err) = &self.open_error {"
)

if old in text:
    text = text.replace(old, new, 1)
    log("Replacement made")
else:
    log("ERROR: pattern not found")

with open(path, "wb") as f:
    f.write(text.encode("utf-8"))

log("Done")
