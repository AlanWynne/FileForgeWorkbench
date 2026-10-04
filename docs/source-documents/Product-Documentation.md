# Product Documentation

What you are describing is actually a very common evolution of an open-source project.

During development, the repository accumulates:

Design notes
Architecture discussions
ADRs (Architecture Decision Records)
Meeting notes
Experiments and proofs of concept
AI-generated design documents
Draft requirements
Historical discussions
Temporary migration plans

As the project matures, you want the repository to transition from a development repository to a product repository.

I would not recommend deleting the historical information. GitHub already gives you a mechanism for preserving history without shipping it as part of the product.

Typical Approach
During Development

Keep everything.

For FileForgeWorkbench, this might look like:

/docs
    /requirements
    /architecture
    /design-notes
    /research
    /proof-of-concepts
    /meeting-notes
    /adrs


This is your project's institutional memory.

Release Preparation Phase

Create a distinction between:

Developer Documentation
/docs/developer


Contains:

Architecture
ADRs
Internal design
Plugin framework
Storage model
Dataset catalog design
JES emulator design
CICS emulator design
User Documentation
/docs/user


Contains:

Installation Guide
Getting Started
User Manual
Tutorial
Reference Guide
Command Reference
Help System
/help


or

/docs/help


Contains the markdown that becomes the integrated help viewer.

Release Branch

A common pattern is:

main


contains all history.

Then:

release/v1


contains only documentation intended for users.

You don't delete the development docs. You simply don't publish them.

Better Approach for FFWB

Given how much design material you are generating, I would move the historical documentation to a separate repository entirely.

Example:

Repository 1
FileForgeWorkbench


Contains:

src/
docs/user/
docs/developer/
help/

Repository 2
FileForgeWorkbench-Design-History


Contains:

requirements/
research/
architecture-evolution/
adrs/
meeting-notes/
ai-sessions/
prototypes/


Think of this as the equivalent of an engineering notebook.

This keeps the production repository much cleaner.

What Most Mature Projects Publish

A released project typically contains:

README.md
LICENSE
CONTRIBUTING.md
CHANGELOG.md

/docs
    GettingStarted.md
    UserGuide.md
    CommandReference.md
    PluginDevelopment.md
    ArchitectureOverview.md

/help


But does not normally contain:

OldExperiments.md
AlternativeArchitectureV3.md
MeetingNotes2025.md
RejectedIdeas.md
ScratchPad.md


Those remain in Git history or a separate design repository.

What I Would Do for FileForgeWorkbench

Given the way you are developing FFWB, I would plan three documentation tiers:

Tier 1 - Product

Shipped with the release.

README.md
User Guide
Installation Guide
Help System
Command Reference
Tutorials

Tier 2 - Developer

Kept in GitHub.

Architecture
Plugin Framework
Storage Model
Extension APIs
ADR Index

Tier 3 - Historical Archive

Moved to a separate repository.

Requirements evolution
AI conversations
Design alternatives
Research notes
Early subsystem designs
Discarded concepts
Roadmaps


The advantage is that when someone sees FileForgeWorkbench v1.0, they are presented with a clean, professional product repository, while you still retain the complete design history that led to the product.

For a project like FFWB, with extensive requirements, architecture documents, ADRs, and AI-assisted design records, this "Product Repo + Design History Repo" model is probably the cleanest long-term structure.