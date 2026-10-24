# FFWB Command Interpreter -- End-to-End Explainer

STATUS: EXPLAINER. This document explains, in plain terms and top to bottom, how
a command typed in FileForgeWorkbench (FFWB) travels from the command line to the
code that runs it, and how command-verb localisation fits in WITHOUT lengthening
that path. It is a reading aid over the authoritative specs
(`command-framework/{requirements,design}.md`, `command-environments/*`,
`localization/{requirements,design}.md` Requirement 11 / CR-NR-109) and the
code it cites. It authors no behaviour; the specs remain the source of truth.
Plain ASCII only.

Source: consolidated from the read-only investigation
`.agents/tasks/command-interpreter-model/report.md` (owner-approved for
promotion). All symbol/file references were verified against the code; the
localisation alias/resolver files reflect the in-flight CR-NR-109 worktree.

---

## 1. The idea in one paragraph

Every command -- typed on the `Command ===>` line, clicked in a menu, or fired by
a key -- goes through ONE front door and ONE classifier. A verb has a stable
canonical (root) form; the root is the identity that gets dispatched, recorded in
history, and persisted. The root is bound to the code that runs it through the
framework's single dispatch seam. Localisation does NOT add a stage to this path:
the message catalogue (`.ftl` / Fluent) is used only to DISPLAY a root as a
localised label, never to decide what runs. The one localisation touch on the
INPUT path is a verb-token rewrite that turns a typed localised surface back into
its root before the unchanged front door dispatches it -- and that rewrite is a
no-op in English.

---

## 2. The execution path (typed command -> running code)

### 2.1 One front door

Every command string funnels through:

- `crates/ff-desktop/src/shell/dispatch.rs` ::
  `WorkbenchShell::dispatch_command_string(raw)`

Ordered steps:

1. `=` handling -- a leading `=` reinitialises the active tab to the POM and
   re-dispatches the remainder (CR-CH-052). One place, applied once.
2. (CR-NR-109, INPUT localisation) `rewrite_verb_to_root(raw)` -- rewrites the
   verb TOKEN to its root, leaving the argument and its case untouched. At English
   / no overlay it returns the string unchanged, so behaviour is byte-identical.
   This is the ONLY localisation insertion on the input path.
3. `run_command_prelude` -- history record, current-menu option key, the EXIT
   family (`EXIT`/`QUIT`/`X`/`LOGOFF`), POM / chained fastpaths. If it claims the
   string, dispatch returns here.
4. `ff_command::resolve_target` -- the single classifier (see 2.3). On a resolved
   target it calls `dispatch_command_target` and returns.
5. Otherwise `run_command_ladder` -- the hand-written `if` arms, ending in the
   editor semantics engine. `EDIT` is handled here (see 2.4).

### 2.2 The verb / argument split

`shell/helpers::split_verb_arg` separates the verb token (matched
case-insensitively) from its argument (case preserved). Ladder arms use
`verb_arg(cmd, "EDIT")` to peel the verb and keep the remainder.

### 2.3 resolve_target and CommandTarget

- `crates/ff-command/src/command_target.rs` :: `resolve_target` returns a
  `CommandTarget`, one of five variants: `Menu`, `CustomWorkspace`, `Function`,
  `Macro`, `External`.
- Resolution order (first match wins): user command definition -> built-in
  workspace verb/fastpath -> registered Command_ID (`Function`) -> resolvable menu
  name (`Menu`) -> macro name (`Macro`) -> error.
- `crates/ff-desktop/src/shell/target_dispatch.rs` ::
  `dispatch_command_target` routes each variant to one concrete action
  (`Function` -> `handle_command`; `Menu` -> `open_menu_by_name`;
  `CustomWorkspace` -> the matching `nav_to_kind` / `open_*`; `External` ->
  `run_external_target`; `Macro` -> deferred).

So the general binding is: root verb -> `resolve_target` classification -> one
`CommandTarget` variant -> one concrete handler. That IS "the verb is tied to its
executor", expressed through the single dispatch seam -- not a literal
one-verb-to-one-program pointer table.

### 2.4 Worked example: `edit Alans.cobol.master(helloW)`

