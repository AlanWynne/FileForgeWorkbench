# Tasks -- Screen Snapshot Service + SCRM (CR-NR-098)

Ordered, independently completable tasks. `[ ]` = pending, `[x]` = complete.
Each task cross-references the requirements criteria it satisfies. NO task is
pre-marked complete. TDD per testing.md: write the failing test first.

Waves follow design.md Section 8. Waves 0-3 are the SCRM build (gated). The
later panel-extraction refactors (design.md Wave 4+) are behaviour-preserving and
tracked as separate CRs when they land; they are NOT in this list.

## Wave 0 -- foundation crates (no ff-desktop churn)

- [ ] 1. Create the `ff-screen-model` crate (no egui, no SCRM logic)
  - [ ] 1.1 Add the crate to the workspace members and define `ScreenModel` and
        its elements (Title, Field{label,value,attrs}, Table, Message, StatusBar,
        command_line, cursor, dimensions). Attrs carry colour/highlight/
        protection/sensitive. (Req 3.1-3.8, 13.5)
  - [ ] 1.2 Define the `ScreenProvider` trait (`screen_model(&self) -> ScreenModel`).
        (Req 2.1)
  - [ ] 1.3 Implement the PlainText renderer with Unicode box-drawing default and
        ASCII fallback. (Req 4.2, 5.1, 5.2, 5.4)
  - [ ] 1.4 Implement the Ansi renderer (SGR colour codes). (Req 4.3, 5.3)
  - [ ] 1.5 Implement the Markdown renderer (fenced code block; table element ->
        Markdown table). (Req 4.4)
  - [ ] 1.6 Implement the Html renderer (colours + layout). (Req 4.5)
  - [ ] 1.7 Implement the Yaml/AI renderer (title/fields/tables/buttons/cursor).
        (Req 4.6)
  - [ ] 1.8 Define `SnapshotFormat` enum (PlainText, Ansi, Markdown, Html, Yaml).
        (Req 5.5)
  - [ ] 1.9 Proptest: every renderer preserves all visible field text as
        selectable characters (the hard constraint). (Req 1.1, 1.2)

- [ ] 2. Create the `ff-scrm` crate (no egui; depends on ff-screen-model)
  - [ ] 2.1 Add the crate; define `ScreenCollection` and `ScreenCapture` data
        model incl. DIDL fields (DialogState id). (Req 17.1, 17.2, 15.1, 15.2)
  - [ ] 2.2 Implement native zip archive persistence (collection.yaml, screens/,
        images/, metadata/), zip-compatible. (Req 12.5, 17.3)
  - [ ] 2.3 Implement crash-recovery journal for an active Collection. (Req 18.4)
  - [ ] 2.4 Implement capture rules + rule evaluation (Screen Name, Program Name,
        Message Class, Transaction ID, Dataset Name, User ID; multiple active).
        (Req 9.7-9.9)
  - [ ] 2.5 Implement masking rules + sensitive-field masking on export. (Req 13)
  - [ ] 2.6 Implement the replay state machine (sequence, first/prev/next/last,
        autoplay, adjustable speed, timestamps, elapsed-between). (Req 10.1-10.7)
  - [ ] 2.7 Implement text/markdown/html exporters over a Collection. (Req 12.1-12.3)
  - [ ] 2.8 Proptest: a Collection round-trips through the zip archive unchanged;
        >=10,000 captures supported. (Req 12.5, 18.1)

## Wave 1 -- thin wiring: SNAPSHOT end to end

- [ ] 3. Register SNAPSHOT commands as Function-target Command_IDs
  - [ ] 3.1 Register SNAPSHOT + TEXT/ANSI/MARKDOWN(MD)/HTML/YAML(AI); resolve via
        resolve_target -> Function; NO new hard-coded intercept. (Req 4.1-4.6,
        11.2, 11.3)
  - [ ] 3.2 First `ScreenProvider` impl: the POM (Home Context). (Req 2.1, 2.2)
  - [ ] 3.3 Deliver the rendered snapshot to the clipboard + status confirmation;
        "not capturable" path for a Context with no provider. (Req 6.1-6.3, 2.3)
  - [ ] 3.4 Full-shell egui_kittest: SNAPSHOT on the POM copies selectable text of
        the expected fields. (Req 1.1, 6.1)

## Wave 2 -- collection lifecycle + auto-capture + viewer

