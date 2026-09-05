import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\commands.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\r\n' if b'\r\n' in data else b'\n'

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('sep=' + repr(sep) + '\n')

marker = b'if upper == "GSEARCH"'
if marker in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ALREADY_DONE\n')
    sys.exit(0)

# Note: comment uses UTF-8 em dash \xe2\x80\x94
old = (
    b'        if upper == "FILES" {' + sep +
    b'            // Validates: Requirement 19.3 \xe2\x80\x94 FILES (no =) always opens a NEW tab' + sep +
    b'            self.tabs.open_file_explorer_panel_tab(&self.runtime);' + sep +
    b'            self.open_error = None;' + sep +
    b'            return;' + sep +
    b'        }' + sep + sep +
    b'        if upper == "1" || upper == "=1" || upper == "FILE CATALOGS" {'
)

new = (
    b'        if upper == "FILES" {' + sep +
    b'            // Validates: Requirement 19.3 \xe2\x80\x94 FILES (no =) always opens a NEW tab' + sep +
    b'            self.tabs.open_file_explorer_panel_tab(&self.runtime);' + sep +
    b'            self.open_error = None;' + sep +
    b'            return;' + sep +
    b'        }' + sep + sep +
    b'        if upper == "GSEARCH" || upper == "SEARCH" {' + sep +
    b'            // Validates: global-search Requirement 1.2' + sep +
    b'            self.open_or_focus_search_panel();' + sep +
    b'            self.open_error = None;' + sep +
    b'            return;' + sep +
    b'        }' + sep + sep +
    b'        if upper == "1" || upper == "=1" || upper == "FILE CATALOGS" {'
)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('old_in_data=' + str(old in data) + '\n')

if old not in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ERROR: pattern not found\n')
    sys.exit(1)

data = data.replace(old, new, 1)
with open(path, 'wb') as f:
    f.write(data)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('WRITTEN OK\n')
