# ISPF EARS Requirements -- Index

This folder contains EARS-format requirements extracted from the IBM z/OS ISPF
manuals. Each file covers a distinct aspect of ISPF editor behaviour.

## Source Documents

| Manual | File | Description |
|--------|------|-------------|
| SC19-3621-30 | f54em00_v2r3.md | z/OS ISPF Edit and Edit Macros, V2R3 |
| SC19-3627-40 | f54ug00_v2r4.md | z/OS ISPF User's Guide Vol I, V2R4 |
| SC19-3619-30 | f54dg00_v2r3.md | z/OS ISPF Dialog Developer's Guide, V2R3 |
| SC19-3626-30 | f54sg00_v2r3.md | z/OS ISPF Services Guide, V2R3 |

## Requirements Documents

| File | Aspect |
|------|--------|
| 01-edit-session-lifecycle.md | Session entry, data loading, END/CANCEL/RETURN |
| 02-edit-profile-and-modes.md | Profile storage, display, locking, mode commands |
| 03-line-commands.md | All line commands: insert, delete, copy, move, shift, case, exclude |
| 04-primary-commands.md | All primary commands: find, change, sort, number, bounds, save |
| 05-find-change-search-strings.md | String types, qualifiers, column ranges, label ranges |
| 06-edit-recovery-and-undo.md | Crash recovery, UNDO command, recovery macros |
| 07-syntax-highlighting.md | HILITE command, language detection, colouring categories |
| 08-boundaries-tabs-masks.md | =BNDS>, =TABS>, =MASK>, default bounds by data set type |
| 09-sequence-numbers.md | Standard and COBOL sequence numbers, RENUM, UNNUMBER |
| 10-primary-option-menu-and-navigation.md | POM, action bar, function keys, split screen, jump function |
| 11-edit-macros.md | Macro invocation, ISREDIT commands, initial macros, line command tables |
| 12-hexadecimal-display.md | HEX ON/OFF, HX line command, undisplayable characters |

## EARS Format

Every requirement follows the EARS (Easy Approach to Requirements Syntax) format:

```
WHEN <condition> THE <system> SHALL <behaviour>
```

Optional extensions:
- WHILE <state> WHEN <condition> THE <system> SHALL <behaviour>
- WHERE <feature> THE <system> SHALL <behaviour>

## Notes

- These requirements are derived from IBM documentation for reference purposes.
- They describe ISPF behaviour as a model for FileForge Workbench implementation.
- They are NOT verbatim IBM text -- they are re-expressed as EARS criteria.
- The FileForge implementation may deviate where the architecture differs.
- Before implementing any feature, cross-reference with the relevant spec under
  docs/specs/ and follow the new-requirements-gate process.
