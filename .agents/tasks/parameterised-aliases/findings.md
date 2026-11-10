# Findings: Parameterised Command Aliases (read-only investigation)

Investigation for the owner's change request (original message index 4-5): extend
a command alias from a pure single-verb-to-single-root rewrite into a mapping of a
verb to a ROOT command WITH PARAMETERS and POSITIONAL SUBSTITUTION. Worked
example: alias `ecob` expands to `EDIT ffwb.cobol.master(&1)`; typing `ecob helloW`
resolves to `EDIT ffwb.cobol.master(helloW)`, which then dispatches through the
EXISTING single front door as if typed directly.

ALL work below was read-only. No source file was changed and no spec doc was
edited. Every claim is grounded in the live source read during this investigation;
file paths and line references are to the current worktree.

Owner-confirmed scope from message index 5 (recorded here so the gate inherits it):
- Placeholder syntax: `&1 &2 ...` positional AND `&*` (all remaining args). (Q1)
- `ecob` with no argument behaves exactly as the fully resolved command would with
  an empty/missing parameter -- i.e. the alias expands and whatever `EDIT
  ffwb.cobol.master()` (or with a literal empty substitution) does is what happens.
  The alias layer does NOT add its own "missing arg" error. (Q2)
- Alias-of-alias CHAINS: owner leans against supporting them, asked for a
  recommendation. (Q3/Q4) -- see the "Recursion / chaining" note in section 2.
- Each alias entry needs: Root command, localisation key, Alias (e.g. `ECOB`),
  Command parameter string (e.g. `FFWB.COBOL.MASTER(&1)`). (Q5)
- Menu interaction: owner recalls "whatever is on the command line is appended to
  the command executed" -- see item 3 for the precise, verified answer. (Q6)

---

## Executive summary (read this first)

- The alias layer today is a pure VERB rewrite. One `AliasEntry` maps a `surface`
  verb to a `canonical` (root) verb plus an optional `localisation_key`. There is
  NO parameter/template field and NO positional substitution anywhere in the
  alias path. (Item 1)
- The verb rewrite happens in exactly ONE function, `rewrite_verb_to_root`
  (shell/locale_resolver.rs), called at BOTH command-string seams BEFORE
  `ff_command::resolve_target`: the typed front door (dispatch.rs:87) and the
  menu-option selection path (commands_menu.rs:186 via `activate_menu_option`).
  This is the natural and ONLY place a parameter-template expansion would slot in
  while staying before `resolve_target`. (Item 2)
- The owner's recalled "command-line residue is appended" behaviour EXISTS but
  ONLY on the FUNCTION-KEY path (`dispatch_key_command`, target_dispatch.rs:116),
  which merges `format!("{} {}", command, field)`. It does NOT happen on a
  menu-option CLICK (`activate_menu_option` dispatches `option.command` ALONE) nor
  on a typed Enter (the typed string is dispatched verbatim). So parameterised
  aliases triggered from a MENU option are NOT "free"; they would need the
  template substitution the same as the typed path. (Item 3)
- Argument splitting at the front door is a SINGLE whitespace split: the first
  whitespace-delimited token is the verb, the TRIMMED remainder is the argument
  string (case preserved). There is NO quote-aware tokenisation at this layer, so
  `&1..&N` / `&*` must be defined against this simple whitespace tokenisation of
  the alias INVOCATION's argument string. (Item 4)
- The single dispatch seam is `ff_command::resolve_target` fed by
  `ShellTargetResolver`, dispatched by `dispatch_command_target`
  (shell/target_dispatch.rs). The alias pre-pass must stay a STRING-to-STRING
  rewrite that runs before this seam, so dispatch remains one path. (Item 5)
- Coexistence with the parallel CR-NR-111 gate holds cleanly: CR-NR-111 stores
  the ROOT command on a menu option and derives the localised SURFACE at render
  time. A parameter template belongs to the ALIAS ENTRY (the new field), NOT to
  the menu option, so there is no double-expansion. (Item 6, below)

Next free change-request id: **CR-NR-112** (highest existing is CR-NR-111; highest
CR-CH is CR-CH-065 -- see section "CR ids and EARS numbers").

---

