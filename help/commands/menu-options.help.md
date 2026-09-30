<!-- TOPIC: cmd:HELP -->
<!-- TITLE: HELP Command -->

## Syntax

```
HELP [topic | LINECOMMANDS | KEYS | MACRO | CONFIG | MISSING | OFF]
```

## Description

Opens the Help Context. With no argument it shows the [Help Index](index). With a
command name it shows that command's topic (for example `HELP CHANGE`). Pressing
**F1** is the context-sensitive equivalent.

## Modifiers

- **LINECOMMANDS** shows the [line command summary](line:index).
- **KEYS** shows the current [function key](feature:function_keys) assignments.
- **MACRO** shows the [macro API](feature:macros) reference.
- **CONFIG** shows the [configuration overview](feature:configuration).
- **MISSING** reports help topics that were requested but are not yet authored.
- **OFF** closes the Help Context.

## See Also

- [Help Index](index), [KEYS](cmd:KEYS)

<!-- TOPIC: cmd:KEYS -->
<!-- TITLE: KEYS Command -->

## Syntax

```
KEYS [kind]
```

## Description

Opens the Keys Workspace to view and edit function-key assignments. See the
[function keys feature](feature:function_keys).

## See Also

- [function keys](feature:function_keys)

<!-- TOPIC: cmd:POM -->
<!-- TITLE: POM (Primary Option Menu) -->

## Syntax

```
POM
=X
```

## Description

Opens the Primary Option Menu, the ISPF-style home Context listing the numbered
options (catalogs, files, editor, compilers, and more).

## See Also

- [FILES](cmd:FILES), [SETTINGS](cmd:SETTINGS)

<!-- TOPIC: cmd:SETTINGS -->
<!-- TITLE: SETTINGS Command -->

## Syntax

```
SETTINGS
=0
```

## Description

Opens the Settings menu (configuration, theme, menus, keys, kinds, reset).

## See Also

- [CONFIG overview](feature:configuration), [POM](cmd:POM)

<!-- TOPIC: cmd:FILES -->
<!-- TITLE: FILES Command -->

## Syntax

```
FILES
=1
```

## Description

Opens the Catalog Explorer / File navigator Context to browse and open files.

## See Also

- [POM](cmd:POM)
