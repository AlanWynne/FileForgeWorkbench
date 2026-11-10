# Findings: Menu-Kind Localisation Redesign (read-only investigation)

Investigation for the owner's change request: make menu Kinds localise FULLY
(mnemonic + command alias + description follow the active locale), instead of
only the description re-localising, while keeping command routing unchanged
(a localised alias must resolve back to the root command through the existing
dispatch classifier `ff_command::resolve_target`).

ALL work below was read-only. No source file was changed.

## Executive summary (read this first)

Much of what the owner asks for ALREADY EXISTS, built under the in-progress
CR-NR-109 ("Locale-aware command resolution through the alias layer + menu-kind
options rendered via localised aliases"). The current state is:

- A menu option (`MenuOption`) has three parts: `key` (the 1-4 char mnemonic),
  `command` (the ROOT command string), and `description`.
- The DESCRIPTION already re-localises at render time via a catalogue key
  (`menu-<menu>-opt-<command>-desc`).
- The COMMAND COLUMN already re-localises at render time: the render path shows
  the active-locale ALIAS SURFACE of the option's ROOT command (falling back to
  the raw root when no alias exists).
- A localised alias ALREADY resolves back to the root command on BOTH seams
  (typed command line AND menu-option selection) via a pre-pass
  (`rewrite_verb_to_root` -> `AliasTable::canonical_verb`) that runs BEFORE
  `resolve_target`. Dispatch and persistence are unchanged (the ROOT is the
  dispatched/persisted identity).
- The ONE part that does NOT re-localise today is the `key` (mnemonic): it is
  rendered raw (`{:<4}` of `option.key`) and has no per-locale source.
- The reason a localised POM still shows English in practice is MISSING DATA
  (no `af` alias rows for the built-in menu-option roots), NOT a code gap -- this
  matches the earlier `.agents/tasks/af-menu-options-english/findings.md` verdict.

So the "command becomes the language alias while routing is unchanged" goal is
substantially already delivered by CR-NR-109; the redesign the owner now wants is
the DELTA on top: (a) localise the mnemonic too, and (b) decide the saved-menu
"command" field shape so a saved menu auto-converts to the active language
(first-prize option). See the "Crux" section.

---

## 1. How a menu option is modelled today

Type: `MenuOption` -- `crates/ff-desktop/src/menu_workspace/mod.rs:33-59`.

Fields (exact names):
- `key: String` -- "1-4 character key, stored uppercase" (the mnemonic/character).
- `command: String` -- "Primary command executed when this option is selected"
  (the ROOT command string).
- `description: String` -- "One-line description shown next to the key".
- `enabled: bool`, `group: Option<String>`, `show_in_menu_bar: bool`.
- `target: Option<ff_command::CommandTarget>` -- optional inline target that
  takes precedence over `command` during resolution (menu-workspace Req 10.6).

Container: `MenuFile` -- `mod.rs:64-90` (`title`, `options: Vec<MenuOption>`,
`show_calendar`, `group_separator`, `group_headers`).

Per-tab state: `MenuWorkspaceState` -- `mod.rs:100-245` (holds the backing
`file_path`, the parsed `menu`, load/error/reload bookkeeping). `menu_name_label()`
(`mod.rs:236-244`) returns the UPPERCASED file-stem (e.g. `POM`, `SETTINGS`) --
this is the menu name used to key localised descriptions.

Saved shape today (built-in defaults, CODE-ONLY -- never written to disk,
CR-CH-021): `crates/ff-desktop/src/menu_workspace/defaults.rs`.
- `DEFAULT_POM_TOML` (`defaults.rs:29-67`): options with
  `command = "Settings" | "Catalogs" | "Files" | "Help" | "X"` and an English
  `description`. `key = "0" | "1" | "2" | "3" | "X"`.
- `DEFAULT_SETTINGS_TOML` (`defaults.rs:77-118`): commands `Config`, `Theme`,
  `Menus`, `Keys`, `Kinds`, `Reset bare`.
- The on-disk TOML schema a user menu uses is therefore
  `key` + `command` (ROOT string, mixed-case English) + `description`
  + optional `group`/`show_in_menu_bar`/`[options.target]`.

KEY POINT for the crux: the saved "command" value today is the ROOT command
string, in English, locale-independent. There is NO localisation-key field on a
MenuOption and NO per-locale value stored on the option for the mnemonic.

## 2. Where menus render, and where key/command/mnemonic are emitted

