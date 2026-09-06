# Accessibility Design

## Overview

Accessibility in FileForge Workbench is a cross-cutting concern. There is
no single `ff-accessibility` crate -- instead, each panel and crate must
satisfy the criteria in `requirements.md` for its own rendered output.

The `ff-desktop` binary is the primary implementation site because it owns
all egui rendering. The `ff-theme` crate owns colour tokens and is the
implementation site for contrast ratio validation.

---

## Design Decisions

### 1. No Separate Accessibility Crate

Accessibility is implemented as a set of constraints on existing crates
rather than a new crate. This avoids a circular dependency (every rendering
crate would depend on an accessibility crate that depends on them).

The enforcement mechanism is:
- `ff-theme`: contrast ratio validation at theme load time (Req 1.4)
- `ff-desktop`: focus indicator rendering in the `FocusStop` cycle (Req 3)
- `ff-desktop`: accessible labels on all egui widgets (Req 4)
- `ff-config`: `accessibility.reduce_motion` key (Req 5.2)

### 2. egui Accessibility Limitations

egui 0.29 has limited native accessibility support. The `accesskit` feature
flag enables AccessKit integration which provides screen reader support on
Windows (NVDA/JAWS via UIA), macOS (VoiceOver via NSAccessibility), and
Linux (Orca via AT-SPI2).

Implementation approach:
- Enable `egui/accesskit` feature in `ff-desktop/Cargo.toml`
- Use `egui::Response::labelled_by()` and `egui::Response::on_hover_text()`
  to attach accessible labels
- Use `egui::Context::set_accessibility_enabled(true)` at startup

### 3. Contrast Ratio Validation

The `ff-theme` crate will expose a `contrast_ratio(fg: ColourRGBA, bg: ColourRGBA) -> f32`
pure function. Theme loading calls this for all text/background token pairs
and emits warnings for failures. No theme is rejected -- only warned.

### 4. Focus Indicator Rendering

The existing `FocusStop` enum in `ff-desktop` drives the Tab/Shift+Tab
cycle. Focus indicators are rendered by drawing a coloured rect outline
around the focused element using `ui.painter().rect_stroke()`. The outline
colour is `theme.focus_ring` -- a new colour token added to `UiColours`.

### 5. Reduced Motion

A new `accessibility.reduce_motion` config key (bool, default: auto-detect
from OS) gates all animation paths. The smooth scrolling path in
`editor_panel.rs` and any future transition animations check this key
before animating.

---

## Module Layout

All accessibility work is in existing modules -- no new files are required
beyond the contrast ratio helper in `ff-theme`:

```
ff-theme/src/
  contrast.rs          -- contrast_ratio() pure function, WCAG formula

ff-desktop/src/
  shell/render.rs      -- focus_ring rendering in render_focus_indicator()
  shell/state.rs       -- accessibility.reduce_motion config key wiring
  editor_panel.rs      -- reduce_motion guard on smooth scroll
```

---

## No Design Changes Required for Existing Architecture

The accessibility requirements do not introduce new architectural layers,
new crate dependencies, or new data flows. They are constraints on the
rendering and configuration paths that already exist.
