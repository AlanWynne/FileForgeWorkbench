Below is a Markdown requirements document that would fit well into the FileForgeWorkbench repository as a formal specification. It is written in a style consistent with your existing FFWB specifications and includes references to candidate Rust open-source components that could be evaluated during design and implementation.

# FileForgeWorkbench
# Markdown Viewer and WYSIWYG Markdown Editor
# Software Requirements Specification

| Property | Value |
|-----------|-----------|
| Project | FileForgeWorkbench (FFWB) |
| Component | Markdown Services |
| Version | 1.0 |
| Status | Draft |
| Classification | Functional Specification |
| Author | Alan Wynne |
| Date | 2026-09-26 |

---

# 1. Purpose

This specification defines the requirements for integrating Markdown viewing and Markdown authoring capabilities into FileForgeWorkbench.

The capability shall support:

- Online product help
- Product documentation
- User manuals
- Tutorial content
- Technical specifications
- Requirements documentation
- Architecture documents
- Knowledge-base content
- Markdown file editing

The solution shall be implemented using Rust-compatible technologies and integrate cleanly with the existing egui-based FileForgeWorkbench architecture.

---

# 2. Business Motivation

FileForgeWorkbench documentation is primarily authored in Markdown format.

A Markdown rendering subsystem is therefore required to:

- Display online help
- Render embedded documentation
- Display README files
- Display project specifications
- Enable contextual help

A visual Markdown editing capability is desirable to:

- Simplify document authoring
- Reduce Markdown syntax learning requirements
- Improve usability for non-technical contributors
- Support documentation maintenance

---

# 3. Scope

## 3.1 Included

The solution shall provide:

- Markdown Viewer
- Markdown Editor
- Split View Editor
- Documentation Browser
- Hyperlink Navigation
- Embedded Image Support
- Table Rendering
- Code Block Rendering
- Search Functionality
- Help System Integration

## 3.2 Future Scope

Future releases may provide:

- Full WYSIWYG Editing
- Collaborative Editing
- Real-Time Preview
- Embedded Diagram Editing
- AI-Assisted Document Authoring
- Documentation Generation

---

# 4. Architecture Overview

