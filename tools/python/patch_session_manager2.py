path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\session_manager.rs"
with open(path, "rb") as f:
    data = f.read()

# There are two identical blocks -- the second one is in save_with_workspace
# We already patched the first one. Now patch the second occurrence.
# The second block still has the old pattern (without workspace_name).
old = (
    b"                TabKind::FileEditor => t.path.as_ref().map(|path| SessionTabState {\n"
    b"                    tab_id: format!(\"{}\", t.id.0),\n"
    b"                    tab_kind: PersistedTabKind::FileEditor,\n"
    b"                    uri: Some(path.clone()),\n"
    b"                    viewport_top_line: t.viewport.top_line() as usize,\n"
    b"                    viewport_horizontal_offset: 0,\n"
    b"                    caret_line: t.cursor.cursor_line() as usize,\n"
    b"                    caret_column: t.cursor.cursor_column() as usize,\n"
    b"                    selections: Vec::new(),\n"
    b"                    language_override: None,\n"
    b"                    is_pinned: false,\n"
    b"                    zoom_offset: 0,\n"
    b"                }),\n"
    b"                TabKind::FilesPanel => Some(SessionTabState {\n"
    b"                    tab_id: format!(\"{}\", t.id.0),\n"
    b"                    tab_kind: PersistedTabKind::FilesPanel,\n"
    b"                    uri: None,\n"
    b"                    viewport_top_line: 1,\n"
    b"                    viewport_horizontal_offset: 0,\n"
    b"                    caret_line: 1,\n"
    b"                    caret_column: 1,\n"
    b"                    selections: Vec::new(),\n"
    b"                    language_override: None,\n"
    b"                    is_pinned: false,\n"
    b"                    zoom_offset: 0,\n"
    b"                }),\n"
    b"                TabKind::FileExplorerPanel => Some(SessionTabState {\n"
    b"                    tab_id: format!(\"{}\", t.id.0),\n"
    b"                    tab_kind: PersistedTabKind::FileExplorerPanel,\n"
    b"                    uri: None,\n"
    b"                    viewport_top_line: 1,\n"
    b"                    viewport_horizontal_offset: 0,\n"
    b"                    caret_line: 1,\n"
    b"                    caret_column: 1,\n"
    b"                    selections: Vec::new(),\n"
    b"                    language_override: None,\n"
    b"                    is_pinned: false,\n"
    b"                    zoom_offset: 0,\n"
    b"                }),\n"
)

new = (
    b"                TabKind::FileEditor => t.path.as_ref().map(|path| SessionTabState {\n"
    b"                    tab_id: format!(\"{}\", t.id.0),\n"
    b"                    tab_kind: PersistedTabKind::FileEditor,\n"
    b"                    uri: Some(path.clone()),\n"
    b"                    viewport_top_line: t.viewport.top_line() as usize,\n"
    b"                    viewport_horizontal_offset: 0,\n"
    b"                    caret_line: t.cursor.cursor_line() as usize,\n"
    b"                    caret_column: t.cursor.cursor_column() as usize,\n"
    b"                    selections: Vec::new(),\n"
    b"                    language_override: None,\n"
    b"                    is_pinned: false,\n"
    b"                    zoom_offset: 0,\n"
    b"                    workspace_name: t.workspace_name.clone(),\n"
    b"                }),\n"
    b"                TabKind::FilesPanel => Some(SessionTabState {\n"
    b"                    tab_id: format!(\"{}\", t.id.0),\n"
    b"                    tab_kind: PersistedTabKind::FilesPanel,\n"
    b"                    uri: None,\n"
    b"                    viewport_top_line: 1,\n"
    b"                    viewport_horizontal_offset: 0,\n"
    b"                    caret_line: 1,\n"
    b"                    caret_column: 1,\n"
    b"                    selections: Vec::new(),\n"
    b"                    language_override: None,\n"
    b"                    is_pinned: false,\n"
    b"                    zoom_offset: 0,\n"
    b"                    workspace_name: t.workspace_name.clone(),\n"
    b"                }),\n"
    b"                TabKind::FileExplorerPanel => Some(SessionTabState {\n"
    b"                    tab_id: format!(\"{}\", t.id.0),\n"
    b"                    tab_kind: PersistedTabKind::FileExplorerPanel,\n"
    b"                    uri: None,\n"
    b"                    viewport_top_line: 1,\n"
    b"                    viewport_horizontal_offset: 0,\n"
    b"                    caret_line: 1,\n"
    b"                    caret_column: 1,\n"
    b"                    selections: Vec::new(),\n"
    b"                    language_override: None,\n"
    b"                    is_pinned: false,\n"
    b"                    zoom_offset: 0,\n"
    b"                    workspace_name: t.workspace_name.clone(),\n"
    b"                }),\n"
)

count = data.count(old)
print(f"Pattern found {count} time(s)")
if count > 0:
    data = data.replace(old, new)  # replace all remaining occurrences
    with open(path, "wb") as f:
        f.write(data)
    print("save_with_workspace patched")
else:
    print("No remaining occurrences -- already patched or pattern mismatch")
