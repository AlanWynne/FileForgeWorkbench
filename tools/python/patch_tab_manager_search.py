LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\tab_manager.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\r\n' if b'\r\n' in data else b'\n'

if b'open_search_results_tab' in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ALREADY_DONE\n')
    import sys; sys.exit(0)

# Insert after open_file_explorer_panel_tab closing brace
old = (
    b'        let _ = runtime;' + sep +
    b'    }' + sep +
    sep +
    b'    /// Transform the active tab in-place'
)
new = (
    b'        let _ = runtime;' + sep +
    b'    }' + sep +
    sep +
    b'    /// Open a new Search Results panel tab.' + sep +
    b'    ///' + sep +
    b'    /// Validates: global-search Requirement 1.1' + sep +
    b'    pub fn open_search_results_tab(&mut self, runtime: &Runtime) {' + sep +
    b'        let document = ff_document_model::new_document();' + sep +
    b'        let id = TabId(self.next_id);' + sep +
    b'        self.next_id += 1;' + sep +
    b'        let tab = crate::tab_state::TabState::search_results_panel(id, document);' + sep +
    b'        self.tabs.push(tab);' + sep +
    b'        self.active = self.tabs.len() - 1;' + sep +
    b'        let _ = runtime;' + sep +
    b'    }' + sep +
    sep +
    b'    /// Transform the active tab in-place'
)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('old_in_data=' + str(old in data) + '\n')

if old not in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ERROR: pattern not found\n')
    import sys; sys.exit(1)

data = data.replace(old, new, 1)
with open(path, 'wb') as f:
    f.write(data)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('WRITTEN OK\n')
