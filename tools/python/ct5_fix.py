LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\ct5_fix.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "w", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\caret-and-selection\requirements.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

replacements = [
    (
        b"I want to be able to select and copy text from read-only panels (POM option descriptions, Settings panel values, status bar messages)",
        b"I want to be able to select and copy text from read-only Contexts (POM option descriptions, Settings Context values, status bar messages)"
    ),
    (
        b"14.1 WHEN text is rendered in the Primary Option Menu panel (option labels, descriptions, calendar text)",
        b"14.1 WHEN text is rendered in the Home Context (POM) (option labels, descriptions, calendar text)"
    ),
    (
        b"14.2 WHEN text is rendered in the Settings panel (key names, values, descriptions)",
        b"14.2 WHEN text is rendered in the Settings Context (key names, values, descriptions)"
    ),
    (
        b"14.4 WHEN the user selects text in a read-only panel and presses Ctrl+C",
        b"14.4 WHEN the user selects text in a read-only Context and presses Ctrl+C"
    ),
    (
        b"14.5 THE selectable label behaviour SHALL NOT interfere with existing click-to-navigate interactions (POM option buttons, Settings edit fields).",
        b"14.5 THE selectable label behaviour SHALL NOT interfere with existing click-to-navigate interactions (Home Context option buttons, Settings Context edit fields)."
    ),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new)
        log(f"Replaced: {old[:70]!r}")
        count += 1
    else:
        log(f"NOT FOUND: {old[:70]!r}")

with open(path, "wb") as f:
    f.write(data)

log(f"Done. {count} replacements.")
