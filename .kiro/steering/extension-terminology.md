# Extension Terminology -- "plugin" is being renamed to "extension"

Owner decision (standing): the user-facing and code concept formerly called
"plugin" is renamed to **extension**. The administrator/management UI is the
**Extension Manager**. The end state is that NOTHING in the codebase, specs, or
UI is named "plugin" -- zero occurrences.

## The rollout policy -- gradual, in place, no per-rename confirmation

This is a SLOW, incremental migration, NOT a big-bang find-replace. The rule for
every agent and every change:

- When you TOUCH a piece of code, a spec, a doc, a menu option, a crate, a
  module, a type, a test, or a UI string that is named "plugin" (any case:
  `plugin`, `Plugin`, `PLUGINS`, `plugin_manager`, `ff-plugin`, etc.) FOR ANOTHER
  REASON, rename it to the "extension" equivalent as part of that same change.
  No separate approval is needed -- the owner has pre-approved the rename
  direction.
- Do NOT launch a dedicated sweeping rename pass or rename files/crates/specs you
  are not otherwise working on, UNLESS the owner explicitly asks for a rename
  pass. The migration rides along with real work so it stays low-risk and
  reviewable.
- Prefer the compiler-assisted rename (semantic rename / smart file relocate)
  when renaming a symbol or file so references update consistently; verify the
  scoped build after.

## Naming map (apply when renaming)

| Old (plugin) | New (extension) |
|--------------|-----------------|
| plugin | extension |
| Plugin | Extension |
| PLUGINS (POM option / verb) | EXTENSIONS |
| Plugin Manager / "Plugin Administrator" | Extension Manager |
| plugin_manager (module) | extension_manager |
| ff-plugin (crate) | ff-extension |
| ff-mdx-plugin (crate) | ff-mdx-extension |
| plugin-architecture (spec folder) | extension-architecture |
| plugin-manager-ui (spec folder) | extension-manager-ui |

(Extend this map as new "plugin"-named items are discovered; keep it the single
reference for the canonical new name of each item.)

## Constraints

- ASCII-only per `documentation-charset.md`.
- A rename that changes a crate name, a public type, a spec folder, or a
  persisted/serialized identifier is a bigger step: do it when you are already
  working in that area, land it behaviour-preservingly, and record it in the
  change log. If it would ripple widely beyond the current change, note it as a
  follow-up rather than expanding the current change's blast radius.
- New code/specs authored from now on use "extension" from the start -- never
  introduce a new "plugin"-named item.
