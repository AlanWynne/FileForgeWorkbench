# Root-command help model -- scope decision (owner 2026-11-05, Option A)

Front-loaded BEFORE the alias seed-set descriptions: every (home-grown) root
command is defined with a structured markdown help file that is the SINGLE
SOURCE OF TRUTH for its one-line description and full help.

## The model (owner's six points)
1. Every home-grown root command has a markdown help file -- mandatory part of
   creating the command.
2. The help file follows a REQUIRED STRUCTURE so the one-line description is
   extractable (central design decision: frontmatter field vs fixed
   first-line/heading convention -- the gate proposes options; likely formalises
   the existing context-help cmd:<NAME> topic structure rather than inventing).
3. HELP <cmd> (ISPF) opens the command's markdown help.
4. <cmd> --help (POSIX) opens the SAME file -- a global dispatch convention on
   the single front door.
5. The short description (alias editor, command palette, status lines) is DERIVED
   from the help file -- one source, no duplicated description strings.
6. RULE: no home-grown root command may exist without (a) its help file and (b) a
   one-line description within it.

## Tiered enforcement (owner decision)
- MANDATORY + ENFORCED: FFWB's own home-grown commands.
  - THIS GATE's enforced scope = the FFCMD root commands (EXIT, SNAPSHOT,
    CAPTURE START/STOP/REPLAY, RETRIEVE, ... the BUILTIN_FFCMD roots).
  - FFEDIT conforms LATER (made to comply when we reach the FFEDIT work).
- STRONG GUIDELINE (recommended, NOT enforced): EXTENSIONS. We encourage
  extensions to follow the help-file rule, but do NOT make it a hard adoption
  barrier -- a large third-party extension with a very different architecture
  must remain adoptable without retrofitting the FFWB help-file structure. The
  framework must NOT reject or block an extension command that lacks a conforming
  help file; it degrades gracefully (e.g. no derived description / a fallback).

## Why tiered
Strictness where we own the code keeps our commands self-documenting with one
source of truth; leniency for extensions protects adoptability of big external
capabilities whose architecture we do not control.

## Feeds
- The alias seed-set (X/SS/CS/CT/CR) descriptions INHERIT from this model once
  the FFCMD roots carry help files + one-line descriptions (so the seed aliases
  are authored ONCE, inheriting, not with throwaway literal strings).
- Retrofit: author help files for the existing FFCMD roots; reconcile with the
  partial context-help cmd:<NAME> topics that already exist.

## Process
Docs-only DESIGN GATE now (requirements + design + tasks), owner review, then
implementation. Likely spec homes: command-semantics / command-framework (the
rule + --help dispatch), context-help (the help-file structure + HELP <cmd>
routing), command-configurator (derived short description for the alias/command
UIs). New steering guideline for the extension recommendation.
