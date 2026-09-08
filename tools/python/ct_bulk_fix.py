import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\ct_bulk_fix.txt"
ROOT = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs"

with open(LOG, "w", encoding="utf-8") as f:
    f.write("CT bulk fix log\n")

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Each entry: (filename_fragment, [(old_bytes, new_bytes), ...])
# Using simple byte replacements -- tries both LF and CRLF variants automatically
# by trying the pattern as-is (file may use either)

GLOBAL_REPLACEMENTS = [
    # Primary Option Menu -> Home Context (POM) in user-facing text
    # Keep "Primary Option Menu" as alias note where it appears in glossary/heritage contexts
    # but replace standalone product-facing uses
    (b"the Primary Option Menu tab", b"the Home Context (POM) tab"),
    (b"a Primary Option Menu tab", b"a Home Context (POM) tab"),
    (b"Primary Option Menu tab ", b"Home Context (POM) tab "),
    (b"Primary Option Menu tab.", b"Home Context (POM) tab."),
    (b"Primary Option Menu tab,", b"Home Context (POM) tab,"),
    (b"Primary Option Menu tab)", b"Home Context (POM) tab)"),
    (b"Primary Option Menu tab\n", b"Home Context (POM) tab\n"),
    (b"Primary Option Menu tab\r\n", b"Home Context (POM) tab\r\n"),
    # Floating window -> Detached Workspace
    (b"floating window", b"Detached Workspace"),
    (b"Floating_Window", b"Detached_Workspace"),
    (b"Floating Window", b"Detached Workspace"),
    # Detached View -> Detached Workspace
    (b"Detached View", b"Detached Workspace"),
    # Toolchain Panel -> Compiler Context
    (b"Toolchain Panel", b"Compiler Context"),
    (b"Toolchain_Panel", b"Compiler_Context"),
    # Settings Panel -> Settings Context (standalone references)
    (b"Settings Panel", b"Settings Context"),
    (b"Settings panel", b"Settings Context"),
    # Files Panel -> Catalog Explorer Context
    (b"Files_Panel", b"Catalog_Explorer_Context"),
    (b"Files Panel", b"Catalog Explorer Context"),
    # File Explorer Panel -> File Explorer Context
    (b"File Explorer Panel", b"File Explorer Context"),
    (b"File_Explorer_Panel", b"File_Explorer_Context"),
]

# Per-file targeted replacements (for cases where global is too broad)
FILE_SPECIFIC = {
    "configuration-system/requirements.md": [
        # "Primary Option Menu" remaining reference in Req 15 already fixed; 
        # any remaining are in cross-references which are fine to keep as heritage
    ],
    "context-help/requirements.md": [],
    "database-tool/requirements.md": [],
    "function-keys-and-history/requirements.md": [
        # "Primary Option Menu" in Req 14 context name -- already fixed to "Home Context (POM)"
        # "Settings Panel" in Req 20 -- fix
    ],
    "layout-and-docking/requirements.md": [
        # Floating_Window already in glossary as Detached_Workspace -- but body text may still have it
    ],
    "menu-and-statusbar/requirements.md": [
        # Files Panel in Req 17.6 "Settings, Files Panel, etc."
        (b"Settings, Files Panel, etc.", b"Settings Context, Catalog Explorer Context, etc."),
    ],
    "multi-tab-editor/requirements.md": [],
    "project-master/requirements.md": [],
    "startup-and-session/requirements.md": [
        # File_Tree_Panel -> File Explorer Context in glossary
        (b"File_Tree_Panel root", b"File Explorer Context root"),
        (b"File_Tree_Panel in the left dock zone", b"File Explorer Context in the left dock zone"),
        (b"File_Tree_Panel SHALL show", b"File Explorer Context SHALL show"),
    ],
    "theme-and-appearance/requirements.md": [],
    "virtual-catalog-manager/requirements.md": [
        # Workspace View -> Workspace
        (b"Workspace View", b"Workspace"),
        # Primary Option Menu in Req 1.1 -- keep as heritage alias note
        (b"the Primary Option Menu (or types `1` or `FILES`", b"the Home Context (POM) (or types `1` or `FILES`"),
        (b"the Primary Option Menu (or types `1`", b"the Home Context (POM) (or types `1`"),
        (b"the Primary Option Menu view", b"the Home Context (POM) view"),
        (b"the Primary Option Menu tab", b"the Home Context (POM) tab"),
        # Settings panel in Req 12.5/12.6
        (b"display in the Settings panel.", b"display in the Settings Context."),
        (b"in the Settings panel,", b"in the Settings Context,"),
        (b"in the Settings panel and", b"in the Settings Context and"),
        # File Explorer Panel
        (b"the File Explorer Panel sidebar", b"the File Explorer Context sidebar"),
        (b"the File Explorer Panel content", b"the File Explorer Context content"),
    ],
    "workspace-model/requirements.md": [],
}

total_files = 0
total_replacements = 0

for dirpath, dirs, files in os.walk(ROOT):
    for fn in files:
        if fn != "requirements.md":
            continue
        path = os.path.join(dirpath, fn)
        rel = path.replace(ROOT + os.sep, "").replace("\\", "/")

        with open(path, "rb") as f:
            data = f.read()

        original = data
        count = 0

        # Apply global replacements
        for old, new in GLOBAL_REPLACEMENTS:
            if old in data:
                n = data.count(old)
                data = data.replace(old, new)
                log(f"  [{rel}] global: {old[:50]!r} -> {new[:50]!r} ({n}x)")
                count += n

        # Apply file-specific replacements
        key = rel
        if key in FILE_SPECIFIC:
            for item in FILE_SPECIFIC[key]:
                if not isinstance(item, tuple):
                    continue
                old, new = item
                if old in data:
                    n = data.count(old)
                    data = data.replace(old, new)
                    log(f"  [{rel}] specific: {old[:50]!r} -> {new[:50]!r} ({n}x)")
                    count += n

        if data != original:
            with open(path, "wb") as f:
                f.write(data)
            log(f"[SAVED] {rel} -- {count} replacements")
            total_files += 1
            total_replacements += count

log(f"\nDone. {total_files} files updated, {total_replacements} total replacements.")
