//! CF_HTML composite builder for the picture SNAPSHOT (screen-snapshot-scrm
//! Requirement 23.5c, 23.5d).
//!
//! Word and other rich HTML-aware targets read the Windows registered
//! "HTML Format" clipboard flavor, whose payload is a UTF-8 fragment prefixed by
//! the CF_HTML header. The header carries four BYTE offsets -- `StartHTML`,
//! `EndHTML`, `StartFragment`, `EndFragment` -- that index into the SAME string
//! (counted in UTF-8 bytes from offset 0). This module builds that payload from
//! the PNG picture (inlined as a base64 `data:` image) plus the box-drawn screen
//! text, so a Word paste takes the WHOLE composite: the window picture AND the
//! readable box-drawn text, not one or the other.
//!
//! ## One monospace `<span>` paragraph per line (Req 23.5f)
//!
//! Earlier attempts emitted the whole box as a single `<pre>` with the font on
//! inner `<font>`/`<span>` runs. Word IGNORED the run font: it reshapes the
//! `<pre>` into its own `MsoNormal` paragraph context and resolves the font from
//! that paragraph (its proportional `Aptos` default), dropping our run font, so
//! the columns misaligned.
//!
//! The owner captured Word's OWN HTML export of the SAME pasted dialog in both a
//! broken (proportional) and a fixed (monospaced) form. The monospaced export
//! proves the structure Word actually keeps: ONE PARAGRAPH PER LINE, each line's
//! text wrapped in a `<span style='font-family:Consolas'>`, e.g.
//! `<p class=MsoNormal><span style='font-family:Consolas'>...line...</span></p>`.
//! A font on a per-line `<span>` inside a `<p>` is what Word preserves.
//!
//! So the builder now emits, for each of the N rendered screen lines, its own
//! `<p style="margin:0; white-space:pre;">` whose text is wrapped in a
//! `<span style="font-family:'Consolas','Courier New',monospace">`. `margin:0`
//! stops Word inserting inter-paragraph spacing; `white-space:pre` keeps runs of
//! spaces; the per-line `<span>` font is the proven, essential lever. A
//! document-level monospace default in `<head><style>` is added as
//! belt-and-braces, mirroring the `@font-face`/default Word's good export
//! declares -- but the per-line `<span>` font, not the style block, is what
//! forces fixed-width on paste.
//!
//! base64 and HTML-escaping are implemented in-module so no new dependency is
//! pulled in (the CF_HTML flavor reuses `clipboard-win` for placement only).
//!
//! Validates: screen-snapshot-scrm Requirement 23.5c, 23.5d.