## Item 1: AliasTable / AliasEntry shape, canonical_verb, loading, and TOML schema

EVIDENCE (exact):

- `AliasEntry` -- `crates/ff-desktop/src/shell/environment.rs:51-58`:
  ```rust
  pub(crate) struct AliasEntry {
      pub(crate) surface: String,            // UPPERCASED typeable/displayed surface
      pub(crate) canonical: String,          // the root (canonical) verb -- dispatched/persisted identity
      pub(crate) localisation_key: Option<String>, // optional ff-i18n key for the displayed surface
  }
  ```
  There is NO parameter/template field. The entry is purely (surface verb ->
  root verb [+ display key]).

- `AliasTable` -- `environment.rs:94-100`: holds `entries: Vec<AliasEntry>`.
  Seeded by `AliasTable::ffedit_english()` (environment.rs ~110-175, the X ->
  EXCLUDE / INCLUDE -> SHOW aliases plus identity rows) and
  `AliasTable::identity_for(roots)` (environment.rs ~180-205, one identity row per
  root). Matching is case-insensitive.

- `AliasTable::canonical_verb` -- `environment.rs:202-210`:
  ```rust
  pub(crate) fn canonical_verb(&self, surface: &str) -> Option<&str> {
      let s = surface.trim();
      self.entries.iter()
          .find(|e| e.surface.eq_ignore_ascii_case(s))
          .map(|e| e.canonical.as_str())
  }
  ```
  Returns ONLY the canonical verb &str. No parameters are carried through.

- Per-locale LOADING -- `crates/ff-desktop/src/shell/alias_reader.rs`:
  - TOML deserialisation structs `AliasRowToml` (alias_reader.rs:44-52) and
    `AliasFileToml` (alias_reader.rs:55-60). `AliasRowToml` fields are exactly
    `surface`, `canonical`, `localisation_key` (serde default -> Option). NO
    parameter field.
  - `parse_alias_toml` (alias_reader.rs:68-85) uppercases surface+canonical,
    preserves the key.
  - `read_catalogue_from_dir` (alias_reader.rs:93-120) reads
    `<aliases_dir>/<env>.toml` per environment; a missing file is not an error; a
    malformed file logs a WARN and is skipped.
  - `LocaleAliasStore` (alias_reader.rs:168-...): `seed` /
    `seed_and_overlay_for_locale` / `apply_overlay_from_dir` / `table_for`.
    Reloaded on `ui.locale` change via `reload_active_alias_store_for_locale`
    (alias_reader.rs, process-global `ACTIVE_ALIAS_STORE` slot, mirroring the help
    registry).
  - The store resolves the shipped root via `i18n_bootstrap::resolve_i18n_root`
    and reads `<root>/<locale>/aliases/<env>.toml`.

- EXACT current TOML schema of an existing alias entry -- the ONLY shipped alias
  data file is `crates/ff-i18n/i18n/af/aliases/FFCMD.toml`:
  ```toml
  [[alias]]
  surface = "SOEK"
  canonical = "FIND"
  localisation_key = "verb-find"
  ```
  One `[[alias]]` array-of-tables row per surface form; `localisation_key`
  optional. This is the schema a new `parameters`/`command_template` field would
  extend (additive, `#[serde(default)]`).

NOTE: there is a SEPARATE, unrelated `aliases` concept in `ff-file-ops`
(`crates/ff-file-ops/src/commands.rs` `mod aliases`: NEW/OPEN/SAVE...) and in
`ff-encoding` and `ff-help`. Those are NOT the command-environment alias table and
are NOT implicated by this CR; the relevant type is the `AliasTable` above.

---

## Item 2: rewrite_verb_to_root -- what it rewrites, signature, and where a template would slot in

EVIDENCE (exact): `crates/ff-desktop/src/shell/locale_resolver.rs`.

- Signature (locale_resolver.rs ~105):
  ```rust
  pub(super) fn rewrite_verb_to_root(&self, raw: &str) -> String
  ```
  It is a STRING -> STRING rewrite on the whole command line.

