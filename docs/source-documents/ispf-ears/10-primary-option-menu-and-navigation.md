# ISPF EARS Requirements -- Primary Option Menu and Navigation

Source: z/OS ISPF User's Guide Vol I (SC19-3627-40), Chapters 1-2.

## Introduction

These requirements describe the ISPF Primary Option Menu (POM), the action bar,
function keys, split-screen, and the jump function used to navigate between
ISPF options.

## Glossary

| Term | Meaning |
|------|---------|
| POM | Primary Option Menu -- the top-level ISPF navigation panel |
| Action bar | The row of choices at the top of each ISPF panel |
| Pull-down menu | A list of choices displayed when an action bar item is selected |
| Point-and-shoot | Cursor-sensitive text that executes an action when selected |
| Jump function | Entering =n.m to navigate directly to a nested option |
| Logical screen | An independent ISPF session within a split-screen environment |
| SWAP | Command to move between logical screens |

## Requirements

### Requirement 1 -- Primary Option Menu

1.1 WHEN ISPF starts THE system SHALL display the Primary Option Menu as the
    first panel.

1.2 WHEN the user types an option number on the Option ===> line and presses
    Enter THE system SHALL navigate directly to that option.

1.3 WHEN the user types two option numbers separated by a period (e.g. 3.2) THE
    system SHALL navigate directly to the nested option, bypassing intermediate
    menus.

1.4 WHEN the user types X on the Option ===> line THE system SHALL exit ISPF
    using the log and list defaults.

1.5 WHEN the user presses F3 (Exit) on the Primary Option Menu THE system SHALL
    exit ISPF.

1.6 WHEN the user presses F12 (Cancel) on the Primary Option Menu THE system
    SHALL move the cursor to the Option ===> field.

1.7 WHEN the user selects an option from the Primary Option Menu THE system SHALL
    display the selected panel or function.

1.8 WHEN the user returns from an option THE system SHALL redisplay the Primary
    Option Menu.

### Requirement 2 -- Action Bar

2.1 WHEN the user selects an action bar choice THE system SHALL display the
    corresponding pull-down menu.

2.2 WHEN the user selects a choice from a pull-down menu THE system SHALL execute
    the corresponding action.

2.3 WHEN the user presses F12 (Cancel) while a pull-down is displayed THE system
    SHALL close the pull-down without making a selection.

2.4 WHEN the user presses F10 (Actions) THE system SHALL move the cursor to the
    first action bar choice.

2.5 WHEN the user presses F10 a second time THE system SHALL move the cursor to
    the command line.

2.6 WHEN a pull-down choice is unavailable THE system SHALL display it in a
    distinct colour with an asterisk prefix (or greyed in GUI mode).

2.7 WHEN the user enters a command on the command line before selecting an action
    bar choice THE system SHALL process the command and not display the pull-down,
    except for CANCEL, END, and RETURN which are not processed.

### Requirement 3 -- Function Keys

3.1 WHEN the user presses F1 THE system SHALL display help information for the
    current panel or the most recent short message.

3.2 WHEN the user presses F2 (Split) THE system SHALL divide the screen into two
    logical screens or reposition the split line.

3.3 WHEN the user presses F3 (End) THE system SHALL end the current function and
    return to the previous panel.

3.4 WHEN the user presses F7 (Backward) THE system SHALL scroll the current
    display up by the current scroll amount.

3.5 WHEN the user presses F8 (Forward) THE system SHALL scroll the current
    display down by the current scroll amount.

3.6 WHEN the user presses F9 (Swap) THE system SHALL move the cursor to the
    other logical screen in a split-screen pair.

3.7 WHEN the user presses F16 (Return) THE system SHALL return to the Primary
    Option Menu or the display from which a nested dialog was entered.

3.8 WHEN the user types a number on the command line and presses a scroll key
    THE system SHALL scroll by that number of lines or columns.

### Requirement 4 -- Jump Function

4.1 WHEN the user types =n on any command line THE system SHALL navigate directly
    to option n on the Primary Option Menu without returning through intermediate
    panels.

4.2 WHEN the user types =n.m on any command line THE system SHALL navigate
    directly to sub-option m of option n.

4.3 WHEN the user types =X on any command line THE system SHALL exit ISPF
    entirely if only one logical screen is active.

4.4 WHEN the user types =XALL on any command line THE system SHALL propagate an
    =X to all active logical screens to exit ISPF entirely.

4.5 WHEN =XALL encounters a logical screen running an application that does not
    support =X THE system SHALL suspend =XALL processing at that screen until
    the application is terminated.

### Requirement 5 -- Split Screen

5.1 WHEN the user issues the SPLIT command in single-screen mode THE system SHALL
    divide the display into two logical screens at the cursor position.

5.2 WHEN the user issues SPLIT NEW THE system SHALL add a new logical screen
    without moving the split line.

5.3 WHEN the maximum number of logical screens is reached and the user issues
    SPLIT THE system SHALL display a message and not create a new screen.

5.4 WHEN the user issues the SWAP command with no parameters and two screens
    exist THE system SHALL move the cursor between the two displayed screens.

5.5 WHEN the user issues SWAP LIST THE system SHALL display the ISPF Task List
    from which any active screen can be selected.

5.6 WHEN the user issues SWAP PREV or SWAP NEXT THE system SHALL move to the
    next lower or higher numbered screen respectively.

5.7 WHEN the user issues SWAP n THE system SHALL move to the screen numbered n.

5.8 WHEN the user ends the application on all but one logical screen THE system
    SHALL expand the remaining screen to the full display size.

### Requirement 6 -- Scroll Amounts

6.1 WHEN the scroll amount is set to a number THE system SHALL scroll by that
    many lines or columns.

6.2 WHEN the scroll amount is CSR THE system SHALL scroll so that the line or
    column containing the cursor moves to the edge of the scrollable area.

6.3 WHEN the scroll amount is DATA THE system SHALL scroll by one line or column
    less than a full page.

6.4 WHEN the scroll amount is HALF THE system SHALL scroll by half the number
    of lines or columns in the scrollable area.

6.5 WHEN the scroll amount is MAX THE system SHALL scroll to the limit of the
    data; after processing, the scroll amount SHALL revert to the previous setting.

6.6 WHEN the scroll amount is PAGE THE system SHALL scroll by the full height
    (up/down) or width (left/right) of the scrollable area.

### Requirement 7 -- User Profile Persistence

7.1 WHEN the user changes terminal characteristics or function key definitions
    THE system SHALL save those settings in the user profile for the next session.

7.2 WHEN the user begins a new ISPF session THE system SHALL restore the
    previously saved profile settings including scroll amounts, edit profiles,
    and function key definitions.

7.3 WHEN the user changes the current scroll amount THE system SHALL save it in
    the user profile.
