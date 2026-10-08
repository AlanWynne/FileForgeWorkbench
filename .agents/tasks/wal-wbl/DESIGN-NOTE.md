# Design Note -- WAL / WBL self-logging save formats (+ their Command Environment)

STATUS: DESIGN NOTE FOR OWNER REVIEW. NOT a gated spec. Captures the owner's
WAL/WBL idea (conversation) so it can be reviewed, then turned into its OWN gated
CR (requirements/design/tasks), cross-referenced to CR-CH-058 (consumes the piece
journal) and CR-CH-053 (registers a Command Environment). Plain ASCII only.

This is a SEPARATE capability from CR-CH-058. Plain SAVE stays the default
(re-baseline, as CR-CH-058 specifies); WAL/WBL are additional "Save As" FORMATS
built AFTER the windowed/record foundation. F1 is NOT blocked by or changed for
this.

---

## 1. What it is

Two SAVE-AS file FORMATS that embed the document's change history (the piece-table
`PieceJournal` from CR-CH-058) INSIDE the saved file, so edit history survives
close/reopen. Both are BINARY CONTAINER files (a tar archive, optionally
compressed), NOT plain text with an interleaved log.

- **WAL (.wal) -- write-ahead log.** Base = the ORIGINAL content; plus a FORWARD
  change log. Current state is reached by replaying the log FORWARD from the
  original base.
- **WBL (.wbl) -- write-back log.** Base = the CURRENT content; plus a BACKWARD
  change log. Prior states (back to the original) are reached by walking the log
  BACKWARD from the current base.

Symmetric pair:
- WAL: store ORIGINAL + forward log  -> replay forward to CURRENT.
- WBL: store CURRENT  + backward log -> walk back to ORIGINAL.

WAL optimizes "replay from the start"; WBL optimizes "I have the latest, let me
undo backward". Both carry the full history between original and current; they
differ only in which end is the stored base and which direction the log runs.

## 2. Why a binary tar container (owner decision)

Resolves the "log inside the file vs content still readable" tension cleanly:
- A `.wal`/`.wbl` is a tar archive (optionally compressed). A naive editor sees
  BINARY -> it will not try to edit it as text (no accidental log corruption);
  the user must EXTRACT it first to get raw content.
- A tool that understands the format (a standalone `.wal`/`.wbl` editor, OR an
  FFWB Command Environment for these file types) reads the archive members,
  reconstructs content + log, and treats it correctly.
- Content and log are SEPARATE archive members -- no interleaving, no delimiter
  hacks. Integrity via a manifest checksum (a naive extract-edit-repack is
  detectable).

Proposed archive members (settle exact shape at the gate):
- `manifest` -- format (WAL|WBL), format version, FFWB version, RecordFormat of
  the content, base-direction, checksums of the other members, record counts.
- `base` -- the base content (ORIGINAL for WAL, CURRENT for WBL), stored in the
  content's RecordFormat (CR-CH-058); itself openable as a record stream.
- `log` -- the serialized `PieceJournal` entries (SpliceOp + inverse), ordered
  for forward (WAL) or backward (WBL) replay.

## 3. The FFWB Command Environment for .wal/.wbl (CR-CH-053 fit)

A `.wal`/`.wbl` file's owning Command Environment (same model as the mainframe
CE and the ff-ce-* family) knows how to:
- OPEN: extract the archive, load `base` as a record stream (CR-CH-058 document),
  and RECONSTRUCT the `PieceJournal` from `log` -- so undo/redo history is RESTORED
  ACROSS SESSIONS (the real payoff: edit history persists through save/close/open).
- SAVE: re-pack the archive (base + current log + manifest), per the active
  WAL/WBL direction.
- WALK: commands to replay forward (WAL) / step backward (WBL) through the log --
  i.e. navigate the embedded history. Command vocabulary TBD at the gate.
This registers on the same seam the mainframe CE uses; no new dispatch mechanism.

## 4. How it relates to CR-CH-058 and plain SAVE

- The `PieceJournal` built in CR-CH-058 F1 IS the log. WAL/WBL are two
  SERIALIZATIONS of it (forward-from-original vs backward-from-current).
- Plain SAVE (CR-CH-058) = RE-BASELINE: collapse pieces, drop the journal. That
  stays the DEFAULT. WAL/WBL are explicit "Save As <format>" choices that instead
  PRESERVE the journal in the file (non-re-baselining saves).
- So SAVE has (at least) three modes: plain re-baseline (default), save-as-WAL,
  save-as-WBL. The user chooses WAL/WBL via a Save-As format selection (exact UX
  -- command flag vs dialog -- settled at the gate).

## 5. Scope and placement (recommended)

- OWN CR (new requirement), NOT folded into CR-CH-058. Cross-ref CR-CH-058
  (piece journal) + CR-CH-053 (CE).
- Likely OWN crate (e.g. `ff-wal-wbl` / a log-format crate) depending on
  `ff-document-model` (piece journal) + a tar + a compression lib; do NOT bloat
  `ff-document-model`. The CE adapter lives in / registers via ff-desktop like
  the other CEs.
- Sequenced AFTER CR-CH-058 F1 (which ships plain SAVE). WAL/WBL is an additive
  feature on top; F1 is not reworked for it.

## 6. Open questions for the gate

1. Exact archive member layout + manifest fields; tar flavour; compression
   default (none vs gzip/zstd) and whether it is configurable.
2. Save-As UX: a `SAVE` command format flag (e.g. `SAVE AS WAL` / a `-wal` switch)
   vs a format picker dialog vs both. How the chosen format is remembered for the
   tab.
3. The `.wal`/`.wbl` Command Environment's command vocabulary (walk forward/back,
   jump to a revision, materialize current/original, re-pack).
4. How much history the log spans (original<->current is implied; is there any
   trimming/compaction of a very long log, reusing CR-CH-058 compaction ideas?).
5. Integrity/degradation: manifest checksum mismatch (someone extracted, edited,
   repacked badly) -> open as plain content + discard stale log, with a warning.
6. Does opening a `.wal`/`.wbl` restore the FULL undo/redo stack live in the
   editor (reconstruct PieceJournal), and does a subsequent plain SAVE re-baseline
   it (dropping the embedded history) -- confirm that is the intended interaction.
7. Interaction with the mainframe/other RecordFormats: the `base` member stores
   the content's RecordFormat; a WAL/WBL of a Fixed/FB document is in principle
   possible -- in scope or defer to after the mainframe CE?

## 7. Decisions already made (owner-confirmed in conversation)

- Two SAVE-AS formats: `.wal` and `.wbl`, both binary tar containers (optionally
  compressed), so naive editors see binary and must extract.
- WAL = original base + forward log -> current. WBL = current base + backward
  log -> original.
- The log IS the CR-CH-058 piece journal, serialized; embedded as an archive
  member alongside the base content.
- An FFWB Command Environment for `.wal`/`.wbl` files (CR-CH-053 model) that
  opens (extract + reconstruct journal), saves (repack), and walks the history.
- Own CR + likely own crate; sequenced after CR-CH-058 F1; plain re-baseline
  SAVE remains the default.
