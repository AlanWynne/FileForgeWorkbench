import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\r\n' if b'\r\n' in data else b'\n'

replacements = [
    (b'- [ ] BS-A.1', b'- [x] BS-A.1'),
    (b'- [ ] BS-A.2', b'- [x] BS-A.2'),
    (b'- [ ] BS-A.3', b'- [x] BS-A.3'),
    (b'- [ ] BS-A.4', b'- [x] BS-A.4'),
    (b'- [ ] BS-A.5', b'- [x] BS-A.5'),
    (b'- [ ] BS-A.6', b'- [x] BS-A.6'),
    (b'- [ ] BS-B.1', b'- [x] BS-B.1'),
    (b'- [ ] BS-B.2', b'- [x] BS-B.2'),
    (b'- [ ] BS-B.3', b'- [x] BS-B.3'),
    (b'- [ ] BS-B.4', b'- [x] BS-B.4'),
    (b'- [ ] BS-C.1', b'- [x] BS-C.1'),
    (b'- [ ] BS-C.2', b'- [x] BS-C.2'),
    (b'- [ ] BS-C.3', b'- [x] BS-C.3'),
    (b'- [ ] BS-C.4', b'- [x] BS-C.4'),
    (b'- [ ] BS-C.5', b'- [x] BS-C.5'),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new)
        count += 1

with open(path, 'wb') as f:
    f.write(data)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('replaced: ' + str(count) + '\n')
    lf.write('DONE\n')
