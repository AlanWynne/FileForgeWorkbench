LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

lines = open(path, encoding='utf-8').readlines()
for i, l in enumerate(lines[653:700], start=653):
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write(str(i+1) + ' ' + repr(l) + '\n')
