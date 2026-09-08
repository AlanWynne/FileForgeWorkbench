path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\key_config_dialog.rs"
with open(path, "rb") as f:
    data = f.read()

old = (
    b"pub struct KeyConfigDialog {\n"
    b"    /// Whether the dialog is currently open.\n"
    b"    pub open: bool,\n"
    b"    active_tab: ScopeTab,\n"
    b"    staged_default: ScopeRows,\n"
    b"    staged_contexts: HashMap<String, ScopeRows>,\n"
    b"    original_default: ScopeRows,\n"
    b"    original_contexts: HashMap<String, ScopeRows>,\n"
    b"}\n"
    b"\n"
    b"impl KeyConfigDialog {\n"
    b"    /// Create a new dialog in the closed state with empty staged maps.\n"
    b"    pub fn new() -> Self {\n"
    b"        let empty = ScopeRows::empty_for(\"global\");\n"
    b"        let ctx_map: HashMap<String, ScopeRows> = CONTEXT_NAMES\n"
    b"            .iter()\n"
    b"            .map(|&n| (n.to_string(), ScopeRows::empty_for(n)))\n"
    b"            .collect();\n"
    b"        Self {\n"
    b"            open: false,\n"
    b"            active_tab: ScopeTab::Default,\n"
)

new = (
    b"pub struct KeyConfigDialog {\n"
    b"    /// Whether the dialog is currently open.\n"
    b"    pub open: bool,\n"
    b"    /// When set by `KEYS <name>`, the dialog opens with this context tab pre-selected.\n"
    b"    ///\n"
    b"    /// Validates: CX Requirement 2.2, 2.3\n"
    b"    pub initial_scope: Option<String>,\n"
    b"    active_tab: ScopeTab,\n"
    b"    staged_default: ScopeRows,\n"
    b"    staged_contexts: HashMap<String, ScopeRows>,\n"
    b"    original_default: ScopeRows,\n"
    b"    original_contexts: HashMap<String, ScopeRows>,\n"
    b"}\n"
    b"\n"
    b"impl KeyConfigDialog {\n"
    b"    /// Create a new dialog in the closed state with empty staged maps.\n"
    b"    pub fn new() -> Self {\n"
    b"        let empty = ScopeRows::empty_for(\"global\");\n"
    b"        let ctx_map: HashMap<String, ScopeRows> = CONTEXT_NAMES\n"
    b"            .iter()\n"
    b"            .map(|&n| (n.to_string(), ScopeRows::empty_for(n)))\n"
    b"            .collect();\n"
    b"        Self {\n"
    b"            open: false,\n"
    b"            initial_scope: None,\n"
    b"            active_tab: ScopeTab::Default,\n"
)

if old in data:
    data = data.replace(old, new, 1)
    with open(path, "wb") as f:
        f.write(data)
    print("KeyConfigDialog patched successfully")
else:
    print("ERROR: pattern not found")
    idx = data.find(b"pub struct KeyConfigDialog")
    print(repr(data[idx:idx+800]))
