# egui Key Assignment Reference (egui 0.33)

This document describes exactly which keys and modifier combinations the egui
framework (version 0.33, as pinned in this workspace's root `Cargo.toml`) can
report to FileForgeWorkbench, and therefore which key assignments are even
POSSIBLE to bind. For each category it states:

- what egui exposes,
- what the platform/OS tends to default the key to,
- whether it can be reassigned inside FFWB,
- a recommendation on what it should map to.

All facts about the key set are taken from the egui 0.33 source of truth:
the `egui::Key` enum and the `egui::Modifiers` struct
(`crates/egui/src/data/key.rs` and `crates/egui/src/data/input.rs` in the
egui repository). Content was rephrased for compliance with licensing
restrictions.

Sources:
- egui 0.33 `Key` enum: https://github.com/emilk/egui/blob/0.33.0/crates/egui/src/data/key.rs
- egui 0.33 `Modifiers` struct: https://github.com/emilk/egui/blob/0.33.0/crates/egui/src/data/input.rs
- egui key-input tracking issue #3653: https://github.com/emilk/egui/issues/3653

---

## 1. The short answer

FFWB today binds commands to F1-F12 in four variants (base, Shift+, Ctrl+,
Alt+). egui CAN report considerably more than that, but with three hard limits
that shape what is worth offering:

1. **There is NO Windows / Super / Meta / Command modifier available on Windows
   or Linux.** egui's `Modifiers` struct has exactly five fields: `alt`, `ctrl`,
   `shift`, `mac_cmd`, and `command`. `mac_cmd`/`command` are the Mac Command
   key; on Windows and Linux `command` is just an alias for `ctrl`. There is no
   field for the Windows logo key. So `Win+<key>` combinations CANNOT be bound.
   The OS swallows most Win-key combos anyway.

2. **Several special/system keys are NOT in egui's `Key` enum at all**, so they
   can never be the trigger of a binding: Print Screen, Scroll Lock, Pause/Break,
   Caps Lock, Num Lock, and the Menu/Context (application) key. These are
   discussed in section 5.

3. **egui reports LOGICAL keys** (after the OS keymap is applied), and it has
   dedicated `Copy`/`Cut`/`Paste` key EVENTS in addition to the letter keys.
   This affects how Ctrl+C / Ctrl+V / Ctrl+X behave (section 4).

Everything egui CAN report can, in principle, be exposed as a bindable FFWB
command slot. The recommendations below separate "possible" from "worth doing".

---

## 2. The modifiers egui exposes

| Modifier | egui field | Available on Windows/Linux | Recommendation |
|----------|-----------|----------------------------|----------------|
| Shift | `shift` | Yes | Keep as a binding variant. |
| Ctrl | `ctrl` | Yes | Keep as a binding variant. |
| Alt | `alt` | Yes | Keep, but see the Alt caveat below. |
| Command (Mac) | `command` / `mac_cmd` | On Win/Linux `command` == `ctrl` | Treat `command` as the cross-platform "primary" modifier; it already equals Ctrl on Windows. |
| Windows / Super | (none) | No | Not bindable. Do not offer it. |

Modifier caveats worth encoding in the UI:

- **Alt alone** often activates the OS/app menu bar on Windows. A bare `Alt+<key>`
  binding can clash with menu mnemonics. Prefer `Ctrl+` or `Ctrl+Shift+` for new
  global actions.
- **Alt+Shift together** is flagged by egui itself as a poor cross-platform
  choice: on Mac that combination types special characters and the key press is
  usually not reported to egui at all. Avoid `Alt+Shift+<key>` for anything that
  must work cross-platform.
- egui only reports modifier STATE, not left-vs-right modifier keys. You cannot
  bind "right Ctrl" separately from "left Ctrl".

This means the realistic modifier matrix for a binding is any combination of
`Ctrl`, `Shift`, `Alt` (8 combinations including "none"), optionally treating
`command` as the primary accelerator. That is a superset of the current
base/Shift/Ctrl/Alt scheme and could be extended to `Ctrl+Shift+`,
`Ctrl+Alt+`, `Shift+Alt+`, `Ctrl+Shift+Alt+` if desired.

---

## 3. Function keys F1-F35

egui exposes `F1` through `F35`. Physical keyboards almost always stop at F12
(some media keyboards and terminals expose up to F24 via firmware). F13-F35
exist in the enum but most hardware never emits them.

| Key range | egui reports it | Typical OS default | Reassignable in FFWB | Recommendation |
|-----------|-----------------|--------------------|----------------------|----------------|
| F1 | Yes | Help (apps conventionally) | Yes | Keep F1 = Help / context help. Strong user expectation. |
| F2 | Yes | Rename (in file managers) | Yes | Good for Rename in catalog/file contexts. |
| F3 | Yes | Find-next; FFWB uses it for END/back | Yes | Already used (END/F3). Keep consistent with ISPF muscle memory. |
| F4 | Yes | none strong (Alt+F4 closes window - OS level) | Yes | Free for app use. Note Alt+F4 is OS-reserved, see below. |
| F5 | Yes | Refresh | Yes | Keep F5 = Refresh / reload. Strong expectation. |
| F6-F9 | Yes | none strong | Yes | Free. Assign to frequent editor/catalog actions. |
| F10 | Yes | Activates menu bar (OS/Windows) | Partially | Bare F10 may be intercepted by the window to focus the menu. Prefer binding Shift+F10 / Ctrl+F10 instead of bare F10. |
| F11 | Yes | Full screen (many apps) | Yes | Reasonable to keep as full-screen toggle. |
| F12 | Yes | none strong (dev tools in browsers) | Yes | Free in a native app. |
| F13-F24 | Yes (enum) | none; rarely emitted by hardware | Yes, but | Only expose if you support extended/media keyboards. Safe as "advanced" optional slots. |
| F25-F35 | Yes (enum) | essentially never emitted | Technically | Do not surface in the normal UI; vanishingly rare hardware. |

Recommendation: keep the F1-F12 x (base/Shift/Ctrl/Alt) grid you already have.
Treat bare F10 as a special case (prefer a modified variant). Optionally allow
F13-F24 as hidden "advanced" slots for users with extended keyboards, but do not
clutter the default UI with them.

---

## 4. Copy / Cut / Paste and the clipboard keys

This is the subtlest area. egui exposes the clipboard operations in TWO ways:

1. **Dedicated key events** `Key::Copy`, `Key::Cut`, `Key::Paste`. These are the
   logical clipboard actions, emitted by egui/winit when the platform recognises
   a clipboard gesture (including the dedicated Copy/Cut/Paste keys on some
   keyboards, and the standard Ctrl+C/X/V on Windows/Linux once the backend maps
   them).
2. **The letter keys** `Key::C`, `Key::X`, `Key::V` combined with the `command`
   (= Ctrl on Windows) modifier.

| Combination | egui can report it | Default behaviour | Reassignable | Recommendation |
|-------------|--------------------|--------------------|--------------|----------------|
| Ctrl+C | Yes (as `Copy` event and/or `command`+`C`) | Copy to clipboard | Discouraged | Leave as Copy. Users have an unbreakable expectation. egui/widgets may also consume it for text fields. |
| Ctrl+X | Yes (`Cut` and/or `command`+`X`) | Cut | Discouraged | Leave as Cut. |
| Ctrl+V | Yes (`Paste` and/or `command`+`V`) | Paste | Discouraged | Leave as Paste. |
| Ctrl+A | Yes (`command`+`A`) | Select All (egui text edit uses it) | Discouraged | Leave as Select All inside editors. |
| Ctrl+Z / Ctrl+Y | Yes (`command`+`Z` / `Y`) | Undo / Redo | Context-dependent | Keep as Undo/Redo. egui text widgets also use Ctrl+Z internally. |

Important behavioural note: when focus is inside an egui text widget (command
line, filter field, the editor surface), egui's own widgets CONSUME Ctrl+C/X/V/A
and the arrow/Home/End/Backspace/Delete keys for text editing. A global binding
on those combinations will NOT fire while a text field has focus. So binding a
custom command to Ctrl+C is both inadvisable (user expectation) and unreliable
(swallowed by whatever widget is focused).

Recommendation: do NOT offer Ctrl+C / Ctrl+X / Ctrl+V / Ctrl+A as freely
reassignable command slots. Reserve them for their universal clipboard/select
meaning. If FFWB needs its own copy/cut/paste semantics in a specific Context,
implement them as the HANDLER for the Copy/Cut/Paste events in that Context
rather than as a rebindable global key.

---

## 5. Keys egui does NOT expose (cannot be bound)

These keys are absent from the egui 0.33 `Key` enum, so no binding can use them
as a trigger. If a user asks to bind them, the honest answer is "the framework
cannot see this key".

| Key | In egui `Key` enum | Why / consequence | Recommendation |
|-----|--------------------|--------------------|----------------|
| Print Screen | No | Captured by the OS screenshot subsystem before the app; egui never receives it. | Not bindable. If a "capture screen" feature is wanted, trigger it from a normal bindable key (e.g. a Ctrl+Shift+ combo) and do the capture in code. |
| Scroll Lock | No | Legacy lock key, not reported. | Not bindable. |
| Pause / Break | No | Not reported. | Not bindable. |
| Caps Lock | No | Reported only as a modifier STATE affecting letters, not as a bindable key. | Not bindable. |
| Num Lock | No | Not reported as a key. | Not bindable. |
| Menu / Context (application) key | No | Not in the enum. | Not bindable. Use Shift+F10 for a "context menu" action instead, which IS bindable. |
| Windows / Super / logo key | No (not even as a modifier) | No `Modifiers` field exists; the OS reserves most Win combos. | Not bindable. |
| Media keys (volume, play/pause, etc.) | No | Not in the enum (only `BrowserBack` exists among multimedia keys, and it is noted as not working on web). | Not bindable, except `BrowserBack` (see below). |

### `BrowserBack`
egui DOES expose `Key::BrowserBack` (the Back button on multimedia keyboards;
Android sends it on the Back button). It is documented as not working on web.
On a native Windows build it may be emitted by multimedia keyboards. It is
bindable but niche. Recommendation: optionally map it to the FFWB "back / END"
navigation (consistent with F3/END), but do not rely on it being present.

---

## 6. Punctuation, digit, and letter keys

egui exposes the full set of letters A-Z, digits 0-9 (from both the main row and
numpad, reported as the same logical key), and a broad set of punctuation keys:
`: , \ / | ? ! [ ] { } ` - . + = ; '` and Space.

| Category | egui reports it | Reassignable | Recommendation |
|----------|-----------------|--------------|----------------|
| Letters A-Z + Ctrl | Yes (`command`+letter) | Yes, with care | Fine for app accelerators (e.g. Ctrl+F find, Ctrl+S save) EXCEPT the clipboard/select set in section 4. Beware text-widget consumption when a field is focused. |
| Digits 0-9 + modifier | Yes | Yes | Usable for "go to view N" style accelerators (Ctrl+1..9 is a common, safe pattern). |
| Punctuation + modifier | Yes | Yes | Usable but less discoverable; reserve for power-user actions. Note egui itself warns that Ctrl+Plus may require Shift on some keyboards. |
| Bare letter/digit (no modifier) | Yes | Discouraged globally | A bare key cannot be a global command when ANY text field has focus (the field eats it). Only usable as a Context-local key when no text input is focused (e.g. ISPF-style single-letter line commands), which FFWB already does in its own command layer. |

---

## 7. Navigation and editing keys

| Key | egui reports it | Default meaning | Reassignable | Recommendation |
|-----|-----------------|-----------------|--------------|----------------|
| Arrow Up/Down/Left/Right | Yes | Caret / list navigation | Discouraged globally | Leave to navigation; egui text widgets consume them when focused. |
| Home / End | Yes | Line/document start-end; FFWB uses END for back | Context-dependent | Keep END semantics as-is; consumed by text widgets when focused. |
| Page Up / Page Down | Yes | Scroll a page | Yes, outside text focus | Good for viewport paging. |
| Insert | Yes | Toggle insert/overwrite | Yes | Bindable; reasonable for overwrite-mode toggle. |
| Delete / Backspace | Yes | Delete | Discouraged | Leave to editing; consumed by text widgets when focused. |
| Tab | Yes | Focus traversal (FFWB tab-order contract) | No | Reserved for the unified tab-order model. Do NOT rebind. |
| Enter / Return | Yes | Activate / submit | Context-dependent | Reserved for command submission / default button in most Contexts. |
| Escape | Yes | Cancel / close | Context-dependent | Reserved for cancel/close. Avoid rebinding. |
| Space | Yes | Activate focused control / type a space | Discouraged | Leave to default; consumed by text widgets. |

---

## 7a. Arrow + modifier combinations (Ctrl / Alt / command with the arrows, Home/End, PageUp/Down)

This is the area the first version of this document under-described. The behaviour
splits into two cases that MUST be understood together: what egui's built-in
multiline TextEdit (the editor surface, the command line, any focused text field)
ALREADY does with these combinations, versus what is left free to bind.

All of the following is confirmed from egui 0.33's caret-movement source
(`move_single_cursor` in `crates/egui/src/text_selection/cursor_range.rs`).
Note egui's own code comment "alt on mac, ctrl on windows": the Alt and Ctrl
roles are deliberately swapped between platforms, so on Windows it is CTRL that
does word-wise movement and on Mac it is the Option/Alt key.

### What egui's TextEdit consumes (built-in, NOT free to bind while a field is focused)

| Combination (Windows) | Built-in TextEdit behaviour | Consumed when text field focused | Free to bind globally (no field focused) |
|-----------------------|-----------------------------|----------------------------------|-------------------------------------------|
| Ctrl+Left / Ctrl+Right | Move caret to previous / next WORD | Yes | Risky - inconsistent with field behaviour |
| Ctrl+Up / Ctrl+Down | Jump to START / END of the whole text (the `command` branch; `command` == Ctrl on Windows) | Yes | Risky |
| Ctrl+Home / Ctrl+End | Jump to start / end of the whole text | Yes | Risky |
| Alt+Left / Alt+Right | Same as Ctrl on Windows: previous / next WORD ("alt on mac, ctrl on windows") | Yes | Risky |
| Home / End (no modifier) | Start / end of the current ROW | Yes | Leave alone |
| Shift + any of the above | Same movement but EXTENDS the selection | Yes | Leave alone |

So to answer the question directly: **Ctrl+Up / Ctrl+Down do NOT behave like a
generic "move up/down"** inside a text field - egui binds them to jump-to-top /
jump-to-bottom of the text (document Home/End). And **Ctrl+PageUp / Ctrl+PageDown
have NO built-in egui meaning at all** (see below), so they do not mirror
PageUp/PageDown automatically; whatever they do is up to FFWB.

### What egui does NOT consume (genuinely free to bind)

| Combination | egui TextEdit handling | Recommendation |
|-------------|------------------------|----------------|
| Alt+Up / Alt+Down | None - falls through unhandled | FREE. Good candidates for FFWB "move line up/down" or "move selection up/down" in the editor, matching the common editor convention. |
| Ctrl+PageUp / Ctrl+PageDown | None - not handled by TextEdit | FREE. Good candidates for "previous / next tab (Workspace)" - the common browser/IDE convention. |
| Alt+PageUp / Alt+PageDown | None | FREE, but less discoverable; reserve for power-user actions. |
| PageUp / PageDown (no modifier) | Not moved by `move_single_cursor` (TextEdit does not page) | FREE outside text focus - bind to viewport paging. |
| Ctrl+Left/Right/Up/Down when NO text field is focused | egui only runs the above logic for a focused TextEdit; with nothing focused the event is reported plainly | FREE when the active Context has no focused text widget, but AVOID because the SAME keys silently change meaning the moment a field gains focus. Inconsistent, so do not offer them as global slots. |

### The practical rule for arrow-family bindings

- Treat **Ctrl+Arrow, Ctrl+Home/End, Alt+Left/Right, and all Shift+ variants of
  them** as RESERVED for text navigation. They are consumed by egui whenever a
  text field has focus, so a global binding would fire unpredictably.
- Treat **Alt+Up / Alt+Down, Ctrl+PageUp / Ctrl+PageDown, Alt+PageUp / Alt+PageDown,
  and bare PageUp / PageDown** as the SAFE, bindable members of this family -
  egui leaves them untouched.
- Recommended default assignments if FFWB wants them:
  - Alt+Up / Alt+Down -> move current line (or selection) up / down in the editor.
  - Ctrl+PageUp / Ctrl+PageDown -> previous / next Workspace tab.
  - PageUp / PageDown -> viewport paging (already the natural meaning).

---

## 7b. "Defaulted" vs "available" - an important clarification

The first version of this document implied that Ctrl+F means Find and Ctrl+S
means Save. That wording was misleading, and the user's instinct is correct:
those are **available, not defaulted**.

egui is an immediate-mode GUI toolkit. For the vast majority of combinations it
does NOTHING except REPORT the key event (`command`+`F`, `command`+`S`, a function
key, etc.) to the application each frame. It attaches NO intrinsic action to
Ctrl+F, Ctrl+S, Ctrl+O, the function keys, or any app-level accelerator. Whether
Ctrl+S saves is entirely a decision FFWB makes in its own command layer. So:

- "Ctrl+S = Save" is a CONVENTION FFWB chooses to implement, not an egui default.
- The ONLY combinations egui gives intrinsic, built-in behaviour to are the ones
  its own text/selection widgets consume internally while focused: the clipboard
  set (Ctrl+C / X / V), Select All (Ctrl+A), Undo/Redo (Ctrl+Z / Y), and the
  caret-navigation family documented in section 7a (arrows, Home/End, word-wise
  Ctrl/Alt movement, Backspace/Delete, Tab for focus traversal).
- Everything else is a blank slate: reportable by egui, defaulted by nothing,
  and assigned whatever meaning FFWB's binding configuration gives it.

Put simply: egui decides what it is PHYSICALLY POSSIBLE to report; FFWB decides
what any reportable combination actually DOES. The tables elsewhere in this
document use "default" to mean the platform/user EXPECTATION for a key
(e.g. F5 = Refresh), not an egui-enforced behaviour, except where a row explicitly
says egui/TextEdit consumes it.

---

## 8. Practical recommendation for FFWB's binding UI

1. **Keep** the existing F1-F12 x (base / Shift / Ctrl / Alt) grid - every one of
   those combinations is reportable by egui.
2. **Optionally extend** the modifier set to `Ctrl+Shift+`, `Ctrl+Alt+`,
   `Shift+Alt+`, and `Ctrl+Shift+Alt+` for F-keys and safe letters/digits. All
   are reportable. Warn on `Alt+Shift+` (poor cross-platform) and bare `Alt+`
   and bare `F10` (menu-bar clashes).
3. **Reserve and do NOT offer as rebindable:** Tab, Enter, Escape, the clipboard
   set (Ctrl+C/X/V), Select All (Ctrl+A), the plain arrow/Home/End/Delete/Backspace
   editing keys, AND the consumed arrow-navigation family - Ctrl+Left/Right
   (word), Ctrl+Up/Down and Ctrl+Home/End (document start/end), Alt+Left/Right
   (word), and every Shift+ variant of those. They are consumed by egui text
   widgets while focused or carry unbreakable user expectations.
3a. **Safe to offer from the arrow family** (egui does NOT consume them):
   Alt+Up/Down (recommend: move line/selection up/down), Ctrl+PageUp/PageDown
   (recommend: previous/next Workspace tab), bare PageUp/PageDown (viewport
   paging), and Alt+PageUp/PageDown (advanced).
4. **Grey out or hide as "not supported by the framework":** Windows/Super key,
   Print Screen, Scroll Lock, Pause/Break, Caps Lock, Num Lock, Menu/Context key,
   and media keys. State plainly that egui cannot report them.
5. **Expose F13-F24 as hidden/advanced slots only**, for users with extended
   keyboards; never in the default view. Ignore F25-F35.
6. For a "capture screen" or "context menu" style feature that users associate
   with Print Screen or the Menu key, bind a normal reportable combination
   (e.g. `Ctrl+Shift+<letter>`, or `Shift+F10` for a context menu) and perform
   the action in code, rather than trying to bind the unsupported physical key.

---

## 9. One-line summary per the original question

- Can egui assign function keys to OTHER combinations/special keys? Partly.
  It supports F1-F35 with Ctrl/Shift/Alt (and Mac Command) in any combination.
- Windows/Super+key combinations? No - egui has no Super/Win modifier.
- Print Screen? No - not in egui's key set (the OS captures it).
- Ctrl+C / Ctrl+V / Ctrl+X? Yes, egui reports them (as Copy/Cut/Paste events and
  as command+C/X/V), but they should stay as clipboard operations and are
  unreliable as custom global bindings because focused text widgets consume them.
- Ctrl+Up / Ctrl+Down? egui's TextEdit treats these as jump-to-top /
  jump-to-bottom of the text (document Home/End), NOT a generic move up/down -
  so they are reserved, not free.
- Ctrl+PageUp / Ctrl+PageDown? egui gives them NO built-in meaning; they are
  free to bind (recommended: previous/next Workspace tab).
- Alt+Up / Alt+Down? Free - egui does not consume them (recommended: move
  line/selection up/down). But Alt+Left / Alt+Right ARE consumed (word movement,
  same as Ctrl on Windows), so those are reserved.
- Is Ctrl+F / Ctrl+S DEFAULTED to Find / Save? No. egui attaches no action to
  them; it only reports the event. "Find" / "Save" are conventions FFWB would
  implement. They are AVAILABLE, not defaulted - the user's reading is correct.
