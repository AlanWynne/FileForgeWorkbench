import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\commands.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('file size: ' + str(len(data)) + '\n')
    idx = data.find(b'if upper == "FILES"')
    lf.write('FILES idx: ' + str(idx) + '\n')
    if idx >= 0:
        chunk = data[idx:idx+350]
        lf.write(repr(chunk) + '\n')
    idx2 = data.find(b'if upper == "1" || upper == "=1"')
    lf.write('1/=1 idx: ' + str(idx2) + '\n')
    if idx2 >= 0:
        chunk2 = data[idx2-5:idx2+50]
        lf.write(repr(chunk2) + '\n')
