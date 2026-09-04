LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\bw_group1_patch.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\edit-operations\tasks.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")
log(f"Has CRLF: {b'\\r\\n' in data}")
log(f"Has LF-only: {b'\\n' in data}")

# Find "28. CAPS" in raw bytes
idx = data.find(b"28. CAPS")
if idx >= 0:
    snippet = data[max(0, idx-10):idx+30]
    log(f"Raw bytes around '28. CAPS': {repr(snippet)}")
else:
    log("'28. CAPS' not found in raw bytes")

# Count lines split by \r\n vs \n
crlf_lines = data.split(b"\r\n")
lf_lines = data.split(b"\n")
log(f"Split by CRLF: {len(crlf_lines)} parts")
log(f"Split by LF: {len(lf_lines)} parts")
log("Done")
