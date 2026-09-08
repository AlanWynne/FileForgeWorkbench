path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"
with open(path, "rb") as f:
    data = f.read()

replacements = [
    # CR-NR-046
    (
        b"- **Status**: IN PROGRESS\r\n- **Linked spec**: `docs/specs/function-keys-and-history/requirements.md` (extension to Req 20), `docs/specs/function-keys-and-history/cx-requirements.md` (new)",
        b"- **Status**: DONE\r\n- **Linked spec**: `docs/specs/function-keys-and-history/requirements.md` (extension to Req 20), `docs/specs/function-keys-and-history/cx-requirements.md` (new)"
    ),
    # CR-CH-010
    (
        b"- **Status**: IN PROGRESS\r\n- **Affects**: `docs/specs/layout-and-docking/requirements.md` Req 3; `ff-desktop` command handler; `docs/specs/function-keys-and-history/cx-requirements.md` Req 3",
        b"- **Status**: DONE\r\n- **Affects**: `docs/specs/layout-and-docking/requirements.md` Req 3; `ff-desktop` command handler; `docs/specs/function-keys-and-history/cx-requirements.md` Req 3"
    ),
]

for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        print(f"Updated: {old[:50]!r}...")
    else:
        old_lf = old.replace(b"\r\n", b"\n")
        new_lf = new.replace(b"\r\n", b"\n")
        if old_lf in data:
            data = data.replace(old_lf, new_lf, 1)
            print(f"Updated (LF): {old_lf[:50]!r}...")
        else:
            print(f"WARNING: not found: {old[:50]!r}...")

with open(path, "wb") as f:
    f.write(data)
print("Done")
