# FFWB Command Interpreter -- End-to-End Explainer and Localisation Simplification Verdict

STATUS: Investigation report (READ-ONLY). No source or spec files were modified.
Plain ASCII only. All claims below cite real files/symbols; the localisation
alias/resolver files were read from the in-flight worktree
`.worktrees/crnr109-locale-alias` (the most current realisation of CR-NR-109),
and the dispatch/execution files from the main checkout. Where the two differ it
is called out explicitly.

---

## 0. Summary answer first (the corrected mental model)

Your mental model is right about the PARTS but wrong about the SHAPE of the
chain. The key corrections:

1. There is NO four-step runtime chain `surface -> ftl -> verb-edit -> root ->
   module`. The `.ftl` is NOT consulted on the execution path at ALL. Execution
   of a typed verb is short: the typed string hits ONE front door
   (`dispatch_command_string`), an optional single verb-token rewrite to the
   root, then dispatch to the thing that runs. For `EDIT` specifically that is
   TWO hops: front door -> the EDIT ladder arm, which calls the registered
   `file.open` command handler. That is it.

2. There are TWO separate mappings that run in OPPOSITE directions, and they use
   DIFFERENT machinery:
   - INPUT (surface typed by the user -> root): the `AliasTable` in
     `shell/environment.rs` (`canonical_verb`), driven by the resolver
     `shell/locale_resolver.rs` (`resolve_locale_verb` / `rewrite_verb_to_root`).
     This is a reverse lookup (BEWERK -> EDIT).
   - DISPLAY (root -> localised label for menus/help): `ff_i18n::t(key)` via the
     alias entry's `localisation_key` (`AliasEntry::display_surface`) and the
     menu key convention in `menu_workspace/i18n_keys.rs`. This is the forward
     direction (EDIT -> BEWERK).
   Fluent/`.ftl` is ONE-DIRECTIONAL (key -> text). It therefore CANNOT do the
   input direction by itself: it can turn a key into "BEWERK", but it cannot take
   a typed "BEWERK" and give you back "EDIT". That is precisely why the
   `AliasTable` exists and why it cannot be replaced by the `.ftl`.

3. "The root is strictly tied to its execution module" -- essentially TRUE in the
   project's terms, with one nuance: the root (canonical) verb is the stable
   dispatched/persisted identity, and it is bound to its executor by the dispatch
   classification (`resolve_target` -> a `CommandTarget` variant, or a ladder arm
   for the editor/`EDIT` family). It is not a literal 1:1 "verb string -> one pgm
   pointer" table; the binding is "root verb -> CommandTarget / ladder handler".

4. Your simplification is PARTLY right and worth doing on the DISPLAY side, but
   it does NOT shorten the execution chain (which is already short), and it
   CANNOT remove the alias table on the INPUT side. Detail and a concrete
   recommendation are in section 4. Importantly: there is currently NO shipped
   `verb-edit` key at all -- the thing you want to delete does not yet exist as
   data; only the convention/mechanism exists. So the "simplification" is really
   a decision about which convention to adopt BEFORE the keys are authored.

---

## 1. The execution path (typed command -> running code)

### 1.1 The single front door

Every command string -- the typed `Command ===>` line and the key/menu
fall-through -- funnels through one method:

- `crates/ff-desktop/src/shell/dispatch.rs` ::
  `WorkbenchShell::dispatch_command_string(raw)`

Its ordered steps (main checkout):

1. `=` handling: a leading `=` reinitialises the active tab to the POM and
   re-dispatches the remainder (CR-CH-052). One place, applied once.
2. `run_command_prelude(raw, upper)` -- history record, current-menu option key
   (Req 8.3 stage 1), the EXIT family (`EXIT`/`QUIT`/`X`/`LOGOFF`), POM / chained
   fastpaths. If it claims the string, dispatch returns here.
3. `ff_command::resolve_target(raw, resolver)` -- the single classifier (see
   1.3). On `Ok(target)` it calls `dispatch_command_target(&target)` and returns.
