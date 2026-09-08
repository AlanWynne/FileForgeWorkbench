# ISPF Command Reference

**Document purpose:** Provide a practical reference to commonly used ISPF primary commands and editor line commands.

**Intended use:** User reference and requirements input for the FileForgeWorkbench ISPF-style editor.

> **Note:** ISPF command availability and detailed behaviour can vary by panel, function, installation, product version, and local customisation. Primary commands are entered on the `COMMAND ===>` line. Line commands are entered in the prefix area beside individual data lines.

---

## 1. ISPF Command Types

ISPF editing commonly uses two command types:

1. **Primary commands** are entered on the command line and generally operate on the editor session, data set, member, search state, or displayed data.
2. **Line commands** are entered in the prefix area and operate on a line, a count of lines, or a marked block of lines.

---

## 2. Common Primary Commands

| Command | Purpose | Example or note |
|---|---|---|
| `FIND text` | Locate text. | `FIND ERROR` |
| `RFIND` | Repeat the previous `FIND`. | Often assigned to a function key. |
| `CHANGE old new` | Replace an occurrence of text. | `CHANGE OLD NEW` |
| `CHANGE old new ALL` | Replace all qualifying occurrences. | `CHANGE OLD NEW ALL` |
| `EXCLUDE text` | Hide lines that match the search criterion. | `EXCLUDE DEBUG` |
| `X ALL` | Exclude all displayed lines. | `X ALL` |
| `SHOW ALL` | Show lines that were excluded. | Availability or exact syntax may depend on context. |
| `RESET` | Clear exclusions, pending line commands, or temporary display conditions, depending on context. | `RESET` |
| `TOP` | Position at the first line. | `TOP` |
| `BOTTOM` | Position at the last line. | `BOTTOM` |
| `UP` | Scroll upward. | `UP 10` |
| `DOWN` | Scroll downward. | `DOWN PAGE` |
| `LEFT` | Scroll horizontally to the left. | `LEFT` |
| `RIGHT` | Scroll horizontally to the right. | `RIGHT` |
| `LOCATE n` | Position at a line number or supported label. | `LOCATE 250` |
| `SAVE` | Save changes without leaving the edit session. | `SAVE` |
| `END` | End the current function. In Edit, changes are normally saved according to the active profile and processing context. | `END` |
| `CANCEL` | Leave the edit session without saving the current changes. | `CANCEL` |
| `SORT` | Sort data using the specified range, columns, and order. | `SORT 1 10 A` |
| `COPY member` | Copy data from another member or sequential data set. | `COPY MEMBER1` |
| `MOVE member` | Move data from another supported source. | Exact use depends on the function and context. |
| `CREATE member` | Create a member using selected or specified data. | `CREATE NEWMEM` |
| `DELETE` | Delete qualifying or selected data in contexts that support the primary form. | Do not confuse with the `D` line command. |
| `RETRIEVE` | Recall a previously entered command. | Repeated use cycles through command history where supported. |
| `HEX ON` | Enable hexadecimal display. | `HEX ON` |
| `HEX OFF` | Disable hexadecimal display. | `HEX OFF` |
| `CAPS ON` | Enable uppercase handling in Edit or View where supported. | `CAPS ON` |
| `CAPS OFF` | Preserve mixed-case input where supported. | `CAPS OFF` |
| `NUMBER ON` | Enable sequence-number handling. | Exact operands depend on the edit profile. |
| `NUMBER OFF` | Disable sequence-number handling. | `NUMBER OFF` |
| `UNNUM` | Remove sequence numbers where supported. | Check the active numbering mode before use. |
| `RENUM` | Renumber sequence fields. | `RENUM` |
| `HILITE language` | Enable or configure language-sensitive highlighting. | `HILITE COBOL` |
| `BOUNDS left right` | Define the active editing column boundaries. | `BOUNDS 1 72` |
| `COLS` | Display a column-position ruler. | `COLS` |
| `TABS` | Display or configure tab positions. | `TABS` |
| `MASK` | Display or configure the insert mask. | `MASK` |
| `CUT` | Copy or move selected lines to an ISPF clipboard where supported. | Usually used with line selection. |
| `PASTE` | Insert data from an ISPF clipboard. | `PASTE` |
| `UNDO` | Undo a supported previous edit operation when recovery is enabled. | Availability depends on the edit profile and recovery support. |
| `MODEL` | Insert an available predefined model or template. | `MODEL JCL` |

---

## 3. Primary Command Examples

### 3.1 Find text

```text
COMMAND ===> FIND 'CUSTOMER'
```

### 3.2 Repeat the previous search

```text
COMMAND ===> RFIND
```

### 3.3 Change all occurrences

```text
COMMAND ===> CHANGE 'OLD-NAME' 'NEW-NAME' ALL
```

### 3.4 Set editing bounds

```text
COMMAND ===> BOUNDS 1 72
```

### 3.5 Enable hexadecimal display

```text
COMMAND ===> HEX ON
```

### 3.6 Sort by columns

```text
COMMAND ===> SORT 1 10 A
```

---

## 4. Common Line Commands

Line commands are typed over the line number or in the prefix field.

### 4.1 Insert Commands

| Command | Purpose |
|---|---|
| `I` | Insert one blank line. |
| `In` | Insert *n* blank lines, for example `I5`. |
| `IA` | Explicit insert-after form in implementations that support it. |
| `IB` | Explicit insert-before form in implementations that support it. |

### 4.2 Delete Commands

