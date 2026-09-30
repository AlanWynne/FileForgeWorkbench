<!-- TOPIC: cmd:SAVE -->
<!-- TITLE: SAVE Command -->

## Syntax

```
SAVE
```

## Description

Writes the current buffer to its backing file. A backup may be created depending
on configuration.

## See Also

- [CANCEL](cmd:CANCEL), [END](cmd:END)

<!-- TOPIC: cmd:CANCEL -->
<!-- TITLE: CANCEL Command -->

## Syntax

```
CANCEL
```

## Description

Discards unsaved changes in the current Context and closes it. Prompts if there
are pending edits.

## See Also

- [SAVE](cmd:SAVE), [END](cmd:END)

<!-- TOPIC: cmd:END -->
<!-- TITLE: END Command -->

## Syntax

```
END
```

## Description

Closes the current Context and returns to the previous one (the F3 action). If a
Workspace is split, END collapses the split.

## See Also

- [CANCEL](cmd:CANCEL), [SAVE](cmd:SAVE)

<!-- TOPIC: cmd:UNDO -->
<!-- TITLE: UNDO Command -->

## Syntax

```
UNDO
```

## Description

Reverses the most recent edit transaction.

## See Also

- [REDO](cmd:REDO), [Undo feature](feature:undo)

<!-- TOPIC: cmd:REDO -->
<!-- TITLE: REDO Command -->

## Syntax

```
REDO
```

## Description

Reapplies the most recently undone edit transaction.

## See Also

- [UNDO](cmd:UNDO), [Undo feature](feature:undo)
