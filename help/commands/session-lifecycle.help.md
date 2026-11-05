<!-- TOPIC: cmd:EXIT -->
<!-- TITLE: EXIT Command -->

## Syntax

```
EXIT
```

## Description

Terminates the workbench immediately, closing every Workspace. On a non-editor
Context EXIT always ends the application; its seed alias is `X`.

## See Also

- [QUIT](cmd:QUIT), [LOGOFF](cmd:LOGOFF), [END](cmd:END)

<!-- TOPIC: cmd:QUIT -->
<!-- TITLE: QUIT Command -->

## Syntax

```
QUIT
```

## Description

Terminates the workbench, an alias of EXIT. Closes every Workspace and ends the
application.

## See Also

- [EXIT](cmd:EXIT), [LOGOFF](cmd:LOGOFF)

<!-- TOPIC: cmd:LOGOFF -->
<!-- TITLE: LOGOFF Command -->

## Syntax

```
LOGOFF
```

## Description

Terminates the workbench, an alias of EXIT retained for mainframe familiarity.
Closes every Workspace and ends the application.

## See Also

- [EXIT](cmd:EXIT), [QUIT](cmd:QUIT)

<!-- TOPIC: cmd:START -->
<!-- TITLE: START Command -->

## Syntax

```
START <command>
```

## Description

Opens a Context in a new Workspace tab instead of transforming the current one.
START is the only verb that creates a new tab.

## See Also

- [CLOSE](cmd:CLOSE), [SWAP](cmd:SWAP), [WORKSPACE](cmd:WORKSPACE)

<!-- TOPIC: cmd:MENU -->
<!-- TITLE: MENU Command -->

## Syntax

```
MENU <name>
```

## Description

Opens the named menu as a Workspace, resolving it through the single command
front door. With no argument it opens the Primary Option Menu.

## See Also

- [MENUS](cmd:MENUS), [POM](cmd:POM), [SETTINGS](cmd:SETTINGS)

<!-- TOPIC: cmd:CLOSE -->
<!-- TITLE: CLOSE Command -->

## Syntax

```
CLOSE
```

## Description

Closes the current Workspace tab. When it is the last tab the workbench falls
back to the Home Context rather than exiting.

## See Also

- [START](cmd:START), [END](cmd:END), [EXIT](cmd:EXIT)

<!-- TOPIC: cmd:DOCK -->
<!-- TITLE: DOCK Command -->

## Syntax

```
DOCK
```

## Description

Docks a detached Workspace back into the main workbench window, reversing a
DETACH.

## See Also

- [DETACH](cmd:DETACH), [SPLIT](cmd:SPLIT)

<!-- TOPIC: cmd:RETURN -->
<!-- TITLE: RETURN Command -->

## Syntax

```
RETURN
```

## Description

Unwinds the current tab's navigation stack back to its primary panel in one
step, the inverse of repeated END presses.

## See Also

- [END](cmd:END), [CLOSE](cmd:CLOSE)

<!-- TOPIC: cmd:DETACH -->
<!-- TITLE: DETACH Command -->

## Syntax

```
DETACH
```

## Description

Moves the current Workspace into its own operating-system window as a Detached
Workspace.

## See Also

- [DOCK](cmd:DOCK), [SPLIT](cmd:SPLIT)

<!-- TOPIC: cmd:SPLIT -->
<!-- TITLE: SPLIT Command -->

## Syntax

```
SPLIT
```

## Description

Splits the current Workspace so two Contexts are visible side by side within the
one tab.

## See Also

- [UNSPLIT](cmd:UNSPLIT), [FOCUS](cmd:FOCUS), [DETACH](cmd:DETACH)

<!-- TOPIC: cmd:UNSPLIT -->
<!-- TITLE: UNSPLIT Command -->

## Syntax

```
UNSPLIT
```

## Description

Collapses a split Workspace back to a single Context, reversing a SPLIT.

## See Also

- [SPLIT](cmd:SPLIT), [FOCUS](cmd:FOCUS)

<!-- TOPIC: cmd:FOCUS -->
<!-- TITLE: FOCUS Command -->

## Syntax

```
FOCUS
```

## Description

Moves keyboard focus to the other pane of a split Workspace.

## See Also

- [SPLIT](cmd:SPLIT), [UNSPLIT](cmd:UNSPLIT)

<!-- TOPIC: cmd:SWAP -->
<!-- TITLE: SWAP Command -->

## Syntax

```
SWAP [<number>]
```

## Description

Switches to another open Workspace tab. With a number it swaps directly to that
tab; with none it opens the tab picker.

## See Also

- [START](cmd:START), [WORKSPACE](cmd:WORKSPACE)

<!-- TOPIC: cmd:WORKSPACE -->
<!-- TITLE: WORKSPACE Command -->

## Syntax

```
WORKSPACE <action>
```

## Description

Manages Workspaces: open, clone, pin and move operations on the current or named
Workspace.

## See Also

- [START](cmd:START), [SWAP](cmd:SWAP), [KINDS](cmd:KINDS)

<!-- TOPIC: cmd:PFSHOW -->
<!-- TITLE: PFSHOW Command -->

## Syntax

```
PFSHOW [ON|OFF]
```

## Description

Toggles the function-key label bar shown along the bottom of the workbench.

## See Also

- [KEYS](cmd:KEYS), [COMMAND](cmd:COMMAND)

<!-- TOPIC: cmd:COMMAND -->
<!-- TITLE: COMMAND Command -->

## Syntax

```
COMMAND <action>
```

## Description

Acts on the command line itself, such as moving it to the top or bottom of the
Workspace.

## See Also

- [PFSHOW](cmd:PFSHOW)

<!-- TOPIC: cmd:REFRESH -->
<!-- TITLE: REFRESH Command -->

## Syntax

```
REFRESH
```

## Description

Discards unsaved edits in the current editable Context, reloads it from its
backing store, and stays open.

## See Also

- [SAVE](cmd:SAVE), [CANCEL](cmd:CANCEL), [END](cmd:END)
