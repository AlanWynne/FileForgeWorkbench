# File Forge Workbench — Architecture Brief

**Foundation Architecture for Phase 1 and Beyond**

| Field | Value |
|-------|-------|
| Version | 1.0 |
| Status | Architectural Baseline |
| Audience | Solution Architects, Lead Developers, AI Development Assistants, Future Contributors |

---

## 1. Purpose

This Architecture Brief establishes the foundational architectural decisions for the File Forge Workbench before implementation proceeds beyond the prototype stage.

Its purpose is to prevent architectural drift and ensure that future development activities are aligned to a single, coherent vision.

**All future design, implementation, plugin development, and AI-assisted code generation SHALL conform to the architecture defined in this document.**

---

## 2. Architectural Vision

File Forge is a **cross-platform Rust Workbench Platform**, not merely a graphical editor.

The workbench SHALL provide a unified environment for:

- File browsing
- File editing
- File analysis
- File comparison
- Data transformation
- Mainframe interaction
- Workflow execution
- Plug-in extensibility

The architecture must support future growth without requiring fundamental redesign of the platform.

---

## 3. Architectural Principles

### Principle 1: GUI Independence

The application core must never depend on a specific GUI technology.

The following layers must be GUI-neutral:

- Document Model
- Commands
- Workflows
- Plugins
- Settings
- Themes
- Storage
- Logging
- Connectivity

**The GUI is a replaceable shell around the Workbench Core.**

### Principle 2: Command Driven

Every user action is a command.

Examples:
- Open File
- Save File
- Compare Files
- Rename Dataset
- Search
- Replace
- Connect Host
- Download Member

Menus, toolbars, keyboard shortcuts, automation scripts and future AI agents will invoke commands through the same dispatcher.

### Principle 3: Configuration as Data

The following are persisted as data:

- Layouts
- Themes
- Keymaps
- User Preferences
- Workflow Definitions
- Connection Profiles

**No UI layout or theme configuration shall be hardcoded.**

### Principle 4: Workbench First

Docking, tab management and workspace management are first-class platform concepts.

- Panels are NOT GUI widgets.
- Panels are **Workbench Components** rendered by the GUI layer.

### Principle 5: Extensible by Design

The architecture must assume that:

- New file types will be added
- New editors will be added
- New host connectors will be added
- New workflows will be added
- AI services will be added

The architecture must accommodate this growth through plugins.

---

## 4. High-Level Architecture

```
+------------------------------------------------+
|              GUI Layer                          |
|  (egui, eframe, future Tauri or Slint)         |
+------------------------------------------------+
                     |
+------------------------------------------------+
|          Workbench Controller                   |
+------------------------------------------------+
                     |
+------------------------------------------------+
|              Command Bus                        |
+------------------------------------------------+
                     |
+------------------------------------------------+
|            Core Services                        |
+------------------------------------------------+
| Document | Workflow | Plugin   | Settings       |
| Theme    | Layout   | Search   | Host Connectors|
+------------------------------------------------+
                     |
+------------------------------------------------+
|      Filesystem and Connectivity               |
+------------------------------------------------+
```

---

## 5. Workspace Structure

The solution SHALL be implemented as a Rust Workspace.

### Top-Level Layout

```
file-forge/
├── apps/
│   └── fileforge-desktop
│
├── crates/
│   ├── ff-core
│   ├── ff-workbench
│   ├── ff-commands
│   ├── ff-documents
│   ├── ff-workflows
│   ├── ff-layout
│   ├── ff-themes
│   ├── ff-settings
│   ├── ff-host
│   ├── ff-plugin-api
│   ├── ff-plugin-loader
│   ├── ff-search
│   ├── ff-logging
│   └── ff-ui-egui
│
├── plugins/
│
├── themes/
│
├── layouts/
│
├── docs/
│
└── tests/
```

---

## 6. GUI Separation

### GUI Responsibilities

The GUI layer is responsible only for:

**Rendering:**
- Windows
- Panels
- Tabs
- Menus
- Toolbars
- Dialogs
- Icons
- Themes

**User Input:**
- Keyboard
- Mouse
- Drag-and-Drop
- Clipboard

**View Models:**
- Presentation State
- Rendering State
- Transient UI State

### GUI Must Not Contain

- Business Logic
- File Processing
- Workflow Logic
- Command Logic
- Plugin Logic
- Host Logic

---

## 7. Workbench Model

A Workbench is a collection of visual and logical components.

```rust
pub struct Workbench {
    pub layout: Layout,
    pub documents: DocumentRegistry,
    pub panels: PanelRegistry,
    pub commands: CommandRegistry,
    pub workflows: WorkflowRegistry,
}
```

**The Workbench is the root object of the application.**

---

## 8. Command Architecture

All application functionality is exposed as Commands.

