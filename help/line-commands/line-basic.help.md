<!-- TOPIC: line:D -->
<!-- TITLE: D / DD -- Delete Line(s) -->

## Syntax

```
D        (delete one line)
Dn       (delete n lines)
DD ... DD (delete a block)
```

## Description

Deletes lines from the file.

## Block Form

Mark the first and last line of a range with `DD`; the block between (inclusive)
is deleted on Enter.

## Examples

- `D` on a line deletes that line.
- `DD` on two lines deletes everything between them.

## See Also

- [R / RR](line:R), [Line command reference](line:index)

<!-- TOPIC: line:I -->
<!-- TITLE: I -- Insert Line -->

## Syntax

```
I    (insert one line)
In   (insert n lines)
```

## Description

Inserts one or more new empty lines below the current line, using the active
[insert mask](line:MASK) if set.

## See Also

- [MASK](line:MASK), [Line command reference](line:index)

<!-- TOPIC: line:R -->
<!-- TITLE: R / RR -- Repeat Line(s) -->

## Syntax

```
R        (repeat one line)
Rn       (repeat n times)
RR ... RR (repeat a block)
```

## Description

Duplicates a line or a marked block.

## Block Form

Mark a range with `RR` on the first and last line.

## See Also

- [D / DD](line:D), [Line command reference](line:index)

<!-- TOPIC: line:C -->
<!-- TITLE: C / CC -- Copy Line(s) -->

## Syntax

```
C        (copy one line)
CC ... CC (copy a block)
```

## Description

Copies a line or block to a destination.

## Target Requirements

C and CC require a destination marker: [A](line:A) (after) or B (before) on the
target line.

## Block Form

Mark the source range with `CC` on its first and last line, then an A or B target.

## Examples

- `CC`/`CC` on a range plus `A` on another line copies the block after that line.

## See Also

- [M / MM](line:M), [A / B targets](line:A)

<!-- TOPIC: line:M -->
<!-- TITLE: M / MM -- Move Line(s) -->

## Syntax

```
M        (move one line)
MM ... MM (move a block)
```

## Description

Moves a line or block to a destination, removing it from its original location.

## Target Requirements

M and MM require an [A or B target](line:A).

## See Also

- [C / CC](line:C), [A / B targets](line:A)

<!-- TOPIC: line:A -->
<!-- TITLE: A / B -- Copy/Move Targets -->

## Syntax

```
A    (place after this line)
B    (place before this line)
```

## Description

Marks the destination for a pending [C/CC](line:C) or [M/MM](line:M) operation.
`A` places the source after the target line; `B` places it before.

## See Also

- [C / CC](line:C), [M / MM](line:M)

<!-- TOPIC: line:X -->
<!-- TITLE: X / XX -- Exclude Line(s) -->

## Syntax

```
X        (exclude one line)
XX ... XX (exclude a block)
```

## Description

Hides lines from the display, collapsing them into a placeholder. Reveal them
with the [SHOW](cmd:SHOW) command.

## See Also

- [EXCLUDE command](cmd:EXCLUDE), [SHOW](cmd:SHOW)

<!-- TOPIC: line:U -->
<!-- TITLE: U / UU -- Uppercase Line(s) -->

## Syntax

```
U        (uppercase one line)
UU ... UU (uppercase a block)
```

## Description

Converts the characters on a line or block to uppercase.

## See Also

- [Line command reference](line:index)

<!-- TOPIC: line:shift -->
<!-- TITLE: Shift Commands -->

## Syntax

```
>  >>   shift columns right
<  <<   shift columns left
)  ))   shift data right
(  ((   shift data left
```

## Description

Shift the text of a line or block left or right. The angle-bracket forms shift
the whole column window; the parenthesis forms shift data within the bounds.

## See Also

- [BNDS](line:BNDS), [Line command reference](line:index)

<!-- TOPIC: line:COLS -->
<!-- TITLE: COLS -- Column Ruler -->

## Description

Displays a column-number ruler line above the current line to help align text.

## See Also

- [BNDS](line:BNDS), [TABS](line:TABS)

<!-- TOPIC: line:BNDS -->
<!-- TITLE: BNDS -- Bounds Ruler -->

## Description

Shows and lets you set the left/right editing bounds that many commands respect.

## See Also

- [COLS](line:COLS), [Shift commands](line:shift)

<!-- TOPIC: line:TABS -->
<!-- TITLE: TABS -- Tab Stops -->

## Description

Shows and lets you set tab stops used for tab-key navigation and alignment.

## See Also

- [COLS](line:COLS)

<!-- TOPIC: line:MASK -->
<!-- TITLE: MASK -- Insert Mask -->

## Description

Edits the mask template applied to newly [inserted](line:I) lines.

## See Also

- [I -- insert](line:I)
