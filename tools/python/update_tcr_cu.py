import sys, re

path = r"docs\quality\TCR.md"

with open(path, "rb") as f:
    data = f.read()

# Replace all 🔴 in the Phase CU section with ✅
# Find the section header
header = b"### Phase CU -- Menu Workspace Pattern (CR-NR-045)"
idx = data.find(header)
if idx < 0:
    sys.stdout.write("ERROR: Phase CU section not found\n")
    sys.exit(1)

# Find the next section header after CU
next_section = data.find(b"\n### ", idx + len(header))
if next_section < 0:
    next_section = len(data)

section = data[idx:next_section]
sys.stdout.write(f"Section length: {len(section)} bytes\n")

# Count 🔴 in section
red = section.count("\U0001f534".encode("utf-8"))
sys.stdout.write(f"Red circles in section: {red}\n")

# Replace 🔴 | -- | with ✅ | `menu_workspace` unit tests |
old_pattern = "\U0001f534".encode("utf-8") + b" | -- |"
new_pattern = "\u2705".encode("utf-8") + b" | `menu_workspace` unit tests |"

new_section = section.replace(old_pattern, new_pattern)
data = data[:idx] + new_section + data[next_section:]

with open(path, "wb") as f:
    f.write(data)

sys.stdout.write("TCR CU rows updated to PASS\n")