- [ ] 4. CAPTURE collection lifecycle commands
  - [ ] 4.1 CAPTURE START/STOP/STATUS/LIST/OPEN/SAVE/LOAD/PURGE as Function
        commands. (Req 7.1-7.7, 11.1)
  - [ ] 4.2 CAPTURE SCREEN appends to the active Collection with sequential
        numbering; auto-start a timestamp-named default when none active.
        (Req 8.1-8.4)
- [ ] 5. Automatic capture hook
  - [ ] 5.1 Hook `nav_stack::reconstruct_context` to emit a capture on Context
        transition when Auto Mode is on. (Req 9.1, 9.5, design S4)
  - [ ] 5.2 Post-command model-diff guard (content hash) to catch command-driven
        screen changes without double-capturing a transition. (Req 9.2-9.4, design S4)
  - [ ] 5.3 Configurable capture interval. (Req 9.6)
  - [ ] 5.4 Off-frame async capture write so capture never visibly interrupts
        interaction. (Req 18.2, 18.3)
- [ ] 6. SCRM viewer Context
  - [ ] 6.1 Add WorkspaceKind::ScrmViewer + TabKind + open_scrm_viewer_tab +
        reconstruct_context arm + descriptor mapping. (Req 16.1, 16.3)
  - [ ] 6.2 Implement ScrmViewerContext: WorkspaceContext with replay controls,
        selectable-text screen render, stable first-control egui::Id, returns
        InteriorFocus. (Req 16.1, 16.2, 10.8)
  - [ ] 6.3 CAPTURE REPLAY opens/focuses the viewer for a Collection; replay
        filtering by dialog state. (Req 10.1, 15.3)
  - [ ] 6.4 Mandatory full-shell first-Tab egui_kittest focus test for the viewer.
        (Req 16.2)

## Wave 3 -- exporters, evidence, protected PDF

- [ ] 7. Export commands
  - [ ] 7.1 CAPTURE EXPORT TEXT/MD/HTML wired to ff-scrm exporters; async export.
        (Req 12.1-12.3, 18.3)
  - [ ] 7.2 PDF export with title page, TOC, screen index, page numbering; real
        selectable text (printpdf). (Req 12.4, 12.6)
  - [ ] 7.3 State-transition history export. (Req 15.4)
- [ ] 8. Evidence packages
  - [ ] 8.1 Build evidence package (user/date/session/collection/screens). (Req 14.1, 14.2)
  - [ ] 8.2 Test-case id + pass/fail indicators. (Req 14.3, 14.4)
- [ ] 9. Protected / tamper-evident PDF
  - [ ] 9.1 PRE-SLICE CHECKPOINT: confirm the pinned PDF crate versions can apply
        owner-password encryption + permission flags (edit-lock, copy-allow).
        Surface as a blocker if not; do not work around. (design S5, Req 20.2)
  - [ ] 9.2 CAPTURE EXPORT PDF PROTECTED: copy-enabled, edit-locked via owner
        password; selectable text preserved. (Req 20.1, 20.2, 20.3, 20.7)
  - [ ] 9.3 Optional user (open/read) password. (Req 20.2a, Q7)
  - [ ] 9.4 Embed content hash in PDF + evidence metadata (tamper-evidence) +
        generating user/timestamp/collection id. (Req 20.4, 20.5)
  - [ ] 9.5 Evidence package as a protected PDF. (Req 14.5)
  - [ ] 9.6 Digital signature add-on: design-confirm the signer path before build;
        deferred configurable option, surfaced to owner. (Req 20.4a, Q6)

## Wave 2R -- recorder delta (command + screen timeline)

Extension of CR-NR-098 (owner-confirmed). Records the triggering command on each
auto-Capture so a Collection becomes a command + screen timeline. Reuses
CAPTURE START/STOP/REPLAY and the Collection format -- NO new RECORD verb, NO new
artefact. TDD per testing.md: failing test first; each test carries its
`// Validates:` annotation.

- [ ] 11. TriggeringCommand on the Capture data model (`ff-scrm` / `ff-screen-model`)
  - [ ] 11.1 Add an optional `TriggeringCommand { command_string, target_kind }`
        to `ScreenCapture`; represent absence as `Option`, not an empty-string
        sentinel. (Req 17.2, 17.2a, 9.12)
  - [ ] 11.2 Round-trip `TriggeringCommand` (present and absent) through the zip
        archive persistence unchanged -- extend the Wave-0 round-trip proptest.
        (Req 17.2a, 12.5)
