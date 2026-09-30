<!-- TOPIC: feature:undo -->
<!-- TITLE: Undo and Redo -->

## Overview

Edits are grouped into transactions that can be reversed and reapplied.

## Commands and UI

- [UNDO](cmd:UNDO) reverses the most recent transaction.
- [REDO](cmd:REDO) reapplies the most recently undone transaction.

## See Also

- [UNDO](cmd:UNDO), [REDO](cmd:REDO)

<!-- TOPIC: feature:macros -->
<!-- TITLE: Macro API (Scripting) -->

## Overview

FileForge Workbench embeds a Lua runtime for automation. Macros run per buffer,
can respond to event hooks, and invoke the same commands available on the
command line.

## Scripting Model

- Lua runtime with per-buffer state.
- Event hooks such as OnChar, OnKey, OnOpen, and OnSave.

## API Categories

- Cursor and selection queries.
- Text read/modify operations.
- Command invocation (run any primary command).

## See Also

- [Command history](feature:command_history), [Help Index](index)

<!-- TOPIC: feature:command_history -->
<!-- TITLE: Command History -->

## Overview

The command line records the commands you run. Recall them without retyping.

## Commands and UI

- **F12** (RETRIEVE) recalls the most recent command into the field.
- **Up / Down arrows** step through history while the command field has focus.

## See Also

- [Function keys](feature:function_keys)

<!-- TOPIC: feature:tabs -->
<!-- TITLE: Workspaces and Tabs -->

## Overview

Each open Context lives in a Workspace tab. Tabs can be reordered, split, or
detached into their own window.

## See Also

- [Docking](feature:docking)

<!-- TOPIC: feature:docking -->
<!-- TITLE: Docking and Detaching -->

## Overview

A Workspace can be split into side-by-side regions or detached into a separate OS
window, then re-docked. Function keys F2 (detach) and Shift+F2 (dock) drive this.

## See Also

- [Tabs](feature:tabs), [Function keys](feature:function_keys)

<!-- TOPIC: feature:configuration -->
<!-- TITLE: Configuration Overview -->

## Overview

Settings are stored in TOML configuration and can be edited via the Settings menu
or the CONFIG command. Categories include appearance, editor behaviour, key maps,
and help.

## Commands and UI

- [SETTINGS](cmd:SETTINGS) opens the settings menu.
- The `[help]` section configures the Help panel position and content directory.

## See Also

- [SETTINGS](cmd:SETTINGS), [Help Index](index)
