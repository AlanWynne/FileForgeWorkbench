LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

lines = open(path, encoding='utf-8').readlines()
total = len(lines)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('total lines: ' + str(total) + '\n')
    lf.write('--- summary 1 (around line 516) ---\n')
    for i, l in enumerate(lines[513:530], start=513):
        lf.write(str(i+1) + ' ' + repr(l) + '\n')
    lf.write('--- summary 2 (around line 850) ---\n')
    for i, l in enumerate(lines[847:900], start=847):
        lf.write(str(i+1) + ' ' + repr(l) + '\n')
