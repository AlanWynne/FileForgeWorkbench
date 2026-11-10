# Alias Resolver Investigation -- findings

READ-ONLY investigation. No code changed. Repo: `c:/workspace/apps/FileForgeWorkbench`, branch `main`, tip `dbc65d0`.

## Summary answer (verdict)

- **(a) Current lookup data structure + algorithm:** a **LINEAR SCAN over a `Vec<AliasEntry>`**. Aliases live in `AliasTable { entries: Vec<AliasEntry> }` (`crates/ff-desktop/src/shell/environment.rs`). Every resolution is `self.entries.iter().find(|e| e.surface.eq_ignore_ascii_case(s))` -- O(n). There is NO map, NO sorted structure, NO binary search anywhere in the alias path.
- **(b) Match key + case + CE scoping:** the match key is the **`surface` string**, matched **CASE-INSENSITIVELY** (`eq_ignore_ascii_case`, surfaces also stored UPPERCASED). Resolution is **EXACT match** (no prefix/longest-match). It is **scoped to ONE active Command Environment (CE)** -- the resolver looks up a single `AliasTable` chosen by the active environment name (FFEDIT for an editor Context, FFCMD otherwise) and does **not** fall back to another CE.
- **(c) Does on-disk sort order affect lookup today?** **No -- not at all.** The on-disk TOML rows are loaded into the `Vec` and then matched by a FULL linear `iter().find`. Order within the Vec does not change which row is found (surfaces are unique per environment by validation) and changes only the position in an O(n) scan, which is immaterial at the expected scale. On-disk order is therefore **purely cosmetic to lookup** right now. Sorting the store would NOT enable a binary search until the resolver itself is changed to perform one.
- **(d) Recommended SORT KEY for the editor's save-time sort:** the owner's instinct ("by Alias/surface") is correct, with one refinement grounded in the resolver: sort by **`(environment, surface)` with a case-insensitive (uppercased) surface comparison**. The store is already split into one `<ENV>.toml` file per environment, so in practice this means **sort each environment's rows by uppercased `surface`**. Case-folding matters for ordering because matching is case-insensitive and surfaces are persisted uppercased.
- **(e) ALSO improve the resolver to a map / binary search?** **Not warranted now, and it would be a core/framework change.** The alias set is tiny (built-in FFEDIT verbs plus a handful of user overlays; realistically tens of rows, not thousands), the scan is O(n) over that tiny Vec once per dispatch, and `AliasTable` is a load-bearing framework type consumed widely. Changing its internal structure or lookup algorithm touches a core mechanism and, per `framework-conformance.md`, needs explicit owner confirmation. Recommendation: make the save-time sort **cosmetic + deterministic** (stable diffs, easy human scanning) and do NOT couple it to a resolver change. If a faster lookup is ever justified by scale, the sorted-by-surface store makes a later `binary_search` / `BTreeMap` switch cheap -- but that is a separate, owner-confirmed decision.

---

## Evidence

### 1. Alias resolution path (where/how a typed command is checked against aliases)

Front door -> verb rewrite -> single lookup:

- `crates/ff-desktop/src/shell/dispatch.rs:87` -- the command-line front door calls `let raw = self.rewrite_verb_to_root(raw);` BEFORE the unchanged prelude / `resolve_target` / ladder. This is the one seam where a typed command's verb is rewritten to its root; it comments itself as "NO second dispatch path".
- `crates/ff-desktop/src/shell/locale_resolver.rs` -- `WorkbenchShell::rewrite_verb_to_root` -> `resolve_locale_verb` (and `resolve_template_entry` for parameterised aliases). The actual lookup:

```rust
// locale_resolver.rs, resolve_locale_verb
if let Ok(store) = self.i18n.alias_store.lock() {
    if let Some(table) = store.table_for(env_name) {
        if let Some(root) = table.canonical_verb(trimmed) {
            return LocaleResolveOutcome::Root(root.to_string());
        }
    }
}
```

- The lookup itself, `crates/ff-desktop/src/shell/environment.rs`, `AliasTable::canonical_verb` and `AliasTable::resolve_alias_entry`:

```rust
pub(crate) fn canonical_verb(&self, surface: &str) -> Option<&str> {
    let s = surface.trim();
    self.entries
        .iter()
        .find(|e| e.surface.eq_ignore_ascii_case(s))   // <-- LINEAR SCAN, case-insensitive
        .map(|e| e.canonical.as_str())
}
```

- The data structure the aliases live in at resolution time, `environment.rs`:

```rust
pub(crate) struct AliasTable {
    /// `AliasEntry` rows. Kept as a small Vec; verb counts are tiny and this
    /// avoids a HashMap dependency for E0.
    entries: Vec<AliasEntry>,
}
```

The doc-comment itself states the design intent: "Kept as a small Vec; verb counts are tiny and this avoids a HashMap dependency."

Menu-option-click and keyboard seams route through the same `resolve_locale_verb` (see `locale_resolver.rs` module docs: called at BOTH the command-line front door and the menu-selection entry), so the lookup is single-path.

### 2. Match semantics

- **Key:** the `surface` string (uppercased at construction/overlay; see `environment.rs` `AliasEntry.surface` doc "UPPERCASED typeable/displayed surface form").
- **Case:** case-insensitive -- `eq_ignore_ascii_case` in both `canonical_verb` and `resolve_alias_entry`. Confirmed by test `alias_table_resolves_surface_to_canonical` (`environment.rs`): `t.canonical_verb("find") == Some("FIND")`.
- **Exact vs prefix:** EXACT match only. `iter().find` on full-string equality; no prefix or longest-match logic exists.
- **CE scoping:** scoped to ONE active environment. `resolve_locale_verb(verb, env_name)` reads exactly one table via `store.table_for(env_name)`. The active env is chosen by `active_environment_name()` (`locale_resolver.rs`) = `environments.active_name(kind.tag(), is_home)` -- FFEDIT for an editor Context, FFCMD otherwise. There is NO cross-CE fallback: test `resolution_is_scoped_to_the_active_environment` asserts an FFEDIT surface (`SOEK`) returns `Unknown` under FFCMD. So the correct sort key is scoped per environment -- `(CE, surface)`, not surface across all CEs.