- What it rewrites TODAY (locale_resolver.rs ~105-125): it splits `raw` into
  `(verb, arg)` via `helpers::split_verb_arg`, resolves the VERB to its root via
  `resolve_locale_verb` -> `AliasTable::canonical_verb`, and:
  - if the resolved root differs from the entered surface, returns `root` (when
    arg empty) or `format!("{root} {arg}")` (arg preserved, case preserved);
  - otherwise (root == surface, i.e. en/identity, or Unknown) returns `raw`
    unchanged.
  So today it rewrites the VERB TOKEN ONLY; the argument is passed through
  untouched and is NEVER merged into a template.

- `resolve_locale_verb` (locale_resolver.rs ~75-90) returns
  `LocaleResolveOutcome::Root(String)` or `Unknown(String)` -- again verb-only.

- CALL SITES (both BEFORE `resolve_target`), confirmed:
  1. Typed command-line front door -- `crates/ff-desktop/src/shell/dispatch.rs`,
     inside `dispatch_command_string`, at the line:
     ```rust
     let raw = self.rewrite_verb_to_root(raw);   // dispatch.rs ~line 87
     let raw = raw.as_str();
     ```
     This runs AFTER the `=` strip and BEFORE `run_command_prelude` and
     `ff_command::resolve_target(raw, &resolver)` (dispatch.rs, a few lines
     below). Confirmed sequence in the read of dispatch.rs:54-120.
  2. Menu-option selection -- `crates/ff-desktop/src/shell/commands_menu.rs`,
     inside `activate_menu_option`:
     ```rust
     let root_command = self.rewrite_verb_to_root(command);   // commands_menu.rs ~line 186
     let command = root_command.as_str();
     ```
     Runs before `resolve_and_dispatch_command(command)` ->
     (fall through) `handle_command(command)`. Confirmed in the read of
     commands_menu.rs:166-205.

WHERE PARAMETER-TEMPLATE EXPANSION WOULD SLOT IN (design-relevant, not prescribed
here): `rewrite_verb_to_root` is the single pre-pass. Extending it (or a sibling
pre-pass invoked at the same two sites) to: look up the alias entry; and when the
entry carries a parameter/command template, split the invocation's argument string
into positional tokens and substitute `&1..&N` / `&*` into the template, producing
the fully expanded ROOT command string -- keeps the expansion STRING->STRING and
BEFORE `resolve_target`, so the downstream single front door is unchanged. The
output of the expansion is then what the prelude / `resolve_target` / ladder see,
exactly as a typed-out command. This is the framework-conformant seam (it is the
same seam CR-NR-109/111 use; no new dispatcher).

CAVEAT the design must address: today `rewrite_verb_to_root` only rewrites when the
verb maps to a DIFFERENT root AND preserves the original `arg`. A template alias
inherently consumes the arg (substitutes it into `&1..`), so the "arg preserved"
branch logic must be replaced for template entries (template entries own the whole
reassembly). Non-template entries must keep the current verb-only behaviour
byte-identical (en/identity neutrality, localization Req 10.2).

---

## Item 3: Is command-line residue appended on menu-option / typed-command execution?

VERIFIED ANSWER: residue-append is NOT universal. It exists on ONE path only.

- TYPED Enter -- `crates/ff-desktop/src/shell/commands_line.rs:26-33`
  (`run_command_line`): dispatches the trimmed field text VERBATIM via
  `dispatch_command_string(&original)`. No residue concept -- the field IS the
  command. (So for the typed `ecob helloW` case, the whole string is what the
  alias pre-pass sees; this is the owner's primary worked example and it is
  handled entirely by the pre-pass.)

- FUNCTION-KEY path -- `crates/ff-desktop/src/shell/target_dispatch.rs:116-128`
  (`dispatch_key_command`): THIS is the "residue appended" behaviour the owner
  recalls:
  ```rust
  let original = self.command_text.trim().to_string();
  ...
  if original.is_empty() {
      self.dispatch_bound_command(command);
  } else {
      let merged = format!("{} {}", command.trim(), original);   // <-- append
      self.dispatch_bound_command(&merged);
  }
  ```
  Example (tests_command.rs): type `1`, press F9=SWAP -> dispatches `SWAP 1`.
  So a KEY-bound command does append the command-field content as its argument.

