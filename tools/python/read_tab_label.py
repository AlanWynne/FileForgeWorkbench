path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render_chrome.rs"
with open(path, "rb") as f:
    data = f.read()

# Find the tab title rendering
idx = data.find(b"tab.title.clone()")
print(f"tab.title.clone() at byte {idx}")
print(repr(data[max(0,idx-300):idx+200]))