### Command Lifecycle

```
UI Event
    ↓
Command Request
    ↓
Command Bus
    ↓
Handler Resolution
    ↓
Execution
    ↓
Event Publishing
    ↓
UI Refresh
```

### Command Trait

```rust
pub trait Command {
    fn id(&self) -> CommandId;
    fn execute(&self, context: &mut AppContext) -> Result<CommandResult>;
}
```

### Registry

```rust
pub struct CommandRegistry {
    commands: HashMap<CommandId, Box<dyn Command>>,
}
```

### Benefits

Supports:
- Menus
- Toolbars
- Hotkeys
- Macros
- Automation
- AI Integration

...without duplication.

---

## 9. Async Model

### Recommendation

Use **Tokio Runtime** as the platform-wide async framework.

**Reason:**
- Most mature Rust ecosystem
- Good plugin compatibility
- Networking support
- File operations
- Future remote host support

### UI Thread

Single-threaded. All GUI rendering occurs exclusively on the UI thread.

### Background Tasks

Performed using Tokio workers.

Examples:
- Directory Scans
- FTP Downloads
- SFTP Uploads
- Search Indexing
- File Parsing
- Mainframe Operations
- AI Calls

### Communication

Use message channels:
- `tokio::sync::mpsc`
- or `crossbeam-channel`

...between worker tasks and the UI.

### Rule

**Background threads never directly manipulate GUI state.**

---

## 10. Plugin Architecture

### Objective

Allow external extension without modifying the Workbench.

### Plugin Contract

**Plugins MAY:**
- ✅ Register Commands
- ✅ Register Panels
- ✅ Register Editors
- ✅ Register Workflows
- ✅ Register File Type Handlers
- ✅ Register Host Connectors

**Plugins May NOT:**
- ❌ Directly modify Workbench internals
- ❌ Access internal data structures
- ❌ Bypass command framework
- ❌ Manipulate layouts directly
- ❌ Modify settings storage directly

### Core Trait

```rust
pub trait FileForgePlugin {
    fn metadata(&self) -> PluginMetadata;
    fn initialize(&mut self, context: &PluginContext) -> Result<()>;
    fn shutdown(&mut self);
}
```

### Registration

```rust
pub trait PluginRegistrar {
    fn register_command(&mut self, command: Box<dyn Command>);
    fn register_editor(&mut self, editor: Box<dyn EditorFactory>);
    fn register_workflow(&mut self, workflow: Box<dyn WorkflowFactory>);
}
```

---

## 11. Connectivity Architecture

Connectivity is delivered through **Host Connectors**.

Examples:
- Local Files
- FTP
- FTPS
- SFTP
- SSH
- USS
- z/OS FTP
- z/OSMF
- TN3270
- SharePoint
- OneDrive

### Connector Trait

```rust
pub trait HostConnector {
    fn connect(&mut self);
    fn disconnect(&mut self);
    fn browse(&self);
    fn open(&self);
    fn save(&self);
    fn upload(&self);
    fn download(&self);
}
```

**The Workbench SHALL treat all resources uniformly regardless of source.**

---

## 12. Layout Architecture

The layout system is part of the Workbench Core. The GUI simply renders it.

```rust
pub struct Layout {
    root: LayoutNode,
}
```

Saved as:
```
layouts/
├── default.toml
├── programmer.toml
├── analyst.toml
└── mainframe.toml
```

---

## 13. Theme Architecture

Themes are data.

```
themes/
├── dark.toml
├── light.toml
├── highcontrast.toml
└── ispf.toml
```

The theme engine supplies colours and typography to the GUI layer.

---

## 14. Development Phases

### Phase 1 — Platform Foundation

Deliver:
- Workspace
- Core crates
- Command Bus
- Layout Engine
- Theme Engine
- Plugin API

### Phase 2 — Workbench Shell

Deliver:
- egui frontend
- Docking support
- Panel framework
- Workspace management

### Phase 3 — Document System

Deliver:
- Documents
- Buffers
- Editors
- Session recovery

### Phase 4 — Host Connectivity

Deliver:
- Local file system
- FTP
- SFTP
- SSH

### Phase 5 — Mainframe Integration

Deliver:
- z/OS FTP
- USS
- z/OSMF
- TN3270

### Phase 6 — Plugin Ecosystem

Deliver:
- Plugin SDK
- Plugin Manager
- Marketplace-ready architecture

---

## 15. Final Architectural Decision

File Forge SHALL be implemented as a **modular Rust Workbench Platform** using:

- A multi-crate workspace architecture
- A GUI-independent core
- A command-driven execution model
- Tokio-based asynchronous services
- A controlled plugin contract
- A workbench-centric layout system

**This decision becomes the baseline against which all future design and implementation work should be evaluated.**
