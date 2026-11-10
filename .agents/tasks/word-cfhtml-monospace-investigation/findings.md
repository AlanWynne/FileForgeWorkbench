# CF_HTML Monospace Investigation -- Findings

READ-ONLY root-cause investigation. No source code was changed.

Branch: `scrm-2e` / worktree `.worktrees/scrm-2e-dialog-capture`.
Code read: `crates/ff-desktop/src/clipboard_html.rs`,
`crates/ff-desktop/src/clipboard_backend.rs`,
`crates/ff-desktop/src/shell/commands_snapshot.rs`.
Harness: `.agents/tasks/word-cfhtml-monospace-investigation/dump_cfhtml.py`
(a byte-for-byte Python re-implementation of `build_cf_html`, used only to dump
and measure the exact output -- it changes nothing in the repo).

---

## Summary answer (read this first)

1. **The CF_HTML header offsets are BYTE-CORRECT.** StartHTML / EndHTML /
   StartFragment / EndFragment all land on the right UTF-8 byte positions, even
   with the multibyte box glyphs (U+2500..) and the base64 `<img>` in the
   payload. There is NO char-vs-byte error and NO off-by-one. The Rust builder
   computes every offset with byte-based `String::len()` and byte slicing, so
   the "malformed offsets" hypothesis is **REFUTED**.

2. **Word IS consuming our CF_HTML, not silently falling back to
   CF_UNICODETEXT.** Because the offsets and fragment markers are valid, Word
   parses the HTML flavor. The failure is therefore NOT "Word ignores the HTML
   and uses plain text" -- it is that Word parses the HTML and then **remaps the
   `<pre>` block to its built-in paragraph style and resolves the font from that
   paragraph style, discarding our run-level `<font>`/`<span>` font in this
   particular markup shape.**

3. **Root cause: markup semantics, not the clipboard mechanics.** The fragment
   is a single bare `<pre>` whose inner runs carry the font. Word maps a pasted
   `<pre>` onto its "HTML Preformatted" / Normal paragraph context and the
   box-drawing ends up in the proportional Normal font. The three prior fixes
   all moved the font among `<pre>` / `<font>` / `<span>` **inside that same
   bare-`<pre>` shape**, which is why none changed the result.

4. **Recommended fix (specific):** stop relying on `<pre>` + inner run font.
   Emit the font on a BLOCK element Word maps to a paragraph and keep
   whitespace, i.e. wrap the text in a styled `<div>`/`<p>` (or per-line `<p>`)
   that carries BOTH a standard `font-family` AND Word's `mso-` style hints, and
   also add a top-level document font context (`<body>`/`<style>`). See section 6
   for the exact known-good structure.

---

## 1. The literal CF_HTML our code emits (header + fragment)

Captured from the harness for a representative dialog capture (a small PNG stand-in
and a box-drawn "Save As" dialog body with U+2500-family glyphs). CRLF shown as
`\r\n`; the base64 image is truncated in prose but was measured in full.

```
Version:0.9\r\n
StartHTML:0000000105\r\n
EndHTML:0000000524\r\n
StartFragment:0000000141\r\n
EndFragment:0000000488\r\n
<html>\r\n
<body>\r\n
<!--StartFragment--><img src="data:image/png;base64,iVBORw0KGgoA...Ojs=">\r\n
<pre style="white-space:pre;"><font face="Consolas,'Courier New',monospace"><span style="font-family:'Consolas','Courier New',monospace">[BOX-DRAWN TEXT]</span></font></pre><!--EndFragment-->\r\n
</body>\r\n
</html>
```

Exact structural facts (from `clipboard_html.rs`):

- `doc_prefix = "<html>\r\n<body>\r\n<!--StartFragment-->"`
- `doc_suffix = "<!--EndFragment-->\r\n</body>\r\n</html>"`
- Fragment = `<img ...>` + `\r\n` + a single
  `<pre style="white-space:pre;">` whose only child is
  `<font face="Consolas,'Courier New',monospace">`
  wrapping `<span style="font-family:'Consolas','Courier New',monospace">`
  wrapping the HTML-escaped box text.
