path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\key_config_dialog.rs"
with open(path, "rb") as f:
    data = f.read()

# Patch: apply initial_scope when dialog opens, then clear it
old = (
    b"    if !dialog.open {\n"
    b"        return;\n"
    b"    }\n"
    b"\n"
    b"    let mut save_clicked = false;\n"
)
new = (
    b"    if !dialog.open {\n"
    b"        return;\n"
    b"    }\n"
    b"\n"
    b"    // Apply initial_scope from KEYS <name> command -- Validates: CX Requirement 2.2\n"
    b"    if let Some(ref scope) = dialog.initial_scope.take() {\n"
    b"        let matched = [\"pom\", \"editor\", \"settings\", \"files\", \"hex\", \"toolchain\"]\n"
    b"            .iter()\n"
    b"            .find(|&&n| n == scope.as_str())\n"
    b"            .copied();\n"
    b"        if let Some(name) = matched {\n"
    b"            dialog.active_tab = ScopeTab::Context(name.to_string());\n"
    b"        }\n"
    b"    }\n"
    b"\n"
    b"    let mut save_clicked = false;\n"
)

if old in data:
    data = data.replace(old, new, 1)
    with open(path, "wb") as f:
        f.write(data)
    print("render patched: initial_scope applied")
else:
    print("ERROR: render open pattern not found")
    idx = data.find(b"if !dialog.open")
    print(repr(data[idx:idx+200]))
