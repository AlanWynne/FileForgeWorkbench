# FileForgeWorkbench Software Requirements Specification

## Screen Collection and Replay Manager (SCRM)

### Document ID: FFWB-SRS-SCRM-001

 Version: 1.0
 Status: Draft
 Author: Alan Wynne
 Target Product: FileForgeWorkbench (FFWB)
 Subsystem: User Experience / Session Management
 Classification: Functional Specification

## 1. Introduction

### 1.1 Purpose

The Screen Collection and Replay Manager (SCRM) provides FileForgeWorkbench users with the ability to capture, organize, replay, archive, print, and export screen images generated during emulator or dialog interactions.

The capability is inspired by the IBM Personal Communications (PCOM) Print Screen Collection function which allows users to collect multiple screen captures and subsequently print or archive them.

The SCRM subsystem extends this concept by introducing:

Automatic screen collection
Replay capability
Evidence package generation
Documentation exports
Dialog-state integration
Automated testing support
UX traceability

### 1.2 Scope

The subsystem shall support:

ISPF dialogs
File-AID dialogs
SDSF dialogs
JES interfaces
CICS regions
Database plugins
Custom FFWB dialogs
Future host emulation modules

### 1.3 Business Motivation

The subsystem addresses several business requirements:

Training

Users frequently need to document screen navigation procedures.

Audit

Evidence of processing activities must be retained.

Testing

Test execution requires repeatable evidence.

Incident Investigation

Support teams require visibility into the sequence of user actions.

Documentation

Technical and business documentation often requires screenshots.

## 2. Definitions

Term	DefinitionCapture	A saved representation of a screen
Collection	A logical grouping of captures
Snapshot	An individual captured screen
Replay Session	Reviewing captures sequentially
Evidence Package	Documentation package generated from captures
Dialog State	Logical state defined in FFWB DIDL models
Session	Active user interaction instance

## 3. Architecture Overview

'''

┌──────────────────────────────┐
│  FileForgeWorkbench Session  │
└─────────────┬────────────────┘
              │
┌─────────────┴────────────────┐
│   Screen Collection Manager  │
└─────────────┬────────────────┘
              │
       ┌──────┴─────────┐
       │                │
       │                │   
 ┌─────┴───────┐ ┌──────┴───────┐
 │Capture Store│ │Replay Engine │ 
 └─────┬───────┘ └──────┬───────┘
       │                │   
 ┌─────┴───────┐ ┌──────┴───────┐
 │Export Engine│ │    Viewer    │ 
 └─────────────┘ └──────────────┘

'''

## 4. Functional Requirements

### 4.1 Collection Management

FFWB-SCRM-001

When a user starts screen collection, the system shall create a new collection.

FFWB-SCRM-002

When a collection is created, the system shall assign a unique identifier.

FFWB-SCRM-003

The system shall allow a collection to be named.

FFWB-SCRM-004

The system shall allow descriptive notes to be attached to a collection.

FFWB-SCRM-005

The system shall persist collections between application sessions.

### 4.2 Manual Screen Capture

FFWB-SCRM-006

When the user issues the command:

CAPTURE SCREEN


the system shall capture the current screen.

FFWB-SCRM-007

When a screen is captured, the system shall append it to the active collection.

FFWB-SCRM-008

The system shall assign sequential numbering to captures.

### 4.3 Automatic Screen Capture

FFWB-SCRM-009

When Automatic Capture Mode is enabled, the system shall capture each screen transition.

FFWB-SCRM-010

When an ENTER key causes a screen change, the system shall create a capture.

FFWB-SCRM-011

When a PF key causes a screen change, the system shall create a capture.

FFWB-SCRM-012

When a PA key causes a screen change, the system shall create a capture.

FFWB-SCRM-013

When a menu selection causes navigation, the system shall create a capture.

FFWB-SCRM-014

The system shall support configurable capture intervals.

### 4.4 Conditional Capture

FFWB-SCRM-015

The system shall support capture rules.

FFWB-SCRM-016

When a capture rule evaluates to TRUE, the system shall create a capture.

FFWB-SCRM-017

The system shall support conditions including:

Screen Name
Program Name
Message Class
Transaction ID
Dataset Name
User ID
FFWB-SCRM-018

The system shall support multiple active rules.

### 4.5 Screen Content Preservation

FFWB-SCRM-019

The system shall preserve all screen text.

FFWB-SCRM-020

The system shall preserve all field attributes.

FFWB-SCRM-021

The system shall preserve colour attributes.

FFWB-SCRM-022

The system shall preserve highlighting attributes.

FFWB-SCRM-023

The system shall preserve field protection attributes.

FFWB-SCRM-024

The system shall preserve cursor position.

FFWB-SCRM-025

The system shall record screen dimensions.

FFWB-SCRM-026

The system shall support both text representation and bitmap rendering.

### 4.6 Sensitive Information Handling