/// Build the CF_HTML payload: the picture (base64 `data:image/png` `<img>`), the
/// box-drawn `pre_text` (HTML-escaped, emitted as one monospace `<span>`
/// paragraph per line), and the property/value `tables_html` (the
/// `ff_screen_model` HTML-table emitter output, appended AFTER the box in
/// image/box/tables order, Req 28.2/28.4), wrapped in the CF_HTML header with
/// correct UTF-8 BYTE offsets and the `<!--StartFragment-->` /
/// `<!--EndFragment-->` markers. `tables_html` is empty for a box-only model, in
/// which case the payload is byte-identical to the pre-Req-28 output (Req 28.7).
///
/// The header is written in two passes: a FIRST pass with zero-padded,
/// fixed-width (10-digit) placeholder offsets so the header length is known and
/// stable, then the real offsets patched in. Fixed width keeps every offset's
/// byte position unchanged between the two passes. The `<head><style>` default
/// and the longer per-line fragment both sit inside the byte math, so offsets
/// stay correct automatically.
///
/// Validates: Req 23.5c, 23.5d, 23.5f.
//
// Wired into the live SNAPSHOT payload assembly in FEAT-002
// (`shell/commands_snapshot::place_snapshot_composite`).
pub fn build_cf_html(png: &[u8], pre_text: &str, tables_html: &str) -> String {
    let body = fragment_html(png, pre_text, tables_html);

    // The HTML document around the fragment markers. A document-level monospace
    // default in <head><style> is belt-and-braces (mirrors the @font-face Word's
    // good export declares); the per-line <span> font is the essential lever.
    let doc_prefix = concat!(
        "<html>\r\n",
        "<head>\r\n",
        "<style>\r\n",
        "p { margin:0; }\r\n",
        "p, span { font-family:'Consolas','Courier New',monospace; }\r\n",
        "</style>\r\n",
        "</head>\r\n",
        "<body>\r\n<!--StartFragment-->"
    );
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

/// The monospace font-family list, single-quoted so it nests safely inside the
/// double-quoted HTML `style` attribute. Carried on EACH line's `<span>` (Req
/// 23.5f). Fixed-width families with `monospace` as the final fallback.
const MONOSPACE_FAMILY: &str = "'Consolas','Courier New',monospace";

/// The per-line paragraph style: `margin:0` stops Word inserting inter-paragraph
/// spacing between the box lines, `white-space:pre` preserves runs of spaces so
/// the columns stay aligned.
const LINE_PARAGRAPH_STYLE: &str = "margin:0; white-space:pre;";

/// Emit one monospace paragraph for a SINGLE (already HTML-escaped) line: a
/// `<p>` carrying the no-spacing / preserve-whitespace style, wrapping a `<span>`
/// that carries the monospace font-family. This per-line `<p><span>` shape is the
/// exact structure Word preserved in the owner's monospaced export (Req 23.5f).
///
/// A blank line emits a present-but-empty paragraph (`&nbsp;` inside the span) so
/// the line count and vertical alignment of the box are retained.
fn line_paragraph(escaped_line: &str) -> String {
    let content = if escaped_line.is_empty() {
        "&nbsp;"
    } else {
        escaped_line
    };
    format!(
        "<p style=\"{LINE_PARAGRAPH_STYLE}\">\
         <span style=\"font-family:{MONOSPACE_FAMILY}\">{content}</span></p>"
    )
}

/// The fragment markup: the inlined PNG picture in its own leading paragraph
/// (Word's export shows the image in its own `MsoNormal` paragraph), then the
/// box-drawn text emitted as ONE monospace `<span>` paragraph per line. Lines are
/// split on `\n` (a trailing `\r` is stripped) so one `<p>` is emitted per
/// screen line, blank lines included (Req 23.5f).
fn fragment_html(png: &[u8], pre_text: &str, tables_html: &str) -> String {
    let mut out = format!(
        "<p><img src=\"data:image/png;base64,{}\"></p>\r\n",
        base64_encode(png)
    );
    // Strip a SINGLE trailing newline so a body that ends in "\n" does not emit a
    // spurious extra blank paragraph; interior blank lines are preserved.
    let body = pre_text
        .strip_suffix('\n')
        .map(|s| s.strip_suffix('\r').unwrap_or(s))
        .unwrap_or(pre_text);
    for line in body.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        out.push_str(&line_paragraph(&html_escape(line)));
    }
    // screen-snapshot-scrm Req 28.2/28.4: after the image paragraph and the
    // per-line monospace box paragraphs, append the property/value HTML tables
    // (box-before-tables order). `tables_html` is the `ff_screen_model`
    // HTML-table emitter output; it is already well-formed HTML and is NOT
    // re-escaped. Empty for a box-only model (Req 28.7), leaving the fragment
    // byte-identical to the pre-change output.
    if !tables_html.is_empty() {
        out.push_str("\r\n");
        out.push_str(tables_html);
    }
    out
}

