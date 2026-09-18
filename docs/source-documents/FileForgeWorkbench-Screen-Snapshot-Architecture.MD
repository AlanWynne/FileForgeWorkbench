# FileForgeWorkbench Screen Snapshot Architecture

I think you should treat this exactly the same way that ISPF treats a panel:
The visual rendering and the logical screen representation should be separated.
In ISPF, what you see on the terminal is actually derived from a panel definition and a set of field values. The terminal display is merely one rendering of that model.
For FFWB, I would recommend introducing a Screen Snapshot Service that produces a textual representation of the current UI state independently of egui rendering.
Recommended Architecture
Layer 1 - Dialog/Window State
You already have dialog definitions and state models.
struct ScreenModel {
title: String,
fields: Vec<Field>,
tables: Vec<Table>,
messages: Vec<String>,
status_bar: StatusBar,
command_line: String,
}
This becomes the authoritative representation.
Layer 2 - egui Renderer
The normal renderer draws:
Field -> egui::TextEdit
Table -> egui::Grid
Button -> egui::Button
This is your live GUI.
Layer 3 - Text Snapshot Renderer
A second renderer converts the same ScreenModel into text.
+---------------------------------------------------------------+
| FileForge Workbench - Dataset Properties |
+---------------------------------------------------------------+

Dataset Name . . . . : USER.TEST.PDS
Record Format . . . : FB
LRECL . . . . . . . : 80
Block Size . . . . . : 3120

Commands:
[Save] [Cancel]

Command ===>
This renderer would never interact with egui.
Instead it would render directly from the model.
________________________________________
Why This Is Better
If you attempt to capture the screen from egui widgets themselves:
Window
├─ Button
├─ TextEdit
├─ Grid
└─ Panel
you'll end up fighting the GUI framework.
Questions arise:
•	What if a control is partially hidden?
•	What about scroll areas?
•	What if widgets overlap?
•	What if layouts change?
Instead, rendering from the underlying ScreenModel always produces a clean "logical" representation.
________________________________________
ISPF Style Box Drawing
I would absolutely use Unicode box characters.
┌────────────────────────────────────────────┐
│ Dataset Browser │
├────────────────────────────────────────────┤
│ NAME TYPE RECFM LRECL │
├────────────────────────────────────────────┤
│ CUSTOMER.MASTER PS FB 256 │
│ CUSTOMER.BACKUP PS FB 256 │
└────────────────────────────────────────────┘
Fallback option:
+--------------------------------------------+
| Dataset Browser |
+--------------------------------------------+
For environments where Unicode isn't desired.
________________________________________
ANSI Colour Support
This is the feature I'd definitely include.
Example:
\x1b[31mERROR\x1b[0m
\x1b[32mSUCCESS\x1b[0m
\x1b[33mWARNING\x1b[0m
When pasted into:
•	VS Code
•	Windows Terminal
•	Linux terminals
colours remain visible.
When pasted into Word:
the text remains readable.
Therefore:
enum SnapshotMode {
PlainText,
AnsiColor,
Markdown,
}
________________________________________
Multiple Export Formats
I'd suggest a command:
CAPTURE SCREEN
or
SNAPSHOT
with options:
SNAPSHOT TEXT
SNAPSHOT ANSI
SNAPSHOT MARKDOWN
SNAPSHOT HTML
``
Text Output
screen.txt
Markdown Output
# Dataset Properties

| Property | Value |
|-----------|----------|
| Dataset | USER.TEST.PDS |
| RECFM | FB |
| LRECL | 80 |
 
Very useful for requirements, issue logging and AI prompts.
HTML Output
Preserves colours and layout.
 
________________________________________
AI Integration
Since FFWB is becoming highly AI-assisted, I would also add:
SNAPSHOT AI
This produces a structured format.
screen:
title: Dataset Properties

fields:
- name: Dataset Name
value: USER.TEST.PDS

- name: RECFM
value: FB

- name: LRECL
value: 80

buttons:
- Save
- Cancel

cursor:
field: Dataset Name
An AI agent can understand this far better than a screenshot.
________________________________________
My Preferred Design
For FFWB I would implement all four:
ScreenModel
│
├── egui Renderer
│
├── Text Renderer
│
├── Markdown Renderer
│
└── AI/YAML Renderer
and expose commands such as:
SNAPSHOT
SNAPSHOT TEXT
SNAPSHOT MARKDOWN
SNAPSHOT YAML
SNAPSHOT HTML
This is very much in the spirit of ISPF. Rather than capturing pixels, you are capturing the logical panel definition and state, which is more useful for documentation, Git repositories, defect reports, requirements extraction, and feeding context into AI agents. For FileForgeWorkbench, I would regard this as a core subsystem rather than a utility feature.