- [ ] 12. Populate TriggeringCommand from the dispatch seam
  - [ ] 12.1 At the auto-capture hook (task 5), thread the command just resolved
        by `resolve_target` / dispatched by `shell/target_dispatch.rs` into the
        Capture as its `TriggeringCommand`; NO bespoke intercept. (Req 9.10, 9.11)
  - [ ] 12.2 Leave `TriggeringCommand` absent for interval-driven (Req 9.6) and
        rule-driven (Req 9.7) captures. (Req 9.12)
  - [ ] 12.3 Unit test: a command-caused transition records the canonical command
        string + resolved target kind; an interval/rule capture records none.
        (Req 9.10, 9.12)
- [ ] 13. Show the cause in Replay
  - [ ] 13.1 In `ScrmViewerContext` replay render, show a selectable "caused by:
        <command>" line for a Capture that carries a TriggeringCommand; omit it
        when absent. (Req 10.9, 10.10)
  - [ ] 13.2 Full-shell egui_kittest: a recorded command-caused transition shows
        its cause line as selectable text; an interval capture shows no cause
        line. (Req 10.9, 10.10)

## Wave S -- SCRM menu-bar surfacing (FORWARD-PLAN Phase 2, decision 1)

Surface the already-built SNAPSHOT / CAPTURE family on the menu bar. Pure
surfacing: NO new verb, intercept, dispatcher, or navigation stack; each item
dispatches an EXISTING verb through the single front door (command parity).
Builds ON menu-workspace Req 17 (CR-NR-080) + code-only built-in menu content
(CR-CH-021). The bare-core POM stays EXACTLY `0`/`3`/`X` (CR-CH-073).

- [x] 14. Surface SCRM as a menu-bar entry (Req 22)
  - [x] 14.1 Decouple the compiled default Menu_Bar from the barebones POM: add the
        code-only `DEFAULT_MENU_BAR_TOML` constant (Settings / Help / SCRM
        `SNAPSHOT` entry / hidden terminate) and parse it in `default_menubar_menu()`.
        `DEFAULT_POM_TOML` unchanged. (Req 22.1, 22.2)
  - [x] 14.2 Make the `SNAPSHOT` bar entry a DYNAMIC source (mirroring the
        `THEME LIST` theme picker): `dynamic_menu_options("SNAPSHOT")` yields one
        child per existing `SNAPSHOT <format>` / `CAPTURE ...` verb, each dispatched
        via `dispatch_command_string` (single front door). (Req 22.3, 22.4)
  - [x] 14.3 Tests: unit (default bar carries SNAPSHOT, POM unchanged, dynamic
        source lists existing verbs, command parity at the dispatch seam) + a
        full-shell egui_kittest proving a CLICK on the `CAPTURE START` child starts
        a collection. (Req 22.1-22.5; workspace-conformance)

NOTE: Wave S (task 14) is SUPERSEDED by Wave F below. The hardwired
`DEFAULT_MENU_BAR_TOML` + dynamic-`SNAPSHOT` arrangement is REPLACED by the
owner-confirmed menu-bar-from-POM model (Req 22 re-phrased); task 14 is left
`[x]` as the record of the interim surfacing it delivered.

## Wave F -- forward-plan end-state (CR-NR-114 gate; owner-confirmed decisions)

Builds on the master screen design directive (`docs/specs/master-screen-directive/`)
and the owner-confirmed decisions D(formats)/D(hotkeys)/D(menu-bar model)/D(SCRM
keys-labels)/D(replay-save). TDD per testing.md: failing test first, each with a
`// Validates:` annotation. Every capability resolves through the single front
door; no second dispatcher, no new navigation stack.

- [ ] 15. Enrich the POM / menu ScreenProvider to the FULL screen (Req 3.9-3.13)
  - [ ] 15.1 Populate the menu Screen_Model with window chrome / title bar, menu
        bar, tab header, title line, command line WITH value, POM option rows WITH
        command values, calendar, and PF-key footer. (Req 3.9, 3.11, 3.12)
  - [ ] 15.2 Show the PF-key footer per pfshow on; omit it per pfshow off.
        (Req 3.10)
  - [ ] 15.3 Build it purely from the logical model, not egui widgets. (Req 3.13)
  - [ ] 15.4 Tests: snapshot text contains the calendar + PF-key line when pfshow
        on, omits the PF-key line when off. (Req 3.9, 3.10)