/// Escape the five characters that are significant in HTML text / attributes so
/// the box-drawn text renders literally inside each line's `<span>`.
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
        let html = build_cf_html(&png, "box", "");
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

    // Validates: Req 23.5d, 23.5f -- the picture is inlined as a base64 data URL
    // in its OWN leading paragraph, and each box line is one monospace <span>
    // paragraph (the Word-honored shape), NOT a single <pre>.
    #[test]
    fn cf_html_inlines_png_and_pre_text() {
        let png = [0x89, b'P', b'N', b'G'];
        let html = build_cf_html(&png, "USER.TEST", "");
        assert!(
            html.contains("data:image/png;base64,"),
            "the PNG is inlined as a base64 data URL"
        );
        assert!(
            html.contains("<p><img src=\"data:image/png;base64,"),
            "the image sits in its own leading paragraph"
        );
        assert!(
            html.contains(
                "<p style=\"margin:0; white-space:pre;\">\
                 <span style=\"font-family:'Consolas','Courier New',monospace\">USER.TEST</span></p>"
            ),
            "each box line is a <p> wrapping a monospace <span> (the Word-honored shape)"
        );
        assert!(
            !html.contains("<pre"),
            "the box text is NOT inside a <pre> anymore"
        );
    }

    // Validates: Req 23.5f -- the fragment emits ONE paragraph per screen line,
    // including blank lines, and the count matches the line count exactly. A
    // trailing newline does not add a spurious extra paragraph.
    #[test]
    fn cf_html_emits_one_paragraph_per_line() {
        let png = [0u8];
        // Three content lines with a blank middle line, plus a trailing newline.
        let body = "LINE.ONE\n\nLINE.THREE\n";
        let html = build_cf_html(&png, body, "");

        // One <p ...><span ...> line paragraph per screen line (3), excluding the
        // image paragraph (which is a bare <p><img>, no line-paragraph style).
        let line_para = "<p style=\"margin:0; white-space:pre;\">";
        let count = html.matches(line_para).count();
        assert_eq!(count, 3, "one line paragraph per screen line incl. blanks");

        // The blank middle line is present as a non-collapsing &nbsp; span.
        assert!(
            html.contains(
                "<p style=\"margin:0; white-space:pre;\">\
                 <span style=\"font-family:'Consolas','Courier New',monospace\">&nbsp;</span></p>"
            ),
            "a blank line is retained as a present-but-empty paragraph"
        );
    }

    // Validates: Req 23.5f -- the SECRET is redacted before the fragment is built
    // (single-point redaction): the fragment carries only the placeholder, never
    // the secret value. The caller redacts the model; this asserts build_cf_html
    // faithfully emits whatever (redacted) text it is handed and never the secret.
    #[test]
    fn cf_html_contains_placeholder_not_secret() {
        let png = [0u8];
        // Simulate the already-redacted body the single-point redaction produces.
        let redacted = "Token . . . : ***REDACTED***";
        let html = build_cf_html(&png, redacted, "");
        assert!(
            html.contains("***REDACTED***"),
            "the redaction placeholder is present in the fragment"
        );
        assert!(
            !html.contains("hunter2"),
            "no secret value appears in the fragment"
        );
    }

    // Validates: Req 23.5f -- each line's text is wrapped in a <span> carrying a
    // monospace font-family INSIDE a <p>, matching the owner's monospaced Word
    // export. The box text is NOT inside a bare <pre>, and the font-carrying
    // element is the per-line <span>, not a block element Word would remap.
    #[test]
    fn cf_html_applies_monospace_font_to_each_line_span() {
        let html = build_cf_html(&[0u8], "USER.TEST", "");
        // No <pre> anywhere -- the single-<pre> shape Word dropped the font from
        // is gone.
        assert!(
            !html.contains("<pre"),
            "the box text is no longer inside a <pre> (23.5f)"
        );
        // The monospace font is on a per-line <span> wrapping the text.
        let span_open = "<span style=\"font-family:'Consolas','Courier New',monospace\">";
        assert!(
            html.contains(span_open),
            "a <span> with the monospace family wraps each line's text (23.5f)"
        );
        // That span sits directly inside the per-line <p>, and wraps the text.
        let idx = html.find(span_open).expect("span present") + span_open.len();
        assert!(
            html[idx..].starts_with("USER.TEST</span></p>"),
            "the line text is wrapped by the monospace span inside the line <p>"
        );
        assert!(
            html.contains("Consolas") && html.contains("Courier New"),
            "concrete fixed-width families are named as fallbacks (23.5f)"
        );
    }

    // Validates: Req 23.5d, 23.5f -- after the per-line monospace markup change
    // the four CF_HTML header offsets still round-trip exactly to the fragment
    // markers (the longer per-line fragment + <head> must not desynchronise the
    // byte offsets).
    #[test]
    fn cf_html_offsets_round_trip_after_monospace_style() {
        let png = [0x89, b'P', b'N', b'G', 0, 1, 2, 3];
        // A box glyph exercises multi-byte UTF-8 in the fragment body.
        let text = format!("{}USER.TEST", '\u{250C}');
        let html = build_cf_html(&png, &text, "");

        let start_html = header_int(&html, "StartHTML:");
        let end_html = header_int(&html, "EndHTML:");
        let start_fragment = header_int(&html, "StartFragment:");
        let end_fragment = header_int(&html, "EndFragment:");

        assert!(
            html[..start_fragment].ends_with("<!--StartFragment-->"),
            "StartFragment still indexes just after the start marker"
        );
        assert!(
            html[end_fragment..].starts_with("<!--EndFragment-->"),
            "EndFragment still indexes just before the end marker"
        );
        assert!(
            html.as_bytes()[start_html..].starts_with(b"<html>"),
            "StartHTML still indexes the document start"
        );
        assert_eq!(
            end_html,
            html.len(),
            "EndHTML still indexes the document end"
        );
        // The per-line monospace span lives inside the fragment the offsets
        // bracket (not in the <head> style block before StartFragment).
        assert!(
            html[start_fragment..end_fragment]
                .contains("<span style=\"font-family:'Consolas','Courier New',monospace\">"),
            "the per-line monospace span lives inside the fragment the offsets bracket"
        );
    }

    // Validates: Req 23.5d -- a Unicode box glyph survives into the line span and
    // HTML-significant characters are escaped. The expected glyph is built from a
    // char constant (no raw box glyph in source).
    #[test]
    fn cf_html_escapes_and_preserves_box_glyph() {
        let glyph = '\u{250C}'; // top-left corner
        let text = format!("{glyph}<a>&\"'");
        let html = build_cf_html(&[0u8], &text, "");
        assert!(
            html.contains(glyph),
            "the box glyph survives into the line span"
        );
        assert!(html.contains("&lt;a&gt;"), "angle brackets are escaped");
        assert!(html.contains("&amp;"), "ampersand is escaped");
        assert!(html.contains("&quot;"), "double quote is escaped");
        assert!(html.contains("&#39;"), "single quote is escaped");
    }

    // Validates: screen-snapshot-scrm Req 28.2, 28.4 -- the composite CF_HTML
    // carries, in order, the base64 `<img>`, the monospace box `<span>`
    // paragraphs, AND the property/value `<table>` markup appended after the box.
    #[test]
    fn cf_html_carries_image_box_then_tables_in_order() {
        let png = [0x89, b'P', b'N', b'G'];
        let tables = "  <table class=\"ffwb-table\">\n    \
             <tr><th>Property</th><th>Value</th></tr>\n    \
             <tr><td>editor.tab_size</td><td>4</td></tr>\n  </table>\n";
        let html = build_cf_html(&png, "BOX.LINE", tables);

        let img_idx = html
            .find("<p><img src=\"data:image/png;base64,")
            .expect("image paragraph present");
        let span_idx = html
            .find("<span style=\"font-family:'Consolas','Courier New',monospace\">BOX.LINE")
            .expect("monospace box span present");
        let table_idx = html
            .find("<table class=\"ffwb-table\">")
            .expect("property/value table present");
        assert!(
            img_idx < span_idx && span_idx < table_idx,
            "order is image, then box, then tables (Req 28.4)"
        );
        assert!(html.contains("<td>editor.tab_size</td><td>4</td>"));
    }

    // Validates: screen-snapshot-scrm Req 28.2, 28.5 -- with the tables appended,
    // the four CF_HTML header offsets still round-trip to the fragment markers,
    // and the tables live INSIDE the bracketed fragment.
    #[test]
    fn cf_html_offsets_round_trip_with_tables_present() {
        let png = [0x89, b'P', b'N', b'G', 0, 1, 2, 3];
        let tables = "  <table class=\"ffwb-table\">\n    \
             <tr><td>k</td><td>v</td></tr>\n  </table>\n";
        let html = build_cf_html(&png, "USER.TEST", tables);

        let start_html = header_int(&html, "StartHTML:");
        let end_html = header_int(&html, "EndHTML:");
        let start_fragment = header_int(&html, "StartFragment:");
        let end_fragment = header_int(&html, "EndFragment:");

        assert!(html[..start_fragment].ends_with("<!--StartFragment-->"));
        assert!(html[end_fragment..].starts_with("<!--EndFragment-->"));
        assert!(html.as_bytes()[start_html..].starts_with(b"<html>"));
        assert_eq!(end_html, html.len());
        assert!(
            html[start_fragment..end_fragment].contains("<table class=\"ffwb-table\">"),
            "the appended tables live inside the bracketed fragment (Req 28.5)"
        );
    }

    // Validates: screen-snapshot-scrm Req 28.3, 28.7 -- with NO tables (box-only
    // model) the output is byte-identical to a build with an empty tables arg,
    // i.e. the box-only path is unaffected by this change.
    #[test]
    fn cf_html_box_only_unchanged_when_no_tables() {
        let png = [0x89, b'P', b'N', b'G'];
        let with_empty = build_cf_html(&png, "BOX.LINE", "");
        // No `<table>` markup leaks in when there are no tables.
        assert!(
            !with_empty.contains("<table"),
            "a box-only composite carries no table markup"
        );
        // The trailing tables separator is absent, so the body ends at the last
        // line paragraph exactly as before Req 28.
        assert!(with_empty.contains("BOX.LINE</span></p><!--EndFragment-->"));
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
