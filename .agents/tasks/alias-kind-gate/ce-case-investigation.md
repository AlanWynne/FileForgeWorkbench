# Alias Editor CE Case Investigation (READ-ONLY)

Repo: c:/workspace/apps/FileForgeWorkbench (main at fd35620). No files were
edited. All citations are from the main worktree (not `.worktrees/alias-kind-gate`).

## Summary answer

The owner is right on both counts, and the fix is small.

- **The canonical environment identifier is UPPERCASE** (`FFCMD` / `FFEDIT`),
  defined as the string constants `FFCMD_NAME = "FFCMD"` and
  `FFEDIT_NAME = "FFEDIT"` in `environment_registry.rs`. That uppercase form is
  what is persisted (the override/seed file is named `FFCMD.toml` / `FFEDIT.toml`),
  what the resolver matches against, and what validation checks roots against.
- **The lowercase `ffcmd` the owner saw in the snapshot is purely a DISPLAY
  artefact.** The Alias snapshot/table and the editor combo render the CE via
  `AliasEnvironment::label()`, which hard-codes the lowercase strings `"ffcmd"` /
  `"ffedit"`. It is never the stored or canonical form. The lowercase serde
  `rename_all = "lowercase"` on the enum is a dead code path for the environment,
  because the per-row `environment` field is `#[serde(skip)]` (the ENV is encoded
  in the file NAME, which is uppercase).
- **The command parser DOES uppercase** the typed verb (`dispatch.rs`), and all
  alias/environment matching is `eq_ignore_ascii_case`, so `ffcmd` and `FFCMD`
  resolve identically today. Case is not a correctness bug; it is a consistency
  bug (the display disagrees with the canonical/stored uppercase form).
- **CE and Command are BOTH already constrained dropdowns.** CE offers only
  `AliasEnvironment::ALL` (the two known environments); Command (the Root combo)
  offers only `EnvironmentRegistry::root_commands(env)` plus a blank. A user
  cannot type a free-text environment or an invalid root through the editor.

Recommendation headline: make the CE DISPLAY uppercase (`FFCMD` / `FFEDIT`) so it
matches the canonical and on-disk form. This is a **display-only fix** (change
what `label()` returns, or render `env_name()` instead of `label()` at the two
display sites). Storage is already canonical uppercase; the dropdown constraint
is already correct.

---

## Q1. CE storage + case

### (a) Canonical identifier in code: UPPERCASE string constants

`crates/ff-desktop/src/shell/environment_registry.rs`:

```rust
pub(super) const FFCMD_NAME:  &str = "FFCMD";
pub(super) const FFEDIT_NAME: &str = "FFEDIT";
pub(super) const FFNAV_NAME:  &str = "FFNAV";
pub(super) const MAINFRAME_NAME: &str = "MAINFRAME";
```

These are the stable names used for the active-environment derivation
(`active_name`), ADDRESS-by-name resolution (`resolve`, case-insensitive), and
root enumeration/validation (`root_commands`, `contains_root`). The canonical
case is UPPERCASE.

The per-row selector type is the enum `AliasEnvironment` in
`crates/ff-desktop/src/shell/user_aliases.rs`:

```rust
#[derive(... Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AliasEnvironment { #[default] Ffcmd, Ffedit }

impl AliasEnvironment {
    pub fn env_name(self) -> &'static str {        // -> "FFCMD" / "FFEDIT"
        match self {
            AliasEnvironment::Ffcmd => environment_registry::FFCMD_NAME,
            AliasEnvironment::Ffedit => environment_registry::FFEDIT_NAME,
        }
    }
    pub fn label(self) -> &'static str {           // -> "ffcmd" / "ffedit"
        match self { Ffcmd => "ffcmd", Ffedit => "ffedit" }
    }
    pub fn from_env_name(name: &str) -> Self {     // case-INSENSITIVE map back
        if name.eq_ignore_ascii_case(FFEDIT_NAME) { Ffedit } else { Ffcmd }
    }
}
```