```text
+---------------------+
| Markdown Document   |
+----------+----------+
           |
           v
+---------------------+
| Markdown Parser     |
+----------+----------+
           |
           v
+---------------------+
| Viewer Renderer     |
+----------+----------+
           |
           v
+---------------------+
| egui Presentation   |
+---------------------+

5. Functional Requirements
FR-MDV-001 Markdown File Loading

The system shall load Markdown documents from:

Local Files
Workspaces
Projects
Help Repositories
Embedded Resources
FR-MDV-002 Markdown Rendering

The system shall render:

Headers
Paragraphs
Lists
Numbered Lists
Tables
Hyperlinks
Block Quotes
Code Blocks
Horizontal Rules
FR-MDV-003 Syntax Support

The system shall support:

CommonMark
GitHub Flavored Markdown (GFM)

Including:

Tables
Task Lists
Strikethrough
Fenced Code Blocks
FR-MDV-004 Hyperlink Navigation

The system shall allow hyperlink selection.

The system shall support:

Internal links
External URLs
Document references
FR-MDV-005 Image Rendering

The system shall render:

PNG
JPG
JPEG
GIF
SVG (where supported)
FR-MDV-006 Table Rendering

The system shall render Markdown tables with:

Row headers
Column headers
Alignment support
Scrolling support
FR-MDV-007 Code Block Rendering

The system shall render:

Fixed width fonts
Syntax highlighting
Language identifiers
FR-MDV-008 Search Capability

The system shall support:

Find Text
Find Next
Find Previous
Incremental Search
FR-MDV-009 Documentation Browser

The system shall provide:

Table of Contents
Breadcrumb Navigation
Back Navigation
Forward Navigation
FR-MDV-010 Help System Integration

The system shall allow help files to be stored as Markdown documents.

Commands such as:

HELP
HELP EDITOR
HELP JES
HELP DATASET


shall display rendered Markdown content.

6. Markdown Editor Requirements
FR-MDE-001 Source Editing

The system shall allow direct editing of Markdown source files.

FR-MDE-002 Syntax Highlighting

The editor shall provide syntax highlighting for:

Headers
Links
Code Blocks
Lists
Tables
FR-MDE-003 Auto Save

The editor shall support configurable automatic save operations.

FR-MDE-004 Validation

The system shall identify:

Broken links
Missing images
Invalid Markdown constructs
FR-MDE-005 Editor Integration

Markdown files shall be editable using:

Standard Editor
ISPF Line Commands
ISPF Block Commands
Search Commands
Change Commands
7. Split View Requirements
FR-MDP-001 Preview Mode

The editor shall support a rendered preview mode.

FR-MDP-002 Side-by-Side View

The editor shall support:

+----------------+----------------+
| Markdown       | Preview        |
+----------------+----------------+

FR-MDP-003 Refresh Model

The preview pane shall update:

Manually
Automatically
On Save

depending on user preference.

8. WYSIWYG Requirements
FR-WYS-001 Visual Editing

The system should provide visual editing capabilities.

FR-WYS-002 Formatting Toolbar

The system should provide controls for:

Bold
Italic
Underline
Headings
Lists
Tables
Hyperlinks
FR-WYS-003 Markdown Preservation

The system shall store all documents as standard Markdown.

FR-WYS-004 Roundtrip Compatibility

Opening and saving a Markdown document shall not introduce proprietary formatting.

9. Non-Functional Requirements
NFR-001 Platform Support

The solution shall operate on:

Windows
Linux
macOS
NFR-002 Native Rust

The solution should use Rust libraries wherever practical.

NFR-003 Performance

Markdown rendering for typical documents shall complete in less than one second.

NFR-004 Memory Utilization

The viewer shall support documents exceeding 10 MB.

NFR-005 Accessibility

The solution shall support:

Keyboard Navigation
Theme Integration
High Contrast Themes
10. Candidate Open Source Components

The following open source Rust components should be evaluated.

Markdown Parsing
pulldown-cmark

Purpose:

CommonMark parser

Advantages:

Mature
Widely used
Fast

Repository:

https://github.com/pulldown-cmark/pulldown-cmark

Comrak

Purpose:

GitHub Flavored Markdown parser

Advantages:

CommonMark compatible
GFM support
Good extension support

Repository:

https://github.com/kivikakk/comrak

egui Markdown Rendering
egui_commonmark

Purpose:

Markdown rendering in egui

Advantages:

Native egui integration
CommonMark rendering

Repository:

https://github.com/lampsitter/egui_commonmark

Syntax Highlighting
syntect

Purpose:

Syntax highlighting

Advantages:

Mature
Supports many languages

Repository:

https://github.com/trishume/syntect

Rich Text Editing
egui RichText

Purpose:

Native text rendering

Possible use:

Future WYSIWYG implementation
egui TextEdit

Purpose:

Source editing

Possible use:

Markdown editor foundation
HTML Conversion
comrak HTML Renderer

Purpose:

Markdown-to-HTML conversion

Possible use:

Viewer
Printing
Exporting
Alternative Rendering Engines
markdown-rs

Purpose:

CommonMark ecosystem tools

Repository:

https://github.com/wooorm/markdown-rs

Evaluation recommended.

11. Phased Delivery Plan
Phase 1

Markdown Viewer

Deliverables:

Viewer
Help Browser
Hyperlinks
Images
Search

Recommended Technology:

egui_commonmark
pulldown-cmark
Phase 2

Markdown Editor

Deliverables:

Source Editor
Syntax Highlighting
Validation

Recommended Technology:

TextEdit
syntect
Phase 3

Split View

Deliverables:

Source Pane
Preview Pane
Real-Time Refresh
Phase 4

Visual Markdown Editor

Deliverables:

WYSIWYG Editing
Formatting Toolbar
Rich Document Model
12. Risks
Risk	DescriptionComplexity	Full Typora-style editing is significantly more complex than viewing
Performance	Large documents may require incremental rendering
Compatibility	WYSIWYG editing may introduce Markdown roundtrip issues
Maintenance	Rich text document model increases development effort
13. Success Criteria

The feature shall be considered successful when:

Markdown Help System is operational
Documentation is renderable inside FFWB
Markdown files can be edited
Split Preview mode is available
Standard Markdown is preserved without loss of information

My recommendation for FFWB would be to formally approve **Phase 1 (Viewer)** and **Phase 2 (Editor)** immediately, and make **Phase 4 (True WYSIWYG)** an optional future enhancement. The viewer/editor combination will deliver roughly 90% of the practical value for requirements, help, design documentation, and repository browsing while keeping the implementation achievable within the existing egui architecture.