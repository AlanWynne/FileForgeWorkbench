# File Viewers and Exporters — Architecture Design

## Overview

FileForgeWorkbench uses a family of focused crates to handle file viewing and
document export. Each crate has a single responsibility and can be used
independently, as a plugin within FFWB, or as part of a standalone application.

The first concrete implementation of this pattern is the Markdown Explorer
(`ffmdx`), which serves simultaneously as:

- A **standalone desktop application** (`ffmdx.exe`) built with egui/eframe
- A **FFWB plugin** (`ff-mdx-plugin`) registered via the `ff-plugin` trait system
- A **reusable library** (`ff-md-viewer`) consumed by both of the above

This document defines the naming conventions, crate responsibilities, and
integration patterns for all current and future viewer/exporter crates.

---

## Crate Naming Convention

All viewer and exporter crates follow the `ff-` prefix convention used
throughout the FFWB workspace:

│ Pattern │ Purpose │
│---│---│
│ `ff-<format>-viewer` │ Library: parse, render, watch a specific format │
│ `ff-<format>-export` │ Library: export content to a specific output format │
│ `ff-<name>-app` │ Binary: standalone egui application │
│ `ff-<name>-installer` │ Binary: self-extracting egui installer │
│ `ff-<name>-plugin` │ Library: `FileForgePlugin` + `FileViewer` impl for FFWB │

---

## Current Crates

### `ff-md-viewer` — Markdown Core Library
**Path:** `crates/ff-md-viewer/`
**Type:** `lib`
**Standalone:** yes — no egui dependency

Responsibilities:
- Recursive `.md` file scanning with excluded directory list
  (`node_modules`, `.git`, `target`, `.kiro`, etc.)
- Markdown → HTML fragment rendering via `pulldown-cmark`
  (tables, task lists, strikethrough, footnotes, smart punctuation)
- Live file watching via `notify` with 400 ms debounce
- `FileEntry` struct: relative path + full path

Dependencies: `pulldown-cmark`, `notify`, `crossbeam-channel`, `anyhow`

Used by: `ff-mdx-app`, `ff-mdx-plugin`

---

### `ff-html-export` — Generic HTML Exporter
**Path:** `crates/ff-html-export/`
**Type:** `lib`
**Standalone:** yes — no egui dependency

Responsibilities:
- Accept any pre-rendered HTML body fragment + title
- Wrap in a complete, styled, standalone HTML document
- Write to disk via `export_html_file(body, title, out_path)`
- Light/print-friendly CSS (no dark theme — suitable for sharing)

Dependencies: `anyhow`

Used by: `ff-mdx-app`, `ff-mdx-plugin`, future viewer crates

Future: additional themes (dark, print, branded)

---

### `ff-pdf-export` — Generic PDF Exporter
**Path:** `crates/ff-pdf-export/`
**Type:** `lib`
**Standalone:** yes — no egui dependency

Responsibilities:
- Hand-written PDF 1.7 writer — zero mandatory dependencies
- Produces **selectable, copy-pasteable text** using Base-14 Courier font
- US Letter page size (612 × 792 pt), 54 pt margins, 9 pt / 11 pt leading
- `PdfDocument` / `PdfPage` builder API
- `build_pdf_from_pages(pages)` convenience function
- Optional `protected` feature: owner-password encryption + SHA-256
  tamper-evidence hash via `lopdf` (same pattern as `ff-scrm`)

Dependencies: `anyhow`; optional `lopdf` (feature = `"protected"`)

Relationship to `ff-scrm`: the hand-written PDF writer in
`ff-scrm/src/pdf.rs` and the lopdf encryption in
`ff-scrm/src/pdf_protected.rs` are the direct predecessors of this crate.
`ff-scrm` should be refactored to delegate to `ff-pdf-export` to eliminate
the duplication.

Used by: `ff-mdx-app` (phase 2), `ff-mdx-plugin` (phase 2), `ff-scrm`
(after refactor), future viewer crates

---

### `ff-mdx-app` — Standalone Markdown Explorer Application
**Path:** `crates/ff-mdx-app/`
**Type:** `bin` → `ffmdx.exe`

Responsibilities:
- egui/eframe desktop application
- Left panel: collapsible folder tree, filter/search box
- Right panel: `egui_commonmark` live markdown renderer
- Toolbar: Open Folder, Refresh, Export menu
- Drag-and-drop folder onto window
- Live reload on external file change (via `ff-md-viewer` watcher)
- HTML export via `ff-html-export`
- PDF export via `ff-pdf-export` (phase 2)
- Keyboard shortcuts: `Ctrl+O` open, `F5` refresh, `Ctrl+F` filter

Dependencies: `ff-md-viewer`, `ff-html-export`, `egui`, `eframe`,
`egui_commonmark`, `rfd` (native file dialogs), `anyhow`

Usage:
```
ffmdx                        # scan current directory
ffmdx C:\path\to\docs        # scan specific folder
```

---

### `ff-mdx-installer` — Self-Extracting Installer
**Path:** `crates/ff-mdx-installer/`
**Type:** `bin` → `ffmdx-setup.exe`

