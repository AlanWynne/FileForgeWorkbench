# Proposal: Integrating ADVENT as a Hidden Easter Egg in FileForgeWorkbench

## Executive Summary

This proposal recommends incorporating a recreation of the classic Colossal Cave Adventure (ADVENT) game into FileForgeWorkbench (FFWB) as a hidden Easter egg. The feature would serve as both a tribute to the heritage of mainframe computing and a demonstration of FFWB's dialog engine capabilities.

Unlike a conventional game integration, the ADVENT implementation would be presented as a historically authentic mainframe utility that users can discover through commands, menus, or hidden system interactions, mirroring the culture of many IBM mainframe installations during the 1970s and 1980s.

Objectives
Primary Objectives
Celebrate mainframe computing history.
Provide a fun reward for users exploring FFWB.
Demonstrate the flexibility of the FFWB dialog engine.
Showcase state-driven applications implemented using DIDL/YAML.
Create positive developer engagement.
Secondary Objectives
Provide a reference implementation of complex dialog processing.
Exercise:
Command parsing
State management
Persistent user data
Context-sensitive help
Session recovery
Historical Justification

# Many IBM installations historically maintained unofficial games and utilities on:

VM/CMS
TSO
MVS
VSE

Examples included:

ADVENT
ZORK
TREK
WUMPUS

System programmers often distributed these through shared libraries and EXECs.

Including ADVENT reinforces FFWB's mission of preserving and emulating classic mainframe experiences in a modern environment.

## User Experience

### Discovery

The feature should not be obvious.

Possible discovery methods:

####  Method 1 - Direct Command

ADVENT

or

PLAY ADVENT

#### Method 2 - TSO Style

TSO ADVENT

Method 3 - Hidden Option

On an "About FileForgeWorkbench" panel:

Version: 1.0

Command ===> CAVERN

#### Method 4 - PF Key Sequence

Example:

PF13 PF13 PF24


unlocking:

*** ACCESSING RESTRICTED SYSTEM SOFTWARE ***

LOADING ADVENT...

Visual Design

The game should launch using a classic terminal layout.

Example:

┌───────────────────────────────────────────────┐
│ FileForgeWorkbench Adventure System           │
├───────────────────────────────────────────────┤
│                                               │
│ You are standing at the end of a road before  │
│ a small brick building.                       │
│ Around you is a forest.                       │
│ A small stream flows out of the building.     │
│                                               │
├───────────────────────────────────────────────┤
│ Command ===>                                  │
└───────────────────────────────────────────────┘


The presentation should resemble:

3270 terminals
ISPF dialogs
VM/CMS screens
Architecture
Component Model
ffwb-advent/
│
├── Engine
│   ├── Parser
│   ├── State Manager
│   ├── Action Handler
│   └── Save Manager
│
├── Data
│   ├── rooms.yaml
│   ├── items.yaml
│   ├── vocabulary.yaml
│   └── messages.yaml
│
└── UI
    └── AdventurePanel

Reusing DIDL

The game should be implemented entirely using DIDL state definitions.

Example:

dialog:
  id: ADVENT_ROOM_001

  screen:
    title: "Colossal Cave"

    content:
      - "You are standing at the end of a road."

  commands:

    - keyword: NORTH
      next_state: ROOM_002

    - keyword: ENTER BUILDING
      next_state: ROOM_003


Benefits:

Demonstrates DIDL capability.
No hardcoded game logic.
Easily extensible.
Supports future games.
Command Parser

The parser should support historic adventure commands.

Examples:

GO NORTH
GO SOUTH
UP
DOWN

GET LAMP
DROP LAMP

LOOK

INVENTORY

HELP

OPEN DOOR

Inventory Management

Persistent inventory should be maintained within the session.

Example:

session:

  inventory:
    - keys
    - lamp

  score: 42

  moves: 179

Save and Restore

For authenticity:

SAVE ADVENT1
RESTORE ADVENT1


Stored as:

~/.ffwb/advent/


or

workspace/.ffwb/advent/

Optional Enhancements
Multiple Historic Games

The same engine could later host:

ADVENT
ZORK
TREK
WUMPUS


using identical infrastructure.

Mainframe Dataset Emulation

Instead of files, saves could appear as datasets:

USERID.ADVENT.SAVE01
USERID.ADVENT.SAVE02


visible within Dataset Explorer.

This would be a particularly enjoyable demonstration of catalog and dataset emulation.

JES Submission Joke

A user could submit:

SUBMIT ADVENT


and receive:

JOB ADVENT001 SUBMITTED

followed by:

YOU ARE STANDING AT THE END OF A ROAD...

Humorous Mainframe Integration

The game could pretend to be an ancient utility.

Startup Message:

FFWB Historical Software Archive

Loading Application:

  ADVENT V1977R1M0

Warning:

This application is unsupported.
System programmers deny knowledge of its existence.

Achievement System

Hidden achievements:

Mainframe Archaeologist

    Discover ADVENT.

Cave Explorer

    Reach Room 50.

Senior Systems Programmer

    Complete ADVENT.

Legend of the Cave

    Finish in under 300 moves.

Educational Value

The implementation would also serve as:

An example DIDL application.
A state machine demonstration.
A parser demonstration.
An integration test suite.
A showcase for FFWB dialog capabilities.

This aligns directly with your longer-term goal of using FFWB as a platform for dialog-driven applications inspired by ISPF, File-AID, and classic mainframe software.

## Recommendation

Implement ADVENT as a hidden, discoverable subsystem built entirely on top of the FFWB DIDL dialog engine.

By doing so, the feature becomes:

A tribute to mainframe history.
A fun Easter egg.
A reference implementation for DIDL.
A reusable framework capable of hosting additional classic text adventures in future releases.

The most elegant implementation would be to treat ADVENT exactly like any other FFWB dialog application, proving that the same engine capable of running ISPF-style utilities can also run a classic 1970s interactive adventure game.