4. Otherwise `run_command_ladder(raw, upper)` -- the hand-written `if` arms,
   ending in the editor semantics engine. `EDIT` is handled HERE (see 1.4).

In the worktree (`.worktrees/crnr109-locale-alias/.../shell/dispatch.rs`) there
is exactly ONE added line between step 1 and step 2:

```rust
let raw = self.rewrite_verb_to_root(raw);   // CR-NR-109, localization Req 11.6
let raw = raw.as_str();
```

This is the ONLY localisation insertion on the input path. It rewrites the verb
TOKEN to its root and leaves the argument (and its case) untouched; at `en` / no
overlay it returns the string unchanged, so behaviour is byte-identical.

### 1.2 The verb/argument split

The verb token is split from its argument by `shell/helpers::split_verb_arg`
(verb matched case-insensitively, argument case preserved -- B062; asserted by
`verb_arg_split_preserves_argument_case` in `dispatch.rs` tests). Ladder arms use
`verb_arg(cmd, "EDIT")` to peel the verb and get the remainder.

### 1.3 resolve_target and CommandTarget

- `crates/ff-command/src/command_target.rs` :: `resolve_target<R: TargetResolver>`
  returns a `CommandTarget`, one of FIVE variants:
  `Menu`, `CustomWorkspace`, `Function`, `Macro`, `External`.
- Resolution order (first match wins): user command definition -> built-in
  workspace verb/fastpath -> registered Command_ID (-> `Function`) -> resolvable
  menu name (-> `Menu`) -> macro name (-> `Macro`) -> `Err`.
- The shell binds this to real lookups via `command_config::ShellTargetResolver`.
- `crates/ff-desktop/src/shell/target_dispatch.rs` ::
  `dispatch_command_target(&target)` is the router that turns a resolved target
  into an action:
  - `Function { command_id }` -> `self.handle_command(command_id)` (identical to
    typing the command).
  - `Menu { name }` -> `self.open_menu_by_name(name)`.
  - `CustomWorkspace { workspace_kind, params }` -> the matching `nav_to_kind` /
    `open_*` method (file_explorer, config, keys, theme_editor, ...).
  - `Macro` -> deferred status message (not yet runnable).
  - `External { program, args, .. }` -> `run_external_target(...)` via the
    `ff-shell` adapter.

So the generic mapping is: **root verb -> `resolve_target` classification -> one
`CommandTarget` variant -> one concrete shell method / registered handler.** That
IS the "verb is tied to an executor" binding, expressed through the framework's
single dispatch seam.

### 1.4 How EDIT specifically dispatches

`EDIT` does NOT go through `resolve_target`; it is a ladder arm:

- `crates/ff-desktop/src/shell/commands_ladder_a.rs` :: `try_commands_a`:

```rust
if let Some(rest) = verb_arg(cmd, "EDIT") {
    if rest.is_empty() { /* "EDIT requires a path" error */ }
    else {
        let mut p = CommandParams::new();
        p.insert("path", rest);
        let result = self.dispatch.execute_command("file.open", p);
        ...
    }
    return true;
}
```

The in-code comment explains WHY it is on the ladder and not a `Function` target:
the `Function` dispatch arm calls `handle_command(command_id)` with NO params, so
classifying `EDIT` as `Function { "file.open" }` would DROP the `path` argument.
`EDIT` therefore stays on the ladder so it can carry the path to the registered
`FileOpenHandler` via `dispatch.execute_command("file.open", { path })`.

So for `edit Alans.cobol.master(helloW)` the real runtime hops are:

```
typed string
  -> dispatch_command_string            (front door)
  [-> rewrite_verb_to_root: EDIT is identity at en, unchanged]   (worktree only)
  -> run_command_prelude                (declines)
  -> resolve_target                     (declines -- EDIT is not one of its verbs)
  -> run_command_ladder -> try_commands_a EDIT arm
  -> dispatch.execute_command("file.open", { path = "Alans.cobol.master(helloW)" })
  -> FileOpenHandler opens the dataset in an editor Context
```

