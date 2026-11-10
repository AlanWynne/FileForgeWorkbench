# Gate Summary -- Root-Command Help Model (CR-NR-119)

REQUIREMENTS + DESIGN GATE, docs only. No `.rs` changed, no cargo gate run. All
docs ASCII-only, EARS criteria. Worktree: `.worktrees/command-help-gate`.

## The model (owner's six points, as specified)

Every home-grown root command is defined with a structured Markdown help file that
is the SINGLE SOURCE OF TRUTH for its one-line description AND its full help,
reachable by both `HELP <cmd>` (ISPF) and `<cmd> --help` (POSIX).

## New / changed criteria (by spec home)

### context-help/requirements.md (the model home)
- **Requirement 21** -- Command Help File: required structure + deterministic one-line
  description. Criteria 21.1-21.7.
- **Requirement 22** -- Derived short-description lookup + `HELP <cmd>` routing for FFCMD
  roots. Criteria 22.1-22.8 (22.8 added in revision: enforced-root `cmd:<ROOT>`
  resolution takes precedence over the `KEYS`/`CONFIG` `feature:` special-cases).
- **Requirement 23** -- Enforced help-file + one-liner rule for FFCMD roots; guideline for
  extensions. Criteria 23.1-23.6.

### command-semantics/requirements.md
- **Requirement 15** -- POSIX `--help` global dispatch convention. Criteria 15.1-15.9
  (15.9 added in revision: `--help` applies to single-token verbs; two-word roots
  `FILE CATALOGS` / `RESET BARE` deferred, reachable via `HELP <ROOT>`).

### command-configurator/requirements.md
- **Requirement 9** -- Derived short-description for aliases + inheritance. Criteria
  9.1-9.5.

### command-framework/requirements.md
- **Requirement 3** criteria **8** and **9** -- the tiered description rule (derived
  one-liner for FFCMD roots; guideline, not enforced, for extensions).

## Structure options + recommendation (the central design decision)

| Option | What | Verdict |
|--------|------|---------|
| (a) | YAML frontmatter with an explicit `description:` field | Rejected -- new parse path, drifts from prose, changes every file |
| (b) | Fixed first-line / first-paragraph-after-`# <CMD>`-heading convention | Rejected -- shipped topics lead with `## Syntax`/`## Description`, not an `# <CMD>` heading; fragile |
| **(c)** | **Formalise the EXISTING `cmd:<NAME>` topic structure; extract the first paragraph of `## Description`** | **RECOMMENDED -- reuses shipped format, no new markup, one description, matches Req 7.2** |

Extraction rule (Req 21.3): first non-empty paragraph of `## Description`, inline
markup stripped to plain text, whitespace collapsed; validated non-empty and <= 200
chars (Req 21.4). Locale-aware via the per-locale registry (Req 21.6).

## `--help` precedence decision

- The help flag SHORT-CIRCUITS before the command runs (Req 15.2): side effects do
  NOT execute when a help flag is present.
- The check is a prelude-level branch AFTER `rewrite_verb_to_root` (so aliases are
  canonicalised: `SS --help` -> `cmd:SNAPSHOT`, Req 15.3) and BEFORE
  `resolve_target`/the ladder, routing through an enforced-root-aware help open
  that reuses the EXISTING `open_help` seam (Req 15.6, 22.8).
- Recognised spellings: `--help`, `-h`, `/?`, as the FIRST argument token only
  (Req 15.1, 15.7). Not recorded in history/undo (Req 15.8).
- SCOPE (Req 15.9): single-token verbs only. The two-word roots `FILE CATALOGS`
  and `RESET BARE` cannot be served by a first-arg-token check (their first arg
  token is not a help flag) so their POSIX `--help` is DEFERRED; both remain
  reachable via `HELP FILE CATALOGS` / `HELP RESET BARE`.

## KEYS / CONFIG routing (revision)

`resolve_help_argument` special-cases `KEYS` -> `feature:function_keys` and
`CONFIG`/`CONFIGURATION` -> `feature:configuration` before the `cmd:<NAME>`
fall-through, and both are enforced FFCMD roots. Decision (Req 22.8): for an
enforced root the HELP/`--help` reach-path resolves `cmd:<ROOT>` DIRECTLY (takes
precedence over the keyword table), so `HELP KEYS`/`KEYS --help` open `cmd:KEYS`
and `HELP CONFIG`/`CONFIG --help` open `cmd:CONFIG`. The `feature:*` topics stay
reachable via the Help Index and menu links; only the root-verb resolution is
repointed. `cmd:KEYS` already ships (needs only the repoint); `cmd:CONFIG` is new.

## Enforcement mechanism

- An `ff-desktop` test enumerates `EnvironmentRegistry::root_commands("FFCMD")`
  (= `builtin_ffcmd_roots()`), loads the shipped `help/` tree, and asserts each root
  has a `cmd:<ROOT>` topic with a valid one-liner (Req 23.2/23.3). Authoring a new
  FFCMD root without its help file breaks the suite.
- Lives in a TEST, not a registration gate. FFEDIT excluded now (Req 23.4);
  extensions never asserted (Req 23.5).

## FFCMD root enumeration + retrofit list

Enforced set = `builtin_ffcmd_roots()`:
- Ladder roots: EDIT, VIEW, BROWSE, COMPARE, CREATE, REPLACE, EXIT, QUIT, LOGOFF,
  START, MENU, CLOSE, DOCK, END, RETURN, DETACH, SPLIT, UNSPLIT, FOCUS, SWAP,
  WORKSPACE, PFSHOW, COMMAND, SNAPSHOT, CAPTURE, RETRIEVE, SUBMIT, TIME, STATUS,
  SAVE, CANCEL, REFRESH.
