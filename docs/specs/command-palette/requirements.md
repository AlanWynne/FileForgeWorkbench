# Requirements Document -- Command Palette

## Introduction

This spec defines the Command Palette for FileForgeWorkbench -- a fuzzy-search overlay
(Ctrl+Shift+P) that makes every registered command discoverable without requiring knowledge
of ISPF command syntax. The palette is the highest-impact single feature for reducing the
ISPF learning curve for new users.

The Command Palette is implemented in `ff-desktop` as a new modal overlay panel. It reads
from the existing `ff-command` Command_Registry and requires no new library crates.

**Source references:**
- **WB** = Workbench Architecture Brief -- command-driven architecture
- **GAP** = Phase BQ gap-analysis.md section 3.1 (Command Palette, High priority)
- **EXEC** = Phase BQ executive-assessment.md Recommendation 3

## Glossary

- **Command_Palette**: A modal overlay panel activated by Ctrl+Shift+P that presents a
  fuzzy-searchable list of all registered commands.
- **Fuzzy_Match**: A string matching algorithm that finds a query as a subsequence of a
  target string, scoring matches by contiguity and position.
- **Palette_Entry**: A single row in the Command Palette displaying a command's display
  name, category, description, and bound keyboard shortcut (if any).
- **Recent_Commands**: The list of commands most recently executed via the palette,
  shown at the top of the palette before the user types a query.

---

## Requirements

### Requirement 1: Palette Activation and Dismissal

**User Story:** As a user, I want to open the Command Palette with a keyboard shortcut and
dismiss it without executing a command, so that I can explore available commands without
side effects.

**Source:** GAP 3.1, WB

#### Acceptance Criteria

1. WHEN the user presses Ctrl+Shift+P from any context (editor, POM, File Explorer, Settings),
   THE workbench SHALL open the Command Palette as a modal overlay centered on the screen,
   with the search input field focused and ready for typing.

2. WHEN the Command Palette is open and the user presses Escape, THE workbench SHALL close
   the palette without executing any command and SHALL restore focus to the previously
   focused element.

3. WHEN the Command Palette is open and the user clicks outside the palette area, THE
   workbench SHALL close the palette without executing any command.

4. THE Command Palette SHALL be accessible from the menu bar via `View > Command Palette`.

5. WHEN the Command Palette is already open and Ctrl+Shift+P is pressed again, THE
   workbench SHALL close the palette (toggle behaviour).

---

### Requirement 2: Fuzzy Search Over Commands

**User Story:** As a user, I want to type partial words and abbreviations to find commands,
so that I can locate any command quickly without knowing its exact name.

**Source:** GAP 3.1

#### Acceptance Criteria

1. WHEN the user types characters into the Command Palette search field, THE palette SHALL
   filter the command list in real time (within one egui frame) to show only commands whose
   display name or Command_ID contains the query as a fuzzy match (subsequence match).

2. THE fuzzy matching SHALL score matches by: (a) contiguous character runs score higher
   than scattered matches; (b) matches at word boundaries (start of a word in the display
   name) score higher than mid-word matches; (c) shorter display names score higher than
   longer ones for equal match quality.

3. THE palette SHALL display results sorted by descending match score; ties SHALL be broken
   by alphabetical order of the display name.

4. WHEN the search field is empty, THE palette SHALL display the Recent_Commands list (up
   to 10 entries) followed by all registered commands in alphabetical order.

5. THE fuzzy search SHALL be case-insensitive.

6. WHEN no commands match the query, THE palette SHALL display the message
   "No commands match '<query>'" and SHALL NOT show an empty list.

---

### Requirement 3: Palette Entry Display

**User Story:** As a user, I want each palette entry to show the command name, category,
description, and keyboard shortcut so that I can confirm I am selecting the right command
and learn its shortcut at the same time.

**Source:** GAP 3.1

#### Acceptance Criteria

1. EACH Palette_Entry SHALL display: the command display name (prominent), the category
   label (subdued, e.g., "File", "Edit", "View"), and the bound keyboard shortcut if one
   exists (right-aligned, e.g., "Ctrl+S").

2. WHEN a Palette_Entry is highlighted (keyboard focus or mouse hover), THE palette SHALL
   display the command's full description in a detail area below the list.

3. THE characters in the display name that matched the fuzzy query SHALL be visually
   highlighted (bold or coloured) so the user can see why the entry matched.

4. THE palette SHALL display at most 20 entries at a time; IF more entries match, THE
   palette SHALL show a scrollable list.

---

### Requirement 4: Command Execution from Palette

**User Story:** As a user, I want to execute a command by selecting it in the palette with
Enter or a mouse click, so that I can invoke any command without using the ISPF command field.

**Source:** GAP 3.1, WB

#### Acceptance Criteria

1. WHEN the user presses Enter with a Palette_Entry highlighted, THE workbench SHALL close
   the palette and execute the selected command via the Command_Dispatch with an empty
   CommandParams map.

2. WHEN the user clicks a Palette_Entry, THE workbench SHALL close the palette and execute
   the selected command.

3. WHEN the user presses the Down Arrow key in the palette, THE highlight SHALL advance to
   the next entry; WHEN the user presses the Up Arrow key, THE highlight SHALL move to the
   previous entry; wrapping SHALL occur at the list boundaries.

4. WHEN a command is executed via the palette, THE workbench SHALL add it to the
   Recent_Commands list (most recent first, capped at 10 entries).

5. WHEN a command executed via the palette is disabled (its enabled predicate returns false),
   THE palette SHALL display the entry with a visual disabled style and SHALL NOT execute
   it; pressing Enter on a disabled entry SHALL display a brief status message explaining
   why the command is unavailable.

---

### Requirement 5: Recent Commands

**User Story:** As a user, I want recently used commands shown at the top of the palette
before I type anything, so that I can quickly re-invoke commands I use frequently.

**Source:** GAP 3.1

#### Acceptance Criteria

1. WHEN the Command Palette opens with an empty search field, THE palette SHALL display
   the last 10 commands executed via the palette in reverse-chronological order (most
   recent first) under a "Recently Used" section header.

2. THE Recent_Commands list SHALL be persisted in the session state and restored on next
   launch.

3. WHEN the user types a query, THE Recent_Commands section SHALL be hidden and the full
   fuzzy-filtered list SHALL be shown instead.

4. THE Recent_Commands list SHALL contain only commands that were successfully executed
   (not dismissed or failed due to disabled state).
