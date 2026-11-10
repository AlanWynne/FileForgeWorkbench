"""READ-ONLY harness: replicate crates/ff-desktop/src/clipboard_html.rs::build_cf_html
exactly (byte-for-byte) to dump the literal CF_HTML string and validate the
header offsets. No source code is changed; this only reconstructs the builder's
output for inspection. Offsets in CF_HTML are UTF-8 BYTE counts."""

import base64

B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
MONOSPACE_FAMILY = "'Consolas','Courier New',monospace"
PRE_WHITESPACE_STYLE = "white-space:pre;"


def html_escape(s: str) -> str:
    out = []
    for ch in s:
        if ch == '&':
            out.append("&amp;")
        elif ch == '<':
            out.append("&lt;")
        elif ch == '>':
            out.append("&gt;")
        elif ch == '"':
            out.append("&quot;")
        elif ch == "'":
            out.append("&#39;")
        else:
            out.append(ch)
    return "".join(out)


def base64_encode(data: bytes) -> str:
    # Standard base64, matches the in-module encoder (RFC 4648, no wrapping).
    return base64.b64encode(data).decode("ascii")


def wrap_monospace_runs(escaped_text: str) -> str:
    return (
        '<font face="Consolas,\'Courier New\',monospace">'
        f'<span style="font-family:{MONOSPACE_FAMILY}">{escaped_text}</span>'
        '</font>'
    )


def fragment_html(png: bytes, pre_text: str) -> str:
    return (
        f'<img src="data:image/png;base64,{base64_encode(png)}">\r\n'
        f'<pre style="{PRE_WHITESPACE_STYLE}">{wrap_monospace_runs(html_escape(pre_text))}</pre>'
    )


def build_cf_html(png: bytes, pre_text: str) -> str:
    body = fragment_html(png, pre_text)
    doc_prefix = "<html>\r\n<body>\r\n<!--StartFragment-->"
    doc_suffix = "<!--EndFragment-->\r\n</body>\r\n</html>"

    ph = "0000000000"
    header_template = (
        f"Version:0.9\r\n"
        f"StartHTML:{ph}\r\n"
        f"EndHTML:{ph}\r\n"
        f"StartFragment:{ph}\r\n"
        f"EndFragment:{ph}\r\n"
    )
    # Rust: header_template.len() is BYTE length. doc_prefix/body/doc_suffix
    # .len() are also byte lengths. Replicate with UTF-8 byte lengths.
    header_len = len(header_template.encode("utf-8"))
    start_html = header_len
    start_fragment = start_html + len(doc_prefix.encode("utf-8"))
    end_fragment = start_fragment + len(body.encode("utf-8"))
    end_html = end_fragment + len(doc_suffix.encode("utf-8"))

    header = (
        f"Version:0.9\r\n"
        f"StartHTML:{start_html:010d}\r\n"
        f"EndHTML:{end_html:010d}\r\n"
        f"StartFragment:{start_fragment:010d}\r\n"
        f"EndFragment:{end_fragment:010d}\r\n"
    )
    assert len(header.encode("utf-8")) == header_len, "fixed-width header"
    return header + doc_prefix + body + doc_suffix


def validate(html: str):
    data = html.encode("utf-8")

    def header_int(key: str) -> int:
        i = html.index(key) + len(key)
        j = html.index("\r\n", i)
        return int(html[i:j])

    sh = header_int("StartHTML:")
    eh = header_int("EndHTML:")
    sf = header_int("StartFragment:")
    ef = header_int("EndFragment:")

    print(f"StartHTML    = {sh}")
    print(f"EndHTML      = {eh}")
    print(f"StartFragment= {sf}")
    print(f"EndFragment  = {ef}")
    print(f"payload bytes= {len(data)}")
    print()
    print("CHECK StartHTML points at '<html>':",
          data[sh:sh + 6] == b"<html>", repr(data[sh:sh + 8]))
    # BYTE-based checks (correct way: offsets are byte counts into UTF-8).
    print("BYTE CHECK StartHTML -> '<html>':", data[sh:sh + 6] == b"<html>")
    print("BYTE CHECK bytes before StartFragment end with marker:",
          data[:sf].endswith(b"<!--StartFragment-->"))
    print("BYTE CHECK EndFragment -> just before marker:",
          data[ef:].startswith(b"<!--EndFragment-->"))
    print("BYTE CHECK EndHTML == payload byte len:", eh == len(data))
    # The fragment the offsets bracket, decoded:
    print("FRAGMENT [StartFragment:EndFragment] =")
    print(data[sf:ef].decode("utf-8"))
    # What byte actually sits at StartFragment
    print("bytes AT StartFragment:", repr(data[sf:sf + 12]))
    print("bytes AT EndFragment:", repr(data[ef:ef + 18]))


if __name__ == "__main__":
    import sys
    sys.stdout.reconfigure(encoding="utf-8")
    # A small but realistic PNG stand-in + a multi-byte box-drawing dialog body.
    png = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] + list(range(60)))
    pre_text = (
        "\u250c\u2500\u2500\u2500\u2500\u2500\u2510\r\n"
        "\u2502 Save As  \u2502\r\n"
        "\u2514\u2500\u2500\u2500\u2500\u2500\u2518"
    )
    html = build_cf_html(png, pre_text)
    with open("cfhtml_dump.txt", "w", encoding="utf-8", newline="") as f:
        f.write(html)
    print("=== LITERAL CF_HTML (repr, CRLF visible) ===")
    print(repr(html))
    print()
    print("=== VALIDATION ===")
    validate(html)
