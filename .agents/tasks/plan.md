# Implementation Plan -- Localisation Phase 2 Slice 2 (CR-NR-103 L10N.4, tasks.md task 7, first THREE crates)

Extends the merged Slice 1 (FEAT-001/002/003 on `main` at `9b07352`). Converts
user-facing render literals in three dialog/panel crates to `ff_i18n::t("key")`
lookups, authoring the English keys as per-crate `.ftl` DATA files under
`crates/ff-i18n/i18n/en/`. Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice2`
on branch `feature/l10n-phase2-slice2` (base `main` @ `9b07352`).

## Design decisions (grounded in the Slice 1 code read)

1. **Lookup call shape -- direct `ff_i18n::t("key")`, returning `String`.**
   The seam (`crates/ff-i18n/src/seam.rs`) exposes free `t(key) -> String` and
   `t_args(key, args) -> String`; widgets already accept `String`/`&str`, so a
   render site swaps a string literal for `t("key")` with no API change
   (Req 3.4). Use `t_args` only where a value is interpolated (the dataset-alloc
   validation messages). Rationale: these crates' literals are static display
   strings, so the direct call is the minimal change.

2. **Why NOT the `t_or(key, fallback)` menu helper.** The FEAT-002 menu pattern
   (`crates/ff-desktop/src/menu_workspace/i18n_keys.rs`) uses `t_or` because the
   menu PROSE already lives in the parsed TOML model and must survive a catalogue
   miss. Here the render literal IS the only source, and we author the English
   value in the crate's `.ftl`, so `t("key")` is correct; on a genuine miss the
   seam returns the key itself (visible, diagnosable, Req 3.3). Keep the key the
   authored English identity.

3. **How new `.ftl` files are picked up -- no registration.** `build_bundle`
   (`crates/ff-i18n/src/catalogue.rs`) globs EVERY `*.ftl` directly inside
   `crates/ff-i18n/i18n/en/` and merges them into one bundle. Adding
   `catalog-dialog.ftl`, `dataset-alloc-dialog.ftl`, `theme-editor.ftl` under that
   directory is sufficient; no code change in `ff-i18n`. Because all `.ftl` share
   ONE flat key namespace, every key MUST carry the `<crate-slug>-` prefix so keys
   from different crates never collide (a Fluent duplicate-id across files in one
   bundle is a `BundleBuild` error).

4. **Key convention** (mirrors the FEAT-002 `slug()` rule, applied by hand here):
   `<crate-slug>-<context>-<slug>`, all lowercase, every run of non-alphanumeric
   characters collapsed to a single `-`, leading/trailing `-` trimmed. Crate
   slugs: `catalog-dialog`, `dataset-alloc-dialog`, `theme-editor`. Context is the
   sub-dialog / panel area (e.g. `new`, `edit`, `delete`, `btn`, `err`, `type`,
   `token`). Examples: `catalog-dialog-new-title`, `catalog-dialog-btn-cancel`,
   `dataset-alloc-dialog-err-lrecl-range`, `theme-editor-btn-save-as`.

5. **Cargo wiring.** None of the three target crates currently depend on
   `ff-i18n` (confirmed in each `Cargo.toml`). Each FEAT adds
   `ff-i18n = { path = "../ff-i18n" }` to that crate's `[dependencies]`.

6. **400-line / file-split rule (rust-standards.md).** A small
   `i18n_keys`-style helper module is NOT needed for these crates: the conversion
   is a literal-for-call swap that does not grow the non-test line count
   materially, and no key-derivation logic is required (keys are constant string
   literals at each site). Decision: do NOT add a helper module; keep keys inline
   as string-literal arguments. Re-check line counts after each FEAT (see
   verification) and only split a `_render.rs`/`_dialogs.rs` out if a file crosses
   400 NON-test lines -- not expected here (largest, `theme-editor/src/lib.rs`, is
   well under 400 non-test lines today).

## MUST NOT be localised (Req 4.1 user-facing only, 4.2 verbs stay English, 4.4 never match on a translated string)

- `command` / command-verb values and `command_id` strings (none rendered in
  these three crates, but keep the rule in mind).
- Config KEY names and dot-paths (e.g. `catalogs.default_mainframe_root`).
- `Topic_Key` identifiers and cross-reference keys.
- Enum variant names used as LOGIC keys: `CatalogType`, `Dsorg`, `Recfm`,
  `DialogOutcome`, `AllocOutcome`, `DeleteChoice`, `ThemeEditorAction` variants --
  these are match arms, never routed through the catalogue.
- Stable catalogue/registry data: the protected catalog name literal `"Home"`
  and the `CatalogType::Native` comparison in `delete_dialog::execute_delete` are
  LOGIC comparisons, NOT display -- leave them. (The user-facing SENTENCE that
  mentions the Home catalog IS localised; the comparison string is not.)
- egui `id_salt` / `Id::new(...)` identifiers (e.g. `"catalog_type_selector"`,
  `"theme_editor_select"`, `"theme_editor_hex"`): render identity, never shown.
- Internal-diagnostic text is out of scope for this slice; convert only
  user-facing strings. Do not derive any displayed text by MATCHING on a
  translated string (Req 4.4) -- every site resolves FROM a stable key.

---

- [ ] 1. FEAT-catalog-dialog -- convert `ff-catalog-dialog` render literals.
      Add `ff-i18n` dependency. Author `crates/ff-i18n/i18n/en/catalog-dialog.ftl`
      with keys for every user-facing literal, then replace each literal at its
      render site with `ff_i18n::t("catalog-dialog-...")`. Literals to convert
      (~40):
      - `new_dialog.rs`: window title `"New Catalog"`; the combo type labels
        reached via `catalog_type_label` (`"Mainframe"`, `"POSIX"`, `"Native"`)
        AND the three `selectable_value` labels; field labels `"Catalog Type:"`,
        `"Catalog Name:  "`, `"Description:   "`, `"Repository Path:"`,
        `"Default HLQ:    "`, `"Root Directory: "`, `"Mount Point:    "`,
        `"Root Path:      "`; checkboxes `"Auto-mount on startup"`,
        `"Read-Only"` (two sites), `"Create repository now"`; buttons `"OK"`,
        `"Cancel"`; validation messages in `validate()` (name-format, duplicate
        via `t_args` with `{ $name }`, description-length, repository-path
        required, root-directory required, root-path required) and the
        `"Failed to create repository: {e}"` message in `render` via `t_args`.
      - `edit_dialog.rs`: window title `"Edit Catalog"`; labels
        `"Catalog Name:  "`, `"Catalog Type:  "`, `"Repository Path:"`,
        `"Description:   "`; checkbox `"Auto-mount on startup"`, `"Read-Only"`;
        `"Default HLQ:   "`; buttons `"Save"`, `"Cancel"`; the `validate_edit`
        description-length message.
      - `delete_dialog.rs`: window title `"Delete Catalog"`; the confirm sentence
        `Delete catalog "{name}"? This will unmount it.` via `t_args`
        (`{ $name }`); `"Optionally delete all backing files."`; buttons
        `"Delete Catalog Only"`, `"Delete Catalog and Files"`, `"Cancel"`; the
        user-facing `execute_delete` messages (`"The Home catalog cannot be
        deleted..."`, and `"Failed to delete '{path}': {e}"` via `t_args`).
      - `lib.rs` `catalog_type_label`: keep the function (it feeds the combo), but
        have it return `ff_i18n::t(...)` keyed values (change signature to return
        `String`). NOTE: the three labels are DISPLAY text (Req 4.1), distinct
        from the `CatalogType` enum variants used in match arms, which stay.
      Shared button keys (`catalog-dialog-btn-ok`, `-btn-cancel`, `-btn-save`)
      may be reused across the three dialogs since the namespace is flat.
      Files: `crates/ff-catalog-dialog/Cargo.toml`,
      `crates/ff-catalog-dialog/src/new_dialog.rs`,
      `crates/ff-catalog-dialog/src/edit_dialog.rs`,
      `crates/ff-catalog-dialog/src/delete_dialog.rs`,
      `crates/ff-catalog-dialog/src/lib.rs`,
      `crates/ff-i18n/i18n/en/catalog-dialog.ftl` (new).
      Verify: from the worktree run `cargo check -p ff-catalog-dialog` and
      `cargo clippy -p ff-catalog-dialog -- -D warnings` and
      `cargo fmt -p ff-catalog-dialog -- --check`, reading the redirected logs;
      run the SCOPED existing tests `cargo test -p ff-catalog-dialog` and confirm
      they still pass (behaviour-neutral, Req 10.2). Confirm `catalog-dialog.ftl`
      has no duplicate keys and parses (the check build fails on a parse error).
      Do NOT run the full suite/gate/nextest.

