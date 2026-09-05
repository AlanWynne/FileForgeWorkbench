import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

def patch(path, old, new, label):
    with open(path, 'rb') as f:
        data = f.read()
    sep = b'\r\n' if b'\r\n' in data else b'\n'
    found = old in data
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write(label + ': found=' + str(found) + '\n')
    if found:
        data = data.replace(old, new, 1)
        with open(path, 'wb') as f:
            f.write(data)
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write(label + ': WRITTEN\n')
    return found

# 1. Fix set_top_line -> scroll_to_line in render.rs
render_path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render.rs'
patch(
    render_path,
    b'                                        tab.viewport.set_top_line(line.saturating_sub(1).max(1));',
    b'                                        tab.viewport.scroll_to_line(line.saturating_sub(1).max(1), &tab.cursor.clone());',
    'render set_top_line fix'
)

# 2. Fix session_manager.rs -- two filter_map matches and two active.kind matches
sm_path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\session_manager.rs'
with open(sm_path, 'rb') as f:
    sm = f.read()
sep = b'\n'

# Pattern: the filter_map match arms that end with PrimaryOptionMenu | Untitled | SettingsPanel => None
old_filter = (
    b'                TabKind::PrimaryOptionMenu | TabKind::Untitled | TabKind::SettingsPanel => None,'
)
new_filter = (
    b'                TabKind::PrimaryOptionMenu | TabKind::Untitled | TabKind::SettingsPanel | TabKind::SearchResults => None,'
)
count = sm.count(old_filter)
with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('session_manager filter_map count: ' + str(count) + '\n')
sm = sm.replace(old_filter, new_filter)

# Pattern: active.kind match arms that end with PrimaryOptionMenu | Untitled | SettingsPanel | FileExplorerPanel => None
old_active = (
    b'                | TabKind::Untitled\n'
    b'                | TabKind::SettingsPanel\n'
    b'                | TabKind::FileExplorerPanel => None,'
)
new_active = (
    b'                | TabKind::Untitled\n'
    b'                | TabKind::SettingsPanel\n'
    b'                | TabKind::FileExplorerPanel\n'
    b'                | TabKind::SearchResults => None,'
)
count2 = sm.count(old_active)
with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('session_manager active count: ' + str(count2) + '\n')
sm = sm.replace(old_active, new_active)

with open(sm_path, 'wb') as f:
    f.write(sm)
with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('session_manager: written\n')

# 3. Fix helpers.rs -- context_name_for_kind
helpers_path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\helpers.rs'
with open(helpers_path, 'rb') as f:
    h = f.read()
with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('helpers content around TabKind: ' + repr(h[h.find(b'TabKind::FileExplorerPanel'):h.find(b'TabKind::FileExplorerPanel')+100]) + '\n')

# Find the FileExplorerPanel arm and add SearchResults
old_h = b'TabKind::FileExplorerPanel =>'
if old_h in h:
    # Read the full arm
    idx = h.find(old_h)
    chunk = h[idx:idx+80]
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('helpers arm: ' + repr(chunk) + '\n')

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('DONE\n')
