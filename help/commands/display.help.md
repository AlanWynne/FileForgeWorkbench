<!-- TOPIC: cmd:HEX -->
<!-- TITLE: HEX Command -->

## Syntax

```
HEX [ON|OFF]
```

## Description

Toggles the Hex display mode for the current file, showing byte values alongside
the text. See [Hex mode](mode:hex).

## See Also

- [Hex mode](mode:hex)

<!-- TOPIC: cmd:EXCLUDE -->
<!-- TITLE: EXCLUDE Command -->

## Syntax

```
EXCLUDE 'string' [ALL]
X 'string' [ALL]
```

## Description

Hides lines matching the string, collapsing them into a placeholder row. The
alias is **X**.

## See Also

- [SHOW](cmd:SHOW), [RESET](cmd:RESET)

<!-- TOPIC: cmd:SHOW -->
<!-- TITLE: SHOW Command -->

## Syntax

```
SHOW [ALL]
```

## Description

Reveals lines previously hidden by [EXCLUDE](cmd:EXCLUDE).

## See Also

- [EXCLUDE](cmd:EXCLUDE), [RESET](cmd:RESET)

<!-- TOPIC: cmd:RESET -->
<!-- TITLE: RESET Command -->

## Syntax

```
RESET
```

## Description

Clears pending line commands, exclusions, and error flags in the current file
display.

## See Also

- [EXCLUDE](cmd:EXCLUDE), [SHOW](cmd:SHOW)
