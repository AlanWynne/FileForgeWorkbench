import sys

path = r"crates\ff-desktop\src\shell\update.rs"

with open(path, "rb") as f:
    data = f.read()

old = b"            if let Some(session) = &self.session {"
new = b"            if let Some(_session) = &self.session {"

if old in data:
    data = data.replace(old, new, 1)
    with open(path, "wb") as f:
        f.write(data)
    sys.stdout.write("Fixed unused session variable\n")
else:
    sys.stdout.write("Pattern not found\n")
