import sys

path = r"crates\ff-desktop\src\shell\update.rs"

with open(path, "rb") as f:
    data = f.read()

# The broken line contains the closing brace followed by the old comment text
# We need to replace it with just the closing brace followed by a newline
# The broken text (in UTF-8 bytes) is:
# "        } \xe2\x80\x94 Validates: Requirement 18.1, 18.4 \xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80\xe2\x94\x80"

# Find the broken pattern - look for "        }" followed by the em dash
import re

# Replace the broken line: "        } <em-dash> Validates: Requirement 18.1, 18.4 <box-chars>"
# with just "        }"
# The em dash is \xe2\x80\x94 in UTF-8
broken = b"        } \xe2\x80\x94 Validates: Requirement 18.1, 18.4 "
# Find it
idx = data.find(broken)
if idx >= 0:
    # Find end of line
    end = data.find(b"\r\n", idx)
    if end < 0:
        end = data.find(b"\n", idx)
    if end >= 0:
        sep = b"\r\n" if data[end:end+2] == b"\r\n" else b"\n"
        data = data[:idx] + b"        }" + sep + data[end+len(sep):]
        with open(path, "wb") as f:
            f.write(data)
        sys.stdout.write("Fixed\n")
    else:
        sys.stdout.write("Could not find end of line\n")
else:
    sys.stdout.write(f"Pattern not found, searching...\n")
    # Try to find the em dash anywhere
    em = data.find(b"\xe2\x80\x94")
    if em >= 0:
        sys.stdout.write(f"Em dash at offset {em}: {repr(data[em-20:em+40])}\n")
    else:
        sys.stdout.write("No em dash found\n")
