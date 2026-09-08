path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"
with open(path, "rb") as f:
    data = f.read()

idx = data.find(b"CR-CH-010")
print(repr(data[idx+400:idx+750]))