- [ ] 2. FEAT-dataset-alloc-dialog -- convert `ff-dataset-alloc-dialog` literals.
      Add `ff-i18n` dependency. Author
      `crates/ff-i18n/i18n/en/dataset-alloc-dialog.ftl` and convert (~28):
      - `render.rs`: window title `"Allocate Dataset"`; field labels
        `"Dataset Name:      "`, `"Dataset Org (DSORG):"`, `"Record Format:     "`,
        `"LRECL:             "`, `"  Block Size:"`, `"Directory Blocks:  "`,
        `"GDG Limit (1-255): "`, `"Description:       "`; checkbox
        `"Scratch on roll-off"`; buttons `"Allocate"`, `"Cancel"`.
      - `form.rs` `Dsorg::label` / `Recfm::label`: these return the mainframe
        TECHNICAL tokens (`PS`/`PO`/`PDSE`/`GDG`, `FB`/`F`/`VB`/`V`/`U`). DECISION:
        treat them as DISPLAY labels for the selectors and route through
        `ff_i18n::t` (return `String`), authoring en keys equal to the current
        token text. They are display text beside a logic enum, NOT the enum
        variant itself (the match arms stay), so this respects Req 4.2/4.4. (A
        later French catalogue would keep these tokens English anyway; the key
        just resolves to the identity string.)
      - `validate.rs`: the user-facing error strings -- `"Dataset Name is
        required."`, the LRECL parse + range messages (range via `t_args`
        `{ $lrecl }`), the BLKSIZE parse + `>= LRECL` messages (via `t_args`),
        `"Directory Blocks must be a positive integer."`, the GDG-limit parse +
        range messages (via `t_args`), and in `validate_for_catalog` the
        duplicate-DSN message (via `t_args` `{ $dsn }`).
      Files: `crates/ff-dataset-alloc-dialog/Cargo.toml`,
      `crates/ff-dataset-alloc-dialog/src/render.rs`,
      `crates/ff-dataset-alloc-dialog/src/form.rs`,
      `crates/ff-dataset-alloc-dialog/src/validate.rs`,
      `crates/ff-i18n/i18n/en/dataset-alloc-dialog.ftl` (new).
      Verify: `cargo check -p ff-dataset-alloc-dialog`,
      `cargo clippy -p ff-dataset-alloc-dialog -- -D warnings`,
      `cargo fmt -p ff-dataset-alloc-dialog -- --check`, and the scoped
      `cargo test -p ff-dataset-alloc-dialog` (reading logs). The crate's unit
      tests assert validation ERROR presence (`.is_err()`), not message text, so
      routing through `t` keeps them green; confirm. Do NOT run gate/nextest.

