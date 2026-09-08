path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render_chrome.rs"
with open(path, "rb") as f:
    data = f.read()

old = (
    b"                        let label = if tab.is_modified {\n"
    b"                            format!(\"\xe2\x97\x8f {}\", tab.title)\n"
    b"                        } else {\n"
    b"                            tab.title.clone()\n"
    b"                        };\n"
)

new = (
    b"                        // Validates: CX Requirement 1.4 -- show workspace_name in tab header\n"
    b"                        let base_title = if let Some(ref name) = tab.workspace_name {\n"
    b"                            match tab.kind {\n"
    b"                                crate::tab_state::TabKind::FileEditor\n"
    b"                                | crate::tab_state::TabKind::Untitled => {\n"
    b"                                    format!(\"{}: {}\", name, tab.title)\n"
    b"                                }\n"
    b"                                _ => format!(\"[{}]\", name),\n"
    b"                            }\n"
    b"                        } else {\n"
    b"                            tab.title.clone()\n"
    b"                        };\n"
    b"                        let label = if tab.is_modified {\n"
    b"                            format!(\"\xe2\x97\x8f {}\", base_title)\n"
    b"                        } else {\n"
    b"                            base_title\n"
    b"                        };\n"
)

if old in data:
    data = data.replace(old, new, 1)
    with open(path, "wb") as f:
        f.write(data)
    print("render_chrome.rs patched: workspace_name shown in tab header")
else:
    print("ERROR: pattern not found")
    idx = data.find(b"let label = if tab.is_modified")
    print(repr(data[max(0,idx-50):idx+200]))