- MENU-OPTION CLICK -- `crates/ff-desktop/src/shell/update.rs:160-163` consumes
  `pending_menu_option` and calls
  `self.activate_menu_option(option.target.as_ref(), &option.command)`.
  `activate_menu_option` (commands_menu.rs:166-205) dispatches `option.command`
  ALONE (after `rewrite_verb_to_root`). It does NOT read or append
  `self.command_text`. CONFIRMED: there is no `command_text` reference in the
  menu-option click or `activate_menu_option` code paths.

- TYPED Option_Key (e.g. typing an option mnemonic in a menu) -- goes through
  `try_current_menu_option` -> `activate_menu_option` with the option's own
  command; again no field-append.

IMPLICATION for the CR (decides Q6 / "free from menu"): The owner's recollection
is correct ONLY for the FUNCTION-KEY seam. For MENU OPTIONS the command is executed
with the option's own command string, NOT the command-line field, so a
parameterised alias invoked FROM A MENU OPTION is NOT automatically fed the
command-line argument. Three honest sub-conclusions for the design:
  1. The core worked example (`ecob helloW` TYPED) is fully served by the alias
     pre-pass, independent of any residue behaviour.
  2. An alias used as a MENU OPTION'S `command` whose template needs an argument
     would get NO argument unless the design explicitly routes the command-line
     field into the menu-option activation (a change to `activate_menu_option` /
     the click path). That is OUT of the "free, no extra effort" assumption the
     owner floated; flag it as a design decision.
  3. A KEY-bound alias with a template DOES receive the field as residue today
     (merged before the pre-pass), so a key-bound parameterised alias is close to
     free -- but note the merge happens BEFORE `rewrite_verb_to_root` runs, so the
     pre-pass sees `ECOB helloW` and must template-expand it like the typed path.

---

## Item 4: How command-line arguments are tokenised / split today

EVIDENCE (exact): `crates/ff-desktop/src/shell/helpers.rs`.

- `split_verb_arg` (helpers.rs:263-268) -- the extracting split used by
  `rewrite_verb_to_root`:
  ```rust
  pub(super) fn split_verb_arg(cmd: &str) -> (&str, &str) {
      let c = cmd.trim();
      let (head, rest) = c.split_once(char::is_whitespace).unwrap_or((c, ""));
      (head, rest.trim())
  }
  ```
- `verb_arg` (helpers.rs ~230-242) -- the sibling used by the ladder (e.g. EDIT):
  same rule, `split_once(char::is_whitespace)` then `trim`, matching the verb
  case-insensitively.

So at the front-door / alias layer there is exactly ONE split rule: FIRST
whitespace-delimited token = verb; TRIMMED remainder = the argument STRING (case
preserved, B062). There is NO quote-aware or multi-token argument splitting at this
layer -- the "argument" is a single opaque string. Quote handling exists only
DEEPER, per-command (e.g. `helpers::extract_arg` / `parse_two_args` for CHANGE's
`'old' 'new'`), NOT at the verb/alias boundary.

IMPLICATION for `&1..&N` / `&*`: the positional parameters must be defined against
a tokenisation of the ALIAS INVOCATION's argument string. Since the front door only
gives the alias layer one opaque argument string (everything after the alias verb),
the design must choose and SPECIFY the tokenisation for substitution -- the
simplest framework-consistent choice is whitespace splitting of that remainder
(`split_whitespace`), with `&*` = the whole remainder verbatim (or the
space-joined tokens). The worked example `ecob helloW` yields remainder `helloW`,
one token, so `&1` = `helloW`, matching the owner's example exactly. The design
must state whether quoted multi-word positional args are supported (they are NOT
supported by the current front-door split and would be a new capability).

---

## Item 5: The single command-dispatch seam

EVIDENCE (exact):

- Classifier: `ff_command::resolve_target(raw, &resolver)` called in
  `dispatch_command_string` (dispatch.rs, immediately after the
  `rewrite_verb_to_root` pre-pass and the prelude). The resolver is
  `crate::command_config::ShellTargetResolver::new(&self.command_store.definitions,
  &self.cmd_registry, self.menus_dir())` (dispatch.rs ~105-112).
- `ShellTargetResolver` impl -- `crates/ff-desktop/src/command_config/mod.rs:305-343`
  (`user_command_target`, `builtin_workspace_target` ->
  `builtin_workspace_target_for` at mod.rs:186, `menu_name_target`,
  `macro_name_target`).