- Shared renderer: `render_menu_workspace` --
  `crates/ff-desktop/src/menu_workspace/render.rs:66-73` (signature). It is the
  SINGLE renderer for all menus including the POM. It takes a
  `command_surfaces: &[String]` parameter -- the per-option DISPLAYED command
  text, positionally aligned with `menu.options`.
- Per-row prefix (key + command columns): `option_prefix_job` --
  `crates/ff-desktop/src/menu_workspace/render_layout.rs:114-137`. It emits:
  - the MNEMONIC: `job.append(&format!("{:<4}", option.key), ...)` -- the raw
    `option.key`, NOT localised.
  - the COMMAND: `job.append(&format!("{command_text:<cmd_width$}"), ...)` where
    `command_text` is the DISPLAYED surface passed in (the alias surface), NOT the
    raw `option.command`.
- The DESCRIPTION is drawn separately (two sites: the row label and the
  focused-option result) via `i18n_keys::localised_option_description`.
- The shell computes the surfaces and passes them in: `render_body_menu_workspace`
  (`crates/ff-desktop/src/shell/render_body_arms.rs`) calls
  `self.menu_option_command_surfaces()` then `render_menu_workspace(..., &surfaces)`.

So at the render layer: description localises (catalogue key), command localises
(alias surface), MNEMONIC (`key`) does NOT localise.

## 3. The existing alias table and the KEY QUESTION (resolve_target)

### Where per-locale aliases live
- On-disk DATA: `crates/ff-i18n/i18n/<locale>/aliases/<ENVIRONMENT>.toml`.
  Only ONE such file exists today: `crates/ff-i18n/i18n/af/aliases/FFCMD.toml`,
  containing a single row `surface = "SOEK", canonical = "FIND",
  localisation_key = "verb-find"`. There is NO `af/aliases/FFEDIT.toml`, and the
  `de`, `fr`, `en` locales have NO `aliases/` directory at all.
- In-memory model: `AliasEntry` and `AliasTable` --
  `crates/ff-desktop/src/shell/environment.rs`.
  - `AliasEntry { surface, canonical, localisation_key: Option<String> }`
    (`environment.rs`, approx. lines 40-55). `display_surface()` returns
    `ff_i18n::t(key)` when a key is present, else the raw `surface`.
  - `AliasTable::canonical_verb(surface) -> Option<&str>` resolves a surface form
    (case-insensitive) to its ROOT (canonical) verb. This is the
    surface-to-root lookup.
  - `AliasTable::display_surface_for_canonical(canonical) -> Option<String>` is
    the root-to-displayed-surface lookup the render path uses.
  - `AliasTable::identity_for(roots)` synthesises an identity row
    (`surface == canonical`, no key) for each root so a bare root always
    self-resolves. `apply_locale_overlay_keyed` overlays per-locale rows on top
    (keyed, collision-rejecting, retains the identity base).
- Store + reload: `LocaleAliasStore` seeds identity rows for the FFCMD roots and
  FFEDIT verbs and overlays the active locale's `aliases/*.toml`
  (`crates/ff-desktop/src/shell/alias_reader.rs`); re-seeded on `ui.locale`
  change via `reload_active_alias_store_for_locale`
  (`crates/ff-desktop/src/main.rs` `on_locale_reload`).

### THE KEY QUESTION: does resolve_target already resolve a per-locale alias to the root?

ANSWER: `ff_command::resolve_target` itself does NOT consult the alias table.
BUT the shell already resolves a localised alias to its root BEFORE calling
`resolve_target`, via a separate single-seam pre-pass. So end to end, the
localised-alias-to-root resolution IS ALREADY WIRED -- it just does not live
inside `resolve_target`.

Evidence:
- `resolve_target` (`crates/ff-command/src/command_target.rs:267-305`) resolves
  via the `TargetResolver` trait (user command id, built-in workspace verb,
  registered Command_ID, menu name, macro name). The `TargetResolver` trait
  (`command_target.rs:207-265`) has NO alias-surface lookup method. The shell's
  implementor `ShellTargetResolver`
  (`crates/ff-desktop/src/command_config/mod.rs:168-...`) likewise does not touch
  the alias table.
