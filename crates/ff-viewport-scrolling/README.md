# ff-viewport-scrolling

GUI-independent viewport and scrolling model for FileForgeWorkbench.

## Overview

This crate manages the logical viewport into a document — tracking which portion
is visible, handling scroll commands, maintaining cursor-viewport coordination,
and providing scrollbar models for GUI renderers to consume.

## Architecture

```text
┌─────────────────────────────────────────────────────┐
│  ViewportModel — core state (top_line, visible_count)│
├─────────────────────────────────────────────────────┤
│  CursorModel — cursor position + column affinity     │
├─────────────────────────────────────────────────────┤
│  CaretPolicyEngine — visibility policy (slop/strict) │
├─────────────────────────────────────────────────────┤
│  VerticalScrollbar / HorizontalScrollbar             │
├─────────────────────────────────────────────────────┤
│  SmoothScrollEngine — pixel-level scroll targets     │
├─────────────────────────────────────────────────────┤
│  ScrollCommand — command framework integration       │
└─────────────────────────────────────────────────────┘
```

## Key Design Decisions

- **GUI-independent**: No dependency on egui/winit/wgpu. The model computes
  positions; the shell renders.
- **Command-driven**: All scroll operations are expressible as `ScrollCommand`
  variants for integration with `ff-command`.
- **Configurable**: Caret policies, scroll mode, and wheel speed are runtime
  configurable.
- **Large-file safe**: 64-bit arithmetic for scrollbar mapping, monotonic
  fraction-to-line conversion, precision drag mode.

## Usage

```rust
use ff_viewport_scrolling::{ViewportModel, CursorModel, CaretPolicyEngine};

let mut viewport = ViewportModel::with_line_count(50_000);
viewport.set_visible_count(40);

let mut cursor = CursorModel::new();
let policy = CaretPolicyEngine::default_policy();

// Page down
viewport.scroll_page_down(&mut cursor);

// Cursor movement with auto-scroll
viewport.move_cursor_down(&mut cursor, 80, 50_000, &policy);
```

## Testing

```bash
cargo test -p ff-viewport-scrolling
```

The crate includes 21 unit tests, 8 integration tests, and 14 property-based
tests covering all core invariants.
