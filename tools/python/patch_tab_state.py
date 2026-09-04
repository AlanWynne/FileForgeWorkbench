LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\patch_tab_state.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\tab_state.rs"

with open(path, "rb") as f:
    raw = f.read()

log(f"File size: {len(raw)} bytes")
text = raw.decode("utf-8")

old = "            is_floating: false,\n        }"
new = "            is_floating: false,\n            edit_profile: EditProfile::new(),\n        }"

count = text.count(old)
log(f"Occurrences to patch: {count}")

text = text.replace(old, new)

with open(path, "wb") as f:
    f.write(text.encode("utf-8"))

log("Done")
