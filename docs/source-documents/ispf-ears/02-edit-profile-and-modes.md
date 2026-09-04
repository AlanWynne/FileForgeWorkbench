# ISPF EARS Requirements -- Edit Profile and Modes

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 2.

## Introduction

These requirements describe the edit profile system: how profiles are stored,
displayed, modified, and locked, and how the individual mode settings within a
profile control editor behaviour.

## Glossary

| Term | Meaning |
|------|---------|
| Edit profile | Named set of persistent mode settings for a data set type |
| Mode | A boolean or enumerated setting stored in the profile |
| Locked profile | A profile whose saved state cannot be changed by the editor |
| ZDEFAULT | Special profile used as the template for new profiles |
| Special data modes | Caps, number, pack, stats -- set automatically from data content |

## Requirements

### Requirement 1 -- Profile Storage and Retrieval

1.1 WHEN a user begins an edit session THE editor SHALL load the edit profile
    whose name matches the data set type or the explicitly specified profile name.

1.2 WHEN the user changes any mode setting during a session and the profile is
    not locked THE editor SHALL save the new value to the profile automatically.

1.3 WHEN the user creates more edit profiles than the installation-defined maximum
    THE editor SHALL delete the least-recently-used unlocked profile first.

1.4 WHEN all profiles are locked and the maximum is exceeded THE editor SHALL
    delete the least-recently-used locked profile.

1.5 WHEN no ZDEFAULT profile exists and no configuration table default is set THE
    editor SHALL create new profiles with these defaults: CAPS ON, STATS ON,
    NULLS ON, NUMBER ON, AUTOSAVE ON, NOTE ON, RECOVERY OFF, HEX OFF, TABS OFF,
    AUTONUM OFF, AUTOLIST OFF, PACK OFF, UNLOCK, IMACRO NONE, SETUNDO STG,
    HILITE DEFAULT.

### Requirement 2 -- Displaying the Profile

2.1 WHEN the user issues the PROFILE command with no operands THE editor SHALL
    display the first five profile lines showing current mode settings.

2.2 WHEN the user issues PROFILE with a number from 1 to 8 THE editor SHALL
    display that many lines of the profile.

2.3 WHEN the user issues PROFILE 9 THE editor SHALL display the complete profile
    including MASK, TABS, and BNDS lines even if they are blank or default.

2.4 WHEN the user issues PROFILE 0 THE editor SHALL process the command without
    displaying any profile lines.

2.5 WHEN the MASK and TABS lines contain all blanks THE editor SHALL omit them
    from the default profile display.

2.6 WHEN the BNDS line contains the default boundary positions THE editor SHALL
    omit it from the default profile display.

2.7 WHEN enhanced edit coloring is not enabled THE editor SHALL omit the HILITE
    status line from the profile display.

### Requirement 3 -- Locking a Profile

3.1 WHEN the user issues PROFILE LOCK THE editor SHALL save the current profile
    state and mark it so that subsequent sessions cannot permanently alter it.

3.2 WHEN a locked profile is in use and the user changes a mode during a session
    THE editor SHALL apply the change for the current session only without saving
    it to the profile.

3.3 WHEN a profile is locked THE editor SHALL still change the special data modes
    (caps, number, pack, stats) to match the data being edited.

3.4 WHEN a profile is locked THE editor SHALL NOT save changes to the special
    data modes back to the profile.

3.5 WHEN a profile is locked THE editor SHALL prevent the IMACRO command from
    changing the stored initial macro name.

### Requirement 4 -- Special Data Modes (Auto-Detection)

4.1 WHEN the editor loads a member that contains no lowercase characters THE
    editor SHALL set caps mode on.

4.2 WHEN the editor loads a member that contains lowercase characters THE editor
    SHALL set caps mode off.

4.3 WHEN the editor loads a member that was previously saved in packed format THE
    editor SHALL set pack mode on.

4.4 WHEN the editor loads a member that was not previously saved in packed format
    THE editor SHALL set pack mode off.

4.5 WHEN the editor loads a member that currently has ISPF statistics THE editor
    SHALL set STATS mode on.

4.6 WHEN the editor loads a member that does not have ISPF statistics THE editor
    SHALL set STATS mode off.

4.7 WHEN STATS mode is on and the member has no statistics THE editor SHALL
    display a warning message but SHALL NOT change the STATS mode setting.

4.8 WHEN STATS mode is off and the member has statistics THE editor SHALL
    automatically turn STATS mode on and display a message indicating the change.

### Requirement 5 -- Individual Mode Commands

5.1 WHEN the user issues CAPS ON THE editor SHALL convert all alphabetic input
    to uppercase before storing it.

5.2 WHEN the user issues CAPS OFF THE editor SHALL store alphabetic input in the
    case as typed.

5.3 WHEN the user issues AUTONUM ON THE editor SHALL renumber all lines
    automatically whenever the data is saved.

5.4 WHEN the user issues AUTOSAVE ON THE editor SHALL save data automatically
    when the user issues the END command.

5.5 WHEN the user issues HEX ON THE editor SHALL display data in hexadecimal
    format.

5.6 WHEN the user issues NULLS ON THE editor SHALL write trailing spaces at the
    end of lines as null characters, allowing insertion without overtyping.

5.7 WHEN the user issues NULLS OFF THE editor SHALL write trailing spaces as
    blank characters.

5.8 WHEN the user issues RECOVERY ON THE editor SHALL write data to a temporary
    backup file to enable recovery after a system failure.

5.9 WHEN the user issues PACK ON THE editor SHALL compress the data using
    run-length encoding when saving.

5.10 WHEN the user issues STATS ON THE editor SHALL generate or update ISPF
     statistics for PDS members when saving.

5.11 WHEN the user issues TABS ON THE editor SHALL enable tab processing using
     the positions defined in the TABS line.

### Requirement 6 -- Version and Modification Level

6.1 WHEN the editor creates statistics for a new member THE editor SHALL set the
    version number to 01 and the modification level to 00.

6.2 WHEN the user makes the first change to data in a session and STATS mode is
    on THE editor SHALL increment the modification level by 1.

6.3 WHEN the modification level reaches 99 THE editor SHALL NOT increment it
    further or wrap it back to 00.

6.4 WHEN both STATS mode and standard number mode are on THE editor SHALL replace
    the last two digits of the sequence number for changed lines with the current
    modification level.

6.5 WHEN the user issues the LEVEL command with a value THE editor SHALL set the
    modification level to that value and suppress automatic incrementing for the
    first change.

6.6 WHEN the user issues the VERSION command THE editor SHALL change the version
    number displayed at the top of the edit panel.
