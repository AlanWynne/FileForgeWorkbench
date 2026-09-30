<!-- TOPIC: getting_started -->
<!-- TITLE: Getting Started -->

# Getting Started with FileForge Workbench

FileForge Workbench is a command-driven, ISPF-inspired editing environment. This
walkthrough covers the essentials for a first session.

## Opening a file

- Use the File menu, or type a file path on the `Command ===>` line and press Enter.
- Files open in an Editor Context inside a Workspace tab.

## Basic navigation

- **F7 / F8** scroll up / down by one page (governed by the `SCROLL ===>` amount).
- **UP** and **DOWN** commands scroll by the active SCROLL amount.
- **TOP** and **BOTTOM** jump to the first / last line.
- **LOCATE n** moves to line number n.

## Editing

- In Edit mode, type directly into the text area.
- Prefix-area **line commands** (for example `D` to delete a line, `CC` to copy a
  block) act on individual lines. See [Line Commands](line:index).
- **UNDO** and **REDO** reverse and reapply changes. See [Undo](feature:undo).

## Saving

- **SAVE** writes the current buffer. **CANCEL** discards changes. **END** (F3)
  closes the current Context.

## The command line

- Every action is a command. Menus, buttons, and function keys all invoke the
  same commands you can type on the `Command ===>` line.
- Press **F1** at any time for context-sensitive help on what you are doing.
- **F12** (RETRIEVE) recalls previous commands.

## Arranging the workbench

- Workspaces are tabs; a Workspace can be split or detached into its own window.
- See [Tabs](feature:tabs) and [Docking](feature:docking).

See also: [Help Index](index), [Function Keys](feature:function_keys).