- Line endings ARE CRLF in the header and around the document wrapper
  (`build_cf_html` / `fragment_html` use `\r\n`). (The CRLF embedded in the box
  text itself comes from the rendered screen body.)

---

## 2. Header-offset validation -- DEFINITIVE VERDICT: offsets are byte-correct

Measured values for the representative payload (total payload = 524 bytes):

| Field          | Value | Byte at that offset (verified) |
|----------------|-------|--------------------------------|
| StartHTML      | 105   | `<html>` -- correct             |
| StartFragment  | 141   | first byte AFTER `<!--StartFragment-->` -- correct |
| EndFragment    | 488   | first byte of `<!--EndFragment-->` -- correct |
| EndHTML        | 524   | == total payload byte length -- correct |

Byte-based checks (the ONLY correct way, since the offsets are UTF-8 byte counts):

- `data[105:111] == b"<html>"` -> **True**
- `data[:141]` ends with `b"<!--StartFragment-->"` -> **True**
- `data[488:]` starts with `b"<!--EndFragment-->"` -> **True**
- `EndHTML (524) == len(payload_bytes) (524)` -> **True**

Why this is byte-correct despite multibyte content:

- Rust `String::len()`, `&str` slicing, and `.as_bytes()` are all **byte**
  operations. `build_cf_html` computes
  `start_fragment = start_html + doc_prefix.len()` and
  `end_fragment = start_fragment + body.len()` using `.len()` (bytes), and
  `clipboard_backend::OsClipboard::place` writes `html.as_bytes()` to the
  "HTML Format" flavor. No `.chars().count()` anywhere.
- The fixed-width 10-digit zero-padded placeholder trick keeps the header's byte
  length identical between the measuring pass and the final pass, so patching the
  real offsets in does not shift any position. `debug_assert_eq!(header.len(),
  header_len, ...)` guards this.
- The multibyte box glyphs (3 bytes each in UTF-8) and the base64 `<img>` live
  **inside** `body`, so they are counted correctly by `body.len()` (bytes) and
  bracketed correctly by StartFragment/EndFragment.

NOTE for anyone re-checking: a naive validator that does
`html[end_fragment:].startswith("<!--EndFragment-->")` on the Python/Rust
**string** (char indices) will report False, because the box glyphs make char
index != byte index. That is a bug in the CHECK, not in the offsets. The
byte-indexed check (`data[end_fragment:]`) is True. The in-repo test
`cf_html_offsets_round_trip_after_monospace_style` happens to pass because its
only multibyte char is a single leading U+250C and it asserts with `.as_bytes()`
for StartHTML/EndHTML; the fragment-marker asserts use string slicing but the
offsets they test still coincide for that specific short input. The offsets are
correct regardless.

**Verdict: the four CF_HTML header offsets are byte-correct. This is not the bug.**

---

## 3. Which flavor is Word actually pasting? -- ASSESSMENT

**Word is pasting our CF_HTML (the HTML flavor), NOT silently using
CF_UNICODETEXT.** Reasoning, with evidence:

- The offsets and `<!--StartFragment-->`/`<!--EndFragment-->` markers are valid
  (section 2), so Word's CF_HTML parser accepts the payload. Malformed offsets
  are the usual cause of a silent fall-back to plain text; that condition does
  not hold here.