That is TWO meaningful hops from the root verb to the executing code (ladder arm
-> `file.open` handler). It is NOT the long `ftl -> verb-edit -> root -> pgm`
chain you feared. The `.ftl` never appears on this path.

### 1.5 Where "EDIT" currently lives wrt localisation

Note: `EDIT` is an FFCMD/shell file-open verb. It is NOT in the FFEDIT
`AliasTable` seed (`AliasTable::ffedit_english` in `shell/environment.rs` seeds
editor-buffer verbs: LOCATE/FIND/CHANGE/EXCLUDE/X/SHOW/SORT/CAPS/... and SAVE).
So today `EDIT` has no alias row and no `verb-edit` key anywhere (see section 3).
Your `EDIT` example is the general model; the CR-NR-109 machinery is currently
proven on editor verbs like `FIND` (surface SOEK) in tests.

---

## 2. The localisation layer (two directions)

### 2.1 Direction A -- INPUT resolution (typed localised surface -> root)

This is the "someone typed BEWERK, what do I run?" direction. It is done by the
alias table, NOT the `.ftl`.

- `shell/environment.rs`:
  - `AliasEntry { surface, canonical, localisation_key: Option<String> }` -- one
    row. `canonical` is the root (dispatched/persisted identity).
  - `AliasTable::canonical_verb(surface) -> Option<&str>` -- case-insensitive
    reverse lookup from a typed surface to its root. This is the seam that turns
    `X -> EXCLUDE`, `INCLUDE -> SHOW`, and (with a locale overlay) `SOEK -> FIND`.
  - Locale surfaces are layered on with `apply_locale_overlay_keyed` (base
    English + per-locale overlay, Req 5.5), each overlay row carrying its optional
    `localisation_key`.
- `shell/locale_resolver.rs`:
  - `resolve_locale_verb(verb, env_name) -> LocaleResolveOutcome` applies the
    exact Req 11.6 order against the active environment's active-locale table: a
    localised surface resolves to its root (branch 1.1); a bare root resolves to
    itself via its synthesised identity row (branch 1.2.1); anything else is
    `Unknown` and left as typed (branch 1.2.2).
  - `rewrite_verb_to_root(raw)` is the thin glue the front door calls: split
    verb/arg, resolve the verb to its root, reassemble `root + arg`. Returns the
    string unchanged when the root equals the surface (en/identity) or on
    `Unknown`.

The root is what is then dispatched, recorded in history, and persisted (Req
11.9) -- one stable identity regardless of the surface typed.

### 2.2 Direction B -- DISPLAY (root -> localised label for menus/help)

This is the "I need to SHOW the verb for EDIT in Afrikaans" direction. It uses
`ff_i18n::t(key)`, NOT the alias reverse lookup.

- `shell/environment.rs` :: `AliasEntry::display_surface()`:

```rust
match &self.localisation_key {
    Some(key) => ff_i18n::t(key),   // forward: key -> localised text
    None => self.surface.clone(),   // raw surface (en default, unchanged)
}
```

- `AliasTable::display_surface_for_canonical(canonical)` -- find the row for a
  root and return its `display_surface()`. This is the display accessor the menu
  render path reads.
- `shell/locale_resolver.rs` :: `menu_option_command_surfaces()` computes the
  per-option displayed command column for the active Menu Workspace by looking up
  each option's ROOT via `display_surface_for_canonical`.
- `menu_workspace/i18n_keys.rs` :: `localised_option_command(option, source)`
  returns that surface if present, else the raw root (Req 11.8 fallback). The
  same file holds the menu title/description key convention
  (`menu-<slug>-title`, `menu-<slug>-opt-<command-slug>-desc`) -- note that the
  option `command` value is only an INPUT to a key, never itself localised.

### 2.3 The `ff-i18n` seam itself (why it is one-directional)

- `crates/ff-i18n/src/seam.rs` :: `t(key)` / `t_args(key, args)` -- resolve a key
  against the active catalogue.