### 3. Structure at rest vs at runtime

- **At rest:** per-environment TOML files. User override store: `<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`, rows `[[alias]]` with `surface`, `canonical`, optional `command_template` (`crates/ff-desktop/src/shell/user_aliases.rs`: `UserAliasRowToml`, `parse_user_alias_toml`, `serialise_user_aliases`). The `environment` is the FILE NAME, not a row field.
- **Load into runtime:** parsed rows are overlaid one-at-a-time onto the per-environment `AliasTable` Vec via the `apply_user_overlay_from_base` / `apply_locale_overlay_keyed` seam (`alias_reader.rs`, `alias_reader_user.rs`). The runtime structure is the SAME `Vec<AliasEntry>` that is linearly scanned.
- **Consequence:** because the runtime structure is a linearly-scanned Vec (not a map or sorted+binary-searched structure), on-disk/in-memory order affects only scan POSITION, never correctness. Since surfaces are unique per environment (validation forbids a surface mapping to two roots in one env), and the whole Vec is always scanned on a miss, **on-disk order is cosmetic to lookup today**.

### 4. Performance / scale

- **Scale:** small. The FFEDIT English seed is ~30 identity/alias rows (`AliasTable::ffedit_english`), FFCMD is identity rows for the registered roots, plus whatever handful of user overrides exist. Realistic total is tens of rows, not thousands.
- **Cost:** O(n) per command dispatch, n ~= tens. One `iter().find` under a `Mutex` lock per typed command (and a second for the template pre-pass). Negligible.
- **Is a faster structure warranted?** Not on performance grounds. A `BTreeMap<(CE, surface), ...>` or sorted-Vec + `binary_search` is a real algorithmic improvement only if n grows by orders of magnitude, which is not expected for user aliases.
- **Would sorting the store enable binary search?** Only AFTER the resolver is changed to binary-search. Sorting alone changes nothing because `canonical_verb` does a full linear scan. The two changes are independent; sorting is useful on its own merits (deterministic diffs, human-scannable editor table), and it is a prerequisite that makes a FUTURE binary-search switch cheap.
- **Framework flag:** `AliasTable` is a core type consumed by the resolver, the registry, the menu render, the locale overlay, and the editor. Replacing its internal `Vec` with a map or changing `canonical_verb` to a binary search alters a load-bearing mechanism. Per `framework-conformance.md` that is a FRAMEWORK CHANGE needing explicit owner confirmation. The save-time sort in the editor is NOT a framework change -- it only orders rows the editor already owns before serialising.

### 5. Alias row fields + description field

- Row fields confirmed (`user_aliases.rs` `UserAliasRowToml`): **`surface`**, **`canonical`**, **`command_template`** (optional, `skip_serializing_if = "String::is_empty"`), and an in-memory-only **`environment`** (`#[serde(skip)]`, derived from the file name, not persisted in the file).
- The runtime entry (`environment.rs` `AliasEntry`): `surface`, `canonical`, `command_template: Option<String>`, `is_overlay: bool` (resolution ignores `is_overlay`).
- **No `description` field exists** on either the persisted row or the runtime entry. This corroborates CR-CH-072 (the command-surface `localisation_key`/description indirection was REMOVED; the surface IS the display text). Test `row_parses_surface_canonical_template_without_l18n` explicitly tolerates-and-ignores a stale `localisation_key` and asserts only surface/canonical/template. A Group B "Alias Description" addition would therefore be a genuinely NEW field, not a reinstatement.

---

## Current save flow (for the editor change's baseline)

`crates/ff-desktop/src/shell/alias_editor.rs`:

- `save_user_aliases` clones `self.alias_editor_panel.rows`, validates (`validate_user_alias_rows`), then `write_override_rows` groups by environment and serialises -- **in panel-row order; there is NO sort today.**
- `write_override_rows` filters `rows` by `env` and writes each env's rows in their existing order to `<base>/<locale>/<ENV>.toml`.
- Validation (`validate_user_alias_rows`) already uppercases `surface` and `canonical`, which is why a sort key should compare the uppercased surface.

So the editor change is: insert a deterministic sort (by uppercased `surface`, already grouped per env file) into the save path (and optionally the edit-phase view), with NO resolver change.

---

## Conclusions and recommendations

1. **Lookup today:** linear scan over `Vec<AliasEntry>`, exact, case-insensitive, scoped to one active CE. (`environment.rs::AliasTable::canonical_verb`, `locale_resolver.rs::resolve_locale_verb`.)
2. **On-disk order is cosmetic to lookup.** Sorting does not currently speed up or change resolution.
3. **Persist the store sorted by `(environment, surface-uppercased)`** -- in practice, sort each `<ENV>.toml`'s rows by uppercased `surface`. This matches the resolver's match key (surface, case-insensitive) and the per-CE scoping, gives stable diffs, and makes the editor table orderly. This is an editor-only change, NOT a framework change.
4. **Do NOT change the resolver to a map/binary-search now.** It is unwarranted at the expected scale and is a framework/core change to `AliasTable` requiring owner confirmation per `framework-conformance.md`. The sorted store leaves that door open cheaply for later if scale ever justifies it.
5. **No `description` field exists today** (CR-CH-072); the Group B Alias Description is a new field to add deliberately, not a revert.
