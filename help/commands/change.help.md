<!-- TOPIC: cmd:CHANGE -->
<!-- TITLE: CHANGE Command -->

## Syntax

```
CHANGE 'from' 'to' [ALL] [FIRST|LAST|NEXT|PREV]
```

## Description

Finds the `from` string and replaces it with the `to` string. Without ALL, only
the next occurrence is changed.

## Modifiers

- **ALL** replaces every occurrence in the file (or in the current bounds).
- **FIRST / LAST / NEXT / PREV** choose which occurrence to change.

## Examples

- `CHANGE 'foo' 'bar'` replaces the next foo with bar.
- `CHANGE 'foo' 'bar' ALL` replaces every foo with bar.

## See Also

- [RCHANGE](cmd:RCHANGE) repeats the last CHANGE.
- [FIND](cmd:FIND)

<!-- TOPIC: cmd:RCHANGE -->
<!-- TITLE: RCHANGE Command -->

## Syntax

```
RCHANGE
```

## Description

Repeats the most recent [CHANGE](cmd:CHANGE) at the next occurrence.

## See Also

- [CHANGE](cmd:CHANGE)