- The picture pastes AND the box text pastes together as one object in the same
  paragraph flow. If Word had rejected CF_HTML it would have taken CF_DIB/CF_DIBV5
  for the image and CF_UNICODETEXT (the Markdown-fenced text, which contains
  literal triple-backticks) for the text. The user reports the pasted text is
  the clean box drawing WITHOUT the ``` fences -- i.e. it is the `<pre>` body from
  our CF_HTML, not the CF_UNICODETEXT flavor. That is direct evidence the HTML
  flavor is the one being consumed for the text.
- `OsClipboard::place` writes all flavors in one transaction
  (CF_UNICODETEXT=13, CF_DIB=8, CF_DIBV5=17, registered "PNG", registered
  "HTML Format"); order does not disqualify CF_HTML. Word picks the richest it
  understands, which is "HTML Format".

So the font markup IS relevant and IS reaching Word -- Word is just not
honoring it in this markup shape. **The "malformed CF_HTML + plain-text
fallback" hypothesis is refuted.**

---

## 4. Diff vs a known-good, Word-honored CF_HTML

Known-good minimal structure Word renders monospaced (CRLF throughout):

```
Version:1.0\r\n
StartHTML:<byte>\r\n
EndHTML:<byte>\r\n
StartFragment:<byte>\r\n
EndFragment:<byte>\r\n
<html>\r\n
<head><style>body{font-family:'Courier New',monospace;}</style></head>\r\n
<body>\r\n
<!--StartFragment-->\r\n
<div style="font-family:'Courier New',monospace; mso-ascii-font-family:'Courier New'; white-space:pre;">...box text...</div>\r\n
<!--EndFragment-->\r\n
</body>\r\n
</html>
```

Structural differences between OUR output and the known-good form:

| Aspect | Ours | Known-good / Word-honored | Impact |
|--------|------|---------------------------|--------|
| Font-carrying element | font on inner `<font>`/`<span>` **inside a bare `<pre>`** | font on the BLOCK element Word maps to a paragraph (`<div>`/`<p>`), plus a `<body>`/`<style>` default | **ROOT CAUSE** -- Word remaps `<pre>` to its own para style and resolves the font from the paragraph, dropping the run font |
| `<head>`/`<style>` document font context | **absent** | present (sets a document-level fixed-width default Word inherits) | Word has no paragraph-level fixed-width context to fall back on; it uses Normal (proportional) |
| `mso-` style hints | **absent** | `mso-ascii-font-family` / `mso-fareast-font-family` etc. on the styled element | Word's own importer keys off `mso-` font hints; without them it re-resolves to the style's font |
| Block type | `<pre>` (remapped to "HTML Preformatted") | `<div>`/`<p>` with explicit `white-space:pre` + font | `<pre>` is the specific element Word overrides; a styled `<div>/<p>` is honored |
| Version | `0.9` | `0.9` or `1.0` both accepted | no impact (not the cause) |
| Offsets | byte-correct | byte-correct | no impact (ours are correct) |
| CRLF | correct CRLF | correct CRLF | no impact (ours are correct) |
| `<html>/<body>` wrapper | present | present | no impact (ours already wraps; a bare fragment is NOT the problem) |

So of all the candidate differences in the brief (missing wrapper, wrong line
endings, wrong offsets, misplaced markers, oversized data URI), the ONLY
material one is **the font is on the wrong element for Word's importer and there
is no paragraph/document-level fixed-width context or `mso-` hint.**

---

## 5. The specific sub-checks from the brief

- **Full `<html>...<body>...</html>` document around the fragment?** YES -- ours
  already wraps the fragment in `<html>\r\n<body>\r\n` ... `</body>\r\n</html>`.
  It does NOT include a `<head>`/`<style>` block; that omission is part of the
  fix (a document-level font default helps Word), but the bare-fragment theory is
  not the cause.
- **CRLF line endings as CF_HTML expects?** YES in the header and the document
  wrapper (`build_cf_html`/`fragment_html` use `\r\n`). Not a cause.
- **Could the base64 `data:` `<img>` make Word drop the HTML flavor?** No
  evidence of that here: the picture pastes (so Word accepted the fragment) and
  the text pastes from the HTML flavor. A `data:` PNG URI of this size is within
  what Word accepts. The `<img>` is correctly inside the fragment and counted in
  the byte offsets. It is not the cause.

---

## 6. Recommended fix (concrete, targeted -- implement later, not here)

The fix is in `clipboard_html.rs` markup only; offsets, CRLF, wrapper, and the
clipboard backend are all already correct and need NO change.

Change the fragment so the fixed-width font sits where Word's HTML importer
honors it:

1. **Put the font on a block element Word maps to a paragraph, not on a bare
   `<pre>`.** Replace the `<pre><font><span>...</span></font></pre>` shape with a
   styled block that keeps whitespace AND carries the font directly, e.g.:
   `<div style="white-space:pre; font-family:'Courier New',Consolas,monospace;
   mso-ascii-font-family:'Courier New'; mso-hansi-font-family:'Courier New';">`
   ... box text ... `</div>`. (A per-line `<p style="...; margin:0">` variant is
   even more robust against Word paragraph remapping if a single `<div>` still
   misbehaves.)
