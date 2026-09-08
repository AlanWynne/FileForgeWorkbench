import os
import re

root = r'C:\workspace\VSC\FileForgeWorkbench\docs\specs'
patterns = [
    'Settings Panel', 'Toolchain Panel', 'floating window', 'Detached View',
    'Primary Option Menu', 'Workspace View', 'Content Editor', 'Files Panel',
    'File Explorer Panel'
]
pat = re.compile('|'.join(re.escape(p) for p in patterns), re.IGNORECASE)

hits = []
for dirpath, dirs, files in os.walk(root):
    for fn in files:
        if fn == 'requirements.md':
            path = os.path.join(dirpath, fn)
            try:
                with open(path, 'rb') as f:
                    content = f.read().decode('utf-8', errors='replace')
                matches = pat.findall(content)
                if matches:
                    hits.append((path, set(m.lower() for m in matches)))
            except Exception as e:
                print(f'ERROR {path}: {e}')

for path, terms in sorted(hits):
    short = path.replace(root + os.sep, '')
    print(f'{short}: {sorted(terms)}')

print(f'\nTotal: {len(hits)} files with old terminology')