- [ ] 16. Picture snapshot formats + composite clipboard (NO picker) (Req 4.1, 4.7-4.8, 23)
  - [ ] 16.1 Add the PNG picture flavor as a REAL WINDOW SCREENSHOT of the
        FileForge application window (the window's rendered pixels), NOT a
        text-rasterised image (pinned encoder dep, exact version). This matches
        the REVISED Req 1.3 / 23.2 (the former text-raster PNG is superseded).
        (Req 23.1, 23.2, 1.3, 23.7)
  - [ ] 16.2 Add `SNAPSHOT PDF` reusing `ff-scrm/pdf.rs`, selectable text.
        (Req 23.3)
  - [ ] 16.3 No-arg `SNAPSHOT` places the COMPOSITE clipboard payload (PNG
        picture + Markdown-fenced text, Req 23) directly, with NO format picker
        (Req 4.7 RETIRED the picker); the explicit `SNAPSHOT <format>` verbs are
        RETAINED for single-format output. (Req 4.1, 4.7, 4.8, 23.6)
  - [ ] 16.4 Where the format supports it, produce BOTH the picture AND the
        ANSI/TEXT rendering; the text stays authoritative-selectable, the picture
        is the co-equal image flavor (Req 1.2, 23.4, 23.5).
  - [ ] 16.5 Tests: the PNG is a real window screengrab (not a text raster); bare
        `SNAPSHOT` places the composite with no picker; both-outputs where
        supported. (Req 23.2, 4.1, 4.7, 23.4)

- [ ] 17. Capture START records bitmap + ANSI + Markdown + command (Req 9.13-9.14)
  - [ ] 17.1 At the auto-capture hook, store per Capture the bitmap (task 16
        encoder) + ANSI + Markdown + the triggering-command metadata (confirm
        Wave 2R tasks 11-13). (Req 9.13, 9.10-9.12, 17.2a)
  - [ ] 17.2 The per-transition bitmap is a REAL WINDOW SCREENSHOT (consistent
        with the revised PNG picture flavor, Req 1.3 / 23.2), NOT a text-raster;
        the ANSI/TEXT + Markdown renderings remain Screen_Model-derived. (Req
        9.14 as revised by Req 1.3 / 23.2)
  - [ ] 17.3 Tests: a command-caused transition records command + bitmap + ANSI +
        Markdown; an interval/rule capture records no command. (Req 9.13, 9.12)

- [ ] 18. Capture STOP Save-As dialog (Req 24)
  - [ ] 18.1 On `CAPTURE STOP`, show a Save-As dialog naming the session and
        choosing the output type (ANSI/TEXT | HTML | Markdown | PDF now; ODT/ODP
        deferred). (Req 24.1, 24.2)
  - [ ] 18.2 Embed the bitmap alongside the monospaced text where the type
        supports it; text stays authoritative. (Req 24.3)
  - [ ] 18.3 Dialog invokes the EXISTING `CAPTURE SAVE` / `CAPTURE EXPORT`
        commands through the single front door; persists as the zip archive.
        (Req 24.4)
  - [ ] 18.4 Cancel stops the capture but writes nothing; in-memory collection
        remains. (Req 24.5)
  - [ ] 18.5 If the dialog is a Context, it satisfies the master directive (focus
        + ScreenProvider/opt-out). (Req 24.6)
  - [ ] 18.6 Tests: the dialog choice dispatches the matching export command;
        cancel writes nothing. (Req 24.4, 24.5)

- [ ] 19. Replay: save-to-file + list/open/select + export (Req 10.11-10.14)
  - [ ] 19.1 A captured session saves to the zip archive and can be reopened for
        replay independent of the live session. (Req 10.11)
  - [ ] 19.2 List saved sessions and open/select one from a saved-session picker,
        reachable via the single front door; selecting opens the replay Context.
        (Req 10.12)
  - [ ] 19.3 Export a saved session to PDF now (reuse `ff-scrm/pdf.rs`); ODT/ODP
        deferred; via existing `CAPTURE EXPORT` commands. (Req 10.13)
  - [ ] 19.4 The saved-session picker Context satisfies workspace-conformance
        (InteriorFocus, first-Tab test) and the master directive. (Req 16.2,
        master-directive)
  - [ ] 19.5 Tests: save then reopen replays the same captures; picker lists and
        opens a saved session; export dispatches the matching command. (Req 10.11,
        10.12, 10.13)

- [ ] 20. Menu-bar-from-POM + SCRM menu family + RESET BARE (Req 22, 25)
  - [ ] 20.1 Build the menu bar at RUNTIME from the POM TOML (one bar item per POM
        option); no hardwired bar. (Req 22.1)
  - [ ] 20.2 Build each bar item's dropdown at runtime from that item's submenu
        TOML. (Req 22.2)
  - [ ] 20.3 Add the SCRM option to `DEFAULT_POM_TOML` (label "SCRM : Screen
        Capture and Recording"); record the CR-CH-073 invariant change. (Req 22.3,
        25.2)
  - [ ] 20.4 Add the code-only `DEFAULT_SCRM_MENU_TOML` SCRM menu screen (title
        "FileForge Workbench -- Screen Capture and Recording"; options Snapshot S /
        Capture start CS / Capture stop CT / Capture replay CR / SCRM Help H /
        Exit X); never written to disk (CR-CH-021). (Req 25.1, 25.3)
  - [ ] 20.5 Each SCRM option's command is an EXISTING verb resolved through
        `resolve_target` / `open_menu_by_name`; Snapshot invokes the picker (task
        16), Capture stop invokes the Save-As (task 18), Capture replay opens the
        saved-session picker (task 19). (Req 22.6, 25.4)
  - [ ] 20.6 RESET BARE generates the SCRM submenu screen AND a bare POM
        containing SCRM. (Req 22.8, 25.5)
  - [ ] 20.7 Remove the old hardwired Snapshot bar entry (superseded by
        SCRM-from-POM). (Req 22.4)
  - [ ] 20.8 Tests: the bar echoes the POM at runtime; bare POM contains SCRM;
        SCRM dropdown lists the SCRM options; each option dispatches its verb
        through the single front door (command parity); RESET BARE regenerates
        both. (Req 22.1-22.9, 25.3-25.5)

- [ ] 21. Mnemonic-letter menu-bar accelerators (Req 26)
  - [ ] 21.1 Add Alt+<letter> accelerators per bar item via the EXISTING key map
        framework, reusing the option `key` where a letter, distinct mnemonic
        where a digit. (Req 26.1, 26.2)
  - [ ] 21.2 Avoid collisions with existing Alt-bindings; do not overload POM
        numeric option keys. (Req 26.3, 26.4)
  - [ ] 21.3 An accelerator opens the bar menu through the SAME dispatch seam as a
        click. (Req 26.5)
  - [ ] 21.4 Tests: an accelerator resolves to the bar item's menu via the active
        key map; no collision with an existing Alt-binding. (Req 26.1, 26.3)

## Phase 2E -- dialog capture fixes (Req 23.2a, 23.5f)

- [x] 22. Crop the dialog picture to the dialog rectangle (Req 23.2a)
  - [x] 22.1 Capture the frontmost dialog's on-screen rect at render time and
        convert points -> physical pixels via pixels-per-point. (Req 23.2a)
  - [x] 22.2 Crop the full-framebuffer egui screenshot RGBA buffer to that rect
        (manual crop in ff-screen-model, not a silent whole-window fallback)
        before PNG encode and CF_HTML image embed; whole-window no-dialog capture
        unchanged. Validated by crop_* / points_rect_* /
        no_dialog_open_leaves_capture_uncropped. (Req 23.2a)

- [x] 23. Force monospace in the CF_HTML composite (Req 23.5f)
  - [x] 23.1 Wrap the box-drawn text in a `<pre>` with an explicit monospace font
        style; recompute CF_HTML offsets; keep the `<img>` in the fragment; secret
        stays redacted. Validated by cf_html_pre_forces_monospace_font /
        cf_html_offsets_round_trip_after_monospace_style. (Req 23.5f)

## Phase 2F -- each clipboard text flavor carries BOTH box layout AND property/value tables (Req 28, CR-CH-079)

TDD per testing.md: write the failing test first, then the minimum change. Scope
the SCOPED trio (`cargo check/clippy/fmt -p ff-screen-model` and `-p ff-desktop`)
per gate-policy.md; the full suite + the manual Word/Notepad paste are the owner's
run.

NOTE on numbering: Phase 2F tasks are prefixed `2F.` to avoid visual collision
with SCRM Requirement numbers 24-28 (which exist as separate requirements).

- [x] 2F.1 Plain-text flavor carries box layout AND Markdown tables (Req 28.1, 28.4, 28.6, 28.7, 28.8)
  - [x] 2F.1.1 Failing test (ff-screen-model): `render_markdown` on a model WITH
        `markdown_sections` contains BOTH a Unicode box glyph (U+250C family) AND
        a `| Property | Value |` table header, with the box block BEFORE the first
        table (assert by index). (Req 28.1, 28.4)
  - [x] 2F.1.2 Change `render/markdown.rs::render_markdown` so a present
        `markdown_sections` no longer early-returns tables-only: emit the fenced
        Unicode box-drawn body FIRST, then the `markdown_sections` tables. Keep the
        no-`markdown_sections` path (fenced body + fields table + table elements)
        unchanged. (Req 28.1, 28.4, 28.6, 28.7)
  - [x] 2F.1.3 Regression test: a model with NO `markdown_sections` and no tabular
        content still renders box-only (Req 28.7); a model with only tables still
        renders box + tables. (Req 28.7, 28.8)

- [x] 2F.2 HTML-table emitter for the composite rich flavor (Req 28.2, 28.6, 28.7)
  - [x] 2F.2.1 Failing test (ff-screen-model): a new HTML-table emitter renders
        `markdown_sections` (and `tables`) as `<table>`/`<tr>`/`<td>` with the
        per-group heading as a caption/heading row and a `Property`/`Value` header
        row. (Req 28.2, 28.6)
  - [x] 2F.2.2 Add the emitter in `render/html.rs` (a focused helper that renders
        the tabular content only, NOT the whole model, to avoid duplicating the
        box/chrome). (Req 28.2, 28.6, 28.7)

- [x] 2F.3 Composite CF_HTML carries image + box layout + HTML tables (Req 28.2, 28.4, 28.5)
  - [x] 2F.3.1 Failing test (ff-desktop): `clipboard_html::build_cf_html` output for
        a Config fixture contains, in order, the base64 `<img>`, the monospace box
        `<span>` paragraphs, AND `<table>` markup; and the four CF_HTML offsets
        (StartHTML/EndHTML/StartFragment/EndFragment) still round-trip with the
        tables present. (Req 28.2, 28.4, 28.5)
  - [x] 2F.3.2 Extend `build_cf_html` with a `tables_html` parameter appended inside
        the fragment after the box paragraphs and before `<!--EndFragment-->`;
        keep the per-line monospace `<span>` treatment (Req 23.5f) intact; let the
        existing two-pass offset math recompute. Update the single caller
        (`place_snapshot_composite`) to pass the emitter output. (Req 28.2, 28.4,
        28.5)
  - [x] 2F.3.3 Regression test: `build_cf_html` with NO tables (box-only model) is
        byte-identical to the pre-change output (the box-only path is unaffected).
        (Req 28.3, 28.7)

- [x] 2F.4 End-to-end flavor assertions over a Config snapshot fixture (Req 28.1, 28.2, 28.3)
  - [x] 2F.4.1 Failing test (ff-desktop): over a Config `ScreenModel` fixture, the
        `CF_UNICODETEXT` flavor string contains BOTH the box block AND the Markdown
        `| Property | Value |` tables. (Req 28.1)
  - [x] 2F.4.2 Failing test (ff-desktop): over the same fixture, the `CF_HTML` flavor
        contains BOTH the box `<span>` block AND HTML `<table>` tables, plus the
        base64 image. (Req 28.2)
  - [x] 2F.4.3 Assert the PNG/DIB picture flavor is unchanged by this change
        (picture bytes/placement identical to before). (Req 28.3)
  - [ ] 2F.4.4 MANUAL (record in TCR): paste into Word shows image + box + tables;
        paste into Notepad shows box + tables. Cannot be driven headlessly. (Req
        28.1, 28.2)
    - MANUAL: deferred to owner paste verification; automated headless coverage
      is provided by 2F.4.1/2F.4.2.

- [x] 2F.5 Reconcile single-format consumers (Req 28.8)
  - [x] 2F.5.1 Confirm `SNAPSHOT MARKDOWN` / `SNAPSHOT HTML` / `SNAPSHOT TEXT` and
        the file/PDF export paths still produce their documented output after the
        shared-renderer change; adjust or add tests where the Markdown single-file
        output now also carries the box body. (Req 28.8)
    - Reconciled: the two `tests_config_snapshot_fidelity` assertions that
      asserted a key appears once in the Markdown flavor now assert it appears
      twice (box body + table row); plain-text stays once. `SNAPSHOT HTML`/`TEXT`
      and PDF paths are unchanged (verified by the existing scrm/backend tests).

## Phase 2G -- full-window snapshot FIDELITY: faithful box mirror on snapshot AND capture (Req 29, CR-CH-115)

TDD per testing.md: write the failing test first, then the minimum change. Each
test carries its `// Validates:` annotation. Scope the SCOPED trio
(`cargo check/clippy/fmt -p ff-screen-model` and `-p ff-desktop`) per
gate-policy.md; the full suite + the manual on-screen visual match are the
owner's run. The work lives in TWO shared places (design.md Section 12): the
`ff-screen-model` grid renderer and a shared `ff-desktop` chrome helper, so every
Context inherits the fidelity (Req 29.12) and capture inherits it via the shared
seam (Req 29.14). NOTE on numbering: Phase 2G tasks are prefixed `2G.` to avoid
collision with SCRM Requirement numbers.

- [ ] 2G.1 Grid-aware table rendering in `ff-screen-model` (Req 29.5, 29.6, 29.6a, 29.7, 29.7a, 29.8, 29.9)
  - [ ] 2G.1.1 Failing test (ff-screen-model): `render_plain` of a `Table`
        renders a `|`-separated, FIXED-WIDTH header + data rows (not a `join("  ")`
        space list), filling the column widths. (Req 29.5, 29.6)
  - [ ] 2G.1.2 Change `render/text.rs` table rendering to a fixed-width,
        `|`-separated grid (ASCII `|` in the fallback per Req 5.2); compute each
        column width as max(header, cells, scratch row); pad every cell; a value
        wider than its header WIDENS the column and pads every row (no
        truncation). (Req 29.5, 29.6)
  - [ ] 2G.1.1a Failing test + change: a NARROW grid pads out to the FRAME WIDTH
        `model.dimensions.columns` (not to the widest content line); derive
        `inner_width = max(widest content line, model.dimensions.columns)`.
        (Req 29.6a)
  - [ ] 2G.1.3 Failing test + change: the always-present empty SCRATCH row is a
        PROVIDER-OWNED row carried in `model.tables[*].rows` (a full row of empty
        cells), NOT appended inside the text renderer, so both the plain-text grid
        AND the HTML `<td>` flavor carry it. (Req 29.7, 29.7a)
  - [ ] 2G.1.4 Failing test + change: a combo cell string `[value v]` (empty
        `[ v]`) and an empty editable cell `____` render verbatim and stay aligned
        in the fixed-width grid; the convention is the SAME string in plain-text
        and HTML flavors (the cell text is provider-owned, the renderer stays a
        dumb `|` grid -- no `Table` reshape). (Req 29.8, 29.9)
  - [ ] 2G.1.6 Failing test (ff-screen-model): the HTML flavor (`render/html.rs`)
        of the SAME grid model contains a `<td>` with `[value v]`, a `<td>` with
        `____`, and the trailing scratch-row `<tr>` of empty cells, proving the
        conventions + scratch row are IDENTICAL across flavors (provider-owned
        cell text, consumed by both the `|` grid and the `<td>` emitter with no
        flavor-specific code). (Req 29.7a, 29.8, 29.9)
  - [ ] 2G.1.5 Regression test: a box-only model with no `Table` is unchanged by
        the grid work. (Req 29.16)

- [ ] 2G.2 Shared full-window chrome helper in `ff-desktop` (Req 29.1, 29.2, 29.3, 29.4, 29.10, 29.11, 29.13)
  - [ ] 2G.2.1 Failing test (ff-desktop): a helper applied to a model yields a
        rendering whose FIRST in-frame body row is the window-title-bar line
        `# | FileForge Workbench ... _ [] X` and whose SECOND is the active
        Context's `[XXX]` tab-tag line (sourced from
        `BuiltinKind::default_title()`). The title bar + tab tag are the first two
        `body_lines` (NOT `header_lines`, which `render_plain` emits BELOW the
        body). (Req 29.1, 29.2)
  - [ ] 2G.2.2 Add the shared helper (e.g. `screen_chrome.rs` /
        `screen_providers.rs`): PREPEND the title-bar + tab-tag lines as the first
        two `body_lines` (ahead of any Context body); fold `self.command_text`
        into a TOP `body_line` `Command ===> <value><underscores>` (after the
        title-bar + tab-tag lines) with the underscore run at the POM/menu
        command-line field width, and clear `model.command_line`; set
        `model.status_bar.text` from live status (bottom info bar is a defined
        line even when status is empty); append the F-key line to `footer_lines`
        ONLY when `key_bar_visible` (reuse `screen_snapshot::key_bar_line`). Keep
        the title in the border (emitted once). Pure logical state, no egui.
        (Req 29.3, 29.4, 29.10, 29.11, 29.13)
  - [ ] 2G.2.3 Failing test: the command line renders NEAR THE TOP inside the
        frame (not outside/below), carrying its typed value + underscore input
        area. (Req 29.4)
  - [ ] 2G.2.4 Failing test: the info/status bar renders at the BOTTOM; the F-key
        line is present when `key_bar_visible` is true and ABSENT when false.
        (Req 29.10, 29.11)

