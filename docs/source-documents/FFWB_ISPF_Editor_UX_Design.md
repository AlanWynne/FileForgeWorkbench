# IBM ISPF Editor UX Design Specification

## Purpose
This document describes the visual layout, interaction model, screen regions, and behavioural expectations of the IBM ISPF Editor. It is intended as a UX and implementation specification for FileForgeWorkbench (FFWB) or any ISPF-inspired editor.

---

# 1. Canonical ISPF Editor Screen Layout

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ISPF EDIT - USER.PROCLIB(MEMBER)                          Columns 00001 00072│
├──────────────────────────────────────────────────────────────────────────────┤
│ Command ===>                                                  Scroll ===> CSR│
├──────────────────────────────────────────────────────────────────────────────┤
│ ****** ************************ TOP OF DATA *******************************  │
│                                                                              │
│ 000100  //JOBNAME  JOB ....                                                  │
│ 000200  //STEP01   EXEC PGM=IEFBR14                                          │
│ 000300  //SYSPRINT DD SYSOUT=*                                               │
│ 000400  //SYSIN    DD *                                                      │
│                                                                              │
│                                                                              │
│                                                                              │
│ ****** ********************** BOTTOM OF DATA ******************************  │
├──────────────────────────────────────────────────────────────────────────────┤
│ ====> Cursor Position / Status / Messages                                    │
├──────────────────────────────────────────────────────────────────────────────┤
│ F1=Help F2=Split F3=Exit F4=Return F5=Rfind F6=Change                        │
│ F7=Up   F8=Down  F9=Swap  F10=Left F11=Right F12=Cancel                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

# 2. Screen Regions

## 2.1 Title Bar

Located on the first line.

Displays:

- Application name (ISPF Edit)
- Dataset/Member/File name
- Current editing mode
- Column boundaries
- Optional profile information

Example:

```text
EDIT - USER.PROCLIB(MYJCL)  Columns 00001 00072
```

### UX Requirements

- Always visible.
- Indicates read-only versus edit mode.
- Displays active language profile.
- Displays active bounds.

---

## 2.2 Command Line Area

```text
Command ===>
```

Located directly below the title.

Primary commands are entered here.

Examples:

```text
FIND ERROR ALL
CHANGE ABC XYZ ALL
HEX ON
SAVE
END
```

### Behaviour

- Accepts complete command strings.
- Maintains command history.
- Supports RETRIEVE command.
- Supports command auto-completion.
- Supports keyboard focus restoration.

### FFWB Recommendation

Provide:

- Drop-down history
- AI command suggestions
- Command validation
- Command palette integration

---

## 2.3 Scroll Field

Normally shown to the right of the command line.

```text
Scroll ===> CSR
```

Controls scrolling behaviour.

Common values:

| Value | Meaning |
|---------|---------|
| CSR | Cursor position |
| PAGE | Full page |
| HALF | Half page |
| DATA | Entire dataset |
| Number | Fixed line count |

---

## 2.4 Prefix Area (Line Command Area)

Left-most editable region.

Example:

```text
D
M
CC
RR
```

Used for line commands.

### Typical Commands

| Command | Function |
|-----------|-----------|
| I | Insert |
| D | Delete |
| R | Repeat |
| M | Move |
| C | Copy |
| A | After |
| B | Before |
| X | Exclude |
| CC | Block Copy Start |
| MM | Block Move Start |
 | 

The user enters commands and presses ENTER.

The editor executes all pending line commands.

---

## 2.5 Sequence Number Area

Traditionally:

```text
000100
000200
000300
```

Purpose:

- Line identification
- Sorting
- Renumbering
- Legacy source management

### FFWB Recommendation

Allow:

- Hidden mode
- Relative numbering
- Absolute numbering
- Git diff markers

---

## 2.6 Main Editing Area

Largest region of the screen.

Contains:

- Source code
- JCL
- Text
- Dataset records

Characteristics:

- Fixed-width font
- Record-oriented display
- Horizontal scrolling
- Vertical scrolling
- Colour syntax highlighting (optional)

### UX Requirements

The editor shall support:

- Selection
- Copy
- Paste
- Block operations
- Hex mode
- Unicode mode
- Insert and overwrite modes

---

## 2.7 Special System Lines

Examples:

```text
====== TOP OF DATA ======
====== BOTTOM OF DATA ====
```

These are not actual data records.

They indicate dataset boundaries.

Additional special lines:

```text
=NOTE=
=COLS>
```

### UX Recommendation

Render with distinct colours.

Prevent accidental editing.

---

## 2.8 Message Area

Appears above function key labels.

Examples:

```text
COMMAND COMPLETED
TEXT NOT FOUND
MEMBER SAVED
```

### Message Types

- Informational
- Warning
- Error
- Success

### UX Recommendation

Use:

- Status colours
- Icons
- Persistent history panel

---

## 2.9 Function Key Bar

Located at the bottom.

Example:

```text
F1=Help F2=Split F3=Exit F4=Return
F5=Rfind F6=Change F7=Up F8=Down
```

### Purpose

Provides discovery of shortcuts.

### FFWB Requirements

- Dynamic labels
- Profile-specific bindings
- User-customizable mappings
- Support F1-F24

---

# 3. Cursor Model

ISPF is fundamentally cursor-driven.

The cursor determines:

- FIND start point
- Scroll origin
- Insert location
- Line-command target

### FFWB Recommendation

Support:

- Mouse cursor
- Keyboard cursor
- ISPF CSR mode
- Multi-caret modern editing mode

---

# 4. Panel States

## Browse Mode

Read-only.

Commands that modify content are disabled.

## Edit Mode

Full editing.

## View Mode

Hybrid mode.

Allows exclusion and navigation without modification.

## Hex Mode

Displays character and hexadecimal representations.

## Column Mode

Displays column boundaries.

## Excluded Mode

Collapsed dataset sections.

---

# 5. Colour Model

Traditional ISPF colours:

| Element | Typical Colour |
|----------|----------------|
| Commands | Turquoise |
| Input Fields | Green |
| Errors | Red |
| Headers | White |
| Protected Areas | Blue |
| Messages | Yellow |

---

# 6. Recommended FFWB Screen Regions

1. Title Bar
2. Command Bar
3. Toolbar (optional)
4. Dataset Information Strip
5. Prefix Command Column
6. Sequence Number Column
7. Editing Surface
8. Status Message Area
9. Function Key Bar
10. Footer Diagnostics Area

---

# 7. EGUI Widget Mapping

| ISPF Concept | EGUI Widget |
|--------------|------------|
| Command Line | TextEdit |
| Scroll Field | ComboBox |
| Prefix Area | Small TextEdit |
| Data Area | Virtualised Grid |
| Function Keys | Docked Toolbar |
| Message Area | Status Bar |
| Dataset Boundary Lines | Styled Rows |

---

# 8. Critical Behavioural Principle

The ISPF Editor is not a document editor.

It is a record-oriented interaction system.

Each displayed row corresponds to a physical dataset record.

The UX design should therefore prioritise:

- Rows rather than paragraphs
- Columns rather than text flow
- Commands rather than menus
- Keyboard operation over mouse operation
- Predictability over visual effects

This principle is essential if FileForgeWorkbench is to feel authentic to experienced ISPF users.
