# Analysis Record: accessibility (W4.9)

- **Wave**: 4 (UI, panels, layout)
- **Nature**: CROSS-CUTTING accessibility requirements -- NO dedicated crate.
  Implemented across ff-theme (`contrast.rs` = WCAG verify), ff-desktop (focus /
  keyboard / reduced-motion / -- pending -- screen-reader), etc.
- **Spec files**: requirements.md (168 lines, 5 requirements -- `## Requirement`),
  tasks.md (22 sub-tasks, **16 done / 6 OPEN**), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

N/A -- cross-cutting requirements set, not a code unit. 5 reqs, 168 lines. No split.

---

## 2. Cross-unit consistency

### PA-WATCH-025 RESOLVED CLEAN -- WCAG verifier + high-contrast producer both in ff-theme

The W4.4 watch (high-contrast palette PRODUCER vs WCAG contrast VERIFIER boundary)
resolves cleanly: WCAG contrast VERIFICATION is `ff-theme/src/contrast.rs` -- it
checks palette colour pairs against WCAG AA thresholds (normal text >= 4.5:1, UI
elements >= 3.0:1) and returns a `ContrastWarning` per failing pair. So BOTH the
high-contrast palette (producer, ff-theme Req 5, W4.4) AND the contrast checker
(verifier, accessibility Req 1) live in ff-theme -- a clean single-crate
producer+verifier (the theme owns colours; the checker validates its own palettes).
No split, no duplication, no cross-crate ambiguity. accessibility Req 1 (WCAG AA
Colour Contrast) is IMPLEMENTED via ff-theme contrast.rs. PA-WATCH-025 CLEARED.

### Cross-cutting distribution -- consistent

Accessibility is a cross-cutting concern (like the CR-NR-058 logging or the
Command_Target routing): its requirements are satisfied by the relevant owner
crates rather than one a11y crate. Req 1 (contrast) -> ff-theme; Req 2 (keyboard-
only) -> the command/focus system (distributed); Req 3 (focus indicators) ->
shell/widgets (19 focus_indicator/ring refs); Req 5 (reduced-motion) -> shell/
animation (21 reduced_motion refs); Req 4 (screen reader) -> egui/accesskit
(PENDING). This is the appropriate model for a cross-cutting a11y spec.

### Cross-reference integrity

Cross-refs (theme-and-appearance contrast, the shell for focus/keyboard/motion,
egui/accesskit for screen reader) resolve. No dangling refs.

---

## 3. Completeness -- PA-INCOMPLETE-013 (Screen Reader / accesskit UNBUILT)

16/22 tasks done, **6 OPEN**. BUILT vs UNBUILT:

BUILT:
- Req 1 (WCAG AA Colour Contrast): ff-theme contrast.rs (ContrastWarning per pair).
- Req 2 (Keyboard-Only Operation): distributed keyboard nav (present).
- Req 3 (Focus Indicators): 19 focus_indicator/ring refs (substantially present).
- Req 5 (Reduced Motion): 21 reduced_motion refs (present).

UNBUILT (6 open tasks):
- Req 4 (Screen Reader Support), Task 4.1-4.4: NO `accesskit` feature in
  ff-desktop Cargo.toml (grep 0). Tasks: add `egui/accesskit` feature, accessible
  labels on all buttons, mark the status-bar message area as a LIVE REGION, tests.
  Screen-reader support is ENTIRELY unbuilt.
- Req 3 (Focus Indicators) verification, Task 3.2-3.3: confirm context menus
  respond to keyboard; integration test `all_dialog_fields_reachable_by_tab`.

So the substantive gap is Req 4 (screen-reader/accesskit) -- accessibility-critical
and entirely unbuilt -- plus two Req 3 verification/test tasks. Honest `[ ]`
tracking (accesskit feature genuinely absent). Recorded PA-INCOMPLETE-013 (MEDIUM):
implement Req 4 screen-reader support (egui/accesskit feature + accessible button
labels + live-region status bar + tests) and finish the Req 3 keyboard-reachability
verification. NOTE: full a11y compliance CANNOT be verified by static analysis --
WCAG conformance + screen-reader behaviour require manual testing with assistive
technologies (NVDA/JAWS/VoiceOver) and expert a11y review. The code presence
(accesskit wired, labels applied) is necessary but not sufficient; the spec's
acceptance beyond "feature present + labels applied" needs manual verification.

---

## 4. Logging audit

N/A as a unit -- accessibility is cross-cutting requirements; the owning crates
(ff-theme etc.) have their own logging posture. The ff-theme contrast checker
returns `ContrastWarning` values (surfaced, could be logged via the ff-theme
PA-LOG-030 pass). No standalone PA-LOG raised.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-013 (MEDIUM)**: implement Req 4 Screen Reader Support (egui/
  accesskit feature in ff-desktop, accessible labels on buttons, live-region status
  bar, tests) + finish Req 3 keyboard-reachability verification (3.2/3.3). NOTE:
  static analysis confirms the code SURFACE; full WCAG/screen-reader conformance
  requires MANUAL testing with assistive technologies + expert a11y review -- the
  `[x]` tasks establish the mechanism, not verified compliance.
- **PA-WATCH-025 RESOLVED**: WCAG verifier + high-contrast producer both in ff-theme
  (clean single-crate producer+verifier). No action.
- **PA-TCR (none)**: 8 TCR rows for 5 reqs -- adequate; Req 4 rows come with
  PA-INCOMPLETE-013.

No PA-STD/PA-CONFLICT (cross-cutting spec; owner crates hold the code). No
requirement CHANGE proposed; Req 4 is a correctly-gated pending feature.

---

## Summary

accessibility is a CROSS-CUTTING requirements set (WCAG AA contrast, keyboard-only,
focus indicators, screen reader, reduced motion) satisfied across owner crates
rather than one a11y crate -- the appropriate model. It RESOLVES PA-WATCH-025: WCAG
contrast VERIFICATION (`ff-theme/src/contrast.rs`, AA 4.5:1/3.0:1, ContrastWarning
per pair) lives in the SAME crate as the high-contrast palette producer (W4.4) -- a
clean single-crate producer+verifier, no split/duplication. Req 1 (contrast), Req 2
(keyboard), Req 3 (focus, mostly), Req 5 (reduced-motion) are built. The finding is
PA-INCOMPLETE-013 (MEDIUM): Req 4 Screen Reader Support (egui/accesskit) is ENTIRELY
UNBUILT (no accesskit feature, no accessible labels, no live-region status bar) plus
two Req 3 keyboard-reachability verification tasks -- honestly tracked. IMPORTANT:
full a11y/WCAG conformance requires MANUAL assistive-technology testing + expert
review; the code presence is necessary but not sufficient. No conflict, no split,
no size/ASCII issue.