- `crates/ff-i18n/src/catalogue.rs` :: `Catalogue::resolve_args` -- active bundle
  -> English Identity_Base -> the key itself (miss sentinel). Fluent bundles,
  built per locale from `i18n/<locale>/*.ftl`, with Unicode isolation disabled.

Fluent is a `key -> formatted text` function. There is no reverse index from a
rendered string back to its key, and two keys may render to the same text, so the
reverse is not even well-defined. This is the structural reason the INPUT
direction (2.1) MUST use the alias catalogue and cannot use the `.ftl`.

### 2.4 The crucial point: execution never touches the .ftl

On the execution path (section 1) the ONLY localisation touch is
`rewrite_verb_to_root`, which reads the `AliasTable` (direction A), not the
`.ftl`. `ff_i18n::t` is used for DISPLAY (direction B) and for status/error
message text -- never to decide what code runs. Your worry that a typed command
is "first examined against the ftl, translated to verb-edit, then to root" is the
inverse of how it works: the typed surface is resolved to the root via the alias
TABLE; the `.ftl` is only ever used afterwards to DISPLAY a root as a localised
label.

---

## 3. Evaluating your simplification

Your proposal: drop the `verb-edit` indirection and have the `.ftl` map the root
verb directly to the localised label, i.e. a Fluent key `EDIT = BEWERK` instead
of `verb-edit = BEWERK`.

### 3.1 First, a factual correction about what exists today

There is NO `verb-edit` key -- and in fact NO `verb-<root>` key at all -- in any
shipped `.ftl` under `crates/ff-i18n/i18n/`. A search for `verb-` across the
catalogues finds only:
- Comments like "verb-embedding message" / "verb-prefixed cluster" (describing
  messages that bake an English verb into prose), and
- `profile-locked = ... use { $verb } ...` (a non-translated `$verb` placeable).

The ONLY `verb-find = SOEK` occurrence is a TEST FIXTURE written at runtime
inside unit tests (`zz-test-verbs.ftl` in `environment.rs` /
`tests_common`). So:

- The MECHANISM exists: `AliasEntry.localisation_key: Option<String>` plus
  `display_surface()` calling `ff_i18n::t(key)`.
- The CONVENTION `verb-<root>` is described (the display story in Req 11, the
  `verb-find` test fixture), but no production verb key has been authored yet.

This matters: you are not removing an existing layer of shipped data; you are
choosing which key-naming convention to adopt before the alias/verb catalogues
(Req 11.5, the on-disk `i18n/<locale>/aliases/<environment>.toml` reader) are
populated. That makes the decision low-cost to get right now.

### 3.2 (i) Is the raw root verb a legal Fluent key?

Yes, syntactically. The Fluent grammar defines
`Identifier ::= [a-zA-Z] [a-zA-Z0-9_-]*`
(projectfluent/fluent `spec/fluent.ebnf`; content rephrased for licensing
compliance). `EDIT` starts with a letter and is all alphanumerics, so `EDIT` is a
valid Fluent message identifier and `EDIT = BEWERK` parses fine. Uppercase is
permitted. (Mozilla's own STYLE guidance prefers lowercase-kebab identifiers, but
that is a convention, not a parser rule -- and it explicitly allows exceptions for
identifiers generated at runtime from external sources, which is exactly this
case. Content rephrased for licensing compliance.)

So Fluent syntax does NOT force the `verb-` prefix.

### 3.3 (ii) Is `verb-<root>` an extra SEMANTIC hop, or just a mechanical key?

It is a mechanical, collision-safe key derivation, NOT a semantic hop. Evidence:

- `display_surface()` does a SINGLE `ff_i18n::t(key)` call. Whether that key is
  `verb-edit` or `EDIT`, it is one forward lookup. There is no intermediate
  "translate to verb-edit, then translate verb-edit to root" step anywhere in the
  code -- the alias ROW already holds the canonical root directly
  (`AliasEntry.canonical`), and the key is only used to fetch DISPLAY text.
