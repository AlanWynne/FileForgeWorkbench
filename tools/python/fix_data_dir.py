import sys

path = r"crates\ff-desktop\src\shell\update.rs"

with open(path, "rb") as f:
    data = f.read()

old = b"                    crate::menu_workspace::defaults::ensure_default_menu_files(udd.data_dir());"
new = b"                    crate::menu_workspace::defaults::ensure_default_menu_files(udd.path());"

if old in data:
    data = data.replace(old, new)
    with open(path, "wb") as f:
        f.write(data)
    sys.stdout.write("Fixed data_dir -> path\n")
else:
    sys.stdout.write("Pattern not found\n")