- Dispatch of the resolved target: `self.dispatch_command_target(&target)`
  (dispatch.rs) -> `crates/ff-desktop/src/shell/target_dispatch.rs`
  (`dispatch_command_target`, ~line 160+), whose `Function` arm calls
  `handle_command(command_id)` with NO params, `Menu` arm calls
  `open_menu_by_name`, etc.

PRE-PASS SEAM CONFIRMED: the alias rewrite is a STRING rewrite strictly upstream of
`resolve_target`. Keeping parameter-template expansion in that same pre-pass means
`resolve_target` and `dispatch_command_target` are UNCHANGED, so the design stays
on one dispatch path (framework-conformance mechanism 1).

RELEVANT CONSTRAINT (grounds a key design choice): EDIT is NOT a `Function` target
today precisely because the Function arm cannot carry params. Evidence --
`crates/ff-desktop/src/shell/commands_ladder_a.rs:62-85`:
```rust
// EDIT <path>: DEFERRED from the Step 3 Function family (B080). ...
// The Function-target dispatch arm calls `handle_command(command_id)` with NO
// params, so classifying EDIT to `Function { "file.open" }` would drop the
// path and never reach the handler (Req 8.4 regression). EDIT therefore
// stays on the ladder; revisit if the Function arm learns to carry params.
if let Some(rest) = verb_arg(cmd, "EDIT") {
    if rest.is_empty() { ...edit-requires-path error... }
    else {
        let mut p = CommandParams::new();
        p.insert("path", rest);
        let result = self.dispatch.execute_command("file.open", p);
        ...
    }
}
```
So the worked-example target `EDIT ffwb.cobol.master(helloW)` is handled by the
LADDER's EDIT arm (which forwards `rest` as the `path` param to `file.open`). An
expanded alias string `EDIT ffwb.cobol.master(helloW)` therefore reaches this arm
naturally once the pre-pass produces it -- the alias expansion should produce the
expanded STRING and let the existing EDIT ladder arm consume it, NOT try to route
params through the alias layer. (The parentheses in `ffwb.cobol.master(helloW)`
are just part of the `rest` string; the front-door split only cuts at the first
whitespace, so the dataset-with-member token stays intact.)

---

## Item 6: Coexistence with CR-NR-111 (menu localisation) -- no double-expansion

READ: `.agents/tasks/menu-kind-localisation/findings.md`;
`docs/specs/localization/{design.md,requirements.md}`;
`docs/specs/command-framework/requirements.md` (Req 18);
`docs/specs/command-semantics/requirements.md` (Req 13).

- CR-NR-111 / CR-NR-109 model: a menu option stores the ROOT command
  (`MenuOption.command`, `menu_workspace/mod.rs:33-59`). The DISPLAYED command
  column is derived at render time from the active-locale `AliasTable` surface
  (`menu_option_command_surfaces` in locale_resolver.rs ->
  `display_surface_for_canonical` -> `i18n_keys::localised_option_command`). The
  displayed surface is DISPLAY-ONLY; selecting the option still carries the ROOT
  `option.command` through the `rewrite_verb_to_root` pre-pass (localization Req
  11.6/11.9). localization design.md Req 12 = "store the ROOT, derive the surface
  at render time".
- SEPARATION that must hold (CONFIRMED it does):
  - CR-NR-111's localised surface derivation is on the DISPLAY/alias-SURFACE axis
    (verb <-> localised verb). It does NOT introduce parameters.
  - A parameterised alias's template (`FFWB.COBOL.MASTER(&1)`) belongs to the
    ALIAS ENTRY (the new `AliasEntry`/TOML field), NOT to the `MenuOption`. A menu
    option continues to store a ROOT command (which MAY be an alias verb like
    `ECOB`); the template lives with the alias, so there is a single owner of the
    expansion.
  - No double-expansion: the render path only ever reads the DISPLAY surface
    (`display_surface_for_canonical`) and never expands a template; the EXECUTION
    path runs the pre-pass ONCE (per seam) which would perform template expansion
    once. The display derivation and the execution expansion are different axes
    and different code paths, so they cannot compound.