So there are TWO string renderings of the same enum: `env_name()` = canonical
UPPERCASE (used for files/resolution/validation), and `label()` = lowercase
(used only for display). `from_env_name` is case-insensitive, so a file named
either case maps back cleanly.

### (b) Case written to disk: UPPERCASE `FFCMD.toml`

The ENV is NOT a field inside the file. In `UserAliasRowToml` the `environment`
field is explicitly skipped:

```rust
#[serde(skip)]
pub environment: AliasEnvironment,   // "the ENV is the file name"
```

The ENV is encoded in the FILE NAME, built from `env_name()` (uppercase):

```rust
// user_aliases.rs
pub fn user_alias_override_file(base: &Path, locale: &str, env_name: &str) -> PathBuf {
    base.join(locale).join(format!("{env_name}.toml"))
}
```

Both callers pass `env.env_name()` (uppercase):

- Save grouping: `shell/alias_editor.rs::write_override_rows` ->
  `user_alias_override_file(base, locale, env.env_name())`.
- Description reload: `shell/alias_editor.rs::user_alias_descriptions` -> same.
- First-run seed + reset-bare: `shell/config_bootstrap.rs` writes
  `<alias_base>/<locale>/FFCMD.toml`; `shell/reset_bare.rs` round-trips
  `aliases/en/FFCMD.toml`.
- Shipped seed: `af/aliases/FFEDIT.toml` (`alias_reader.rs` doc comment).

Confirmed by tests asserting the exact file name:
`user_aliases.rs::override_file_path_is_base_locale_env_toml`
(`af/FFCMD.toml`), `config_bootstrap.rs::first_run_missing_only_writes_all_absent_files`
and `alias_seed_contains_x_to_exit_row` (`en/FFCMD.toml`),
`tests_first_run.rs` (`aliases/en/FFCMD.toml written on first run`).

**On disk today the environment is UPPERCASE (`FFCMD.toml` / `FFEDIT.toml`).**
The `#[serde(rename_all = "lowercase")]` on the enum would only matter if the
enum were serialized as a field value, which it is not (`#[serde(skip)]`); so the
lowercase serde spelling never reaches disk for the alias store.

### (c) Where the lowercase DISPLAY comes from

The snapshot `CE` cell is built in
`crates/ff-desktop/src/shell/screen_providers_tables.rs::alias_table()`:

```rust
vec![
    r.act.clone(),
    r.environment.label().to_string(),   // <-- LOWERCASE "ffcmd" in the snapshot
    r.canonical.clone(),
    r.surface.clone(),
    r.command_template.clone(),
    r.description.clone(),
]
// headers: ["Act","CE","Command","Alias","Parameters","Description"]
```

The live editor combo shows the same lowercase text,
`crates/ff-desktop/src/alias_editor_panel/rows.rs::render_ce_combo`:

```rust
egui::ComboBox::from_id_salt(salt)
    .selected_text(env.label())                 // lowercase selected text
    .show_ui(ui, |ui| for choice in AliasEnvironment::ALL {
        ui.selectable_value(env, choice, choice.label());  // lowercase options
    });
```

The filter row also uses `label()` for its CE choices and its filter match
(`alias_editor_panel/host.rs`: `env.label()` for the shown text and
`row.environment.label().eq_ignore_ascii_case(v)` for the match).

**Why the display is lowercase while the owner expects uppercase:** the two
display sites call `label()` (hard-coded lowercase) rather than `env_name()`
(canonical uppercase). This is a deliberate-looking but inconsistent choice; the
enum doc comment even says "Serialises to the lowercase stable spelling", but
that serde spelling is inert here (field skipped). The result is a UI that shows
`ffcmd` while the file is `FFCMD.toml` and the parser/validator work in `FFCMD`.

---

## Q2. Does the parser uppercase? Yes; matching is case-insensitive

- The front-door dispatch uppercases the typed command verb:
  `crates/ff-desktop/src/shell/dispatch.rs`:

  ```rust
  let raw = self.rewrite_verb_to_root(raw);
  let raw = raw.as_str();
  let upper = raw.trim().to_uppercase();   // <-- parser uppercases
  // prelude(raw, &upper) ... then resolve_target(raw, &resolver) ... then ladder
  ```

