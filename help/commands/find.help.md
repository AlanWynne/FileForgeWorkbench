<!-- TOPIC: cmd:FIND -->
<!-- TITLE: FIND Command -->

## Syntax

```
FIND 'string' [ALL] [FIRST|LAST|NEXT|PREV] [CHARS|PREFIX|SUFFIX|WORD]
FIND string
```

## Description

Searches the current file for the given string and moves the cursor to the next
match. Quote the string with single quotes when it contains spaces.

## Modifiers

- **ALL** locates every occurrence (used with counts and EXCLUDE/SHOW).
- **FIRST / LAST / NEXT / PREV** choose the search direction and starting point.
- **CHARS / PREFIX / SUFFIX / WORD** restrict what counts as a match.

## Examples

- `FIND 'error'` moves to the next occurrence of error.
- `FIND 'TODO' ALL` highlights every TODO in the file.

## See Also

- [RFIND](cmd:RFIND) repeats the last FIND.
- [CHANGE](cmd:CHANGE) finds and replaces.

<!-- TOPIC: cmd:RFIND -->
<!-- TITLE: RFIND Command -->

## Syntax

```
RFIND
```

## Description

Repeats the most recent [FIND](cmd:FIND) using the same string and direction.
Commonly bound to a function key for rapid repeat searches.

## Examples

- After `FIND 'error'`, press RFIND (or its key) to jump to the following match.

## See Also

- [FIND](cmd:FIND)