- A concrete guard for the design: a localised alias surface that is ALSO a
  parameter alias (e.g. an `af` surface for `ECOB`) must resolve surface -> root
  ALIAS-VERB and then expand the template. The design should state the order:
  resolve the typed surface to its canonical alias identity first (existing
  CR-NR-109 behaviour), then expand the template for that canonical. Because the
  template is keyed on the alias entry (one entry per canonical), this is a single
  deterministic lookup, not a chain.

RECURSION / CHAINING recommendation (owner Q3/Q4): recommend NOT supporting
alias-of-alias chains in this CR. The existing layer already runs the pre-pass
ONCE per seam and the identity-row design assumes a single surface->root hop; a
chain would require either a bounded fixpoint loop (pick a hard depth cap, e.g. 1,
and error beyond it) or cycle detection. The simplest, safe, framework-consistent
rule: expand EXACTLY ONE alias level (the template's own verb, e.g. `EDIT`, must be
a real root / ladder verb, NOT another alias). If a future CR wants chains, bound
the recursion with an explicit small depth limit and a cycle guard and make it a
separate gated decision. This matches the owner's own lean ("we should not support
this") while recording the enhancement path.

---

## CR ids and EARS numbers (re-verified against live files)

- Change-log `docs/status/change-log.md`:
  - Highest CR-NR id present: **CR-NR-111** (menu-kind localisation, GATE
    AUTHORING). CR-NR-110 (FIND/SEARCH split) also present. A regex check for
    CR-NR-112+ and CR-CH-066+ returned NO matches.
  - Highest CR-CH id present: **CR-CH-065** (no CR-CH-066+).
  - **NEXT FREE CR-NR id: CR-NR-112.**
- `docs/specs/command-framework/requirements.md`: highest EARS requirement is
  **Requirement 18** ("A localised menu-option verb resolves through the single
  classifier -- no new path (CR-NR-111)"). (So a new requirement here would be
  Requirement 19.)
- `docs/specs/command-semantics/requirements.md`: highest EARS requirement is
  **Requirement 13** ("Localised-surface command resolution is behaviour-
  preserving for routing (CR-NR-111)"). (So a new requirement here would be
  Requirement 14.) Note the file numbers requirements out of strict order (Req 8
  appears before Req 7); 13 is nonetheless the highest integer present.

---

## Alias definition / management UI (completion / configurator) -- relevance check

- `docs/specs/command-completion/` and `docs/specs/command-configurator/`: a grep
  for "alias" across both folders' markdown returned NO matches. Neither spec
  currently defines an alias DEFINITION or MANAGEMENT UI. So this CR does NOT
  implicate an existing alias-management surface; parameterised aliases are
  authored as DATA in `i18n/<locale>/aliases/<env>.toml` (the same place and
  mechanism as today's verb aliases). If the owner later wants a UI to define
  parameterised aliases, that would be a separate, additional gated deliverable
  (likely in command-configurator), and should NOT be assumed by this CR.

---

## Open design decisions this gate must settle (surfaced, not decided here)

1. Tokenisation for `&1..&N` / `&*` substitution: confirm whitespace splitting of
   the alias invocation's remainder (the only split the front door provides), and
   state explicitly whether quoted multi-word positional args are in scope (they
   are a NEW capability vs the current front-door split).
2. Empty-argument behaviour (owner Q2): define the literal substitution for a
   missing `&n` (empty string) so `ecob` alone expands to
   `EDIT ffwb.cobol.master()` and lets EDIT's existing handling react -- confirm
   that EDIT with an empty-parenthesised DSN behaves acceptably, or specify the
   substitution result.
3. Menu-option argument sourcing (item 3): decide whether a parameterised alias
   used as a menu option's `command` should receive the command-line field (NOT
   free today) or is simply unsupported-with-args from menus.
4. Chaining/recursion (owner Q3/Q4): adopt the single-level-expansion rule above,
   or explicitly defer chains to a later bounded CR.
5. TOML schema extension: add an additive, `#[serde(default)]` template field to
   `AliasRowToml` / `AliasEntry` (e.g. `command_template = "FFWB.COBOL.MASTER(&1)"`)
   so existing verb-only rows and the English/no-overlay neutrality are unchanged.