- `ff_command::resolve_target` (`crates/ff-command/src/command_target.rs`) does
  not itself change case; it delegates to resolver stages. The shell resolver and
  the alias layer do the matching case-insensitively:
  - Alias surface -> canonical lookup is case-insensitive:
    `shell/environment.rs::canonical_verb` uses `e.surface.eq_ignore_ascii_case(s)`;
    `canonical_verbs_owned` / `display_surface_for_canonical` likewise.
  - Environment-name resolution is case-insensitive:
    `environment_registry.rs::resolve` and `contains_root` use
    `eq_ignore_ascii_case`; `AliasEnvironment::from_env_name` too.
  - `alias_overlay.rs` test `canonical_verb("chercher") == Some("FIND")` confirms
    case-insensitive surface matching.

**So `ffcmd` and `FFCMD` resolve identically today** (and `FFCMD.toml` would load
even if a stray file were lowercase, because `from_env_name` is
case-insensitive). The stored/canonical form IS uppercased; the inconsistency is
only that one display path shows lowercase.

---

## Q3. CE and Command are constrained dropdowns (not free text)

Both cells are enumerated combos in
`crates/ff-desktop/src/alias_editor_panel/rows.rs`:

- **CE combo** (`render_ce_combo`): options are exactly
  `AliasEnvironment::ALL = [Ffcmd, Ffedit]`. The user can only ever pick one of
  the two known Command Environments. There is no free-text CE entry anywhere in
  the row render (`render_data_row` binds CE via `render_ce_combo` only).

- **Command / Root combo** (`render_root_combo`): options are
  `EnvironmentRegistry::root_commands(env.env_name())` plus a leading blank
  (`"--"`). So Command is ALSO a constrained dropdown of the valid canonical root
  commands for the currently-selected CE. Changing CE re-validates the root
  (`revalidate_root_for_env`): a root not valid in the new environment is cleared.

Constraint is enforced end-to-end:

- By construction the dropdown pair is always valid:
  `rows.rs` test `dropdown_pair_satisfies_contains_root` asserts every offered
  root satisfies `contains_root(env, root)`.
- Save-side validation independently rejects an invalid pair even if it arrived
  another way: `shell/alias_editor.rs::validate_user_alias_rows` calls
  `EnvironmentRegistry::contains_root(row.environment.env_name(), &canonical)` and
  also uppercases surface/canonical and enforces the no-shadow rule. Tests:
  `validate_rejects_root_not_valid_in_declared_environment`,
  `validate_uppercases_and_preserves_template`.

**Conclusion:** a user cannot enter an invalid or free-text environment, and
cannot enter a free-text / invalid root; the CE can only ever resolve to a valid
environment and the Command to a valid root of that environment. (Note: the two
dropdowns are v1-limited to FFCMD/FFEDIT per the Req 5.18 / design 9.10.5 comment
in `rows.rs`.)

---

## Q4. Recommendation (no code)

Canonical case: **UPPERCASE `FFCMD` / `FFEDIT`** is the canonical identifier
(it is the `*_NAME` constant, the on-disk file name, the resolver/validator key).
The CE should be DISPLAYED uppercase to match. Storage is already uppercase, so
nothing changes there.

Where normalisation should happen / what to change, classified:

1. **CE display in the live editor combo** --
   `alias_editor_panel/rows.rs::render_ce_combo` (`selected_text` and
   `selectable_value` both use `env.label()`).
   Classification: **display-only fix** (render `env.env_name()` instead of
   `label()`, or change `label()` to return the uppercase form).

2. **CE display in the Alias snapshot/table** (the cell the owner saw) --
   `shell/screen_providers_tables.rs::alias_table()` uses
   `r.environment.label()`.
   Classification: **display-only fix** (use `env_name()` here too).

