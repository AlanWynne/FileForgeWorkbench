#!/usr/bin/env python3
"""One-shot byte-faithful splitter for ff-encoding/src/convert.rs (PA-STD-006).

Splits the monolithic convert.rs into convert/{mod,codecs,stream}.rs by LINE
RANGE, operating on RAW BYTES so non-ASCII test fixtures (e-acute, emoji) are
preserved exactly. PowerShell Set-Content mangled these; Python binary mode
does not.

Line ranges (1-based, inclusive) determined from the source:
  header (doc + imports + 3 types + 2 public fns) : 1-91
  codecs (9 private conversion fns)                : 92-407
  stream (StreamDecoder/Encoder + helper)          : 408-535
  tests                                            : 536-667

Writes:
  convert/mod.rs    = header + module wiring + tests
  convert/codecs.rs = codecs header + body (private fns -> pub(super))
  convert/stream.rs = stream header + body

Idempotent enough for a one-shot; run once, then verify with cargo test.
"""
import os

SRC = os.path.join("crates", "ff-encoding", "src", "convert.rs")
OUTDIR = os.path.join("crates", "ff-encoding", "src", "convert")

CODECS_HEADER = (
    b"//! Encoding <-> UTF-8 codec implementations (single-byte, UTF-16, DBCS).\n"
    b"//!\n"
    b"//! Private helpers for the public conversion API in the parent module.\n"
    b"\n"
    b"use crate::encoding::Encoding;\n"
    b"use crate::error::EncodingError;\n"
    b"\n"
    b"use super::{ConversionIssue, ConversionResult, UnmappableAction};\n"
    b"\n\n"
)

STREAM_HEADER = (
    b"//! Streaming chunk-based encoder/decoder built on the conversion API.\n"
    b"\n"
    b"use crate::encoding::Encoding;\n"
    b"use crate::error::EncodingError;\n"
    b"\n"
    b"use super::{convert_from_utf8, convert_to_utf8, ConversionResult, UnmappableAction};\n"
    b"\n"
)

MOD_WIRING = (
    b"\n\n"
    b"mod codecs;\n"
    b"mod stream;\n"
    b"\n"
    b"use codecs::{\n"
    b"    convert_dbcs_to_utf8, convert_single_byte_to_utf8, convert_utf16_to_utf8,\n"
    b"    convert_utf8_to_dbcs, convert_utf8_to_single_byte, convert_utf8_to_utf16,\n"
    b"};\n"
    b"\n"
    b"pub use stream::{StreamDecoder, StreamEncoder};\n"
    b"\n\n"
)


def make_pub_super(body: bytes) -> bytes:
    """Prefix line-start `fn ` with `pub(super) ` (byte-level, line-anchored)."""
    out = []
    for line in body.split(b"\n"):
        if line.startswith(b"fn "):
            out.append(b"pub(super) " + line)
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    # Split into lines preserving content; rejoin ranges with \n.
    lines = data.split(b"\n")
    # 1-based inclusive ranges -> 0-based slices.
    header = b"\n".join(lines[0:91])       # lines 1-91
    codecs = b"\n".join(lines[91:407])     # lines 92-407
    stream = b"\n".join(lines[407:535])    # lines 408-535
    tests = b"\n".join(lines[535:])        # lines 536-end

    os.makedirs(OUTDIR, exist_ok=True)

    mod_bytes = header + MOD_WIRING + tests
    codecs_bytes = CODECS_HEADER + make_pub_super(codecs)
    stream_bytes = STREAM_HEADER + stream

    with open(os.path.join(OUTDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(OUTDIR, "codecs.rs"), "wb") as f:
        f.write(codecs_bytes)
    with open(os.path.join(OUTDIR, "stream.rs"), "wb") as f:
        f.write(stream_bytes)

    os.remove(SRC)

    print(f"mod.rs    {len(mod_bytes)} bytes")
    print(f"codecs.rs {len(codecs_bytes)} bytes")
    print(f"stream.rs {len(stream_bytes)} bytes")
    print("removed original convert.rs")


if __name__ == "__main__":
    main()