`EDIT` is NOT a `resolve_target` verb; it is a ladder arm, deliberately, because
it must carry a path argument (the `Function` dispatch arm passes no params, which
would drop the path). The real runtime hops are:

```
typed string
  -> dispatch_command_string                 (front door)
  [-> rewrite_verb_to_root: EDIT is identity at en, unchanged]
  -> run_command_prelude                      (declines)
  -> resolve_target                           (declines; EDIT is not its verb)
  -> run_command_ladder -> try_commands_a EDIT arm
  -> dispatch.execute_command("file.open", { path = "Alans.cobol.master(helloW)" })
  -> FileOpenHandler opens the dataset in an editor Context
```

That is TWO meaningful hops from the root verb to the executing code (ladder arm
-> `file.open` handler), plus one no-op verb rewrite. It is NOT a long
`ftl -> verb-edit -> root -> program` chain. The `.ftl` never appears on this
path.

(Note: `EDIT` is an FFCMD/shell file-open verb; it is not seeded in the FFEDIT
editor alias table. The CR-NR-109 localisation machinery is currently proven on
editor verbs such as `FIND` in tests.)

---

## 3. The localisation model: two one-way maps, different machinery

Localisation is TWO separate mappings that run in OPPOSITE directions and use
DIFFERENT machinery. This is the single most important thing to understand.

### 3.1 Direction A -- INPUT resolution (typed surface -> root)

"The user typed BEWERK; what do I run?" Done by the ALIAS TABLE, not the `.ftl`.

- `shell/environment.rs`: `AliasEntry { surface, canonical, localisation_key }`;
  `AliasTable::canonical_verb(surface)` is a case-insensitive REVERSE lookup from
  a typed surface to its root (`X -> EXCLUDE`, and with a locale overlay
  `SOEK -> FIND`). Locale surfaces are layered with `apply_locale_overlay_keyed`
  (base English + per-locale overlay).
- `shell/locale_resolver.rs`: `resolve_locale_verb` applies the Req 11.6 order
  (localised surface -> root; bare root -> itself via its identity row; else
  Unknown). `rewrite_verb_to_root` is the thin glue the front door calls; it
  returns the string unchanged at English / identity / Unknown.

The root is then dispatched, recorded, and persisted (Req 11.9) -- one stable
identity regardless of the surface typed.

### 3.2 Direction B -- DISPLAY (root -> localised label)

"I need to SHOW the verb for this root in Afrikaans." Done by `ff_i18n::t(key)`.

- `shell/environment.rs` :: `AliasEntry::display_surface()` returns
  `ff_i18n::t(localisation_key)` when a key is present, else the raw surface.
- `AliasTable::display_surface_for_canonical`, and
  `locale_resolver::menu_option_command_surfaces`, compute the displayed command
  column for the active Menu Workspace by looking up each option's ROOT.
- `menu_workspace/i18n_keys.rs` :: `localised_option_command` returns that
  surface, else the raw root (Req 11.8 fallback). The option `command` value is
  only an INPUT to a key, never itself localised.

### 3.3 Why Fluent cannot do the INPUT direction

- `crates/ff-i18n/src/seam.rs` :: `t` / `t_args` resolve a key against the active
  catalogue; `catalogue.rs` falls back active bundle -> English Identity_Base ->
  the key itself.
- Fluent is a `key -> formatted text` function. There is no reverse index from a
  rendered string back to its key, and two keys may render to the same text, so
  the reverse is not even well-defined.

Therefore the INPUT direction MUST use the alias catalogue and CANNOT use the
`.ftl`. Execution touches only the alias-table side (Direction A), and only as a
no-op at English.

---

## 4. The `verb-<root>` display-key convention (and a common misconception)

A natural simplification to propose is: drop a `verb-edit` key and let the `.ftl`
map the root directly, `EDIT = BEWERK`. The facts:

- There is currently NO shipped `verb-<root>` key in any catalogue under
  `crates/ff-i18n/i18n/`; only the MECHANISM (`localisation_key` +
  `display_surface`) and the CONVENTION exist. So this is a choice of key-naming
  convention to make BEFORE the verb keys are authored, not the removal of an
  existing layer.