3. **CE filter row display + match** --
   `alias_editor_panel/host.rs` uses `env.label()` for the shown filter text and
   `row.environment.label().eq_ignore_ascii_case(v)` for the filter match.
   Classification: **display-only fix** for the shown text. The match is
   `eq_ignore_ascii_case`, so it keeps working whether the display is upper or
   lower; still, change it in step with `label()` for consistency so an uppercase
   filter value and uppercase cell agree exactly.

4. **CE storage (file name + persisted form)** -- `user_alias_override_file`
   builds the name from `env_name()` (uppercase) and `environment` is
   `#[serde(skip)]`.
   Classification: **already-correct** (no change; disk is already `FFCMD.toml`).

5. **Parser / resolver / root validation** -- dispatch uppercases; all matching
   is `eq_ignore_ascii_case`.
   Classification: **already-correct**.

6. **CE and Command as constrained dropdowns** -- both are enumerated combos
   with save-side `contains_root` validation.
   Classification: **already-correct** (no dropdown-constraint fix needed). The
   only open question is intentional scope: the CE list is hard-limited to the
   two v1 environments (`AliasEnvironment::ALL`); if the Alias Kind should offer
   more environments later, that list (and the `AliasEnvironment` enum) is the
   single place to widen it -- that would be a feature change, not a bug fix.

Decide-one note on HOW to make the display uppercase: prefer having the two/three
display sites render `env.env_name()` (the single canonical source) rather than
changing `label()`'s returned strings, because `label()`'s lowercase value is
also used as the filter MATCH token in `host.rs`; routing display through
`env_name()` keeps one canonical uppercase string for both display and file name
and avoids a second place that defines the spelling. Equivalently, redefine
`label()` to return the canonical uppercase and update the enum doc comment
(which currently claims a lowercase "stable spelling" that is actually inert).
Either is a display-only change; no storage migration is required because files
are already uppercase and `from_env_name` reads either case.

### One-line disposition

- CE case: display-only fix (3 sites) to show `FFCMD`/`FFEDIT`; storage,
  resolution, and dropdown-constraint are already correct.
- Command cell: already a constrained dropdown of valid roots; already-correct.

---

## Evidence index (file : symbol)

- `shell/environment_registry.rs` : `FFCMD_NAME`/`FFEDIT_NAME` (canonical
  uppercase), `root_commands`, `contains_root`, `resolve` (case-insensitive).
- `shell/user_aliases.rs` : `AliasEnvironment` (`label()` lowercase,
  `env_name()` uppercase, `from_env_name` case-insensitive),
  `UserAliasRowToml.environment` (`#[serde(skip)]`),
  `user_alias_override_file` (file name from `env_name`), test
  `override_file_path_is_base_locale_env_toml` (`FFCMD.toml`).
- `shell/alias_editor.rs` : `write_override_rows`, `user_alias_descriptions`
  (both use `env.env_name()`), `validate_user_alias_rows` (uppercases + roots +
  `contains_root`).
- `alias_editor_panel/rows.rs` : `render_ce_combo` (CE options =
  `AliasEnvironment::ALL`, display `label()`), `render_root_combo` (Command
  options = `root_commands(env)` + blank), `revalidate_root_for_env`, test
  `dropdown_pair_satisfies_contains_root`.
- `alias_editor_panel/host.rs` : filter CE uses `env.label()` for display and
  `eq_ignore_ascii_case` for match.
- `shell/screen_providers_tables.rs` : `alias_table()` CE cell =
  `r.environment.label()` (the lowercase snapshot value the owner saw).
- `shell/dispatch.rs` : `to_uppercase()` of the typed verb before
  prelude/`resolve_target`/ladder.
- `ff-command/src/command_target.rs` : `resolve_target` (delegates; no case
  change itself).
- `shell/environment.rs` / `shell/alias_overlay.rs` : `canonical_verb` and
  overlay matching via `eq_ignore_ascii_case` (case-insensitive).
- `shell/config_bootstrap.rs`, `shell/reset_bare.rs`, `shell/tests_first_run.rs`
  : seed/reset write & read `aliases/<locale>/FFCMD.toml` (uppercase).

Content was rephrased for compliance with licensing restrictions (all citations
are from this repository's own source).
