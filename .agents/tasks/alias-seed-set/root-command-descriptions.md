# Root Command Descriptions and Alias Description Inheritance -- Findings

READ-ONLY investigation. Repo root `c:/workspace/apps/FileForgeWorkbench`, main at
660cf22. No files were edited.

## Summary answer

- **Root commands do NOT have a single, structured, per-command description.**
  Descriptions exist in THREE disconnected places, and none of them covers the
  FFCMD root verbs that the seed aliases map to (EXIT, SNAPSHOT, CAPTURE,
  RETRIEVE):
  1. `ff_command::CommandMetadata.description` -- only for the handful of
     commands REGISTERED in the `CommandRegistry` (today: `file.open`,
     `file.exit`, `menu.open`, `config.open`).
  2. `ff-file-ops::FileCommandMetadata.description` -- a static table for the
     `file.*` Command_Ids (keyed by `file.exit`, not by the verb `EXIT`).
  3. Context-help `cmd:<NAME>` topics (markdown `TITLE` + body) -- a de-facto
     human description source, but only for a listed subset of commands.
  The FFCMD/FFEDIT root verb SETS (`BUILTIN_WORKSPACE_VERB_ROOTS`,
  `BUILTIN_FFCMD_LADDER_VERB_ROOTS`) are **bare uppercase strings with no
  description channel at all**, and `EnvironmentRegistry::root_commands` returns
  `Vec<String>` (names only).

- **The alias row description is never auto-populated from the target command.**
  It is purely user-entered / seed-literal, sourced from the per-locale User
  Alias Override Store, and the resolver `AliasTable` carries no description.

- **The `X -> EXIT` seed row ships with an EMPTY description** (`description:
  String::new()` in `config_bootstrap.rs`). There is no "Exit the application"
  literal on the seed row; that string lives only on the UNRELATED `file.exit`
  registered command / `ff-file-ops` table.

- **There is a "registered commands must have a description" rule
  (command-framework Req 3.1), but it does NOT reach the root ladder verbs.**
  No rule requires every ROOT verb (EXIT, SNAPSHOT, ...) to have a description,
  and there is no single source of truth an alias could query for one.

Net: an alias CANNOT today inherit its canonical command's description, because
for the seed-alias roots (EXIT/SNAPSHOT/CAPTURE/RETRIEVE) no description exists
anywhere to inherit.

---

## 1. Root command descriptions -- where (if anywhere) are they stored?

### 1a. Registered commands: `CommandMetadata.description` (structured, but tiny set)

`CommandMetadata` carries a one-sentence `description`:

- `crates/ff-command/src/metadata.rs:12` -- `pub struct CommandMetadata { pub
  display_name, pub description, pub category, pub default_shortcut, pub icon }`.
  Doc comment on `description`: "One-sentence description of what the command
  does."

But only FOUR commands are actually registered in the shell's `CommandRegistry`:

- `crates/ff-desktop/src/shell/construct_commands.rs:register_builtin_commands`
  registers `file.open` ("Open a file from disk"), `file.exit` ("Exit the
  application"), `menu.open` ("Open or return to a menu"), `config.open`
  ("Browse all configuration keys..."). These are the only metadata-bearing
  registry entries the shell constructs.

Crucially these are keyed by DOTTED Command_Id (`file.exit`), not by the typeable
root verb `EXIT`. The seed alias `X -> EXIT` targets the VERB `EXIT`, which is a
ladder verb, not this registered `file.exit` id.

### 1b. `ff-file-ops` static metadata table (structured, file.* only)

- `crates/ff-file-ops/src/commands.rs:all_command_metadata()` returns a
  `Vec<FileCommandMetadata>` with `description` for each `file.*` id, e.g.
  `FILE_EXIT` -> description "Exit the application" (commands.rs ~line 152-156).
  Again keyed by `file.*` id, disconnected from the FFCMD root verb set.

### 1c. FFCMD / FFEDIT root verb sets -- NO descriptions

- `crates/ff-desktop/src/command_config/mod.rs:176` --
  `BUILTIN_WORKSPACE_VERB_ROOTS: &[&str]` = bare strings (`"CONFIG"`, `"KEYS"`,
  `"FILES"`, `"COMMANDS"`, `"LOG"`, `"MACROS"`, ...).
- `crates/ff-desktop/src/command_config/mod.rs:221` --
  `BUILTIN_FFCMD_LADDER_VERB_ROOTS: &[&str]` = bare strings including `"EXIT"`,
  `"SNAPSHOT"`, `"CAPTURE"`, `"RETRIEVE"`, `"EDIT"`, `"VIEW"`, `"BROWSE"`,
  `"SAVE"`, `"CANCEL"`, `"REFRESH"`, etc.
- `builtin_workspace_target_for` (mod.rs:274) classifies a verb into a
  `CommandTarget` but attaches no description.
- `crates/ff-desktop/src/shell/environment_registry.rs:234
  root_commands(env_name) -> Vec<String>` -- the canonical ROOT enumeration for
  an environment returns NAMES ONLY. There is no description-bearing variant.

So the exact roots the owner's seed aliases point at (SS->SNAPSHOT, CS->CAPTURE,
CT->CAPTURE?/CR->RETRIEVE, X->EXIT) have NO structured description.

