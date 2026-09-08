path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\key_config_dialog.rs"
with open(path, "rb") as f:
    data = f.read()

idx = data.find(b"pub struct KeyConfigDialog")
print(repr(data[idx:idx+700]))
