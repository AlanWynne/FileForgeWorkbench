<!-- TOPIC: cmd:UP -->
<!-- TITLE: UP Command -->

## Syntax

```
UP [n | PAGE | HALF | MAX | CSR]
```

## Description

Scrolls the view toward the top of the file by the active `SCROLL ===>` amount,
or by the explicit amount given.

## Examples

- `UP` scrolls up by the current SCROLL amount.
- `UP 10` scrolls up 10 lines.
- `UP MAX` scrolls to the top.

## See Also

- [DOWN](cmd:DOWN), [TOP](cmd:TOP), [BOTTOM](cmd:BOTTOM), [LOCATE](cmd:LOCATE)

<!-- TOPIC: cmd:DOWN -->
<!-- TITLE: DOWN Command -->

## Syntax

```
DOWN [n | PAGE | HALF | MAX | CSR]
```

## Description

Scrolls the view toward the bottom of the file by the active SCROLL amount, or by
the explicit amount given.

## See Also

- [UP](cmd:UP), [TOP](cmd:TOP), [BOTTOM](cmd:BOTTOM)

<!-- TOPIC: cmd:TOP -->
<!-- TITLE: TOP Command -->

## Syntax

```
TOP
```

## Description

Moves to the first line of the file.

## See Also

- [BOTTOM](cmd:BOTTOM), [UP](cmd:UP)

<!-- TOPIC: cmd:BOTTOM -->
<!-- TITLE: BOTTOM Command -->

## Syntax

```
BOTTOM
```

## Description

Moves to the last line of the file.

## See Also

- [TOP](cmd:TOP), [DOWN](cmd:DOWN)

<!-- TOPIC: cmd:LOCATE -->
<!-- TITLE: LOCATE Command -->

## Syntax

```
LOCATE n
```

## Description

Moves the cursor and view to line number `n`.

## Examples

- `LOCATE 1` goes to the first line.
- `LOCATE 500` goes to line 500.

## See Also

- [TOP](cmd:TOP), [BOTTOM](cmd:BOTTOM)
