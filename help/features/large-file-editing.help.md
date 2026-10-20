<!-- TOPIC: feature:large_file_editing -->
<!-- TITLE: Editing Large Files (Piece Table) -->

# Editing Large Files

> DRAFT NOTE: This topic is a conceptual draft included for reference. It describes
> the intended piece-table model, not a verified account of the shipped editor.
> Before the help pass, cross-check it against the actual implementation and the
> `large-file-performance` and `document-model` specs under `docs/specs/`, and
> reconcile any differences. Not yet wired into the live help index.

FileForge Workbench can open and edit very large files without loading the whole
file into a single mutable buffer in memory. It does this with a data structure
called a **piece table** (also known as a piece list or piece chain), paired with
an **index** that maps a document position to the right piece quickly.

This page explains the model so you understand why large files open fast, why
edits stay cheap no matter the file size, and how undo and redo work.

## The core idea

Rather than hold one editable copy of the text, the editor keeps the text in two
buffers that are never rewritten in place, and describes the *current* document as
an ordered list of spans ("pieces") that point into those buffers.

- **Original buffer** -- the file's bytes as they exist on disk. Read-only for the
  whole editing session. It is never modified and never grows.
- **Add buffer** -- starts empty and is *append-only*. Every character you type is
  appended here; nothing in it is ever moved or overwritten.

Because neither buffer is ever mutated in place, any span pointing into them stays
valid for the life of the session. That immutability is what makes the structure
cheap and safe.

## What a piece is

A piece is a small descriptor -- a span, not a copy of the text:

```
Piece {
    source: Original | Add   // which buffer the text lives in
    start:  offset           // where the span begins in that buffer
    length: count            // how long the span is
}
```

The current document is simply the concatenation of all pieces, in order:

```
document = for each piece in order:
               buffer[piece.source][piece.start .. piece.start + piece.length]
```

The **piece list** is that ordered sequence. Editing the document means editing
the list of pieces -- never the text bytes themselves.

## Why this works for large files

- **Fast open.** Opening a file does not copy or scan the whole thing into an
  editable structure. The original buffer can be read on demand (or memory-mapped)
  from disk, and the piece list starts as a single piece spanning the whole file.
- **Edits are independent of file size.** Inserting or deleting touches only a few
  descriptors in the list, not the millions of bytes around them. A one-character
  insert in a 2 GB file costs the same as in a 2 KB file.
- **No whole-file rewrite on every keystroke.** Typed text is appended to the add
  buffer; the document bytes already on disk are left where they are.

## Starting state

Open a file of N bytes. The add buffer is empty and the piece list holds exactly
one piece covering the entire original:

```
Original buffer: "The quick brown fox"   (length 19)
Add buffer:      ""
Piece list:      [ (Original, 0, 19) ]
```

Reading the document walks that one piece and yields the whole file.

## Inserting text

Suppose you insert "very " at document offset 10:

1. Append the typed text to the add buffer:
   `Add buffer: "very "` (start 0, length 5).
2. Find the piece that contains offset 10 (the single original piece).
3. Split it into three pieces: the part before, the new add-buffer piece, and the
   part after.

```
Piece list becomes:
  [ (Original, 0, 10),    // "The quick "
    (Add,      0, 5),     // "very "
    (Original, 10, 9) ]   // "brown fox"
```

Walking the list now yields "The quick very brown fox". The original buffer never
moved -- only the descriptors changed.

## Deleting text

Deletion removes a range by trimming or splitting pieces. No bytes are erased from
any buffer. There are three cases:

- The range covers whole pieces -> drop those pieces from the list.
- The range starts or ends inside a piece -> shorten that piece (adjust its start
  and/or length).
- The range sits in the middle of one piece -> split it into the surviving head
  and tail and drop the middle.

```
Before: [ (Original, 0, 19) ]      // "The quick brown fox"
Delete offset 10..15
After:  [ (Original, 0, 10),       // "The quick "
          (Original, 15, 4) ]      // surviving tail
```

The deleted bytes still physically exist in the original buffer; they are simply
no longer referenced by any piece.

## The index -- finding a piece fast

Walking the list from the start to find "which piece holds document offset X"
costs time proportional to the number of pieces, and the piece count grows with
every edit. To keep lookups fast the editor maintains an index over the pieces:

- **Cumulative offset table** -- each piece's starting document offset is tracked
  so a binary search locates the containing piece in O(log pieces).
- **Balanced tree (the usual production choice)** -- pieces are the leaves of a
  balanced tree whose internal nodes cache the total length (and newline count) of
  their subtree. Lookup, insert, and split all become O(log pieces).

Caching newline counts per subtree also gives fast conversion between a line
number and a byte offset, which the editor needs constantly for scrolling,
rendering, and cursor movement in a large file.

## Undo and redo

Because the buffers are immutable, an edit is fully described by the change it made
to the piece list. To undo, the editor restores the previous piece-list state; to
redo, it reapplies the change. No text is ever copied back -- undo and redo swap
descriptors, which is why they stay fast on large files. See [Undo](feature:undo).

## Tradeoffs

- Reading the whole document means walking and concatenating pieces; the tree
  index keeps this efficient.
- The add buffer grows even for text that was later deleted, until the document is
  compacted (typically on save).
- A long session fragments the document into many pieces; the balanced-tree index
  keeps that from slowing lookups.

## See Also

- [Undo and Redo](feature:undo)
- [Getting Started](index)
- [Help Index](index)
