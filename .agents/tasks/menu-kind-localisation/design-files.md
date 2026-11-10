# Design Files -- Menu-Kind Localisation (CR-NR-111)

Handoff note naming every design.md (and the requirements.md this design step
adjusted) the reviewer and the tasks step should read. Produced so the design
file set need not be recovered from git (design-review finding 5).

## Governing design

- `docs/specs/localization/design.md`
  - Section: "Menu Kinds localise FULLY -- mnemonic + command alias + description
    (CR-NR-111, Requirement 12)". This is the GOVERNING design for the owner's
    request. It settles the storage-shape crux (store the ROOT, Option A),
    rejects storing a localisation key (Option B), states the alias->root
    resolution is ALREADY wired via the `rewrite_verb_to_root` ->
    `resolve_target` pre-pass (NOT a framework change; moving it INSIDE
    `resolve_target` WOULD be and is rejected), specifies the mnemonic seam
    (catalogue key, Option ii), first prize, second prize (load/reconstruct-time
    by name + locale; built-in stays code-only), per-part fallback, and
    migration/back-compat (none needed).
  - Section: "Review responses -- CR-NR-111 design review (iteration 2)" records
    how each review finding was resolved.

## Render-model and chrome deltas

- `docs/specs/menu-workspace/design.md`
  - Section: "Design Delta: Full menu-option localisation -- mnemonic + command
    alias + description (CR-NR-111)". Owns the `MenuOption` model (unchanged
    shape), the shared renderer, `option_prefix_job`, the mnemonic surface
    derivation, the second-prize load/reconstruct-time resolution + persistence
    contract, the built-in code-only clause, the realised mnemonic contract, and
    the DISPLAY-vs-MATCH acceptance-criterion cross-reference.
- `docs/specs/menu-and-statusbar/design.md`
  - Section: "Design Delta: Menu row chrome localises fully -- mnemonic, command,
    description (Requirement 20, CR-NR-111)". Owns only the guarantee that
    full-row localisation adds/changes NO dispatch path (command-framework Req 2
    preserved).

## Requirements touched by this design step

- `docs/specs/menu-workspace/requirements.md` -- added Req 21.8 (displayed
  mnemonic is display-only; stored `key` remains the selection token), per review
  finding 4.

## Requirements authored by the prior requirements step (read for alignment)

- `docs/specs/localization/requirements.md` -- Requirement 12 (governing EARS).
- `docs/specs/menu-workspace/requirements.md` -- Requirement 21.
- `docs/specs/menu-and-statusbar/requirements.md` -- Requirement 20.
- `docs/specs/command-framework/requirements.md`,
  `docs/specs/command-semantics/requirements.md`,
  `docs/specs/workspace-kinds/requirements.md` -- cross-referenced; no design
  delta needed (the alias->root resolution path is already wired, per findings).

## Storage-shape decision (one line)

Store the ROOT command (locale-independent, Option A); displayed alias/mnemonic/
description are DERIVED at render time from the per-locale DATA. No framework
delta flagged -- the alias->root resolution is already wired via the pre-pass
before `resolve_target`; moving it inside `resolve_target` would be a framework
change and is explicitly rejected.