### 1d. Command Configurator `CommandDefinition.description` (user-authored, optional)

- `crates/ff-desktop/src/command_config/mod.rs:40 CommandDefinition` has
  `#[serde(default)] pub description: Option<String>` ("Optional one-line
  description"). This is for USER-authored command definitions in
  `commands/commands.toml` (command-configurator Req 1.3). It is optional,
  user-entered, and does not cover built-in roots.

### 1e. Context-help `cmd:<NAME>` topics (de-facto description source, partial)

- `crates/ff-help/src/topic_key.rs` -- `TopicKey::command("CHANGE")` =>
  `"cmd:CHANGE"`; `TopicCategory::Command` for the `cmd:` namespace.
- `crates/ff-help/src/topic.rs:59 HelpTopic` has `title`, `body` (markdown),
  `syntax: Option<String>`, `aliases`, `see_also`. There is a human TITLE and a
  Description SECTION inside the markdown body, but NO dedicated one-line
  "description" field.
- Shipped `cmd:` topics (`help/commands/*.help.md`) exist for: SAVE, CANCEL, END,
  UNDO, REDO, FIND, RFIND, CHANGE, RCHANGE, HEX, EXCLUDE, SHOW, RESET, HELP, KEYS,
  POM, SETTINGS, FILES, UP, DOWN, TOP, BOTTOM, LOCATE.
- **No `cmd:EXIT`, `cmd:SNAPSHOT`, `cmd:CAPTURE`, or `cmd:RETRIEVE` topic
  exists** (grep of `help/**/*.md` for TOPIC lines and for those verbs found
  none; RETRIEVE appears only as prose in `index.help.md`). So even the help
  path cannot supply a description for the seed-alias roots today.

Conclusion for (1): there is NO single source of truth for a root command's
description. Descriptions are scattered across the registry (4 commands), the
`ff-file-ops` table (`file.*`), optional user `CommandDefinition`s, and partial
help topics -- and the FFCMD root verb set itself has none.

---

## 2. Alias description today -- auto-populated or literal?

The alias row `description` is NEVER derived from the target command. It is a
free-text field persisted in the per-locale user override store.

- `crates/ff-desktop/src/alias_editor_panel/state.rs:16 AliasRowEdit` --
  `pub description: String` ("CR-CH-075 ... human-readable description (`""` =
  none). Column: Description. Persists through the user override row.").
- `crates/ff-desktop/src/shell/alias_editor.rs:73 load_effective_alias_rows` --
  the comment states plainly: "The resolver AliasTable carries no description (it
  is a user-override field, not a resolution concern), so the description is
  sourced from the User_Locale_Override_Store files for the active locale, keyed
  by (environment, UPPERCASED surface)." The description is looked up from
  `user_alias_descriptions()`, defaulting to `""` when absent.
- `crates/ff-desktop/src/command_config/requirements` (spec) Req 7.3 confirms
  Description is an editable field that PERSISTS; it is user-entered, not derived.

### The seed row's description is EMPTY, not "Exit the application"

- `crates/ff-desktop/src/shell/config_bootstrap.rs:write_alias_seed` builds the
  seed row as:
  ```rust
  UserAliasRowToml {
      surface: "X", canonical: "EXIT",
      environment: AliasEnvironment::Ffcmd,
      command_template: String::new(),
      description: String::new(),   // <-- EMPTY
  }
  ```
  Both the first-run write (`BootstrapMode::FirstRunMissingOnly`) and Reset bare
  (`BootstrapMode::RebuildOverwrite`) call this one shared helper
  (`write_builtin_defaults`), so the Reset-bare rebuild writes the same
  description-less row. This is exactly the owner's observation: a reset-bare
  alias has no description.
- The "Exit the application" string the owner may be thinking of lives only on
  the UNRELATED registered `file.exit` command
  (`construct_commands.rs:43`) and the `ff-file-ops` `FILE_EXIT` table entry
  (`commands.rs:152-156`) -- NOT on the seed alias and NOT on the `EXIT` verb.