FFWB-SCRM-027

When a field is classified as sensitive, the system shall support masking during export.

FFWB-SCRM-028

The system shall support configurable masking rules.

FFWB-SCRM-029

The system shall allow exports with or without masking.

## 5. Replay Functionality

### 5.1 Replay Engine

FFWB-SCRM-030

The system shall provide replay of collected screens.

FFWB-SCRM-031

The system shall display screens in collection sequence.

FFWB-SCRM-032

The system shall support:

First
Previous
Next
Last

navigation actions.

FFWB-SCRM-033

The system shall support automatic playback.

FFWB-SCRM-034

The system shall support adjustable playback speed.

FFWB-SCRM-035

The system shall display capture timestamps during replay.

FFWB-SCRM-036

The system shall display the elapsed time between captures.

## 6. Export Requirements

### 6.1 Text Export

FFWB-SCRM-037

The system shall export collections as plain text.

### 6.2 Markdown Export

FFWB-SCRM-038

The system shall export collections as Markdown.

FFWB-SCRM-039

The system shall generate Markdown code blocks containing screen text.

### 6.3 HTML Export 

FFWB-SCRM-040

The system shall export collections as HTML.

FFWB-SCRM-041

The system shall render colours and attributes within HTML exports.

### 6.4 PDF Export

FFWB-SCRM-042

The system shall export collections as PDF.

FFWB-SCRM-043

The system shall generate a title page.

FFWB-SCRM-044

The system shall generate a table of contents.

FFWB-SCRM-045

The system shall generate a screen index.

FFWB-SCRM-046

The system shall allow page numbering.

### 6.5 Native Collection Export

FFWB-SCRM-047

The system shall support export to a native archive format.

FFWB-SCRM-048

The archive format shall contain:

collection.yaml
screens/
images/
metadata/

FFWB-SCRM-049

The archive format shall be zip-compatible.

### 7. Evidence Package Generation

FFWB-SCRM-050

The system shall support creation of evidence packages.

FFWB-SCRM-051

The evidence package shall include:

User
Date
Session
Collection
Screens
FFWB-SCRM-052

The system shall support inclusion of test-case identifiers.

FFWB-SCRM-053

The system shall support pass/fail status indicators.

### 8. DIDL Integration

FFWB-SCRM-054

Where a dialog state is available, the system shall record the associated dialog state.

FFWB-SCRM-055

The system shall store the DIDL state identifier.

FFWB-SCRM-056

The system shall permit replay filtering by dialog state.

FFWB-SCRM-057

The system shall permit export of state transition history.

## 9. Commands

The following commands shall be supported:

CAPTURE START
CAPTURE STOP
CAPTURE SCREEN
CAPTURE STATUS
CAPTURE LIST
CAPTURE OPEN
CAPTURE SAVE
CAPTURE LOAD
CAPTURE REPLAY
CAPTURE EXPORT TEXT
CAPTURE EXPORT MD
CAPTURE EXPORT HTML
CAPTURE EXPORT PDF
CAPTURE PURGE

## 10. Data Model

ScreenCollection
 ├── CollectionId
 ├── Name
 ├── Description
 ├── CreatedBy
 ├── CreatedTimestamp
 ├── SessionId
 └── Screens[]

ScreenCapture
 ├── CaptureId
 ├── SequenceNumber
 ├── Timestamp
 ├── ScreenName
 ├── ProgramName
 ├── DialogState
 ├── CursorRow
 ├── CursorColumn
 ├── TextBuffer
 ├── AttributeBuffer
 ├── ImageBuffer
 ├── Notes
 └── Metadata

## 11. Non-Functional Requirements

FFWB-SCRM-NFR-001

The subsystem shall support collections containing at least 10,000 captures.

FFWB-SCRM-NFR-002

Capture operations shall not visibly interrupt user interaction.

FFWB-SCRM-NFR-003

The subsystem shall support asynchronous export processing.

FFWB-SCRM-NFR-004

The subsystem shall support crash recovery for active collections.

FFWB-SCRM-NFR-005

The subsystem shall support future integration with automated test frameworks.

FFWB-SCRM-NFR-006

The subsystem shall support future integration with AI-assisted workflow analysis.

## 12. Future Enhancements

The architecture shall allow implementation of:

Screen differencing
Session recording
User activity heat maps
Process mining
AI-generated walkthrough documentation
AI-generated training manuals
Voice-narrated replay sessions
Automated test evidence generation
Export to PowerPoint
Git-managed screen collections
Architectural Recommendation

Within FileForgeWorkbench, the Screen Collection and Replay Manager should be implemented as a reusable platform service rather than being tied exclusively to terminal emulation. This allows the same capture engine to support ISPF, File-AID, SDSF, database plugins, custom dialogs, future CICS emulation, automated testing frameworks, DIDL state machines, and AI-assisted documentation generation, making it a core FFWB platform capability rather than merely a screenshot utility.
