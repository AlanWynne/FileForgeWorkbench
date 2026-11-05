<!-- TOPIC: cmd:EDIT -->
<!-- TITLE: EDIT Command -->

## Syntax

```
EDIT <path>
```

## Description

Opens a file for editing in an Editor Context, where its contents can be changed
and saved.

## See Also

- [VIEW](cmd:VIEW), [BROWSE](cmd:BROWSE), [CREATE](cmd:CREATE)

<!-- TOPIC: cmd:VIEW -->
<!-- TITLE: VIEW Command -->

## Syntax

```
VIEW <path>
```

## Description

Opens a file in a read-only View Context, so its contents can be inspected
without the risk of accidental changes.

## See Also

- [EDIT](cmd:EDIT), [BROWSE](cmd:BROWSE)

<!-- TOPIC: cmd:BROWSE -->
<!-- TITLE: BROWSE Command -->

## Syntax

```
BROWSE <path>
```

## Description

Opens a file in a read-only Browse Context for scrolling through large content
quickly.

## See Also

- [EDIT](cmd:EDIT), [VIEW](cmd:VIEW)

<!-- TOPIC: cmd:COMPARE -->
<!-- TITLE: COMPARE Command -->

## Syntax

```
COMPARE <path1> <path2>
```

## Description

Opens two files side by side and highlights the differences between them.

## See Also

- [EDIT](cmd:EDIT), [REPLACE](cmd:REPLACE)

<!-- TOPIC: cmd:CREATE -->
<!-- TITLE: CREATE Command -->

## Syntax

```
CREATE <path>
```

## Description

Creates a new file at the given path and opens it for editing.

## See Also

- [EDIT](cmd:EDIT), [REPLACE](cmd:REPLACE)

<!-- TOPIC: cmd:REPLACE -->
<!-- TITLE: REPLACE Command -->

## Syntax

```
REPLACE <path>
```

## Description

Writes the current buffer to the given path, overwriting any existing file
there.

## See Also

- [CREATE](cmd:CREATE), [SAVE](cmd:SAVE)