- The alias-to-root translation happens in the shell front door, BEFORE
  `resolve_target`:
  - Command line: `dispatch_command_string`
    (`crates/ff-desktop/src/shell/dispatch.rs:63-...`) calls
    `self.rewrite_verb_to_root(raw)` FIRST, then runs the prelude and
    `ff_command::resolve_target(raw, &resolver)` on the rewritten (ROOT) string.
  - Menu selection: `activate_menu_option`
    (`crates/ff-desktop/src/shell/commands_menu.rs:166-...`) also calls
    `self.rewrite_verb_to_root(command)` before dispatch, so the typed line and
    menu selection are provably one path.
  - `rewrite_verb_to_root` / `resolve_locale_verb`
    (`crates/ff-desktop/src/shell/locale_resolver.rs`) look the verb up in the
    ACTIVE environment's active-locale `AliasTable` via `canonical_verb`,
    returning the ROOT. At `en`/no overlay the table is identity-only, so
    verb == root and behaviour is byte-identical.

Conclusion for the design: the localised-alias -> root resolution for both the
typed and the selected seam is ALREADY in place and already does NOT change
routing (the ROOT is dispatched/persisted). This directly satisfies the owner's
"this should not affect how commands are routed" requirement. See the
"Framework-delta question" at the end for whether moving this INTO `resolve_target`
would be a framework change (it would, and it is NOT needed).

## 4. How descriptions currently re-localise on a language change (the seam to extend)

- The seam: `crates/ff-desktop/src/menu_workspace/i18n_keys.rs`.
  - `menu_option_desc_key(menu_name, command)` builds the key
    `menu-<menu-slug>-opt-<command-slug>-desc`.
  - `localised_option_description(menu_name, option)` = `t_or(key, &option.description)`:
    the catalogue value when present, else the parsed English `description`
    (fallback, so a menu always renders).
  - `t_or(key, fallback)` calls `ff_i18n::t(key)`; the seam returns the key
    itself on a miss, which is the fallback sentinel.
- The command column uses the sibling helper in the SAME file:
  `localised_option_command(option, display_surface_source)` returns the alias
  surface when present (non-blank), else the raw `option.command` (ROOT).
  The surface source is computed shell-side in `menu_option_command_surfaces`
  (`crates/ff-desktop/src/shell/locale_resolver.rs`) via
  `display_surface_for_canonical`.
- On a language change, `on_locale_reload` (`main.rs`) sets the new locale on the
  i18n seam AND re-seeds/overlays the alias store, so BOTH the catalogue-keyed
  descriptions and the alias-surface command column recompute on the next frame.

To extend localisation to the MNEMONIC, this is the seam to mirror: either a new
catalogue key (`menu-<menu>-opt-<command>-key` style), or -- more consistent with
the owner's "command carries the localised alias" framing -- derive the mnemonic
from the localised alias surface (e.g. first character of the localised command).
The design step must choose; this file only records the seam.

## 5. Workspace Kinds / menu-Kind chrome (where a menu Kind's chrome comes from)

- The POM/Settings/menu Contexts are instances of the Menu_Workspace pattern
  backed by a named TOML file; menus are CONTENT, not mechanism.
- Kind chrome (title / menu-bar / key-list / profile) comes from the Workspace
  Kinds registry: `docs/specs/workspace-kinds/requirements.md` Req 3 (tab title
  and Title_Line derived from the Kind config), Req 4 (per-Kind menu bar and key
  list). The nav/descriptor machinery lives in
  `crates/ff-desktop/src/shell/nav_stack.rs` (navigate_to, descriptor_for_current_context,
  reconstruct_custom) and the `WorkspaceContext` framework in
  `crates/ff-desktop/src/shell/workspace_context.rs` (returns `InteriorFocus`;
  dispatched via `render_workspace_context`).
- Relevance to this change: LOW. The menu-Kind title already localises via the
  Title_Line + the `menu-<menu>-title` catalogue key; the option content
  (key/command/description) is the Menu_Workspace concern in section 1-4. No
  Kinds-registry change is implied by localising the option mnemonic/command.

## 6. Change log and highest CR id

- Change log file: `docs/status/change-log.md` (new requirements and change
  requests; entries appended by the prompt-triage rule).
- Highest existing ids found:
  - NR series: `CR-NR-110` (FIND vs SEARCH verb split).
  - CH series: `CR-CH-065` (diacritic-insensitive command input).
- The NEXT new-capability id to allocate is therefore `CR-NR-111` (if logged as a
  NEW REQUIREMENT) or `CR-CH-066` (if logged as a CHANGE REQUEST). Note this work
  is a close continuation of the IN-PROGRESS `CR-NR-109`; the authoring step
  should decide whether to EXTEND CR-NR-109 (Requirement 11 already covers the
  alias-surface command rendering and the unified resolver) or open a new id for
  the mnemonic-localisation + saved-shape delta.

