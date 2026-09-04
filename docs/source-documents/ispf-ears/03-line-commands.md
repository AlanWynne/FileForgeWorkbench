# ISPF EARS Requirements -- Line Commands

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 9.

## Introduction

These requirements describe the full set of ISPF editor line commands: how they
are entered, what they do, and how block forms and numeric suffixes work.

## Glossary

| Term | Meaning |
|------|---------|
| Line command field | The 6-column field on the left of each data line |
| Block command | A line command entered on two lines to affect all lines between them |
| Destination command | A or B line command marking where copied or moved lines go |
| Pending command | A line command waiting for a destination or partner block command |

## Requirements

### Requirement 1 -- Entering Line Commands

1.1 WHEN the user types a line command in the line command field and presses
    Enter THE editor SHALL execute that command on the line or lines indicated.

1.2 WHEN the user types a line command with a numeric suffix (e.g. D3) THE editor
    SHALL apply the command to that many consecutive lines starting at the
    indicated line.

1.3 WHEN the user types the same line command letter twice (e.g. CC, DD) THE
    editor SHALL treat it as the start of a block command and wait for the
    matching closing command on another line.

1.4 WHEN the user types a line command on the command line prefixed with a colon
    (:) THE editor SHALL apply the command to the line containing the cursor.

1.5 WHEN a line command is pending and the user presses Enter without completing
    it THE editor SHALL display an error indicator on the pending line.

### Requirement 2 -- Insert and Delete

2.1 WHEN the user enters the I line command THE editor SHALL insert one blank
    line after the indicated line.

2.2 WHEN the user enters I with a numeric suffix (e.g. I5) THE editor SHALL
    insert that many blank lines after the indicated line.

2.3 WHEN the user enters the D line command THE editor SHALL delete the indicated
    line.

2.4 WHEN the user enters D with a numeric suffix (e.g. D3) THE editor SHALL
    delete that many consecutive lines starting at the indicated line.

2.5 WHEN the user enters the DD block command on two lines THE editor SHALL
    delete all lines from the first DD to the second DD inclusive.

### Requirement 3 -- Copy and Move

3.1 WHEN the user enters the C line command and an A or B destination THE editor
    SHALL copy the indicated line to after (A) or before (B) the destination line.

3.2 WHEN the user enters C with a numeric suffix and a destination THE editor
    SHALL copy that many consecutive lines to the destination.

3.3 WHEN the user enters the CC block command on two lines and a destination THE
    editor SHALL copy all lines between and including the two CC lines to the
    destination.

3.4 WHEN the user enters the M line command and a destination THE editor SHALL
    move the indicated line to the destination and remove it from its original
    position.

3.5 WHEN the user enters M with a numeric suffix and a destination THE editor
    SHALL move that many consecutive lines to the destination.

3.6 WHEN the user enters the MM block command on two lines and a destination THE
    editor SHALL move all lines between and including the two MM lines to the
    destination.

3.7 WHEN the user enters the A line command THE editor SHALL mark that line as
    the after-destination for a pending copy or move.

3.8 WHEN the user enters the B line command THE editor SHALL mark that line as
    the before-destination for a pending copy or move.

### Requirement 4 -- Repeat

4.1 WHEN the user enters the R line command THE editor SHALL insert one duplicate
    of the indicated line immediately after it.

4.2 WHEN the user enters R with a numeric suffix (e.g. R3) THE editor SHALL
    insert that many duplicates of the indicated line.

4.3 WHEN the user enters the RR block command on two lines THE editor SHALL
    duplicate all lines between and including the two RR lines once.

### Requirement 5 -- Overlay

5.1 WHEN the user enters the O line command and a source line THE editor SHALL
    overlay the destination line with non-blank characters from the source line,
    leaving destination characters where the source has blanks.

5.2 WHEN the user enters the OO block command on two lines and a source THE
    editor SHALL overlay each destination line with the corresponding source line.

### Requirement 6 -- Shift Commands

6.1 WHEN the user enters the ( line command THE editor SHALL shift the line two
    columns to the left within the current bounds (column shift left).

6.2 WHEN the user enters ) THE editor SHALL shift the line two columns to the
    right within the current bounds (column shift right).

6.3 WHEN the user enters ( or ) with a numeric suffix THE editor SHALL shift by
    that many columns instead of the default two.

6.4 WHEN a column shift would move a non-blank character past a boundary THE
    editor SHALL delete that character (destructive shift).

6.5 WHEN the user enters the < line command THE editor SHALL shift the data
    portion of the line left without shifting labels or comments (data shift left).

6.6 WHEN the user enters > THE editor SHALL shift the data portion of the line
    right (data shift right).

6.7 WHEN a data shift left would move a non-blank character past the left bound
    THE editor SHALL stop the shift and mark the line with an ==ERR> flag.

6.8 WHEN the user enters the (( or )) block command on two lines THE editor SHALL
    apply the column shift to all lines between and including the two command lines.

6.9 WHEN the user enters the << or >> block command on two lines THE editor SHALL
    apply the data shift to all lines between and including the two command lines.

### Requirement 7 -- Case Conversion

7.1 WHEN the user enters the UC line command THE editor SHALL convert all
    alphabetic characters on that line to uppercase.

7.2 WHEN the user enters the LC line command THE editor SHALL convert all
    alphabetic characters on that line to lowercase.

7.3 WHEN the user enters UCC or LCC block commands on two lines THE editor SHALL
    apply the case conversion to all lines between and including the two command
    lines.

### Requirement 8 -- Exclude and Show

8.1 WHEN the user enters the X line command THE editor SHALL exclude that line
    from the display, replacing it with a message showing how many lines are
    excluded.

8.2 WHEN the user enters X with a numeric suffix THE editor SHALL exclude that
    many consecutive lines.

8.3 WHEN the user enters the XX block command on two lines THE editor SHALL
    exclude all lines between and including the two XX lines.

8.4 WHEN the user enters the S line command on an excluded-lines message THE
    editor SHALL redisplay the first excluded line in that group.

8.5 WHEN the user enters S with a numeric suffix on an excluded-lines message THE
    editor SHALL redisplay that many lines from the excluded group.

8.6 WHEN the user enters the F line command THE editor SHALL redisplay the first
    line of an excluded group.

8.7 WHEN the user enters the L line command THE editor SHALL redisplay the last
    line of an excluded group.

### Requirement 9 -- Text Entry and Formatting

9.1 WHEN the user enters the TE line command THE editor SHALL enter text entry
    mode for that line, allowing free-form text input that wraps automatically.

9.2 WHEN the user enters the TF line command THE editor SHALL reflow the
    paragraph starting at that line to fit within the current bounds.

9.3 WHEN the user enters the TS line command THE editor SHALL split the line at
    the cursor position, creating two lines.

### Requirement 10 -- Special Line Commands

10.1 WHEN the user enters the COLS line command THE editor SHALL insert a
     temporary column identification line above the indicated line.

10.2 WHEN the user enters the BOUNDS line command THE editor SHALL insert or
     update the =BNDS> boundary line.

10.3 WHEN the user enters the TABS line command THE editor SHALL insert or update
     the =TABS> tab definition line.

10.4 WHEN the user enters the MASK line command THE editor SHALL insert or update
     the =MASK> mask line.

10.5 WHEN the user enters the MD (Make Dataline) line command on a special
     temporary line THE editor SHALL convert that line to a permanent data line
     that is saved with the data set.

10.6 WHEN the user enters the HX line command THE editor SHALL display that line
     in hexadecimal format with two rows of hex digits below the character row.