Responsibilities:
- Small egui GUI installer (no console window)
- Embeds `ffmdx.exe` as `payload.zip` via `include_bytes!`
- Extracts to user-chosen directory (default: `%LOCALAPPDATA%\Programs\ffmdx\`)
- Updates Windows user PATH via `winreg`
- Cross-platform: PATH update is Windows-only (`#[cfg(windows)]`),
  extraction works on all platforms
- Silent mode support (future: CLI flag `--silent`)

Build process (`build-installer.bat` in workspace root):
1. `cargo build -p ff-mdx-app --release` → produces `ffmdx.exe`
2. PowerShell `Compress-Archive` → `payload.zip`
3. `cargo build -p ff-mdx-installer --release` → produces `ffmdx-setup.exe`

Dependencies: `egui`, `eframe`, `zip`, `dirs`, `anyhow`;
Windows: `winreg`

---

### `ff-mdx-plugin` — FFWB Plugin
**Path:** `crates/ff-mdx-plugin/`
**Type:** `lib`

Responsibilities:
- Implements `FileForgePlugin` from `ff-plugin`
- Implements `FileViewer` from `ff-viewers`
  - `viewer_key`: `"mdx-markdown"`
  - `supported_extensions`: `["md", "markdown"]`
  - `supported_mime_types`: `["text/markdown", "text/x-markdown"]`
  - `render(&[u8]) -> String`: delegates to `ff-md-viewer::render_to_html`
- Registers `Capability::Viewers` with FFWB capability registry
- Stateless renderer — `on_content_changed` is a no-op

Dependencies: `ff-md-viewer`, `ff-html-export`, `ff-plugin`, `ff-viewers`,
`ff-logging`, `toml`, `anyhow`

---

## Dependency Graph

```
ff-md-viewer ──────────────────────────┐
                                       ├──► ff-mdx-app  (ffmdx.exe)
ff-html-export ────────────────────────┤
                                       └──► ff-mdx-plugin  (FFWB)

ff-pdf-export ─────────────────────────────► ff-mdx-app  (phase 2)
                                             ff-mdx-plugin  (phase 2)
                                             ff-scrm  (after refactor)

ff-mdx-app ────────────────────────────────► ff-mdx-installer  (payload)

ff-plugin  ────────────────────────────────► ff-mdx-plugin
ff-viewers ────────────────────────────────► ff-mdx-plugin
```

---

## Future Viewer Crates

Following the same pattern, future crates would be:

│ Crate │ Format │ Notes │
│---│---│---│
│ `ff-html-viewer` │ HTML │ render HTML files in FFWB viewer panel │
│ `ff-pdf-viewer` │ PDF │ render PDF pages via `lopdf` or `pdfium` │
│ `ff-csv-viewer` │ CSV/TSV │ already partially in `ff-viewers/built_in/csv_table.rs` │
│ `ff-image-viewer` │ PNG/JPG/GIF │ already partially in `ff-viewers/built_in/image.rs` │
│ `ff-rst-viewer` │ reStructuredText │ for Python project docs │
│ `ff-asciidoc-viewer` │ AsciiDoc │ technical documentation format │

Each would follow the same structure:
- `ff-<format>-viewer` lib crate (no egui)
- `ff-<format>-plugin` FFWB plugin crate
- Optionally a standalone `ff-<format>-app` binary

---

## Integration with FFWB Plugin System

Plugins register via the `FileForgePlugin` trait (`ff-plugin`):

```rust
impl FileForgePlugin for MdxPlugin {
    fn metadata(&self) -> &PluginMetadata { ... }
    fn plugin_capabilities(&self) -> &[Capability] {
        // Declares Capability::Viewers with mime_types = ["text/markdown"]
    }
    fn activate(&mut self) -> Result<(), PluginError> { Ok(()) }
    // ...
}
```

The viewer itself implements `FileViewer` from `ff-viewers`:

```rust
impl FileViewer for MdxFileViewer {
    fn viewer_key(&self) -> &str { "mdx-markdown" }
    fn supported_extensions(&self) -> &[&str] { &["md", "markdown"] }
    fn render(&self, content: &[u8]) -> String {
        ff_md_viewer::render_to_html(&String::from_utf8_lossy(content))
    }
    // ...
}
```

FFWB's `ViewerPanel` hosts the rendered output. The platform calls
`render()` and displays the returned string. The viewer is stateless —
`on_content_changed` triggers a re-render via the platform's refresh cycle.

---

## Workspace Registration

All crates are members of the FFWB workspace (`Cargo.toml`):

```toml
[workspace]
members = [
    # ... existing members ...
    "crates/ff-md-viewer",
    "crates/ff-html-export",
    "crates/ff-pdf-export",
    "crates/ff-mdx-app",
    "crates/ff-mdx-installer",
    "crates/ff-mdx-plugin",
]
```

Shared workspace dependencies (`egui`, `eframe`, `crossbeam-channel`, etc.)
are declared once in the root `Cargo.toml` and referenced with
`.workspace = true` in each crate.

---

## Build Scripts

### `build-ffmdx-installer.bat` (workspace root)
```bat
cargo build -p ff-mdx-app --release
powershell Compress-Archive -Force ffmdx.exe payload.zip
cargo build -p ff-mdx-installer --release
```
Output: `target/release/ffmdx-setup.exe`

### Quick dev run
```
cargo run -p ff-mdx-app -- C:\path\to\docs
```

---

*Document created during initial Rust port of Markdown Explorer (mdx) into
the FileForgeWorkbench crate family. See also:*
*`Markdown-Viewer-and-WYSIWYG-Markdown-Editor.md` in this directory.*