## 7. Existing spec docs -- relevant criteria and highest EARS numbers

Read to avoid contradiction. Highest top-level Requirement number per folder
(so new criteria can be numbered without collision):

- `docs/specs/localization/` -- HIGHEST = Requirement 11 (sub-criteria 11.1-11.13).
  Requirement 11 ("Locale-aware alias resolution and menu-kind options rendered
  via localised aliases", CR-NR-109) ALREADY specifies, in EARS:
  - 11.1: AliasTable entry carries an optional localisation key resolved via `t(key)`.
  - 11.3/11.4: auto-synthesised, editable, persisted identity rows (root always
    resolvable; a user may rename a root's surface + assign a key WITHOUT changing
    the root).
  - 11.5: on-disk per-locale per-environment alias reader
    (`i18n/<locale>/aliases/<env>.toml`).
  - 11.6: the unified resolver order, IDENTICAL for the command line and a
    selected menu-kind option (surface -> root, fallback English).
  - 11.7: "A menu-kind Context ... SHALL reference a command ALIAS as each option
    (not a raw command string). AT render THE option SHALL DISPLAY the
    active-locale surface ...; WHEN selected THE same verb SHALL be fed through
    the resolver of 11.6, reaching the root command."
  - 11.8: display fallback to Identity_Base/root; 11.9: root remains
    recorded/persisted; 11.10: collision -> retain Identity_Base, fall back.
  - 11.11-11.13 (CR-CH-065 amendment, pending): diacritic-insensitive input fold.
  This Requirement is the DIRECT governing criterion for the owner's request; the
  owner's "menu carries the localised alias of the command" is essentially 11.7.
  The DELTA not yet covered: localising the MNEMONIC (`key`), and the explicit
  saved-menu "command" field shape decision (see Crux).
- `docs/specs/menu-workspace/` -- HIGHEST = Requirement 20. Relevant: Req 1
  (Menu File Format: key/command/description), Req 10 (Menu options reference a
  Command Target; inline `[options.target]` precedence), Req 19 (one
  Option-Selection path; the Menu Workspace is a dumb dispatcher), Req 20
  (config-driven centered title; POM tab/command). The option model and the
  single-dispatch property are load-bearing here.
- `docs/specs/command-framework/` -- HIGHEST = Requirement 17. Relevant: Req 8
  (Unified Command Target + `resolve_target`), Req 14 (Selection-equals-command),
  Req 15 (typed line resolves through the single classifier), Req 17 (keyboard
  shortcuts through the single dispatch path). Any new criterion must NOT add a
  second dispatcher.
- `docs/specs/command-semantics/` -- HIGHEST = Requirement 12 (note numbering is
  non-contiguous: 1-12 with 7/8 out of order; 12 = FIND vs SEARCH split,
  CR-NR-110). Relevant: Req 1 (command pipeline parse/normalize/resolve), Req 3
  (case-insensitive command names).
- `docs/specs/workspace-kinds/` -- HIGHEST = Requirement 8. Relevant: Req 3 (tab
  title/Title_Line from Kind config), Req 4 (per-Kind menu bar/key list). Low
  direct relevance (menu option content is a Menu_Workspace concern).
- `docs/specs/menu-and-statusbar/` -- HIGHEST top-level number = Requirement 18
  (numbering non-contiguous: 1-13, 16-18, plus 11). Relevant: Req 2
  (Menu-Command integration -- every menu item invokes a registered command
  through the framework), Req 17 (Tab window chrome: Title_Line + Command_Line).

New EARS criteria for this change should be added under localization
Requirement 11 (e.g. 11.14+) or a new Requirement 12 in that folder, cross-
referencing menu-workspace Req 1/10/19 and command-framework Req 8/14/15.

---

## Crux: what a SAVED menu should store for the "command" field

The design must settle: for first-prize auto-conversion (a menu created in
English auto-displays in Afrikaans when the locale changes), should the saved
menu "command" field store (A) the ROOT command (locale-independent; derive the
displayed alias at render time), or (B) a LOCALISATION KEY (derive both displayed
alias and root at render/dispatch time)?

Evidence bearing on the choice:

- CURRENT saved shape is (A): the ROOT command string, English, locale-independent
  (section 1; `defaults.rs`; `MenuOption.command`). User menus author the root
  string; built-in menus are code-only English roots.
- The render path ALREADY derives the displayed alias from the root at render time
  via `display_surface_for_canonical(option.command)` (section 2/3). So option (A)
  ALREADY produces auto-conversion of the COMMAND COLUMN with NO change to the
  saved file -- provided the per-locale alias DATA exists for that root. This is
  exactly the owner's first-prize ("make a menu kind easily convertible to the
  localised language"): the SAME saved English menu renders localised when the
  locale has alias rows, because the root is stable and the surface is derived.
- The alias DATA is keyed by ROOT (canonical) -> localised surface + localisation
  key (`AliasTable`, `af/aliases/FFCMD.toml`). The root is the join key. This
  favours (A): the menu stores the root, the per-locale alias catalogue (authored
  once per locale, shared across every menu that references that root) supplies
  the surface. Option (B) would duplicate per-menu what the alias catalogue
  already holds per-locale, and would need a new field on `MenuOption`.
- Requirement 11.9 mandates the ROOT be the recorded/persisted identity; storing a
  localisation key as the persisted "command" (B) would contradict 11.9 unless the
  key still resolves to a stable root. Option (A) satisfies 11.9 directly.
- The resolver (section 3) already maps any localised surface back to the root for
  BOTH seams, so a menu that stored a localised surface would still dispatch
  correctly -- but it would then be locale-pinned (an `af` surface saved in the
  file renders wrong under `fr`), defeating auto-conversion. Storing the ROOT (A)
  keeps the saved menu locale-neutral and auto-converting.

Evidence summary: the current shape (A = ROOT) plus per-locale alias DATA already
delivers the owner's first-prize for the COMMAND column; the gap is DATA
(per-locale alias rows for the menu-option roots) plus the MNEMONIC.

### The mnemonic/character

- Current value: `MenuOption.key` (section 1), a 1-4 char uppercase string
  rendered RAW (`{:<4}` of `option.key`, `render_layout.rs:131`). It does NOT
  re-localise and there is NO per-locale source for it anywhere (no catalogue key,
  no alias field for the key). The alias table localises the COMMAND surface only.
- So to "localise the mnemonic" there is currently nothing to read from; the
  design must introduce a source. Two shapes consistent with the existing
  machinery: (i) derive the mnemonic from the localised command surface (e.g.
  first non-space char of the alias surface), or (ii) add a catalogue key for the
  key, mirroring the description seam (`menu-<menu>-opt-<command>-key`). Choosing
  between these is a design decision, deliberately NOT made here.

---

## Framework-delta question

Does `ff_command::resolve_target` already resolve a per-locale alias to its root
command, or is wiring that a genuine framework change the owner must confirm?

Finding, with evidence: `resolve_target` itself does NOT resolve a per-locale
alias -- its `TargetResolver` trait has no alias lookup
(`crates/ff-command/src/command_target.rs:207-305`) and the shell's
`ShellTargetResolver` does not touch the alias table
(`crates/ff-desktop/src/command_config/mod.rs`). HOWEVER, the shell ALREADY
performs the localised-alias -> root translation in a single pre-pass
(`rewrite_verb_to_root` -> `AliasTable::canonical_verb`) that runs BEFORE
`resolve_target` on BOTH command-string seams
(`dispatch_command_string`, `activate_menu_option`), so end to end a localised
alias DOES resolve to the root with routing unchanged (the ROOT is dispatched and
persisted). This is the EXISTING, wired behaviour (CR-NR-109, localization
Req 11.6), and it satisfies the owner's "should not affect how commands are
routed".

Therefore: NO framework change is required to deliver the owner's request. The
alias-to-root resolution is already wired at the correct seam, and the menu-kind
command column already renders the localised alias. Moving the alias lookup INSIDE
`resolve_target` (so the `ff-command` crate consults an alias table) WOULD be a
framework change (it reshapes the `TargetResolver` contract / adds an alias
dependency to the dispatch classifier, a core mechanism per
framework-conformance.md) and would need express owner confirmation -- but it is
NOT necessary and should be avoided; the pre-pass seam is the established design.

The actual remaining work is NON-framework: (1) author per-locale alias DATA rows
for the built-in menu-option roots (Settings/Catalogs/Files/Help/X and
Config/Theme/Menus/Keys/Kinds/Reset bare) plus matching catalogue entries, which
is a gated DATA task; and (2) the DELTA the owner newly asks for -- localise the
MNEMONIC and formally settle the saved "command" field shape (evidence favours
keeping the ROOT, option A) -- which is a gated requirements/design change under
localization Requirement 11, building ON the existing alias-surface + unified-
resolver framework (no new dispatcher, no new persistence format).
