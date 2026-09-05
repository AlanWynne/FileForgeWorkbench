import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\helpers.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\n'

if b'TabKind::SearchResults' in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ALREADY_DONE\n')
    sys.exit(0)

old = b'        TabKind::FileExplorerPanel => Some("files"),' + sep + b'    }' + sep + b'}'
new = (
    b'        TabKind::FileExplorerPanel => Some("files"),' + sep +
    b'        TabKind::SearchResults => Some("search"),' + sep +
    b'    }' + sep +
    b'}'
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
