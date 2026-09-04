path = r"docs\specs\project-master\tasks.md"
with open(path, "rb") as f:
    data = f.read()

old = b"Tasks 27.1\xe2\x80\x9327.4"
new = b"Tasks 27.1-27.4"
if old in data:
    data = data.replace(old, new, 1)
    with open(path, "wb") as f:
        f.write(data)
    print("Fixed")
else:
    print("Pattern not found")
