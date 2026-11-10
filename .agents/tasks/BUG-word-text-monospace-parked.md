# BUG (PARKED/DEFERRED): SCRM dialog/screen text does not render MONOSPACED when pasted into Microsoft Word

Status: PARKED by owner decision 2026-11-01. The current behaviour is acceptable
for now (the PICTURE flavor pastes into Word faithfully and is inherently
monospaced; the TEXT pastes but in Word's proportional Normal font). To be
revisited later. This report logs what was attempted so the next attempt does
not repeat the dead ends.

## Symptom
Copying a SCRM dialog/screen (dialog Copy button, Ctrl+C, SNAPSHOT) and pasting
into Microsoft Word renders the box-drawn TEXT in Word's default proportional
font (Aptos/Normal), so the box-drawing (|, -, corners) misaligns. Pasting into
Notepad/plain-text targets is fine (plain text). The PICTURE (cropped dialog /
window screenshot, CF_DIB/CF_DIBV5) pastes correctly and IS faithful/monospaced.

## What works (do NOT re-touch when revisiting)
- The multi-flavor clipboard (CF_HTML + CF_DIB/CF_DIBV5 + CF_UNICODETEXT) in one
  transaction.
- The real window/dialog screenshot PICTURE, cropped to the dialog rect, pastes
  into Word correctly.
- CF_HTML header byte-offsets (StartHTML/EndHTML/StartFragment/EndFragment) are
  byte-correct (verified: no char-vs-byte / off-by-one error).
- Notepad/plain-text paste (CF_UNICODETEXT Markdown) is correct.

## Attempts that did NOT fix the Word monospace (all failed in a real paste)
1. `<pre style="font-family:'Consolas','Courier New',monospace; white-space:pre">`
   -- Word ignored the font on the <pre>.
2. Font moved onto inner runs inside the <pre> via
   `<font face="Consolas,...">` + `<span style="font-family:...">`. -- still
   proportional in Word.
3. Rebuilt the fragment as ONE <p> PER LINE, each line wrapped in
   `<span style="font-family:'Consolas',...">` (matching a Word "Save as HTML"
   export of a MANUALLY-monospaced paste, FFWB-PasteExamples-exported-from-word-
   text-monospaced.htm). Added a <head><style> monospace default too. -- STILL
   proportional in Word after paste.

## Evidence captured (in the worktree root, for the next attempt)
- FFWB-PasteExample.html / .md -- a plain-text/markdown paste result.
- FFWB-PasteExamples-exported-from-word.htm -- Word's export of our paste
  (BROKEN: box lines in MsoNormal/Aptos, only stray junction glyphs got a span).
- FFWB-PasteExamples-exported-from-word-text-monospaced.htm -- Word's export AFTER
  the owner manually set the text monospaced (GOOD TARGET: each line
  `<p class=MsoNormal><span style='font-family:Consolas'>...</span></p>`).
- FFWB-PasteExamples-Exported-From-Word.htm -- Word export after attempt 3 (still
  proportional: MsoNormal/Aptos).
- FFWB-PasteExamples*.docx -- the pasted documents.

## Key unknown for the next attempt (why guessing failed)
The Word "Save as HTML" exports are Word's RE-SERIALIZATION, not the clipboard
bytes we wrote -- reasoning from them has been misleading. Attempt 3 copied the
exact shape of the known-good export and STILL failed, which suggests either
(a) Word is NOT actually consuming our CF_HTML for the text (it may prefer
CF_UNICODETEXT despite the valid offsets), or (b) a subtle CF_HTML structural
issue we cannot see from Word's re-export.

## Recommended approach when UNPARKED (do NOT guess at markup again)
Capture the ACTUAL bytes our app writes to the "HTML Format" clipboard entry
(not Word's re-export): either add a temporary debug dump in
clipboard_backend/clipboard_html that writes the literal CF_HTML we place to a
file on each Copy, OR use a clipboard inspector (NirSoft InsideClipboard, no
admin) right after a Copy to view the raw HTML Format entry. Compare those REAL
bytes against a minimal known-good CF_HTML that Word renders monospaced. Only
then decide the fix. Do NOT iterate on markup by pasting-and-eyeballing again.

## Scope when unparked
clipboard_html.rs (CF_HTML builder) and possibly clipboard_backend.rs (format
registration/order). Spec: screen-snapshot-scrm Req 23.5f (CF_HTML monospace).
Everything else in SCRM is unaffected.