- Spec confirmation: command-configurator Requirement 8.1 defines the seed row
  as surface/canonical/environment ONLY; no description is specified. The current
  seed set is just `X -> EXIT` -- SS/CS/CT/CR are NOT yet in the code or the
  spec (Req 8 "Out of scope" currently reads "Turning any verb other than `X`
  into a seed alias"). The owner's request to add SS/CS/CT/CR is a pending,
  not-yet-gated change.

---

## 3. Can an alias inherit the command description?

**Not today -- there is no lookup to inherit from for the seed-alias roots.**

- To inherit, an alias would need a function `description_for_root(env, verb) ->
  Option<String>`. No such function exists. The nearest enumerations
  (`EnvironmentRegistry::root_commands`, `builtin_workspace_target_for`) return
  names/targets with no description.
- For a verb that happens to be a registered command (none of the seed roots
  are), you could read `CommandRegistry::get(id).metadata().description` -- but
  the alias canonical is a VERB (`EXIT`), not a dotted id (`file.exit`), and
  there is no verb->id map exposed for this.
- The only broad human-text source is context-help `cmd:<NAME>`. Using it as the
  default description couples alias seeding to:
  - **Help registry availability** -- the help topic registry must be loaded
    (`shell::help::reload_active_help_registry_for_locale`), which is a separate
    subsystem from alias bootstrap; the seed write runs during construction
    (`first_run_write_builtin_config`) and must not depend on help being ready.
  - **Locale** -- help topics are per-locale; a `cmd:` TITLE is "EXIT Command",
    not a one-line action phrase, and for the seed roots the topics do not exist
    at all. So help is NOT a reliable default source for EXIT/SNAPSHOT/CAPTURE/
    RETRIEVE right now.

Verdict: inheritance is blocked by the absence of any per-root description
source. It is viable only AFTER a description source for root verbs exists.

---

## 4. Is there a "commands must have a description" rule?

**Partially -- for REGISTERED commands only; not for root verbs.**

- **command-framework Requirement 3.1** (`docs/specs/command-framework/
  requirements.md`, AC "EACH registered command SHALL have associated metadata
  containing: a display name ..., a description (one-sentence summary ...), and a
  category ..."). This is the existing rule -- but it binds `CommandMetadata` on
  REGISTERED commands, of which there are four in the shell today.
- **file-operations Requirement 10.2** -- each registered file command SHALL
  include metadata including description.
- **command-semantics Requirement 7.1** -- `HELP` (no args) SHALL list all
  primary commands with "one-line descriptions", implying primary commands have
  one-line descriptions, but it does not mandate a storage field or location.
- **context-help Requirement 17.3** -- shipped help SHALL include `cmd:<NAME>`
  topics for a listed set of primary commands "including at minimum" FIND ...
  KEYS + POM/SETTINGS/FILES. EXIT/SNAPSHOT/CAPTURE/RETRIEVE are NOT in that
  minimum list.
- **No steering rule** (`.kiro/steering/*`) requires every root command to have
  a description. `command-framework.md` (framework-conformance) governs dispatch,
  not descriptions.

Where such a rule would naturally live: **command-framework** (extend Req 3 so
it covers canonical ROOT verbs, not only registry-registered ids), with
**command-semantics** Req 7.1 as the consumer, and **context-help** Req 17.3 as
the content channel. The alias Description column is specced in
**command-configurator** Req 7.

What it would take to give every root a description:
- Count of built-in roots: `BUILTIN_WORKSPACE_VERB_ROOTS` (15 entries) +
  `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (~32 entries) + the two code-only built-in
  menus' option roots unioned by `builtin_ffcmd_roots()`, plus the FFEDIT
  canonical verbs. On the order of ~50 FFCMD roots.
- Of those, a minority already have `cmd:` help topics (SAVE, CANCEL, END, HELP,
  KEYS, POM, SETTINGS, FILES, and the editor/navigation verbs) whose TITLE/first
  line could seed a description; the rest (EXIT, SNAPSHOT, CAPTURE, RETRIEVE,
  START, DOCK, SPLIT, ...) have none and would need authored text.

---

## 5. Recommendation (no code)

### (a) Seed aliases: ship explicit literal descriptions NOW vs inherit

Recommend **ship explicit literal descriptions on the seed rows now.** It is
quick, fully decoupled, and matches how the Description column already works
(literal, per-row, persisted). Inheritance is not available because no per-root
description source exists for these verbs, so "inherit" is currently a no-op that
would still leave the rows blank.

Suggested literal descriptions (for whenever SS/CS/CT/CR + X are seeded):
- `X -> EXIT`         : "Exit the application"
- `SS -> SNAPSHOT`    : "Capture a screen snapshot"
- `CS -> CAPTURE`     : "Capture the current screen collection"
- `CR -> CAPTURE REPLAY` or `RETRIEVE` : description to match the actual root
  (confirm the CR target with the owner -- the SCRM menu maps `CR` to
  `CAPTURE REPLAY`; see `menu_workspace/defaults.rs` SCRM option list).
  Owner note: messages 5-6 list the desired reset-bare set as SS, CS, CT, CR;
  the exact canonical root for CT/CR needs the owner's confirmation before
  seeding.

These literals live in the one shared `write_builtin_defaults` /
`write_alias_seed` helper (`config_bootstrap.rs`), so first-run and Reset bare
stay identical.

### (b) A "every root command has a description" rule + single source of truth

Recommend **yes, establish the rule, with the source of truth being a
command-definition-style field, not the help topic.** Reasoning:
- Help topics are per-locale markdown with a TITLE, not a one-line action
  phrase, and are an availability/locale dependency unsuitable as the canonical
  description store (section 3). They should REMAIN the long-form help, and MAY
  mirror the short description, but should not BE it.
- The natural home is a structured per-command description on the command model
  (extend `CommandMetadata` coverage to the root verbs, or add a small
  root-verb -> metadata table analogous to `ff-file-ops::all_command_metadata`).
  That gives `HELP` (command-semantics Req 7.1), command completion
  (command-completion Req 1.3, which already reads "display name or description
  from Command_Metadata"), and any future alias-inherit a single lookup.
- This is a non-trivial, gated change (command-framework Req 3 extension +
  authoring ~50 descriptions). It should be a separate change request from the
  seed-alias fix.

### (c) Smallest change that gives Reset-bare seed aliases a sensible description

**Set a literal `description` string on each seed `UserAliasRowToml` in
`config_bootstrap.rs::write_alias_seed`** (today it is `String::new()`). This is
the minimal, self-contained fix: it touches one helper, flows through both
first-run and Reset bare automatically, round-trips through the existing optional
`description` field (command-configurator Req 7.3), and needs no new lookup,
no help-registry coupling, and no framework change. It does require the Req 8
seed set and its "Out of scope" line to be updated first if SS/CS/CT/CR are being
added (a gated change request, per the owner's messages 5-6).

### Cost classification

| Option | Cost | Coupling / risk |
|--------|------|-----------------|
| (c) literal descriptions on seed rows | LOW (one helper, data-only) | none new; needs owner confirm of CT/CR canonical roots + Req 8 update |
| (a) as above, decoupled | LOW | same |
| help-topic inheritance as default | MEDIUM | help-registry availability + locale + missing topics for the seed roots; not viable now |
| (b) structured per-root description + rule | MEDIUM-HIGH | extend command-framework Req 3, author ~50 descriptions, new lookup; separate gated CR |

**Bottom line:** do (c)/(a) now -- ship literal descriptions on the seed rows --
and raise (b) as a separate gated change request if the project wants a single
source of truth for root-command descriptions.

---

## Evidence index (file : symbol)

- `crates/ff-command/src/metadata.rs:12` -- `CommandMetadata.description`.
- `crates/ff-desktop/src/shell/construct_commands.rs` -- the 4 registered
  commands with metadata (file.open/exit, menu.open, config.open).
- `crates/ff-file-ops/src/commands.rs:all_command_metadata` -- `file.*`
  description table (FILE_EXIT "Exit the application").
- `crates/ff-desktop/src/command_config/mod.rs:176,221,274` --
  `BUILTIN_WORKSPACE_VERB_ROOTS`, `BUILTIN_FFCMD_LADDER_VERB_ROOTS`,
  `builtin_workspace_target_for` (no descriptions).
- `crates/ff-desktop/src/shell/environment_registry.rs:234 root_commands` ->
  `Vec<String>` (names only).
- `crates/ff-desktop/src/command_config/mod.rs:40 CommandDefinition.description`
  (user-authored, optional).
- `crates/ff-help/src/topic.rs:59 HelpTopic` (title + body, no 1-line field);
  `help/commands/*.help.md` (cmd: topics; none for EXIT/SNAPSHOT/CAPTURE/RETRIEVE).
- `crates/ff-desktop/src/alias_editor_panel/state.rs:16 AliasRowEdit.description`.
- `crates/ff-desktop/src/shell/alias_editor.rs:73 load_effective_alias_rows`
  (description from user override store; resolver carries none).
- `crates/ff-desktop/src/shell/config_bootstrap.rs:write_alias_seed` --
  seed `X -> EXIT` with `description: String::new()`.
- Specs: command-framework Req 3.1 (registered-command metadata rule);
  file-operations Req 10.2; command-semantics Req 7.1; context-help Req 17.3;
  command-configurator Req 7.2/7.3 (Description column) and Req 8 (X -> EXIT seed,
  out-of-scope excludes other verbs).
