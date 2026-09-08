import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\ct_final_fix.txt"
ROOT = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs"

with open(LOG, "w", encoding="utf-8") as f:
    f.write("CT final fix\n")

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

def fix(path, replacements):
    with open(path, "rb") as f:
        data = f.read()
    original = data
    count = 0
    for old, new in replacements:
        if old in data:
            n = data.count(old)
            data = data.replace(old, new)
            log(f"  Replaced ({n}x): {old[:70]!r}")
            count += n
        else:
            log(f"  NOT FOUND: {old[:70]!r}")
    if data != original:
        with open(path, "wb") as f:
            f.write(data)
        log(f"  SAVED ({count} replacements)")
    return count

# --- function-keys-and-history ---
p = os.path.join(ROOT, "function-keys-and-history", "requirements.md")
fix(p, [
    # L278: "settings panel" in user story
    (b"settings panel, file browser", b"Settings Context, file browser"),
    # L336: "previous screen" -- acceptable, no change needed (it's a user story metaphor)
])

# --- startup-and-session ---
p = os.path.join(ROOT, "startup-and-session", "requirements.md")
fix(p, [
    # Req 14 heading -- keep "ISPF Primary Option Menu" as it's the heritage name in the heading
    # L301: "single tab displaying the Primary Option Menu" -> "single Workspace displaying the Home Context (POM)"
    (b"a single tab displaying the Primary Option Menu.", b"a single Workspace displaying the Home Context (POM)."),
    # L308: title line format -- keep "Primary Option Menu" as it's the literal display text
    # L310: "THE Primary Option Menu SHALL display a numbered list" -- keep (it's the component name)
    # L326: "THE Primary Option Menu SHALL display a live calendar" -- keep
    # L332: "mirror the Primary Option Menu entries" -- keep (it's a reference to the component)
    # L334: "Primary Option Menu, file editor" -- fix to "Home Context (POM), Editor Context"
    (b"(Primary Option Menu, file editor, utility panel, etc.)", b"(Home Context (POM), Editor Context, utility panel, etc.)"),
    # L435/437: "WHEN the Primary Option Menu is displayed" -- keep (it's the component name in criteria)
    # L466: "File Explorer panel" in user story
    (b"I want POM option 2 to open a File Explorer panel that shows", b"I want POM option 2 to open a File Explorer Context that shows"),
    # L478: "from the Primary Option Menu (by clicking" -- keep as heritage reference
    # L490: already fixed to File_Explorer_Context
    (b"return the tab to the Primary Option Menu view", b"return the Workspace to the Home Context (POM) view"),
    (b"return the tab to the Primary Option Menu view.", b"return the Workspace to the Home Context (POM) view."),
])

# --- virtual-catalog-manager ---
p = os.path.join(ROOT, "virtual-catalog-manager", "requirements.md")
fix(p, [
    # L44: cross-reference note -- "Files panel" -> "Catalog Explorer Context"
    (b"Explorer tree reused/embedded within the Files panel |", b"Explorer tree reused/embedded within the Catalog Explorer Context |"),
    # L69: user story "Files panel" -> "Catalog Explorer Context"
    (b"I want POM option 1 to open a dedicated Files panel", b"I want POM option 1 to open a dedicated Catalog Explorer Context"),
    # L266: "from the Files panel" -> "from the Catalog Explorer Context"
    (b"directly from the Files panel, so that I can manage", b"directly from the Catalog Explorer Context, so that I can manage"),
    # L361: "from the Files panel alongside" -> "from the Catalog Explorer Context alongside"
    (b"from the Files panel alongside my mainframe", b"from the Catalog Explorer Context alongside my mainframe"),
    # L393: "content area of the Files panel" -> "content area of the Catalog Explorer Context"
    (b"content area of the Files panel to show", b"content area of the Catalog Explorer Context to show"),
    # L512: "so that the Files panel shows" -> "so that the Catalog Explorer Context shows"
    (b"so that the Files panel shows", b"so that the Catalog Explorer Context shows"),
    # L531: "visible in the Files panel" -> "visible in the Catalog Explorer Context"
    (b"visible in the Files panel on the same launch.", b"visible in the Catalog Explorer Context on the same launch."),
    # L630: "Primary Option Menu option `1`" -- keep as ISPF heritage reference
    # Workspace View remaining
    (b"Workspace View", b"Workspace"),
])

log("\nAll done.")
