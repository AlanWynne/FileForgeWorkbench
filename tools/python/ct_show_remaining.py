import re, os

ROOT = r'C:\workspace\VSC\FileForgeWorkbench\docs\specs'
files = [
    os.path.join(ROOT, 'configuration-system', 'requirements.md'),
    os.path.join(ROOT, 'function-keys-and-history', 'requirements.md'),
    os.path.join(ROOT, 'startup-and-session', 'requirements.md'),
    os.path.join(ROOT, 'virtual-catalog-manager', 'requirements.md'),
]
patterns = ['Settings Panel', 'settings panel', 'Primary Option Menu', 'Files Panel', 'File Explorer Panel', 'Workspace View']
pat = re.compile('|'.join(re.escape(p) for p in patterns), re.IGNORECASE)

for path in files:
    with open(path, 'rb') as f:
        lines = f.read().decode('utf-8', errors='replace').splitlines()
    hits = [(i+1, l.strip()[:130]) for i, l in enumerate(lines) if pat.search(l)]
    if hits:
        print(f'\n=== {os.path.basename(os.path.dirname(path))} ===')
        for ln, text in hits:
            print(f'  L{ln}: {text}')
