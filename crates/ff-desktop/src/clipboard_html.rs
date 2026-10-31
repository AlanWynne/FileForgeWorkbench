//! CF_HTML composite builder for the picture SNAPSHOT (screen-snapshot-scrm
//! Requirement 23.5c, 23.5d).
//!
//! Word and other rich HTML-aware targets read the Windows registered
//! "HTML Format" clipboard flavor, whose payload is a UTF-8 fragment prefixed by
//! the CF_HTML header. The header carries four BYTE offsets -- `StartHTML`,
//! `EndHTML`, `StartFragment`, `EndFragment` -- that index into the SAME string
//! (counted in UTF-8 bytes from offset 0). This module builds that payload from
//! the PNG picture (inlined as a base64 `data:` image) plus the box-drawn screen
//! text (inside a monospaced `<pre>`), so a Word paste takes the WHOLE composite:
//! the window picture AND the readable box-drawn text, not one or the other.
//!
//! base64 and HTML-escaping are implemented in-module so no new dependency is
//! pulled in (the CF_HTML flavor reuses `clipboard-win` for placement only).
//!
//! Validates: screen-snapshot-scrm Requirement 23.5c, 23.5d.

/// Build the CF_HTML payload: the picture (base64 `data:image/png` `<img>`) and
/// the box-drawn `pre_text` (HTML-escaped inside a monospaced `<pre>`), wrapped
/// in the CF_HTML header with correct UTF-8 BYTE offsets and the
/// `<!--StartFragment-->` / `<!--EndFragment-->` markers.
///
/// The header is written in two passes: a FIRST pass with zero-padded,
/// fixed-width (10-digit) placeholder offsets so the header length is known and
/// stable, then the real offsets patched in. Fixed width keeps every offset's
/// byte position unchanged between the two passes.
///
/// Validates: Req 23.5c, 23.5d.
//
// Wired into the live SNAPSHOT payload assembly in FEAT-002
// (`shell/commands_snapshot::place_snapshot_composite`).
pub fn build_cf_html(png: &[u8], pre_text: &str) -> String {
    let body = fragment_html(png, pre_text);

    // The HTML document around the fragment markers.
    let doc_prefix = "<html>\r\n<body>\r\n<!--StartFragment-->";
    let doc_suffix = "<!--EndFragment-->\r\n</body>\r\n</html>";

    // Build the header with 10-digit zero-padded placeholders so its byte length
    // is fixed, then compute the real offsets against that fixed length.
    let header_template = format!(
        "Version:0.9\r\n\
         StartHTML:{ph}\r\n\
         EndHTML:{ph}\r\n\
         StartFragment:{ph}\r\n\
         EndFragment:{ph}\r\n",
        ph = "0000000000"
    );

    let header_len = header_template.len();
    let start_html = header_len;
    let start_fragment = start_html + doc_prefix.len();
    let end_fragment = start_fragment + body.len();
    let end_html = end_fragment + doc_suffix.len();

    let header = format!(
        "Version:0.9\r\n\
         StartHTML:{start_html:010}\r\n\
         EndHTML:{end_html:010}\r\n\
         StartFragment:{start_fragment:010}\r\n\
         EndFragment:{end_fragment:010}\r\n"
    );
    debug_assert_eq!(header.len(), header_len, "fixed-width header offsets");

    format!("{header}{doc_prefix}{body}{doc_suffix}")
}

/// The fragment markup: the inlined PNG picture above the box-drawn `<pre>` text.
fn fragment_html(png: &[u8], pre_text: &str) -> String {
    format!(
        "<img src=\"data:image/png;base64,{}\">\r\n\
         <pre style=\"font-family:monospace\">{}</pre>",
        base64_encode(png),
        html_escape(pre_text)
    )
}

/// Escape the five characters that are significant in HTML text / attributes so
/// the box-drawn text renders literally inside the `<pre>`.
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Standard base64 alphabet (RFC 4648), no line wrapping.
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode `data` as standard base64 with `=` padding, in-module (no dependency).
fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64[((n >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64[((n >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[(n & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse the integer value following `key:` in the CF_HTML header.
    fn header_int(html: &str, key: &str) -> usize {
        let start = html.find(key).expect("header key present") + key.len();
        let rest = &html[start..];
        let end = rest.find("\r\n").expect("header line terminated");
        rest[..end].parse().expect("header value is an integer")
    }

    // Validates: Req 23.5c, 23.5d -- the four CF_HTML header offsets index the
    // right bytes: StartFragment/EndFragment bracket the fragment markers and
    // StartHTML/EndHTML bracket the whole document.
    #[test]
    fn cf_html_header_offsets_index_the_markers() {
        let png = [0u8, 1, 2, 3, 4, 5, 6, 7];
        let html = build_cf_html(&png, "box");
        let bytes = html.as_bytes();

        let start_html = header_int(&html, "StartHTML:");
        let end_html = header_int(&html, "EndHTML:");
        let start_fragment = header_int(&html, "StartFragment:");
        let end_fragment = header_int(&html, "EndFragment:");

        // StartHTML points at the document start "<html>".
        assert!(
            bytes[start_html..].starts_with(b"<html>"),
            "StartHTML indexes the document start"
        );
        // StartFragment points just AFTER the StartFragment comment marker.
        assert!(
            html[..start_fragment].ends_with("<!--StartFragment-->"),
            "StartFragment indexes just after the start marker"
        );
        // EndFragment points just BEFORE the EndFragment comment marker.
        assert!(
            html[end_fragment..].starts_with("<!--EndFragment-->"),
            "EndFragment indexes just before the end marker"
        );
        // EndHTML is the end of the whole payload.
        assert_eq!(end_html, bytes.len(), "EndHTML indexes the document end");
        assert!(
            start_html < start_fragment && start_fragment < end_fragment,
            "offsets are strictly ordered"
        );
    }

    // Validates: Req 23.5d -- the picture is inlined as a base64 data URL and
    // the box-drawn text sits inside the monospaced <pre>.
    #[test]
    fn cf_html_inlines_png_and_pre_text() {
        let png = [0x89, b'P', b'N', b'G'];
        let html = build_cf_html(&png, "USER.TEST");
        assert!(
            html.contains("data:image/png;base64,"),
            "the PNG is inlined as a base64 data URL"
        );
        assert!(
            html.contains("<pre style=\"font-family:monospace\">USER.TEST</pre>"),
            "the box-drawn text is inside the monospaced <pre>"
        );
    }

    // Validates: Req 23.5d -- a Unicode box glyph survives into the <pre> and
    // HTML-significant characters are escaped. The expected glyph is built from a
    // char constant (no raw box glyph in source).
    #[test]
    fn cf_html_escapes_and_preserves_box_glyph() {
        let glyph = '\u{250C}'; // top-left corner
        let text = format!("{glyph}<a>&\"'");
        let html = build_cf_html(&[0u8], &text);
        assert!(
            html.contains(glyph),
            "the box glyph survives into the <pre>"
        );
        assert!(html.contains("&lt;a&gt;"), "angle brackets are escaped");
        assert!(html.contains("&amp;"), "ampersand is escaped");
        assert!(html.contains("&quot;"), "double quote is escaped");
        assert!(html.contains("&#39;"), "single quote is escaped");
    }

    // Validates: Req 23.5d -- base64 encodes with correct padding for each
    // input-length residue (0, 1, 2 bytes beyond a 3-byte group).
    #[test]
    fn base64_padding_is_correct() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"M"), "TQ==");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"Many"), "TWFueQ==");
    }
}
