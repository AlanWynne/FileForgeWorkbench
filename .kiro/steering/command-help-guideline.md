# Command Help Model -- Enforced for FFCMD Roots, a Guideline for Extensions (CR-NR-119)

Owner decision (standing, messages 6-8 and 12, 2026-11-05): every home-grown root
command is defined with a structured Markdown help file that is the SINGLE SOURCE
OF TRUTH for its one-line description AND its full help, reachable by both
`HELP <cmd>` (ISPF) and `<cmd> --help` (POSIX). The one-line description is
DERIVED from that help file (the first paragraph of its `## Description` section),
so no description string is duplicated across code and docs.

This rule is TIERED. Where FFWB owns the code it is ENFORCED; for extensions it is
a STRONG GUIDELINE, never a hard adoption barrier.

## Enforced (FFWB's own commands)

- Scope of enforcement for the current phase = the FFCMD ROOT COMMANDS
  (`builtin_ffcmd_roots()` / `EnvironmentRegistry::root_commands("FFCMD")`): every
  such root MUST have a `cmd:<ROOT>` Markdown help topic under `help/` with a
  non-empty `## Description` whose first paragraph yields a valid one-line
  description (context-help Requirement 21, 23).
- An automated `ff-desktop` test asserts this for every FFCMD root; authoring a new
  FFCMD root without its help file breaks the suite. See context-help
  Requirement 23.2/23.3.
- FFEDIT roots are DEFERRED: they conform later (owner message 12), on the roadmap,
  NOT enforced in this phase (context-help Requirement 23.4).
- `HELP <cmd>` and `<cmd> --help` (and `-h`, `/?`) both resolve to the SAME
  `cmd:<ROOT>` topic through the single dispatch front door and the one `open_help`
  seam -- no new dispatcher (command-semantics Requirement 15; framework-conformance
  mechanism 1).

## Guideline (extensions -- encouraged, NOT enforced)

- Extension commands are ENCOURAGED to ship a conforming `cmd:<ROOT>` help file so
  they gain the same `HELP <cmd>` / `--help` reach and a derived short description
  in the alias editor and command palette.
- The framework MUST NOT reject, block, or fail to register an extension command
  that lacks a conforming help file. A large third-party extension with a very
  different architecture must remain adoptable WITHOUT retrofitting the FFWB
  help-file structure (owner message 12).
- A missing extension help file DEGRADES GRACEFULLY: the derived short-description
  lookup returns `None` (the consumer shows an empty description), and
  `HELP <extension-cmd>` / `<extension-cmd> --help` show the existing "No help
  available for: <cmd>" affordance (context-help Requirement 22.6, 23.5).
- Do NOT add any registration-time validation of extension help. The enforcement
  test's SCOPE is FFCMD roots only; it must never assert anything about extension
  commands.

## Why tiered

Strictness where we own the code keeps our commands self-documenting with one
source of truth; leniency for extensions protects the adoptability of big external
capabilities whose architecture we do not control.

## Where the model is specified

- context-help Requirement 21 (help-file structure + one-liner extraction),
  Requirement 22 (short-description lookup + `HELP <cmd>` routing), Requirement 23
  (tiered enforcement).
- command-semantics Requirement 15 (the `--help` / `-h` / `/?` global convention).
- command-configurator Requirement 9 (derived alias description + inheritance).
- command-framework Requirement 3.8/3.9 (the tiered description rule).
- Design: `docs/specs/context-help/design.md`, section "CR-NR-119 -- Root-Command
  Help Model (design)".