- [ ] 3. FEAT-theme-editor -- convert `ff-theme-editor` literals.
      Add `ff-i18n` dependency. Author `crates/ff-i18n/i18n/en/theme-editor.ftl`
      and convert (~70):
      - `lib.rs` `render`: `"Theme:"`, the empty-selection placeholder
        `"(none)"`, buttons `"Set Active"`, `"Reset to built-in"`; the reset-
        confirm sentence `Reset '{name}' to its baseline and discard edits?` via
        `t_args` (`{ $name }`), buttons `"Confirm reset"`, `"Cancel"`;
        `"New name:"`, buttons `"Copy"`, `"Save As"`, `"Save"`, `"Export..."`,
        `"Import..."`; the `"invalid hex"` swatch message; the empty-state
        `"Select a theme to edit its colours."`; the advisory heading
        `"Contrast advisories (below AA):"`; and the per-advisory format in
        `recompute_advisories` (`{pair}: contrast {ratio}:1 (below AA 4.5:1)`)
        converted to `t_args` with `{ $pair }` and `{ $ratio }` (keep the numeric
        formatting in Rust, pass the formatted ratio as a string arg).
      - `editable_surface.rs`: the ~50 `colour_token!("<label>", ...)` display
        labels (`"Chrome: panel fill"`, `"Syntax: keyword"`, ... each `Group:
        field`). DECISION: the token `label` field is `&'static str` and is used
        BOTH as display text AND as a uniqueness/lookup key in the render loop and
        tests (e.g. `find(|t| t.label() == "Chrome: window fill")`). To avoid
        turning a display string into a match key (Req 4.4), DO NOT change the
        `&'static str` identity label. Instead add a SEPARATE display accessor:
        give `EditableToken` a stable `key: &'static str` (e.g.
        `theme-editor-token-chrome-panel-fill`) in the macro, keep `label` as the
        English identity, and render via `ff_i18n::t(token.key)` with the authored
        en value equal to the current label text. The existing `label()` stays the
        stable identity used by tests and the uniqueness assertion. This adds one
        field to the macro + a `key()` accessor; author one en key per token.
        (If, after this change, `editable_surface.rs` exceeds 400 non-test lines,
        split the `SURFACE` table into a `surface_table.rs` include -- re-check in
        verification; it is ~230 non-test lines now so a split is unlikely.)
      Files: `crates/ff-theme-editor/Cargo.toml`,
      `crates/ff-theme-editor/src/lib.rs`,
      `crates/ff-theme-editor/src/editable_surface.rs`,
      `crates/ff-i18n/i18n/en/theme-editor.ftl` (new).
      Verify: `cargo check -p ff-theme-editor`,
      `cargo clippy -p ff-theme-editor -- -D warnings`,
      `cargo fmt -p ff-theme-editor -- --check`, and scoped
      `cargo test -p ff-theme-editor` (reading logs). Confirm the
      `all_tokens_have_unique_non_empty_labels`,
      `chrome_token_round_trips_*`, and `surface_covers_*` tests still pass (they
      assert on `label()`, which we keep as the stable identity). Confirm
      `theme-editor.ftl` parses with no duplicate keys. After the change, check
      `editable_surface.rs` and `lib.rs` are each under 400 non-test lines. Do
      NOT run gate/nextest.

## Cross-FEAT notes for the integration step

- The three FEATs are INDEPENDENT (different crates, different `.ftl` files, no
  shared edits beyond each adding its own `.ftl` under `i18n/en/`). Execute in
  the listed order anyway (single worktree, sequential commits).
- No change to `ff-i18n` source is required or permitted in this slice; only new
  DATA `.ftl` files are added there.
- The owner runs the authoritative full `cargo gate --build` / nextest suite
  manually; agents stop at scoped compile + lint + fmt + scoped tests per the
  testing split rule.
- Framework conformance: this is a purely additive presentation-layer change
  (localization Req 10.1) through the existing `ff_i18n` seam -- no dispatch,
  navigation, focus, or persistence mechanism is touched.