- The existing menu keys (`menu-pom-title`, `menu-settings-opt-reset-bare-desc`)
  are derived by the same style of mechanical slug (`i18n_keys::slug`): lowercase,
  collapse non-alphanumerics to `-`. A `verb-<root>` key is the same idea applied
  to a verb.

So the `verb-edit` your mental model treats as a costly middle stage is really
just "the string we hand to `t()`". The runtime cost of your simplification vs
the convention is identical: one `t()` call either way.

What the prefix BUYS (the reasons a convention exists):
- Namespace/collision safety. All catalogue keys share one flat namespace per
  locale (every `.ftl` in a locale is merged into one bundle --
  `catalogue.rs::build_bundle`). A bare `EDIT` key risks colliding with some
  other author's `EDIT` message; `verb-edit` is self-describing and unlikely to
  clash. With a bare-root convention you would want a discipline that root verbs
  never collide with other message keys.
- Mechanical derivability. `verb-<slug(root)>` can be generated from the root
  with zero per-verb authoring, matching the existing `menu-*` pattern. A raw
  root key works too, but roots can contain characters a Fluent identifier
  forbids: a root like `SAVE AS` or `MENU.OPEN` is NOT a legal identifier (space,
  `.`), whereas `verb-save-as` / `verb-menu-open` (slugged) always is. So the
  prefix+slug convention is what guarantees a legal key for EVERY root; a
  "raw root as key" rule only works for single-token alphabetic roots.

### 3.4 (iii) Can the `.ftl` ever replace the alias table for INPUT?

No. This is the load-bearing constraint and it does not change under your
proposal:

- Fluent resolves `key -> text`. Making the key `EDIT` and the value `BEWERK`
  still only lets you go `EDIT -> "BEWERK"`. It gives you NO way to take a typed
  `BEWERK` and recover `EDIT`.
- The INPUT direction (what the user types) is a surface -> root REVERSE lookup,
  and it is inherently many-surfaces-to-one-root (BEWERK and REDIGEER both ->
  EDIT; X -> EXCLUDE). Only the `AliasTable` holds that mapping.
- Therefore the alias catalogue is REQUIRED for input regardless of how display
  keys are named. Your proposal only touches direction B (display); direction A
  (the alias table) is untouched and must remain.

### 3.5 Verdict and concrete recommendation

- Is the execution chain as long as you feared? No. It is two hops for `EDIT`
  (front door -> ladder arm -> `file.open`), with at most one extra verb-token
  rewrite that is a no-op at `en`. The `.ftl` is not on the execution path. No
  change is needed to "shorten" it because it is already short.

- Is the display-side `verb-edit` key "avoidable"? Technically yes -- Fluent would
  accept `EDIT = BEWERK` for single-token alphabetic roots. But "avoidable" is not
  the same as "better". What you would GAIN: a marginally more direct-looking key
  for simple roots, and one fewer naming convention to explain. What you would
  LOSE: (a) guaranteed legal keys for multi-token / punctuated roots (SAVE AS,
  MENU.OPEN) -- you would have to slug anyway, re-introducing a derivation; (b)
  namespace collision safety in the single merged bundle; (c) consistency with the
  already-shipped `menu-*` / `msg-*` key conventions. These are real and the
  prefix is cheap, so the recommendation is to KEEP a mechanically-derived,
  prefixed key convention.

- RECOMMENDED convention (a small refinement of what is there, not your removal):
  use `verb-<slug(root)>` as the display key, derived mechanically from the root
  exactly like `i18n_keys::slug` does for menus. Author it ONCE per root. This
  keeps: one forward `t()` call for display; the root as the single stable
  identity; the alias table as the sole INPUT mechanism; and legal, collision-safe
  keys for every possible root. The alias ROW already carries the root directly,
  so there is no "verb-edit -> root" semantic hop to remove -- that hop does not
  exist in the code.

- IMPORTANT framework note: do NOT pursue any variant of this that makes the
  `.ftl` drive INPUT resolution or that adds a second verb-dispatch path. That
  would reintroduce the A3 "divergent dispatch" bug class
  (`docs/PATTERNS-AND-LESSONS.md` section A3) and violate
  `framework-conformance.md` (single front door, single AliasTable seam, single
  `ff_i18n` seam). Keep input on `rewrite_verb_to_root` + `AliasTable`, display on
  `t(key)`.