- [ ] 2G.3 Wire every per-Context builder through the shared helper (Req 29.12, 29.15, 29.16)
  - [ ] 2G.3.1 Call the shared chrome helper from each `*_screen_model()` builder
        in `screen_providers.rs` and `screen_providers_config.rs` (Alias, Config,
        Theme, Menus, Keys, Kinds, Command Configurator, Search Results, Event
        Log, Macro Library, Plugin Manager, Help, SCRM viewer) so every Context
        gains the chrome. (Req 29.12)
  - [ ] 2G.3.2 Reconcile the POM/menu path: add ONLY the title-bar + tab-tag
        header lines to `menu_workspace_screen_model_full` /
        `menu_screen_chrome`, keeping its two-column body + top command line + PF
        footer; this RE-ADDS the title bar + `[POM]` tag that the 2026-11-01 Bug-2
        correction removed. (Req 29.15)
  - [ ] 2G.3.3 Alias provider populates its row cells with the `[value v]` combo
        convention for the CE column (and any combo column) and `____` for empty
        editable cells, so the grid mirrors the live Alias table. (Req 29.8, 29.9)
  - [ ] 2G.3.4 A field-only / no-table Context still gains the top/bottom chrome
        around its body (regression). (Req 29.16)

- [ ] 2G.4 GENERAL-scope proof: a markdown_sections Context AND a table Context (Req 29.12)
  - [ ] 2G.4.1 Full-shell egui_kittest (ff-desktop): over the live ALIAS Editor
        (a TABLE Context), a SNAPSHOT's box text contains the title-bar line, the
        `[ALIASES]` tag, a `|`-separated header, a combo `[value v]` cell, an
        empty `____` cell, the scratch row, the info bar, and the F-key line only
        when visible. (Req 29.1, 29.2, 29.5, 29.7, 29.8, 29.9, 29.10, 29.11)
  - [ ] 2G.4.2 Full-shell egui_kittest (ff-desktop): over the live CONFIG panel
        (a `markdown_sections` Context), a SNAPSHOT's box text contains the
        title-bar line, the `[CONFIG]` tag, the top `Command ===>` line, the info
        bar, and the F-key line only when visible -- proving the fidelity is
        GENERAL, not Alias-specific. (Req 29.12)
  - [ ] 2G.4.3 HTML-flavor assertion (ff-desktop): the HTML text flavor of the
        live ALIAS (table) Context snapshot contains a `<td>` with `[value v]`, a
        `<td>` with `____`, and the trailing scratch-row `<tr>`, proving the cell
        conventions + scratch row are identical in the HTML flavor (Req 29.7a,
        29.8, 29.9).

