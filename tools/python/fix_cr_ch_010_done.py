path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"
with open(path, "rb") as f:
    data = f.read()

old = b"cx-requirements.md` Req 3\r\n- **Status**: IN PROGRESS\r\n\r\n### CR-NR-047"
new = b"cx-requirements.md` Req 3\r\n- **Status**: DONE\r\n\r\n### CR-NR-047"

if old in data:
    data = data.replace(old, new, 1)
    with open(path, "wb") as f:
        f.write(data)
    print("CR-CH-010 updated to DONE")
else:
    old_lf = old.replace(b"\r\n", b"\n")
    new_lf = new.replace(b"\r\n", b"\n")
    if old_lf in data:
        data = data.replace(old_lf, new_lf, 1)
        with open(path, "wb") as f:
            f.write(data)
        print("CR-CH-010 updated to DONE (LF)")
    else:
        print("ERROR: pattern not found")
