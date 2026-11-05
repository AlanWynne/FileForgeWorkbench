<!-- TOPIC: cmd:CONFIG -->
<!-- TITLE: CONFIG Command -->

## Syntax

```
CONFIG
```

## Description

Opens the Configuration Context, a browsable and editable view of every
workbench configuration key.

## See Also

- [SETTINGS](cmd:SETTINGS), [THEME](cmd:THEME), [KEYS](cmd:KEYS)

<!-- TOPIC: cmd:THEME -->
<!-- TITLE: THEME Command -->

## Syntax

```
THEME
```

## Description

Opens the Theme Editor, where colour themes can be copied, edited, saved and
selected.

## See Also

- [CONFIG](cmd:CONFIG), [SETTINGS](cmd:SETTINGS)

<!-- TOPIC: cmd:CATALOGS -->
<!-- TITLE: CATALOGS Command -->

## Syntax

```
CATALOGS
```

## Description

Opens the Catalog Explorer Context for managing the virtual catalog and its
datasets.

## See Also

- [FILES](cmd:FILES), [FILE CATALOGS](cmd:FILE CATALOGS)

<!-- TOPIC: cmd:COMMANDS -->
<!-- TITLE: COMMANDS Command -->

## Syntax

```
COMMANDS
```

## Description

Opens the Command Configurator, where user-defined command definitions can be
created and edited.

## See Also

- [ALIASES](cmd:ALIASES), [MENUS](cmd:MENUS)

<!-- TOPIC: cmd:LOG -->
<!-- TITLE: LOG Command -->

## Syntax

```
LOG
```

## Description

Opens the Event Log Context, which lists recent workbench events for review and
filtering.

## See Also

- [STATUS](cmd:STATUS)

<!-- TOPIC: cmd:PLUGINS -->
<!-- TITLE: PLUGINS Command -->

## Syntax

```
PLUGINS
```

## Description

Opens the Extension Manager, which lists registered extensions and their state
and details.

## See Also

- [MACROS](cmd:MACROS), [COMMANDS](cmd:COMMANDS)

<!-- TOPIC: cmd:MACROS -->
<!-- TITLE: MACROS Command -->

## Syntax

```
MACROS
```

## Description

Opens the Macro Library Context, listing the Lua macros discovered in the
configured macro directories.

## See Also

- [PLUGINS](cmd:PLUGINS), [COMMANDS](cmd:COMMANDS)

<!-- TOPIC: cmd:SEARCH -->
<!-- TITLE: SEARCH Command -->

## Syntax

```
SEARCH <text>
```

## Description

Searches within the current file or Context and opens the Search Results
Context with the matches.

## See Also

- [GSEARCH](cmd:GSEARCH), [FIND](cmd:FIND)

<!-- TOPIC: cmd:GSEARCH -->
<!-- TITLE: GSEARCH Command -->

## Syntax

```
GSEARCH <text>
```

## Description

Runs a global search across files and opens the Search Results Context with the
matches from every scanned file.

## See Also

- [SEARCH](cmd:SEARCH), [FIND](cmd:FIND)

<!-- TOPIC: cmd:KINDS -->
<!-- TITLE: KINDS Command -->

## Syntax

```
KINDS
```

## Description

Opens the Workspace Kinds Editor, where each Kind's title, menu bar, key list
and profile are configured.

## See Also

- [MENUS](cmd:MENUS), [KEYS](cmd:KEYS), [WORKSPACE](cmd:WORKSPACE)

<!-- TOPIC: cmd:MENUS -->
<!-- TITLE: MENUS Command -->

## Syntax

```
MENUS
```

## Description

Opens the Menus Editor, where menu screens can be created, changed and saved.

## See Also

- [MENU](cmd:MENU), [KINDS](cmd:KINDS), [ALIASES](cmd:ALIASES)

<!-- TOPIC: cmd:ALIASES -->
<!-- TITLE: ALIASES Command -->

## Syntax

```
ALIASES
```

## Description

Opens the Alias Editor, where command aliases can be created, edited and saved
for the active environment.

## See Also

- [COMMANDS](cmd:COMMANDS), [MENUS](cmd:MENUS)

<!-- TOPIC: cmd:FILE CATALOGS -->
<!-- TITLE: FILE CATALOGS Command -->

## Syntax

```
FILE CATALOGS
```

## Description

Opens the Catalog Explorer Context from the Home Context, the two-word form of
the CATALOGS verb.

## See Also

- [CATALOGS](cmd:CATALOGS), [FILES](cmd:FILES)

<!-- TOPIC: cmd:RESET BARE -->
<!-- TITLE: RESET BARE Command -->

## Syntax

```
RESET BARE
```

## Description

Archives the current configuration and rebuilds the barebones defaults, a
recovery action for a corrupt or unwanted configuration.

## See Also

- [CONFIG](cmd:CONFIG), [SETTINGS](cmd:SETTINGS)
