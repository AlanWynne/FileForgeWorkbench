path = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\session_manager.rs"
lines = open(path, encoding="utf-8").readlines()
for i, l in enumerate(lines[540:], 541):
    print(f"{i}: {l}", end="")
