# Accessibility Requirements

## Introduction

This sub-project defines cross-cutting accessibility requirements for
FileForge Workbench. It covers WCAG AA colour contrast, keyboard-only
operation, screen reader support, and focus indicators across all panels.

These requirements apply to every panel, dialog, and interactive element
in `ff-desktop`. They are cross-cutting -- each implementing crate must
satisfy the criteria that apply to its rendered output.

## Glossary

| Term | Definition |
|------|-----------|
| WCAG AA | Web Content Accessibility Guidelines 2.1 Level AA -- the minimum compliance target |
| Focus indicator | A visible outline or highlight showing which element has keyboard focus |
| Screen reader | Assistive technology that reads UI content aloud (NVDA, JAWS, VoiceOver) |
| Keyboard-only operation | The ability to perform every user action without a pointing device |
| Contrast ratio | The luminance ratio between foreground and background colours |
| Interactive element | Any button, text field, checkbox, list item, or menu item the user can activate |

---

## Requirement 1: WCAG AA Colour Contrast

**User Story:** As a user with low vision, I want all text and interactive
elements to meet WCAG AA contrast ratios, so that I can read the interface
without assistive magnification.

**Source:** Gap analysis section 7.5 -- WCAG AA compliance (MISSING, High priority).

### Acceptance Criteria

1. WHEN any text is rendered in any panel or dialog, THE contrast ratio
   between the text foreground colour and its background SHALL be at least
   4.5:1 for normal text (below 18pt) and at least 3:1 for large text
   (18pt or above).
2. WHEN any interactive element (button, checkbox, text field border,
   list item highlight) is rendered, THE contrast ratio between the
   element boundary or fill and its surrounding background SHALL be at
   least 3:1.
3. THE three built-in themes (dark, light, high-contrast) SHALL each
   satisfy criteria 1.1 and 1.2 for all colour token combinations used
   in those themes.
4. WHEN a user-defined custom theme is loaded, THE configuration system
   SHALL emit a warning via the logging subsystem for any colour token
   pair that fails the 4.5:1 ratio -- the theme SHALL still load.
5. THE high-contrast theme SHALL achieve a minimum contrast ratio of
   7:1 for all text (WCAG AAA for text) to serve users with severe
   low vision.

---

## Requirement 2: Keyboard-Only Operation

**User Story:** As a user who cannot use a pointing device, I want every
action in the workbench to be reachable via keyboard alone, so that I
can use the full feature set without a mouse.

**Source:** Gap analysis section 7.5 -- keyboard-only operation (PARTIAL, High priority).

### Acceptance Criteria

1. WHEN the workbench is running, EVERY interactive element in every
   panel and dialog SHALL be reachable by keyboard navigation (Tab,
   Shift+Tab, arrow keys, Enter, Space, Escape) without requiring a
   mouse click.
2. THE Tab/Shift+Tab focus cycle SHALL traverse all interactive elements
   in a logical reading order (top-to-bottom, left-to-right within each
   panel).
3. WHEN a modal dialog is open, THE keyboard focus SHALL be trapped
   within the dialog -- Tab SHALL NOT move focus to elements behind
   the dialog.
4. WHEN a context menu is open, THE arrow keys SHALL navigate menu items
   and Enter SHALL activate the focused item.
5. WHEN a dropdown or combo box is open, THE arrow keys SHALL navigate
   options and Enter SHALL select the focused option.
6. EVERY command available via mouse click SHALL also be available via
   a keyboard shortcut or via the Command Field / Command Palette.
7. THE workbench SHALL NOT rely on hover-only interactions -- any
   information shown on hover SHALL also be accessible via keyboard
   focus.

---

## Requirement 3: Focus Indicators

**User Story:** As a keyboard user, I want a clearly visible focus
indicator on every interactive element, so that I always know which
element will be activated when I press Enter or Space.

**Source:** Gap analysis section 7.5 -- focus indicators (PARTIAL, Medium priority).

### Acceptance Criteria

1. WHEN any interactive element receives keyboard focus, THE element
   SHALL display a visible focus indicator (outline, highlight, or
   border change) that meets a 3:1 contrast ratio against the adjacent
   background.
2. THE focus indicator SHALL be visible in all three built-in themes
   (dark, light, high-contrast).
3. WHEN focus moves from one element to another, THE previous element's
   focus indicator SHALL be removed and the new element's indicator
   SHALL appear within the same frame.
4. THE focus indicator style SHALL be consistent across all panels --
   the same visual treatment SHALL be used for buttons, list items,
   text fields, and tab headers.
5. WHEN the `FocusStop` cycle (Tab/Shift+Tab) is active, THE currently
   focused stop SHALL display the focus indicator defined in criterion
   3.1.

---

## Requirement 4: Screen Reader Support

**User Story:** As a screen reader user, I want the workbench to expose
semantic information about its UI elements, so that my screen reader can
announce the purpose and state of each element.

**Source:** Gap analysis section 7.5 -- screen reader support (MISSING, High priority).

### Acceptance Criteria

1. WHEN egui renders an interactive element (button, text field,
   checkbox, list item), THE element SHALL carry an accessible label
   that describes its purpose -- either from the visible label text or
   from an explicit accessibility annotation.
2. WHEN a button's state changes (enabled/disabled, pressed/unpressed),
   THE state change SHALL be reflected in the element's accessible
   properties so that screen readers can announce it.
3. WHEN a status message is written to the status bar, THE message text
   SHALL be exposed as a live region so that screen readers announce it
   without requiring the user to navigate to the status bar.
4. WHEN a dialog opens, THE dialog title SHALL be announced by the
   screen reader and focus SHALL move to the first interactive element
   in the dialog.
5. WHEN a list or tree is navigated by keyboard, THE screen reader
   SHALL announce the focused item's label, its position in the list
   (e.g. "item 3 of 12"), and any state (expanded/collapsed for tree
   nodes).
6. THE workbench SHALL be tested for compatibility with at least one
   screen reader on each supported platform: NVDA on Windows, VoiceOver
   on macOS, Orca on Linux.

---

## Requirement 5: Reduced Motion

**User Story:** As a user with a vestibular disorder, I want the
workbench to avoid unnecessary animations, so that motion-triggered
symptoms are not provoked.

**Source:** Gap analysis section 7.5 -- reduced motion support (MISSING, Low priority).

### Acceptance Criteria

1. WHEN the host OS reports a "reduce motion" preference (Windows:
   `SystemParametersInfo SPI_GETCLIENTAREAANIMATION`; macOS:
   `NSWorkspace.shared.accessibilityDisplayShouldReduceMotion`; Linux:
   `gtk-enable-animations = false`), THE workbench SHALL disable all
   non-essential animations (panel slide transitions, smooth scrolling
   animation, progress bar pulse).
2. THE `accessibility.reduce_motion` configuration key SHALL allow the
   user to override the OS preference independently.
3. WHEN reduced motion is active, scroll operations SHALL jump
   immediately to the target position rather than animating.
