# Menus Editor drop-down: why SCRM is missing (READ-ONLY findings)

## Summary answer

The Menus Editor's "Menu:" drop-down is populated from
`MenusEditorState.available`, which is rebuilt by
`WorkbenchShell::refresh_menus_editor_list` in
`crates/ff-desktop/src/shell/menus_editor.rs`. That function builds the list as:

- a **hardcoded pair** `["POM", "Settings"]`, PLUS
- a **disk scan** of `<menus_dir>/*.toml` for user menu files.

SCRM is missing because it falls into **neither bucket**: it is NOT in the
hardcoded pair (only POM and Settings are), and it is NOT on disk (the SCRM
built-in menu is code-only and, per framework mechanism 4 / CR-CH-021, is
deliberately never written to `menus/scrm.toml`). So the editor is not "looking
in the wrong place" for a saved SCRM file -- there is correctly no SCRM file to
find, and the editor's hardcoded built-in set simply omits SCRM.

This is a direct inconsistency with the command resolver, whose
`BUILTIN_MENU_NAMES` DOES include `"scrm"`. The resolver knows three built-in
menus; the editor offers only two.

The correct fix is an **editor-list change only** (no new framework mechanism):
make `refresh_menus_editor_list` seed the list from the full set of compiled
built-in menu names (POM, Settings, SCRM) unioned with the on-disk user
`*.toml` files, deduplicated. Editing a built-in in the editor already persists
as a user override file via the existing save path, which is allowed and keeps
the compiled default as the baseline.

---

## Evidence

### 1. Where the drop-down list comes from

**UI binding (iterates `state.available`):**
`crates/ff-desktop/src/menus_editor_panel/render.rs`, in `pub fn render`:

```rust
let combo = egui::ComboBox::from_id_salt("menus_editor_select")
    .selected_text(current)
    .show_ui(ui, |ui| {
        for name in state.available.clone() {
            if ui.selectable_label(..., &name).clicked() {
                action = MenusEditorAction::Select(name.clone());
            }
        }
    });
```

**State field:**
`crates/ff-desktop/src/menus_editor_panel/state.rs`:

```rust
/// Editable menu names: "POM", "Settings", plus every user menu file.
pub available: Vec<String>,
```

**Population (the authoritative source of the list):**
`crates/ff-desktop/src/shell/menus_editor.rs`, `fn refresh_menus_editor_list`:

```rust
/// Rebuild the editor's available-menus list: POM + Settings + every user
/// `menus/<name>.toml` on disk (deduplicated, POM/Settings first).
fn refresh_menus_editor_list(&mut self) {
    let mut names = vec!["POM".to_string(), "Settings".to_string()];
    let menus_dir = self.menus_dir();
    if let Ok(entries) = std::fs::read_dir(&menus_dir) {
        let mut user: Vec<String> = entries
            .flatten()
            .filter_map(|e| {
                // ... only *.toml, skipping stems pom/settings ...
            })
            .collect();
        user.sort();
        names.extend(user);
    }
    self.menus_editor_panel.available = names;
}
```

So the list source is: **hardcoded `["POM","Settings"]` + a `read_dir` scan of
`menus_dir` for user `*.toml` files** (excluding `pom`/`settings` stems to avoid
duplicating the built-ins). It is NOT derived from `BUILTIN_MENU_NAMES`.

`refresh_menus_editor_list` is called from `open_menus_editor` (on editor open)
and from `save_working_menu` (after a successful save), both in the same file.

### 2. Why SCRM specifically is absent

SCRM exists only as a compiled, code-only menu and is never materialised to
disk, so the disk scan never produces it, and the hardcoded seed omits it:

- SCRM built-in content is `DEFAULT_SCRM_MENU_TOML` and the parsed baseline
  `recovery_scrm_menu()` in `crates/ff-desktop/src/menu_workspace/defaults.rs`.
  Its doc comment states it is "CODE-ONLY, never written to disk (CR-CH-021)".
- Tests in the same file assert the file must never appear on disk:
  `!menus_dir.join("scrm.toml").exists()` -- "scrm.toml must NOT be
  materialised (SCRM menu is code-only, Req 22.8/25.5)".
- The editor's hardcoded seed in `refresh_menus_editor_list` is literally
  `vec!["POM".to_string(), "Settings".to_string()]` -- SCRM is not in it.

Net: the disk scan correctly yields no `scrm.toml` (by design), and the
hardcoded set has only POM and Settings, so SCRM never enters `available`. The
owner's framing is answerable precisely: SCRM "was not saved where the editor
looks" is true but EXPECTED (it must not be saved to disk); the actual bug is
that the editor's built-in seed does not include SCRM, so it is not looking at
the code-only built-in set the resolver uses.

### 3. Resolver-vs-editor inconsistency

Resolver (`crates/ff-desktop/src/command_config/mod.rs`, line 135):

```rust
const BUILTIN_MENU_NAMES: &[&str] = &["pom", "settings", "scrm"];
```