- Workspace-verb roots: FILE CATALOGS, CONFIG, KEYS, THEME, FILES, CATALOGS,
  COMMANDS, LOG, PLUGINS, MACROS, GSEARCH, SEARCH, KINDS, MENUS, ALIASES.
- Plus built-in menu option roots: HELP, X, RESET BARE, SETTINGS.

Topics EXIST today for: SAVE, CANCEL, END, UNDO, REDO, HELP, KEYS, POM, SETTINGS,
FILES, and the navigation verbs (UP/DOWN/TOP/BOTTOM/LOCATE). RETROFIT (author
`cmd:<ROOT>` with a non-empty `## Description`) for the MISSING roots, notably:
EXIT, SNAPSHOT, CAPTURE (START/STOP/REPLAY), RETRIEVE, START, MENU, CLOSE, DOCK,
RETURN, DETACH, SPLIT, UNSPLIT, FOCUS, SWAP, WORKSPACE, PFSHOW, COMMAND, SUBMIT,
TIME, STATUS, REFRESH, EDIT, VIEW, BROWSE, COMPARE, CREATE, REPLACE, CONFIG, THEME,
CATALOGS, COMMANDS, LOG, PLUGINS, MACROS, GSEARCH, SEARCH, KINDS, MENUS, ALIASES,
FILE CATALOGS, RESET BARE.

## Tiered-enforcement statement

- MANDATORY + ENFORCED now: FFCMD root commands (this gate).
- DEFERRED: FFEDIT roots conform later (on the roadmap, not enforced now).
- STRONG GUIDELINE, NOT enforced: EXTENSIONS -- encouraged to ship a conforming
  help file; the framework MUST NOT reject/block an extension command lacking one;
  degrades gracefully (no derived description; `HELP`/`--help` show "No help
  available"). Recorded in `.kiro/steering/command-help-guideline.md`.

## Owner-decision points (each with a recommendation)

1. Help-file structure: **Option (c)** -- formalise the existing `cmd:<NAME>` topic
   and extract the `## Description` first paragraph. (RECOMMENDED)
2. `--help` precedence: **short-circuit BEFORE the command runs.** (RECOMMENDED)
3. Help-flag spellings: **`--help` + `-h` + `/?`**, first argument token only.
   (RECOMMENDED)
4. Alias description inheritance: **at DISPLAY time**, not written into the
   persisted seed row (decouples from help-registry readiness at construction).
   (RECOMMENDED)
5. Two-word roots (FILE CATALOGS, RESET BARE): key as `cmd:FILE CATALOGS` /
   `cmd:RESET BARE` (confirm `TopicKey` accepts a space, or normalise to the
   resolved root). HELP reach-path covered now; their POSIX `--help` is DEFERRED
   as a known limitation (Req 15.9). (RECOMMENDED -- confirm `TopicKey` space in
   implementation)
6. KEYS / CONFIG root resolution: enforced-root `cmd:<ROOT>` resolution takes
   PRECEDENCE over the `resolve_help_argument` `feature:` special-cases, so
   `HELP KEYS`/`KEYS --help` open `cmd:KEYS` and `HELP CONFIG`/`CONFIG --help`
   open `cmd:CONFIG`; `feature:*` topics stay reachable via the Index/menus
   (Req 22.8). (RECOMMENDED)

## Framework verdict

NO framework change. Builds ON the single dispatch front door
(`dispatch_command_string`: `=` strip -> `rewrite_verb_to_root` ->
`run_command_prelude` -> `resolve_target` -> `run_command_ladder`) and the
context-help subsystem. The `--help` convention is a prelude-level short-circuit
reusing the EXISTING `open_help` seam -- NO new dispatcher, NO new navigation
stack, NO new persistence format (framework-conformance mechanisms 1 and 5). The
help-file structure reuses the EXISTING `ff-help` topic format + registry;
enforcement lives in a test; extensions are explicitly not blocked.

## Deliverables

- Spec requirements: context-help (Req 21-23), command-semantics (Req 15),
  command-configurator (Req 9), command-framework (Req 3.8/3.9).
- Design: `docs/specs/context-help/design.md` (section "CR-NR-119 -- Root-Command
  Help Model (design)").
- Tasks: `docs/specs/context-help/tasks.md` (tasks 24-29 + coverage mapping).
- Steering guideline: `.kiro/steering/command-help-guideline.md`.
- Change log: `docs/status/change-log.md` -- CR-NR-119.

## Review responses (revision 1 -- review.json verdict CHANGES_REQUESTED)

- **Finding 1 (MEDIUM, KEYS/CONFIG diverted to `feature:` topics): ADDRESSED.**
  Deleted the false "no routing change is needed" claim in design.md; documented
  that `resolve_help_argument` diverts `KEYS` and `CONFIG` (both enforced roots)
  to `feature:` topics and CHOSE recommended option (a) -- enforced-root
  `cmd:<ROOT>` resolution takes precedence. Added context-help Req 22.8, updated
  the design `--help` pseudocode to an enforced-root-aware `open_root_help`, added
  tasks 25.4 and the 26.2 note, updated the coverage mapping.
- **Finding 2 (MEDIUM, `--help` cannot serve two-word roots): ADDRESSED.**
  Added a "Known limitation -- two-word roots" subsection to design.md and chose
  recommended option (a) -- `--help` applies to single-token verbs this version;
  `FILE CATALOGS` / `RESET BARE` POSIX `--help` deferred, both still reachable via
  `HELP <ROOT>`. Added command-semantics Req 15.9, updated the Req 15 design-note,
  tasks 25.2/25.5, and the coverage mapping.

GATE_READY_FOR_APPROVAL
