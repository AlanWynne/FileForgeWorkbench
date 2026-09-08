path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\key_config_dialog.rs"
with open(path, "rb") as f:
    data = f.read()

# Find render function
idx = data.find(b"pub fn render(")
print(f"render at byte {idx}")
print(repr(data[idx:idx+500]))