| Command | Purpose |
|---|---|
| `D` | Delete one line. |
| `Dn` | Delete *n* lines beginning with the marked line, for example `D5`. |
| `DD ... DD` | Delete a block delimited by two `DD` markers. |

Example:

```text
DD  First line to delete
    Lines inside the block
DD  Last line to delete
```

### 4.3 Repeat Commands

| Command | Purpose |
|---|---|
| `R` | Repeat one line. |
| `Rn` | Repeat a line *n* times, for example `R5`. |
| `RR ... RR` | Repeat a marked block. |

### 4.4 Copy Commands and Targets

| Command | Purpose |
|---|---|
| `C` | Mark one line as the copy source. |
| `CC ... CC` | Mark a block as the copy source. |
| `A` | Insert the copied or moved source after this line. |
| `B` | Insert the copied or moved source before this line. |

Example:

```text
CC  First line of copy block
    Lines inside the block
CC  Last line of copy block
 A  Copy the block after this line
```

### 4.5 Move Commands and Targets

| Command | Purpose |
|---|---|
| `M` | Mark one line as the move source. |
| `MM ... MM` | Mark a block as the move source. |
| `A` | Move the source after this line. |
| `B` | Move the source before this line. |

### 4.6 Exclude Commands

| Command | Purpose |
|---|---|
| `X` | Exclude one line from the display without deleting it. |
| `Xn` | Exclude *n* lines, for example `X5`. |
| `XX ... XX` | Exclude a marked block. |

### 4.7 Shift Commands

| Command | Purpose |
|---|---|
| `>` | Shift one line to the right. |
| `>n` | Shift one line right by a specified amount or repeat count, depending on the active semantics. |
| `>> ... >>` | Shift a block to the right. |
| `<` | Shift one line to the left. |
| `<n` | Shift one line left by a specified amount or repeat count, depending on the active semantics. |
| `<< ... <<` | Shift a block to the left. |
| `)` | Shift right while respecting active bounds. |
| `)) ... ))` | Bounds-aware right shift for a block. |
| `(` | Shift left while respecting active bounds. |
| `(( ... ((` | Bounds-aware left shift for a block. |

### 4.8 Overlay Commands

| Command | Purpose |
|---|---|
| `O` | Overlay copied data onto a target line. |
| `OO ... OO` | Mark or process an overlay block where supported. |

### 4.9 Case-Conversion Commands

| Command | Purpose |
|---|---|
| `UC` | Convert one line to uppercase. |
| `UCC ... UCC` | Convert a block to uppercase. |
| `LC` | Convert one line to lowercase. |
| `LCC ... LCC` | Convert a block to lowercase. |

### 4.10 Tagging Extensions

The following commands are useful in FileForgeWorkbench's proposed ISPF-inspired model, but should be identified as project extensions unless confirmed for the target ISPF environment.

| Command | Purpose |
|---|---|
| `T` | Tag one line for a later operation. |
| `TT ... TT` | Tag a block. |
| `U` | Remove a tag from one line. |
| `UU ... UU` | Remove tags from a block. |

---

## 5. Frequently Used Command Set

A practical daily-use command set for editing COBOL, PL/I, JCL, REXX, assembler, or other mainframe source includes:

```text
FIND
RFIND
CHANGE
CHANGE ALL
EXCLUDE
RESET
TOP
BOTTOM
HEX ON
HEX OFF
COLS
BOUNDS
CC / A
CC / B
MM / A
MM / B
DD
RR
X
UC
LC
SAVE
END
CANCEL
RETRIEVE
```

---

## 6. Recommended FileForgeWorkbench Minimum Command Set

The following commands provide a useful first implementation milestone for an ISPF-inspired editor.

### 6.1 Minimum Primary Commands

```text
FIND
RFIND
CHANGE
EXCLUDE
SHOW
RESET
TOP
BOTTOM
UP
DOWN
LEFT
RIGHT
LOCATE
SAVE
END
CANCEL
HEX
COLS
BOUNDS
```

### 6.2 Minimum Line Commands

```text
I
D
Dn
DD
R
Rn
RR
C
CC
M
MM
A
B
X
Xn
XX
>
>>
<
<<
```

### 6.3 Recommended Implementation Principle

```text
Line commands define the source, target, block, or scope.
Primary commands define the operation or transformation.
The editor context resolves both into an executable action.
```

---

## 7. Compatibility Considerations for FileForgeWorkbench

When implementing these commands, FileForgeWorkbench should document the following for each command:

- Canonical command name and abbreviations.
- Valid editor modes, such as Browse, View, and Edit.
- Complete operand syntax.
- Default scope.
- Treatment of excluded lines.
- Interaction with active column bounds.
- Interaction with pending line commands.
- Error behaviour for incomplete block commands.
- Undo and transaction behaviour.
- Behaviour with very large files.
- Differences from IBM ISPF.
- FileForgeWorkbench-specific extensions.

A compatibility matrix should distinguish among:

- IBM-compatible behaviour.
- Behaviour inspired by ISPF but intentionally changed.
- FileForgeWorkbench extensions.
- Commands not yet implemented.

---

## 8. Important Distinctions

### Exclude versus delete

```text
EXCLUDE = hide data from the current display
DELETE  = remove data from the document
```

### Copy versus move

```text
COPY = duplicate the source at the target
MOVE = relocate the source to the target
```

### Primary versus line command

```text
Primary command = entered on COMMAND ===>
Line command    = entered in the line-prefix area
```

---

## 9. Reference Note

This document is a practical command summary rather than a replacement for the IBM z/OS ISPF documentation. Exact syntax and command availability should be verified against the ISPF function being used and the applicable z/OS release.