In short: your instinct that there should be a clean "root tied to executor, and
a simple forward map for the label" is CORRECT and is essentially what the code
already does. The only misconception is the imagined long runtime chain through
the `.ftl`; that chain is not real. The `verb-<root>` key is a one-call display
convenience, not a semantic stage, and the alias table is irreducible for input.

---

## 4. Where it is documented (inventory + gap)

### 4.1 Existing docs and their accuracy

- `docs/specs/command-environments/EXPLAINER-command-environments.md` -- STRONG.
  Explains the single front door, the three dispatch stages (prelude ->
  `resolve_target` -> ladder), why the editor verbs are on the ladder, and the
  FFCMD/FFEDIT environment model. This is the closest thing to the execution-path
  half of what you want. It does NOT cover the two-direction localisation story
  (it predates / sits beside CR-NR-109).
- `docs/specs/command-environments/{requirements,design,environments-vision}.md`
  -- authoritative for the Command Environment model and the `AliasTable`
  (Req 6a). Accurate; environment-centric, not localisation-centric.
- `docs/specs/command-framework/*` -- authoritative for `resolve_target`, the five
  `CommandTarget` variants, the resolution order and shadowing rule (Req 8), and
  the single-front-door requirement (Req 2.1, 8.3, 8.4). Accurate and matches the
  code read above.
- `docs/specs/command-semantics/*` -- the editor semantics engine the ladder ends
  in (the terminal for LOCATE/FIND/CHANGE/... bodies). Relevant for "what the verb
  actually does" but not for dispatch wiring.
- `docs/specs/localization/{requirements,design}.md` -- Req 11 (CR-NR-109) is the
  authoritative spec for the two-direction localisation: Req 11.1/11.2 (alias row
  carries an optional localisation key; display via `t(key)`), Req 11.3/11.4
  (synthesised identity rows, overlay replaces without changing the root), Req
  11.5 (on-disk alias-catalogue reader), Req 11.6 (the exact input resolution
  order), Req 11.7/11.8 (menu-kind display), Req 11.9 (root is the persisted
  identity). Accurate and matches `environment.rs` / `locale_resolver.rs`.
  Req 1.x is AMENDED by CR-NR-109 to allow locale-aware DISPLAY surfaces while
  keeping the root source-language.
- `docs/PATTERNS-AND-LESSONS.md` section A3 (divergent dispatch, B080 class) --
  the governing lesson: a verb must behave identically whether typed, clicked, or
  keyed. Directly relevant to why the simplification must not add a second path.

### 4.2 Is there a single consolidated end-to-end explainer? No.

The execution-path story lives in the command-environments EXPLAINER; the
two-direction localisation story lives scattered across localization Req 11 and
the worktree code comments. No single document walks "typed verb -> dispatch ->
execution, PLUS the two-direction localisation" top to bottom. Given that this is
exactly the question you asked, one SHOULD be created (owner approval first, per
the requirements gate -- this report proposes, it does not author the spec).

### 4.3 Recommended outline for a consolidated explainer

Suggested location: `docs/specs/command-framework/EXPLAINER-command-interpreter.md`
(or a cross-cutting `docs/project-management/` explainer), STATUS: EXPLAINER,
reading aid over the authoritative specs. Proposed sections:

1. One-paragraph summary: one front door, one classifier, root = stable identity,
   `.ftl` is display-only.
2. The execution path: `dispatch_command_string` -> prelude -> `resolve_target`
   (the five `CommandTarget` variants) -> `dispatch_command_target` / ladder, with
   the worked `EDIT <dataset>` example and its two real hops.
3. Why some verbs are on the ladder (EDIT carries a path param; editor verbs run
   against the active buffer) -- cite the `commands_ladder_a.rs` EDIT comment.