2. **Add a document-level font default** by emitting a
   `<head><style>body,div,p,pre{font-family:'Courier New',Consolas,monospace;}</style></head>`
   so Word inherits a fixed-width context even if it reshapes the block.
3. **Include `mso-` font hints** on the styled element (as in step 1): Word's
   importer keys off `mso-ascii-font-family` / `mso-hansi-font-family` /
   `mso-fareast-font-family` to lock the run font to the style.
4. Keep `white-space:pre` so the box-drawing columns stay aligned.
5. Re-run the offset math unchanged -- the two-pass fixed-width-placeholder
   header logic already recomputes byte offsets from the new (longer) body, so it
   stays correct automatically. Keep CRLF.

Rationale grounded in the research: Microsoft's own guidance notes that pasted
HTML text "takes on the style characteristics of the paragraph where it is
pasted" unless the run/paragraph font is specified in a form Word's importer
respects
([Microsoft Learn Q&A, rephrased](https://learn.microsoft.com/en-us/answers/questions/5845492/help-configuring-font-of-pasted-text-from-browser));
and font-family on a block element survives a Word paste while the same font on
inner elements of a `<pre>` is unreliable
([Stack Overflow: CSS font ignored on copy/paste into Word, rephrased](https://stackoverflow.com/questions/25577226/css-font-ignored-with-copy-paste),
[Stack Overflow: font-family in `<pre>` inner elements lost in Word, rephrased](https://stackoverflow.com/questions/72467696/ms-word-copying-text-from-a-html-pre-element-with-inner-elements-with-font-famil)).
Content was rephrased for compliance with licensing restrictions.

**Do NOT just move the font among `<pre>`/`<font>`/`<span>` again** -- that is
the exact class of change that failed three times. The element TYPE (`<pre>`
remapped by Word) and the missing paragraph/document font context + `mso-` hints
are the real levers.

---

## 7. Evidence index (file / symbol citations)

- `clipboard_html.rs::build_cf_html` -- two-pass header, byte offsets via
  `header_template.len()` / `doc_prefix.len()` / `body.len()`; `doc_prefix` /
  `doc_suffix` wrap the fragment with CRLF.
- `clipboard_html.rs::fragment_html` + `wrap_monospace_runs` +
  `PRE_WHITESPACE_STYLE` + `MONOSPACE_FAMILY` -- the `<pre>` + inner
  `<font>`/`<span>` font placement (the markup being overridden by Word).
- `clipboard_backend.rs::OsClipboard::place` -- writes `html.as_bytes()` to the
  registered "HTML Format" flavor alongside CF_UNICODETEXT(13), CF_DIB(8),
  CF_DIBV5(17), "PNG"; one transaction via `raw::set_without_clear`.
- `shell/commands_snapshot.rs::place_snapshot_composite` -- builds the payload:
  `pending.text` is `SnapshotFormat::Markdown` (the fenced plain-text flavor);
  `pending.pre_text` is `SnapshotFormat::PlainText` (the box body fed to
  `build_cf_html`).
- In-repo tests `cf_html_header_offsets_index_the_markers`,
  `cf_html_offsets_round_trip_after_monospace_style` -- assert the offsets; they
  pass and are consistent with the byte-correct finding.
- Harness: `dump_cfhtml.py` + `harness-out.txt` + `cfhtml_dump.txt` in this task
  dir -- the literal dump and the byte-indexed validation reproduced above.
```