- `EDIT = BEWERK` is syntactically legal Fluent (identifiers are
  `[a-zA-Z][a-zA-Z0-9_-]*`; uppercase is allowed). So Fluent does not force the
  `verb-` prefix.
- `verb-<root>` is NOT an extra semantic hop. `display_surface()` is a SINGLE
  `t(key)` call; the alias row already holds the canonical root directly. There is
  no "translate to verb-edit, then verb-edit to root" step anywhere in the code.
  Runtime cost is one `t()` call either way.
- What the prefix+slug convention BUYS: (a) a LEGAL key for EVERY root, including
  multi-token or punctuated roots (`SAVE AS`, `MENU.OPEN`) that are not valid
  Fluent identifiers raw -- `verb-save-as` / `verb-menu-open` always are;
  (b) collision safety in the single merged per-locale bundle; (c) consistency
  with the shipped `menu-*` key convention (same mechanical `slug`).

RECOMMENDATION: keep a mechanically-derived `verb-<slug(root)>` display key. It is
cheap, always legal, collision-safe, and convention-consistent. The execution
chain is already short and does not change. And under NO variant may the `.ftl`
drive INPUT resolution or a second verb-dispatch path -- that would reintroduce
the "divergent dispatch" bug class (PATTERNS-AND-LESSONS section A3) and violate
`framework-conformance.md` (single front door, single AliasTable seam, single
`ff_i18n` seam).

---

## 5. Framework guardrails

- Single dispatch seam: `resolve_target` -> `dispatch_command_target`; the typed
  line, menu clicks, and keys all route through the one front door.
- Single INPUT seam: `rewrite_verb_to_root` + `AliasTable`.
- Single DISPLAY seam: `ff_i18n::t(key)`.
- Governing rules: `framework-conformance.md` (single command dispatch);
  `PATTERNS-AND-LESSONS.md` section A3 (a verb must behave identically whether
  typed, clicked, or keyed).

---

## 6. File / symbol index

- Front door: `crates/ff-desktop/src/shell/dispatch.rs`
  (`dispatch_command_string`, `rewrite_verb_to_root` call).
- Classifier: `crates/ff-command/src/command_target.rs`
  (`resolve_target`, `CommandTarget`, `TargetResolver`).
- Target router: `crates/ff-desktop/src/shell/target_dispatch.rs`
  (`dispatch_command_target`).
- EDIT handler: `crates/ff-desktop/src/shell/commands_ladder_a.rs`
  (`try_commands_a`, EDIT arm -> `file.open`).
- Alias table (INPUT) + display accessor:
  `crates/ff-desktop/src/shell/environment.rs`
  (`AliasEntry`, `canonical_verb`, `display_surface`,
  `display_surface_for_canonical`, `apply_locale_overlay_keyed`).
- Locale resolver (INPUT): `crates/ff-desktop/src/shell/locale_resolver.rs`
  (`resolve_locale_verb`, `rewrite_verb_to_root`, `menu_option_command_surfaces`).
- Menu display keys: `crates/ff-desktop/src/menu_workspace/i18n_keys.rs`
  (`localised_option_command`, `menu_title_key`, `slug`).
- i18n seam + catalogue: `crates/ff-i18n/src/seam.rs` (`t`, `t_args`),
  `crates/ff-i18n/src/catalogue.rs` (`resolve_args`, `build_bundle`).
- Authoritative specs: `command-framework/{requirements,design}.md` (Req 8 / 2.1);
  `command-environments/EXPLAINER-command-environments.md` + requirements/design;
  `localization/{requirements,design}.md` Requirement 11 (CR-NR-109).

---

## 7. Related documents

- `command-environments/EXPLAINER-command-environments.md` -- the Command
  Environment model (FFCMD / FFEDIT and the ladder), the execution-path companion
  to this doc.
- `localization/requirements.md` Requirement 11 -- the authoritative
  two-direction localisation spec (CR-NR-109).
- `docs/PATTERNS-AND-LESSONS.md` section A3 -- the divergent-dispatch lesson.
