Existing Edit Entry Pannel:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Menu  RefList  RefMode  Utilities  Help                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                               Edit Entry Panel                              │
│Command ===>                                                                 │
│                                                                             │
│ISPF Library:                                                                │
│   Project . . . ________CIW2                                                │
│   Group . . . . ________ . . . ________ . . . ________ . . . ________       │
│   Type  . . . . ________                                                    │
│   Member  . . . ________  (Blank or pattern for member selection list)      │
│                                                                             │
│Other Partitioned, Sequential or VSAM Data Set, or z/OS UNIX file:           │
│   Name . . . . . ____________________________________________               │
│   Volume Serial . .           (If not cataloged)                            │
│                                                                             │
│                                        Options                              │
│PDSE Generation  . . .                     Confirm Cancel/Move/Replace       │
│Initial Macro  . . . .                     Mixed Mode                        │
│Profile Name . . . . .                     Preserve VB record length         │
│Format Name  . . . . .                                                       │
│Data Set Password  . .                  Data Encoding                        │
│Record Length  . . . .                     1. ASCII                          │
│Line Command Table . .                     2. UTF-8                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```text


Proposed Edit Entry Pannel:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Menu  RefList  RefMode  Utilities  Help                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                               Edit Entry Panel                              │
│Command ===> ______________________________________________________________  │
├─────────────────────────────────────────────────────────────────────────────┤
│File Filter ===> __________________________________________________________  │
├────────────────────────┬────────────────────────────────────────────────────┤
│  │ File Tree         ▲ │ File List for current position in the tree       ▲ │
│  ├─┐                 ░ │ There should be columns for the file attributes  ░ │
│  │ |                 █ │ Clicking on a column  header should cuse the     █ │
│  │ ├─┐               █ │ file list to be sorted by the column, clicking   █ │
│  │ │ │               ░ │ second time on a column header toggles the       ░ │
│                      ░ │ sort order                                       ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ░ │                                                  ░ │
│                      ▼ │                                                  ▼ │
└────────────────────────┴────────────────────────────────────────────────────┘
```text


