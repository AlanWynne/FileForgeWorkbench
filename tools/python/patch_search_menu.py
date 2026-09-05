import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

# --- 1. Update MENU_BAR_TOP_LEVEL_LABELS in mod.rs ---
mod_path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\mod.rs'
with open(mod_path, 'rb') as f:
    mod_data = f.read()

sep = b'\r\n' if b'\r\n' in mod_data else b'\n'

if b'"Search"' in mod_data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('mod.rs: Search already present\n')
else:
    old_labels = (
        b'    "Edit",' + sep +
        b'    "Help",' + sep +
        b'];'
    )
    new_labels = (
        b'    "Search",' + sep +
        b'    "Edit",' + sep +
        b'    "Help",' + sep +
        b'];'
    )
    if old_labels in mod_data:
        mod_data = mod_data.replace(old_labels, new_labels, 1)
        with open(mod_path, 'wb') as f:
            f.write(mod_data)
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('mod.rs: labels updated\n')
    else:
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('mod.rs: ERROR labels pattern not found\n')

# --- 2. Update assert count 12->13 in render_chrome.rs ---
chrome_path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render_chrome.rs'
with open(chrome_path, 'rb') as f:
    chrome_data = f.read()

sep2 = b'\r\n' if b'\r\n' in chrome_data else b'\n'

if b'            13,' in chrome_data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('render_chrome.rs: assert already 13\n')
else:
    old_assert = (
        b'        debug_assert_eq!(' + sep2 +
        b'            super::MENU_BAR_TOP_LEVEL_LABELS.len(),' + sep2 +
        b'            12,' + sep2 +
        b'            "render_menu_bar must contain one menu_button per super::MENU_BAR_TOP_LEVEL_LABELS entry"' + sep2 +
        b'        );'
    )
    new_assert = (
        b'        debug_assert_eq!(' + sep2 +
        b'            super::MENU_BAR_TOP_LEVEL_LABELS.len(),' + sep2 +
        b'            13,' + sep2 +
        b'            "render_menu_bar must contain one menu_button per super::MENU_BAR_TOP_LEVEL_LABELS entry"' + sep2 +
        b'        );'
    )
    if old_assert in chrome_data:
        chrome_data = chrome_data.replace(old_assert, new_assert, 1)
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('render_chrome.rs: assert updated to 13\n')
    else:
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('render_chrome.rs: ERROR assert pattern not found\n')

# --- 3. Add Search menu button after View menu in render_chrome.rs ---
# Find the View menu closing and insert Search after it
if b'ui.menu_button("Search"' in chrome_data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('render_chrome.rs: Search menu already present\n')
else:
    # Find the Utilities menu start to insert before it
    old_utilities = (
        b'                // \xe2\x80\x94\xe2\x80\x94 Utilities \xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94' + sep2 +
        b'                ui.menu_button("Utilities"'
    )
    # Try a simpler anchor
    old_utilities2 = b'                ui.menu_button("Utilities"'
    idx = chrome_data.find(old_utilities2)
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('Utilities idx: ' + str(idx) + '\n')
    if idx >= 0:
        search_block = (
            b'                // \xe2\x80\x94\xe2\x80\x94 Search \xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94\xe2\x80\x94' + sep2 +
            b'                ui.menu_button("Search", |ui| {' + sep2 +
            b'                    if ui.button("Find in Files  Ctrl+Shift+F").clicked() {' + sep2 +
            b'                        self.open_or_focus_search_panel();' + sep2 +
            b'                        ui.close_menu();' + sep2 +
            b'                    }' + sep2 +
            b'                });' + sep2 +
            b'                ui.menu_button("Utilities"'
        )
        chrome_data = chrome_data[:idx] + search_block + chrome_data[idx + len(old_utilities2):]
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('render_chrome.rs: Search menu inserted\n')
    else:
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('render_chrome.rs: ERROR Utilities anchor not found\n')

with open(chrome_path, 'wb') as f:
    chrome_data_to_write = chrome_data
    f.write(chrome_data_to_write)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('render_chrome.rs: written\n')
with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('DONE\n')
