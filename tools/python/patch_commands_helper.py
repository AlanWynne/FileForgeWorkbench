import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\commands.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\n'

# Check if the fn definition is already present
if b'fn open_or_focus_search_panel' in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ALREADY_DONE\n')
    sys.exit(0)

# Find the last `\n}` which closes the impl block
idx = data.rfind(b'\n}')
with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('last_brace_at: ' + str(idx) + '\n')

helper = (
    sep +
    b'    /// Open the Search Results panel, or focus it if already open.' + sep +
    b'    ///' + sep +
    b'    /// Validates: global-search Requirement 1.1, 1.3' + sep +
    b'    pub(super) fn open_or_focus_search_panel(&mut self) {' + sep +
    b'        use crate::tab_state::TabKind;' + sep +
    b'        for i in 0..self.tabs.len() {' + sep +
    b'            if self.tabs.tabs()[i].kind == TabKind::SearchResults {' + sep +
    b'                self.tabs.set_active(i);' + sep +
    b'                return;' + sep +
    b'            }' + sep +
    b'        }' + sep +
    b'        self.tabs.open_search_results_tab(&self.runtime);' + sep +
    b'    }' + sep
)

data = data[:idx] + helper + data[idx:]
with open(path, 'wb') as f:
    f.write(data)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('WRITTEN OK\n')