Editor (`crates/ff-desktop/src/shell/menus_editor.rs`,
`refresh_menus_editor_list`):

```rust
let mut names = vec!["POM".to_string(), "Settings".to_string()];
```

The resolver recognises three built-in menus (`pom`, `settings`, `scrm`); the
editor offers only two (`POM`, `Settings`). SCRM is the exact delta. The two
built-in-menu sets are maintained independently and have drifted -- CR-NR-114
added `scrm` to the resolver set but did not update the editor seed.

### 4. Recommended fix (framework-conformant)

**Landing point:** `WorkbenchShell::refresh_menus_editor_list` in
`crates/ff-desktop/src/shell/menus_editor.rs`.

**Change:** seed the list from the full compiled built-in menu set (POM,
Settings, SCRM) instead of the hardcoded pair, then union with the on-disk user
`*.toml` files, deduplicated (case-insensitively, as the existing skip-logic
already does for `pom`/`settings`; extend the skip to `scrm`). Concretely, seed
`names` with the three built-in display names and have the disk-scan filter also
drop a `scrm` stem so a future user `menus/scrm.toml` override does not produce
a duplicate entry.

To avoid re-introducing the same drift, source the built-in names from a single
shared constant rather than a second hardcoded literal. `BUILTIN_MENU_NAMES`
lives in `command_config/mod.rs` as lowercase resolver tokens; the editor needs
display-cased names (`POM`, `Settings`, SCRM). Either (a) expose the built-in
set as the single source of truth and map tokens to display names in the
editor, or (b) at minimum add `"SCRM"` to the editor seed with a comment tying
it to `BUILTIN_MENU_NAMES`. Option (a) is preferable because it makes the two
lists provably agree.

**Framework-conformance note:** this is PURELY an editor-list change. No new
dispatch path, navigation stack, focus mechanism, or persistence format is
introduced, so it is not a framework change under
`framework-conformance.md`. It does NOT write any built-in menu to disk:
listing a built-in name does not create a file. The code-only-built-in rule
(mechanism 4 / CR-CH-021) is respected -- the SCRM baseline stays compiled.

**How editing a built-in persists today (confirmed from the save path):** when
the editor saves, `save_working_menu` -> `write_menu_file`
(`crates/ff-desktop/src/shell/menus_editor.rs`) writes to
`<menus_dir>/<stem>.toml`, where `menu_file_stem` maps POM -> `pom.toml`,
Settings -> `settings.toml`, else a slug. That is, editing-and-saving a built-in
creates a **user override file** (e.g. `menus/scrm.toml`), which is explicitly
allowed: the override takes precedence while the compiled default remains the
baseline (per the defaults doc comments: "A user `menus/scrm.toml` OVERRIDES
it"). Merely LISTING SCRM in the drop-down writes nothing; a file appears only
if the user actually saves an edit, at which point an override file is the
intended, allowed outcome. So the fix is consistent with code-only built-ins.

### 5. Related defect found in passing (NOT fixed)

Even if SCRM were listed, the editor would **mis-load it**.
`load_menu_for_editor` in `crates/ff-desktop/src/shell/menus_editor.rs`:

```rust
match name.to_ascii_uppercase().as_str() {
    "SETTINGS" => crate::menu_workspace::defaults::recovery_settings_menu(),
    _ => crate::menu_workspace::defaults::recovery_pom_menu(),
}
```

The built-in fallback handles only `SETTINGS` explicitly; every other name
(including `SCRM`) falls through to `recovery_pom_menu()`. So selecting SCRM
(once listed) with no user `scrm.toml` present would load the POM baseline, not
the SCRM baseline. The correct fix for this related defect is to add an `"SCRM"
=> recovery_scrm_menu()` arm (both symbols exist in
`menu_workspace/defaults.rs`). This must be fixed together with the list fix for
SCRM editing to actually work; flagged here per instructions, not implemented.

---

## Conclusions

1. Drop-down source: `MenusEditorState.available`, rebuilt by
   `refresh_menus_editor_list` as hardcoded `["POM","Settings"]` + a disk scan
   of `menus_dir/*.toml`.
2. SCRM missing because: it is neither in the hardcoded pair nor on disk (it is
   code-only and must not be written to disk), so it never enters the list. The
   editor's built-in seed simply omits SCRM.
3. Inconsistency: resolver `BUILTIN_MENU_NAMES = ["pom","settings","scrm"]`
   vs editor seed `["POM","Settings"]`. SCRM is the delta.
4. Fix: in `refresh_menus_editor_list`, list the union of the compiled built-in
   menu names (POM, Settings, SCRM -- ideally from the single shared
   `BUILTIN_MENU_NAMES` source) and the on-disk user `*.toml` files,
   deduplicated. Editor-list-only change; no framework change; code-only
   built-ins preserved (editing persists as an allowed user override).
5. Related defect: `load_menu_for_editor` falls back to `recovery_pom_menu()`
   for any non-Settings built-in, so SCRM would load the POM baseline; add an
   `"SCRM" => recovery_scrm_menu()` arm alongside the list fix.