- [ ] 2G.5 Capture-path parity: the same faithful box on capture (Req 29.14)
  - [ ] 2G.5.1 Failing test (ff-desktop): for the same Context, the model from
        the CAPTURE seam (`active_screen_model()`) renders to the IDENTICAL box
        text as the SNAPSHOT seam (`active_screen_capture()` -> model), proving
        snapshot and capture share one renderer and no divergent capture path is
        introduced. (Req 29.14)
  - [ ] 2G.5.2 Test (ff-desktop): a CAPTURE SCREEN / auto-capture of a table
        Context records a capture whose rendered box contains the title bar, tab
        tag, `|`-separated grid, scratch row, and info bar. (Req 29.14)

- [ ] 2G.6 MANUAL + reconciliation (Req 29.3, 29.16)
  - [ ] 2G.6.1 Confirm the title appears exactly ONCE (border, not duplicated in
        body) across the covered Contexts. (Req 29.3)
  - [ ] 2G.6.2 Confirm the PNG picture flavor (Req 23.2) and the property/value
        table flavors (Req 28) are UNCHANGED by the box-layout work. (Req 29.16)
  - [ ] 2G.6.3 MANUAL (record in TCR): the on-screen SNAPSHOT of the Alias Editor
        visually matches the live framed window top-to-bottom; cannot be driven
        headlessly. (Req 29.1-29.11)

## Cross-cutting

- [ ] 10. TCR + verification
  - [ ] 10.1 Add/flip TCR rows for every Req 1-20 criterion as it is covered.
  - [ ] 10.2 Each wave ends with a clean full verify.ps1 (nextest) run; ai-review.log empty.
  - [ ] 10.3 Confirm ff-desktop line count / rebuild-trigger reduction is
        measured before/after (Req 19.4).