4. The two-direction localisation model, with a diagram:
   - INPUT: surface -> root via `AliasTable::canonical_verb` /
     `resolve_locale_verb` / `rewrite_verb_to_root`.
   - DISPLAY: root -> label via `AliasEntry.display_surface` -> `ff_i18n::t(key)`.
   - A box stating "execution never consults the `.ftl`".
5. Why Fluent cannot do input (key -> text is one-directional; many surfaces map
   to one root) and why the alias catalogue is required.
6. The `verb-<root>` display-key convention: what it is (a mechanical slug), what
   it is not (a semantic hop), and why the prefix/slug is kept (legal + collision
   -safe keys for every root).
7. Framework guardrails: single dispatch seam, single AliasTable seam, single
   `ff_i18n` seam; cite framework-conformance.md and PATTERNS-AND-LESSONS A3.
8. Status note: CR-NR-109 is in-flight (worktree `crnr109-locale-alias`); no
   production `verb-*` keys authored yet; the mechanism is proven on FFEDIT verbs
   (FIND/SOEK) in tests.

---

## 5. Conclusions

1. The interpreter is already the clean shape you want: a single front door, a
   single classifier, and a root verb bound to its executor through one
   `CommandTarget` variant (or a ladder arm for the editor/`EDIT` family). The
   feared long runtime chain through the `.ftl` does not exist.
2. Localisation is genuinely two separate one-way mappings in opposite
   directions, using different machinery (alias table for input, `ff_i18n::t` for
   display). Execution uses only the alias-table side, and only as a no-op at `en`.
3. Your simplification is a DISPLAY-SIDE convention choice, not an
   execution-chain change. Dropping a prefixed display key is syntactically
   possible but not recommended; keep a mechanically-derived `verb-<root>` key for
   legal, collision-safe, convention-consistent keys. The alias table cannot be
   replaced by the `.ftl` for input, under any variant of the proposal.
4. No single end-to-end explainer exists today; recommend authoring one (gated,
   owner-approved) using the outline in 4.3, consolidating the command-environments
   EXPLAINER execution story with the CR-NR-109 two-direction localisation story.

### File/symbol index (for the owner's follow-up reading)

- Front door: `crates/ff-desktop/src/shell/dispatch.rs`
  (`dispatch_command_string`); worktree adds `rewrite_verb_to_root` call.
- Classifier: `crates/ff-command/src/command_target.rs`
  (`resolve_target`, `CommandTarget`, `TargetResolver`).
- Target router: `crates/ff-desktop/src/shell/target_dispatch.rs`
  (`dispatch_command_target`).
- EDIT handler: `crates/ff-desktop/src/shell/commands_ladder_a.rs`
  (`try_commands_a`, EDIT arm -> `file.open`).
- Alias table (INPUT) + display accessor: worktree
  `crates/ff-desktop/src/shell/environment.rs`
  (`AliasEntry`, `AliasTable::canonical_verb`, `display_surface`,
  `display_surface_for_canonical`, `apply_locale_overlay_keyed`).
- Locale resolver (INPUT): worktree
  `crates/ff-desktop/src/shell/locale_resolver.rs`
  (`resolve_locale_verb`, `rewrite_verb_to_root`, `menu_option_command_surfaces`).
- Menu display keys: worktree
  `crates/ff-desktop/src/menu_workspace/i18n_keys.rs`
  (`localised_option_command`, `menu_title_key`, `slug`).
- i18n seam + catalogue: `crates/ff-i18n/src/seam.rs` (`t`, `t_args`),
  `crates/ff-i18n/src/catalogue.rs` (`Catalogue::resolve_args`, `build_bundle`).
- Fluent identifier grammar: projectfluent/fluent `spec/fluent.ebnf`
  (`Identifier ::= [a-zA-Z] [a-zA-Z0-9_-]*`). Content rephrased for licensing
  compliance.
- Specs: command-environments EXPLAINER + requirements/design; command-framework
  Req 8/2.1; localization Req 11 (CR-NR-109); PATTERNS-AND-LESSONS A3.
